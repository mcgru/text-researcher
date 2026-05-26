pub mod props;
pub mod text;

use ratatui::crossterm::event::KeyEvent;
use ratatui::layout::Rect;
use ratatui::Frame;

/// Actions that panels can trigger.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    None,
    Quit,
    Save,
    OpenFile,
    SwitchLanguage(String),
    EditWord { word_index: usize },
    SwitchFocus(usize),
    Help,
}

/// Trait for all TUI panels.
pub trait Panel {
    fn render(&self, frame: &mut Frame, area: Rect, focused: bool);
    fn handle_input(&mut self, key: KeyEvent) -> Action;
    fn title(&self) -> &str;
    fn focusable(&self) -> bool {
        true
    }
}

pub use props::PropsPane;
pub use text::TextPane;
