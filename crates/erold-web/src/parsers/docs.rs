//! Documentation site parser
//!
//! Generic parser for documentation websites using HTML extraction.

use async_trait::async_trait;

use super::{SourceInfo, SourceParser, SourceType};
use crate::content::extract_content;
use crate::error::Result;
use crate::WebFetcher;

/// Generic documentation parser
pub struct DocsParser;

#[async_trait]
impl SourceParser for DocsParser {
    fn can_handle(&self, url: &str) -> bool {
        // Handle common documentation sites
        let doc_domains = [
            "docs.rs",
            "doc.rust-lang.org",
            "developer.mozilla.org",
            "docs.python.org",
            "nodejs.org/docs",
            "react.dev",
            "nextjs.org/docs",
            "typescriptlang.org/docs",
            "angular.io/docs",
            "vuejs.org/guide",
            "learn.microsoft.com",
        ];

        doc_domains.iter().any(|domain| url.contains(domain))
    }

    fn source_type(&self) -> SourceType {
        SourceType::Documentation
    }

    async fn parse(&self, fetcher: &WebFetcher, url: &str) -> Result<SourceInfo> {
        let html = fetcher.fetch(url).await?;
        let content = extract_content(&html)?;

        // Extract keywords from URL path and title
        let keywords = extract_keywords_from_url(url);

        // Extract fields before converting to markdown
        let title = content.title.clone().unwrap_or_else(|| url.to_string());
        let links = content.links.iter().map(|l| l.href.clone()).collect();
        let markdown = content.to_markdown();

        Ok(SourceInfo {
            source_type: SourceType::Documentation,
            title,
            description: None,
            version: None,
            content: markdown,
            links,
            keywords,
        })
    }
}

fn extract_keywords_from_url(url: &str) -> Vec<String> {
    // Extract meaningful words from URL path
    let url = url
        .trim_start_matches("https://")
        .trim_start_matches("http://");

    let parts: Vec<&str> = url.split('/').collect();
    let mut keywords = Vec::new();

    for part in parts.iter().skip(1) {
        // Skip common words
        if matches!(
            part.to_lowercase().as_str(),
            "docs" | "api" | "guide" | "reference" | "manual" | "en" | "latest" | "stable"
        ) {
            continue;
        }

        // Clean and add non-empty parts
        let clean = part
            .trim_end_matches(".html")
            .trim_end_matches(".md")
            .replace('-', " ")
            .replace('_', " ");

        if !clean.is_empty() && clean.len() > 2 {
            keywords.push(clean);
        }
    }

    keywords
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_handle() {
        let parser = DocsParser;
        assert!(parser.can_handle("https://docs.rs/tokio"));
        assert!(parser.can_handle("https://developer.mozilla.org/en-US/docs/Web/API"));
        assert!(parser.can_handle("https://react.dev/learn"));
        assert!(!parser.can_handle("https://github.com/rust-lang/rust"));
    }

    #[test]
    fn test_extract_keywords() {
        let keywords = extract_keywords_from_url("https://docs.rs/tokio/latest/tokio/sync/index.html");
        assert!(keywords.contains(&"tokio".to_string()));
        assert!(keywords.contains(&"sync".to_string()));
    }
}
