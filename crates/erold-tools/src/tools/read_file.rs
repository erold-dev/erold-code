//! Read file tool
//!
//! Reads file contents and records the read in the security gate.

use async_trait::async_trait;
use serde::Deserialize;
use tokio::fs;

use crate::{ToolDefinition, ToolOutput, ToolError, Result};
use crate::registry::ToolContext;
use super::Tool;

/// Parameters for read_file
#[derive(Debug, Deserialize)]
struct ReadFileParams {
    /// Path to the file
    path: String,
    /// Optional line offset (1-based)
    #[serde(default)]
    offset: Option<usize>,
    /// Optional line limit
    #[serde(default)]
    limit: Option<usize>,
}

/// Tool for reading file contents
pub struct ReadFileTool;

#[async_trait]
impl Tool for ReadFileTool {
    fn name(&self) -> &'static str {
        "read_file"
    }

    fn description(&self) -> &'static str {
        "Read the contents of a file. Must be called before editing a file."
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
                        "description": "The path to the file to read"
                    },
                    "offset": {
                        "type": "integer",
                        "description": "Line number to start reading from (1-based)"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of lines to read"
                    }
                },
                "required": ["path"]
            }),
        }
    }

    async fn execute(&self, params: serde_json::Value, context: &ToolContext) -> Result<ToolOutput> {
        let params: ReadFileParams = serde_json::from_value(params)
            .map_err(|e| ToolError::InvalidParams(e.to_string()))?;

        // Resolve path relative to working directory
        let path = if std::path::Path::new(&params.path).is_absolute() {
            std::path::PathBuf::from(&params.path)
        } else {
            context.working_dir().join(&params.path)
        };

        // Check file exists
        if !path.exists() {
            return Ok(ToolOutput::error(format!("File not found: {}", path.display())));
        }

        // Read file content
        let content = fs::read_to_string(&path)
            .await
            .map_err(|e| ToolError::Io(e))?;

        // Record the read in security gate
        context.on_file_read(&path.to_string_lossy()).await?;

        // Apply offset and limit
        let lines: Vec<&str> = content.lines().collect();
        let offset = params.offset.unwrap_or(1).saturating_sub(1);
        let limit = params.limit.unwrap_or(lines.len());

        let selected: Vec<String> = lines
            .iter()
            .skip(offset)
            .take(limit)
            .enumerate()
            .map(|(i, line)| format!("{:>6}\t{}", offset + i + 1, line))
            .collect();

        Ok(ToolOutput::text(selected.join("\n")))
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
    async fn test_read_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        tokio::fs::write(&file_path, "line 1\nline 2\nline 3").await.unwrap();

        let security = Arc::new(RwLock::new(SecurityGate::new()));
        let context = ToolContext::new(security.clone(), temp_dir.path().to_path_buf());

        let tool = ReadFileTool;
        let params = serde_json::json!({
            "path": file_path.to_string_lossy()
        });

        let output = tool.execute(params, &context).await.unwrap();
        let text = output.as_text().unwrap();

        assert!(text.contains("line 1"));
        assert!(text.contains("line 2"));
        assert!(text.contains("line 3"));
    }

    #[tokio::test]
    async fn test_read_with_offset_and_limit() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        tokio::fs::write(&file_path, "line 1\nline 2\nline 3\nline 4\nline 5").await.unwrap();

        let security = Arc::new(RwLock::new(SecurityGate::new()));
        let context = ToolContext::new(security.clone(), temp_dir.path().to_path_buf());

        let tool = ReadFileTool;
        let params = serde_json::json!({
            "path": file_path.to_string_lossy(),
            "offset": 2,
            "limit": 2
        });

        let output = tool.execute(params, &context).await.unwrap();
        let text = output.as_text().unwrap();

        assert!(!text.contains("line 1"));
        assert!(text.contains("line 2"));
        assert!(text.contains("line 3"));
        assert!(!text.contains("line 4"));
    }
}
