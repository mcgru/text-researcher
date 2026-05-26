use std::collections::HashMap;
use std::path::Path;

use log::debug;
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
    conn: Connection,
}

impl OpenCorporaDict {
    /// Open the dictionary database at the given path.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, CoreError> {
        let conn = Connection::open(path)
            .map_err(|e| CoreError::DictionaryError(format!("failed to open dictionary: {}", e)))?;

        // Enable WAL for concurrent reads
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=OFF;")
            .map_err(|e| CoreError::DictionaryError(format!("pragma failed: {}", e)))?;

        debug!("OpenCorpora dictionary opened");
        Ok(OpenCorporaDict { conn })
    }

    /// Look up all entries for a word form.
    pub fn lookup(&self, word: &str) -> Result<Vec<DictEntry>, CoreError> {
        let word_lower = word.to_lowercase();
        let word_clean = word_lower.trim_matches(|c: char| !c.is_alphanumeric() && c != '-');

        // Find matching forms
        let mut stmt = self.conn.prepare(
            "SELECT f.id, f.text, f.lemma_id, l.text
             FROM forms f
             JOIN lemmata l ON f.lemma_id = l.id
             WHERE f.text = ?1
             LIMIT 20"
        ).map_err(|e| CoreError::DictionaryError(format!("prepare failed: {}", e)))?;

        let rows: Vec<(i64, String, i64, String)> = stmt
            .query_map([&word_clean], |row| {
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
            let form_grammemes = self.get_form_grammemes(form_id, &mut gram_cache)?;
            let lemma_grammemes = self.get_lemma_grammemes(lemma_id, &mut gram_cache)?;

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
    fn get_form_grammemes(&self, form_id: i64, cache: &mut HashMap<String, Grammeme>) -> Result<Vec<Grammeme>, CoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT grammeme_v FROM form_grammemes WHERE form_id = ?1"
        ).map_err(|e| CoreError::DictionaryError(format!("prepare failed: {}", e)))?;

        let codes: Vec<String> = stmt
            .query_map([form_id], |row| row.get(0))
            .map_err(|e| CoreError::DictionaryError(format!("query failed: {}", e)))?
            .filter_map(|r| r.ok())
            .collect();

        self.resolve_grammemes(&codes, cache)
    }

    /// Get grammemes for a lemma.
    fn get_lemma_grammemes(&self, lemma_id: i64, cache: &mut HashMap<String, Grammeme>) -> Result<Vec<Grammeme>, CoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT grammeme_v FROM lemma_grammemes WHERE lemma_id = ?1"
        ).map_err(|e| CoreError::DictionaryError(format!("prepare failed: {}", e)))?;

        let codes: Vec<String> = stmt
            .query_map([lemma_id], |row| row.get(0))
            .map_err(|e| CoreError::DictionaryError(format!("query failed: {}", e)))?
            .filter_map(|r| r.ok())
            .collect();

        self.resolve_grammemes(&codes, cache)
    }

    /// Resolve grammeme codes to names, using cache.
    fn resolve_grammemes(&self, codes: &[String], cache: &mut HashMap<String, Grammeme>) -> Result<Vec<Grammeme>, CoreError> {
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

            let mut stmt = self.conn.prepare(&sql)
                .map_err(|e| CoreError::DictionaryError(format!("prepare failed: {}", e)))?;

            let params: Vec<&dyn rusqlite::types::ToSql> = missing.iter()
                .map(|s| s as &dyn rusqlite::types::ToSql)
                .collect();

            let resolved: Vec<Grammeme> = stmt
                .query_map(params.as_slice(), |row| {
                    Ok(Grammeme {
                        code: String::new(), // filled below
                        name: row.get::<_, String>(2).unwrap_or_default(),
                        alias: row.get::<_, String>(1).unwrap_or_default(),
                    })
                })
                .map_err(|e| CoreError::DictionaryError(format!("grammeme query failed: {}", e)))?
                .filter_map(|r| r.ok())
                .collect();

            for (i, gram) in resolved.into_iter().enumerate() {
                let code = missing[i].to_string();
                let g = Grammeme { code: code.clone(), name: gram.name, alias: gram.alias };
                cache.insert(code, g.clone());
                result.push(g);
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
