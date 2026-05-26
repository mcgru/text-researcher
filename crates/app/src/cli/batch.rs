use std::fs;
use std::path::Path;

use text_researcher_core::{Analyzer, GlobalConfig, ModelManager};

use crate::cli::args::Cli;

/// Run batch analysis: load text, analyze, output JSON.
pub fn run_batch(cli: &Cli) -> anyhow::Result<()> {
    let input_path = cli.input.as_ref().expect("input file required for batch mode");

    // Load config
    let config = GlobalConfig::load()?;
    let language = cli.language.clone().unwrap_or(config.default_language.clone());

    // Read input file
    let text = fs::read_to_string(input_path)
        .map_err(|e| anyhow::anyhow!("failed to read input file '{}': {}", input_path.display(), e))?;

    // Initialize model and analyzer
    let model_dir = std::env::var("UDPIPE_MODEL_DIR")
        .unwrap_or(config.model_dir.clone());
    let model_manager = ModelManager::new(model_dir.into());
    let mut analyzer = Analyzer::new(model_manager);

    // Analyze
    let sentences = analyzer
        .analyze(&text, &language)
        .map_err(|e| anyhow::anyhow!("analysis failed: {}", e))?;

    // Serialize to JSON
    let json = serde_json::to_string_pretty(sentences)?;

    // Output
    if let Some(output_path) = &cli.output {
        fs::write(output_path, &json)
            .map_err(|e| anyhow::anyhow!("failed to write output: {}", e))?;
        eprintln!("Output written to {}", output_path.display());
    } else {
        println!("{}", json);
    }

    Ok(())
}

/// Download a UDPipe model for the given language.
pub fn run_download_model(language: &str) -> anyhow::Result<()> {
    let config = GlobalConfig::load()?;
    let model_dir = std::env::var("UDPIPE_MODEL_DIR")
        .unwrap_or(config.model_dir.clone());
    let model_dir = Path::new(&model_dir);

    if !model_dir.exists() {
        fs::create_dir_all(model_dir)?;
    }

    let model_path = model_dir.join(format!("{}.udpipe", language));
    if model_path.exists() {
        eprintln!("Model already exists: {}", model_path.display());
        return Ok(());
    }

    // UDPipe 2 models: download from LINDAT/CLARIN
    let url = format!(
        "https://lindat.mff.cuni.cz/repository/xmlui/bitstream/handle/11234/1-XXXX/{}.udpipe",
        language
    );

    eprintln!("Downloading model for '{}'...", language);
    eprintln!("URL: {}", url);
    eprintln!("Note: automatic download requires the model URL.");
    eprintln!("Please download manually from https://lindat.mff.cuni.cz/repository/xmlui/handle/11234/1-4923");
    eprintln!("and place it at: {}", model_path.display());

    Err(anyhow::anyhow!(
        "automatic download not yet implemented. Place {}.udpipe in {}",
        language,
        model_dir.display()
    ))
}
