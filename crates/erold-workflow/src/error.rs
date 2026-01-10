//! Workflow error types
//!
//! Comprehensive error handling with context and recoverability.

use std::path::PathBuf;
use thiserror::Error;

/// Result type for workflow operations
pub type Result<T> = std::result::Result<T, WorkflowError>;

/// Workflow error types
#[derive(Error, Debug)]
pub enum WorkflowError {
    // =========================================================================
    // API Errors
    // =========================================================================

    /// API client error
    #[error("API error: {0}")]
    Api(#[from] erold_api::ApiError),

    /// Network timeout
    #[error("Request timed out after {duration_secs} seconds")]
    Timeout { duration_secs: u64 },

    // =========================================================================
    // Security Gate Errors
    // =========================================================================

    /// Attempted to edit file without reading it first
    #[error("Security: Must read file before editing: {}", path.display())]
    MustReadBeforeEdit { path: PathBuf },

    /// Attempted to execute without an approved plan
    #[error("Security: No approved plan. Cannot execute.")]
    NoPlanApproved,

    /// Plan was rejected by human reviewer
    #[error("Plan rejected: {}", reason.as_deref().unwrap_or("No reason provided"))]
    PlanRejected { reason: Option<String> },

    /// Approval wait timed out
    #[error("Approval timeout after {} seconds. Plan still pending.", timeout_secs)]
    ApprovalTimeout { timeout_secs: u64 },

    /// File operation outside allowed paths
    #[error("Security: Path traversal detected: {}", path.display())]
    PathTraversal { path: PathBuf },

    /// File too large to process
    #[error("Security: File too large ({size_bytes} bytes, max {max_bytes})")]
    FileTooLarge { size_bytes: usize, max_bytes: usize },

    // =========================================================================
    // State Errors
    // =========================================================================

    /// Invalid state transition
    #[error("Invalid state: Cannot {action} while in {current_state} state")]
    InvalidState {
        current_state: String,
        action: String,
    },

    /// Workflow already running
    #[error("Workflow already in progress. Wait for completion or reset.")]
    AlreadyRunning,

    // =========================================================================
    // Resource Errors
    // =========================================================================

    /// Task not found
    #[error("Task not found: {task_id}")]
    TaskNotFound { task_id: String },

    /// Project not found
    #[error("Project not found: {project_id}")]
    ProjectNotFound { project_id: String },

    /// Knowledge article not found
    #[error("Knowledge not found: {knowledge_id}")]
    KnowledgeNotFound { knowledge_id: String },

    // =========================================================================
    // Validation Errors
    // =========================================================================

    /// Invalid input
    #[error("Validation error: {message}")]
    Validation { message: String },

    /// Plan has too many subtasks
    #[error("Plan too large: {count} subtasks (max {max})")]
    PlanTooLarge { count: usize, max: usize },

    /// Empty plan
    #[error("Plan cannot be empty")]
    EmptyPlan,

    // =========================================================================
    // Phase Errors
    // =========================================================================

    /// Preprocessing failed
    #[error("Preprocessing failed: {message}")]
    PreprocessingFailed { message: String },

    /// Planning failed
    #[error("Planning failed: {message}")]
    PlanningFailed { message: String },

    /// Execution failed
    #[error("Execution failed: {message}")]
    ExecutionFailed { message: String },

    /// Enrichment failed
    #[error("Enrichment failed: {message}")]
    EnrichmentFailed { message: String },

    // =========================================================================
    // Internal Errors
    // =========================================================================

    /// Internal error (should not happen)
    #[error("Internal error: {message}")]
    Internal { message: String },
}

impl WorkflowError {
    /// Check if this error is recoverable
    ///
    /// Recoverable errors can be retried or the workflow can continue.
    #[must_use]
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::Timeout { .. }
            | Self::Api(erold_api::ApiError::RateLimited { .. })
            | Self::ApprovalTimeout { .. }
        )
    }

    /// Check if this is a security error
    ///
    /// Security errors should be logged and reported.
    #[must_use]
    pub fn is_security_error(&self) -> bool {
        matches!(
            self,
            Self::MustReadBeforeEdit { .. }
            | Self::NoPlanApproved
            | Self::PathTraversal { .. }
            | Self::FileTooLarge { .. }
        )
    }

    /// Get a safe error message for logging
    ///
    /// Removes any potentially sensitive information.
    #[must_use]
    pub fn safe_message(&self) -> String {
        match self {
            // Don't log file paths in production
            Self::MustReadBeforeEdit { .. } => "Security: Must read before edit".to_string(),
            Self::PathTraversal { .. } => "Security: Path traversal attempt".to_string(),
            // Other errors are safe to log as-is
            other => other.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_errors() {
        let err = WorkflowError::MustReadBeforeEdit {
            path: PathBuf::from("/secret/file.txt"),
        };
        assert!(err.is_security_error());
        assert!(!err.is_recoverable());
    }

    #[test]
    fn test_recoverable_errors() {
        let err = WorkflowError::Timeout { duration_secs: 30 };
        assert!(err.is_recoverable());
        assert!(!err.is_security_error());
    }

    #[test]
    fn test_safe_message_hides_paths() {
        let err = WorkflowError::PathTraversal {
            path: PathBuf::from("/etc/passwd"),
        };
        let safe = err.safe_message();
        assert!(!safe.contains("/etc/passwd"));
        assert!(safe.contains("Security"));
    }
}
