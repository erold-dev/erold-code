//! Web content fetcher

use crate::error::{WebError, Result};
use tracing::debug;

/// Web content fetcher
#[derive(Debug, Clone)]
pub struct WebFetcher {
    http: reqwest::Client,
}

impl WebFetcher {
    /// Create a new web fetcher
    ///
    /// # Errors
    /// Returns error if HTTP client creation fails
    pub fn new() -> Result<Self> {
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("erold-cli/0.1.0")
            .build()
            .map_err(WebError::Http)?;

        Ok(Self { http })
    }

    /// Fetch content from a URL
    ///
    /// # Errors
    /// Returns error if fetch fails
    pub async fn fetch(&self, url: &str) -> Result<String> {
        debug!(url = %url, "Fetching web content");

        let response = self.http.get(url).send().await?;

        if !response.status().is_success() {
            if response.status() == reqwest::StatusCode::NOT_FOUND {
                return Err(WebError::NotFound(url.to_string()));
            }
            if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                return Err(WebError::RateLimited);
            }
            return Err(WebError::Http(
                response.error_for_status().unwrap_err()
            ));
        }

        let content = response.text().await?;
        Ok(content)
    }
}

impl Default for WebFetcher {
    fn default() -> Self {
        Self::new().expect("Failed to create web fetcher")
    }
}
