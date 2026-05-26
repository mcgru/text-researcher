use std::path::PathBuf;

#[derive(clap::Parser, Debug)]
#[command(name = "text-researcher", version, about = "Linguistic text analysis tool")]
pub struct Cli {
    /// Input text file (batch mode, default)
    #[arg(required_unless_present = "interactive")]
    pub input: Option<PathBuf>,

    /// Language code (default: from config, or "ru")
    #[arg(short = 'l', long = "lang")]
    pub language: Option<String>,

    /// Output file for JSON results (default: stdout)
    #[arg(short = 'o', long = "output")]
    pub output: Option<PathBuf>,

    /// Launch interactive TUI mode
    #[arg(short = 'i', long = "interactive")]
    pub interactive: bool,

    /// Download a language model
    #[arg(long = "download-model")]
    pub download_model: Option<String>,

    /// Verbose logging
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,

    /// Number of parallel threads (default: all logical CPUs)
    #[arg(short = 'j', long = "jobs")]
    pub jobs: Option<usize>,

    /// Number of words to prefetch ahead/behind cursor (-1 = entire text, default)
    #[arg(long = "prefetch", default_value = "-1")]
    pub prefetch: isize,

    /// Output format: txt (word: props) or json (default: txt)
    #[arg(short = 'f', long = "format", default_value = "txt")]
    pub format: String,
}
