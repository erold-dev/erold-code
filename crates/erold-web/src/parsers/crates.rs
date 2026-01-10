//! crates.io parser
//!
//! Parses Rust crate information from crates.io API.

use async_trait::async_trait;
use serde::Deserialize;

use super::{SourceInfo, SourceParser, SourceType};
use crate::error::{Result, WebError};
use crate::WebFetcher;

/// crates.io parser
pub struct CratesParser;

#[derive(Debug, Deserialize)]
struct CrateResponse {
    #[serde(rename = "crate")]
    crate_info: CrateInfo,
}

#[derive(Debug, Deserialize)]
struct CrateInfo {
    name: String,
    description: Option<String>,
    max_version: String,
    documentation: Option<String>,
    repository: Option<String>,
    homepage: Option<String>,
    keywords: Option<Vec<String>>,
    readme: Option<String>,
}

#[async_trait]
impl SourceParser for CratesParser {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("crates.io/crates/") || url.contains("docs.rs/")
    }

    fn source_type(&self) -> SourceType {
        SourceType::Crate
    }

    async fn parse(&self, fetcher: &WebFetcher, url: &str) -> Result<SourceInfo> {
        // Extract crate name from URL
        let crate_name = extract_crate_name(url)
            .ok_or_else(|| WebError::Parse("Invalid crates.io URL".to_string()))?;

        // Fetch from crates.io API
        let api_url = format!("https://crates.io/api/v1/crates/{crate_name}");
        let json = fetcher.fetch(&api_url).await?;

        let response: CrateResponse = serde_json::from_str(&json)?;
        let crate_info = response.crate_info;

        let mut links = Vec::new();
        if let Some(ref docs) = crate_info.documentation {
            links.push(docs.clone());
        }
        if let Some(ref repo) = crate_info.repository {
            links.push(repo.clone());
        }
        if let Some(ref home) = crate_info.homepage {
            links.push(home.clone());
        }

        // Try to fetch README if not included
        let content = if crate_info.readme.is_some() {
            crate_info.readme.unwrap_or_default()
        } else {
            // Try to get from docs.rs
            let docs_url = format!("https://docs.rs/{crate_name}/latest/{crate_name}/");
            fetcher.fetch(&docs_url).await.unwrap_or_default()
        };

        Ok(SourceInfo {
            source_type: SourceType::Crate,
            title: crate_info.name,
            description: crate_info.description,
            version: Some(crate_info.max_version),
            content,
            links,
            keywords: crate_info.keywords.unwrap_or_default(),
        })
    }
}

fn extract_crate_name(url: &str) -> Option<String> {
    if url.contains("crates.io/crates/") {
        let parts: Vec<&str> = url.split("crates.io/crates/").collect();
        if parts.len() > 1 {
            let name = parts[1].split('/').next()?;
            return Some(name.to_string());
        }
    }
    if url.contains("docs.rs/") {
        let parts: Vec<&str> = url.split("docs.rs/").collect();
        if parts.len() > 1 {
            let name = parts[1].split('/').next()?;
            return Some(name.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_handle() {
        let parser = CratesParser;
        assert!(parser.can_handle("https://crates.io/crates/tokio"));
        assert!(parser.can_handle("https://docs.rs/serde"));
        assert!(!parser.can_handle("https://github.com/tokio-rs/tokio"));
    }

    #[test]
    fn test_extract_crate_name() {
        assert_eq!(
            extract_crate_name("https://crates.io/crates/tokio"),
            Some("tokio".to_string())
        );
        assert_eq!(
            extract_crate_name("https://docs.rs/serde/latest/serde/"),
            Some("serde".to_string())
        );
    }
}
