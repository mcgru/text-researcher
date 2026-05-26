use std::collections::HashMap;

use log::info;
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use super::{Action, Panel};

/// Menu bar panel.
pub struct MenuBar {
    active: bool,
}

impl MenuBar {
    pub fn new() -> Self {
        MenuBar { active: false }
    }
}

impl Panel for MenuBar {
    fn render(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let style = if focused {
            Style::default().fg(Color::Black).bg(Color::White)
        } else {
            Style::default().fg(Color::White).bg(Color::DarkGray)
        };
        frame.render_widget(
            Paragraph::new(" File  Edit  View  Help ").style(style),
            area,
        );
    }

    fn handle_input(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::F(10) | KeyCode::Char('m') => {
                self.active = !self.active;
            }
            _ => {}
        }
        Action::None
    }

    fn title(&self) -> &str {
        "Menu"
    }

    fn focusable(&self) -> bool {
        true
    }
}

/// Status bar showing current file, position, language, and dirty flag.
pub struct StatusBar {
    filename: String,
    line: usize,
    col: usize,
    language: String,
    dirty: bool,
}

impl StatusBar {
    pub fn new() -> Self {
        StatusBar {
            filename: String::new(),
            line: 1,
            col: 1,
            language: "RU".into(),
            dirty: false,
        }
    }

    pub fn update(&mut self, filename: &str, line: usize, col: usize, language: &str, dirty: bool) {
        self.filename = filename.to_string();
        self.line = line;
        self.col = col;
        self.language = language.to_string();
        self.dirty = dirty;
    }
}

impl Panel for StatusBar {
    fn render(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let dirty_mark = if self.dirty { " *" } else { "" };
        let text = format!(
            " {} | Ln {}, Col {} | {} {}",
            if self.filename.is_empty() { "(no file)" } else { &self.filename },
            self.line,
            self.col,
            self.language,
            dirty_mark,
        );
        let style = Style::default().fg(Color::White).bg(Color::DarkGray);
        frame.render_widget(Paragraph::new(text).style(style), area);
    }

    fn handle_input(&mut self, _key: KeyEvent) -> Action {
        Action::None
    }

    fn title(&self) -> &str {
        "Status"
    }

    fn focusable(&self) -> bool {
        false
    }
}

/// Keyboard shortcuts handler.
pub struct ShortcutHandler {
    shortcuts: HashMap<KeyCode, Action>,
}

impl ShortcutHandler {
    pub fn new() -> Self {
        let mut shortcuts = HashMap::new();
        shortcuts.insert(KeyCode::Char('s'), Action::Save);      // Ctrl+S handled by event loop
        shortcuts.insert(KeyCode::Char('o'), Action::OpenFile);
        shortcuts.insert(KeyCode::Char('q'), Action::Quit);
        shortcuts.insert(KeyCode::Char('l'), Action::SwitchLanguage("next".into()));
        shortcuts.insert(KeyCode::Char('e'), Action::EditWord { word_index: 0 });
        ShortcutHandler { shortcuts }
    }

    pub fn handle(&self, key: KeyEvent) -> Option<Action> {
        self.shortcuts.get(&key.code).cloned()
    }
}
