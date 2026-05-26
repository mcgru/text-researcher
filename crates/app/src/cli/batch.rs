use std::fs;
use std::path::Path;

use text_researcher_core::{GlobalConfig, OpenCorporaDict};

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
    let dict_path = std::env::var("DICT_PATH")
        .unwrap_or_else(|_| ".data/dict.opcorpora.sqlite3.db".into());
    let dict = OpenCorporaDict::open(&dict_path)
        .map_err(|e| anyhow::anyhow!("failed to open dictionary: {}", e))?;

    // Extract words
    let words: Vec<&str> = text.split_whitespace().collect();

    // Parallel lookup
    let results = dict.lookup_batch(&words);

    // Build output
    let output: String = match cli.format.as_str() {
        "json" => {
            let entries: Vec<String> = words.iter().map(|word| {
                let word_clean = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '-');
                if let Some(entry_list) = results.get(*word) {
                    // Pick the first entry with actual grammemes (skip virtual-only lemmas)
                    let best = entry_list.iter().find(|e| {
                        !e.grammemes.is_empty() || e.pos.is_some()
                    }).or_else(|| entry_list.first());

                    if let Some(e) = best {
                        let mut feats = serde_json::Map::new();
                        if let Some(ref pos) = e.pos {
                            feats.insert("pos".into(), pos.clone().into());
                        }
                        for gram in &e.grammemes {
                            feats.insert(gram.name.clone(), gram.alias.clone().into());
                        }
                        let features_str = serde_json::to_string(&feats).unwrap_or_else(|_| "{}".into());
                        format!(
                            r#"{{"word":"{}","lemma":"{}","features":{}}}"#,
                            *word, e.lemma, features_str
                        )
                    } else {
                        format!(r#"{{"word":"{}","lemma":"{}","features":{{}}}}"#, *word, word_clean)
                    }
                } else {
                    format!(r#"{{"word":"{}","lemma":"{}","features":{{}}}}"#, *word, word_clean)
                }
            }).collect();
            entries.join("\n")
        }
        _ => {
            // Text format: word: PROP=val, PROP=val, ...
            let lines: Vec<String> = words.iter().map(|word| {
                if let Some(entry_list) = results.get(*word) {
                    if let Some(entry) = entry_list.first() {
                        let mut props = Vec::new();
                        props.push(format!("lemma={}", entry.lemma));
                        if let Some(ref pos) = entry.pos {
                            props.push(format!("POS={}", pos));
                        }
                        for gram in &entry.grammemes {
                            props.push(format!("{}={}", gram.name, gram.alias));
                        }
                        return format!("{}: {}", word, props.join(", "));
                    }
                }
                format!("{}: (не найдено)", word)
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
