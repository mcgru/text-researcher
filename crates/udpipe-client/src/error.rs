use std::{io, process, time::Duration};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UdpipelineError {
    #[error("model not found: {0}")]
    ModelNotFound(String),

    #[error("subprocess failed: {0}")]
    SubprocessFailed(String),

    #[error("failed to parse CoNLL-U output: {0}")]
    ParseError(String),

    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("timeout after {0:?}")]
    Timeout(Duration),
}

impl From<process::ExitStatus> for UdpipelineError {
    fn from(status: process::ExitStatus) -> Self {
        UdpipelineError::SubprocessFailed(format!("exit code: {}", status))
    }
}
