//! Search tool
//!
//! Searches for files and content using glob patterns and regex.

use async_trait::async_trait;
use serde::Deserialize;
use std::path::PathBuf;
use tokio::fs;

use crate::{ToolDefinition, ToolOutput, ToolError, Result};
use crate::registry::ToolContext;
use super::Tool;

/// Parameters for search
#[derive(Debug, Deserialize)]
struct SearchParams {
    /// Glob pattern for finding files
    #[serde(default)]
    pattern: Option<String>,
    /// Regex pattern to search for in file contents
    #[serde(default)]
    query: Option<String>,
    /// Directory to search in
    #[serde(default)]
    path: Option<String>,
    /// Maximum results to return
    #[serde(default)]
    limit: Option<usize>,
}

/// Tool for searching files and content
pub struct SearchTool;

#[async_trait]
impl Tool for SearchTool {
    fn name(&self) -> &'static str {
        "search"
    }

    fn description(&self) -> &'static str {
        "Search for files by pattern or search within file contents"
    }

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.name().to_string(),
            description: self.description().to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "pattern": {
                        "type": "string",
                        "description": "Glob pattern for finding files (e.g., **/*.rs)"
                    },
                    "query": {
                        "type": "string",
                        "description": "Text to search for within files"
                    },
                    "path": {
                        "type": "string",
                        "description": "Directory to search in (defaults to current directory)"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of results (default 50)"
                    }
                }
            }),
        }
    }

    async fn execute(&self, params: serde_json::Value, context: &ToolContext) -> Result<ToolOutput> {
        let params: SearchParams = serde_json::from_value(params)
            .map_err(|e| ToolError::InvalidParams(e.to_string()))?;

        let base_path = params.path
            .map(|p| {
                if std::path::Path::new(&p).is_absolute() {
                    PathBuf::from(p)
                } else {
                    context.working_dir().join(p)
                }
            })
            .unwrap_or_else(|| context.working_dir().to_path_buf());

        let limit = params.limit.unwrap_or(50);

        // If we have a glob pattern, find matching files
        if let Some(pattern) = &params.pattern {
            let files = find_files(&base_path, pattern, limit).await?;

            if files.is_empty() {
                return Ok(ToolOutput::text("No files found matching pattern"));
            }

            // If we also have a query, search within those files
            if let Some(query) = &params.query {
                let results = search_in_files(&files, query, limit).await?;
                if results.is_empty() {
                    return Ok(ToolOutput::text("No matches found"));
                }
                return Ok(ToolOutput::text(results.join("\n\n")));
            }

            // Just return file list
            let output: Vec<String> = files
                .iter()
                .map(|f| f.display().to_string())
                .collect();
            return Ok(ToolOutput::text(output.join("\n")));
        }

        // If only query, search in all files
        if let Some(query) = &params.query {
            let files = find_files(&base_path, "**/*", 1000).await?;
            let results = search_in_files(&files, query, limit).await?;
            if results.is_empty() {
                return Ok(ToolOutput::text("No matches found"));
            }
            return Ok(ToolOutput::text(results.join("\n\n")));
        }

        Ok(ToolOutput::error("Provide either 'pattern' or 'query' parameter"))
    }
}

/// Find files matching a glob pattern
async fn find_files(base: &std::path::Path, pattern: &str, limit: usize) -> Result<Vec<PathBuf>> {
    use std::collections::VecDeque;

    let mut results = Vec::new();
    let mut queue = VecDeque::new();
    queue.push_back(base.to_path_buf());

    // Simple pattern matching (basic glob support)
    let is_match = |path: &std::path::Path, pattern: &str| -> bool {
        let path_str = path.to_string_lossy();

        // Handle ** (recursive)
        if pattern.contains("**") {
            let parts: Vec<&str> = pattern.split("**").collect();
            if parts.len() == 2 {
                let prefix = parts[0].trim_end_matches('/');
                let suffix = parts[1].trim_start_matches('/');

                if !prefix.is_empty() && !path_str.contains(prefix) {
                    return false;
                }
                if !suffix.is_empty() {
                    // Check file extension or pattern
                    return path_str.ends_with(suffix.trim_start_matches('*'));
                }
                return true;
            }
        }

        // Simple wildcard matching
        if pattern.contains('*') {
            let parts: Vec<&str> = pattern.split('*').collect();
            let mut pos = 0;
            for part in parts {
                if part.is_empty() {
                    continue;
                }
                if let Some(found) = path_str[pos..].find(part) {
                    pos += found + part.len();
                } else {
                    return false;
                }
            }
            return true;
        }

        path_str.contains(pattern)
    };

    while let Some(dir) = queue.pop_front() {
        if results.len() >= limit {
            break;
        }

        let mut entries = match fs::read_dir(&dir).await {
            Ok(e) => e,
            Err(_) => continue,
        };

        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();

            // Skip hidden files and common ignored directories
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name.starts_with('.') || name == "node_modules" || name == "target" {
                continue;
            }

            if path.is_dir() {
                queue.push_back(path);
            } else if path.is_file() && is_match(&path, pattern) {
                results.push(path);
                if results.len() >= limit {
                    break;
                }
            }
        }
    }

    Ok(results)
}

/// Search for content within files
async fn search_in_files(files: &[PathBuf], query: &str, limit: usize) -> Result<Vec<String>> {
    let mut results = Vec::new();
    let query_lower = query.to_lowercase();

    for file in files {
        if results.len() >= limit {
            break;
        }

        // Skip binary files (simple check)
        let ext = file.extension().and_then(|e| e.to_str()).unwrap_or("");
        if is_binary_extension(ext) {
            continue;
        }

        let content = match fs::read_to_string(file).await {
            Ok(c) => c,
            Err(_) => continue,
        };

        for (line_num, line) in content.lines().enumerate() {
            if results.len() >= limit {
                break;
            }

            if line.to_lowercase().contains(&query_lower) {
                results.push(format!(
                    "{}:{}: {}",
                    file.display(),
                    line_num + 1,
                    line.trim()
                ));
            }
        }
    }

    Ok(results)
}

fn is_binary_extension(ext: &str) -> bool {
    matches!(
        ext.to_lowercase().as_str(),
        "png" | "jpg" | "jpeg" | "gif" | "ico" | "pdf"
            | "exe" | "dll" | "so" | "dylib"
            | "zip" | "tar" | "gz" | "rar"
            | "woff" | "woff2" | "ttf" | "eot"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use erold_workflow::SecurityGate;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_search_by_pattern() {
        let temp_dir = TempDir::new().unwrap();
        tokio::fs::write(temp_dir.path().join("test.rs"), "fn main() {}").await.unwrap();
        tokio::fs::write(temp_dir.path().join("test.txt"), "hello").await.unwrap();

        let security = Arc::new(RwLock::new(SecurityGate::new()));
        let context = ToolContext::new(security.clone(), temp_dir.path().to_path_buf());

        let tool = SearchTool;
        let output = tool.execute(serde_json::json!({
            "pattern": "*.rs"
        }), &context).await.unwrap();

        let text = output.as_text().unwrap();
        assert!(text.contains("test.rs"));
        assert!(!text.contains("test.txt"));
    }

    #[tokio::test]
    async fn test_search_by_query() {
        let temp_dir = TempDir::new().unwrap();
        tokio::fs::write(temp_dir.path().join("test.txt"), "hello world\ngoodbye world").await.unwrap();

        let security = Arc::new(RwLock::new(SecurityGate::new()));
        let context = ToolContext::new(security.clone(), temp_dir.path().to_path_buf());

        let tool = SearchTool;
        let output = tool.execute(serde_json::json!({
            "query": "hello"
        }), &context).await.unwrap();

        let text = output.as_text().unwrap();
        assert!(text.contains("hello world"));
        assert!(!text.contains("goodbye"));
    }
}
