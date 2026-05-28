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

/// A backend that queries both SQLite and PostgreSQL in parallel.
pub struct MultiBackend {
    sqlite: Option<sqlite::SqliteBackend>,
    postgres: Option<postgres::PostgresBackend>,
}

impl DictBackend for MultiBackend {
    fn lookup(&self, word: &str) -> Result<Vec<DictEntry>, CoreError> {
        let mut results = Vec::new();
        if let Some(ref be) = self.sqlite {
            if let Ok(entries) = be.lookup(word) {
                results.extend(entries);
            }
        }
        if let Some(ref be) = self.postgres {
            if let Ok(entries) = be.lookup(word) {
                results.extend(entries);
            }
        }
        Ok(results)
    }

    fn lookup_batch(&self, words: &[&str]) -> HashMap<String, Vec<DictEntry>> {
        let mut combined = HashMap::new();
        if let Some(ref be) = self.sqlite {
            let found = be.lookup_batch(words);
            for (k, v) in found {
                combined.entry(k).or_insert_with(Vec::new).extend(v);
            }
        }
        if let Some(ref be) = self.postgres {
            let found = be.lookup_batch(words);
            for (k, v) in found {
                combined.entry(k).or_insert_with(Vec::new).extend(v);
            }
        }
        combined
    }
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
        let config = crate::GlobalConfig::load().unwrap_or_default();
        let backend = std::env::var("DICT_BACKEND").unwrap_or_else(|_| {
            // Auto-detect: if both paths are set, use "both"
            if !config.dict_path.is_empty() && !config.postgres_url.is_empty() {
                "both".into()
            } else if !config.postgres_url.is_empty() {
                "postgres".into()
            } else {
                "sqlite".into()
            }
        });
        DictConfig {
            backend,
            sqlite_path: std::env::var("DICT_PATH")
                .unwrap_or(config.dict_path),
            postgres_url: std::env::var("DATABASE_URL")
                .unwrap_or(config.postgres_url),
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
        "both" => {
            let sql = if !config.sqlite_path.is_empty() {
                Some(sqlite::SqliteBackend::open(&config.sqlite_path)?)
            } else { None };
            let pg = if !config.postgres_url.is_empty() {
                Some(postgres::PostgresBackend::open(&config.postgres_url)?)
            } else { None };
            Ok(Box::new(MultiBackend { sqlite: sql, postgres: pg }))
        }
        other => Err(CoreError::ConfigError(format!(
            "unknown dictionary backend: '{}'. Use 'sqlite', 'postgres', or 'both'",
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
