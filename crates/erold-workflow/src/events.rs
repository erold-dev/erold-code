//! Workflow events (Observer pattern)
//!
//! Events emitted during workflow execution for monitoring and logging.

use crate::state::WorkflowState;
use chrono::{DateTime, Utc};
use std::fmt;

/// Events emitted during workflow execution
#[derive(Debug, Clone)]
pub enum WorkflowEvent {
    /// Workflow started
    Started {
        task_description: String,
        timestamp: DateTime<Utc>,
    },

    /// State changed
    StateChanged {
        from: WorkflowState,
        to: WorkflowState,
        timestamp: DateTime<Utc>,
    },

    /// Preprocessing started
    PreprocessingStarted {
        timestamp: DateTime<Utc>,
    },

    /// Knowledge fetched
    KnowledgeFetched {
        total_count: usize,
        relevant_count: usize,
        expired_count: usize,
        timestamp: DateTime<Utc>,
    },

    /// Guidelines fetched from erold.dev
    GuidelinesFetched {
        topics: Vec<String>,
        count: usize,
        timestamp: DateTime<Utc>,
    },

    /// Knowledge refreshed from internet
    KnowledgeRefreshed {
        knowledge_id: String,
        source_url: String,
        timestamp: DateTime<Utc>,
    },

    /// Plan created
    PlanCreated {
        task_id: String,
        subtask_count: usize,
        timestamp: DateTime<Utc>,
    },

    /// Waiting for approval
    AwaitingApproval {
        task_id: String,
        timestamp: DateTime<Utc>,
    },

    /// Plan approved
    PlanApproved {
        task_id: String,
        timestamp: DateTime<Utc>,
    },

    /// Plan rejected
    PlanRejected {
        task_id: String,
        reason: Option<String>,
        timestamp: DateTime<Utc>,
    },

    /// Subtask started
    SubtaskStarted {
        index: usize,
        title: String,
        timestamp: DateTime<Utc>,
    },

    /// Subtask completed
    SubtaskCompleted {
        index: usize,
        title: String,
        timestamp: DateTime<Utc>,
    },

    /// File read
    FileRead {
        // Path omitted for security - just log the event
        timestamp: DateTime<Utc>,
    },

    /// File modified
    FileModified {
        // Path omitted for security - just log the event
        timestamp: DateTime<Utc>,
    },

    /// Security gate blocked operation
    SecurityBlocked {
        gate: String,
        reason: String,
        timestamp: DateTime<Utc>,
    },

    /// Learning saved
    LearningSaved {
        title: String,
        category: String,
        timestamp: DateTime<Utc>,
    },

    /// Mistake recorded
    MistakeRecorded {
        title: String,
        timestamp: DateTime<Utc>,
    },

    /// Workflow completed
    Completed {
        task_id: String,
        duration_secs: u64,
        learnings_count: usize,
        timestamp: DateTime<Utc>,
    },

    /// Workflow failed
    Failed {
        error: String,
        recoverable: bool,
        timestamp: DateTime<Utc>,
    },
}

impl WorkflowEvent {
    /// Get event timestamp
    #[must_use]
    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            Self::Started { timestamp, .. }
            | Self::StateChanged { timestamp, .. }
            | Self::PreprocessingStarted { timestamp, .. }
            | Self::KnowledgeFetched { timestamp, .. }
            | Self::GuidelinesFetched { timestamp, .. }
            | Self::KnowledgeRefreshed { timestamp, .. }
            | Self::PlanCreated { timestamp, .. }
            | Self::AwaitingApproval { timestamp, .. }
            | Self::PlanApproved { timestamp, .. }
            | Self::PlanRejected { timestamp, .. }
            | Self::SubtaskStarted { timestamp, .. }
            | Self::SubtaskCompleted { timestamp, .. }
            | Self::FileRead { timestamp, .. }
            | Self::FileModified { timestamp, .. }
            | Self::SecurityBlocked { timestamp, .. }
            | Self::LearningSaved { timestamp, .. }
            | Self::MistakeRecorded { timestamp, .. }
            | Self::Completed { timestamp, .. }
            | Self::Failed { timestamp, .. } => *timestamp,
        }
    }

    /// Get event name for logging
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Started { .. } => "workflow.started",
            Self::StateChanged { .. } => "workflow.state_changed",
            Self::PreprocessingStarted { .. } => "workflow.preprocessing_started",
            Self::KnowledgeFetched { .. } => "workflow.knowledge_fetched",
            Self::GuidelinesFetched { .. } => "workflow.guidelines_fetched",
            Self::KnowledgeRefreshed { .. } => "workflow.knowledge_refreshed",
            Self::PlanCreated { .. } => "workflow.plan_created",
            Self::AwaitingApproval { .. } => "workflow.awaiting_approval",
            Self::PlanApproved { .. } => "workflow.plan_approved",
            Self::PlanRejected { .. } => "workflow.plan_rejected",
            Self::SubtaskStarted { .. } => "workflow.subtask_started",
            Self::SubtaskCompleted { .. } => "workflow.subtask_completed",
            Self::FileRead { .. } => "workflow.file_read",
            Self::FileModified { .. } => "workflow.file_modified",
            Self::SecurityBlocked { .. } => "workflow.security_blocked",
            Self::LearningSaved { .. } => "workflow.learning_saved",
            Self::MistakeRecorded { .. } => "workflow.mistake_recorded",
            Self::Completed { .. } => "workflow.completed",
            Self::Failed { .. } => "workflow.failed",
        }
    }

    /// Check if this is an error event
    #[must_use]
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Failed { .. } | Self::SecurityBlocked { .. })
    }
}

impl fmt::Display for WorkflowEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Started { task_description, .. } => {
                write!(f, "Workflow started: {}", truncate(task_description, 50))
            }
            Self::StateChanged { from, to, .. } => {
                write!(f, "State: {:?} -> {:?}", from, to)
            }
            Self::PreprocessingStarted { .. } => {
                write!(f, "Preprocessing started")
            }
            Self::KnowledgeFetched { total_count, relevant_count, expired_count, .. } => {
                write!(
                    f,
                    "Knowledge fetched: {} total, {} relevant, {} expired",
                    total_count, relevant_count, expired_count
                )
            }
            Self::GuidelinesFetched { topics, count, .. } => {
                write!(
                    f,
                    "Guidelines fetched: {} guidelines for topics [{}]",
                    count,
                    topics.join(", ")
                )
            }
            Self::KnowledgeRefreshed { knowledge_id, .. } => {
                write!(f, "Knowledge refreshed: {}", knowledge_id)
            }
            Self::PlanCreated { task_id, subtask_count, .. } => {
                write!(f, "Plan created: {} ({} subtasks)", task_id, subtask_count)
            }
            Self::AwaitingApproval { task_id, .. } => {
                write!(f, "Awaiting approval: {}", task_id)
            }
            Self::PlanApproved { task_id, .. } => {
                write!(f, "Plan approved: {}", task_id)
            }
            Self::PlanRejected { task_id, reason, .. } => {
                write!(
                    f,
                    "Plan rejected: {} ({})",
                    task_id,
                    reason.as_deref().unwrap_or("no reason")
                )
            }
            Self::SubtaskStarted { index, title, .. } => {
                write!(f, "Subtask {}: {}", index + 1, truncate(title, 40))
            }
            Self::SubtaskCompleted { index, title, .. } => {
                write!(f, "Subtask {} completed: {}", index + 1, truncate(title, 40))
            }
            Self::FileRead { .. } => write!(f, "File read"),
            Self::FileModified { .. } => write!(f, "File modified"),
            Self::SecurityBlocked { gate, reason, .. } => {
                write!(f, "Security blocked [{}]: {}", gate, reason)
            }
            Self::LearningSaved { title, category, .. } => {
                write!(f, "Learning saved [{}]: {}", category, truncate(title, 40))
            }
            Self::MistakeRecorded { title, .. } => {
                write!(f, "Mistake recorded: {}", truncate(title, 40))
            }
            Self::Completed { task_id, duration_secs, learnings_count, .. } => {
                write!(
                    f,
                    "Workflow completed: {} ({}s, {} learnings)",
                    task_id, duration_secs, learnings_count
                )
            }
            Self::Failed { error, recoverable, .. } => {
                write!(
                    f,
                    "Workflow failed{}: {}",
                    if *recoverable { " (recoverable)" } else { "" },
                    truncate(error, 50)
                )
            }
        }
    }
}

/// Truncate string for display
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

/// Handler for workflow events
pub trait WorkflowEventHandler: Send + Sync {
    /// Handle an event
    fn handle(&self, event: &WorkflowEvent);
}

/// Default event handler that logs events
pub struct LoggingEventHandler;

impl WorkflowEventHandler for LoggingEventHandler {
    fn handle(&self, event: &WorkflowEvent) {
        if event.is_error() {
            tracing::error!(
                event = event.name(),
                timestamp = %event.timestamp(),
                "{}",
                event
            );
        } else {
            tracing::info!(
                event = event.name(),
                timestamp = %event.timestamp(),
                "{}",
                event
            );
        }
    }
}

/// No-op event handler for testing
pub struct NoOpEventHandler;

impl WorkflowEventHandler for NoOpEventHandler {
    fn handle(&self, _event: &WorkflowEvent) {
        // Do nothing
    }
}
