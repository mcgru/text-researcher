use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
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

    pub fn word_at(&self, index: usize) -> Option<&str> {
        self.words.get(index).map(|w| w.word.as_str())
    }

    pub fn cursor_word_index(&self) -> usize {
        self.cursor_word_index
    }

    pub fn word_count(&self) -> usize {
        self.words.len()
    }

    fn move_cursor(&mut self, delta: isize) -> bool {
        let new = self.cursor_word_index as isize + delta;
        if new >= 0 && (new as usize) < self.words.len() {
            self.cursor_word_index = new as usize;
            true
        } else {
            false
        }
    }

    fn move_to_line(&mut self, delta: isize) -> bool {
        let line_width = 20;
        let new = self.cursor_word_index as isize + delta * line_width;
        if new >= 0 && (new as usize) < self.words.len() {
            self.cursor_word_index = new as usize;
            true
        } else if new < 0 {
            self.cursor_word_index = 0;
            true
        } else {
            self.cursor_word_index = self.words.len().saturating_sub(1);
            true
        }
    }

    fn goto_start(&mut self) -> bool {
        self.cursor_word_index = 0;
        true
    }

    fn goto_end(&mut self) -> bool {
        if !self.words.is_empty() {
            self.cursor_word_index = self.words.len() - 1;
        }
        true
    }
}

impl Panel for TextPane {
    fn render(&self, frame: &mut Frame, area: Rect, _focused: bool) {
        let highlight_style = Style::default().bg(Color::DarkGray).fg(Color::White);

        let area_width = area.width as usize;
        if area_width < 3 {
            return;
        }

        // Build lines wrapping at area width
        let mut lines: Vec<Line> = Vec::new();
        let mut current_line: Vec<Span> = Vec::new();
        let mut current_width = 0usize;
        let mut word_idx = 0usize;

        for ws in &self.words {
            let word_width = ws.word.chars().count();
            let space_width = if current_width > 0 { 1 } else { 0 };

            if current_width + space_width + word_width > area_width && current_width > 0 {
                // Wrap to new line
                lines.push(Line::from(std::mem::take(&mut current_line)));
                current_width = 0;
            }

            if current_width > 0 {
                current_line.push(Span::raw(" "));
                current_width += 1;
            }

            let span = if word_idx == self.cursor_word_index {
                Span::styled(&ws.word, highlight_style)
            } else {
                Span::raw(&ws.word)
            };
            current_line.push(span);
            current_width += word_width;
            word_idx += 1;
        }

        if !current_line.is_empty() {
            lines.push(Line::from(current_line));
        }

        let paragraph = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title(" Text "));
        frame.render_widget(paragraph, area);
    }

    fn handle_input(&mut self, key: KeyEvent) -> Action {
        let _moved = match key.code {
            KeyCode::Right | KeyCode::Char('l') => self.move_cursor(1),
            KeyCode::Left | KeyCode::Char('h') => self.move_cursor(-1),
            KeyCode::Down | KeyCode::Char('j') => self.move_to_line(1),
            KeyCode::Up | KeyCode::Char('k') => self.move_to_line(-1),
            KeyCode::Char('w') => self.move_cursor(1),
            KeyCode::Char('b') => self.move_cursor(-1),
            KeyCode::Char('e') => true,
            KeyCode::Char('0') => self.goto_start(),
            KeyCode::Char('$') => self.goto_end(),
            KeyCode::Char('g') => self.goto_start(),
            KeyCode::Char('G') => self.goto_end(),
            _ => false,
        };
        Action::None
    }

    fn title(&self) -> &str {
        "Text"
    }
}

/// Parse text into words, cleaning punctuation/quotes, tracking byte offsets.
fn parse_words(text: &str) -> Vec<WordSpan> {
    let mut words = Vec::new();
    let mut byte_pos = 0;

    for token in text.split_whitespace() {
        if let Some(pos) = text[byte_pos..].find(token) {
            let start = byte_pos + pos;
            let end = start + token.len();
            let cleaned = clean_word(token);
            if !cleaned.is_empty() {
                words.push(WordSpan {
                    word: cleaned,
                    start_byte: start,
                    end_byte: end,
                });
            }
            byte_pos = end;
        }
    }

    words
}

/// Strip punctuation, quotes, and non-letter/digit/hyphen characters from word edges.
fn clean_word(word: &str) -> String {
    word.trim_matches(|c: char| !c.is_alphanumeric() && c != '-').to_string()
}
