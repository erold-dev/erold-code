//! Write file tool
//!
//! Writes content to a file with security gate enforcement.

use async_trait::async_trait;
use serde::Deserialize;
use tokio::fs;

use crate::{ToolDefinition, ToolOutput, ToolError, Result};
use crate::registry::ToolContext;
use super::Tool;

/// Parameters for write_file
#[derive(Debug, Deserialize)]
struct WriteFileParams {
    /// Path to the file
    path: String,
    /// Content to write
    content: String,
}

/// Tool for writing file contents
pub struct WriteFileTool;

#[async_trait]
impl Tool for WriteFileTool {
    fn name(&self) -> &'static str {
        "write_file"
    }

    fn description(&self) -> &'static str {
        "Write content to a file. For existing files, use read_file first. Requires an approved plan."
    }

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.name().to_string(),
            description: self.description().to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "The path to the file to write"
                    },
                    "content": {
                        "type": "string",
                        "description": "The content to write to the file"
                    }
                },
                "required": ["path", "content"]
            }),
        }
    }

    async fn execute(&self, params: serde_json::Value, context: &ToolContext) -> Result<ToolOutput> {
        let params: WriteFileParams = serde_json::from_value(params)
            .map_err(|e| ToolError::InvalidParams(e.to_string()))?;

        // Resolve path relative to working directory
        let path = if std::path::Path::new(&params.path).is_absolute() {
            std::path::PathBuf::from(&params.path)
        } else {
            context.working_dir().join(&params.path)
        };

        // For existing files, check security gate
        if path.exists() {
            context.on_file_edit(&path.to_string_lossy()).await?;
        } else {
            // For new files, still need approved plan
            let guard = context.security().read().await;
            guard.check_can_modify()
                .map_err(|_| ToolError::NoPlanApproved)?;
        }

        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| ToolError::Io(e))?;
        }

        // Write the file
        fs::write(&path, &params.content)
            .await
            .map_err(|e| ToolError::Io(e))?;

        Ok(ToolOutput::text(format!("Wrote {} bytes to {}", params.content.len(), path.display())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use erold_workflow::SecurityGate;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_write_new_file_requires_plan() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("new.txt");

        let security = Arc::new(RwLock::new(SecurityGate::new()));
        let context = ToolContext::new(security.clone(), temp_dir.path().to_path_buf());

        let tool = WriteFileTool;
        let params = serde_json::json!({
            "path": file_path.to_string_lossy(),
            "content": "new content"
        });

        // Should fail - no approved plan
        let result = tool.execute(params, &context).await;
        assert!(matches!(result, Err(ToolError::NoPlanApproved)));
    }

    #[tokio::test]
    async fn test_write_new_file_with_approved_plan() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("new.txt");

        let security = Arc::new(RwLock::new(SecurityGate::new()));
        security.write().await.approve_plan();
        let context = ToolContext::new(security.clone(), temp_dir.path().to_path_buf());

        let tool = WriteFileTool;
        let params = serde_json::json!({
            "path": file_path.to_string_lossy(),
            "content": "new content"
        });

        let result = tool.execute(params, &context).await;
        assert!(result.is_ok());
        assert!(file_path.exists());
    }

    #[tokio::test]
    async fn test_write_existing_file_requires_read_first() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("existing.txt");
        tokio::fs::write(&file_path, "original").await.unwrap();

        let security = Arc::new(RwLock::new(SecurityGate::new()));
        security.write().await.approve_plan();
        let context = ToolContext::new(security.clone(), temp_dir.path().to_path_buf());

        let tool = WriteFileTool;
        let params = serde_json::json!({
            "path": file_path.to_string_lossy(),
            "content": "updated"
        });

        // Should fail - didn't read first
        let result = tool.execute(params, &context).await;
        assert!(matches!(result, Err(ToolError::MustReadFirst(_))));
    }
}
