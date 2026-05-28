use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Default values used when no config file exists.
pub const DEFAULT_LANGUAGE: &str = "ru";
pub const DEFAULT_MODEL_DIR: &str = "./models";
pub const DEFAULT_LOG_LEVEL: &str = "info";
pub const DEFAULT_CHUNK_SIZE: usize = 10;
pub const DEFAULT_DICT_PATH: &str = ".data/dict.opcorpora.sqlite3.db";
pub const DEFAULT_PG_URL: &str = "";

/// Global configuration from `~/.config/text-researcher/config.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    #[serde(default = "default_language")]
    pub default_language: String,
    #[serde(default = "default_model_dir")]
    pub model_dir: String,
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default = "default_chunk_size")]
    pub batch_chunk_size: usize,
    #[serde(default = "default_dict_path")]
    pub dict_path: String,
    #[serde(default = "default_pg_url")]
    pub postgres_url: String,
}

fn default_language() -> String { DEFAULT_LANGUAGE.into() }
fn default_model_dir() -> String { DEFAULT_MODEL_DIR.into() }
fn default_log_level() -> String { DEFAULT_LOG_LEVEL.into() }
fn default_chunk_size() -> usize { DEFAULT_CHUNK_SIZE }
fn default_dict_path() -> String { DEFAULT_DICT_PATH.into() }
fn default_pg_url() -> String { DEFAULT_PG_URL.into() }

impl Default for GlobalConfig {
    fn default() -> Self {
        GlobalConfig {
            default_language: DEFAULT_LANGUAGE.into(),
            model_dir: DEFAULT_MODEL_DIR.into(),
            log_level: DEFAULT_LOG_LEVEL.into(),
            batch_chunk_size: DEFAULT_CHUNK_SIZE,
            dict_path: DEFAULT_DICT_PATH.into(),
            postgres_url: DEFAULT_PG_URL.into(),
        }
    }
}

impl GlobalConfig {
    /// Load global config from the standard location.
    pub fn load() -> Result<Self, crate::error::CoreError> {
        let path = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("text-researcher")
            .join("config.json");

        if path.exists() {
            let content = std::fs::read_to_string(&path)
                .map_err(|e| crate::error::CoreError::ConfigError(format!(
                    "failed to read {}: {}", path.display(), e
                )))?;
            serde_json::from_str(&content)
                .map_err(|e| crate::error::CoreError::ConfigError(format!(
                    "failed to parse {}: {}", path.display(), e
                )))
        } else {
            Ok(GlobalConfig::default())
        }
    }
}

/// Session-level configuration from `.trconf` next to the analyzed file.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_cursor_position: Option<CursorPosition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui_preferences: Option<UiPreferences>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorPosition {
    pub line: usize,
    pub word_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiPreferences {
    #[serde(default)]
    pub theme: String,
}

impl SessionConfig {
    /// Load session config from a `.trconf` file.
    /// Returns default if the file doesn't exist.
    pub fn load(path: &std::path::Path) -> Result<Self, crate::error::CoreError> {
        if !path.exists() {
            return Ok(SessionConfig::default());
        }

        let content = std::fs::read_to_string(path)
            .map_err(|e| crate::error::CoreError::ConfigError(format!(
                "failed to read {}: {}", path.display(), e
            )))?;

        // Try JSON first, then YAML
        serde_json::from_str(&content)
            .or_else(|_| {
                serde_yaml::from_str(&content).map_err(|e| {
                    crate::error::CoreError::ConfigError(format!(
                        "failed to parse {} as JSON or YAML: {}", path.display(), e
                    ))
                })
            })
    }

    /// Resolve language: session → global → default.
    pub fn resolve_language(&self, global: &GlobalConfig) -> String {
        self.language.clone().unwrap_or_else(|| global.default_language.clone())
    }
}

/// Build the config directory path, creating it if needed.
pub fn ensure_config_dir() -> Result<PathBuf, crate::error::CoreError> {
    let dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("text-researcher");
    std::fs::create_dir_all(&dir)
        .map_err(|e| crate::error::CoreError::ConfigError(format!(
            "failed to create config dir: {}", e
        )))?;
    Ok(dir)
}
