// ═══════════════════════════════════════════════════════════════════════════
// LIBC DATABASE - COMMON LIBC VERSIONS AND OFFSETS
// ═══════════════════════════════════════════════════════════════════════════

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Libc version entry with common offsets
#[derive(Debug, Clone)]
pub struct LibcVersion {
    pub name: String,
    pub build_id: String,
    pub system: u64,
    pub execve: u64,
    pub sh_string: u64,
    pub bin_sh_string: u64,
    pub dup2: u64,
    pub read: u64,
    pub write: u64,
    pub open: u64,
    pub mprotect: u64,
    pub malloc_hook: u64,
    pub free_hook: u64,
    pub realloc_hook: u64,
    pub one_gadgets: Vec<u64>,
}

/// Libc match result with confidence scoring
#[derive(Debug, Clone)]
pub struct LibcMatch {
    pub version: LibcVersion,
    pub confidence: f32,
}

/// Response structure from libc.rip API
#[derive(Debug, Deserialize, Serialize)]
struct LibcRipResponse {
    id: String,
    buildid: Option<String>,
    md5: String,
    sha1: String,
    sha256: String,
    download_url: String,
    symbols: HashMap<String, String>,
}

impl LibcVersion {
    pub fn new(name: &str, build_id: &str) -> Self {
        LibcVersion {
            name: name.to_string(),
            build_id: build_id.to_string(),
            system: 0,
            execve: 0,
            sh_string: 0,
            bin_sh_string: 0,
            dup2: 0,
            read: 0,
            write: 0,
            open: 0,
            mprotect: 0,
            malloc_hook: 0,
            free_hook: 0,
            realloc_hook: 0,
            one_gadgets: Vec::new(),
        }
    }
}

/// Libc database
pub struct LibcDatabase {
    pub versions: HashMap<String, LibcVersion>,
}

impl Default for LibcDatabase {
    fn default() -> Self {
        Self::new()
    }
}

impl LibcDatabase {
    /// Create new libc database with pre-loaded versions
    pub fn new() -> Self {
        let mut db = LibcDatabase {
            versions: HashMap::new(),
        };

        db.load_common_versions();
        db
    }

    /// Load common libc versions
    fn load_common_versions(&mut self) {
        // Ubuntu 20.04 - libc 2.31
        let mut ubuntu2004 = LibcVersion::new(
            "ubuntu20.04-2.31",
            "6e0cdbd5c76c0813b7aad4f671a16bdb6f1c8cbb",
        );
        ubuntu2004.system = 0x50d60;
        ubuntu2004.execve = 0xe6e30;
        ubuntu2004.bin_sh_string = 0x1b45bd;
        ubuntu2004.sh_string = 0x1b45c0;
        ubuntu2004.dup2 = 0x110ab0;
        ubuntu2004.read = 0x111080;
        ubuntu2004.write = 0x1110d0;
        ubuntu2004.open = 0x10f4b0;
        ubuntu2004.mprotect = 0x11bae0;
        ubuntu2004.malloc_hook = 0x1ecb70;
        ubuntu2004.free_hook = 0x1eeb28;
        ubuntu2004.realloc_hook = 0x1ecb60;
        ubuntu2004.one_gadgets = vec![0x4f3d5, 0x4f432, 0x10a41c];
        self.versions.insert("ubuntu20.04".to_string(), ubuntu2004);

        // Ubuntu 18.04 - libc 2.27
        let mut ubuntu1804 = LibcVersion::new(
            "ubuntu18.04-2.27",
            "b5381a457906d279073822a5ceb24c4bfef94ddb",
        );
        ubuntu1804.system = 0x4f440;
        ubuntu1804.execve = 0xe4e30;
        ubuntu1804.bin_sh_string = 0x1b3e9a;
        ubuntu1804.sh_string = 0x1b3e9d;
        ubuntu1804.dup2 = 0x110290;
        ubuntu1804.read = 0x110070;
        ubuntu1804.write = 0x1100c0;
        ubuntu1804.malloc_hook = 0x1ecb70;
        ubuntu1804.free_hook = 0x1eeb28;
        ubuntu1804.realloc_hook = 0x1ecb60;
        ubuntu1804.one_gadgets = vec![0x4f2c5, 0x4f322, 0x10a38c];
        self.versions.insert("ubuntu18.04".to_string(), ubuntu1804);

        // Ubuntu 22.04 - libc 2.35
        let mut ubuntu2204 = LibcVersion::new(
            "ubuntu22.04-2.35",
            "d1df43cc9efc2a1e36e4ac69e5e4c1a06a2cb0f4",
        );
        ubuntu2204.system = 0x50d70;
        ubuntu2204.execve = 0xeb2a0;
        ubuntu2204.bin_sh_string = 0x1d8678;
        ubuntu2204.sh_string = 0x1d867b;
        ubuntu2204.dup2 = 0x114340;
        ubuntu2204.read = 0x114920;
        ubuntu2204.write = 0x114970;
        ubuntu2204.mprotect = 0x120470;
        ubuntu2204.one_gadgets = vec![0x50a47, 0xebc81, 0xebc85];
        self.versions.insert("ubuntu22.04".to_string(), ubuntu2204);

        // Debian 10 (buster) - libc 2.28
        let mut debian10 =
            LibcVersion::new("debian10-2.28", "1e94beb079e278650d725fa3f61ad0b8d0d13f0c");
        debian10.system = 0x52290;
        debian10.execve = 0xe7880;
        debian10.bin_sh_string = 0x19b0c3;
        debian10.sh_string = 0x19b0c6;
        debian10.malloc_hook = 0x1e6c00;
        debian10.free_hook = 0x1e8bb8;
        debian10.one_gadgets = vec![0x52293, 0x52290, 0x10a324];
        self.versions.insert("debian10".to_string(), debian10);

        log::info!("Loaded {} libc versions", self.versions.len());
    }

    /// Get libc version by name
    pub fn get(&self, name: &str) -> Option<&LibcVersion> {
        self.versions.get(name)
    }

    /// Find libc version by build ID
    pub fn find_by_build_id(&self, build_id: &str) -> Option<&LibcVersion> {
        self.versions.values().find(|v| v.build_id == build_id)
    }

    /// List all available libc versions
    pub fn list(&self) -> Vec<&LibcVersion> {
        self.versions.values().collect()
    }

    /// Calculate absolute address from base
    pub fn resolve_address(&self, libc_name: &str, base_addr: u64, symbol: &str) -> Option<u64> {
        let libc = self.get(libc_name)?;

        let offset = match symbol {
            "system" => libc.system,
            "execve" => libc.execve,
            "/bin/sh" => libc.bin_sh_string,
            "sh" => libc.sh_string,
            "dup2" => libc.dup2,
            "read" => libc.read,
            "write" => libc.write,
            "open" => libc.open,
            "mprotect" => libc.mprotect,
            "__malloc_hook" => libc.malloc_hook,
            "__free_hook" => libc.free_hook,
            "__realloc_hook" => libc.realloc_hook,
            _ => return None,
        };

        Some(base_addr + offset)
    }

    /// Get one-gadget addresses
    pub fn get_one_gadgets(&self, libc_name: &str, base_addr: u64) -> Option<Vec<u64>> {
        let libc = self.get(libc_name)?;
        Some(
            libc.one_gadgets
                .iter()
                .map(|&offset| base_addr + offset)
                .collect(),
        )
    }

    /// Identify libc version from leaked address with confidence scoring
    pub fn identify(&self, leak_addr: u64, symbol: &str) -> Result<LibcVersion, String> {
        log::info!(
            "[*] Identifying libc from leaked address: 0x{:x} ({})",
            leak_addr,
            symbol
        );

        let offset = leak_addr & 0xFFF;
        let mut matches = Vec::new();

        for libc in self.versions.values() {
            let symbol_offset = match symbol {
                "system" => libc.system,
                "execve" => libc.execve,
                "/bin/sh" => libc.bin_sh_string,
                "sh" => libc.sh_string,
                "dup2" => libc.dup2,
                "read" => libc.read,
                "write" => libc.write,
                "open" => libc.open,
                "mprotect" => libc.mprotect,
                "__malloc_hook" => libc.malloc_hook,
                "__free_hook" => libc.free_hook,
                "__realloc_hook" => libc.realloc_hook,
                _ => continue,
            };

            let expected_offset = symbol_offset & 0xFFF;

            if expected_offset == offset {
                let confidence = self.calculate_confidence(libc, &leak_addr.to_string(), offset);
                matches.push(LibcMatch {
                    version: libc.clone(),
                    confidence,
                });
            }
        }

        if matches.is_empty() {
            log::warn!("[!] No libc matches found in local database, trying libc.rip API");
            return self.identify_via_api(leak_addr, symbol);
        }

        if matches.len() > 1 {
            log::warn!("[!] Multiple libc candidates found, using highest confidence");
            for m in &matches {
                log::info!(
                    "[*] Candidate: {} (confidence: {:.1}%)",
                    m.version.name,
                    m.confidence
                );
            }
        }

        matches.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        let best = &matches[0];
        log::info!(
            "[+] Libc match: {} (confidence: {:.1}%)",
            best.version.name,
            best.confidence
        );

        Ok(best.version.clone())
    }

    /// Calculate confidence score for libc match
    fn calculate_confidence(&self, candidate: &LibcVersion, build_id: &str, _offset: u64) -> f32 {
        if candidate.build_id == build_id {
            return 100.0;
        }

        let partial_match = if candidate.build_id.len() >= 8 && build_id.len() >= 8 {
            candidate.build_id[..8] == build_id[..8]
        } else {
            false
        };

        if partial_match {
            return 75.0;
        }

        50.0
    }

    /// Identify libc via libc.rip API with fallback to local database
    fn identify_via_api(&self, leak_addr: u64, symbol: &str) -> Result<LibcVersion, String> {
        let offset = leak_addr & 0xFFF;

        let client = match reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
        {
            Ok(c) => c,
            Err(_) => {
                log::warn!("[!] Failed to create HTTP client, using local database only");
                return Err(format!(
                    "No libc match found for {} offset 0x{:x}",
                    symbol, offset
                ));
            }
        };

        let url = "https://libc.rip/api/find";
        let mut query = HashMap::new();
        query.insert(symbol.to_string(), format!("{:x}", offset));

        log::info!("[*] Querying libc.rip API...");

        let response = match client.post(url).json(&query).send() {
            Ok(r) => r,
            Err(_) => {
                log::warn!("[!] libc.rip API request failed, network unavailable");
                return Err(format!(
                    "No libc match found for {} offset 0x{:x}",
                    symbol, offset
                ));
            }
        };

        if !response.status().is_success() {
            log::warn!("[!] libc.rip returned status: {}", response.status());
            return Err(format!("libc.rip API error: {}", response.status()));
        }

        let results: Vec<LibcRipResponse> = match response.json() {
            Ok(r) => r,
            Err(e) => {
                log::warn!("[!] Failed to parse libc.rip response: {}", e);
                return Err(format!("Failed to parse API response: {}", e));
            }
        };

        if results.is_empty() {
            log::warn!("[-] No results from libc.rip");
            return Err(format!(
                "No libc match found for {} offset 0x{:x}",
                symbol, offset
            ));
        }

        let mut matches: Vec<LibcMatch> = Vec::new();

        for result in results.iter().take(5) {
            let mut libc_version =
                LibcVersion::new(&result.id, result.buildid.as_deref().unwrap_or(""));

            for (sym, hex_val) in &result.symbols {
                if let Ok(val) = u64::from_str_radix(hex_val.trim_start_matches("0x"), 16) {
                    match sym.as_str() {
                        "system" => libc_version.system = val,
                        "execve" => libc_version.execve = val,
                        "read" => libc_version.read = val,
                        "write" => libc_version.write = val,
                        "dup2" => libc_version.dup2 = val,
                        "open" => libc_version.open = val,
                        "mprotect" => libc_version.mprotect = val,
                        "__malloc_hook" => libc_version.malloc_hook = val,
                        "__free_hook" => libc_version.free_hook = val,
                        "__realloc_hook" => libc_version.realloc_hook = val,
                        _ => {}
                    }
                }
            }

            let confidence = 90.0 - (matches.len() as f32 * 5.0);

            matches.push(LibcMatch {
                version: libc_version,
                confidence: confidence.max(50.0),
            });
        }

        if matches.len() > 1 {
            log::warn!("[!] Multiple libc candidates found from API, using highest confidence");
            for m in &matches {
                log::info!(
                    "[*] Candidate: {} (confidence: {:.1}%)",
                    m.version.name,
                    m.confidence
                );
            }
        }

        matches.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        let best = &matches[0];
        log::info!(
            "[+] Libc match: {} (confidence: {:.1}%)",
            best.version.name,
            best.confidence
        );

        Ok(best.version.clone())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// HELPER FUNCTIONS
// ────────────────────────────────────────────────────────────────────────────

/// Get libc database instance
pub fn get_libc_db() -> LibcDatabase {
    LibcDatabase::new()
}

/// Quick resolve symbol address
pub fn resolve_libc_symbol(libc_name: &str, base_addr: u64, symbol: &str) -> Option<u64> {
    let db = get_libc_db();
    db.resolve_address(libc_name, base_addr, symbol)
}

/// Get one-gadget addresses for a libc version
pub fn get_one_gadgets(libc_name: &str, base_addr: u64) -> Result<Vec<u64>, String> {
    let db = LibcDatabase::new();
    db.get_one_gadgets(libc_name, base_addr)
        .ok_or_else(|| format!("Libc version '{}' not found", libc_name))
}

/// List all available libc versions
pub fn list_libc_versions() {
    let db = get_libc_db();
    println!("Available Libc Versions:");
    println!("{:-<70}", "");

    for libc in db.list() {
        println!("{:20} - Build ID: {}", libc.name, &libc.build_id[..16]);
        println!("  system:        0x{:x}", libc.system);
        println!("  /bin/sh:       0x{:x}", libc.bin_sh_string);
        println!("  __malloc_hook: 0x{:x}", libc.malloc_hook);
        println!("  one-gadgets:   {} available", libc.one_gadgets.len());
        println!();
    }
}

/// Helper to build ret2libc chain with automatic symbol resolution
pub fn auto_ret2libc(libc_name: &str, base_addr: u64) -> Result<Vec<u64>, String> {
    let db = get_libc_db();
    let libc = db
        .get(libc_name)
        .ok_or(format!("Libc version '{}' not found", libc_name))?;

    let system = base_addr + libc.system;
    let bin_sh = base_addr + libc.bin_sh_string;

    // Simple ret2libc: pop_rdi + /bin/sh + system
    // (pop_rdi gadget must be found separately)
    Ok(vec![bin_sh, system])
}

/// Identify libc version from leaked address
pub fn identify_libc(leak_addr: u64, symbol: &str) -> Result<LibcVersion, String> {
    let db = get_libc_db();
    db.identify(leak_addr, symbol)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_libc_db_creation() {
        let db = LibcDatabase::new();
        assert!(!db.versions.is_empty());
    }

    #[test]
    fn test_get_libc_version() {
        let db = LibcDatabase::new();
        let libc = db.get("ubuntu20.04");
        assert!(libc.is_some());
        assert_eq!(libc.unwrap().system, 0x50d60);
    }

    #[test]
    fn test_resolve_address() {
        let db = LibcDatabase::new();
        let base = 0x7ffff7a0d000u64;
        let system_addr = db.resolve_address("ubuntu20.04", base, "system");
        assert!(system_addr.is_some());
        assert_eq!(system_addr.unwrap(), base + 0x50d60);
    }

    #[test]
    fn test_one_gadgets() {
        let db = LibcDatabase::new();
        let base = 0x7ffff7a0d000u64;
        let gadgets = db.get_one_gadgets("ubuntu20.04", base);
        assert!(gadgets.is_some());
        assert!(!gadgets.unwrap().is_empty());
    }

    #[test]
    fn test_auto_ret2libc() {
        let result = auto_ret2libc("ubuntu20.04", 0x7ffff7a0d000);
        assert!(result.is_ok());
        let chain = result.unwrap();
        assert_eq!(chain.len(), 2); // /bin/sh + system
    }

    #[test]
    fn test_identify_libc_local() {
        let db = LibcDatabase::new();

        let leaked_addr = 0x7ffff7a5dd60u64;
        let result = db.identify(leaked_addr, "system");

        assert!(result.is_ok());
        let libc = result.unwrap();
        assert!(!libc.name.is_empty());
        assert_eq!(libc.system & 0xFFF, leaked_addr & 0xFFF);
    }

    #[test]
    fn test_confidence_scoring() {
        let db = LibcDatabase::new();
        let libc = db.get("ubuntu20.04").unwrap();

        let confidence_exact = db.calculate_confidence(libc, &libc.build_id, 0x0);
        assert_eq!(confidence_exact, 100.0);

        let confidence_partial = db.calculate_confidence(libc, &libc.build_id[..8], 0x0);
        assert_eq!(confidence_partial, 75.0);

        let confidence_none = db.calculate_confidence(libc, "different_build_id", 0x0);
        assert_eq!(confidence_none, 50.0);
    }

    #[test]
    fn test_identify_multiple_matches() {
        let db = LibcDatabase::new();

        let leaked_addr = 0x7ffff7a0d000u64 + 0x050d60u64;
        let result = db.identify(leaked_addr, "system");

        assert!(result.is_ok());
        let libc = result.unwrap();
        assert_eq!(libc.system & 0xFFF, leaked_addr & 0xFFF);
    }
}
