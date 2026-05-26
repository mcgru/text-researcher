//! Core analysis engine for text-researcher.
//!
//! Provides text analysis, model management, project state,
//! and configuration handling.

pub mod analyzer;
pub mod config;
pub mod error;
pub mod model;

pub use analyzer::Analyzer;
pub use config::{GlobalConfig, SessionConfig};
pub use error::CoreError;
pub use model::ModelManager;
