use thiserror::Error;

#[derive(Error, Debug)]
pub enum SecurityError {
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Dangerous command detected: {0}")]
    DangerousCommand(String),

    #[error("Sensitive file access denied: {0}")]
    SensitiveFileAccess(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}
