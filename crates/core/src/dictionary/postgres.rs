use std::collections::HashMap;

use log::debug;
use postgres::{Client, NoTls};
use rayon::prelude::*;
use r2d2::Pool;
use r2d2_postgres::PostgresConnectionManager;

use super::{DictBackend, DictEntry, Grammeme, is_pos};
use crate::error::CoreError;

pub struct PostgresBackend {
    pool: Pool<PostgresConnectionManager<NoTls>>,
    url: String,
}

impl PostgresBackend {
    pub fn open(url: &str) -> Result<Self, CoreError> {
        let manager = PostgresConnectionManager::new(
            url.parse()
                .map_err(|e| CoreError::DictionaryError(format!("bad postgres URL: {}", e)))?,
            NoTls,
        );
        let pool = Pool::builder()
            .max_size(8)
            .build(manager)
            .map_err(|e| CoreError::DictionaryError(format!("pool creation failed: {}", e)))?;

        debug!("PostgresBackend opened: {}", url);
        Ok(PostgresBackend { pool, url: url.to_string() })
    }

    fn lookup_impl(&self, conn: &mut Client, word: &str) -> Result<Vec<DictEntry>, CoreError> {
        let rows = conn.query(
            "SELECT f.id, f.text, f.lemma_id, l.text
             FROM forms f
             JOIN lemmata l ON f.lemma_id = l.id
             WHERE f.text = $1
             LIMIT 20",
            &[&word],
        ).map_err(|e| CoreError::DictionaryError(format!("query: {}", e)))?;

        if rows.is_empty() {
            return Ok(Vec::new());
        }

        let mut entries = Vec::new();
        let mut gram_cache: HashMap<String, Grammeme> = HashMap::new();

        for row in &rows {
            let form_id: i64 = row.get(0);
            let form_text: String = row.get(1);
            let lemma_id: i64 = row.get(2);
            let lemma_text: String = row.get(3);

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

impl DictBackend for PostgresBackend {
    fn lookup(&self, word: &str) -> Result<Vec<DictEntry>, CoreError> {
        let cleaned = word.to_lowercase();
        let cleaned = cleaned.trim_matches(|c: char| !c.is_alphanumeric() && c != '-').to_string();
        let mut conn = self.pool.get()
            .map_err(|e| CoreError::DictionaryError(format!("pool: {}", e)))?;
        self.lookup_impl(&mut conn, &cleaned)
    }

    fn lookup_batch(&self, words: &[&str]) -> HashMap<String, Vec<DictEntry>> {
        let results: std::sync::Mutex<HashMap<String, Vec<DictEntry>>> = std::sync::Mutex::new(HashMap::new());

        words.par_iter().for_each(|word| {
            let cleaned: String = word.to_lowercase()
                .trim_matches(|c: char| !c.is_alphanumeric() && c != '-')
                .to_string();

            if cleaned.is_empty() { return; }

            if let Ok(mut conn) = self.pool.get() {
                if let Ok(entries) = self.lookup_impl(&mut conn, &cleaned) {
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
    conn: &mut Client, form_id: i64, cache: &mut HashMap<String, Grammeme>,
) -> Result<Vec<Grammeme>, CoreError> {
    let rows = conn.query("SELECT grammeme_v FROM form_grammemes WHERE form_id = $1", &[&form_id])
        .map_err(|e| CoreError::DictionaryError(format!("query: {}", e)))?;
    let codes: Vec<String> = rows.iter().map(|r| r.get(0)).collect();
    resolve_grammemes(conn, &codes, cache)
}

fn get_lemma_grammemes(
    conn: &mut Client, lemma_id: i64, cache: &mut HashMap<String, Grammeme>,
) -> Result<Vec<Grammeme>, CoreError> {
    let rows = conn.query("SELECT grammeme_v FROM lemma_grammemes WHERE lemma_id = $1", &[&lemma_id])
        .map_err(|e| CoreError::DictionaryError(format!("query: {}", e)))?;
    let codes: Vec<String> = rows.iter().map(|r| r.get(0)).collect();
    resolve_grammemes(conn, &codes, cache)
}

fn resolve_grammemes(
    conn: &mut Client, codes: &[String], cache: &mut HashMap<String, Grammeme>,
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

    let placeholders: Vec<String> = (1..=missing.len()).map(|i| format!("${}", i)).collect();
    let sql = format!("SELECT name, alias, description FROM grammemes WHERE name IN ({})", placeholders.join(","));

    let params: Vec<&(dyn postgres::types::ToSql + Sync)> = missing.iter()
        .map(|s| s as &(dyn postgres::types::ToSql + Sync)).collect();

    let rows = conn.query(&sql, &params)
        .map_err(|e| CoreError::DictionaryError(format!("grammeme query: {}", e)))?;

    let mut lookup: HashMap<String, Grammeme> = HashMap::new();
    for row in &rows {
        let code: String = row.get(0);
        let alias: String = row.get(1);
        let name: String = row.get(2);
        lookup.insert(code, Grammeme { code: String::new(), name, alias });
    }

    for code in &missing {
        if let Some(gram) = lookup.get(*code) {
            let g = Grammeme { code: (*code).to_string(), name: gram.name.clone(), alias: gram.alias.clone() };
            cache.insert((*code).to_string(), g.clone());
            result.push(g);
        }
    }

    Ok(result)
}
