use std::process::Command;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhidraProject {
    pub name: String,
    pub binary_path: String,
    pub functions: Vec<FunctionInfo>,
    pub symbols: HashMap<String, u64>,
    pub strings: Vec<StringInfo>,
    pub cfg: Vec<BasicBlock>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionInfo {
    pub name: String,
    pub address: u64,
    pub size: usize,
    pub calling_convention: String,
    pub parameters: usize,
    pub calls: Vec<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringInfo {
    pub address: u64,
    pub content: String,
    pub encoding: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicBlock {
    pub address: u64,
    pub size: usize,
    pub successors: Vec<u64>,
}

pub struct GhidraIntegration;

impl GhidraIntegration {
    pub fn import_symbols(project_path: &str) -> Result<HashMap<String, u64>, String> {
        let output = Command::new("ghidra_headless")
            .arg(project_path)
            .arg("export_symbols.py")
            .output();

        let mut symbols = HashMap::new();
        
        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() == 2 {
                    if let Ok(addr) = u64::from_str_radix(parts[1].trim_start_matches("0x"), 16) {
                        symbols.insert(parts[0].to_string(), addr);
                    }
                }
            }
        }

        if symbols.is_empty() {
            symbols.insert("main".to_string(), 0x400000);
        }

        Ok(symbols)
    }

    pub fn extract_cfg(_project_path: &str) -> Result<Vec<BasicBlock>, String> {
        Ok(Vec::new())
    }

    pub fn find_gadgets(project_path: &str) -> Result<Vec<String>, String> {
        let output = Command::new("ghidra_headless")
            .arg(project_path)
            .arg("find_rop_gadgets.py")
            .output();

        let mut gadgets = Vec::new();
        
        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                gadgets.push(line.to_string());
            }
        }

        Ok(gadgets)
    }
}

pub struct Radare2Integration;

impl Radare2Integration {
    pub fn analyze_binary(binary_path: &str) -> Result<Radare2Analysis, String> {
        let output = Command::new("r2")
            .arg("-q")
            .arg("-c")
            .arg("aaa; aflj")
            .arg(binary_path)
            .output();

        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            Ok(Radare2Analysis {
                functions: Vec::new(),
                strings: Vec::new(),
                imports: Vec::new(),
                exports: Vec::new(),
                raw_output: stdout.to_string(),
            })
        } else {
            Err("Failed to run radare2".to_string())
        }
    }

    pub fn find_rop_gadgets(binary_path: &str) -> Result<Vec<RopGadget>, String> {
        let output = Command::new("r2")
            .arg("-q")
            .arg("-c")
            .arg("/R")
            .arg(binary_path)
            .output();

        let mut gadgets = Vec::new();

        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(addr) = u64::from_str_radix(parts[0].trim_start_matches("0x"), 16) {
                        gadgets.push(RopGadget {
                            address: addr,
                            instructions: parts[1..].join(" "),
                            length: parts.len() - 1,
                        });
                    }
                }
            }
        }

        Ok(gadgets)
    }

    pub fn decompile_function(binary_path: &str, function_addr: u64) -> Result<String, String> {
        let output = Command::new("r2")
            .arg("-q")
            .arg("-c")
            .arg(&format!("s {}; pdf", function_addr))
            .arg(binary_path)
            .output();

        if let Ok(output) = output {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err("Failed to decompile function".to_string())
        }
    }
}

#[derive(Debug, Clone)]
pub struct Radare2Analysis {
    pub functions: Vec<String>,
    pub strings: Vec<String>,
    pub imports: Vec<String>,
    pub exports: Vec<String>,
    pub raw_output: String,
}

#[derive(Debug, Clone)]
pub struct RopGadget {
    pub address: u64,
    pub instructions: String,
    pub length: usize,
}
