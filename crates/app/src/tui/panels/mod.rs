pub mod log;
pub mod menu;
pub mod props;
pub mod text;

use std::collections::HashMap;
use std::fs;

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::Rect;
use ratatui::Frame;
use serde::{Deserialize, Serialize};

pub use log::LogPane;
pub use menu::{MenuBar, ShortcutHandler, StatusBar};
pub use props::PropsPane;
pub use text::TextPane;

// ── Panel trait ──

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    None,
    Quit,
    Save,
    OpenFile,
    SwitchLanguage(String),
    EditWord { word_index: usize },
    SwitchFocus(usize),
    Help,
}

pub trait Panel {
    fn render(&self, frame: &mut Frame, area: Rect, focused: bool);
    fn handle_input(&mut self, key: KeyEvent) -> Action;
    fn title(&self) -> &str;
    fn focusable(&self) -> bool { true }
}

// ── Project persistence ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectFile {
    pub meta: ProjectMeta,
    pub source: SourceRef,
    pub cursor: CursorPos,
    pub edits: HashMap<usize, TokenOverride>,
    pub language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMeta {
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceRef {
    pub file_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorPos {
    pub line: usize,
    pub word_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenOverride {
    pub upostag: Option<String>,
    pub lemma: Option<String>,
    pub feats: Option<HashMap<String, String>>,
}

/// Save project state to a .trproj file (TOML format).
pub fn save_project(project: &ProjectFile, path: &str) -> Result<(), String> {
    let toml_str = toml::to_string_pretty(project).map_err(|e| format!("TOML serialization failed: {}", e))?;
    let tmp = format!("{}.tmp", path);
    fs::write(&tmp, &toml_str).map_err(|e| format!("write failed: {}", e))?;
    fs::rename(&tmp, path).map_err(|e| format!("rename failed: {}", e))?;
    Ok(())
}

/// Load project state from a .trproj file (TOML or JSON).
pub fn load_project(path: &str) -> Result<ProjectFile, String> {
    let content = fs::read_to_string(path).map_err(|e| format!("read failed: {}", e))?;
    // Try TOML first
    toml::from_str(&content)
        .or_else(|_| serde_json::from_str(&content)
            .map_err(|e| format!("JSON parse failed: {}", e)))
        .map_err(|e| format!("{}", e))
}
