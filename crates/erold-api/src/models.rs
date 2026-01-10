//! API data models
//!
//! Represents all data structures returned by the Erold API.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// =============================================================================
// Task Models
// =============================================================================

/// Task status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TaskStatus {
    Backlog,
    Analysis,
    Todo,
    #[serde(alias = "in_progress")]
    InProgress,
    #[serde(alias = "in_review")]
    InReview,
    Bug,
    Blocked,
    Done,
}

/// Task priority
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Urgent,
    Critical,
}

/// Agent execution status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

/// Agent execution tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentExecution {
    pub status: AgentExecutionStatus,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

/// Execution log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionLogEntry {
    pub timestamp: DateTime<Utc>,
    #[serde(rename = "type")]
    pub entry_type: String,
    pub message: String,
    pub percent: Option<i32>,
}

/// Decision made during execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub description: String,
    pub chose: String,
    pub reason: String,
    pub alternatives: Vec<String>,
}

/// Subtask within a task
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Subtask {
    pub id: String,
    pub title: String,
    pub completed: bool,
    pub order: i32,
    /// Keywords for knowledge search
    #[serde(default)]
    pub keywords: Vec<String>,
    /// IDs of knowledge articles that helped
    #[serde(default)]
    pub knowledge_used: Vec<String>,
    /// Agent observations
    pub notes: Option<String>,
    /// Decisions made during this subtask
    #[serde(default)]
    pub decisions: Vec<Decision>,
}

/// Full task model
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub project_id: String,
    pub project_title: Option<String>,

    // Assignment
    pub assigned_to: Option<String>,
    pub assignee_type: Option<String>,
    pub agent_id: Option<String>,
    pub agent_name: Option<String>,

    // Agent execution
    pub agent_execution: Option<AgentExecution>,
    #[serde(default)]
    pub execution_log: Vec<ExecutionLogEntry>,
    #[serde(default)]
    pub tools_used: Vec<String>,

    // Progress
    pub progress_percent: Option<i32>,
    #[serde(default)]
    pub subtasks: Vec<Subtask>,

    // Blocking
    pub block_reason: Option<String>,
    #[serde(default)]
    pub blocked_by: Vec<String>,

    // Completion
    pub completion_summary: Option<String>,
    pub agent_notes: Option<String>,

    // Metadata
    #[serde(default)]
    pub tags: Vec<String>,
    pub due_date: Option<DateTime<Utc>>,
    pub estimated_hours: Option<f32>,
    pub actual_hours: Option<f32>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub created_by: Option<String>,
}

/// Create task request
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTask {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<TaskStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<TaskPriority>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_name: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub subtasks: Vec<CreateSubtask>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Create subtask request
#[derive(Debug, Clone, Serialize)]
pub struct CreateSubtask {
    pub title: String,
    #[serde(default)]
    pub completed: bool,
    pub order: i32,
}

/// Update task request
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTask {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<TaskStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<TaskPriority>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress_percent: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtasks: Option<Vec<Subtask>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_execution: Option<AgentExecution>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_log: Option<Vec<ExecutionLogEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools_used: Option<Vec<String>>,
}

// =============================================================================
// Project Models
// =============================================================================

/// Project status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProjectStatus {
    Planning,
    Active,
    Started,
    #[serde(alias = "in_progress", alias = "in-progress")]
    InProgress,
    #[serde(alias = "on_hold", alias = "on-hold")]
    OnHold,
    Completed,
    Cancelled,
    Archived,
}

/// Project model
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    /// Project title (API uses "title", aliased from "name" for compatibility)
    #[serde(alias = "name")]
    pub title: String,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub status: ProjectStatus,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub assigned_users: Vec<String>,
    pub task_count: Option<i32>,
    pub completed_tasks: Option<i32>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Project {
    /// Get the display name (title)
    #[must_use]
    pub fn name(&self) -> &str {
        &self.title
    }
}

// =============================================================================
// Knowledge Models
// =============================================================================

/// Knowledge category
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KnowledgeCategory {
    Architecture,
    Api,
    Deployment,
    Testing,
    Security,
    Performance,
    Workflow,
    Conventions,
    Troubleshooting,
    Other,
}

/// Source type for knowledge auto-refresh
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceType {
    Docs,
    Npm,
    Crates,
    GitHub,
    Manual,
}

/// Knowledge article
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Knowledge {
    pub id: String,
    pub title: String,
    pub content: String,
    pub category: KnowledgeCategory,
    #[serde(default)]
    pub tags: Vec<String>,
    pub project_id: Option<String>,

    // Source tracking
    pub source: Option<String>,
    pub agent_id: Option<String>,
    pub agent_name: Option<String>,

    // Freshness (for auto-refresh)
    pub ttl_days: Option<u32>,
    pub source_url: Option<String>,
    pub source_type: Option<SourceType>,
    pub last_refreshed_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub auto_refresh: bool,

    // Metadata
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub created_by: Option<String>,
}

impl Knowledge {
    /// Check if this knowledge entry has expired based on TTL
    #[must_use]
    pub fn is_expired(&self) -> bool {
        let Some(ttl_days) = self.ttl_days else {
            return false; // No TTL means never expires
        };

        let check_date = self.last_refreshed_at.or(self.updated_at);
        let Some(date) = check_date else {
            return true; // No date means treat as expired
        };

        let age = Utc::now().signed_duration_since(date);
        age.num_days() > i64::from(ttl_days)
    }
}

/// Create knowledge request
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateKnowledge {
    pub title: String,
    pub content: String,
    pub category: KnowledgeCategory,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl_days: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_type: Option<SourceType>,
    #[serde(default)]
    pub auto_refresh: bool,
}

/// Update knowledge request
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateKnowledge {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<KnowledgeCategory>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_refreshed_at: Option<DateTime<Utc>>,
}

// =============================================================================
// Context & Dashboard Models
// =============================================================================

/// AI context response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Context {
    pub active_project: Option<Project>,
    #[serde(default)]
    pub current_tasks: Vec<Task>,
    #[serde(default)]
    pub blockers: Vec<Task>,
    #[serde(default)]
    pub relevant_knowledge: Vec<Knowledge>,
}

/// Dashboard response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dashboard {
    pub project_count: i32,
    pub task_count: i32,
    pub open_tasks: i32,
    pub blocked_tasks: i32,
    #[serde(default)]
    pub my_tasks: Vec<Task>,
    #[serde(default)]
    pub upcoming_due: Vec<Task>,
    #[serde(default)]
    pub recent_completed: Vec<Task>,
}

// =============================================================================
// API Response Wrapper
// =============================================================================

/// Standard API response wrapper
#[derive(Debug, Clone, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ApiErrorResponse>,
}

/// API error response
#[derive(Debug, Clone, Deserialize)]
pub struct ApiErrorResponse {
    pub message: String,
    pub code: Option<String>,
    pub details: Option<serde_json::Value>,
}

// =============================================================================
// Guideline Models (from erold.dev/api/v1/guidelines)
// =============================================================================

/// Guideline priority level
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GuidelinePriority {
    Critical,
    Recommended,
    Optional,
}

/// Guideline confidence level
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GuidelineConfidence {
    Established,
    Emerging,
    Experimental,
}

/// AI metadata for a guideline
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct GuidelineAI {
    /// The core rule, under 280 chars
    pub prompt_snippet: String,
    /// When should this guideline be applied
    #[serde(default)]
    pub applies_when: Vec<String>,
    /// When should this guideline be skipped
    #[serde(default)]
    pub does_not_apply_when: Vec<String>,
    /// Priority level
    pub priority: GuidelinePriority,
    /// Confidence level
    pub confidence: GuidelineConfidence,
}

/// A coding guideline from erold.dev
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Guideline {
    /// Unique identifier (e.g., "frontend/react/component-structure")
    pub id: String,
    /// Guideline title
    pub title: String,
    /// URL-friendly slug
    pub slug: String,
    /// Topic (e.g., "frontend", "backend", "security")
    pub topic: String,
    /// Category within topic (e.g., "react", "fastapi")
    pub category: String,
    /// Brief description
    pub description: Option<String>,
    /// Full markdown content
    pub content: Option<String>,
    /// AI-specific metadata
    pub ai: Option<GuidelineAI>,
    /// Tags for filtering
    #[serde(default)]
    pub tags: Vec<String>,
    /// Version
    pub version: Option<String>,
}

/// Guidelines API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuidelinesResponse {
    pub guidelines: Vec<Guideline>,
    pub total: Option<usize>,
}
