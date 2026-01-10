//! Workflow Engine for the Erold CLI
//!
//! Implements a 4-phase workflow with security gates:
//!
//! 1. **Preprocessing** - Fetch context, refresh stale knowledge
//! 2. **Planning** - Create plan, wait for human approval
//! 3. **Execution** - Run with gates, per-subtask preprocessing
//! 4. **Enrichment** - Save learnings, complete task
//!
//! # Design Patterns Used
//!
//! - **State Machine**: Workflow progresses through defined states
//! - **Builder**: Complex configuration with `WorkflowBuilder`
//! - **Repository**: Abstract data access via `WorkflowRepository` trait
//! - **Strategy**: Pluggable knowledge search strategies
//! - **Observer**: Event emission for monitoring
//!
//! # Security Features
//!
//! - Mandatory plan approval before any file modifications
//! - Read-before-edit enforcement
//! - No secrets in logs (tracing with skip)
//! - Input validation on all operations
//! - Timeout handling for all async operations
//!
//! # Example
//!
//! ```rust,ignore
//! use erold_workflow::{WorkflowEngine, WorkflowConfig};
//! use erold_workflow::repository::LiveWorkflowRepository;
//! use std::sync::Arc;
//!
//! // Create repository
//! let client = erold_api::EroldClient::new("https://api.erold.dev", "api_key");
//! let repository = Arc::new(LiveWorkflowRepository::new(client, "project_id"));
//!
//! // Build engine
//! let engine = WorkflowEngine::builder(repository)
//!     .config(WorkflowConfig::default())
//!     .with_logging()
//!     .build();
//!
//! // Run workflow
//! engine.start("Add authentication feature", "project_id").await?;
//! ```

mod config;
mod context;
mod error;
mod events;
mod repository;
mod state;
mod engine;
mod security;

pub mod phases;

// Re-exports
pub use config::{WorkflowConfig, WorkflowConfigBuilder};
pub use context::{PreprocessedContext, SubtaskContext, ExecutionContext, Learning, Mistake};
pub use error::{WorkflowError, Result};
pub use events::{WorkflowEvent, WorkflowEventHandler, LoggingEventHandler, NoOpEventHandler};
pub use repository::{WorkflowRepository, LiveWorkflowRepository};
pub use state::{WorkflowState, StateTransition, StateMachine, InvalidTransition};
pub use engine::{WorkflowEngine, WorkflowBuilder};
pub use security::{SecurityGate, FileTracker, InputValidator};
