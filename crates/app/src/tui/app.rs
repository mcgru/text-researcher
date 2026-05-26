use std::io;

use log::info;
use ratatui::crossterm;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::{DefaultTerminal, Frame};

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
        }
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
                    Focus::Text => self.text.handle_input(key),
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
