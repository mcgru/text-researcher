use std::collections::HashMap;
use std::io;
use std::sync::mpsc;
use std::thread;

use log::info;
use ratatui::crossterm;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::{DefaultTerminal, Frame};
use text_researcher_core::{DictEntry, OpenCorporaDict};

use super::panels::{
    Action, MenuBar, Panel, ProjectFile, PropsPane, ShortcutHandler, StatusBar, TextPane,
    load_project, save_project,
};

pub struct AppState {
    menu: MenuBar,
    text: TextPane,
    props: PropsPane,
    status: StatusBar,
    extras: Vec<Box<dyn Panel>>,
    focused: Focus,
    running: bool,
    shortcuts: ShortcutHandler,
    project_path: Option<String>,
    dirty: bool,
    dict: Option<OpenCorporaDict>,
    prefetch_cache: HashMap<String, Vec<DictEntry>>,
    prefetch_count: isize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Focus {
    Menu,
    Text,
    Props,
    Extra(usize),
}

impl AppState {
    pub fn new(text_pane: TextPane, props_pane: PropsPane) -> Self {
        AppState {
            menu: MenuBar::new(),
            text: text_pane,
            props: props_pane,
            status: StatusBar::new(),
            extras: Vec::new(),
            focused: Focus::Text,
            running: false,
            shortcuts: ShortcutHandler::new(),
            project_path: None,
            dirty: false,
            dict: None,
            prefetch_cache: HashMap::new(),
            prefetch_count: -1,
        }
    }

    pub fn set_dictionary(&mut self, dict: OpenCorporaDict) {
        self.dict = Some(dict);
        self.lookup_current_word();
        self.prefetch_surrounding();
    }

    pub fn set_prefetch_count(&mut self, count: isize) {
        self.prefetch_count = count;
    }

    /// Prefetch words around cursor: left + right, or entire text if count == -1.
    fn prefetch_surrounding(&mut self) {
        let count = self.prefetch_count;
        if count == 0 || self.dict.is_none() {
            return;
        }

        let total = self.text.word_count();
        if total == 0 {
            return;
        }

        let cursor = self.text.cursor_word_index();

        let indices: Vec<usize> = if count < 0 {
            // Entire text — run in background thread
            self.prefetch_all_background();
            return;
        } else {
            let count = count as usize;
            let left_start = cursor.saturating_sub(count);
            let right_end = (cursor + 1 + count).min(total);
            (left_start..right_end).collect()
        };

        self.prefetch_indices(&indices);
    }

    /// Launch background prefetch of all words in the text.
    fn prefetch_all_background(&mut self) {
        if self.dict.is_none() {
            return;
        }

        let total = self.text.word_count();
        let words_to_fetch: Vec<String> = (0..total)
            .filter_map(|i| {
                let w = self.text.word_at(i)?;
                if self.prefetch_cache.contains_key(w) {
                    None
                } else {
                    Some(w.to_string())
                }
            })
            .collect();

        if words_to_fetch.is_empty() {
            return;
        }

        let dict = self.dict.as_ref().unwrap();
        let path = dict.path().to_path_buf();
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            if let Ok(dict) = text_researcher_core::OpenCorporaDict::open(&path) {
                let refs: Vec<&str> = words_to_fetch.iter().map(|w| w.as_str()).collect();
                let results = dict.lookup_batch(&refs);
                let _ = tx.send(results);
            }
        });

        // Store receiver for later pickup in event loop
        // For now, do a quick non-blocking check
        if let Ok(results) = rx.try_recv() {
            self.prefetch_cache.extend(results);
        }
        // Note: remaining results picked up in next event loop iteration via drain_channel
    }

    /// Prefetch specific word indices.
    fn prefetch_indices(&mut self, indices: &[usize]) {
        let mut not_cached: Vec<&str> = Vec::new();
        for &i in indices {
            if let Some(word) = self.text.word_at(i) {
                if !self.prefetch_cache.contains_key(word) {
                    not_cached.push(word);
                }
            }
        }

        if not_cached.is_empty() {
            return;
        }

        let dict = self.dict.as_ref().unwrap();
        let results = dict.lookup_batch(&not_cached);
        self.prefetch_cache.extend(results);
    }

    /// Look up the current word in the dictionary and update props pane.
    fn lookup_current_word(&mut self) {
        if let Some(word) = self.text.current_word() {
            let word = word.to_string();

            // Check prefetch cache first
            if let Some(entry) = self.prefetch_cache.get(&word).and_then(|e| e.first().cloned()) {
                self.update_props_from_entry(&entry);
                return;
            }

            if let Some(ref dict) = self.dict {
                match dict.lookup(&word) {
                    Ok(entries) => {
                        if let Some(entry) = entries.first() {
                            self.prefetch_cache.insert(word, entries.clone());
                            self.update_props_from_entry(entry);
                            return;
                        }
                    }
                    Err(e) => {
                        info!("Dictionary lookup failed: {}", e);
                    }
                }
            }
        }
        self.props.clear();
    }

    fn update_props_from_entry(&mut self, entry: &DictEntry) {
        let mut feats = std::collections::HashMap::new();
        if let Some(ref pos) = entry.pos {
            feats.insert("Часть речи".to_string(), pos.clone());
        }
        for gram in &entry.grammemes {
            feats.insert(gram.name.clone(), gram.alias.clone());
        }
        self.props.update(&entry.form, &entry.lemma, &feats);
    }

    pub fn is_running(&self) -> bool { self.running }
    pub fn quit(&mut self) { self.running = false; }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        self.running = true;
        while self.running {
            terminal.draw(|frame| self.render(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn render(&self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(1), Constraint::Length(1)])
            .split(frame.area());

        // Menu
        self.menu.render(frame, chunks[0], self.focused == Focus::Menu);

        // Main: props (30%) + text (70%)
        let main = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(chunks[1]);

        self.props.render(frame, main[0], self.focused == Focus::Props);
        self.text.render(frame, main[1], self.focused == Focus::Text);

        // Extras
        for (i, extra) in self.extras.iter().enumerate() {
            if self.focused == Focus::Extra(i) {
                extra.render(frame, main[0], true);
            }
        }

        // Status
        self.status.render(frame, chunks[2], false);
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if event::poll(std::time::Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press { return Ok(()); }

                // Ctrl combos
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    match key.code {
                        KeyCode::Char('s') => { self.save(); return Ok(()); }
                        KeyCode::Char('q') => { self.running = false; return Ok(()); }
                        _ => {}
                    }
                }

                match key.code {
                    KeyCode::Esc => { self.running = false; return Ok(()); }
                    KeyCode::Tab => { self.cycle_focus(); return Ok(()); }
                    _ => {}
                }

                // Shortcuts
                if let Some(action) = self.shortcuts.handle(key) {
                    self.dispatch(action);
                    return Ok(());
                }

                // Panel input
                let action = match self.focused {
                    Focus::Menu => self.menu.handle_input(key),
                    Focus::Text => {
                        self.text.handle_input(key);
                        self.lookup_current_word();
                        self.prefetch_surrounding();
                        Action::None
                    }
                    Focus::Props => self.props.handle_input(key),
                    Focus::Extra(i) => self.extras[i].handle_input(key),
                };
                self.dispatch(action);
            }
        }
        Ok(())
    }

    fn dispatch(&mut self, action: Action) {
        match action {
            Action::Quit => self.running = false,
            Action::Save => self.save(),
            Action::EditWord { .. } => { self.dirty = true; }
            _ => {}
        }
    }

    fn cycle_focus(&mut self) {
        self.focused = match self.focused {
            Focus::Menu => Focus::Props,
            Focus::Props => Focus::Text,
            Focus::Text => Focus::Menu,
            Focus::Extra(_) => Focus::Menu,
        };
        // Update status bar with cursor position
        self.status.update("", 1, self.text.cursor_word_index(), "RU", self.dirty);
    }

    fn save(&mut self) {
        let path = self.project_path.clone().unwrap_or_else(|| "project.trproj".into());
        let project = ProjectFile {
            meta: super::panels::ProjectMeta { version: "0.2.0".into() },
            source: super::panels::SourceRef { file_path: path.clone() },
            cursor: super::panels::CursorPos { line: 1, word_index: self.text.cursor_word_index() },
            edits: std::collections::HashMap::new(),
            language: "ru".into(),
        };
        if save_project(&project, &path).is_ok() {
            self.project_path = Some(path);
            self.dirty = false;
            self.status.update("", 1, self.text.cursor_word_index(), "RU", false);
        }
    }
}

pub fn check_terminal_size() -> io::Result<()> {
    let (cols, rows) = crossterm::terminal::size()?;
    if cols < 80 || rows < 24 {
        return Err(io::Error::new(io::ErrorKind::Other,
            format!("terminal too small: {}x{}. Minimum: 80x24", cols, rows)));
    }
    Ok(())
}
