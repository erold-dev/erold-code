//! Erold API HTTP client

use crate::error::{ApiError, Result};
use crate::models::*;
use reqwest::{Client, Response, StatusCode};
use std::time::Duration;
use tracing::{debug, instrument};

/// Erold API client
#[derive(Debug, Clone)]
pub struct EroldClient {
    http: Client,
    base_url: String,
    api_key: String,
    tenant_id: String,
    project_id: Option<String>,
}

impl EroldClient {
    /// Create a new API client
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
            project_id: None,
        })
    }

    /// Set the active project ID
    #[must_use]
    pub fn with_project(mut self, project_id: impl Into<String>) -> Self {
        self.project_id = Some(project_id.into());
        self
    }

    /// Get the current project ID
    #[must_use]
    pub fn project_id(&self) -> Option<&str> {
        self.project_id.as_deref()
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
                // If wrapped but no data or error
                if let Some(error) = wrapped.error {
                    return Err(ApiError::Api {
                        status: status.as_u16(),
                        code: error.code.unwrap_or_default(),
                        message: error.message,
                    });
                }
            }
            // Fall back to direct parsing (for non-wrapped responses)
            let body = serde_json::from_str::<T>(&body_text)?;
            return Ok(body);
        }

        // Handle error responses
        match status {
            StatusCode::NOT_FOUND => Err(ApiError::NotFound(body_text)),
            StatusCode::UNAUTHORIZED => Err(ApiError::Unauthorized(body_text)),
            StatusCode::TOO_MANY_REQUESTS => {
                Err(ApiError::RateLimited { retry_after: 60 })
            }
            _ => {
                // Try to parse as API error
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

    // =========================================================================
    // Projects
    // =========================================================================

    /// List all projects
    #[instrument(skip(self))]
    pub async fn list_projects(&self) -> Result<Vec<Project>> {
        debug!("Listing projects");
        let response = self
            .http
            .get(self.url("/projects"))
            .header("X-API-Key", &self.api_key)
            .header("X-Tenant-ID", &self.tenant_id)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Get a single project by ID
    #[instrument(skip(self))]
    pub async fn get_project(&self, id: &str) -> Result<Project> {
        debug!(project_id = %id, "Getting project");
        let response = self
            .http
            .get(self.url(&format!("/projects/{id}")))
            .header("X-API-Key", &self.api_key)
            .header("X-Tenant-ID", &self.tenant_id)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Create a new project
    #[instrument(skip(self))]
    pub async fn create_project(&self, title: &str, description: Option<&str>) -> Result<Project> {
        debug!(title = %title, "Creating project");

        let mut body = serde_json::json!({
            "title": title,
            "status": "active"
        });

        if let Some(desc) = description {
            body["description"] = serde_json::json!(desc);
        }

        let response = self
            .http
            .post(self.url("/projects"))
            .header("X-API-Key", &self.api_key)
            .header("X-Tenant-ID", &self.tenant_id)
            .json(&body)
            .send()
            .await?;

        self.handle_response(response).await
    }

    // =========================================================================
    // Tasks
    // =========================================================================

    /// List tasks with optional filters
    #[instrument(skip(self))]
    pub async fn list_tasks(&self, project_id: Option<&str>) -> Result<Vec<Task>> {
        let mut url = self.url("/tasks");
        if let Some(pid) = project_id.or(self.project_id.as_deref()) {
            url = format!("{url}?projectId={pid}");
        }

        debug!("Listing tasks");
        let response = self
            .http
            .get(&url)
            .header("X-API-Key", &self.api_key)
            .header("X-Tenant-ID", &self.tenant_id)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Get a single task by ID
    #[instrument(skip(self))]
    pub async fn get_task(&self, id: &str) -> Result<Task> {
        debug!(task_id = %id, "Getting task");
        let response = self
            .http
            .get(self.url(&format!("/tasks/{id}")))
            .header("X-API-Key", &self.api_key)
            .header("X-Tenant-ID", &self.tenant_id)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Create a new task
    #[instrument(skip(self, task))]
    pub async fn create_task(&self, project_id: &str, task: &CreateTask) -> Result<Task> {
        debug!(project_id = %project_id, title = %task.title, "Creating task");
        let response = self
            .http
            .post(self.url(&format!("/projects/{project_id}/tasks")))
            .header("X-API-Key", &self.api_key)
            .header("X-Tenant-ID", &self.tenant_id)
            .json(task)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Update a task
    #[instrument(skip(self, update))]
    pub async fn update_task(&self, id: &str, update: &UpdateTask) -> Result<Task> {
        debug!(task_id = %id, "Updating task");
        let response = self
            .http
            .patch(self.url(&format!("/tasks/{id}")))
            .header("X-API-Key", &self.api_key)
            .header("X-Tenant-ID", &self.tenant_id)
            .json(update)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Start a task (set to in-progress)
    #[instrument(skip(self))]
    pub async fn start_task(&self, id: &str) -> Result<Task> {
        debug!(task_id = %id, "Starting task");

        // Use PATCH to update status to "in-progress"
        let body = serde_json::json!({ "status": "inProgress" });

        let response = self
            .http
            .patch(self.url(&format!("/tasks/{id}")))
            .header("X-API-Key", &self.api_key)
            .header("X-Tenant-ID", &self.tenant_id)
            .json(&body)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Complete a task
    #[instrument(skip(self))]
    pub async fn complete_task(&self, id: &str, summary: Option<&str>) -> Result<Task> {
        debug!(task_id = %id, "Completing task");

        // Use PATCH to update status to "done" (API doesn't have a /complete endpoint)
        let mut body = serde_json::json!({ "status": "done" });
        if let Some(s) = summary {
            body["completionSummary"] = serde_json::json!(s);
        }

        let response = self
            .http
            .patch(self.url(&format!("/tasks/{id}")))
            .header("X-API-Key", &self.api_key)
            .header("X-Tenant-ID", &self.tenant_id)
            .json(&body)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Block a task with a reason
    #[instrument(skip(self))]
    pub async fn block_task(&self, id: &str, reason: &str) -> Result<Task> {
        debug!(task_id = %id, reason = %reason, "Blocking task");

        // Use PATCH to update status to "blocked"
        let body = serde_json::json!({
            "status": "blocked",
            "blockReason": reason
        });

        let response = self
            .http
            .patch(self.url(&format!("/tasks/{id}")))
            .header("X-API-Key", &self.api_key)
            .header("X-Tenant-ID", &self.tenant_id)
            .json(&body)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Get my assigned tasks
    #[instrument(skip(self))]
    pub async fn get_my_tasks(&self) -> Result<Vec<Task>> {
        debug!("Getting my tasks");
        let response = self
            .http
            .get(self.url("/tasks/mine"))
            .header("X-API-Key", &self.api_key)
            .header("X-Tenant-ID", &self.tenant_id)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Get blocked tasks
    #[instrument(skip(self))]
    pub async fn get_blocked_tasks(&self) -> Result<Vec<Task>> {
        debug!("Getting blocked tasks");
        let response = self
            .http
            .get(self.url("/tasks/blocked"))
            .header("X-API-Key", &self.api_key)
            .header("X-Tenant-ID", &self.tenant_id)
            .send()
            .await?;

        self.handle_response(response).await
    }

    // =========================================================================
    // Knowledge
    // =========================================================================

    /// List knowledge articles
    #[instrument(skip(self))]
    pub async fn list_knowledge(
        &self,
        category: Option<KnowledgeCategory>,
        project_id: Option<&str>,
    ) -> Result<Vec<Knowledge>> {
        let mut url = self.url("/knowledge");
        let mut params = vec![];

        if let Some(cat) = category {
            params.push(format!("category={}", serde_json::to_string(&cat).unwrap().trim_matches('"')));
        }
        if let Some(pid) = project_id.or(self.project_id.as_deref()) {
            params.push(format!("projectId={pid}"));
        }

        if !params.is_empty() {
            url = format!("{url}?{}", params.join("&"));
        }

        debug!("Listing knowledge");
        let response = self
            .http
            .get(&url)
            .header("X-API-Key", &self.api_key)
            .header("X-Tenant-ID", &self.tenant_id)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Search knowledge by query
    #[instrument(skip(self))]
    pub async fn search_knowledge(&self, query: &str) -> Result<Vec<Knowledge>> {
        debug!(query = %query, "Searching knowledge");
        let url = format!("{}?search={}", self.url("/knowledge"), urlencoding::encode(query));

        let response = self
            .http
            .get(&url)
            .header("X-API-Key", &self.api_key)
            .header("X-Tenant-ID", &self.tenant_id)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Search knowledge by category and keywords
    #[instrument(skip(self))]
    pub async fn search_knowledge_by_category(
        &self,
        category: &str,
        keywords: &[String],
    ) -> Result<Vec<Knowledge>> {
        debug!(category = %category, keywords = ?keywords, "Searching knowledge by category");
        let query = keywords.join(" ");
        let url = format!(
            "{}?category={}&search={}",
            self.url("/knowledge"),
            category,
            urlencoding::encode(&query)
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

    /// Get a single knowledge article
    #[instrument(skip(self))]
    pub async fn get_knowledge(&self, id: &str) -> Result<Knowledge> {
        debug!(knowledge_id = %id, "Getting knowledge");
        let response = self
            .http
            .get(self.url(&format!("/knowledge/{id}")))
            .header("X-API-Key", &self.api_key)
            .header("X-Tenant-ID", &self.tenant_id)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Create a knowledge article
    #[instrument(skip(self, knowledge))]
    pub async fn create_knowledge(&self, knowledge: &CreateKnowledge) -> Result<Knowledge> {
        debug!(title = %knowledge.title, category = ?knowledge.category, "Creating knowledge");
        let response = self
            .http
            .post(self.url("/knowledge"))
            .header("X-API-Key", &self.api_key)
            .header("X-Tenant-ID", &self.tenant_id)
            .json(knowledge)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Update a knowledge article
    #[instrument(skip(self, update))]
    pub async fn update_knowledge(&self, id: &str, update: &UpdateKnowledge) -> Result<Knowledge> {
        debug!(knowledge_id = %id, "Updating knowledge");
        let response = self
            .http
            .patch(self.url(&format!("/knowledge/{id}")))
            .header("X-API-Key", &self.api_key)
            .header("X-Tenant-ID", &self.tenant_id)
            .json(update)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Delete a knowledge article
    #[instrument(skip(self))]
    pub async fn delete_knowledge(&self, id: &str) -> Result<()> {
        debug!(knowledge_id = %id, "Deleting knowledge");
        let response = self
            .http
            .delete(self.url(&format!("/knowledge/{id}")))
            .header("X-API-Key", &self.api_key)
            .header("X-Tenant-ID", &self.tenant_id)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            self.handle_response::<()>(response).await
        }
    }

    // =========================================================================
    // Context & Dashboard
    // =========================================================================

    /// Get AI context (optimized for agents)
    #[instrument(skip(self))]
    pub async fn get_context(&self) -> Result<Context> {
        debug!("Getting context");
        let response = self
            .http
            .get(self.url("/context"))
            .header("X-API-Key", &self.api_key)
            .header("X-Tenant-ID", &self.tenant_id)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Get dashboard data
    #[instrument(skip(self))]
    pub async fn get_dashboard(&self) -> Result<Dashboard> {
        debug!("Getting dashboard");
        let response = self
            .http
            .get(self.url("/dashboard"))
            .header("X-API-Key", &self.api_key)
            .header("X-Tenant-ID", &self.tenant_id)
            .send()
            .await?;

        self.handle_response(response).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = EroldClient::new(
            "https://api.erold.dev/v1",
            "test_key",
            "test_tenant",
        );
        assert!(client.is_ok());
    }

    #[test]
    fn test_with_project() {
        let client = EroldClient::new(
            "https://api.erold.dev/v1",
            "test_key",
            "test_tenant",
        )
        .unwrap()
        .with_project("proj_123");

        assert_eq!(client.project_id(), Some("proj_123"));
    }
}
