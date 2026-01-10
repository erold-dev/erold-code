//! Bash command execution tool
//!
//! Executes shell commands with timeout and output limits.

use async_trait::async_trait;
use serde::Deserialize;
use std::process::Stdio;
use tokio::process::Command;
use tokio::time::{timeout, Duration};

use crate::{ToolDefinition, ToolOutput, ToolError, Result};
use crate::registry::ToolContext;
use super::Tool;

/// Default timeout for commands (2 minutes)
const DEFAULT_TIMEOUT_MS: u64 = 120_000;

/// Maximum output size (30KB)
const MAX_OUTPUT_SIZE: usize = 30_000;

/// Parameters for bash
#[derive(Debug, Deserialize)]
struct BashParams {
    /// Command to execute
    command: String,
    /// Working directory (optional)
    #[serde(default)]
    cwd: Option<String>,
    /// Timeout in milliseconds
    #[serde(default)]
    timeout_ms: Option<u64>,
}

/// Tool for executing bash commands
pub struct BashTool;

#[async_trait]
impl Tool for BashTool {
    fn name(&self) -> &'static str {
        "bash"
    }

    fn description(&self) -> &'static str {
        "Execute a bash command. Use for git, npm, docker, and other system operations."
    }

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.name().to_string(),
            description: self.description().to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "The bash command to execute"
                    },
                    "cwd": {
                        "type": "string",
                        "description": "Working directory for the command"
                    },
                    "timeout_ms": {
                        "type": "integer",
                        "description": "Timeout in milliseconds (default 120000)"
                    }
                },
                "required": ["command"]
            }),
        }
    }

    async fn execute(&self, params: serde_json::Value, context: &ToolContext) -> Result<ToolOutput> {
        let params: BashParams = serde_json::from_value(params)
            .map_err(|e| ToolError::InvalidParams(e.to_string()))?;

        // Validate command (basic security checks)
        if params.command.is_empty() {
            return Ok(ToolOutput::error("Command cannot be empty"));
        }

        // Resolve working directory
        let cwd = params.cwd
            .map(|p| {
                if std::path::Path::new(&p).is_absolute() {
                    std::path::PathBuf::from(p)
                } else {
                    context.working_dir().join(p)
                }
            })
            .unwrap_or_else(|| context.working_dir().to_path_buf());

        let timeout_duration = Duration::from_millis(
            params.timeout_ms.unwrap_or(DEFAULT_TIMEOUT_MS).min(600_000)
        );

        // Execute command
        let mut cmd = Command::new("sh");
        cmd.arg("-c")
            .arg(&params.command)
            .current_dir(&cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let child = cmd.spawn().map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

        // Wait with timeout
        let result = timeout(timeout_duration, child.wait_with_output()).await;

        match result {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                let mut result_text = String::new();

                // Add stdout
                if !stdout.is_empty() {
                    let truncated = truncate_output(&stdout, MAX_OUTPUT_SIZE);
                    result_text.push_str(&truncated);
                }

                // Add stderr
                if !stderr.is_empty() {
                    if !result_text.is_empty() {
                        result_text.push_str("\n\n--- stderr ---\n");
                    }
                    let truncated = truncate_output(&stderr, MAX_OUTPUT_SIZE / 2);
                    result_text.push_str(&truncated);
                }

                // Add exit code if non-zero
                if !output.status.success() {
                    let code = output.status.code().unwrap_or(-1);
                    result_text.push_str(&format!("\n\nExit code: {}", code));
                }

                if result_text.is_empty() {
                    result_text = "(no output)".to_string();
                }

                Ok(ToolOutput::text(result_text))
            }
            Ok(Err(e)) => {
                Ok(ToolOutput::error(format!("Command failed: {}", e)))
            }
            Err(_) => {
                Ok(ToolOutput::error(format!(
                    "Command timed out after {} seconds",
                    timeout_duration.as_secs()
                )))
            }
        }
    }
}

/// Truncate output to max size
fn truncate_output(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!(
            "{}...\n\n[truncated {} characters]",
            &s[..max_len.saturating_sub(50)],
            s.len() - max_len + 50
        )
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
    async fn test_simple_command() {
        let temp_dir = TempDir::new().unwrap();
        let security = Arc::new(RwLock::new(SecurityGate::new()));
        let context = ToolContext::new(security.clone(), temp_dir.path().to_path_buf());

        let tool = BashTool;
        let output = tool.execute(serde_json::json!({
            "command": "echo 'hello world'"
        }), &context).await.unwrap();

        let text = output.as_text().unwrap();
        assert!(text.contains("hello world"));
    }

    #[tokio::test]
    async fn test_command_with_cwd() {
        let temp_dir = TempDir::new().unwrap();
        tokio::fs::create_dir_all(temp_dir.path().join("subdir")).await.expect("create subdir");
        tokio::fs::write(temp_dir.path().join("subdir/test.txt"), "content").await.expect("write test file");

        let security = Arc::new(RwLock::new(SecurityGate::new()));
        let context = ToolContext::new(security.clone(), temp_dir.path().to_path_buf());

        let tool = BashTool;
        let output = tool.execute(serde_json::json!({
            "command": "ls",
            "cwd": "subdir"
        }), &context).await.unwrap();

        let text = output.as_text().unwrap();
        assert!(text.contains("test.txt"));
    }

    #[tokio::test]
    async fn test_command_failure() {
        let temp_dir = TempDir::new().unwrap();
        let security = Arc::new(RwLock::new(SecurityGate::new()));
        let context = ToolContext::new(security.clone(), temp_dir.path().to_path_buf());

        let tool = BashTool;
        let output = tool.execute(serde_json::json!({
            "command": "exit 1"
        }), &context).await.unwrap();

        let text = output.as_text().unwrap();
        assert!(text.contains("Exit code: 1"));
    }

    #[tokio::test]
    async fn test_timeout() {
        let temp_dir = TempDir::new().unwrap();
        let security = Arc::new(RwLock::new(SecurityGate::new()));
        let context = ToolContext::new(security.clone(), temp_dir.path().to_path_buf());

        let tool = BashTool;
        let output = tool.execute(serde_json::json!({
            "command": "sleep 10",
            "timeout_ms": 100
        }), &context).await.unwrap();

        let error = output.as_error().unwrap();
        assert!(error.contains("timed out"));
    }
}
