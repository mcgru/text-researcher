use std::collections::HashMap;
use std::io;
use std::sync::mpsc;
use std::thread;

use log::info;
use ratatui::crossterm;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::{DefaultTerminal, Frame};
use text_researcher_core::{DictBackend, DictConfig, DictEntry, open_backend};

use super::panels::{
    Action, LogPane, MenuBar, Panel, ProjectFile, PropsPane, ShortcutHandler, StatusBar, TextPane,
    load_project, save_project,
};

pub struct AppState {
    menu: MenuBar,
    text: TextPane,
    props: PropsPane,
    status: StatusBar,
    log: LogPane,
    extras: Vec<Box<dyn Panel>>,
    focused: Focus,
    running: bool,
    shortcuts: ShortcutHandler,
    project_path: Option<String>,
    dirty: bool,
    dict: Option<Box<dyn DictBackend>>,
    dict_config: Option<DictConfig>,
    prefetch_cache: HashMap<String, Option<Vec<DictEntry>>>,
    prefetch_count: isize,
    prefetch_rx: Option<mpsc::Receiver<HashMap<String, Option<Vec<DictEntry>>>>>,
    prefetch_log_rx: Option<mpsc::Receiver<(String, bool)>>,
    prefetch_pending: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Focus {
    Menu,
    Text,
    Props,
    Log,
    Extra(usize),
}

impl AppState {
    pub fn new(text_pane: TextPane, props_pane: PropsPane) -> Self {
        AppState {
            menu: MenuBar::new(),
            text: text_pane,
            props: props_pane,
            status: StatusBar::new(),
            log: LogPane::new(),
            extras: Vec::new(),
            focused: Focus::Text,
            running: false,
            shortcuts: ShortcutHandler::new(),
            project_path: None,
            dirty: false,
            dict: None,
            dict_config: None,
            prefetch_cache: HashMap::new(),
            prefetch_count: -1,
            prefetch_rx: None,
            prefetch_log_rx: None,
            prefetch_pending: false,
        }
    }

    pub fn set_dictionary(&mut self, dict: Box<dyn DictBackend>, config: DictConfig) {
        self.dict = Some(dict);
        self.dict_config = Some(config);
        // Immediate: show properties for first word
        self.lookup_current_word();
        // Then start background prefetch
        self.start_background_prefetch();
    }

    pub fn set_prefetch_count(&mut self, count: isize) {
        self.prefetch_count = count;
    }

    /// Start background prefetch of remaining words in small batches with 50ms pauses.
    fn start_background_prefetch(&mut self) {
        if self.dict.is_none() || self.prefetch_count == 0 || self.prefetch_pending {
            return;
        }

        let total = self.text.word_count();
        let cursor = self.text.cursor_word_index();

        let indices: Vec<usize> = if self.prefetch_count < 0 {
            (0..total).filter(|&i| i != cursor).collect()
        } else {
            let n = self.prefetch_count as usize;
            let left = cursor.saturating_sub(n);
            let right = (cursor + 1 + n).min(total);
            (left..right).filter(|&i| i != cursor).collect()
        };

        let words: Vec<String> = indices.iter()
            .filter_map(|&i| self.text.word_at(i))
            .filter(|w| !self.prefetch_cache.contains_key(*w))
            .map(|w| w.to_string())
            .collect();

        if words.is_empty() {
            return;
        }

        self.log.log(&format!("prefetch {} words", words.len()));
        self.prefetch_pending = true;

        let dict_config = self.dict_config.clone().unwrap();
        let (tx, rx) = mpsc::channel();
        let (log_tx, log_rx) = mpsc::channel();

        self.prefetch_log_rx = Some(log_rx);

        thread::spawn(move || {
            let batch_size = 2usize;
            let pause = std::time::Duration::from_millis(50);
            let mut results: HashMap<String, Option<Vec<DictEntry>>> = HashMap::new();

            if let Ok(dict) = open_backend(&dict_config) {
                for chunk in words.chunks(batch_size) {
                    let refs: Vec<&str> = chunk.iter().map(|w| w.as_str()).collect();
                    let found = dict.lookup_batch(&refs);

                    for word in chunk {
                        if let Some(entries) = found.get(word.as_str()) {
                            results.insert(word.clone(), Some(entries.clone()));
                            let _ = log_tx.send((word.clone(), true));
                        } else {
                            results.insert(word.clone(), None);
                            let _ = log_tx.send((word.clone(), false));
                        }
                    }

                    // 50ms pause between batches for keyboard responsiveness
                    std::thread::sleep(pause);
                }
            } else {
                for word in &words {
                    results.insert(word.clone(), None);
                    let _ = log_tx.send((word.clone(), false));
                }
            }

            let _ = tx.send(results);
        });

        self.prefetch_rx = Some(rx);
    }

    /// Poll for background prefetch results and log messages.
    fn poll_prefetch(&mut self) {
        // Drain log messages
        if let Some(ref rx) = self.prefetch_log_rx {
            loop {
                match rx.try_recv() {
                    Ok((word, found)) => {
                        self.log.log_prefetch_word(&word, found);
                    }
                    Err(mpsc::TryRecvError::Empty) => break,
                    Err(mpsc::TryRecvError::Disconnected) => {
                        self.prefetch_log_rx = None;
                        break;
                    }
                }
            }
        }

        if !self.prefetch_pending {
            return;
        }
        if let Some(ref rx) = self.prefetch_rx {
            if let Ok(results) = rx.try_recv() {
                let count = results.len();
                let found_count = results.values().filter(|v| v.is_some()).count();
                self.prefetch_cache.extend(results);
                self.prefetch_pending = false;
                self.prefetch_rx = None;
                self.log.log(&format!("done: {}/{} found", found_count, count));
            }
        }
    }

    /// Look up the current word in the dictionary and update props pane.
    fn lookup_current_word(&mut self) {
        if let Some(word) = self.text.current_word() {
            let word = word.to_string();

            // Check prefetch cache first
            match self.prefetch_cache.get(&word) {
                Some(Some(entries)) => {
                    if let Some(entry) = entries.first() {
                        let form = entry.form.clone();
                        let lemma = entry.lemma.clone();
                        let pos = entry.pos.clone();
                        let grammemes: Vec<_> = entry.grammemes.iter().map(|g| (g.name.clone(), g.alias.clone())).collect();
                        self.update_props_from_parts(&form, &lemma, &pos, &grammemes);
                        return;
                    }
                }
                Some(None) => {
                    // Negative cache: word not in dictionary
                    self.props.clear();
                    return;
                }
                None => {} // Not in cache, query DB
            }

            if let Some(ref dict) = self.dict {
                match dict.lookup(&word) {
                    Ok(entries) => {
                        if entries.is_empty() {
                            self.prefetch_cache.insert(word, None); // negative cache
                            self.props.clear();
                        } else {
                            self.prefetch_cache.insert(word.clone(), Some(entries.clone()));
                            if let Some(entry) = entries.first() {
                                self.update_props_from_entry(entry);
                            }
                        }
                        return;
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
        let pos = entry.pos.clone();
        let grammemes: Vec<_> = entry.grammemes.iter().map(|g| (g.name.clone(), g.alias.clone())).collect();
        self.update_props_from_parts(&entry.form, &entry.lemma, &pos, &grammemes);
    }

    fn update_props_from_parts(&mut self, form: &str, lemma: &str, pos: &Option<String>, grammemes: &[(String, String)]) {
        let mut feats = HashMap::new();
        if let Some(ref p) = pos {
            feats.insert("Часть речи".to_string(), p.clone());
        }
        for (name, alias) in grammemes {
            feats.insert(name.clone(), alias.clone());
        }
        self.props.update(form, lemma, &feats);
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
            .constraints([
                Constraint::Length(1),  // menu
                Constraint::Min(1),     // main area
                Constraint::Length(4),  // log
                Constraint::Length(1),  // status
            ])
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
        self.status.render(frame, chunks[3], false);

        // Log
        self.log.render(frame, chunks[2], self.focused == Focus::Log);
    }

    fn handle_events(&mut self) -> io::Result<()> {
        // Always poll for background prefetch results
        self.poll_prefetch();

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
                        if !self.prefetch_pending {
                            self.start_background_prefetch();
                        }
                        Action::None
                    }
                    Focus::Props => self.props.handle_input(key),
                    Focus::Log => self.log.handle_input(key),
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
            Focus::Text => Focus::Log,
            Focus::Log => Focus::Menu,
            Focus::Extra(_) => Focus::Menu,
        };
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
