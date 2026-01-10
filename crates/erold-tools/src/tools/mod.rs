//! Tool implementations
//!
//! Each tool implements the `Tool` trait for consistent execution.

pub mod read_file;
pub mod write_file;
pub mod edit_file;
pub mod search;
pub mod bash;

use async_trait::async_trait;
use crate::{ToolDefinition, ToolOutput, Result};
use crate::registry::ToolContext;

/// Trait for implementing tools
#[async_trait]
pub trait Tool: Send + Sync {
    /// Get the tool name
    fn name(&self) -> &'static str;

    /// Get the tool description
    fn description(&self) -> &'static str;

    /// Get the tool definition for LLM
    fn definition(&self) -> ToolDefinition;

    /// Execute the tool with given parameters
    async fn execute(&self, params: serde_json::Value, context: &ToolContext) -> Result<ToolOutput>;
}
