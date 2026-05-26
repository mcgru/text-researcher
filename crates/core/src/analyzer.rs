use std::collections::HashMap;

use udpipe_client::Sentence;

use crate::error::CoreError;
use crate::model::ModelManager;

/// Text analyzer with in-memory caching.
pub struct Analyzer {
    model_manager: ModelManager,
    /// Cache: text → parsed sentences (key is the raw text)
    cache: HashMap<String, Vec<Sentence>>,
}

impl Analyzer {
    /// Create a new Analyzer with the given ModelManager.
    pub fn new(model_manager: ModelManager) -> Self {
        Analyzer {
            model_manager,
            cache: HashMap::new(),
        }
    }

    /// Analyze full text. Results are cached — repeated calls return cached data.
    pub fn analyze(&mut self, text: &str, language: &str) -> Result<&Vec<Sentence>, CoreError> {
        if !self.cache.contains_key(text) {
            let pipeline = self.model_manager.get(language)?;
            let sentences = pipeline
                .tokenize(text)
                .map_err(|e| CoreError::AnalysisFailed(format!("UDPipe analysis failed: {}", e)))?;
            self.cache.insert(text.to_string(), sentences);
        }
        Ok(self.cache.get(text).unwrap())
    }

    /// Get a flat token at the given word index across all sentences.
    /// Word index is global across the entire text.
    pub fn analyze_at(&mut self, text: &str, language: &str, word_index: usize) -> Result<Option<&udpipe_client::Token>, CoreError> {
        let sentences = self.analyze(text, language)?;
        let mut count = 0usize;
        for sentence in sentences {
            for token in &sentence.tokens {
                if count == word_index {
                    return Ok(Some(token));
                }
                count += 1;
            }
        }
        Ok(None)
    }

    /// Clear the analysis cache.
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}
