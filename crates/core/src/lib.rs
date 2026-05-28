//! Core analysis engine for text-researcher.
//!
//! Provides text analysis, model management, project state,
//! and configuration handling.

pub mod analyzer;
pub mod config;
pub mod dictionary;
pub mod error;
pub mod model;
pub mod morphology;

pub use analyzer::Analyzer;
pub use config::{GlobalConfig, SessionConfig};
pub use dictionary::{DictBackend, DictConfig, DictEntry, Grammeme, open_backend};
pub use error::CoreError;
pub use model::ModelManager;
