mod cli;

use clap::Parser;
use cli::args::Cli;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if cli.verbose {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    } else {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    }

    // Dispatch
    if let Some(language) = &cli.download_model {
        return cli::batch::run_download_model(language);
    }

    if cli.interactive {
        println!("TUI mode not yet implemented (coming in Epic 2)");
        return Ok(());
    }

    // Batch mode (default)
    cli::batch::run_batch(&cli)
}
