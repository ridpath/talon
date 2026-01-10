#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use crate::memory_tools;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolicLink {
    pub name: String,
    pub target_expression: String,
    pub address: u64,
    pub link_type: SymlinkType,
    pub is_active: bool,
    pub cached_value: Vec<u8>,
    pub target_pid: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SymlinkType {
    Memory,
    Register,
    SegmentOffset,
    Symbol,
    KernelObject,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentReference {
    pub segment: String,
    pub offset: u64,
}

pub struct SysbioticExecutor {
    symlinks: Arc<RwLock<HashMap<String, SymbolicLink>>>,
    segment_cache: Arc<RwLock<HashMap<String, u64>>>,
    target_pid: Option<u32>,
}

impl SysbioticExecutor {
    pub fn new() -> Self {
        SysbioticExecutor {
            symlinks: Arc::new(RwLock::new(HashMap::new())),
            segment_cache: Arc::new(RwLock::new(HashMap::new())),
            target_pid: None,
        }
    }

    pub fn set_target_pid(&mut self, pid: u32) {
        self.target_pid = Some(pid);
        log::info!("Symbiotic executor targeting PID: {}", pid);
    }

    pub async fn create_symlink(
        &self,
        name: &str,
        target_expr: &str,
        link_type: &str,
    ) -> Result<(), String> {
        let parsed_type = match link_type {
            "memory" => SymlinkType::Memory,
            "register" => SymlinkType::Register,
            "segment_offset" => SymlinkType::SegmentOffset,
            "symbol" => SymlinkType::Symbol,
            "kernel_object" => SymlinkType::KernelObject,
            _ => return Err(format!("Unknown symlink type: {}", link_type)),
        };

        let address = self.resolve_target_expression(target_expr).await?;

        let symlink = SymbolicLink {
            name: name.to_string(),
            target_expression: target_expr.to_string(),
            address,
            link_type: parsed_type.clone(),
            is_active: true,
            cached_value: Vec::new(),
            target_pid: self.target_pid,
        };

        log::info!("Created symlink '{}' -> 0x{:x} (type: {:?}, pid: {:?})", 
                  name, address, parsed_type, self.target_pid);
        self.symlinks.write().await.insert(name.to_string(), symlink);
        Ok(())
    }

    async fn resolve_target_expression(&self, expr: &str) -> Result<u64, String> {
        if expr.contains(':') {
            let parts: Vec<&str> = expr.split(':').collect();
            if parts.len() == 2 {
                let segment = parts[0].trim_matches(|c| c == '$' || c == '[' || c == ']');
                let offset_str = parts[1].trim_matches(|c| c == '[' || c == ']');
                let offset = self.parse_hex_or_dec(offset_str)?;
                
                let segment_base = self.resolve_segment(segment).await?;
                return Ok(segment_base + offset);
            }
        }

        if expr.starts_with('@') {
            return self.resolve_symbol(expr).await;
        }

        if expr.starts_with("0x") {
            return self.parse_hex_or_dec(expr);
        }

        Err(format!("Cannot resolve expression: {}", expr))
    }

    async fn resolve_segment(&self, segment: &str) -> Result<u64, String> {
        let cache = self.segment_cache.read().await;
        if let Some(&base) = cache.get(segment) {
            return Ok(base);
        }
        drop(cache);

        let base_addr = match segment {
            "gs" => self.read_gs_base().await?,
            "fs" => self.read_fs_base().await?,
            "es" | "cs" | "ss" | "ds" => 0,
            _ => return Err(format!("Unknown segment: {}", segment)),
        };

        self.segment_cache.write().await.insert(segment.to_string(), base_addr);
        Ok(base_addr)
    }

    async fn read_gs_base(&self) -> Result<u64, String> {
        #[cfg(target_os = "linux")]
        {
            use std::arch::asm;
            let mut base: u64;
            unsafe {
                asm!(
                    "mov {}, gs",
                    out(reg) base,
                    options(nostack, preserves_flags)
                );
            }
            Ok(base)
        }

        #[cfg(target_os = "windows")]
        {
            Ok(0x0)
        }

        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        {
            Ok(0x0)
        }
    }

    async fn read_fs_base(&self) -> Result<u64, String> {
        #[cfg(target_os = "linux")]
        {
            use std::arch::asm;
            let mut base: u64;
            unsafe {
                asm!(
                    "mov {}, fs",
                    out(reg) base,
                    options(nostack, preserves_flags)
                );
            }
            Ok(base)
        }

        #[cfg(target_os = "windows")]
        {
            Ok(0x0)
        }

        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        {
            Ok(0x0)
        }
    }

    async fn resolve_symbol(&self, symbol_expr: &str) -> Result<u64, String> {
        let parts: Vec<&str> = symbol_expr.trim_start_matches('@').split('!').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid symbol format: {}. Expected @lib!symbol", symbol_expr));
        }

        let _library = parts[0];
        let _symbol = parts[1];

        Ok(0x0)
    }

    fn parse_hex_or_dec(&self, s: &str) -> Result<u64, String> {
        if s.starts_with("0x") || s.starts_with("0X") {
            u64::from_str_radix(&s[2..], 16)
                .map_err(|e| format!("Failed to parse hex: {}", e))
        } else {
            s.parse::<u64>()
                .map_err(|e| format!("Failed to parse number: {}", e))
        }
    }

    pub async fn read_symlink(&self, name: &str) -> Result<Vec<u8>, String> {
        let symlinks = self.symlinks.read().await;
        let symlink = symlinks.get(name)
            .ok_or_else(|| format!("Symlink not found: {}", name))?;

        let pid = symlink.target_pid.or(self.target_pid)
            .ok_or_else(|| "No target PID set for symlink".to_string())?;
        
        let data = memory_tools::read_process_memory(pid, symlink.address as usize, 8)?;
        log::debug!("Read {} bytes from PID {} at 0x{:x}", data.len(), pid, symlink.address);
        Ok(data)
    }

    pub async fn write_symlink(&self, name: &str, value: &[u8]) -> Result<(), String> {
        let symlinks = self.symlinks.read().await;
        let symlink = symlinks.get(name)
            .ok_or_else(|| format!("Symlink not found: {}", name))?;

        let pid = symlink.target_pid.or(self.target_pid)
            .ok_or_else(|| "No target PID set for symlink".to_string())?;
        let address = symlink.address;
        
        drop(symlinks);
        
        memory_tools::write_process_memory(pid, address as usize, value)?;

        self.symlinks.write().await.get_mut(name).unwrap().cached_value = value.to_vec();
        log::info!("Wrote {} bytes to PID {} at 0x{:x} via symlink '{}'", 
                  value.len(), pid, address, name);
        Ok(())
    }

    pub async fn remove_symlink(&self, name: &str) -> Result<(), String> {
        self.symlinks.write().await.remove(name)
            .ok_or_else(|| format!("Symlink not found: {}", name))?;
        Ok(())
    }

    pub async fn list_symlinks(&self) -> Vec<String> {
        self.symlinks.read().await.keys().cloned().collect()
    }

    pub async fn sync_all(&self) -> Result<Vec<(String, Vec<u8>)>, String> {
        let mut changes = Vec::new();
        let symlinks = self.symlinks.read().await;

        for (name, symlink) in symlinks.iter() {
            if symlink.is_active {
                let pid = symlink.target_pid.or(self.target_pid);
                if let Some(pid) = pid {
                    if let Ok(current_value) = memory_tools::read_process_memory(pid, symlink.address as usize, 8) {
                        if current_value != symlink.cached_value {
                            log::debug!("Symlink '{}' changed: {:?} -> {:?}", name, symlink.cached_value, current_value);
                            changes.push((name.clone(), current_value.clone()));
                        }
                    }
                }
            }
        }

        if !changes.is_empty() {
            log::info!("Sync detected {} changed symlinks", changes.len());
        }
        Ok(changes)
    }

    pub async fn get_symlink_info(&self, name: &str) -> Result<SymbolicLink, String> {
        let symlinks = self.symlinks.read().await;
        symlinks.get(name)
            .cloned()
            .ok_or_else(|| format!("Symlink not found: {}", name))
    }
}
