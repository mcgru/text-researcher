use ratatui::crossterm::event::KeyEvent;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use super::{Action, Panel};

/// Debug/log panel showing prefetch activity, focusable and scrollable.
pub struct LogPane {
    messages: Vec<String>,
    current_line: String,
    scroll: usize,
}

const MAX_MESSAGES: usize = 200;
const VISIBLE_LINES: usize = 4;

impl LogPane {
    pub fn new() -> Self {
        LogPane {
            messages: Vec::new(),
            current_line: String::new(),
            scroll: 0,
        }
    }

    /// Flush current line if non-empty, then push a new log message.
    pub fn log(&mut self, msg: &str) {
        self.flush_line();
        self.messages.push(msg.to_string());
        self.scroll_to_bottom();
    }

    /// Log a prefetched word — accumulates on current line, comma-separated.
    pub fn log_prefetch_word(&mut self, word: &str, found: bool) {
        let mark = if found { "✔" } else { "✘" };
        if !self.current_line.is_empty() {
            self.current_line.push_str(", ");
        }
        self.current_line.push_str(&format!("{} {}", mark, word));

        // Flush if line gets long
        if self.current_line.len() > 80 {
            self.flush_line();
        }
    }

    fn flush_line(&mut self) {
        if !self.current_line.is_empty() {
            self.messages.push(std::mem::take(&mut self.current_line));
            self.scroll_to_bottom();
        }
    }

    fn scroll_to_bottom(&mut self) {
        if self.messages.len() > MAX_MESSAGES {
            let excess = self.messages.len() - MAX_MESSAGES;
            self.messages.drain(0..excess);
        }
        self.scroll = self.messages.len().saturating_sub(VISIBLE_LINES);
    }
}

impl Panel for LogPane {
    fn render(&self, frame: &mut Frame, area: Rect, focused: bool) {
        // Build display lines: completed lines + current_line if any
        let mut display_lines: Vec<String> = Vec::new();
        display_lines.extend(self.messages.iter().cloned());
        if !self.current_line.is_empty() {
            display_lines.push(self.current_line.clone());
        }

        let visible_start = self.scroll.min(display_lines.len().saturating_sub(1));
        let visible_end = (visible_start + VISIBLE_LINES).min(display_lines.len());

        let lines: Vec<Line> = if display_lines.is_empty() {
            vec![Line::from("")]
        } else {
            display_lines[visible_start..visible_end]
                .iter()
                .map(|m| Line::from(m.as_str()))
                .collect()
        };

        let style = if focused {
            Style::default().fg(Color::White).bg(Color::DarkGray)
        } else {
            Style::default().fg(Color::Gray).bg(Color::Black)
        };

        frame.render_widget(
            Paragraph::new(lines).style(style).block(Block::default().borders(Borders::ALL).title(" Log ")),
            area,
        );
    }

    fn handle_input(&mut self, key: KeyEvent) -> Action {
        use ratatui::crossterm::event::KeyCode;
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.scroll = self.scroll.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let max = self.messages.len().saturating_sub(VISIBLE_LINES).max(0);
                if self.scroll < max {
                    self.scroll += 1;
                }
            }
            KeyCode::Home | KeyCode::Char('g') => {
                self.scroll = 0;
            }
            KeyCode::End | KeyCode::Char('G') => {
                let total = self.messages.len() + if self.current_line.is_empty() { 0 } else { 1 };
                self.scroll = total.saturating_sub(VISIBLE_LINES).max(0);
            }
            _ => {}
        }
        Action::None
    }

    fn title(&self) -> &str {
        "Log"
    }

    fn focusable(&self) -> bool {
        true
    }
}
