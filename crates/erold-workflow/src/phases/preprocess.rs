//! Phase 1: Preprocessing
//!
//! Fetches context from Erold, refreshes stale knowledge,
//! and filters for relevance.

use erold_api::{Knowledge, Task};

/// Context after preprocessing
#[derive(Debug, Clone)]
pub struct PreprocessedContext {
    pub relevant_knowledge: Vec<Knowledge>,
    pub relevant_tasks: Vec<Task>,
    pub past_mistakes: Vec<Knowledge>,
}

/// Subtask-specific context
#[derive(Debug, Clone)]
pub struct SubtaskContext {
    pub keywords: Vec<String>,
    pub relevant_knowledge: Vec<Knowledge>,
    pub past_mistakes: Vec<Knowledge>,
}

// Preprocessor implementation will be added
