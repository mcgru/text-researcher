use std::fs;
use std::path::Path;

use text_researcher_core::{DictConfig, GlobalConfig, open_backend};
use text_researcher_core::morphology::compact_features;

use crate::cli::args::Cli;

/// Run batch analysis: lookup all words in dictionary, output txt or json.
pub fn run_batch(cli: &Cli) -> anyhow::Result<()> {
    let input_path = cli.input.as_ref().expect("input file required for batch mode");

    // Load config
    let config = GlobalConfig::load()?;

    // Read input file
    let text = fs::read_to_string(input_path)
        .map_err(|e| anyhow::anyhow!("failed to read input file '{}': {}", input_path.display(), e))?;

    // Load dictionary
    let dict_config = DictConfig::from_env_or_default();
    let dict = open_backend(&dict_config)
        .map_err(|e| anyhow::anyhow!("failed to open dictionary: {}", e))?;

    // Extract words and clean punctuation
    let words: Vec<String> = text.split_whitespace()
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric() && c != '-').to_string())
        .filter(|w| !w.is_empty())
        .collect();
    let word_refs: Vec<&str> = words.iter().map(|w| w.as_str()).collect();

    // Parallel lookup
    let results = dict.lookup_batch(&word_refs);

    // Build output
    let output: String = match cli.format.as_str() {
        "json" => {
            let entries: Vec<String> = words.iter().map(|word| {
                let props_str = build_compact(word, &results);
                format!(r#"{{"word":"{}","props":"{}"}}"#, word, props_str)
            }).collect();
            entries.join("\n")
        }
        _ => {
            // Text format: word\t@lem:..., @pos:...
            let lines: Vec<String> = words.iter().map(|word| {
                let props_str = build_compact(word, &results);
                format!("{}\t{}", word, props_str)
            }).collect();
            lines.join("\n")
        }
    };

    // Output
    if let Some(output_path) = &cli.output {
        fs::write(output_path, &output)
            .map_err(|e| anyhow::anyhow!("failed to write output: {}", e))?;
        eprintln!("Output written to {}", output_path.display());
    } else {
        println!("{}", output);
    }

    Ok(())
}

/// Build compact feature string for a word from lookup results.
fn build_compact(word: &str, results: &std::collections::HashMap<String, Vec<text_researcher_core::DictEntry>>) -> String {
    if let Some(entry_list) = results.get(word) {
        let best = entry_list.iter().find(|e| !e.grammemes.is_empty() || e.pos.is_some())
            .or_else(|| entry_list.first());
        if let Some(e) = best {
            let codes: Vec<String> = e.grammemes.iter().map(|g| g.code.clone()).collect();
            let pos = e.pos.clone().unwrap_or_default();
            return compact_features(&e.lemma, &pos, &codes);
        }
    }
    String::new()
}

/// Download a UDPipe model for the given language (stub).
pub fn run_download_model(language: &str) -> anyhow::Result<()> {
    let model_dir_path = std::env::var("UDPIPE_MODEL_DIR")
        .unwrap_or_else(|_| "./models".into());
    let model_dir = Path::new(&model_dir_path);

    if !model_dir.exists() {
        fs::create_dir_all(model_dir)?;
    }

    let model_path = model_dir.join(format!("{}.udpipe", language));
    if model_path.exists() {
        eprintln!("Model already exists: {}", model_path.display());
        return Ok(());
    }

    eprintln!("Please download the model for '{}' manually from:", language);
    eprintln!("  https://lindat.mff.cuni.cz/repository/xmlui/handle/11234/1-4923");
    eprintln!("and place it at: {}", model_path.display());

    Err(anyhow::anyhow!(
        "automatic download not yet implemented. Place {}.udpipe in {}",
        language,
        model_dir.display()
    ))
}
