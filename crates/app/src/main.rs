mod cli;
mod tui;

use clap::Parser;
use cli::args::Cli;
use text_researcher_core::{DictConfig, GlobalConfig, open_backend};

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

    if cli.init {
        return run_init();
    }

    if cli.interactive {
        return run_tui(&cli);
    }

    cli::batch::run_batch(&cli)
}

fn run_tui(cli: &Cli) -> anyhow::Result<()> {
    tui::app::check_terminal_size()?;

    // Enable mouse capture for double-click support
    ratatui::crossterm::execute!(std::io::stdout(), ratatui::crossterm::event::EnableMouseCapture)?;

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
    ratatui::crossterm::execute!(std::io::stdout(), ratatui::crossterm::event::DisableMouseCapture)?;
    result?;
    Ok(())
}

fn run_init() -> anyhow::Result<()> {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("text-researcher");
    let config_path = config_dir.join("config.json");

    if config_path.exists() {
        // Backup with timestamp suffix
        let now = chrono::Local::now();
        let backup = config_dir.join(format!(
            "config.json.{}",
            now.format("%Y%m%d-%H%M%S")
        ));
        std::fs::copy(&config_path, &backup)
            .map_err(|e| anyhow::anyhow!("failed to backup config: {}", e))?;
        eprintln!("Existing config backed up to {}", backup.display());
    } else {
        std::fs::create_dir_all(&config_dir)
            .map_err(|e| anyhow::anyhow!("failed to create config dir: {}", e))?;
    }

    let default_config = GlobalConfig::default();
    let json = serde_json::to_string_pretty(&default_config)?;

    // Build documented config with comments
    let documented = format!(
        r#"// text-researcher global configuration
// Location: {}
//
// Fields:
//   default_language    — default language for analysis (e.g., "ru", "en")
//   model_dir           — directory with UDPipe .udpipe model files
//   log_level           — logging level: "error", "warn", "info", "debug", "trace"
//   batch_chunk_size    — words per batch chunk for progressive output (default: 10)
//
// Dictionary backend is configured via:
//   DICT_BACKEND env   — "sqlite" (default) or "postgres"
//   DICT_PATH env      — path to SQLite database
//   DATABASE_URL env   — PostgreSQL connection string
//   or in config.json: {{ "dictionary": {{ "backend": "sqlite", ... }} }}
{}
"#,
        config_path.display(),
        json
    );

    std::fs::write(&config_path, &documented)
        .map_err(|e| anyhow::anyhow!("failed to write config: {}", e))?;
    eprintln!("Config written to {}", config_path.display());

    Ok(())
}
