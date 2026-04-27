//! Configuration module.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// WTK configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub tracking: TrackingConfig,

    #[serde(default)]
    pub display: DisplayConfig,

    #[serde(default)]
    pub filters: FiltersConfig,

    #[serde(default)]
    pub hooks: HooksConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            tracking: TrackingConfig::default(),
            display: DisplayConfig::default(),
            filters: FiltersConfig::default(),
            hooks: HooksConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,

    #[serde(default = "default_history_days")]
    pub history_days: u32,

    /// Token counting strategy: "bytes" (default, fast) or "cl100k" (real BPE, +5-30ms/call).
    /// Can be overridden by `WTK_TOKENIZER` env var or per-command CLI flag.
    #[serde(default = "default_tokenizer")]
    pub tokenizer: String,
}

impl Default for TrackingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            history_days: 90,
            tokenizer: default_tokenizer(),
        }
    }
}

fn default_tokenizer() -> String {
    "bytes".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    #[serde(default = "default_true")]
    pub colors: bool,

    #[serde(default = "default_max_width")]
    pub max_width: usize,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            colors: true,
            max_width: 120,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiltersConfig {
    #[serde(default)]
    pub ignore_dirs: Vec<String>,

    #[serde(default)]
    pub ignore_files: Vec<String>,
}

impl Default for FiltersConfig {
    fn default() -> Self {
        Self {
            ignore_dirs: vec![
                ".git".to_string(),
                "node_modules".to_string(),
                "target".to_string(),
            ],
            ignore_files: vec!["*.lock".to_string()],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HooksConfig {
    #[serde(default)]
    pub claude_code: bool,

    #[serde(default)]
    pub powershell: bool,

    #[serde(default)]
    pub cmd: bool,
}

impl Default for HooksConfig {
    fn default() -> Self {
        Self {
            claude_code: false,
            powershell: false,
            cmd: false,
        }
    }
}

// Default value functions
fn default_true() -> bool {
    true
}

fn default_history_days() -> u32 {
    90
}

fn default_max_width() -> usize {
    120
}

/// Load configuration from file.
pub fn load() -> Result<Config> {
    let config_path = get_config_path()?;

    if config_path.exists() {
        let content = fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config from {:?}", config_path))?;
        let config: Config =
            toml::from_str(&content).with_context(|| "Failed to parse config file")?;
        Ok(config)
    } else {
        Ok(Config::default())
    }
}

/// Save configuration to file.
pub fn save(config: &Config) -> Result<()> {
    let config_path = get_config_path()?;

    // Create directory if needed
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let content = toml::to_string_pretty(config)?;
    fs::write(&config_path, content)
        .with_context(|| format!("Failed to write config to {:?}", config_path))?;

    Ok(())
}

fn get_config_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir().context("Could not find config directory")?;
    Ok(config_dir.join("wtk").join("config.toml"))
}
