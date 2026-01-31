// ═══════════════════════════════════════════════════════════════════════════
// LIBC DATABASE - COMMON LIBC VERSIONS AND OFFSETS
// ═══════════════════════════════════════════════════════════════════════════

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
        let mut ubuntu2004 = LibcVersion::new("ubuntu20.04-2.31", "6e0cdbd5c76c0813b7aad4f671a16bdb6f1c8cbb");
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
        let mut ubuntu1804 = LibcVersion::new("ubuntu18.04-2.27", "b5381a457906d279073822a5ceb24c4bfef94ddb");
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
        let mut ubuntu2204 = LibcVersion::new("ubuntu22.04-2.35", "d1df43cc9efc2a1e36e4ac69e5e4c1a06a2cb0f4");
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
        let mut debian10 = LibcVersion::new("debian10-2.28", "1e94beb079e278650d725fa3f61ad0b8d0d13f0c");
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
        self.versions.values()
            .find(|v| v.build_id == build_id)
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
        Some(libc.one_gadgets.iter().map(|&offset| base_addr + offset).collect())
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
    let libc = db.get(libc_name)
        .ok_or(format!("Libc version '{}' not found", libc_name))?;
    
    let system = base_addr + libc.system;
    let bin_sh = base_addr + libc.bin_sh_string;
    
    // Simple ret2libc: pop_rdi + /bin/sh + system
    // (pop_rdi gadget must be found separately)
    Ok(vec![bin_sh, system])
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
}
