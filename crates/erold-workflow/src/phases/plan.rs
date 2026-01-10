//! Phase 2: Planning
//!
//! Creates plan with subtasks and waits for human approval.

/// Plan item (subtask)
#[derive(Debug, Clone)]
pub struct PlanItem {
    pub title: String,
    pub description: Option<String>,
    pub keywords: Vec<String>,
}

/// Created plan
#[derive(Debug, Clone)]
pub struct Plan {
    pub task_id: String,
    pub title: String,
    pub items: Vec<PlanItem>,
}

/// Approval result
#[derive(Debug, Clone)]
pub enum ApprovalResult {
    Approved,
    Rejected { reason: Option<String> },
    Timeout,
}

// Planner implementation will be added
