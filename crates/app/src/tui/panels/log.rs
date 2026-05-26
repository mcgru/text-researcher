use ratatui::crossterm::event::KeyEvent;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use super::{Action, Panel};

/// Debug/log panel showing prefetch activity at the bottom of the screen.
pub struct LogPane {
    messages: Vec<String>,
    max_lines: usize,
}

impl LogPane {
    pub fn new() -> Self {
        LogPane {
            messages: Vec::new(),
            max_lines: 6,
        }
    }

    pub fn log(&mut self, msg: &str) {
        self.messages.push(msg.to_string());
        if self.messages.len() > self.max_lines {
            self.messages.remove(0);
        }
    }

    pub fn log_prefetch_done(&mut self, word: &str, found: bool) {
        let mark = if found { "✔" } else { "✘" };
        self.log(&format!(" {} {}", mark, word));
    }

    pub fn log_prefetch_start(&mut self, word: &str) {
        self.log(&format!(" … {}", word));
    }
}

impl Panel for LogPane {
    fn render(&self, frame: &mut Frame, area: Rect, _focused: bool) {
        let lines: Vec<Line> = self.messages.iter().map(|m| {
            Line::from(m.as_str())
        }).collect();

        let style = Style::default().fg(Color::Gray).bg(Color::Black);
        frame.render_widget(
            Paragraph::new(lines).style(style),
            area,
        );
    }

    fn handle_input(&mut self, _key: KeyEvent) -> Action {
        Action::None
    }

    fn title(&self) -> &str {
        "Log"
    }

    fn focusable(&self) -> bool {
        false
    }
}
