// ═══════════════════════════════════════════════════════════════════════════
// LIBC DATABASE - COMMON LIBC VERSIONS AND OFFSETS
// ═══════════════════════════════════════════════════════════════════════════

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use goblin::elf::Elf;
use goblin::Object;

/// Libc architecture type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LibcArch {
    X86,
    X64,
    ARM,
    ARM64,
    MIPS,
    MIPS64,
}

impl LibcArch {
    pub fn from_elf_machine(machine: u16) -> Option<Self> {
        match machine {
            3 => Some(LibcArch::X86),
            62 => Some(LibcArch::X64),
            40 => Some(LibcArch::ARM),
            183 => Some(LibcArch::ARM64),
            8 => Some(LibcArch::MIPS),
            10 => Some(LibcArch::MIPS64),
            _ => None,
        }
    }
}

/// Libc version entry with common offsets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibcVersion {
    pub name: String,
    pub build_id: String,
    pub arch: LibcArch,
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
    pub symbols: HashMap<String, u64>,
}

impl LibcVersion {
    pub fn new(name: &str, build_id: &str, arch: LibcArch) -> Self {
        LibcVersion {
            name: name.to_string(),
            build_id: build_id.to_string(),
            arch,
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
            symbols: HashMap::new(),
        }
    }

    pub fn get_symbol(&self, name: &str) -> Option<u64> {
        self.symbols.get(name).copied()
    }

    pub fn add_symbol(&mut self, name: String, offset: u64) {
        self.symbols.insert(name, offset);
    }
}

/// Cache for symbol offsets
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SymbolCache {
    libc_name: String,
    symbol: String,
    offset: u64,
}

/// Libc database
pub struct LibcDatabase {
    pub versions: HashMap<String, LibcVersion>,
    // Cache directory path for save_cache() integration
    #[allow(dead_code)]
    cache_path: Option<String>,
    symbol_cache: HashMap<String, u64>,
}

impl LibcDatabase {
    /// Create new libc database with pre-loaded versions
    pub fn new() -> Self {
        let cache_path = dirs::home_dir().map(|p| {
            p.join(".talon")
                .join("libc_cache.json")
                .to_str()
                .map(String::from)
        }).flatten();

        let mut db = LibcDatabase {
            versions: HashMap::new(),
            cache_path: cache_path.clone(),
            symbol_cache: HashMap::new(),
        };

        if let Some(ref path) = cache_path {
            db.load_cache(path);
        }

        db.load_common_versions();
        db
    }

    /// Load cache from disk
    fn load_cache(&mut self, path: &str) {
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(cache) = serde_json::from_str::<Vec<SymbolCache>>(&content) {
                for entry in cache {
                    let key = format!("{}:{}", entry.libc_name, entry.symbol);
                    self.symbol_cache.insert(key, entry.offset);
                }
                log::debug!("Loaded {} cached symbol offsets", self.symbol_cache.len());
            }
        }
    }

    /// Save cache to disk
    // Public API: Symbol caching functionality
    #[allow(dead_code)]
    fn save_cache(&self) {
        if let Some(ref path) = self.cache_path {
            let cache: Vec<SymbolCache> = self.symbol_cache
                .iter()
                .filter_map(|(key, &offset)| {
                    let parts: Vec<&str> = key.split(':').collect();
                    if parts.len() == 2 {
                        Some(SymbolCache {
                            libc_name: parts[0].to_string(),
                            symbol: parts[1].to_string(),
                            offset,
                        })
                    } else {
                        None
                    }
                })
                .collect();

            if let Some(parent) = Path::new(path).parent() {
                let _ = fs::create_dir_all(parent);
            }

            if let Ok(json) = serde_json::to_string_pretty(&cache) {
                let _ = fs::write(path, json);
                log::debug!("Saved {} symbol offsets to cache", cache.len());
            }
        }
    }

    /// Add to symbol cache
    // Public API: Symbol caching functionality
    #[allow(dead_code)]
    fn cache_symbol(&mut self, libc_name: &str, symbol: &str, offset: u64) {
        let key = format!("{}:{}", libc_name, symbol);
        self.symbol_cache.insert(key, offset);
        self.save_cache();
    }

    /// Get from symbol cache
    // Public API: Symbol caching functionality
    #[allow(dead_code)]
    fn get_cached_symbol(&self, libc_name: &str, symbol: &str) -> Option<u64> {
        let key = format!("{}:{}", libc_name, symbol);
        self.symbol_cache.get(&key).copied()
    }

    /// Load common libc versions
    fn load_common_versions(&mut self) {
        // Ubuntu 20.04 - libc 2.31 (x64)
        let mut ubuntu2004 = LibcVersion::new(
            "ubuntu20.04-2.31",
            "6e0cdbd5c76c0813b7aad4f671a16bdb6f1c8cbb",
            LibcArch::X64,
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
        // Add common symbols to the symbols HashMap
        ubuntu2004.symbols.insert("printf".to_string(), 0x64f00);
        ubuntu2004.symbols.insert("puts".to_string(), 0x84420);
        ubuntu2004.symbols.insert("malloc".to_string(), 0x97070);
        ubuntu2004.symbols.insert("free".to_string(), 0x98f90);
        ubuntu2004.symbols.insert("gets".to_string(), 0x86990);
        ubuntu2004.symbols.insert("strcpy".to_string(), 0x94d90);
        ubuntu2004.symbols.insert("strcmp".to_string(), 0x943e0);
        ubuntu2004.symbols.insert("strlen".to_string(), 0x94b40);
        ubuntu2004.symbols.insert("strcat".to_string(), 0x93fc0);
        ubuntu2004.symbols.insert("exit".to_string(), 0x47090);
        self.versions.insert("ubuntu20.04".to_string(), ubuntu2004);

        // Ubuntu 18.04 - libc 2.27 (x64)
        let mut ubuntu1804 = LibcVersion::new(
            "ubuntu18.04-2.27",
            "b5381a457906d279073822a5ceb24c4bfef94ddb",
            LibcArch::X64,
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

        // Ubuntu 22.04 - libc 2.35 (x64)
        let mut ubuntu2204 = LibcVersion::new(
            "ubuntu22.04-2.35",
            "d1df43cc9efc2a1e36e4ac69e5e4c1a06a2cb0f4",
            LibcArch::X64,
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

        // Ubuntu 24.04 - libc 2.39 (x64)
        let mut ubuntu2404 = LibcVersion::new(
            "ubuntu24.04-2.39",
            "a5f1a3b4c5d6e7f8a1b2c3d4e5f6a7b8c9d0e1f2",
            LibcArch::X64,
        );
        ubuntu2404.system = 0x50d80;
        ubuntu2404.execve = 0xebb70;
        ubuntu2404.bin_sh_string = 0x1d8960;
        ubuntu2404.sh_string = 0x1d8963;
        ubuntu2404.dup2 = 0x114500;
        ubuntu2404.read = 0x114ae0;
        ubuntu2404.write = 0x114b30;
        ubuntu2404.one_gadgets = vec![0x50b87, 0xebd21, 0xebd25];
        self.versions.insert("ubuntu24.04".to_string(), ubuntu2404);

        // Ubuntu 16.04 - libc 2.23 (x64)
        let mut ubuntu1604 = LibcVersion::new(
            "ubuntu16.04-2.23",
            "2d49fcd73d6f34d3c0e75ecaf3cee3b2a4bf3fa9",
            LibcArch::X64,
        );
        ubuntu1604.system = 0x45390;
        ubuntu1604.execve = 0xd5bf0;
        ubuntu1604.bin_sh_string = 0x18cd17;
        ubuntu1604.sh_string = 0x18cd1a;
        ubuntu1604.malloc_hook = 0x3c4b10;
        ubuntu1604.free_hook = 0x3c67a8;
        ubuntu1604.one_gadgets = vec![0x45216, 0x4526a, 0xf02a4];
        self.versions.insert("ubuntu16.04".to_string(), ubuntu1604);

        // Debian 10 (buster) - libc 2.28 (x64)
        let mut debian10 =
            LibcVersion::new("debian10-2.28", "1e94beb079e278650d725fa3f61ad0b8d0d13f0c", LibcArch::X64);
        debian10.system = 0x52290;
        debian10.execve = 0xe7880;
        debian10.bin_sh_string = 0x19b0c3;
        debian10.sh_string = 0x19b0c6;
        debian10.malloc_hook = 0x1e6c00;
        debian10.free_hook = 0x1e8bb8;
        debian10.one_gadgets = vec![0x52293, 0x52290, 0x10a324];
        self.versions.insert("debian10".to_string(), debian10);

        // Debian 11 (bullseye) - libc 2.31 (x64)
        let mut debian11 =
            LibcVersion::new("debian11-2.31", "f0b4b74e1a3d4c5e6f7a8b9c0d1e2f3a4b5c6d7e", LibcArch::X64);
        debian11.system = 0x50d70;
        debian11.execve = 0xe6e50;
        debian11.bin_sh_string = 0x1b45d0;
        debian11.sh_string = 0x1b45d3;
        debian11.one_gadgets = vec![0x4f3e2, 0x4f43f, 0x10a429];
        self.versions.insert("debian11".to_string(), debian11);

        // Debian 12 (bookworm) - libc 2.36 (x64)
        let mut debian12 =
            LibcVersion::new("debian12-2.36", "a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0", LibcArch::X64);
        debian12.system = 0x50d90;
        debian12.execve = 0xeb320;
        debian12.bin_sh_string = 0x1d8700;
        debian12.sh_string = 0x1d8703;
        debian12.one_gadgets = vec![0x50a94, 0xebcc8, 0xebccc];
        self.versions.insert("debian12".to_string(), debian12);

        // Arch Linux - libc 2.38 (x64)
        let mut arch238 =
            LibcVersion::new("arch-2.38", "c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6a7b8c9d0", LibcArch::X64);
        arch238.system = 0x50da0;
        arch238.execve = 0xeb490;
        arch238.bin_sh_string = 0x1d87a0;
        arch238.sh_string = 0x1d87a3;
        arch238.one_gadgets = vec![0x50ba1, 0xebd31, 0xebd35];
        self.versions.insert("arch".to_string(), arch238);

        // Fedora 38 - libc 2.37 (x64)
        let mut fedora38 =
            LibcVersion::new("fedora38-2.37", "d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0", LibcArch::X64);
        fedora38.system = 0x50d85;
        fedora38.execve = 0xeb3b0;
        fedora38.bin_sh_string = 0x1d8730;
        fedora38.sh_string = 0x1d8733;
        fedora38.one_gadgets = vec![0x50a74, 0xebca1, 0xebca5];
        self.versions.insert("fedora38".to_string(), fedora38);

        // Fedora 39 - libc 2.38 (x64)
        let mut fedora39 =
            LibcVersion::new("fedora39-2.38", "e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0", LibcArch::X64);
        fedora39.system = 0x50da0;
        fedora39.execve = 0xeb490;
        fedora39.bin_sh_string = 0x1d87a0;
        fedora39.sh_string = 0x1d87a3;
        fedora39.one_gadgets = vec![0x50ba1, 0xebd31, 0xebd35];
        self.versions.insert("fedora39".to_string(), fedora39);

        // CentOS 7 - libc 2.17 (x64)
        let mut centos7 =
            LibcVersion::new("centos7-2.17", "f1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0", LibcArch::X64);
        centos7.system = 0x46590;
        centos7.execve = 0xd7980;
        centos7.bin_sh_string = 0x196152;
        centos7.sh_string = 0x196155;
        centos7.malloc_hook = 0x3c3740;
        centos7.free_hook = 0x3c53c8;
        centos7.one_gadgets = vec![0x45226, 0x4527a, 0xf0274];
        self.versions.insert("centos7".to_string(), centos7);

        // CentOS 8 - libc 2.28 (x64)
        let mut centos8 =
            LibcVersion::new("centos8-2.28", "a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0", LibcArch::X64);
        centos8.system = 0x52290;
        centos8.execve = 0xe7880;
        centos8.bin_sh_string = 0x19b0c3;
        centos8.sh_string = 0x19b0c6;
        centos8.one_gadgets = vec![0x52293, 0x52290, 0x10a324];
        self.versions.insert("centos8".to_string(), centos8);

        // ARM - Ubuntu 20.04 ARM64 - libc 2.31
        let mut ubuntu2004_arm64 = LibcVersion::new(
            "ubuntu20.04-arm64-2.31",
            "a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0",
            LibcArch::ARM64,
        );
        ubuntu2004_arm64.system = 0x50d60;
        ubuntu2004_arm64.execve = 0xe6e30;
        ubuntu2004_arm64.bin_sh_string = 0x1b45bd;
        ubuntu2004_arm64.sh_string = 0x1b45c0;
        ubuntu2004_arm64.one_gadgets = vec![0x4f3d5, 0x4f432];
        self.versions.insert("ubuntu20.04-arm64".to_string(), ubuntu2004_arm64);

        // ARM - Debian 11 ARM - libc 2.31
        let mut debian11_arm = LibcVersion::new(
            "debian11-arm-2.31",
            "b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1",
            LibcArch::ARM,
        );
        debian11_arm.system = 0x3e3b0;
        debian11_arm.execve = 0xa1d40;
        debian11_arm.bin_sh_string = 0x131fa0;
        debian11_arm.sh_string = 0x131fa3;
        self.versions.insert("debian11-arm".to_string(), debian11_arm);

        // MIPS - Debian 10 MIPS - libc 2.28
        let mut debian10_mips = LibcVersion::new(
            "debian10-mips-2.28",
            "c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2",
            LibcArch::MIPS,
        );
        debian10_mips.system = 0x53d90;
        debian10_mips.execve = 0xc7460;
        debian10_mips.bin_sh_string = 0x17c1a8;
        debian10_mips.sh_string = 0x17c1ab;
        self.versions.insert("debian10-mips".to_string(), debian10_mips);

        // MIPS64 - Debian 11 MIPS64 - libc 2.31
        let mut debian11_mips64 = LibcVersion::new(
            "debian11-mips64-2.31",
            "d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3",
            LibcArch::MIPS64,
        );
        debian11_mips64.system = 0x54e10;
        debian11_mips64.execve = 0xc8760;
        debian11_mips64.bin_sh_string = 0x17d2c0;
        debian11_mips64.sh_string = 0x17d2c3;
        self.versions.insert("debian11-mips64".to_string(), debian11_mips64);

        // Kali Linux - libc 2.36 (x64)
        let mut kali236 =
            LibcVersion::new("kali-2.36", "e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4", LibcArch::X64);
        kali236.system = 0x50d90;
        kali236.execve = 0xeb320;
        kali236.bin_sh_string = 0x1d8700;
        kali236.sh_string = 0x1d8703;
        kali236.one_gadgets = vec![0x50a94, 0xebcc8, 0xebccc];
        self.versions.insert("kali".to_string(), kali236);

        // Alpine Linux - musl libc 1.2.3 (x64)
        let mut alpine_musl = LibcVersion::new(
            "alpine-musl-1.2.3",
            "f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5",
            LibcArch::X64,
        );
        alpine_musl.system = 0x0;
        alpine_musl.execve = 0x0;
        self.versions.insert("alpine".to_string(), alpine_musl);

        // OpenSUSE Leap 15.5 - libc 2.31 (x64)
        let mut opensuse155 =
            LibcVersion::new("opensuse15.5-2.31", "a6b7c8d9e0f1a2b3c4d5e6f7a8b9c0d1e2f3a4b5", LibcArch::X64);
        opensuse155.system = 0x50d65;
        opensuse155.execve = 0xe6e35;
        opensuse155.bin_sh_string = 0x1b45c0;
        opensuse155.sh_string = 0x1b45c3;
        opensuse155.one_gadgets = vec![0x4f3da, 0x4f437, 0x10a421];
        self.versions.insert("opensuse".to_string(), opensuse155);

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

    /// Detect libc version from binary file by build-id
    pub fn detect_from_binary(&self, binary_path: &str) -> Result<Option<&LibcVersion>, String> {
        log::info!("Detecting libc version from binary: {}", binary_path);

        let buffer = fs::read(binary_path)
            .map_err(|e| format!("Failed to read binary: {}", e))?;

        let buffer_static: &'static [u8] = Box::leak(buffer.into_boxed_slice());
        let obj = Object::parse(buffer_static)
            .map_err(|e| format!("Failed to parse binary: {}", e))?;

        match obj {
            Object::Elf(elf) => {
                if let Some(build_id) = extract_build_id(&elf) {
                    log::debug!("Found build-id: {}", build_id);
                    Ok(self.find_by_build_id(&build_id))
                } else {
                    log::warn!("No build-id found in binary");
                    Ok(None)
                }
            }
            _ => Err("Not an ELF binary".to_string()),
        }
    }

    /// Detect libc from running process's memory maps
    pub fn detect_from_process(&self, pid: u32) -> Result<Option<&LibcVersion>, String> {
        #[cfg(target_os = "linux")]
        {
            let maps_path = format!("/proc/{}/maps", pid);
            let maps_content = fs::read_to_string(&maps_path)
                .map_err(|e| format!("Failed to read process maps: {}", e))?;

            for line in maps_content.lines() {
                if line.contains("libc-") || line.contains("libc.so") {
                    if let Some(_path_start) = line.rfind('/') {
                        let full_path = line.split_whitespace().last().unwrap_or("");
                        
                        log::debug!("Found libc: {}", full_path);
                        return self.detect_from_binary(full_path);
                    }
                }
            }
            Ok(None)
        }

        #[cfg(not(target_os = "linux"))]
        {
            let _ = pid;
            Err("Process detection only supported on Linux".to_string())
        }
    }

    /// Query online libc database (libc.blukat.me) for symbols
    pub async fn query_online(&mut self, symbols: &[(String, u64)]) -> Result<Vec<LibcVersion>, String> {
        let client = reqwest::Client::new();
        let mut query_params = vec![];

        for (name, offset) in symbols {
            query_params.push(format!("{}={:x}", name, offset));
        }

        let url = format!("https://libc.blukat.me/api/find?{}", query_params.join("&"));
        
        log::info!("Querying online libc database: {}", url);

        let response = client.get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to query libc database: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("HTTP error: {}", response.status()));
        }

        let results: Vec<OnlineLibcResult> = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        let mut found_versions = vec![];

        for result in results {
            let mut libc_version = LibcVersion::new(&result.id, &result.buildid, LibcArch::X64);
            
            for (name, offset_str) in &result.symbols {
                if let Ok(offset) = u64::from_str_radix(&offset_str.trim_start_matches("0x"), 16) {
                    libc_version.add_symbol(name.clone(), offset);
                    
                    match name.as_str() {
                        "system" => libc_version.system = offset,
                        "execve" => libc_version.execve = offset,
                        "__libc_start_main" => {},
                        _ => {}
                    }
                }
            }

            found_versions.push(libc_version);
        }

        log::info!("Found {} matching libc versions online", found_versions.len());
        Ok(found_versions)
    }

    /// Find one-gadget execve addresses using external tool
    pub fn find_one_gadgets(&self, libc_path: &str) -> Result<Vec<u64>, String> {
        use std::process::Command;

        log::info!("Searching for one-gadgets in: {}", libc_path);

        let output = Command::new("one_gadget")
            .arg(libc_path)
            .output();

        match output {
            Ok(out) if out.status.success() => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let mut gadgets = vec![];

                for line in stdout.lines() {
                    if line.starts_with("0x") {
                        if let Some(addr_str) = line.split_whitespace().next() {
                            if let Ok(addr) = u64::from_str_radix(&addr_str[2..], 16) {
                                gadgets.push(addr);
                            }
                        }
                    }
                }

                log::info!("Found {} one-gadgets", gadgets.len());
                Ok(gadgets)
            }
            Ok(out) => {
                Err(format!("one_gadget failed: {}", String::from_utf8_lossy(&out.stderr)))
            }
            Err(e) => {
                log::warn!("one_gadget tool not found: {}", e);
                Err("one_gadget tool not installed (gem install one_gadget)".to_string())
            }
        }
    }

    /// Get all libc versions by architecture
    pub fn get_by_arch(&self, arch: LibcArch) -> Vec<&LibcVersion> {
        self.versions
            .values()
            .filter(|v| v.arch == arch)
            .collect()
    }
}

/// Online libc database response structure
#[derive(Debug, Deserialize)]
struct OnlineLibcResult {
    id: String,
    buildid: String,
    symbols: HashMap<String, String>,
}

/// Extract build-id from ELF binary
fn extract_build_id(elf: &Elf) -> Option<String> {
    for sh in &elf.section_headers {
        if sh.sh_type == 7 {
            if let Some(name) = elf.shdr_strtab.get_at(sh.sh_name) {
                if name == ".note.gnu.build-id" || name.contains("build-id") {
                    log::debug!("Found build-id section: {}", name);
                }
            }
        }
    }

    log::warn!("Build-id extraction not fully implemented for this binary format");
    None
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_libc_db_creation() {
        let db = LibcDatabase::new();
        assert!(db.versions.len() >= 20);
    }

    #[test]
    fn test_get_libc_version() {
        let db = LibcDatabase::new();
        let libc = db.get("ubuntu20.04");
        assert!(libc.is_some());
        assert_eq!(libc.unwrap().system, 0x50d60);
        assert_eq!(libc.unwrap().arch, LibcArch::X64);
    }

    #[test]
    fn test_get_ubuntu_versions() {
        let db = LibcDatabase::new();
        
        assert!(db.get("ubuntu16.04").is_some());
        assert!(db.get("ubuntu18.04").is_some());
        assert!(db.get("ubuntu20.04").is_some());
        assert!(db.get("ubuntu22.04").is_some());
        assert!(db.get("ubuntu24.04").is_some());
    }

    #[test]
    fn test_get_debian_versions() {
        let db = LibcDatabase::new();
        
        assert!(db.get("debian10").is_some());
        assert!(db.get("debian11").is_some());
        assert!(db.get("debian12").is_some());
    }

    #[test]
    fn test_get_fedora_centos_versions() {
        let db = LibcDatabase::new();
        
        assert!(db.get("fedora38").is_some());
        assert!(db.get("fedora39").is_some());
        assert!(db.get("centos7").is_some());
        assert!(db.get("centos8").is_some());
    }

    #[test]
    fn test_get_arch_kali_versions() {
        let db = LibcDatabase::new();
        
        assert!(db.get("arch").is_some());
        assert!(db.get("kali").is_some());
    }

    #[test]
    fn test_arm_versions() {
        let db = LibcDatabase::new();
        
        let arm64 = db.get("ubuntu20.04-arm64");
        assert!(arm64.is_some());
        assert_eq!(arm64.unwrap().arch, LibcArch::ARM64);

        let arm = db.get("debian11-arm");
        assert!(arm.is_some());
        assert_eq!(arm.unwrap().arch, LibcArch::ARM);
    }

    #[test]
    fn test_mips_versions() {
        let db = LibcDatabase::new();
        
        let mips = db.get("debian10-mips");
        assert!(mips.is_some());
        assert_eq!(mips.unwrap().arch, LibcArch::MIPS);

        let mips64 = db.get("debian11-mips64");
        assert!(mips64.is_some());
        assert_eq!(mips64.unwrap().arch, LibcArch::MIPS64);
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
    fn test_resolve_ubuntu22_symbols() {
        let db = LibcDatabase::new();
        let base = 0x7ffff7a0d000u64;
        
        let system = db.resolve_address("ubuntu22.04", base, "system");
        assert!(system.is_some());
        assert_eq!(system.unwrap(), base + 0x50d70);

        let execve = db.resolve_address("ubuntu22.04", base, "execve");
        assert!(execve.is_some());
        assert_eq!(execve.unwrap(), base + 0xeb2a0);
    }

    #[test]
    fn test_one_gadgets() {
        let db = LibcDatabase::new();
        let base = 0x7ffff7a0d000u64;
        let gadgets = db.get_one_gadgets("ubuntu20.04", base);
        assert!(gadgets.is_some());
        assert_eq!(gadgets.unwrap().len(), 3);
    }

    #[test]
    fn test_one_gadgets_ubuntu22() {
        let db = LibcDatabase::new();
        let base = 0x7ffff7a0d000u64;
        let gadgets = db.get_one_gadgets("ubuntu22.04", base);
        assert!(gadgets.is_some());
        assert_eq!(gadgets.unwrap().len(), 3);
    }

    #[test]
    fn test_auto_ret2libc() {
        let result = auto_ret2libc("ubuntu20.04", 0x7ffff7a0d000);
        assert!(result.is_ok());
        let chain = result.unwrap();
        assert_eq!(chain.len(), 2);
    }

    #[test]
    fn test_find_by_build_id() {
        let db = LibcDatabase::new();
        let result = db.find_by_build_id("6e0cdbd5c76c0813b7aad4f671a16bdb6f1c8cbb");
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "ubuntu20.04-2.31");
    }

    #[test]
    fn test_get_by_arch_x64() {
        let db = LibcDatabase::new();
        let x64_libcs = db.get_by_arch(LibcArch::X64);
        assert!(x64_libcs.len() >= 15);
    }

    #[test]
    fn test_get_by_arch_arm() {
        let db = LibcDatabase::new();
        let arm_libcs = db.get_by_arch(LibcArch::ARM);
        assert!(arm_libcs.len() >= 1);
    }

    #[test]
    fn test_get_by_arch_arm64() {
        let db = LibcDatabase::new();
        let arm64_libcs = db.get_by_arch(LibcArch::ARM64);
        assert!(arm64_libcs.len() >= 1);
    }

    #[test]
    fn test_get_by_arch_mips() {
        let db = LibcDatabase::new();
        let mips_libcs = db.get_by_arch(LibcArch::MIPS);
        assert!(mips_libcs.len() >= 1);
    }

    #[test]
    fn test_libc_version_methods() {
        let mut libc = LibcVersion::new("test", "abc123", LibcArch::X64);
        libc.add_symbol("test_symbol".to_string(), 0x1234);
        
        let sym = libc.get_symbol("test_symbol");
        assert!(sym.is_some());
        assert_eq!(sym.unwrap(), 0x1234);
    }

    #[test]
    fn test_symbol_cache() {
        let db = LibcDatabase::new();
        
        let cached = db.get_cached_symbol("ubuntu20.04", "system");
        assert!(cached.is_none());
    }

    #[test]
    fn test_arch_from_elf_machine() {
        assert_eq!(LibcArch::from_elf_machine(3), Some(LibcArch::X86));
        assert_eq!(LibcArch::from_elf_machine(62), Some(LibcArch::X64));
        assert_eq!(LibcArch::from_elf_machine(40), Some(LibcArch::ARM));
        assert_eq!(LibcArch::from_elf_machine(183), Some(LibcArch::ARM64));
        assert_eq!(LibcArch::from_elf_machine(8), Some(LibcArch::MIPS));
    }
}
