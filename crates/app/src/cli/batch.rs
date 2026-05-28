use std::fs;
use std::io::{self, BufWriter, Write};
use std::path::Path;

use text_researcher_core::{DictConfig, GlobalConfig, open_backend};
use text_researcher_core::morphology::compact_features;

use crate::cli::args::Cli;

const CHUNK_SIZE: usize = 100;

/// Run batch analysis: lookup all words in dictionary, output txt or json.
pub fn run_batch(cli: &Cli) -> anyhow::Result<()> {
    let input_path = cli.input.as_ref().expect("input file required for batch mode");

    // Load config
    let _config = GlobalConfig::load()?;

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

    // Output stream: file or stdout
    let mut writer: Box<dyn Write> = if let Some(output_path) = &cli.output {
        let file = fs::File::create(output_path)
            .map_err(|e| anyhow::anyhow!("failed to create output file: {}", e))?;
        Box::new(BufWriter::new(file))
    } else {
        Box::new(io::stdout().lock())
    };

    let is_json = cli.format == "json";

    // Process in chunks
    for chunk in words.chunks(CHUNK_SIZE) {
        let refs: Vec<&str> = chunk.iter().map(|w| w.as_str()).collect();
        let results = dict.lookup_batch(&refs);

        for word in chunk {
            let line = if is_json {
                let props_str = build_compact(word, &results);
                let indiv = build_individual(word, &results);
                format!(r#"{{"word":"{}","props":"{}"{}}}"#, word, props_str, indiv)
            } else {
                let props_str = build_compact(word, &results);
                format!("{}\t{}", word, props_str)
            };
            writeln!(writer, "{}", line)?;
        }
        writer.flush()?;
    }

    if cli.output.is_some() {
        eprintln!("Output written to {}", cli.output.as_ref().unwrap().display());
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

/// Build individual property pairs for JSON output.
fn build_individual(word: &str, results: &std::collections::HashMap<String, Vec<text_researcher_core::DictEntry>>) -> String {
    if let Some(entry_list) = results.get(word) {
        let best = entry_list.iter().find(|e| !e.grammemes.is_empty() || e.pos.is_some())
            .or_else(|| entry_list.first());
        if let Some(e) = best {
            let mut parts = vec![format!(r#""lem":"{}""#, e.lemma)];
            if let Some(ref pos) = e.pos {
                parts.push(format!(r#""pos":"{}""#, pos));
            }
            for gram in &e.grammemes {
                let short = text_researcher_core::morphology::feature_for_value(&gram.code);
                parts.push(format!(r#""{}":"{}""#, short, gram.code));
            }
            return format!(",{}", parts.join(","));
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
