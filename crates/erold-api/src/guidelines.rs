//! Guidelines API client
//!
//! Fetches coding guidelines from erold.dev/api/v1/guidelines.
//! This is a public API - no authentication required.

use crate::error::{ApiError, Result};
use crate::models::{Guideline, GuidelinesResponse};
use reqwest::Client;
use std::time::Duration;
use tracing::{debug, instrument};

/// Default guidelines API base URL
pub const GUIDELINES_API_URL: &str = "https://erold.dev/api/v1/guidelines";

/// Guidelines API client
///
/// Fetches coding guidelines from the public erold.dev API.
/// Guidelines are organized by topic (e.g., "frontend", "backend", "security")
/// and category (e.g., "react", "fastapi", "typescript").
#[derive(Debug, Clone)]
pub struct GuidelinesClient {
    http: Client,
    base_url: String,
}

/// Filter options for fetching guidelines
#[derive(Debug, Clone, Default)]
pub struct GuidelinesFilter {
    /// Filter by topic (e.g., "frontend", "backend", "security")
    pub topic: Option<String>,
    /// Filter by category (e.g., "react", "fastapi")
    pub category: Option<String>,
    /// Filter by tags
    pub tags: Vec<String>,
    /// Maximum number of guidelines to return
    pub limit: Option<usize>,
}

impl GuidelinesFilter {
    /// Create a new empty filter
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by topic
    #[must_use]
    pub fn topic(mut self, topic: impl Into<String>) -> Self {
        self.topic = Some(topic.into());
        self
    }

    /// Filter by category
    #[must_use]
    pub fn category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    /// Add a tag filter
    #[must_use]
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Set maximum number of results
    #[must_use]
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Build query parameters
    fn to_query_params(&self) -> Vec<(String, String)> {
        let mut params = vec![];

        if let Some(ref topic) = self.topic {
            params.push(("topic".to_string(), topic.clone()));
        }
        if let Some(ref category) = self.category {
            params.push(("category".to_string(), category.clone()));
        }
        for tag in &self.tags {
            params.push(("tag".to_string(), tag.clone()));
        }
        if let Some(limit) = self.limit {
            params.push(("limit".to_string(), limit.to_string()));
        }

        params
    }
}

impl GuidelinesClient {
    /// Create a new guidelines client with default URL
    ///
    /// # Errors
    /// Returns error if HTTP client creation fails
    pub fn new() -> Result<Self> {
        Self::with_url(GUIDELINES_API_URL)
    }

    /// Create a new guidelines client with custom URL
    ///
    /// # Errors
    /// Returns error if HTTP client creation fails
    pub fn with_url(base_url: impl Into<String>) -> Result<Self> {
        let http = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(ApiError::Http)?;

        Ok(Self {
            http,
            base_url: base_url.into(),
        })
    }

    /// Fetch all guidelines (optionally filtered)
    #[instrument(skip(self))]
    pub async fn fetch(&self, filter: Option<GuidelinesFilter>) -> Result<Vec<Guideline>> {
        let mut url = self.base_url.clone();

        if let Some(f) = filter {
            let params = f.to_query_params();
            if !params.is_empty() {
                let query: Vec<String> = params
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
                    .collect();
                url = format!("{}?{}", url, query.join("&"));
            }
        }

        debug!(url = %url, "Fetching guidelines");

        let response = self.http.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(ApiError::Api {
                status,
                code: "GUIDELINES_FETCH_ERROR".to_string(),
                message: body,
            });
        }

        let guidelines_response: GuidelinesResponse = response.json().await?;
        Ok(guidelines_response.guidelines)
    }

    /// Fetch guidelines for a specific topic
    #[instrument(skip(self))]
    pub async fn fetch_by_topic(&self, topic: &str) -> Result<Vec<Guideline>> {
        self.fetch(Some(GuidelinesFilter::new().topic(topic))).await
    }

    /// Fetch guidelines for a specific topic and category
    #[instrument(skip(self))]
    pub async fn fetch_by_topic_and_category(
        &self,
        topic: &str,
        category: &str,
    ) -> Result<Vec<Guideline>> {
        self.fetch(Some(
            GuidelinesFilter::new().topic(topic).category(category),
        ))
        .await
    }

    /// Fetch guidelines by tags
    #[instrument(skip(self))]
    pub async fn fetch_by_tags(&self, tags: &[&str]) -> Result<Vec<Guideline>> {
        let mut filter = GuidelinesFilter::new();
        for tag in tags {
            filter = filter.tag(*tag);
        }
        self.fetch(Some(filter)).await
    }

    /// Fetch a single guideline by ID
    #[instrument(skip(self))]
    pub async fn fetch_by_id(&self, id: &str) -> Result<Guideline> {
        let url = format!("{}/{}", self.base_url, urlencoding::encode(id));
        debug!(url = %url, "Fetching guideline by ID");

        let response = self.http.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            if status == 404 {
                return Err(ApiError::NotFound(format!("Guideline not found: {id}")));
            }
            let body = response.text().await.unwrap_or_default();
            return Err(ApiError::Api {
                status,
                code: "GUIDELINE_FETCH_ERROR".to_string(),
                message: body,
            });
        }

        let guideline: Guideline = response.json().await?;
        Ok(guideline)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = GuidelinesClient::new();
        assert!(client.is_ok());
    }

    #[test]
    fn test_filter_builder() {
        let filter = GuidelinesFilter::new()
            .topic("frontend")
            .category("react")
            .tag("hooks")
            .limit(10);

        assert_eq!(filter.topic, Some("frontend".to_string()));
        assert_eq!(filter.category, Some("react".to_string()));
        assert_eq!(filter.tags, vec!["hooks".to_string()]);
        assert_eq!(filter.limit, Some(10));
    }

    #[test]
    fn test_filter_to_query_params() {
        let filter = GuidelinesFilter::new()
            .topic("backend")
            .category("fastapi");

        let params = filter.to_query_params();
        assert!(params.contains(&("topic".to_string(), "backend".to_string())));
        assert!(params.contains(&("category".to_string(), "fastapi".to_string())));
    }
}
