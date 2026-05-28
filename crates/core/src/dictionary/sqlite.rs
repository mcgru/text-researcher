use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use log::debug;
use rayon::prelude::*;
use rusqlite::Connection;

use super::{DictBackend, DictEntry, Grammeme, is_pos};
use crate::error::CoreError;

pub struct SqliteBackend {
    conn: Mutex<Connection>,
    path: PathBuf,
}

impl SqliteBackend {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, CoreError> {
        let path = path.as_ref().to_path_buf();
        let conn = Connection::open(&path)
            .map_err(|e| CoreError::DictionaryError(format!("failed to open sqlite: {}", e)))?;

        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=OFF;")
            .map_err(|e| CoreError::DictionaryError(format!("pragma failed: {}", e)))?;

        debug!("SqliteBackend opened: {}", path.display());
        Ok(SqliteBackend { conn: Mutex::new(conn), path })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn lookup_impl(&self, conn: &Connection, word: &str) -> Result<Vec<DictEntry>, CoreError> {
        let mut stmt = conn.prepare(
            "SELECT f.id, f.text, f.lemma_id, l.text
             FROM forms f
             JOIN lemmata l ON f.lemma_id = l.id
             WHERE f.text = ?1
             LIMIT 20"
        ).map_err(|e| CoreError::DictionaryError(format!("prepare: {}", e)))?;

        let rows: Vec<(i64, String, i64, String)> = stmt
            .query_map([word], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)))
            .map_err(|e| CoreError::DictionaryError(format!("query: {}", e)))?
            .filter_map(|r| r.ok())
            .collect();

        if rows.is_empty() {
            return Ok(Vec::new());
        }

        let mut entries = Vec::new();
        let mut gram_cache: HashMap<String, Grammeme> = HashMap::new();

        for (form_id, form_text, lemma_id, lemma_text) in rows {
            let form_grammemes = get_form_grammemes(conn, form_id, &mut gram_cache)?;
            let lemma_grammemes = get_lemma_grammemes(conn, lemma_id, &mut gram_cache)?;

            let pos = lemma_grammemes.iter()
                .find(|g| is_pos(&g.code))
                .map(|g| g.code.clone());

            let mut all = Vec::new();
            all.extend(lemma_grammemes.into_iter().filter(|g| !is_pos(&g.code)));
            all.extend(form_grammemes);

            entries.push(DictEntry { form: form_text, lemma: lemma_text, pos, grammemes: all });
        }

        Ok(entries)
    }
}

impl DictBackend for SqliteBackend {
    fn lookup(&self, word: &str) -> Result<Vec<DictEntry>, CoreError> {
        let cleaned = word.to_lowercase();
        let cleaned = cleaned.trim_matches(|c: char| !c.is_alphanumeric() && c != '-').to_string();
        let conn = self.conn.lock().unwrap();
        self.lookup_impl(&conn, &cleaned)
    }

    fn lookup_batch(&self, words: &[&str]) -> HashMap<String, Vec<DictEntry>> {
        let results: Mutex<HashMap<String, Vec<DictEntry>>> = Mutex::new(HashMap::new());

        words.par_iter().for_each(|word| {
            let cleaned: String = word.to_lowercase()
                .trim_matches(|c: char| !c.is_alphanumeric() && c != '-')
                .to_string();

            if cleaned.is_empty() { return; }

            if let Ok(conn) = Connection::open(&self.path) {
                let _ = conn.execute_batch("PRAGMA journal_mode=WAL;");
                if let Ok(entries) = self.lookup_impl(&conn, &cleaned) {
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
}

// ── Grammeme helpers ──

fn get_form_grammemes(
    conn: &Connection, form_id: i64, cache: &mut HashMap<String, Grammeme>,
) -> Result<Vec<Grammeme>, CoreError> {
    let mut stmt = conn.prepare("SELECT grammeme_v FROM form_grammemes WHERE form_id = ?1")
        .map_err(|e| CoreError::DictionaryError(format!("prepare: {}", e)))?;
    let codes: Vec<String> = stmt.query_map([form_id], |r| r.get(0))
        .map_err(|e| CoreError::DictionaryError(format!("query: {}", e)))?
        .filter_map(|r| r.ok()).collect();
    resolve_grammemes(conn, &codes, cache)
}

fn get_lemma_grammemes(
    conn: &Connection, lemma_id: i64, cache: &mut HashMap<String, Grammeme>,
) -> Result<Vec<Grammeme>, CoreError> {
    let mut stmt = conn.prepare("SELECT grammeme_v FROM lemma_grammemes WHERE lemma_id = ?1")
        .map_err(|e| CoreError::DictionaryError(format!("prepare: {}", e)))?;
    let codes: Vec<String> = stmt.query_map([lemma_id], |r| r.get(0))
        .map_err(|e| CoreError::DictionaryError(format!("query: {}", e)))?
        .filter_map(|r| r.ok()).collect();
    resolve_grammemes(conn, &codes, cache)
}

fn resolve_grammemes(
    conn: &Connection, codes: &[String], cache: &mut HashMap<String, Grammeme>,
) -> Result<Vec<Grammeme>, CoreError> {
    let mut result = Vec::new();
    let mut missing: Vec<&str> = Vec::new();

    for code in codes {
        if let Some(g) = cache.get(code) {
            result.push(g.clone());
        } else {
            missing.push(code);
        }
    }

    if missing.is_empty() { return Ok(result); }

    let placeholders: Vec<String> = (1..=missing.len()).map(|i| format!("?{}", i)).collect();
    let sql = format!("SELECT name, alias, description FROM grammemes WHERE name IN ({})", placeholders.join(","));

    let mut stmt = conn.prepare(&sql)
        .map_err(|e| CoreError::DictionaryError(format!("prepare: {}", e)))?;

    let params: Vec<&dyn rusqlite::types::ToSql> = missing.iter()
        .map(|s| s as &dyn rusqlite::types::ToSql).collect();

    let resolved: Vec<(String, Grammeme)> = stmt
        .query_map(params.as_slice(), |row| Ok((
            row.get::<_, String>(0)?,
            Grammeme { code: String::new(), name: row.get::<_, String>(2).unwrap_or_default(), alias: row.get::<_, String>(1).unwrap_or_default() },
        )))
        .map_err(|e| CoreError::DictionaryError(format!("grammeme query: {}", e)))?
        .filter_map(|r| r.ok()).collect();

    let mut lookup: HashMap<String, Grammeme> = HashMap::new();
    for (code, gram) in resolved { lookup.insert(code, gram); }

    for code in &missing {
        if let Some(gram) = lookup.get(*code) {
            let g = Grammeme { code: (*code).to_string(), name: gram.name.clone(), alias: gram.alias.clone() };
            cache.insert((*code).to_string(), g.clone());
            result.push(g);
        }
    }

    Ok(result)
}
