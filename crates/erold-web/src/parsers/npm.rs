//! npm package parser
//!
//! Parses npm package information from the npm registry API.

use async_trait::async_trait;
use serde::Deserialize;

use super::{SourceInfo, SourceParser, SourceType};
use crate::error::{Result, WebError};
use crate::WebFetcher;

/// npm package parser
pub struct NpmParser;

#[derive(Debug, Deserialize)]
struct NpmPackage {
    name: String,
    description: Option<String>,
    #[serde(rename = "dist-tags")]
    dist_tags: Option<DistTags>,
    readme: Option<String>,
    keywords: Option<Vec<String>>,
    homepage: Option<String>,
    repository: Option<Repository>,
}

#[derive(Debug, Deserialize)]
struct DistTags {
    latest: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Repository {
    url: Option<String>,
}

#[async_trait]
impl SourceParser for NpmParser {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("npmjs.com/package/") || url.contains("registry.npmjs.org/")
    }

    fn source_type(&self) -> SourceType {
        SourceType::Npm
    }

    async fn parse(&self, fetcher: &WebFetcher, url: &str) -> Result<SourceInfo> {
        // Extract package name from URL
        let package_name = extract_package_name(url)
            .ok_or_else(|| WebError::Parse("Invalid npm URL".to_string()))?;

        // Fetch from npm registry API
        let api_url = format!("https://registry.npmjs.org/{package_name}");
        let json = fetcher.fetch(&api_url).await?;

        let package: NpmPackage = serde_json::from_str(&json)?;

        let version = package.dist_tags.and_then(|dt| dt.latest);

        let mut links = Vec::new();
        if let Some(ref homepage) = package.homepage {
            links.push(homepage.clone());
        }
        if let Some(ref repo) = package.repository {
            if let Some(ref url) = repo.url {
                links.push(url.clone());
            }
        }

        Ok(SourceInfo {
            source_type: SourceType::Npm,
            title: package.name,
            description: package.description,
            version,
            content: package.readme.unwrap_or_default(),
            links,
            keywords: package.keywords.unwrap_or_default(),
        })
    }
}

fn extract_package_name(url: &str) -> Option<String> {
    // Handle scoped packages (@org/package)
    if url.contains("npmjs.com/package/") {
        let parts: Vec<&str> = url.split("npmjs.com/package/").collect();
        if parts.len() > 1 {
            return Some(parts[1].trim_end_matches('/').to_string());
        }
    }
    if url.contains("registry.npmjs.org/") {
        let parts: Vec<&str> = url.split("registry.npmjs.org/").collect();
        if parts.len() > 1 {
            return Some(parts[1].trim_end_matches('/').to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_handle() {
        let parser = NpmParser;
        assert!(parser.can_handle("https://www.npmjs.com/package/react"));
        assert!(parser.can_handle("https://registry.npmjs.org/express"));
        assert!(!parser.can_handle("https://github.com/facebook/react"));
    }

    #[test]
    fn test_extract_package_name() {
        assert_eq!(
            extract_package_name("https://www.npmjs.com/package/react"),
            Some("react".to_string())
        );
        assert_eq!(
            extract_package_name("https://www.npmjs.com/package/@types/node"),
            Some("@types/node".to_string())
        );
    }
}
