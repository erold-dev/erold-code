//! Tool error types

use thiserror::Error;

pub type Result<T> = std::result::Result<T, ToolError>;

#[derive(Error, Debug)]
pub enum ToolError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Tool not found: {0}")]
    NotFound(String),

    #[error("Invalid parameters: {0}")]
    InvalidParams(String),

    #[error("Must read file before editing: {0}")]
    MustReadFirst(String),

    #[error("No approved plan. Cannot modify files.")]
    NoPlanApproved,

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
}
