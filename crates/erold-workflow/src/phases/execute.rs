//! Phase 3: Execution
//!
//! Executes plan with per-subtask preprocessing and gates.

use erold_api::Decision;

/// Result of executing a subtask
#[derive(Debug, Clone)]
pub struct SubtaskResult {
    pub learnings: Vec<Learning>,
    pub decisions: Vec<Decision>,
    pub notes: Option<String>,
}

/// Something learned during execution
#[derive(Debug, Clone)]
pub struct Learning {
    pub title: String,
    pub content: String,
    pub category: String,
    pub tags: Vec<String>,
}

/// A mistake that was made
#[derive(Debug, Clone)]
pub struct Mistake {
    pub what_failed: String,
    pub problem: String,
    pub wrong_approach: String,
    pub why_failed: String,
    pub correct_approach: String,
    pub tags: Vec<String>,
}

/// Full execution result
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub task_id: String,
    pub task_title: String,
    pub learnings: Vec<Learning>,
    pub mistakes: Vec<Mistake>,
    pub decisions: Vec<Decision>,
}

// Executor implementation will be added
