use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use super::{Action, Panel};

/// The main text display panel with word-by-word cursor navigation.
pub struct TextPane {
    text: String,
    words: Vec<WordSpan>,
    cursor_word_index: usize,
    scroll_offset: usize,
}

#[derive(Debug, Clone)]
struct WordSpan {
    word: String,
    start_byte: usize,
    end_byte: usize,
}

impl TextPane {
    pub fn new() -> Self {
        TextPane {
            text: String::new(),
            words: Vec::new(),
            cursor_word_index: 0,
            scroll_offset: 0,
        }
    }

    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
        self.words = parse_words(text);
        self.cursor_word_index = 0;
        self.scroll_offset = 0;
    }

    pub fn current_word(&self) -> Option<&str> {
        self.words.get(self.cursor_word_index).map(|w| w.word.as_str())
    }

    pub fn cursor_word_index(&self) -> usize {
        self.cursor_word_index
    }

    fn move_cursor(&mut self, delta: isize) {
        let new = self.cursor_word_index as isize + delta;
        if new >= 0 && (new as usize) < self.words.len() {
            self.cursor_word_index = new as usize;
        }
    }

    fn move_to_line(&mut self, delta: isize) {
        // Approximate: move roughly one visual line's worth of words
        let line_width = 20; // rough estimate
        let new = self.cursor_word_index as isize + delta * line_width;
        if new >= 0 && (new as usize) < self.words.len() {
            self.cursor_word_index = new as usize;
        } else if new < 0 {
            self.cursor_word_index = 0;
        } else {
            self.cursor_word_index = self.words.len().saturating_sub(1);
        }
    }

    fn goto_start(&mut self) {
        self.cursor_word_index = 0;
    }

    fn goto_end(&mut self) {
        if !self.words.is_empty() {
            self.cursor_word_index = self.words.len() - 1;
        }
    }
}

impl Panel for TextPane {
    fn render(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let highlight_style = if focused {
            Style::default().bg(Color::DarkGray).fg(Color::White)
        } else {
            Style::default().bg(Color::Gray)
        };

        let mut spans: Vec<Span> = Vec::new();
        for (i, ws) in self.words.iter().enumerate() {
            if i == self.cursor_word_index {
                spans.push(Span::styled(&ws.word, highlight_style));
            } else {
                spans.push(Span::raw(&ws.word));
            }
            spans.push(Span::raw(" "));
        }

        let line = Line::from(spans);
        frame.render_widget(Paragraph::new(line), area);
    }

    fn handle_input(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Right | KeyCode::Char('l') => self.move_cursor(1),
            KeyCode::Left | KeyCode::Char('h') => self.move_cursor(-1),
            KeyCode::Down | KeyCode::Char('j') => self.move_to_line(1),
            KeyCode::Up | KeyCode::Char('k') => self.move_to_line(-1),
            KeyCode::Char('w') => self.move_cursor(1),   // next word
            KeyCode::Char('b') => self.move_cursor(-1),  // prev word
            KeyCode::Char('e') => { /* end of word: fine for now */ }
            KeyCode::Char('0') => self.goto_start(),
            KeyCode::Char('$') => self.goto_end(),
            KeyCode::Char('g') => self.goto_start(), // simplified gg
            KeyCode::Char('G') => self.goto_end(),
            _ => {}
        }
        Action::None
    }

    fn title(&self) -> &str {
        "Text"
    }
}

/// Parse text into words, tracking byte offsets.
fn parse_words(text: &str) -> Vec<WordSpan> {
    let mut words = Vec::new();
    let mut byte_pos = 0;

    for word in text.split_whitespace() {
        if let Some(pos) = text[byte_pos..].find(word) {
            let start = byte_pos + pos;
            let end = start + word.len();
            words.push(WordSpan {
                word: word.to_string(),
                start_byte: start,
                end_byte: end,
            });
            byte_pos = end;
        }
    }

    words
}
