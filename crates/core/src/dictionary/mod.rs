pub mod postgres;
pub mod sqlite;

use std::collections::HashMap;

use crate::error::CoreError;

/// A single morphological entry from OpenCorpora.
#[derive(Debug, Clone)]
pub struct DictEntry {
    pub form: String,
    pub lemma: String,
    pub pos: Option<String>,
    pub grammemes: Vec<Grammeme>,
}

#[derive(Debug, Clone)]
pub struct Grammeme {
    pub code: String,
    pub name: String,
    pub alias: String,
}

/// Common interface for dictionary backends.
pub trait DictBackend: Send + Sync {
    fn lookup(&self, word: &str) -> Result<Vec<DictEntry>, CoreError>;
    fn lookup_batch(&self, words: &[&str]) -> HashMap<String, Vec<DictEntry>>;
}

/// Dictionary configuration from config.json.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Default)]
pub struct DictConfig {
    #[serde(default = "default_backend")]
    pub backend: String,
    #[serde(default = "default_sqlite_path")]
    pub sqlite_path: String,
    #[serde(default)]
    pub postgres_url: String,
}

fn default_backend() -> String { "sqlite".into() }
fn default_sqlite_path() -> String { ".data/dict.opcorpora.sqlite3.db".into() }

impl DictConfig {
    pub fn from_env_or_default() -> Self {
        let backend = std::env::var("DICT_BACKEND").unwrap_or_else(|_| "sqlite".into());
        DictConfig {
            backend: backend.clone(),
            sqlite_path: std::env::var("DICT_PATH")
                .unwrap_or_else(|_| default_sqlite_path()),
            postgres_url: std::env::var("DATABASE_URL").unwrap_or_default(),
        }
    }
}

/// Open a dictionary backend from configuration.
pub fn open_backend(config: &DictConfig) -> Result<Box<dyn DictBackend>, CoreError> {
    match config.backend.as_str() {
        "sqlite" => {
            let backend = sqlite::SqliteBackend::open(&config.sqlite_path)?;
            Ok(Box::new(backend))
        }
        "postgres" => {
            let backend = postgres::PostgresBackend::open(&config.postgres_url)?;
            Ok(Box::new(backend))
        }
        other => Err(CoreError::ConfigError(format!(
            "unknown dictionary backend: '{}'. Use 'sqlite' or 'postgres'",
            other
        ))),
    }
}

pub(crate) fn is_pos(code: &str) -> bool {
    matches!(code,
        "NOUN" | "VERB" | "ADJF" | "ADJS" | "ADVB" | "COMP" |
        "PRTF" | "PRTS" | "GRND" | "INFN" | "PRED" |
        "PREP" | "CONJ" | "PRCL" | "INTJ" | "NUMR" | "NPRO" | "ADV"
    )
}
