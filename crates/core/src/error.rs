use thiserror::Error;
use udpipe_client::UdpipelineError;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("model not found for language: {0}")]
    ModelNotFound(String),

    #[error("analysis failed: {0}")]
    AnalysisFailed(String),

    #[error("config error: {0}")]
    ConfigError(String),

    #[error("UDPipe error: {0}")]
    Udpipeline(#[from] UdpipelineError),
}
