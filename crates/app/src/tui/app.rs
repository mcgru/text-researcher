use std::io;

use log::info;
use ratatui::crossterm;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::{DefaultTerminal, Frame};

use super::panels::{Action, Panel};

/// Main TUI application state.
pub struct AppState {
    panels: Vec<Box<dyn Panel>>,
    focused: usize,
    running: bool,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            panels: Vec::new(),
            focused: 0,
            running: false,
        }
    }

    pub fn add_panel(&mut self, panel: Box<dyn Panel>) {
        self.panels.push(panel);
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    /// Run the TUI event loop.
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        self.running = true;

        while self.running {
            terminal.draw(|frame| self.render(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn render(&self, frame: &mut Frame) {
        // Layout: menu (3 lines), main area (flex), status (1 line)
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(1),
                Constraint::Length(1),
            ])
            .split(frame.area());

        // Render menu area placeholder
        let menu_area = chunks[0];
        let main_area = chunks[1];
        let status_area = chunks[2];

        // Render focused panel in main area
        if let Some(panel) = self.panels.get(self.focused) {
            panel.render(frame, main_area, true);
        }

        // Status bar
        self.render_status(frame, status_area);

        // Menu bar
        self.render_menu(frame, menu_area);
    }

    fn render_status(&self, frame: &mut Frame, area: Rect) {
        use ratatui::widgets::Paragraph;
        let status_text = format!(
            " Q=Quit | Tab=Switch Focus | ?=Help | Panels: {} | Focus: {}/{} ",
            self.panels.len(),
            self.focused + 1,
            self.panels.len(),
        );
        frame.render_widget(Paragraph::new(status_text), area);
    }

    fn render_menu(&self, frame: &mut Frame, area: Rect) {
        use ratatui::widgets::Paragraph;
        let menu_text = " File  Edit  View  Help ";
        frame.render_widget(Paragraph::new(menu_text), area);
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if event::poll(std::time::Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                // Global shortcuts
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            self.running = false;
                            return Ok(());
                        }
                        KeyCode::Tab => {
                            self.focused = (self.focused + 1) % self.panels.len().max(1);
                            return Ok(());
                        }
                        KeyCode::Char('?') => {
                            info!("Help requested");
                            return Ok(());
                        }
                        _ => {}
                    }

                    // Dispatch to focused panel
                    if let Some(panel) = self.panels.get_mut(self.focused) {
                        let action = panel.handle_input(key);
                        match action {
                            Action::Quit => self.running = false,
                            _ => {} // Other actions handled later
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

/// Check terminal size constraints.
pub fn check_terminal_size() -> io::Result<()> {
    let (cols, rows) = crossterm::terminal::size()?;
    if cols < 80 || rows < 24 {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "terminal too small: {}x{}. Minimum: 80x24",
                cols, rows
            ),
        ));
    }
    Ok(())
}
