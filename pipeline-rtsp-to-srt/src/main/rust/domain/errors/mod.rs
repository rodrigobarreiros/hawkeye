use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Invalid RTSP URL: {0}")]
    InvalidRtspUrl(String),

    #[error("Invalid SRT URL: {0}")]
    InvalidSrtUrl(String),

    #[error("Invalid port: port cannot be zero")]
    InvalidPort,

    #[error("Invalid backoff multiplier: must be > 1.0")]
    InvalidBackoffMultiplier,

    #[error("Pipeline creation failed: {0}")]
    PipelineCreationFailed(String),

    #[error("Pipeline execution failed: {0}")]
    PipelineExecutionFailed(String),

    #[error("Bridge not running")]
    BridgeNotRunning,
}

pub type Result<T> = std::result::Result<T, DomainError>;
