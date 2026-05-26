use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use log::debug;
use rayon::prelude::*;
use rusqlite::Connection;

use crate::error::CoreError;

/// A single morphological entry from OpenCorpora.
#[derive(Debug, Clone)]
pub struct DictEntry {
    pub form: String,
    pub lemma: String,
    pub pos: Option<String>,           // часть речи (NOUN, VERB, etc.)
    pub grammemes: Vec<Grammeme>,       // граммемы формы + леммы
}

#[derive(Debug, Clone)]
pub struct Grammeme {
    pub code: String,   // e.g., "sing", "nomn"
    pub name: String,   // e.g., "единственное число"
    pub alias: String,  // e.g., "ед"
}

/// OpenCorpora dictionary backed by SQLite.
pub struct OpenCorporaDict {
    conn: Mutex<Connection>,
    path: PathBuf,
}

impl OpenCorporaDict {
    /// Open the dictionary database at the given path.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, CoreError> {
        let path = path.as_ref().to_path_buf();
        let conn = Connection::open(&path)
            .map_err(|e| CoreError::DictionaryError(format!("failed to open dictionary: {}", e)))?;

        // Enable WAL for concurrent reads
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=OFF;")
            .map_err(|e| CoreError::DictionaryError(format!("pragma failed: {}", e)))?;

        debug!("OpenCorpora dictionary opened: {}", path.display());
        Ok(OpenCorporaDict { conn: Mutex::new(conn), path })
    }

    /// Return the database file path.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Look up all entries for a word form.
    pub fn lookup(&self, word: &str) -> Result<Vec<DictEntry>, CoreError> {
        let word_clean = word.to_lowercase();
        let word_clean = word_clean.trim_matches(|c: char| !c.is_alphanumeric() && c != '-');
        let conn = self.conn.lock().unwrap();
        self.lookup_impl(&conn, word_clean)
    }

    /// Look up multiple words in parallel using all CPU cores.
    ///
    /// Each thread opens its own SQLite connection for concurrent reads.
    /// Returns a map from word → entries.
    pub fn lookup_batch(&self, words: &[&str]) -> HashMap<String, Vec<DictEntry>> {
        let results: Mutex<HashMap<String, Vec<DictEntry>>> = Mutex::new(HashMap::new());

        words.par_iter().for_each(|word| {
            let word_clean = word.to_lowercase();
            let word_clean: String = word_clean
                .trim_matches(|c: char| !c.is_alphanumeric() && c != '-')
                .to_string();

            if word_clean.is_empty() {
                return;
            }

            // Open per-thread connection
            if let Ok(conn) = Connection::open(&self.path) {
                let _ = conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=OFF;");
                if let Ok(entries) = self.lookup_impl(&conn, &word_clean) {
                    if !entries.is_empty() {
                        if let Ok(mut map) = results.lock() {
                            map.insert((*word).to_string(), entries);
                        }
                    }
                }
            }
        });

        results.into_inner().unwrap_or_default()
    }

    /// Internal lookup using a specific connection.
    fn lookup_impl(&self, conn: &Connection, word: &str) -> Result<Vec<DictEntry>, CoreError> {
        let mut stmt = conn.prepare(
            "SELECT f.id, f.text, f.lemma_id, l.text
             FROM forms f
             JOIN lemmata l ON f.lemma_id = l.id
             WHERE f.text = ?1
             LIMIT 20"
        ).map_err(|e| CoreError::DictionaryError(format!("prepare failed: {}", e)))?;

        let rows: Vec<(i64, String, i64, String)> = stmt
            .query_map([word], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })
            .map_err(|e| CoreError::DictionaryError(format!("query failed: {}", e)))?
            .filter_map(|r| r.ok())
            .collect();

        if rows.is_empty() {
            return Ok(Vec::new());
        }

        let mut entries = Vec::new();
        let mut gram_cache: HashMap<String, Grammeme> = HashMap::new();

        for (form_id, form_text, lemma_id, lemma_text) in rows {
            let form_grammemes = self.get_form_grammemes(conn, form_id, &mut gram_cache)?;
            let lemma_grammemes = self.get_lemma_grammemes(conn, lemma_id, &mut gram_cache)?;

            let pos = lemma_grammemes.iter()
                .find(|g| is_pos(&g.code))
                .map(|g| g.code.clone());

            let mut all_grammemes = Vec::new();
            all_grammemes.extend(lemma_grammemes.into_iter().filter(|g| !is_pos(&g.code)));
            all_grammemes.extend(form_grammemes);

            entries.push(DictEntry {
                form: form_text,
                lemma: lemma_text,
                pos,
                grammemes: all_grammemes,
            });
        }

        Ok(entries)
    }

    /// Get grammemes for a form, resolving codes to names.
    fn get_form_grammemes(&self, conn: &Connection, form_id: i64, cache: &mut HashMap<String, Grammeme>) -> Result<Vec<Grammeme>, CoreError> {
        let mut stmt = conn.prepare(
            "SELECT grammeme_v FROM form_grammemes WHERE form_id = ?1"
        ).map_err(|e| CoreError::DictionaryError(format!("prepare failed: {}", e)))?;

        let codes: Vec<String> = stmt
            .query_map([form_id], |row| row.get(0))
            .map_err(|e| CoreError::DictionaryError(format!("query failed: {}", e)))?
            .filter_map(|r| r.ok())
            .collect();

        self.resolve_grammemes(conn, &codes, cache)
    }

    /// Get grammemes for a lemma.
    fn get_lemma_grammemes(&self, conn: &Connection, lemma_id: i64, cache: &mut HashMap<String, Grammeme>) -> Result<Vec<Grammeme>, CoreError> {
        let mut stmt = conn.prepare(
            "SELECT grammeme_v FROM lemma_grammemes WHERE lemma_id = ?1"
        ).map_err(|e| CoreError::DictionaryError(format!("prepare failed: {}", e)))?;

        let codes: Vec<String> = stmt
            .query_map([lemma_id], |row| row.get(0))
            .map_err(|e| CoreError::DictionaryError(format!("query failed: {}", e)))?
            .filter_map(|r| r.ok())
            .collect();

        self.resolve_grammemes(conn, &codes, cache)
    }

    /// Resolve grammeme codes to names, using cache.
    fn resolve_grammemes(&self, conn: &Connection, codes: &[String], cache: &mut HashMap<String, Grammeme>) -> Result<Vec<Grammeme>, CoreError> {
        let mut result = Vec::new();
        let mut missing: Vec<&str> = Vec::new();

        for code in codes {
            if let Some(g) = cache.get(code) {
                result.push(g.clone());
            } else {
                missing.push(code);
            }
        }

        if !missing.is_empty() {
            let placeholders: Vec<String> = missing.iter().enumerate()
                .map(|(i, _)| format!("?{}", i + 1))
                .collect();
            let sql = format!(
                "SELECT name, alias, description FROM grammemes WHERE name IN ({})",
                placeholders.join(",")
            );

            let mut stmt = conn.prepare(&sql)
                .map_err(|e| CoreError::DictionaryError(format!("prepare failed: {}", e)))?;

            let params: Vec<&dyn rusqlite::types::ToSql> = missing.iter()
                .map(|s| s as &dyn rusqlite::types::ToSql)
                .collect();

            let resolved: Vec<(String, Grammeme)> = stmt
                .query_map(params.as_slice(), |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        Grammeme {
                            code: String::new(), // filled below
                            name: row.get::<_, String>(2).unwrap_or_default(),
                            alias: row.get::<_, String>(1).unwrap_or_default(),
                        },
                    ))
                })
                .map_err(|e| CoreError::DictionaryError(format!("grammeme query failed: {}", e)))?
                .filter_map(|r| r.ok())
                .collect();

            // Build lookup map: code → (name, alias)
            let mut lookup: std::collections::HashMap<String, Grammeme> = std::collections::HashMap::new();
            for (code, gram) in resolved {
                lookup.insert(code, gram);
            }

            // Match each missing code to its resolved grammeme
            for code in &missing {
                if let Some(gram) = lookup.get(*code) {
                    let g = Grammeme {
                        code: (*code).to_string(),
                        name: gram.name.clone(),
                        alias: gram.alias.clone(),
                    };
                    cache.insert((*code).to_string(), g.clone());
                    result.push(g);
                }
                // Codes not found (e.g. "@v") are silently skipped
            }
        }

        Ok(result)
    }
}

fn is_pos(code: &str) -> bool {
    matches!(code,
        "NOUN" | "VERB" | "ADJF" | "ADJS" | "ADVB" | "COMP" |
        "PRTF" | "PRTS" | "GRND" | "INFN" | "PRED" |
        "PREP" | "CONJ" | "PRCL" | "INTJ" | "NUMR" | "NPRO" | "ADV"
    )
}
