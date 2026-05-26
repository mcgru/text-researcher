use std::io::Write;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

use log::{debug, info};

use crate::conllu::{self, Sentence};
use crate::error::UdpipelineError;

/// Manages a UDPipe 2 subprocess for morphological analysis.
pub struct Udpipeline {
    model_path: String,
    process: Option<Child>,
    timeout: Duration,
}

impl Udpipeline {
    /// Create a new pipeline for the given UDPipe model.
    ///
    /// The model file must exist at `model_path`.
    /// The subprocess is spawned lazily on first `tokenize()` call.
    pub fn new(model_path: impl Into<String>) -> Self {
        Udpipeline {
            model_path: model_path.into(),
            process: None,
            timeout: Duration::from_secs(60),
        }
    }

    /// Set the timeout for subprocess operations.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Ensure the subprocess is running.
    fn ensure_process(&mut self) -> Result<&mut Child, UdpipelineError> {
        if self.process.is_none() {
            info!("Starting UDPipe 2 subprocess for model: {}", self.model_path);

            let child = Command::new("python3")
                .args(["-m", "udpipe2", "--model", &self.model_path])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .map_err(|e| UdpipelineError::SubprocessFailed(format!(
                    "failed to start python3 -m udpipe2: {}. Is udpipe2 installed?",
                    e
                )))?;

            self.process = Some(child);
            debug!("UDPipe 2 subprocess started");
        }
        Ok(self.process.as_mut().unwrap())
    }

    /// Tokenize and analyze text.
    ///
    /// Returns sentences with full morphological analysis (lemma, POS, features).
    pub fn tokenize(&mut self, text: &str) -> Result<Vec<Sentence>, UdpipelineError> {
        let child = self.ensure_process()?;

        // Write text to subprocess stdin
        {
            let stdin = child
                .stdin
                .as_mut()
                .ok_or_else(|| UdpipelineError::SubprocessFailed("stdin not available".into()))?;

            // UDPipe 2 expects raw text, one sentence per line or paragraph
            writeln!(stdin, "{}", text)
                .map_err(|e| UdpipelineError::Io(e))?;
            stdin
                .flush()
                .map_err(|e| UdpipelineError::Io(e))?;
        }

        // Read CoNLL-U output from stdout
        let stdout = child
            .stdout
            .as_mut()
            .ok_or_else(|| UdpipelineError::SubprocessFailed("stdout not available".into()))?;

        let mut output = String::new();
        use std::io::Read;
        stdout
            .read_to_string(&mut output)
            .map_err(|e| UdpipelineError::Io(e))?;

        debug!("UDPipe output: {} bytes", output.len());

        // Parse CoNLL-U
        conllu::parse_conllu(&output)
    }
}

impl Drop for Udpipeline {
    fn drop(&mut self) {
        if let Some(mut child) = self.process.take() {
            // Close stdin to signal end of input
            drop(child.stdin.take());
            // Try to wait briefly, then kill
            let start = std::time::Instant::now();
            while start.elapsed() < Duration::from_secs(2) {
                if child.try_wait().ok().flatten().is_some() {
                    break;
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            let _ = child.kill();
            debug!("UDPipe 2 subprocess terminated");
        }
    }
}
