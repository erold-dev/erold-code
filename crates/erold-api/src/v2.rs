//! V2 API client — Context Engine endpoints
//!
//! Provides access to the v2 context engine: fragments, intents, and context retrieval.

use crate::error::{ApiError, Result};
use crate::models::*;
use reqwest::{Client, Response, StatusCode};
use std::time::Duration;
use tracing::{debug, instrument};

/// V2 API client for the context engine
#[derive(Debug, Clone)]
pub struct EroldV2Client {
    http: Client,
    base_url: String,
    api_key: String,
    tenant_id: String,
}

impl EroldV2Client {
    /// Create a new V2 API client
    ///
    /// # Errors
    /// Returns error if HTTP client creation fails
    pub fn new(
        base_url: impl Into<String>,
        api_key: impl Into<String>,
        tenant_id: impl Into<String>,
    ) -> Result<Self> {
        let http = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(ApiError::Http)?;

        Ok(Self {
            http,
            base_url: base_url.into(),
            api_key: api_key.into(),
            tenant_id: tenant_id.into(),
        })
    }

    // =========================================================================
    // Internal helpers
    // =========================================================================

    fn url(&self, path: &str) -> String {
        format!(
            "{}/tenants/{}/{}",
            self.base_url,
            self.tenant_id,
            path.trim_start_matches('/')
        )
    }

    async fn handle_response<T: serde::de::DeserializeOwned>(
        &self,
        response: Response,
    ) -> Result<T> {
        let status = response.status();
        let body_text = response.text().await.unwrap_or_default();

        if status.is_success() {
            // Try to parse as wrapped response first
            if let Ok(wrapped) = serde_json::from_str::<ApiResponse<T>>(&body_text) {
                if wrapped.success {
                    if let Some(data) = wrapped.data {
                        return Ok(data);
                    }
                }
                if let Some(error) = wrapped.error {
                    return Err(ApiError::Api {
                        status: status.as_u16(),
                        code: error.code.unwrap_or_default(),
                        message: error.message,
                    });
                }
            }
            // Fall back to direct parsing
            let body = serde_json::from_str::<T>(&body_text)?;
            return Ok(body);
        }

        match status {
            StatusCode::NOT_FOUND => Err(ApiError::NotFound(body_text)),
            StatusCode::UNAUTHORIZED => Err(ApiError::Unauthorized(body_text)),
            StatusCode::TOO_MANY_REQUESTS => Err(ApiError::RateLimited { retry_after: 60 }),
            _ => {
                if let Ok(api_response) = serde_json::from_str::<ApiResponse<()>>(&body_text) {
                    if let Some(error) = api_response.error {
                        return Err(ApiError::Api {
                            status: status.as_u16(),
                            code: error.code.unwrap_or_default(),
                            message: error.message,
                        });
                    }
                }
                Err(ApiError::Api {
                    status: status.as_u16(),
                    code: "UNKNOWN".to_string(),
                    message: body_text,
                })
            }
        }
    }

    /// Handle a response that returns no meaningful body (e.g. 204)
    async fn handle_empty_response(&self, response: Response) -> Result<()> {
        let status = response.status();
        if status.is_success() {
            return Ok(());
        }

        let body_text = response.text().await.unwrap_or_default();
        match status {
            StatusCode::NOT_FOUND => Err(ApiError::NotFound(body_text)),
            StatusCode::UNAUTHORIZED => Err(ApiError::Unauthorized(body_text)),
            _ => Err(ApiError::Api {
                status: status.as_u16(),
                code: "UNKNOWN".to_string(),
                message: body_text,
            }),
        }
    }

    // =========================================================================
    // Context
    // =========================================================================

    /// Fetch full project context (project info, tech info, active intents, recent fragments)
    #[instrument(skip(self))]
    pub async fn get_context(&self, project_id: &str) -> Result<ContextV2Response> {
        debug!(project_id = %project_id, "Fetching v2 context");
        let response = self
            .http
            .get(self.url(&format!("/projects/{project_id}/context")))
            .header("X-API-Key", &self.api_key)
            .header("X-Tenant-ID", &self.tenant_id)
            .send()
            .await?;

        self.handle_response(response).await
    }

    // =========================================================================
    // Fragments
    // =========================================================================

    /// Log an event as a fragment
    #[instrument(skip(self))]
    pub async fn log_event(
        &self,
        project_id: &str,
        content: &str,
        event_type: &str,
        intent_id: Option<&str>,
    ) -> Result<()> {
        debug!(project_id = %project_id, event_type = %event_type, "Logging event");

        let body = LogEventRequest {
            content: content.to_string(),
            event_type: event_type.to_string(),
            intent_id: intent_id.map(String::from),
        };

        let response = self
            .http
            .post(self.url(&format!("/projects/{project_id}/fragments")))
            .header("X-API-Key", &self.api_key)
            .header("X-Tenant-ID", &self.tenant_id)
            .json(&body)
            .send()
            .await?;

        self.handle_empty_response(response).await
    }

    /// Search fragments by query
    #[instrument(skip(self))]
    pub async fn search_fragments(
        &self,
        project_id: &str,
        query: &str,
    ) -> Result<Vec<Fragment>> {
        debug!(project_id = %project_id, query = %query, "Searching fragments");

        let url = format!(
            "{}?search={}",
            self.url(&format!("/projects/{project_id}/fragments")),
            urlencoding::encode(query)
        );

        let response = self
            .http
            .get(&url)
            .header("X-API-Key", &self.api_key)
            .header("X-Tenant-ID", &self.tenant_id)
            .send()
            .await?;

        self.handle_response(response).await
    }

    // =========================================================================
    // Intents
    // =========================================================================

    /// List intents, optionally filtered by status
    #[instrument(skip(self))]
    pub async fn list_intents(
        &self,
        project_id: &str,
        status: Option<&str>,
    ) -> Result<Vec<Intent>> {
        debug!(project_id = %project_id, status = ?status, "Listing intents");

        let mut url = self.url(&format!("/projects/{project_id}/intents"));
        if let Some(s) = status {
            url = format!("{url}?status={s}");
        }

        let response = self
            .http
            .get(&url)
            .header("X-API-Key", &self.api_key)
            .header("X-Tenant-ID", &self.tenant_id)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Create a new intent
    #[instrument(skip(self))]
    pub async fn create_intent(
        &self,
        project_id: &str,
        title: &str,
    ) -> Result<Intent> {
        debug!(project_id = %project_id, title = %title, "Creating intent");

        let body = CreateIntentRequest {
            title: title.to_string(),
        };

        let response = self
            .http
            .post(self.url(&format!("/projects/{project_id}/intents")))
            .header("X-API-Key", &self.api_key)
            .header("X-Tenant-ID", &self.tenant_id)
            .json(&body)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Complete an intent with a summary
    #[instrument(skip(self))]
    pub async fn complete_intent(
        &self,
        project_id: &str,
        intent_id: &str,
        summary: &str,
    ) -> Result<Intent> {
        debug!(project_id = %project_id, intent_id = %intent_id, "Completing intent");

        let body = CompleteIntentRequest {
            summary: summary.to_string(),
        };

        let response = self
            .http
            .patch(self.url(&format!("/projects/{project_id}/intents/{intent_id}/complete")))
            .header("X-API-Key", &self.api_key)
            .header("X-Tenant-ID", &self.tenant_id)
            .json(&body)
            .send()
            .await?;

        self.handle_response(response).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_v2_client_creation() {
        let client = EroldV2Client::new(
            "https://api.erold.dev/v2",
            "test_key",
            "test_tenant",
        );
        assert!(client.is_ok());
    }
}
