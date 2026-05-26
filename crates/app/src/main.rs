mod cli;
mod tui;

use clap::Parser;
use cli::args::Cli;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if cli.verbose {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    } else {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    }

    if let Some(language) = &cli.download_model {
        return cli::batch::run_download_model(language);
    }

    if cli.interactive {
        return run_tui();
    }

    cli::batch::run_batch(&cli)
}

fn run_tui() -> anyhow::Result<()> {
    tui::app::check_terminal_size()?;

    let mut terminal = ratatui::init();
    let mut app = tui::app::AppState::new();

    // Placeholder panel
    struct PlaceholderPanel {
        title: String,
    }
    impl tui::panels::Panel for PlaceholderPanel {
        fn render(&self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect, _focused: bool) {
            use ratatui::widgets::Paragraph;
            frame.render_widget(
                Paragraph::new(format!("{} — press Tab to switch panels, q to quit", self.title)),
                area,
            );
        }
        fn handle_input(&mut self, _key: ratatui::crossterm::event::KeyEvent) -> tui::panels::Action {
            tui::panels::Action::None
        }
        fn title(&self) -> &str {
            &self.title
        }
    }

    app.add_panel(Box::new(PlaceholderPanel { title: "TextPane".into() }));
    app.add_panel(Box::new(PlaceholderPanel { title: "PropsPane".into() }));

    let result = app.run(&mut terminal);
    ratatui::restore();
    result?;
    Ok(())
}
