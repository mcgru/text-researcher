use std::collections::HashMap;

use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use super::{Action, Panel};

/// Displays morphological properties of the word under cursor.
pub struct PropsPane {
    word: String,
    lemma: String,
    upostag: String,
    features: HashMap<String, String>,
}

impl PropsPane {
    pub fn new() -> Self {
        PropsPane {
            word: String::new(),
            lemma: String::new(),
            upostag: String::new(),
            features: HashMap::new(),
        }
    }

    /// Update with token data from the analyzer.
    pub fn update(&mut self, word: &str, lemma: &str, upostag: &str, features: &HashMap<String, String>) {
        self.word = word.to_string();
        self.lemma = lemma.to_string();
        self.upostag = upostag.to_string();
        self.features = features.clone();
    }

    /// Clear props (no word selected).
    pub fn clear(&mut self) {
        self.word.clear();
        self.lemma.clear();
        self.upostag.clear();
        self.features.clear();
    }
}

impl Panel for PropsPane {
    fn render(&self, frame: &mut Frame, area: Rect, _focused: bool) {
        let mut lines = Vec::new();

        if self.word.is_empty() {
            lines.push(Line::from(Span::styled(
                "Нет данных",
                Style::default().fg(Color::DarkGray),
            )));
        } else {
            // Word at top
            lines.push(Line::from(vec![
                Span::styled(&self.word, Style::default().fg(Color::White).add_modifier(ratatui::style::Modifier::BOLD)),
            ]));
            lines.push(Line::from(""));

            // Lemma
            lines.push(Line::from(vec![
                Span::styled("Лемма : ", Style::default().fg(Color::Cyan)),
                Span::raw(&self.lemma),
            ]));

            // Part of speech
            lines.push(Line::from(vec![
                Span::styled("Часть речи : ", Style::default().fg(Color::Cyan)),
                Span::raw(upostag_human(&self.upostag)),
            ]));

            // Features
            let mut feats: Vec<&String> = self.features.keys().collect();
            feats.sort();
            for key in feats {
                if let Some(value) = self.features.get(key) {
                    lines.push(Line::from(vec![
                        Span::styled(format!("{} : ", key), Style::default().fg(Color::Yellow)),
                        Span::raw(value),
                    ]));
                }
            }
        }

        frame.render_widget(Paragraph::new(lines), area);
    }

    fn handle_input(&mut self, key: KeyEvent) -> Action {
        if key.code == KeyCode::Char('e') {
            return Action::EditWord { word_index: 0 }; // word_index filled by app
        }
        Action::None
    }

    fn title(&self) -> &str {
        "Properties"
    }
}

/// Human-readable UPOS tag name.
fn upostag_human(tag: &str) -> String {
    match tag {
        "NOUN" => "Существительное".into(),
        "VERB" => "Глагол".into(),
        "ADJ" => "Прилагательное".into(),
        "ADV" => "Наречие".into(),
        "PRON" => "Местоимение".into(),
        "DET" => "Определитель".into(),
        "ADP" => "Предлог".into(),
        "CONJ" => "Союз".into(),
        "PART" => "Частица".into(),
        "INTJ" => "Междометие".into(),
        "NUM" => "Числительное".into(),
        "PROPN" => "Имя собственное".into(),
        "PUNCT" => "Пунктуация".into(),
        "SYM" => "Символ".into(),
        "X" => "Другое".into(),
        _ => tag.to_string(),
    }
}
