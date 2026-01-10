//! Tool registry and execution context
//!
//! Manages tool registration and provides execution context
//! with security gate integration.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{Tool, ToolDefinition, ToolOutput, ToolError, Result};
use erold_workflow::SecurityGate;

/// Execution context for tools
///
/// Provides access to the security gate and other shared state.
pub struct ToolContext {
    /// Security gate for file operation enforcement
    security: Arc<RwLock<SecurityGate>>,
    /// Current working directory
    working_dir: std::path::PathBuf,
}

impl ToolContext {
    /// Create a new tool context
    pub fn new(security: Arc<RwLock<SecurityGate>>, working_dir: std::path::PathBuf) -> Self {
        Self { security, working_dir }
    }

    /// Get the security gate
    #[must_use]
    pub fn security(&self) -> &Arc<RwLock<SecurityGate>> {
        &self.security
    }

    /// Get the working directory
    #[must_use]
    pub fn working_dir(&self) -> &std::path::Path {
        &self.working_dir
    }

    /// Record a file read
    pub async fn on_file_read(&self, path: &str) -> Result<()> {
        self.security.write().await
            .on_file_read(path)
            .map_err(|e| ToolError::ExecutionFailed(e.to_string()))
    }

    /// Check and record a file edit
    pub async fn on_file_edit(&self, path: &str) -> Result<()> {
        self.security.write().await
            .on_file_edit(path)
            .map_err(|e| match e {
                erold_workflow::WorkflowError::NoPlanApproved => ToolError::NoPlanApproved,
                erold_workflow::WorkflowError::MustReadBeforeEdit { path } => {
                    ToolError::MustReadFirst(path.display().to_string())
                }
                other => ToolError::ExecutionFailed(other.to_string()),
            })
    }

    /// Check if a file can be edited
    pub async fn can_edit(&self, path: &str) -> bool {
        let guard = self.security.read().await;
        guard.check_can_modify().is_ok() && guard.file_tracker().check_edit(path).is_ok()
    }
}

/// Registry for managing available tools
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    /// Create a new empty registry
    #[must_use]
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Create a registry with default tools
    #[must_use]
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();

        // Register file tools
        registry.register(Arc::new(crate::tools::read_file::ReadFileTool));
        registry.register(Arc::new(crate::tools::write_file::WriteFileTool));
        registry.register(Arc::new(crate::tools::edit_file::EditFileTool));

        // Register search tools
        registry.register(Arc::new(crate::tools::search::SearchTool));

        // Register shell tools
        registry.register(Arc::new(crate::tools::bash::BashTool));

        registry
    }

    /// Register a tool
    pub fn register(&mut self, tool: Arc<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    /// Get a tool by name
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Arc<dyn Tool>> {
        self.tools.get(name)
    }

    /// Get all tool definitions
    #[must_use]
    pub fn definitions(&self) -> Vec<ToolDefinition> {
        self.tools.values().map(|t| t.definition()).collect()
    }

    /// Execute a tool by name
    pub async fn execute(
        &self,
        name: &str,
        params: serde_json::Value,
        context: &ToolContext,
    ) -> Result<ToolOutput> {
        let tool = self.tools.get(name)
            .ok_or_else(|| ToolError::NotFound(name.to_string()))?;

        tool.execute(params, context).await
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = ToolRegistry::with_defaults();
        assert!(registry.get("read_file").is_some());
        assert!(registry.get("write_file").is_some());
        assert!(registry.get("edit_file").is_some());
        assert!(registry.get("search").is_some());
        assert!(registry.get("bash").is_some());
    }

    #[test]
    fn test_definitions() {
        let registry = ToolRegistry::with_defaults();
        let defs = registry.definitions();
        assert!(!defs.is_empty());
    }
}
