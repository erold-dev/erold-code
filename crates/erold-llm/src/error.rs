//! LLM error types

use thiserror::Error;

pub type Result<T> = std::result::Result<T, LlmError>;

#[derive(Error, Debug)]
pub enum LlmError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("API error ({status}): {message}")]
    Api { status: u16, message: String },

    #[error("Rate limited. Retry after {retry_after} seconds")]
    RateLimited { retry_after: u64 },

    #[error("Invalid response: {0}")]
    InvalidResponse(String),
}
