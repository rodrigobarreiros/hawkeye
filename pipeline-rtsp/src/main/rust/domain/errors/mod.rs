use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Invalid path: {0}")]
    InvalidPath(PathBuf),

    #[error("Path is not a file: {0}")]
    PathNotFile(PathBuf),

    #[error("Invalid port: port cannot be zero")]
    InvalidPort,

    #[error("Port {0} requires root privileges")]
    PortRequiresRoot(u16),

    #[error("Invalid mount point: {0}")]
    InvalidMountPoint(String),

    #[error("Mount point too long (max 100 characters)")]
    MountPointTooLong,

    #[error("Server initialization failed")]
    ServerInitFailed,

    #[error("Unsupported codec: {0}")]
    UnsupportedCodec(String),

    #[error("Unsupported container format: {0}")]
    UnsupportedContainer(String),
}

pub type Result<T> = std::result::Result<T, DomainError>;
