use ratatui::crossterm::event::KeyEvent;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use super::{Action, Panel};

/// Debug/log panel showing prefetch activity, focusable and scrollable.
pub struct LogPane {
    messages: Vec<String>,
    scroll: usize,
}

const MAX_MESSAGES: usize = 100;
const VISIBLE_LINES: usize = 4;

impl LogPane {
    pub fn new() -> Self {
        LogPane {
            messages: Vec::new(),
            scroll: 0,
        }
    }

    pub fn log(&mut self, msg: &str) {
        self.messages.push(msg.to_string());
        if self.messages.len() > MAX_MESSAGES {
            self.messages.remove(0);
        }
        // Auto-scroll to bottom
        self.scroll = self.messages.len().saturating_sub(VISIBLE_LINES);
    }
}

impl Panel for LogPane {
    fn render(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let visible_start = self.scroll.min(self.messages.len().saturating_sub(1));
        let visible_end = (visible_start + VISIBLE_LINES).min(self.messages.len());

        let lines: Vec<Line> = if self.messages.is_empty() {
            vec![Line::from("")]
        } else {
            self.messages[visible_start..visible_end]
                .iter()
                .map(|m| Line::from(m.as_str()))
                .collect()
        };

        let style = if focused {
            Style::default().fg(Color::White).bg(Color::DarkGray)
        } else {
            Style::default().fg(Color::Gray).bg(Color::Black)
        };

        frame.render_widget(Paragraph::new(lines).style(style), area);
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
                self.scroll = self.messages.len().saturating_sub(VISIBLE_LINES).max(0);
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
