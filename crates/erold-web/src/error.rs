//! Web fetching error types

use thiserror::Error;

pub type Result<T> = std::result::Result<T, WebError>;

#[derive(Error, Debug)]
pub enum WebError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Rate limited")]
    RateLimited,
}
