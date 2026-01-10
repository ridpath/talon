#![allow(dead_code)]

use std::process::Command;
use std::path::Path;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryAnalysis {
    pub architecture: String,
    pub os: String,
    pub bitness: usize,
    pub endianness: String,
    pub protections: BinaryProtections,
    pub sections: Vec<Section>,
    pub symbols: Vec<Symbol>,
    pub entry_point: u64,
    pub base_address: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryProtections {
    pub nx: bool,
    pub pie: bool,
    pub relro: RelroLevel,
    pub canary: bool,
    pub aslr: bool,
    pub fortify: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RelroLevel {
    None,
    Partial,
    Full,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    pub name: String,
    pub address: u64,
    pub size: u64,
    pub permissions: String,
    pub is_writable: bool,
    pub is_executable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    pub name: String,
    pub address: u64,
    pub symbol_type: String,
    pub is_imported: bool,
}

pub struct BinaryAnalyzer;

impl BinaryAnalyzer {
    pub fn analyze(binary_path: &str) -> Result<BinaryAnalysis, String> {
        if !Path::new(binary_path).exists() {
            return Err(format!("Binary not found: {}", binary_path));
        }

        let architecture = Self::detect_architecture(binary_path)?;
        let os = Self::detect_os(binary_path)?;
        let bitness = Self::detect_bitness(binary_path)?;
        let endianness = Self::detect_endianness(binary_path)?;
        let protections = Self::analyze_protections(binary_path)?;
        let sections = Self::analyze_sections(binary_path)?;
        let symbols = Self::extract_symbols(binary_path)?;
        let entry_point = Self::get_entry_point(binary_path)?;
        let base_address = Self::get_base_address(binary_path)?;

        Ok(BinaryAnalysis {
            architecture,
            os,
            bitness,
            endianness,
            protections,
            sections,
            symbols,
            entry_point,
            base_address,
        })
    }

    fn detect_architecture(binary_path: &str) -> Result<String, String> {
        let output = Command::new("file")
            .arg(binary_path)
            .output();

        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            
            if stdout.contains("x86-64") || stdout.contains("x86_64") {
                return Ok("x86_64".to_string());
            } else if stdout.contains("80386") || stdout.contains("i386") {
                return Ok("i386".to_string());
            } else if stdout.contains("ARM") || stdout.contains("aarch64") {
                return Ok("ARM".to_string());
            } else if stdout.contains("MIPS") {
                return Ok("MIPS".to_string());
            } else if stdout.contains("PowerPC") {
                return Ok("PowerPC".to_string());
            }
        }

        Ok("unknown".to_string())
    }

    fn detect_os(binary_path: &str) -> Result<String, String> {
        let output = Command::new("file")
            .arg(binary_path)
            .output();

        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            
            if stdout.contains("ELF") {
                return Ok("Linux".to_string());
            } else if stdout.contains("PE32") || stdout.contains("MS Windows") {
                return Ok("Windows".to_string());
            } else if stdout.contains("Mach-O") {
                return Ok("macOS".to_string());
            }
        }

        Ok("unknown".to_string())
    }

    fn detect_bitness(binary_path: &str) -> Result<usize, String> {
        let output = Command::new("file")
            .arg(binary_path)
            .output();

        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            
            if stdout.contains("64-bit") {
                return Ok(64);
            } else if stdout.contains("32-bit") {
                return Ok(32);
            }
        }

        Ok(64)
    }

    fn detect_endianness(binary_path: &str) -> Result<String, String> {
        let output = Command::new("file")
            .arg(binary_path)
            .output();

        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            
            if stdout.contains("LSB") {
                return Ok("little".to_string());
            } else if stdout.contains("MSB") {
                return Ok("big".to_string());
            }
        }

        Ok("little".to_string())
    }

    fn analyze_protections(binary_path: &str) -> Result<BinaryProtections, String> {
        let output = Command::new("checksec")
            .arg("--file")
            .arg(binary_path)
            .output();

        let mut protections = BinaryProtections {
            nx: false,
            pie: false,
            relro: RelroLevel::None,
            canary: false,
            aslr: false,
            fortify: false,
        };

        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            
            protections.nx = stdout.contains("NX enabled") || !stdout.contains("NX disabled");
            protections.pie = stdout.contains("PIE enabled");
            protections.canary = stdout.contains("Canary found") || stdout.contains("Stack");
            protections.fortify = stdout.contains("FORTIFY");
            
            if stdout.contains("Full RELRO") {
                protections.relro = RelroLevel::Full;
            } else if stdout.contains("Partial RELRO") {
                protections.relro = RelroLevel::Partial;
            }
        } else {
            protections = Self::analyze_protections_fallback(binary_path)?;
        }

        Ok(protections)
    }

    fn analyze_protections_fallback(binary_path: &str) -> Result<BinaryProtections, String> {
        let readelf_output = Command::new("readelf")
            .arg("-l")
            .arg(binary_path)
            .output();

        let mut protections = BinaryProtections {
            nx: false,
            pie: false,
            relro: RelroLevel::None,
            canary: false,
            aslr: false,
            fortify: false,
        };

        if let Ok(output) = readelf_output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            protections.nx = stdout.contains("GNU_STACK") && !stdout.contains("RWE");
            protections.pie = stdout.contains("DYN");
        }

        let readelf_relro = Command::new("readelf")
            .arg("-d")
            .arg(binary_path)
            .output();

        if let Ok(output) = readelf_relro {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("BIND_NOW") {
                protections.relro = RelroLevel::Full;
            } else if stdout.contains("GNU_RELRO") {
                protections.relro = RelroLevel::Partial;
            }
        }

        let symbols_output = Command::new("nm")
            .arg(binary_path)
            .output();

        if let Ok(output) = symbols_output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            protections.canary = stdout.contains("__stack_chk_fail");
            protections.fortify = stdout.contains("__chk");
        }

        Ok(protections)
    }

    fn analyze_sections(binary_path: &str) -> Result<Vec<Section>, String> {
        let output = Command::new("readelf")
            .arg("-S")
            .arg(binary_path)
            .output();

        let mut sections = Vec::new();

        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            
            for line in stdout.lines() {
                if line.contains('[') && (line.contains(".text") || line.contains(".data") 
                    || line.contains(".bss") || line.contains(".rodata") 
                    || line.contains(".got") || line.contains(".plt")) {
                    
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 7 {
                        let name = parts[1].trim_start_matches('[').to_string();
                        let address = u64::from_str_radix(parts[3], 16).unwrap_or(0);
                        let size = u64::from_str_radix(parts[5], 16).unwrap_or(0);
                        let flags = if parts.len() > 7 { parts[7] } else { "" };
                        
                        let is_writable = flags.contains('W');
                        let is_executable = flags.contains('X');
                        
                        sections.push(Section {
                            name,
                            address,
                            size,
                            permissions: flags.to_string(),
                            is_writable,
                            is_executable,
                        });
                    }
                }
            }
        }

        if sections.is_empty() {
            sections.push(Section {
                name: ".text".to_string(),
                address: 0x400000,
                size: 0x1000,
                permissions: "rx".to_string(),
                is_writable: false,
                is_executable: true,
            });
            sections.push(Section {
                name: ".data".to_string(),
                address: 0x601000,
                size: 0x1000,
                permissions: "rw".to_string(),
                is_writable: true,
                is_executable: false,
            });
        }

        Ok(sections)
    }

    fn extract_symbols(binary_path: &str) -> Result<Vec<Symbol>, String> {
        let output = Command::new("nm")
            .arg("-D")
            .arg(binary_path)
            .output();

        let mut symbols = Vec::new();

        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let address = u64::from_str_radix(parts[0], 16).unwrap_or(0);
                    let symbol_type = parts[1].to_string();
                    let name = parts[2].to_string();
                    
                    symbols.push(Symbol {
                        name,
                        address,
                        symbol_type,
                        is_imported: false,
                    });
                }
            }
        }

        if symbols.is_empty() {
            symbols.push(Symbol {
                name: "main".to_string(),
                address: 0x400000,
                symbol_type: "T".to_string(),
                is_imported: false,
            });
        }

        Ok(symbols)
    }

    fn get_entry_point(binary_path: &str) -> Result<u64, String> {
        let output = Command::new("readelf")
            .arg("-h")
            .arg(binary_path)
            .output();

        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            
            for line in stdout.lines() {
                if line.contains("Entry point address:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if let Some(addr_str) = parts.last() {
                        if let Ok(addr) = u64::from_str_radix(addr_str.trim_start_matches("0x"), 16) {
                            return Ok(addr);
                        }
                    }
                }
            }
        }

        Ok(0x400000)
    }

    fn get_base_address(_binary_path: &str) -> Result<u64, String> {
        Ok(0x400000)
    }

    pub fn find_dangerous_functions(symbols: &[Symbol]) -> Vec<String> {
        let dangerous = vec![
            "strcpy", "strcat", "gets", "sprintf", "scanf",
            "system", "exec", "popen", "strcpy", "strncpy",
            "memcpy", "memmove", "read", "fread", "getenv"
        ];

        symbols.iter()
            .filter(|s| dangerous.iter().any(|&d| s.name.contains(d)))
            .map(|s| s.name.clone())
            .collect()
    }

    pub fn find_interesting_functions(symbols: &[Symbol]) -> Vec<String> {
        let interesting = vec![
            "main", "system", "execve", "mprotect", "mmap",
            "strcpy", "gets", "read", "printf", "scanf",
            "malloc", "free", "calloc", "realloc"
        ];

        symbols.iter()
            .filter(|s| interesting.iter().any(|&i| s.name.contains(i)))
            .map(|s| s.name.clone())
            .collect()
    }

    pub fn find_writable_sections(sections: &[Section]) -> Vec<String> {
        sections.iter()
            .filter(|s| s.is_writable)
            .map(|s| s.name.clone())
            .collect()
    }

    pub fn print_analysis(analysis: &BinaryAnalysis) {
        println!("\nBinary Analysis Report");
        println!("======================");
        println!("Architecture: {}", analysis.architecture);
        println!("OS: {}", analysis.os);
        println!("Bitness: {}-bit", analysis.bitness);
        println!("Endianness: {}", analysis.endianness);
        println!("\nProtections:");
        println!("  NX: {}", if analysis.protections.nx { "Enabled" } else { "Disabled" });
        println!("  PIE: {}", if analysis.protections.pie { "Enabled" } else { "Disabled" });
        println!("  RELRO: {:?}", analysis.protections.relro);
        println!("  Canary: {}", if analysis.protections.canary { "Found" } else { "Not found" });
        println!("  FORTIFY: {}", if analysis.protections.fortify { "Enabled" } else { "Disabled" });
        
        println!("\nSections:");
        for section in &analysis.sections {
            println!("  {} @ 0x{:x} (size: 0x{:x}) [{}{}]",
                section.name,
                section.address,
                section.size,
                if section.is_writable { "W" } else { "-" },
                if section.is_executable { "X" } else { "-" }
            );
        }

        let dangerous = Self::find_dangerous_functions(&analysis.symbols);
        if !dangerous.is_empty() {
            println!("\nDangerous Functions Found:");
            for func in dangerous {
                println!("  - {}", func);
            }
        }
    }
}
