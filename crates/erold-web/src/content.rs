//! Content extraction from web pages
//!
//! Extracts readable content from HTML, removing navigation, ads, etc.

use scraper::{Html, Selector};
use crate::error::Result;

/// Extracted content from a web page
#[derive(Debug, Clone)]
pub struct ExtractedContent {
    /// Page title
    pub title: Option<String>,
    /// Main content text
    pub text: String,
    /// Code blocks found
    pub code_blocks: Vec<CodeBlock>,
    /// Links found
    pub links: Vec<Link>,
}

/// A code block from the page
#[derive(Debug, Clone)]
pub struct CodeBlock {
    /// Language if specified
    pub language: Option<String>,
    /// Code content
    pub code: String,
}

/// A link from the page
#[derive(Debug, Clone)]
pub struct Link {
    /// Link text
    pub text: String,
    /// Link URL
    pub href: String,
}

/// Extract readable content from HTML
///
/// # Errors
/// Returns error if parsing fails
pub fn extract_content(html: &str) -> Result<ExtractedContent> {
    let document = Html::parse_document(html);

    // Extract title
    let title = extract_title(&document);

    // Remove unwanted elements before extracting text
    let text = extract_main_text(&document);

    // Extract code blocks
    let code_blocks = extract_code_blocks(&document);

    // Extract links
    let links = extract_links(&document);

    Ok(ExtractedContent {
        title,
        text,
        code_blocks,
        links,
    })
}

fn extract_title(doc: &Html) -> Option<String> {
    let selector = Selector::parse("title").ok()?;
    doc.select(&selector)
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
}

fn extract_main_text(doc: &Html) -> String {
    // Try to find main content area first
    let main_selectors = [
        "main",
        "article",
        "[role=\"main\"]",
        ".content",
        ".main-content",
        "#content",
        "#main",
    ];

    for selector_str in main_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(main) = doc.select(&selector).next() {
                let text = extract_text_from_element(&main);
                if !text.is_empty() {
                    return clean_text(&text);
                }
            }
        }
    }

    // Fall back to body
    if let Ok(selector) = Selector::parse("body") {
        if let Some(body) = doc.select(&selector).next() {
            return clean_text(&extract_text_from_element(&body));
        }
    }

    String::new()
}

fn extract_text_from_element(element: &scraper::ElementRef) -> String {
    // Skip certain tags
    let skip_tags = ["script", "style", "nav", "header", "footer", "aside", "noscript"];

    let mut text = String::new();

    for node in element.children() {
        if let Some(el) = scraper::ElementRef::wrap(node) {
            let tag_name = el.value().name();

            if skip_tags.contains(&tag_name) {
                continue;
            }

            // Add newlines for block elements
            if is_block_element(tag_name) && !text.is_empty() && !text.ends_with('\n') {
                text.push('\n');
            }

            text.push_str(&extract_text_from_element(&el));

            if is_block_element(tag_name) {
                text.push('\n');
            }
        } else if let Some(text_node) = node.value().as_text() {
            text.push_str(text_node);
        }
    }

    text
}

fn is_block_element(tag: &str) -> bool {
    matches!(
        tag,
        "div" | "p" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "li" | "br" | "tr" | "section"
    )
}

fn clean_text(text: &str) -> String {
    // Replace multiple whitespace with single space
    let re = regex::Regex::new(r"\s+").unwrap();
    let text = re.replace_all(text, " ");

    // Split into lines and process
    let lines: Vec<&str> = text
        .split('\n')
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();

    lines.join("\n")
}

fn extract_code_blocks(doc: &Html) -> Vec<CodeBlock> {
    let mut blocks = Vec::new();

    // Look for <pre><code> blocks
    if let Ok(selector) = Selector::parse("pre code, pre") {
        for element in doc.select(&selector) {
            let code = element.text().collect::<String>();
            if code.trim().is_empty() {
                continue;
            }

            // Try to detect language from class
            let language = element
                .value()
                .classes()
                .find(|c| c.starts_with("language-") || c.starts_with("lang-"))
                .map(|c| c.trim_start_matches("language-").trim_start_matches("lang-").to_string());

            blocks.push(CodeBlock {
                language,
                code: code.trim().to_string(),
            });
        }
    }

    blocks
}

fn extract_links(doc: &Html) -> Vec<Link> {
    let mut links = Vec::new();

    if let Ok(selector) = Selector::parse("a[href]") {
        for element in doc.select(&selector) {
            let text = element.text().collect::<String>().trim().to_string();
            if let Some(href) = element.value().attr("href") {
                // Skip anchor links and javascript
                if href.starts_with('#') || href.starts_with("javascript:") {
                    continue;
                }

                links.push(Link {
                    text,
                    href: href.to_string(),
                });
            }
        }
    }

    links
}

/// Convert extracted content to markdown
impl ExtractedContent {
    /// Convert to markdown format
    #[must_use]
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();

        if let Some(ref title) = self.title {
            md.push_str(&format!("# {title}\n\n"));
        }

        md.push_str(&self.text);

        if !self.code_blocks.is_empty() {
            md.push_str("\n\n## Code Examples\n\n");
            for block in &self.code_blocks {
                let lang = block.language.as_deref().unwrap_or("");
                md.push_str(&format!("```{lang}\n{}\n```\n\n", block.code));
            }
        }

        md
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_title() {
        let html = r#"<html><head><title>Test Page</title></head><body></body></html>"#;
        let content = extract_content(html).unwrap();
        assert_eq!(content.title, Some("Test Page".to_string()));
    }

    #[test]
    fn test_extract_text() {
        let html = r#"
            <html>
            <body>
                <main>
                    <h1>Hello World</h1>
                    <p>This is a test paragraph.</p>
                </main>
            </body>
            </html>
        "#;
        let content = extract_content(html).unwrap();
        assert!(content.text.contains("Hello World"));
        assert!(content.text.contains("test paragraph"));
    }

    #[test]
    fn test_extract_code_blocks() {
        let html = r#"
            <html>
            <body>
                <pre><code class="language-rust">fn main() {}</code></pre>
            </body>
            </html>
        "#;
        let content = extract_content(html).unwrap();
        // May find 1 or more blocks depending on selector matching
        assert!(!content.code_blocks.is_empty());
        // Check that at least one has the right language and content
        let rust_block = content.code_blocks.iter().find(|b| b.language == Some("rust".to_string()));
        assert!(rust_block.is_some());
        assert!(rust_block.unwrap().code.contains("fn main"));
    }

    #[test]
    fn test_to_markdown() {
        let content = ExtractedContent {
            title: Some("Test".to_string()),
            text: "Hello world".to_string(),
            code_blocks: vec![CodeBlock {
                language: Some("rust".to_string()),
                code: "fn main() {}".to_string(),
            }],
            links: vec![],
        };

        let md = content.to_markdown();
        assert!(md.contains("# Test"));
        assert!(md.contains("```rust"));
    }
}
