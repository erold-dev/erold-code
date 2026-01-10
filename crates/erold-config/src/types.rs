//! Configuration types

use serde::{Deserialize, Serialize};

/// Main configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EroldConfig {
    #[serde(default)]
    pub api: ApiConfig,
    #[serde(default)]
    pub workflow: WorkflowConfig,
    #[serde(default)]
    pub llm: LlmConfig,
}

impl Default for EroldConfig {
    fn default() -> Self {
        Self {
            api: ApiConfig::default(),
            workflow: WorkflowConfig::default(),
            llm: LlmConfig::default(),
        }
    }
}

/// API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub url: String,
    pub timeout_secs: u64,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            url: "https://api.erold.dev/api/v1".to_string(),
            timeout_secs: 30,
        }
    }
}

/// Workflow configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConfig {
    /// Require plan creation before execution
    pub require_plan: bool,
    /// Require human approval of plan
    pub require_approval: bool,
    /// Require file to be read before editing
    pub require_read_before_edit: bool,
    /// Automatically save learnings after tasks
    pub auto_enrich: bool,
    /// Timeout for approval wait (seconds)
    pub approval_timeout_secs: u64,
    /// Poll interval for approval check (seconds)
    pub approval_poll_secs: u64,
}

impl Default for WorkflowConfig {
    fn default() -> Self {
        Self {
            // All safety features ON by default
            require_plan: true,
            require_approval: true,
            require_read_before_edit: true,
            auto_enrich: true,
            approval_timeout_secs: 300,
            approval_poll_secs: 5,
        }
    }
}

/// LLM configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f32,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            model: "gpt-4o".to_string(),
            max_tokens: 8192,
            temperature: 0.0,
        }
    }
}

/// Project linking info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectLink {
    pub project_id: String,
    pub project_name: String,
    pub tenant_id: String,
    pub linked_at: String,
}
