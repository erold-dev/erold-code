//! Content parsers for different source types
//!
//! Each parser knows how to extract relevant information from specific sources.

pub mod docs;
pub mod npm;
pub mod github;
pub mod crates;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::WebFetcher;

/// Information extracted from a web source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceInfo {
    /// Source type
    pub source_type: SourceType,
    /// Title or name
    pub title: String,
    /// Description
    pub description: Option<String>,
    /// Version if applicable
    pub version: Option<String>,
    /// Main content/documentation
    pub content: String,
    /// Related URLs
    pub links: Vec<String>,
    /// Keywords/tags
    pub keywords: Vec<String>,
}

/// Type of web source
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceType {
    /// npm package
    Npm,
    /// Rust crate from crates.io
    Crate,
    /// GitHub repository
    GitHub,
    /// Documentation site
    Documentation,
    /// Generic web page
    Generic,
}

/// Parser for a specific source type
#[async_trait]
pub trait SourceParser: Send + Sync {
    /// Check if this parser can handle the URL
    fn can_handle(&self, url: &str) -> bool;

    /// Get the source type
    fn source_type(&self) -> SourceType;

    /// Parse content from URL
    async fn parse(&self, fetcher: &WebFetcher, url: &str) -> Result<SourceInfo>;
}

/// Registry of source parsers
pub struct ParserRegistry {
    parsers: Vec<Box<dyn SourceParser>>,
}

impl ParserRegistry {
    /// Create a new registry with default parsers
    #[must_use]
    pub fn new() -> Self {
        Self {
            parsers: vec![
                Box::new(npm::NpmParser),
                Box::new(crates::CratesParser),
                Box::new(github::GitHubParser),
                Box::new(docs::DocsParser),
            ],
        }
    }

    /// Find a parser for the given URL
    #[must_use]
    pub fn find_parser(&self, url: &str) -> Option<&dyn SourceParser> {
        self.parsers.iter().find(|p| p.can_handle(url)).map(|p| p.as_ref())
    }

    /// Parse URL using appropriate parser
    pub async fn parse(&self, fetcher: &WebFetcher, url: &str) -> Result<SourceInfo> {
        if let Some(parser) = self.find_parser(url) {
            parser.parse(fetcher, url).await
        } else {
            // Fall back to generic parsing
            docs::DocsParser.parse(fetcher, url).await
        }
    }
}

impl Default for ParserRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_registry() {
        let registry = ParserRegistry::new();

        // Test npm URL
        assert!(registry.find_parser("https://www.npmjs.com/package/react").is_some());

        // Test crates.io URL
        assert!(registry.find_parser("https://crates.io/crates/tokio").is_some());

        // Test GitHub URL
        assert!(registry.find_parser("https://github.com/rust-lang/rust").is_some());
    }
}
