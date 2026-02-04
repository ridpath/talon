#![allow(dead_code)]

use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

use crate::interpreter::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptCheckpoint {
    pub name: String,
    pub timestamp: u64,
    pub variables: HashMap<String, String>,
    pub constants: HashMap<String, String>,
    pub script_state: String,
    pub execution_point: usize,
    pub network_state: Vec<NetworkConnection>,
    pub memory_state: Vec<MemorySnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConnection {
    pub id: u64,
    pub host: String,
    pub port: u16,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySnapshot {
    pub address: u64,
    pub size: usize,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyFork {
    pub name: String,
    pub parent: String,
    pub modifications: Vec<String>,
    pub commands: Vec<String>,
}

pub struct ScriptContinuity {
    checkpoints: HashMap<String, ScriptCheckpoint>,
    forks: HashMap<String, StrategyFork>,
}

impl ScriptContinuity {
    pub fn new() -> Self {
        ScriptContinuity {
            checkpoints: HashMap::new(),
            forks: HashMap::new(),
        }
    }

    pub fn create_checkpoint(
        &mut self,
        name: &str,
        variables: &HashMap<String, Value>,
        constants: &HashMap<String, Value>,
        script_state: &str,
        execution_point: usize,
    ) -> Result<(), String> {
        let variables_serialized = variables
            .iter()
            .map(|(k, v)| (k.clone(), v.to_string()))
            .collect();

        let constants_serialized = constants
            .iter()
            .map(|(k, v)| (k.clone(), v.to_string()))
            .collect();

        let checkpoint = ScriptCheckpoint {
            name: name.to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            variables: variables_serialized,
            constants: constants_serialized,
            script_state: script_state.to_string(),
            execution_point,
            network_state: Vec::new(),
            memory_state: Vec::new(),
        };

        self.checkpoints.insert(name.to_string(), checkpoint);
        self.save_checkpoint_to_disk(name)?;
        Ok(())
    }

    fn save_checkpoint_to_disk(&self, name: &str) -> Result<(), String> {
        let checkpoint = self
            .checkpoints
            .get(name)
            .ok_or_else(|| format!("Checkpoint not found: {}", name))?;

        let json_data = serde_json::to_string_pretty(checkpoint)
            .map_err(|e| format!("Failed to serialize checkpoint: {}", e))?;

        let path = format!("checkpoints/{}.checkpoint", name);
        fs::create_dir_all("checkpoints")
            .map_err(|e| format!("Failed to create checkpoints directory: {}", e))?;

        let file =
            File::create(&path).map_err(|e| format!("Failed to create checkpoint file: {}", e))?;

        let mut encoder = GzEncoder::new(file, Compression::best());
        encoder
            .write_all(json_data.as_bytes())
            .map_err(|e| format!("Failed to write checkpoint: {}", e))?;

        encoder
            .finish()
            .map_err(|e| format!("Failed to finalize checkpoint: {}", e))?;

        Ok(())
    }

    pub fn load_checkpoint(&mut self, name: &str) -> Result<ScriptCheckpoint, String> {
        let path = format!("checkpoints/{}.checkpoint", name);

        if !Path::new(&path).exists() {
            return Err(format!("Checkpoint file not found: {}", path));
        }

        let file =
            File::open(&path).map_err(|e| format!("Failed to open checkpoint file: {}", e))?;

        let mut decoder = GzDecoder::new(file);
        let mut json_data = String::new();
        decoder
            .read_to_string(&mut json_data)
            .map_err(|e| format!("Failed to decompress checkpoint: {}", e))?;

        let checkpoint: ScriptCheckpoint = serde_json::from_str(&json_data)
            .map_err(|e| format!("Failed to deserialize checkpoint: {}", e))?;

        self.checkpoints
            .insert(name.to_string(), checkpoint.clone());
        Ok(checkpoint)
    }

    pub fn fork_strategy(&mut self, name: &str, parent: &str) -> Result<(), String> {
        let fork = StrategyFork {
            name: name.to_string(),
            parent: parent.to_string(),
            modifications: Vec::new(),
            commands: Vec::new(),
        };

        self.forks.insert(name.to_string(), fork);
        Ok(())
    }

    pub fn merge_strategy(&mut self, source: &str, target: &str) -> Result<Vec<String>, String> {
        let source_fork = self
            .forks
            .get(source)
            .ok_or_else(|| format!("Source fork not found: {}", source))?;

        let merged_commands = source_fork.commands.clone();

        if let Some(target_fork) = self.forks.get_mut(target) {
            target_fork.commands.extend(merged_commands.clone());
        }

        Ok(merged_commands)
    }

    pub fn list_checkpoints(&self) -> Vec<String> {
        self.checkpoints.keys().cloned().collect()
    }

    pub fn list_forks(&self) -> Vec<String> {
        self.forks.keys().cloned().collect()
    }
}
