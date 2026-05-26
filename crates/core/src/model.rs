use std::collections::HashMap;
use std::path::PathBuf;

use log::{debug, info};
use udpipe_client::Udpipeline;

use crate::error::CoreError;

/// Manages loaded UDPipe models by language code.
pub struct ModelManager {
    model_dir: PathBuf,
    pipelines: HashMap<String, Udpipeline>,
}

impl ModelManager {
    /// Create a new ModelManager scanning the given directory for .udpipe files.
    pub fn new(model_dir: PathBuf) -> Self {
        ModelManager {
            model_dir,
            pipelines: HashMap::new(),
        }
    }

    /// List available languages (models found in the model directory).
    pub fn available_languages(&self) -> Vec<String> {
        let mut langs = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&self.model_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "udpipe") {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        langs.push(stem.to_string());
                    }
                }
            }
        }
        langs.sort();
        debug!("Available languages: {:?}", langs);
        langs
    }

    /// Load a model for the given language.
    ///
    /// Looks for `<model_dir>/<language>.udpipe`.
    pub fn load(&mut self, language: &str) -> Result<(), CoreError> {
        if self.pipelines.contains_key(language) {
            return Ok(());
        }

        let model_path = self.model_dir.join(format!("{}.udpipe", language));
        if !model_path.exists() {
            return Err(CoreError::ModelNotFound(format!(
                "model file not found: {}. Run 'text-researcher download-model {}' or place the model manually.",
                model_path.display(), language
            )));
        }

        info!("Loading UDPipe model for language: {}", language);
        let pipeline = Udpipeline::new(model_path.to_string_lossy().to_string());
        self.pipelines.insert(language.to_string(), pipeline);
        Ok(())
    }

    /// Get a reference to the pipeline for a language.
    /// Returns error if the model is not loaded.
    pub fn get(&mut self, language: &str) -> Result<&mut Udpipeline, CoreError> {
        if !self.pipelines.contains_key(language) {
            self.load(language)?;
        }
        Ok(self.pipelines.get_mut(language).unwrap())
    }
}
