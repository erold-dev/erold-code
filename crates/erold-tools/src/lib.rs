//! Tool implementations for the Erold CLI
//!
//! Provides tools for file operations, shell commands, and search.
//! All tools integrate with the security gate for read-before-edit enforcement.

mod error;
mod registry;
mod tools;

pub use error::{ToolError, Result};
pub use registry::{ToolRegistry, ToolContext};
pub use tools::{
    Tool,
    read_file::ReadFileTool,
    write_file::WriteFileTool,
    edit_file::EditFileTool,
    search::SearchTool,
    bash::BashTool,
};

/// Tool definition for LLM
#[derive(Debug, Clone)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// Tool execution output
#[derive(Debug, Clone)]
pub enum ToolOutput {
    Text(String),
    Json(serde_json::Value),
    Error(String),
}

impl ToolOutput {
    #[must_use]
    pub fn text(s: impl Into<String>) -> Self {
        Self::Text(s.into())
    }

    #[must_use]
    pub fn json(v: serde_json::Value) -> Self {
        Self::Json(v)
    }

    #[must_use]
    pub fn error(s: impl Into<String>) -> Self {
        Self::Error(s.into())
    }

    /// Check if this is an error output
    #[must_use]
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error(_))
    }

    /// Get the text content if any
    #[must_use]
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text(s) => Some(s),
            _ => None,
        }
    }

    /// Get the error message if any
    #[must_use]
    pub fn as_error(&self) -> Option<&str> {
        match self {
            Self::Error(s) => Some(s),
            _ => None,
        }
    }
}

impl std::fmt::Display for ToolOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text(s) => write!(f, "{s}"),
            Self::Json(v) => write!(f, "{}", serde_json::to_string_pretty(v).unwrap_or_default()),
            Self::Error(e) => write!(f, "Error: {e}"),
        }
    }
}
