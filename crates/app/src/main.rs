mod cli;
mod tui;

use clap::Parser;
use cli::args::Cli;
use text_researcher_core::{DictConfig, GlobalConfig, open_backend};
use text_researcher_core::morphology;

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

    if cli.help_features {
        return run_help_features();
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

fn run_help_features() -> anyhow::Result<()> {
    println!("\nMorphological Features Reference");
    println!("===============================\n");
    println!("{:<6} {:<14} {:<16} {:<10} {}", "Short", "Name (EN)", "Short RU", "Med RU", "Description");
    println!("{:-<6} {:-<14} {:-<16} {:-<10} {:-<50}", "", "", "", "", "");
    for f in morphology::FEATURES {
        println!(
            "{:<6} {:<14} {:<16} {:<10} {}",
            f.short_en, f.name_en, f.short_ru, f.medium_ru, ""
        );
        println!(
            "{:<6} {:<14} {:<16} {:<10}   Values: {}",
            "", "", "", "", f.possible_values.join(", ")
        );
    }

    println!("\n\nParts of Speech Reference");
    println!("=========================\n");
    println!("{:<6} {:<10} {:<16} {}", "Code", "Short RU", "Med RU", "Description");
    println!("{:-<6} {:-<10} {:-<16} {:-<50}", "", "", "", "");
    for p in morphology::POS_TYPES {
        println!(
            "{:<6} {:<10} {:<16} {}",
            p.code, p.short_ru, p.medium_ru, p.description_ru
        );
    }
    println!();

    Ok(())
}

fn run_init() -> anyhow::Result<()> {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("text-researcher");
    std::fs::create_dir_all(&config_dir)?;

    let now = chrono::Local::now();
    let ts = now.format("%Y%m%d-%H%M%S");

    let config_path = if config_dir.join("config.json").exists() {
        config_dir.join(format!("config.json.{}", ts))
    } else {
        config_dir.join("config.json")
    };
    let doc_path = config_dir.join(format!("config.json.txt.{}", ts));

    let default_config = GlobalConfig::default();
    let json = serde_json::to_string_pretty(&default_config)?;

    // Write clean config
    std::fs::write(&config_path, &json)?;
    eprintln!("Config: {}", config_path.display());

    // Write documentation
    let doc = format!(
        "text-researcher global configuration\n\
         Location: {config}\n\
         \n\
         Fields:\n\
           default_language     - default language (e.g., \"ru\", \"en\")\n\
           model_dir            - directory with UDPipe .udpipe model files\n\
           log_level            - logging: \"error\", \"warn\", \"info\", \"debug\", \"trace\"\n\
           batch_chunk_size     - words per batch chunk (default: 10)\n\
         \n\
         Dictionary backend (env vars or config.json dictionary section):\n\
           DICT_BACKEND  - \"sqlite\" (default) or \"postgres\"\n\
           DICT_PATH     - path to SQLite .db file\n\
           DATABASE_URL  - PostgreSQL connection string\n\
         \n\
         Example config.json:\n\
{json}",
        config = config_path.display(),
        json = json,
    );
    std::fs::write(&doc_path, &doc)?;
    eprintln!("Docs:    {}", doc_path.display());

    Ok(())
}
