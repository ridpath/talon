#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryBinding {
    pub name: String,
    pub address: u64,
    pub size: usize,
    pub mem_type: MemoryType,
    pub current_value: Vec<u8>,
    pub is_reactive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryType {
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Int8,
    Int16,
    Int32,
    Int64,
    Float32,
    Float64,
    String,
    Bytes,
}

impl MemoryType {
    pub fn size(&self) -> usize {
        match self {
            MemoryType::UInt8 | MemoryType::Int8 => 1,
            MemoryType::UInt16 | MemoryType::Int16 => 2,
            MemoryType::UInt32 | MemoryType::Int32 | MemoryType::Float32 => 4,
            MemoryType::UInt64 | MemoryType::Int64 | MemoryType::Float64 => 8,
            MemoryType::String | MemoryType::Bytes => 0,
        }
    }
}

pub struct ReactiveMemoryManager {
    bindings: Arc<RwLock<HashMap<String, MemoryBinding>>>,
    watches: Arc<RwLock<HashMap<u64, Vec<String>>>>,
}

impl ReactiveMemoryManager {
    pub fn new() -> Self {
        ReactiveMemoryManager {
            bindings: Arc::new(RwLock::new(HashMap::new())),
            watches: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn bind_memory(&self, name: &str, address: u64, mem_type: &str) -> Result<(), String> {
        let parsed_type = match mem_type {
            "uint8" => MemoryType::UInt8,
            "uint16" => MemoryType::UInt16,
            "uint32" => MemoryType::UInt32,
            "uint64" => MemoryType::UInt64,
            "int8" => MemoryType::Int8,
            "int16" => MemoryType::Int16,
            "int32" => MemoryType::Int32,
            "int64" => MemoryType::Int64,
            "float32" => MemoryType::Float32,
            "float64" => MemoryType::Float64,
            "string" => MemoryType::String,
            "bytes" => MemoryType::Bytes,
            _ => return Err(format!("Unknown memory type: {}", mem_type))
        };

        let size = if parsed_type.size() == 0 { 256 } else { parsed_type.size() };
        let current_value = self.read_memory(address, size).await?;

        let binding = MemoryBinding {
            name: name.to_string(),
            address,
            size,
            mem_type: parsed_type,
            current_value,
            is_reactive: true,
        };

        self.bindings.write().await.insert(name.to_string(), binding);
        Ok(())
    }

    pub async fn unbind_memory(&self, name: &str) -> Result<(), String> {
        self.bindings.write().await.remove(name)
            .ok_or_else(|| format!("No binding found for: {}", name))?;
        Ok(())
    }

    pub async fn read_binding(&self, name: &str) -> Result<Vec<u8>, String> {
        let bindings = self.bindings.read().await;
        let binding = bindings.get(name)
            .ok_or_else(|| format!("No binding found for: {}", name))?;
        
        if binding.is_reactive {
            self.read_memory(binding.address, binding.size).await
        } else {
            Ok(binding.current_value.clone())
        }
    }

    pub async fn write_binding(&self, name: &str, value: &[u8]) -> Result<(), String> {
        let bindings = self.bindings.read().await;
        let binding = bindings.get(name)
            .ok_or_else(|| format!("No binding found for: {}", name))?;
        
        self.write_memory(binding.address, value).await?;
        drop(bindings);
        
        self.bindings.write().await.get_mut(name).unwrap().current_value = value.to_vec();
        Ok(())
    }

    async fn read_memory(&self, _address: u64, size: usize) -> Result<Vec<u8>, String> {
        #[cfg(target_os = "linux")]
        {
            use std::fs::File;
            use std::io::{Read, Seek, SeekFrom};
            
            let mut file = File::open("/proc/self/mem")
                .map_err(|e| format!("Failed to open memory: {}", e))?;
            file.seek(SeekFrom::Start(address))
                .map_err(|e| format!("Failed to seek memory: {}", e))?;
            
            let mut buffer = vec![0u8; size];
            file.read_exact(&mut buffer)
                .map_err(|e| format!("Failed to read memory: {}", e))?;
            
            Ok(buffer)
        }
        
        #[cfg(not(target_os = "linux"))]
        {
            Ok(vec![0u8; size])
        }
    }

    async fn write_memory(&self, _address: u64, _value: &[u8]) -> Result<(), String> {
        #[cfg(target_os = "linux")]
        {
            use std::fs::OpenOptions;
            use std::io::{Write, Seek, SeekFrom};
            
            let mut file = OpenOptions::new()
                .write(true)
                .open("/proc/self/mem")
                .map_err(|e| format!("Failed to open memory for writing: {}", e))?;
            
            file.seek(SeekFrom::Start(address))
                .map_err(|e| format!("Failed to seek memory: {}", e))?;
            
            file.write_all(value)
                .map_err(|e| format!("Failed to write memory: {}", e))?;
            
            Ok(())
        }
        
        #[cfg(not(target_os = "linux"))]
        {
            Ok(())
        }
    }

    pub async fn watch_memory(&self, address: u64, _size: usize, callback: String) -> Result<(), String> {
        let mut watches = self.watches.write().await;
        watches.entry(address).or_insert_with(Vec::new).push(callback);
        Ok(())
    }

    pub async fn poll_changes(&self) -> Result<Vec<(String, Vec<u8>)>, String> {
        let mut changes = Vec::new();
        let bindings = self.bindings.read().await;
        
        for (name, binding) in bindings.iter() {
            if binding.is_reactive {
                let new_value = self.read_memory(binding.address, binding.size).await?;
                if new_value != binding.current_value {
                    changes.push((name.clone(), new_value));
                }
            }
        }
        
        Ok(changes)
    }
}
