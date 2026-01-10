//! API error types

use thiserror::Error;

/// Result type for API operations
pub type Result<T> = std::result::Result<T, ApiError>;

/// API error types
#[derive(Error, Debug)]
pub enum ApiError {
    /// HTTP request failed
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON serialization/deserialization failed
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// API returned an error response
    #[error("API error ({status}): {message}")]
    Api {
        status: u16,
        code: String,
        message: String,
    },

    /// Resource not found
    #[error("Not found: {0}")]
    NotFound(String),

    /// Authentication failed
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    /// Rate limited
    #[error("Rate limited. Retry after {retry_after} seconds")]
    RateLimited { retry_after: u64 },

    /// Invalid configuration
    #[error("Configuration error: {0}")]
    Config(String),
}

impl ApiError {
    /// Check if this error is retryable
    #[must_use]
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Http(_) | Self::RateLimited { .. }
        )
    }
}
