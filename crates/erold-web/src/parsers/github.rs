//! GitHub repository parser
//!
//! Parses GitHub repository information from the GitHub API.

use async_trait::async_trait;
use serde::Deserialize;

use super::{SourceInfo, SourceParser, SourceType};
use crate::error::{Result, WebError};
use crate::WebFetcher;

/// GitHub repository parser
pub struct GitHubParser;

#[derive(Debug, Deserialize)]
struct GitHubRepo {
    #[allow(dead_code)]
    name: String,
    full_name: String,
    description: Option<String>,
    homepage: Option<String>,
    html_url: String,
    topics: Option<Vec<String>>,
    default_branch: String,
}

#[async_trait]
impl SourceParser for GitHubParser {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("github.com/") && !url.contains("gist.github.com")
    }

    fn source_type(&self) -> SourceType {
        SourceType::GitHub
    }

    async fn parse(&self, fetcher: &WebFetcher, url: &str) -> Result<SourceInfo> {
        // Extract owner/repo from URL
        let (owner, repo) = extract_repo_info(url)
            .ok_or_else(|| WebError::Parse("Invalid GitHub URL".to_string()))?;

        // Fetch from GitHub API
        let api_url = format!("https://api.github.com/repos/{owner}/{repo}");
        let json = fetcher.fetch(&api_url).await?;

        let repo_info: GitHubRepo = serde_json::from_str(&json)?;

        // Try to fetch README
        let readme = fetch_readme(fetcher, &owner, &repo, &repo_info.default_branch).await;

        let mut links = vec![repo_info.html_url.clone()];
        if let Some(ref home) = repo_info.homepage {
            if !home.is_empty() {
                links.push(home.clone());
            }
        }

        Ok(SourceInfo {
            source_type: SourceType::GitHub,
            title: repo_info.full_name,
            description: repo_info.description,
            version: None, // Repos don't have versions (releases do)
            content: readme.unwrap_or_default(),
            links,
            keywords: repo_info.topics.unwrap_or_default(),
        })
    }
}

fn extract_repo_info(url: &str) -> Option<(String, String)> {
    // Remove protocol and www
    let url = url
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_start_matches("www.");

    if !url.starts_with("github.com/") {
        return None;
    }

    let path = url.trim_start_matches("github.com/");
    let parts: Vec<&str> = path.split('/').collect();

    if parts.len() >= 2 {
        let owner = parts[0].to_string();
        let repo = parts[1].trim_end_matches(".git").to_string();
        Some((owner, repo))
    } else {
        None
    }
}

async fn fetch_readme(
    fetcher: &WebFetcher,
    owner: &str,
    repo: &str,
    branch: &str,
) -> Option<String> {
    // Try different README filenames
    let filenames = ["README.md", "readme.md", "README", "readme"];

    for filename in filenames {
        let url = format!(
            "https://raw.githubusercontent.com/{owner}/{repo}/{branch}/{filename}"
        );
        if let Ok(content) = fetcher.fetch(&url).await {
            return Some(content);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_handle() {
        let parser = GitHubParser;
        assert!(parser.can_handle("https://github.com/rust-lang/rust"));
        assert!(parser.can_handle("https://www.github.com/tokio-rs/tokio"));
        assert!(!parser.can_handle("https://gitlab.com/some/repo"));
        assert!(!parser.can_handle("https://gist.github.com/user/123"));
    }

    #[test]
    fn test_extract_repo_info() {
        assert_eq!(
            extract_repo_info("https://github.com/rust-lang/rust"),
            Some(("rust-lang".to_string(), "rust".to_string()))
        );
        assert_eq!(
            extract_repo_info("https://github.com/tokio-rs/tokio/tree/master"),
            Some(("tokio-rs".to_string(), "tokio".to_string()))
        );
        assert_eq!(
            extract_repo_info("https://github.com/owner/repo.git"),
            Some(("owner".to_string(), "repo".to_string()))
        );
    }
}
