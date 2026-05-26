use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single token with morphological analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub id: u32,
    pub word: String,
    pub lemma: String,
    pub upostag: String,
    pub xpostag: String,
    pub feats: HashMap<String, String>,
    pub head: Option<i32>,
    pub deprel: Option<String>,
}

/// A sentence containing tokens.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sentence {
    pub text: String,
    pub tokens: Vec<Token>,
}

/// Parse CoNLL-U format output from UDPipe.
///
/// Expected format (tab-separated columns):
/// ID FORM LEMMA UPOS XPOS FEATS HEAD DEPREL DEPS MISC
/// Lines starting with `#` are comments (metadata).
pub fn parse_conllu(input: &str) -> Result<Vec<Sentence>, crate::error::UdpipelineError> {
    let mut sentences = Vec::new();
    let mut current_tokens = Vec::new();
    let mut current_text = String::new();

    for line in input.lines() {
        let line = line.trim();

        if line.is_empty() {
            // Empty line: end of sentence
            if !current_tokens.is_empty() {
                sentences.push(Sentence {
                    text: std::mem::take(&mut current_text),
                    tokens: std::mem::take(&mut current_tokens),
                });
            }
            continue;
        }

        if line.starts_with('#') {
            // Comment line
            if let Some(text) = line.strip_prefix("# text = ") {
                current_text = text.to_string();
            }
            continue;
        }

        // Parse token line: ID FORM LEMMA UPOS XPOS FEATS HEAD DEPREL DEPS MISC
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() < 6 {
            continue; // skip malformed
        }

        // Skip multi-word tokens (ID contains dash)
        if fields[0].contains('-') {
            continue;
        }

        let id: u32 = fields[0]
            .parse()
            .map_err(|e| crate::error::UdpipelineError::ParseError(format!("bad ID '{}': {}", fields[0], e)))?;

        let feats = parse_feats(fields[5]);

        let head: Option<i32> = if fields.len() > 6 && !fields[6].is_empty() && fields[6] != "_" {
            fields[6].parse().ok()
        } else {
            None
        };

        let deprel: Option<String> = if fields.len() > 7 && !fields[7].is_empty() && fields[7] != "_" {
            Some(fields[7].to_string())
        } else {
            None
        };

        current_tokens.push(Token {
            id,
            word: fields[1].to_string(),
            lemma: fields[2].to_string(),
            upostag: fields[3].to_string(),
            xpostag: fields[4].to_string(),
            feats,
            head,
            deprel,
        });
    }

    // Don't forget last sentence without trailing newline
    if !current_tokens.is_empty() {
        sentences.push(Sentence {
            text: current_text,
            tokens: current_tokens,
        });
    }

    Ok(sentences)
}

/// Parse FEATS column: `Animacy=Inan|Case=Nom|Gender=Masc`
fn parse_feats(feats_str: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    if feats_str.is_empty() || feats_str == "_" {
        return map;
    }
    for pair in feats_str.split('|') {
        if let Some((key, value)) = pair.split_once('=') {
            map.insert(key.to_string(), value.to_string());
        }
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_conllu_single_sentence() {
        let input = "\
# text = Это пример.
1\tЭто\tэто\tDET\tDT\tAnimacy=Inan|Case=Nom\t_\t_\t_\t_
2\tпример\tпример\tNOUN\tNN\tCase=Nom|Gender=Masc|Number=Sing\t_\t_\t_\t_
";

        let sentences = parse_conllu(input).unwrap();
        assert_eq!(sentences.len(), 1);
        assert_eq!(sentences[0].text, "Это пример.");
        assert_eq!(sentences[0].tokens.len(), 2);

        let t1 = &sentences[0].tokens[0];
        assert_eq!(t1.word, "Это");
        assert_eq!(t1.lemma, "это");
        assert_eq!(t1.upostag, "DET");
        assert_eq!(t1.feats.get("Animacy").unwrap(), "Inan");

        let t2 = &sentences[0].tokens[1];
        assert_eq!(t2.word, "пример");
        assert_eq!(t2.upostag, "NOUN");
        assert_eq!(t2.feats.get("Gender").unwrap(), "Masc");
    }

    #[test]
    fn test_parse_conllu_multiple_sentences() {
        let input = "\
# text = Привет.
1\tПривет\tпривет\tINTJ\tUH\t_\t_\t_\t_\t_

# text = Как дела?
1\tКак\tкак\tADV\tRB\t_\t_\t_\t_\t_
2\tдела\tдело\tNOUN\tNN\tCase=Nom|Gender=Neut|Number=Plur\t_\t_\t_\t_
";

        let sentences = parse_conllu(input).unwrap();
        assert_eq!(sentences.len(), 2);
        assert_eq!(sentences[0].tokens.len(), 1);
        assert_eq!(sentences[1].tokens.len(), 2);
    }

    #[test]
    fn test_parse_empty() {
        let sentences = parse_conllu("").unwrap();
        assert!(sentences.is_empty());
    }

    #[test]
    fn test_parse_feats_empty() {
        let feats = parse_feats("_");
        assert!(feats.is_empty());
    }
}
