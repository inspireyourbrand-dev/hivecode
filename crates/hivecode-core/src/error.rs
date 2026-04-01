//! Error types for HiveCode core operations

use thiserror::Error;

/// Result type for HiveCode core operations
pub type Result<T> = std::result::Result<T, HiveCodeError>;

/// Errors that can occur in HiveCode core operations
#[derive(Error, Debug)]
pub enum HiveCodeError {
    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Failed to load config file: {0}")]
    ConfigFileError(#[from] std::io::Error),

    #[error("Failed to parse config TOML: {0}")]
    TomlError(#[from] toml::de::Error),

    #[error("State lock poisoned: {0}")]
    StateLockError(String),

    #[error("Conversation error: {0}")]
    ConversationError(String),

    #[error("Invalid message format: {0}")]
    InvalidMessage(String),

    #[error("Token limit exceeded: {0}")]
    TokenLimitExceeded(String),

    #[error("Provider error: {0}")]
    ProviderError(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] rusqlite::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Environment variable error: {0}")]
    EnvError(String),

    #[error("IO error: {0}")]
    IOError(String),

    #[error("Authentication error: {0}")]
    AuthError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = HiveCodeError::ConfigError("test error".to_string());
        assert_eq!(err.to_string(), "Configuration error: test error");
    }

    #[test]
    fn test_result_type() {
        let result: Result<i32> = Err(HiveCodeError::Internal("test".to_string()));
        assert!(result.is_err());
    }
}
