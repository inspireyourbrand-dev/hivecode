//! Error types for the providers crate.

use thiserror::Error;

/// Errors that can occur when interacting with LLM providers.
#[derive(Error, Debug)]
pub enum ProviderError {
    /// HTTP request failed
    #[error("HTTP error: {0}")]
    HttpError(String),

    /// JSON parsing failed
    #[error("JSON parse error: {0}")]
    ParseError(#[from] serde_json::Error),

    /// Rate limit exceeded
    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    /// Authentication failed
    #[error("Authentication failed: {0}")]
    AuthError(String),

    /// Model not found
    #[error("Model not found: {0}")]
    ModelNotFound(String),

    /// Request timeout
    #[error("Request timeout after {0}s")]
    Timeout(u64),

    /// Stream error
    #[error("Stream error: {0}")]
    StreamError(String),

    /// Connection refused
    #[error("Connection refused: {0}")]
    ConnectionRefused(String),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// Generic error
    #[error("{0}")]
    Other(String),
}

impl From<reqwest::Error> for ProviderError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            ProviderError::Timeout(30)
        } else if err.is_connect() {
            ProviderError::ConnectionRefused(err.to_string())
        } else if err.status() == Some(reqwest::StatusCode::UNAUTHORIZED) {
            ProviderError::AuthError("Unauthorized - check API key".to_string())
        } else if err.status() == Some(reqwest::StatusCode::TOO_MANY_REQUESTS) {
            ProviderError::RateLimit("Too many requests".to_string())
        } else {
            ProviderError::HttpError(err.to_string())
        }
    }
}
