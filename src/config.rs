use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TalonConfig {
    #[serde(default = "default_lm_studio_url")]
    pub lm_studio_url: String,

    #[serde(default = "default_lm_studio_model")]
    pub lm_studio_model: String,

    #[serde(default = "default_verbosity")]
    pub verbosity: String,

    #[serde(default)]
    pub enable_colors: bool,

    #[serde(default)]
    pub enable_progress_bars: bool,

    #[serde(default = "default_exploit_db_url")]
    pub exploit_db_url: String,

    #[serde(default)]
    pub default_arch: String,

    #[serde(default)]
    pub default_os: String,

    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
}

fn default_lm_studio_url() -> String {
    "http://localhost:1234".to_string()
}

fn default_lm_studio_model() -> String {
    "local-model".to_string()
}

fn default_verbosity() -> String {
    "normal".to_string()
}

fn default_exploit_db_url() -> String {
    "https://www.exploit-db.com/exploits/".to_string()
}

fn default_timeout() -> u64 {
    300
}

impl Default for TalonConfig {
    fn default() -> Self {
        TalonConfig {
            lm_studio_url: default_lm_studio_url(),
            lm_studio_model: default_lm_studio_model(),
            verbosity: default_verbosity(),
            enable_colors: true,
            enable_progress_bars: true,
            exploit_db_url: default_exploit_db_url(),
            default_arch: "x86_64".to_string(),
            default_os: "linux".to_string(),
            timeout_seconds: default_timeout(),
        }
    }
}

impl TalonConfig {
    pub fn load() -> Self {
        if let Some(config_path) = Self::get_config_path() {
            if config_path.exists() {
                if let Ok(content) = fs::read_to_string(&config_path) {
                    if let Ok(config) = toml::from_str::<TalonConfig>(&content) {
                        log::info!("Loaded config from {:?}", config_path);
                        return config;
                    }
                }
            }
        }

        log::debug!("Using default configuration");
        TalonConfig::default()
    }

    pub fn save(&self) -> Result<(), String> {
        let config_path = Self::get_config_path().ok_or("Could not determine config directory")?;

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }

        let toml_str = toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        fs::write(&config_path, toml_str).map_err(|e| format!("Failed to write config: {}", e))?;

        log::info!("Saved config to {:?}", config_path);
        Ok(())
    }

    pub fn get_config_path() -> Option<PathBuf> {
        if let Some(proj_dirs) = ProjectDirs::from("", "", "talon") {
            Some(proj_dirs.config_dir().join("config.toml"))
        } else {
            None
        }
    }

    pub fn print_config_location() {
        if let Some(path) = Self::get_config_path() {
            println!("Config file: {}", path.display());
        } else {
            println!("Config file: <platform-specific location not found>");
        }
    }
}
