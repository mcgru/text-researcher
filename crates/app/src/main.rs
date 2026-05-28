mod cli;
mod tui;

use clap::Parser;
use cli::args::Cli;
use text_researcher_core::{DictConfig, open_backend};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if let Some(jobs) = cli.jobs {
        rayon::ThreadPoolBuilder::new()
            .num_threads(jobs)
            .build_global()
            .map_err(|e| anyhow::anyhow!("thread pool: {}", e))?;
    }

    if cli.verbose {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    } else {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    }

    if let Some(language) = &cli.download_model {
        return cli::batch::run_download_model(language);
    }

    if cli.interactive {
        return run_tui(&cli);
    }

    cli::batch::run_batch(&cli)
}

fn run_tui(cli: &Cli) -> anyhow::Result<()> {
    tui::app::check_terminal_size()?;

    let mut terminal = ratatui::init();
    let mut text_pane = tui::panels::TextPane::new();
    if let Some(input) = &cli.input {
        text_pane.set_text(&std::fs::read_to_string(input)?);
    }
    let props_pane = tui::panels::PropsPane::new();
    let mut app = tui::app::AppState::new(text_pane, props_pane);
    app.set_prefetch_count(cli.prefetch);

    // Load dictionary
    let dict_config = DictConfig::from_env_or_default();
    match open_backend(&dict_config) {
        Ok(dict) => {
            eprintln!("Dictionary loaded (backend: {})", dict_config.backend);
            app.set_dictionary(dict, dict_config);
        }
        Err(e) => {
            eprintln!("Dictionary not available: {} (set DICT_BACKEND/DICT_PATH/DATABASE_URL)", e);
        }
    }

    let result = app.run(&mut terminal);
    ratatui::restore();
    result?;
    Ok(())
}
