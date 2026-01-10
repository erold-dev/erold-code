//! Web fetching for knowledge refresh
//!
//! Fetches and parses content from various sources:
//! - Documentation sites
//! - npm registry
//! - crates.io
//! - GitHub

mod error;
mod fetcher;
mod content;
pub mod parsers;

pub use error::{WebError, Result};
pub use fetcher::WebFetcher;
pub use content::{ExtractedContent, CodeBlock, Link, extract_content};
pub use parsers::{SourceInfo, SourceType, SourceParser, ParserRegistry};
