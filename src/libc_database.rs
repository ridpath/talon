use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

const LIBC_RIP_API: &str = "https://libc.rip/api";
const LIBC_BLUKAT_API: &str = "https://libc.blukat.me/d";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibcMatch {
    pub id: String,
    pub md5: String,
    pub sha1: String,
    pub sha256: String,
    pub download_url: String,
    pub symbols: HashMap<String, u64>,
}

#[derive(Debug, Clone)]
pub struct LibcDatabase {
    cache_dir: PathBuf,
}

impl LibcDatabase {
    pub fn new() -> Result<Self, String> {
        let home = directories::UserDirs::new()
            .ok_or("Could not find home directory")?
            .home_dir()
            .to_path_buf();
        let cache_dir = home.join(".talon").join("libc");

        fs::create_dir_all(&cache_dir)
            .map_err(|e| format!("Failed to create libc cache directory: {}", e))?;

        Ok(LibcDatabase { cache_dir })
    }

    pub fn search(&self, symbols: HashMap<String, u64>) -> Result<Vec<LibcMatch>, String> {
        if symbols.is_empty() {
            return Err("No symbols provided for search".to_string());
        }

        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        let url = format!("{}/find", LIBC_RIP_API);

        let response = client
            .post(&url)
            .json(&symbols)
            .send()
            .map_err(|e| format!("Failed to query libc.rip: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("libc.rip returned status: {}", response.status()));
        }

        let results: Vec<LibcRipResult> = response
            .json()
            .map_err(|e| format!("Failed to parse libc.rip response: {}", e))?;

        let matches: Vec<LibcMatch> = results
            .into_iter()
            .map(|r| LibcMatch {
                id: r.id.clone(),
                md5: r.md5.clone(),
                sha1: r.sha1.clone(),
                sha256: r.sha256.clone(),
                download_url: r.download_url.clone(),
                symbols: r.symbols.clone(),
            })
            .collect();

        Ok(matches)
    }

    pub fn search_one(&self, symbol: &str, address: u64) -> Result<Vec<LibcMatch>, String> {
        let mut symbols = HashMap::new();

        let offset = address & 0xFFF;
        symbols.insert(symbol.to_string(), offset);

        self.search(symbols)
    }

    pub fn download(&self, libc_match: &LibcMatch) -> Result<PathBuf, String> {
        let filename = format!("libc6_{}.so", libc_match.id);
        let local_path = self.cache_dir.join(&filename);

        if local_path.exists() {
            log::info!("Libc already cached: {}", local_path.display());
            return Ok(local_path);
        }

        log::info!("Downloading libc from: {}", libc_match.download_url);

        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        let response = client
            .get(&libc_match.download_url)
            .send()
            .map_err(|e| format!("Failed to download libc: {}", e))?;

        if !response.status().is_success() {
            return Err(format!(
                "Download failed with status: {}",
                response.status()
            ));
        }

        let bytes = response
            .bytes()
            .map_err(|e| format!("Failed to read response bytes: {}", e))?;

        fs::write(&local_path, &bytes)
            .map_err(|e| format!("Failed to write libc to disk: {}", e))?;

        log::info!("Libc downloaded to: {}", local_path.display());
        Ok(local_path)
    }

    pub fn get_symbols(&self, libc_path: &str) -> Result<HashMap<String, u64>, String> {
        use goblin::elf::Elf;

        let data = fs::read(libc_path).map_err(|e| format!("Failed to read libc file: {}", e))?;

        let elf = Elf::parse(&data).map_err(|e| format!("Failed to parse ELF: {}", e))?;

        let mut symbols = HashMap::new();

        for sym in &elf.syms {
            if sym.st_value > 0 {
                if let Some(name) = elf.strtab.get_at(sym.st_name) {
                    symbols.insert(name.to_string(), sym.st_value);
                }
            }
        }

        Ok(symbols)
    }

    pub fn find_symbol(&self, libc_path: &str, symbol_name: &str) -> Result<u64, String> {
        let symbols = self.get_symbols(libc_path)?;

        symbols
            .get(symbol_name)
            .copied()
            .ok_or_else(|| format!("Symbol '{}' not found in libc", symbol_name))
    }
}

#[derive(Debug, Deserialize)]
struct LibcRipResult {
    id: String,
    md5: String,
    sha1: String,
    sha256: String,
    download_url: String,
    symbols: HashMap<String, u64>,
}

pub fn libc_search(symbol: &str, leaked_addr: u64) -> Result<Vec<LibcMatch>, String> {
    let db = LibcDatabase::new()?;
    db.search_one(symbol, leaked_addr)
}

pub fn libc_search_multi(symbols: HashMap<String, u64>) -> Result<Vec<LibcMatch>, String> {
    let db = LibcDatabase::new()?;
    db.search(symbols)
}

pub fn libc_download(id_or_url: &str) -> Result<PathBuf, String> {
    let db = LibcDatabase::new()?;

    if id_or_url.starts_with("http://") || id_or_url.starts_with("https://") {
        let libc_match = LibcMatch {
            id: "custom".to_string(),
            md5: String::new(),
            sha1: String::new(),
            sha256: String::new(),
            download_url: id_or_url.to_string(),
            symbols: HashMap::new(),
        };
        db.download(&libc_match)
    } else {
        Err("Please use libc_search first to get download URL".to_string())
    }
}

pub fn libc_symbols(libc_path: &str) -> Result<HashMap<String, u64>, String> {
    let db = LibcDatabase::new()?;
    db.get_symbols(libc_path)
}

pub fn libc_symbol(libc_path: &str, symbol_name: &str) -> Result<u64, String> {
    let db = LibcDatabase::new()?;
    db.find_symbol(libc_path, symbol_name)
}

pub fn libc_offset(libc_path: &str, symbol1: &str, symbol2: &str) -> Result<i64, String> {
    let db = LibcDatabase::new()?;
    let addr1 = db.find_symbol(libc_path, symbol1)?;
    let addr2 = db.find_symbol(libc_path, symbol2)?;
    Ok((addr2 as i64) - (addr1 as i64))
}
