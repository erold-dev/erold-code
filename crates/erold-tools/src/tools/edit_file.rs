//! Edit file tool
//!
//! Performs exact string replacements in files with security gate enforcement.

use async_trait::async_trait;
use serde::Deserialize;
use tokio::fs;

use crate::{ToolDefinition, ToolOutput, ToolError, Result};
use crate::registry::ToolContext;
use super::Tool;

/// Parameters for edit_file
#[derive(Debug, Deserialize)]
struct EditFileParams {
    /// Path to the file
    path: String,
    /// String to find
    old_string: String,
    /// String to replace with
    new_string: String,
    /// Whether to replace all occurrences
    #[serde(default)]
    replace_all: bool,
}

/// Tool for editing file contents
pub struct EditFileTool;

#[async_trait]
impl Tool for EditFileTool {
    fn name(&self) -> &'static str {
        "edit_file"
    }

    fn description(&self) -> &'static str {
        "Edit a file by replacing exact string matches. Must read the file first."
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
                        "description": "The path to the file to edit"
                    },
                    "old_string": {
                        "type": "string",
                        "description": "The exact string to find and replace"
                    },
                    "new_string": {
                        "type": "string",
                        "description": "The string to replace with"
                    },
                    "replace_all": {
                        "type": "boolean",
                        "description": "If true, replace all occurrences. Default is false (replace first only)."
                    }
                },
                "required": ["path", "old_string", "new_string"]
            }),
        }
    }

    async fn execute(&self, params: serde_json::Value, context: &ToolContext) -> Result<ToolOutput> {
        let params: EditFileParams = serde_json::from_value(params)
            .map_err(|e| ToolError::InvalidParams(e.to_string()))?;

        // Validate inputs
        if params.old_string == params.new_string {
            return Ok(ToolOutput::error("old_string and new_string are identical"));
        }

        // Resolve path
        let path = if std::path::Path::new(&params.path).is_absolute() {
            std::path::PathBuf::from(&params.path)
        } else {
            context.working_dir().join(&params.path)
        };

        // Check file exists
        if !path.exists() {
            return Ok(ToolOutput::error(format!("File not found: {}", path.display())));
        }

        // Check security gate (must have read first and have approved plan)
        context.on_file_edit(&path.to_string_lossy()).await?;

        // Read current content
        let content = fs::read_to_string(&path)
            .await
            .map_err(|e| ToolError::Io(e))?;

        // Check old_string exists
        if !content.contains(&params.old_string) {
            return Ok(ToolOutput::error(format!(
                "String not found in file. Cannot find:\n{}\n\nMake sure the string matches exactly including whitespace.",
                truncate(&params.old_string, 200)
            )));
        }

        // Check uniqueness if not replace_all
        if !params.replace_all {
            let count = content.matches(&params.old_string).count();
            if count > 1 {
                return Ok(ToolOutput::error(format!(
                    "String appears {} times in file. Use replace_all=true or provide more context to make it unique.",
                    count
                )));
            }
        }

        // Perform replacement
        let new_content = if params.replace_all {
            content.replace(&params.old_string, &params.new_string)
        } else {
            content.replacen(&params.old_string, &params.new_string, 1)
        };

        let replacements = if params.replace_all {
            content.matches(&params.old_string).count()
        } else {
            1
        };

        // Write back
        fs::write(&path, &new_content)
            .await
            .map_err(|e| ToolError::Io(e))?;

        Ok(ToolOutput::text(format!(
            "Made {} replacement(s) in {}",
            replacements,
            path.display()
        )))
    }
}

/// Truncate a string for display
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use erold_workflow::SecurityGate;
    use tempfile::TempDir;
    use crate::tools::read_file::ReadFileTool;

    #[tokio::test]
    async fn test_edit_requires_read_first() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        tokio::fs::write(&file_path, "hello world").await.unwrap();

        let security = Arc::new(RwLock::new(SecurityGate::new()));
        security.write().await.approve_plan();
        let context = ToolContext::new(security.clone(), temp_dir.path().to_path_buf());

        let tool = EditFileTool;
        let params = serde_json::json!({
            "path": file_path.to_string_lossy(),
            "old_string": "hello",
            "new_string": "goodbye"
        });

        // Should fail - didn't read first
        let result = tool.execute(params, &context).await;
        assert!(matches!(result, Err(ToolError::MustReadFirst(_))));
    }

    #[tokio::test]
    async fn test_edit_after_read() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        tokio::fs::write(&file_path, "hello world").await.unwrap();

        let security = Arc::new(RwLock::new(SecurityGate::new()));
        security.write().await.approve_plan();
        let context = ToolContext::new(security.clone(), temp_dir.path().to_path_buf());

        // Read first
        let read_tool = ReadFileTool;
        read_tool.execute(serde_json::json!({
            "path": file_path.to_string_lossy()
        }), &context).await.unwrap();

        // Now edit should work
        let edit_tool = EditFileTool;
        let result = edit_tool.execute(serde_json::json!({
            "path": file_path.to_string_lossy(),
            "old_string": "hello",
            "new_string": "goodbye"
        }), &context).await;

        assert!(result.is_ok());

        let content = tokio::fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(content, "goodbye world");
    }

    #[tokio::test]
    async fn test_edit_non_unique_fails() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        tokio::fs::write(&file_path, "hello hello hello").await.unwrap();

        let security = Arc::new(RwLock::new(SecurityGate::new()));
        security.write().await.approve_plan();
        let context = ToolContext::new(security.clone(), temp_dir.path().to_path_buf());

        // Read first
        let read_tool = ReadFileTool;
        read_tool.execute(serde_json::json!({
            "path": file_path.to_string_lossy()
        }), &context).await.unwrap();

        // Edit should fail - not unique
        let edit_tool = EditFileTool;
        let result = edit_tool.execute(serde_json::json!({
            "path": file_path.to_string_lossy(),
            "old_string": "hello",
            "new_string": "goodbye"
        }), &context).await.unwrap();

        assert!(result.is_error());
    }

    #[tokio::test]
    async fn test_edit_replace_all() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        tokio::fs::write(&file_path, "hello hello hello").await.unwrap();

        let security = Arc::new(RwLock::new(SecurityGate::new()));
        security.write().await.approve_plan();
        let context = ToolContext::new(security.clone(), temp_dir.path().to_path_buf());

        // Read first
        let read_tool = ReadFileTool;
        read_tool.execute(serde_json::json!({
            "path": file_path.to_string_lossy()
        }), &context).await.unwrap();

        // Edit with replace_all
        let edit_tool = EditFileTool;
        let result = edit_tool.execute(serde_json::json!({
            "path": file_path.to_string_lossy(),
            "old_string": "hello",
            "new_string": "goodbye",
            "replace_all": true
        }), &context).await;

        assert!(result.is_ok());

        let content = tokio::fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(content, "goodbye goodbye goodbye");
    }
}
