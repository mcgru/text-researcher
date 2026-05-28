use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use super::{Action, Panel};

/// Displays morphological properties of the word under cursor.
///
/// Properties are displayed as: "name : alias, code" where:
/// - `name` — Russian full name (e.g., "единственное число")
/// - `alias` — Russian abbreviation (e.g., "ед")
/// - `code` — English code from UD/OpenCorpora (e.g., "sing")
///
/// Reference: `docs/morph-features.json`, `docs/table-morph-types.json`
pub struct PropsPane {
    word: String,
    lemma: String,
    upostag: String,
    features: Vec<(String, String, String)>, // (name_ru, alias_ru, code_en)
}

impl PropsPane {
    pub fn new() -> Self {
        PropsPane {
            word: String::new(),
            lemma: String::new(),
            upostag: String::new(),
            features: Vec::new(),
        }
    }

    /// Update with dictionary entry data.
    /// `features`: Vec of (name_ru, alias_ru, code_en)
    pub fn update(&mut self, word: &str, lemma: &str, pos: &str, features: Vec<(String, String, String)>) {
        self.word = word.to_string();
        self.lemma = lemma.to_string();
        self.upostag = pos.to_string();
        self.features = features;
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
                Span::styled(format!(", {}", &self.upostag), Style::default().fg(Color::DarkGray)),
            ]));

            // Features
            for (name, alias, code) in &self.features {
                lines.push(Line::from(vec![
                    Span::styled(format!("{} : ", name), Style::default().fg(Color::Yellow)),
                    Span::raw(alias),
                    Span::styled(format!(", {}", code), Style::default().fg(Color::DarkGray)),
                ]));
            }
        }

        frame.render_widget(
            Paragraph::new(lines)
                .wrap(Wrap { trim: false })
                .block(Block::default().borders(Borders::ALL).title(" Properties ")),
            area,
        );
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
        // Universal Dependencies
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
        "NUMR" => "Числительное".into(),
        "PROPN" => "Имя собственное".into(),
        "PUNCT" => "Пунктуация".into(),
        "SYM" => "Символ".into(),
        "X" => "Другое".into(),
        // OpenCorpora
        "ADJF" => "Прилагательное (полн.)".into(),
        "ADJS" => "Прилагательное (кратк.)".into(),
        "ADVB" => "Наречие".into(),
        "COMP" => "Компаратив".into(),
        "PRTF" => "Причастие (полн.)".into(),
        "PRTS" => "Причастие (кратк.)".into(),
        "GRND" => "Деепричастие".into(),
        "INFN" => "Инфинитив".into(),
        "PRED" => "Предикатив".into(),
        "PREP" => "Предлог".into(),
        "PRCL" => "Частица".into(),
        "NPRO" => "Местоимение-сущ.".into(),
        _ => tag.to_string(),
    }
}
