use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct BuildCache {
    pub cache_dir: PathBuf,
}

impl BuildCache {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let home_dir = dirs::home_dir().ok_or("Unable to determine home directory")?;
        let cache_dir = home_dir.join(".talon_cache");

        if !cache_dir.exists() {
            fs::create_dir_all(&cache_dir)?;
            Self::set_permissions(&cache_dir)?;
            Self::create_gitignore(&cache_dir)?;
        }

        Ok(BuildCache { cache_dir })
    }

    #[cfg(unix)]
    fn set_permissions(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        use std::os::unix::fs::PermissionsExt;
        let permissions = fs::Permissions::from_mode(0o700);
        fs::set_permissions(path, permissions)?;
        Ok(())
    }

    #[cfg(not(unix))]
    fn set_permissions(_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn create_gitignore(cache_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let gitignore_path = cache_dir.join(".gitignore");
        if !gitignore_path.exists() {
            fs::write(gitignore_path, "*\n")?;
        }
        Ok(())
    }

    pub fn compute_hash(cargo_toml: &str, main_rs: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(cargo_toml.as_bytes());
        hasher.update(main_rs.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn get_cached_binary_path(&self, hash: &str) -> PathBuf {
        self.cache_dir.join(format!("{}.bin", hash))
    }

    pub fn get_cached_metadata_path(&self, hash: &str) -> PathBuf {
        self.cache_dir.join(format!("{}.meta", hash))
    }

    pub fn check_cache(&self, hash: &str) -> bool {
        let binary_path = self.get_cached_binary_path(hash);
        let metadata_path = self.get_cached_metadata_path(hash);
        binary_path.exists() && metadata_path.exists()
    }

    pub fn store_cache(
        &self,
        hash: &str,
        binary_path: &Path,
        cargo_toml: &str,
        main_rs: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let cached_binary = self.get_cached_binary_path(hash);
        let cached_metadata = self.get_cached_metadata_path(hash);

        fs::copy(binary_path, &cached_binary)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let permissions = fs::Permissions::from_mode(0o755);
            fs::set_permissions(&cached_binary, permissions)?;
        }

        let metadata = CacheMetadata {
            hash: hash.to_string(),
            cargo_toml: cargo_toml.to_string(),
            main_rs: main_rs.to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        let metadata_json = serde_json::to_string_pretty(&metadata)?;
        fs::write(cached_metadata, metadata_json)?;

        Ok(())
    }

    pub fn retrieve_cache(
        &self,
        hash: &str,
        output_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let cached_binary = self.get_cached_binary_path(hash);

        if !cached_binary.exists() {
            return Err("Cache entry not found".into());
        }

        fs::copy(&cached_binary, output_path)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let permissions = fs::Permissions::from_mode(0o755);
            fs::set_permissions(output_path, permissions)?;
        }

        Ok(())
    }

    pub fn clean_old_entries(&self, max_age_days: u64) -> Result<usize, Box<dyn std::error::Error>> {
        let max_age_secs = max_age_days * 24 * 60 * 60;
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut cleaned = 0;

        for entry in fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("meta") {
                if let Ok(metadata_content) = fs::read_to_string(&path) {
                    if let Ok(metadata) = serde_json::from_str::<CacheMetadata>(&metadata_content) {
                        if current_time - metadata.timestamp > max_age_secs {
                            let hash = &metadata.hash;
                            let binary_path = self.get_cached_binary_path(hash);
                            
                            if binary_path.exists() {
                                fs::remove_file(binary_path)?;
                            }
                            fs::remove_file(&path)?;
                            cleaned += 1;
                        }
                    }
                }
            }
        }

        Ok(cleaned)
    }

    pub fn get_cache_stats(&self) -> Result<CacheStats, Box<dyn std::error::Error>> {
        let mut total_entries = 0;
        let mut total_size = 0u64;

        for entry in fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("bin") {
                total_entries += 1;
                if let Ok(metadata) = fs::metadata(&path) {
                    total_size += metadata.len();
                }
            }
        }

        Ok(CacheStats {
            total_entries,
            total_size_bytes: total_size,
        })
    }
}

impl Default for BuildCache {
    fn default() -> Self {
        Self::new().expect("Failed to initialize build cache")
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct CacheMetadata {
    hash: String,
    cargo_toml: String,
    main_rs: String,
    timestamp: u64,
}

#[derive(Debug)]
pub struct CacheStats {
    pub total_entries: usize,
    pub total_size_bytes: u64,
}

impl CacheStats {
    pub fn total_size_mb(&self) -> f64 {
        self.total_size_bytes as f64 / (1024.0 * 1024.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_computation() {
        let cargo_toml = "[package]\nname = \"test\"\n";
        let main_rs = "fn main() {}\n";
        
        let hash1 = BuildCache::compute_hash(cargo_toml, main_rs);
        let hash2 = BuildCache::compute_hash(cargo_toml, main_rs);
        
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64);
    }

    #[test]
    fn test_hash_changes_with_content() {
        let cargo_toml = "[package]\nname = \"test\"\n";
        let main_rs1 = "fn main() {}\n";
        let main_rs2 = "fn main() { println!(\"hello\"); }\n";
        
        let hash1 = BuildCache::compute_hash(cargo_toml, main_rs1);
        let hash2 = BuildCache::compute_hash(cargo_toml, main_rs2);
        
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_cache_initialization() {
        let cache = BuildCache::new();
        assert!(cache.is_ok());
        
        let cache = cache.unwrap();
        assert!(cache.cache_dir.exists());
        
        let gitignore = cache.cache_dir.join(".gitignore");
        assert!(gitignore.exists());
    }

    #[test]
    fn test_cache_paths() {
        let cache = BuildCache::new().unwrap();
        let hash = "abc123";
        
        let binary_path = cache.get_cached_binary_path(hash);
        let metadata_path = cache.get_cached_metadata_path(hash);
        
        assert!(binary_path.to_string_lossy().contains("abc123.bin"));
        assert!(metadata_path.to_string_lossy().contains("abc123.meta"));
    }

    #[test]
    fn test_cache_stats() {
        let cache = BuildCache::new().unwrap();
        let stats = cache.get_cache_stats();
        assert!(stats.is_ok());
    }
}
