mod cli;
mod tui;

use clap::Parser;
use cli::args::Cli;
use text_researcher_core::OpenCorporaDict;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Configure parallelism
    if let Some(jobs) = cli.jobs {
        rayon::ThreadPoolBuilder::new()
            .num_threads(jobs)
            .build_global()
            .map_err(|e| anyhow::anyhow!("failed to set thread pool: {}", e))?;
    }
    // Default: rayon uses all logical CPUs

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

    // Load OpenCorpora dictionary if available
    let dict_path = std::env::var("DICT_PATH").unwrap_or_else(|_| ".data/dict.opcorpora.sqlite3.db".into());
    match OpenCorporaDict::open(&dict_path) {
        Ok(dict) => {
            eprintln!("Dictionary loaded: {}", dict_path);
            app.set_dictionary(dict);
        }
        Err(e) => {
            eprintln!("Dictionary not available: {} (set DICT_PATH env var)", e);
        }
    }

    let result = app.run(&mut terminal);
    ratatui::restore();
    result?;
    Ok(())
}
