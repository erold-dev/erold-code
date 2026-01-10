//! Workflow state machine
//!
//! Type-safe state transitions with compile-time guarantees.

use std::fmt;

/// Workflow states
///
/// The workflow progresses through these states in order.
/// Invalid transitions are prevented at compile time where possible,
/// and at runtime otherwise.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WorkflowState {
    /// Initial state - workflow not started
    Idle,
    /// Preprocessing phase - fetching context, checking knowledge TTL
    Preprocessing,
    /// Planning phase - creating plan, waiting for approval
    Planning,
    /// Waiting for human approval
    AwaitingApproval,
    /// Execution phase - running subtasks
    Executing,
    /// Enrichment phase - saving learnings, completing task
    Enriching,
    /// Workflow completed successfully
    Completed,
    /// Workflow failed
    Failed,
    /// Workflow cancelled by user
    Cancelled,
}

impl WorkflowState {
    /// Check if this is a terminal state
    #[must_use]
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }

    /// Check if this is an active state (workflow in progress)
    #[must_use]
    pub fn is_active(&self) -> bool {
        matches!(
            self,
            Self::Preprocessing
                | Self::Planning
                | Self::AwaitingApproval
                | Self::Executing
                | Self::Enriching
        )
    }

    /// Get valid transitions from this state
    #[must_use]
    pub fn valid_transitions(&self) -> &'static [WorkflowState] {
        match self {
            Self::Idle => &[Self::Preprocessing, Self::Failed, Self::Cancelled],
            Self::Preprocessing => &[Self::Planning, Self::Failed, Self::Cancelled],
            Self::Planning => &[Self::AwaitingApproval, Self::Failed, Self::Cancelled],
            Self::AwaitingApproval => &[Self::Executing, Self::Planning, Self::Failed, Self::Cancelled],
            Self::Executing => &[Self::Enriching, Self::Failed, Self::Cancelled],
            Self::Enriching => &[Self::Completed, Self::Failed, Self::Cancelled],
            Self::Completed | Self::Failed | Self::Cancelled => &[],
        }
    }

    /// Check if a transition to the target state is valid
    #[must_use]
    pub fn can_transition_to(&self, target: WorkflowState) -> bool {
        self.valid_transitions().contains(&target)
    }
}

impl fmt::Display for WorkflowState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Idle => write!(f, "Idle"),
            Self::Preprocessing => write!(f, "Preprocessing"),
            Self::Planning => write!(f, "Planning"),
            Self::AwaitingApproval => write!(f, "Awaiting Approval"),
            Self::Executing => write!(f, "Executing"),
            Self::Enriching => write!(f, "Enriching"),
            Self::Completed => write!(f, "Completed"),
            Self::Failed => write!(f, "Failed"),
            Self::Cancelled => write!(f, "Cancelled"),
        }
    }
}

/// State transition result
#[derive(Debug, Clone)]
pub struct StateTransition {
    /// Previous state
    pub from: WorkflowState,
    /// New state
    pub to: WorkflowState,
    /// Reason for transition (optional)
    pub reason: Option<String>,
}

impl StateTransition {
    /// Create a new state transition
    #[must_use]
    pub fn new(from: WorkflowState, to: WorkflowState) -> Self {
        Self {
            from,
            to,
            reason: None,
        }
    }

    /// Create a state transition with a reason
    #[must_use]
    pub fn with_reason(from: WorkflowState, to: WorkflowState, reason: impl Into<String>) -> Self {
        Self {
            from,
            to,
            reason: Some(reason.into()),
        }
    }
}

/// State machine for workflow progression
#[derive(Debug)]
pub struct StateMachine {
    current: WorkflowState,
    history: Vec<StateTransition>,
}

impl StateMachine {
    /// Create a new state machine starting at Idle
    #[must_use]
    pub fn new() -> Self {
        Self {
            current: WorkflowState::Idle,
            history: Vec::new(),
        }
    }

    /// Get current state
    #[must_use]
    pub fn current(&self) -> WorkflowState {
        self.current
    }

    /// Get transition history
    #[must_use]
    pub fn history(&self) -> &[StateTransition] {
        &self.history
    }

    /// Attempt to transition to a new state
    ///
    /// Returns `Ok(transition)` if successful, `Err` if invalid transition.
    pub fn transition(&mut self, to: WorkflowState) -> Result<StateTransition, InvalidTransition> {
        if !self.current.can_transition_to(to) {
            return Err(InvalidTransition {
                from: self.current,
                to,
            });
        }

        let transition = StateTransition::new(self.current, to);
        self.current = to;
        self.history.push(transition.clone());
        Ok(transition)
    }

    /// Attempt to transition with a reason
    pub fn transition_with_reason(
        &mut self,
        to: WorkflowState,
        reason: impl Into<String>,
    ) -> Result<StateTransition, InvalidTransition> {
        if !self.current.can_transition_to(to) {
            return Err(InvalidTransition {
                from: self.current,
                to,
            });
        }

        let transition = StateTransition::with_reason(self.current, to, reason);
        self.current = to;
        self.history.push(transition.clone());
        Ok(transition)
    }

    /// Force transition to Failed state (always allowed)
    pub fn fail(&mut self, reason: impl Into<String>) -> StateTransition {
        let transition = StateTransition::with_reason(self.current, WorkflowState::Failed, reason);
        self.current = WorkflowState::Failed;
        self.history.push(transition.clone());
        transition
    }

    /// Force transition to Cancelled state (always allowed from non-terminal states)
    pub fn cancel(&mut self, reason: impl Into<String>) -> Result<StateTransition, InvalidTransition> {
        if self.current.is_terminal() {
            return Err(InvalidTransition {
                from: self.current,
                to: WorkflowState::Cancelled,
            });
        }

        let transition = StateTransition::with_reason(self.current, WorkflowState::Cancelled, reason);
        self.current = WorkflowState::Cancelled;
        self.history.push(transition.clone());
        Ok(transition)
    }

    /// Check if workflow can proceed
    #[must_use]
    pub fn can_proceed(&self) -> bool {
        !self.current.is_terminal()
    }

    /// Check if workflow is complete
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.current == WorkflowState::Completed
    }

    /// Check if workflow failed
    #[must_use]
    pub fn is_failed(&self) -> bool {
        self.current == WorkflowState::Failed
    }

    /// Reset to Idle state (only from terminal states)
    pub fn reset(&mut self) -> Result<(), InvalidTransition> {
        if !self.current.is_terminal() {
            return Err(InvalidTransition {
                from: self.current,
                to: WorkflowState::Idle,
            });
        }

        self.current = WorkflowState::Idle;
        self.history.clear();
        Ok(())
    }
}

impl Default for StateMachine {
    fn default() -> Self {
        Self::new()
    }
}

/// Error for invalid state transitions
#[derive(Debug, Clone)]
pub struct InvalidTransition {
    pub from: WorkflowState,
    pub to: WorkflowState,
}

impl fmt::Display for InvalidTransition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Invalid transition: cannot go from {} to {}",
            self.from, self.to
        )
    }
}

impl std::error::Error for InvalidTransition {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_workflow_progression() {
        let mut sm = StateMachine::new();

        assert_eq!(sm.current(), WorkflowState::Idle);

        // Normal progression
        assert!(sm.transition(WorkflowState::Preprocessing).is_ok());
        assert!(sm.transition(WorkflowState::Planning).is_ok());
        assert!(sm.transition(WorkflowState::AwaitingApproval).is_ok());
        assert!(sm.transition(WorkflowState::Executing).is_ok());
        assert!(sm.transition(WorkflowState::Enriching).is_ok());
        assert!(sm.transition(WorkflowState::Completed).is_ok());

        assert!(sm.is_complete());
        assert!(!sm.can_proceed());
        assert_eq!(sm.history().len(), 6);
    }

    #[test]
    fn test_invalid_transitions() {
        let mut sm = StateMachine::new();

        // Can't skip to Executing from Idle
        assert!(sm.transition(WorkflowState::Executing).is_err());

        // Proceed normally to Planning
        sm.transition(WorkflowState::Preprocessing).unwrap();
        sm.transition(WorkflowState::Planning).unwrap();

        // Can't go backwards to Idle
        assert!(sm.transition(WorkflowState::Idle).is_err());
    }

    #[test]
    fn test_fail_from_any_state() {
        let mut sm = StateMachine::new();
        sm.transition(WorkflowState::Preprocessing).unwrap();

        sm.fail("Something went wrong");

        assert!(sm.is_failed());
        assert!(!sm.can_proceed());
    }

    #[test]
    fn test_cancel_from_non_terminal() {
        let mut sm = StateMachine::new();
        sm.transition(WorkflowState::Preprocessing).unwrap();

        assert!(sm.cancel("User cancelled").is_ok());
        assert_eq!(sm.current(), WorkflowState::Cancelled);
    }

    #[test]
    fn test_cannot_cancel_from_terminal() {
        let mut sm = StateMachine::new();
        sm.fail("Error");

        assert!(sm.cancel("Try to cancel").is_err());
    }

    #[test]
    fn test_reset() {
        let mut sm = StateMachine::new();
        sm.transition(WorkflowState::Preprocessing).unwrap();
        sm.fail("Error");

        assert!(sm.reset().is_ok());
        assert_eq!(sm.current(), WorkflowState::Idle);
        assert!(sm.history().is_empty());
    }

    #[test]
    fn test_plan_rejection_allows_replanning() {
        let mut sm = StateMachine::new();
        sm.transition(WorkflowState::Preprocessing).unwrap();
        sm.transition(WorkflowState::Planning).unwrap();
        sm.transition(WorkflowState::AwaitingApproval).unwrap();

        // Plan rejected - go back to Planning
        assert!(sm.transition(WorkflowState::Planning).is_ok());

        // Try again
        sm.transition(WorkflowState::AwaitingApproval).unwrap();
        sm.transition(WorkflowState::Executing).unwrap();

        assert_eq!(sm.current(), WorkflowState::Executing);
    }
}
