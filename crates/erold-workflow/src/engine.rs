//! Workflow engine (Builder pattern)
//!
//! The main workflow orchestrator that coordinates all phases
//! and maintains the workflow state machine.

use std::sync::Arc;
use std::time::Instant;
use chrono::Utc;
use tokio::sync::RwLock;

use crate::config::WorkflowConfig;
use crate::context::{PreprocessedContext, ExecutionContext, SubtaskContext, Learning, Mistake};
use crate::error::{WorkflowError, Result};
use crate::events::{WorkflowEvent, WorkflowEventHandler, LoggingEventHandler};
use crate::repository::WorkflowRepository;
use crate::security::{SecurityGate, FileTracker};
use crate::state::{StateMachine, WorkflowState};
use erold_api::{Task, Knowledge, CreateTask, CreateSubtask, CreateKnowledge, TaskPriority, KnowledgeCategory};

/// Main workflow engine
///
/// Orchestrates the 4-phase workflow:
/// 1. Preprocessing - Fetch context, check knowledge TTL
/// 2. Planning - Create task/subtasks, wait for approval
/// 3. Execution - Run subtasks with security gates
/// 4. Enrichment - Save learnings, complete task
pub struct WorkflowEngine<R: WorkflowRepository> {
    /// Configuration (immutable after creation)
    config: WorkflowConfig,
    /// Repository for data access
    repository: Arc<R>,
    /// State machine
    state: RwLock<StateMachine>,
    /// Security gate
    security: RwLock<SecurityGate>,
    /// Event handlers
    event_handlers: Vec<Arc<dyn WorkflowEventHandler>>,
    /// Current execution context (if running)
    execution_context: RwLock<Option<ExecutionContext>>,
    /// Preprocessed context from phase 1
    preprocessed_context: RwLock<Option<PreprocessedContext>>,
    /// Start time for duration tracking
    start_time: RwLock<Option<Instant>>,
}

impl<R: WorkflowRepository> std::fmt::Debug for WorkflowEngine<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorkflowEngine")
            .field("config", &self.config)
            .field("event_handlers_count", &self.event_handlers.len())
            .finish_non_exhaustive()
    }
}

impl<R: WorkflowRepository> WorkflowEngine<R> {
    /// Create a new workflow engine with builder
    pub fn builder(repository: Arc<R>) -> WorkflowBuilder<R> {
        WorkflowBuilder::new(repository)
    }

    /// Get current workflow state
    pub async fn state(&self) -> WorkflowState {
        self.state.read().await.current()
    }

    /// Get security gate (for external tool integration)
    pub async fn security_gate(&self) -> tokio::sync::RwLockReadGuard<'_, SecurityGate> {
        self.security.read().await
    }

    /// Emit an event to all handlers
    fn emit_event(&self, event: WorkflowEvent) {
        for handler in &self.event_handlers {
            handler.handle(&event);
        }
    }

    /// Start the workflow with a task description
    ///
    /// This begins Phase 1: Preprocessing
    pub async fn start(&self, task_description: &str, project_id: &str) -> Result<()> {
        // Validate state
        {
            let state = self.state.read().await;
            if state.current() != WorkflowState::Idle {
                return Err(WorkflowError::AlreadyRunning);
            }
        }

        // Record start time
        *self.start_time.write().await = Some(Instant::now());

        // Emit started event
        self.emit_event(WorkflowEvent::Started {
            task_description: task_description.to_string(),
            timestamp: Utc::now(),
        });

        // Transition to Preprocessing
        {
            let mut state = self.state.write().await;
            let transition = state.transition(WorkflowState::Preprocessing)
                .map_err(|e| WorkflowError::InvalidState {
                    current_state: e.from.to_string(),
                    action: "start preprocessing".to_string(),
                })?;

            self.emit_event(WorkflowEvent::StateChanged {
                from: transition.from,
                to: transition.to,
                timestamp: Utc::now(),
            });
        }

        self.emit_event(WorkflowEvent::PreprocessingStarted {
            timestamp: Utc::now(),
        });

        // Run preprocessing
        self.run_preprocessing(task_description, project_id).await
    }

    /// Phase 1: Preprocessing
    async fn run_preprocessing(&self, task_description: &str, _project_id: &str) -> Result<()> {
        // Extract keywords from task description (simple extraction for now)
        let keywords = self.extract_keywords(task_description);

        // Search for relevant knowledge
        let all_knowledge = self.repository
            .search_knowledge(task_description)
            .await?;

        // Separate expired and valid knowledge
        let (valid, expired): (Vec<_>, Vec<_>) = all_knowledge
            .into_iter()
            .partition(|k| !k.is_expired());

        // Filter troubleshooting knowledge (past mistakes) from valid ones
        let (past_mistakes, relevant_knowledge): (Vec<_>, Vec<_>) = valid
            .into_iter()
            .partition(|k| k.category == KnowledgeCategory::Troubleshooting);

        // Emit knowledge fetched event
        self.emit_event(WorkflowEvent::KnowledgeFetched {
            total_count: relevant_knowledge.len() + past_mistakes.len() + expired.len(),
            relevant_count: relevant_knowledge.len(),
            expired_count: expired.len(),
            timestamp: Utc::now(),
        });

        // Store preprocessed context
        let context = PreprocessedContext {
            relevant_knowledge,
            relevant_tasks: Vec::new(), // Could fetch related tasks here
            past_mistakes,
            keywords,
        };

        *self.preprocessed_context.write().await = Some(context);

        // Transition to Planning
        {
            let mut state = self.state.write().await;
            let transition = state.transition(WorkflowState::Planning)
                .map_err(|e| WorkflowError::InvalidState {
                    current_state: e.from.to_string(),
                    action: "start planning".to_string(),
                })?;

            self.emit_event(WorkflowEvent::StateChanged {
                from: transition.from,
                to: transition.to,
                timestamp: Utc::now(),
            });
        }

        Ok(())
    }

    /// Phase 2: Create plan and wait for approval
    pub async fn create_plan(
        &self,
        title: &str,
        description: &str,
        project_id: &str,
        subtask_titles: Vec<String>,
    ) -> Result<Task> {
        // Validate state
        {
            let state = self.state.read().await;
            if state.current() != WorkflowState::Planning {
                return Err(WorkflowError::InvalidState {
                    current_state: state.current().to_string(),
                    action: "create plan".to_string(),
                });
            }
        }

        // Validate subtask count
        if subtask_titles.is_empty() {
            return Err(WorkflowError::EmptyPlan);
        }
        if subtask_titles.len() > self.config.max_subtasks() {
            return Err(WorkflowError::PlanTooLarge {
                count: subtask_titles.len(),
                max: self.config.max_subtasks(),
            });
        }

        // Create task via repository (without subtasks - API doesn't support inline creation)
        let create_task = CreateTask {
            title: title.to_string(),
            description: Some(description.to_string()),
            status: None,
            priority: Some(TaskPriority::Medium),
            assignee_type: Some("agent".to_string()),
            agent_id: None,
            agent_name: Some("erold".to_string()),
            subtasks: Vec::new(), // Empty - we'll track subtasks locally
            tags: Vec::new(),
        };

        let mut task = self.repository.create_task(project_id, &create_task).await?;

        // Build subtasks locally for tracking (API doesn't support inline subtask creation)
        task.subtasks = subtask_titles
            .iter()
            .enumerate()
            .map(|(i, title)| erold_api::Subtask {
                id: format!("subtask_{i}"),
                title: title.clone(),
                completed: false,
                order: i as i32,
                keywords: Vec::new(),
                knowledge_used: Vec::new(),
                notes: None,
                decisions: Vec::new(),
            })
            .collect();

        // Emit plan created event
        self.emit_event(WorkflowEvent::PlanCreated {
            task_id: task.id.clone(),
            subtask_count: task.subtasks.len(),
            timestamp: Utc::now(),
        });

        // Initialize execution context
        let exec_ctx = ExecutionContext::new(
            task.id.clone(),
            task.title.clone(),
            project_id,
            task.subtasks.len(),
        );
        *self.execution_context.write().await = Some(exec_ctx);

        // Transition to AwaitingApproval if required
        if self.config.require_approval() {
            let mut state = self.state.write().await;
            let transition = state.transition(WorkflowState::AwaitingApproval)
                .map_err(|e| WorkflowError::InvalidState {
                    current_state: e.from.to_string(),
                    action: "await approval".to_string(),
                })?;

            self.emit_event(WorkflowEvent::StateChanged {
                from: transition.from,
                to: transition.to,
                timestamp: Utc::now(),
            });

            self.emit_event(WorkflowEvent::AwaitingApproval {
                task_id: task.id.clone(),
                timestamp: Utc::now(),
            });
        }

        Ok(task)
    }

    /// Approve the plan and begin execution
    pub async fn approve_plan(&self) -> Result<()> {
        // Validate state
        {
            let state = self.state.read().await;
            if state.current() != WorkflowState::AwaitingApproval {
                return Err(WorkflowError::InvalidState {
                    current_state: state.current().to_string(),
                    action: "approve plan".to_string(),
                });
            }
        }

        // Mark plan approved in security gate
        self.security.write().await.approve_plan();

        // Mark approved in execution context
        if let Some(ctx) = self.execution_context.write().await.as_mut() {
            ctx.approve_plan();
        }

        let task_id = self.execution_context.read().await
            .as_ref()
            .map(|c| c.task_id.clone())
            .unwrap_or_default();

        self.emit_event(WorkflowEvent::PlanApproved {
            task_id: task_id.clone(),
            timestamp: Utc::now(),
        });

        // Transition to Executing
        {
            let mut state = self.state.write().await;
            let transition = state.transition(WorkflowState::Executing)
                .map_err(|e| WorkflowError::InvalidState {
                    current_state: e.from.to_string(),
                    action: "start executing".to_string(),
                })?;

            self.emit_event(WorkflowEvent::StateChanged {
                from: transition.from,
                to: transition.to,
                timestamp: Utc::now(),
            });
        }

        Ok(())
    }

    /// Reject the plan
    pub async fn reject_plan(&self, reason: Option<String>) -> Result<()> {
        // Validate state
        {
            let state = self.state.read().await;
            if state.current() != WorkflowState::AwaitingApproval {
                return Err(WorkflowError::InvalidState {
                    current_state: state.current().to_string(),
                    action: "reject plan".to_string(),
                });
            }
        }

        let task_id = self.execution_context.read().await
            .as_ref()
            .map(|c| c.task_id.clone())
            .unwrap_or_default();

        self.emit_event(WorkflowEvent::PlanRejected {
            task_id: task_id.clone(),
            reason: reason.clone(),
            timestamp: Utc::now(),
        });

        // Transition back to Planning for revision
        {
            let mut state = self.state.write().await;
            let transition = state.transition(WorkflowState::Planning)
                .map_err(|e| WorkflowError::InvalidState {
                    current_state: e.from.to_string(),
                    action: "revise plan".to_string(),
                })?;

            self.emit_event(WorkflowEvent::StateChanged {
                from: transition.from,
                to: transition.to,
                timestamp: Utc::now(),
            });
        }

        Err(WorkflowError::PlanRejected { reason })
    }

    /// Start executing a subtask
    pub async fn start_subtask(&self, index: usize, title: &str) -> Result<SubtaskContext> {
        // Validate state
        {
            let state = self.state.read().await;
            if state.current() != WorkflowState::Executing {
                return Err(WorkflowError::InvalidState {
                    current_state: state.current().to_string(),
                    action: "start subtask".to_string(),
                });
            }
        }

        // Ensure plan is approved
        {
            let ctx = self.execution_context.read().await;
            if let Some(ctx) = ctx.as_ref() {
                if !ctx.is_plan_approved() {
                    return Err(WorkflowError::NoPlanApproved);
                }
            }
        }

        self.emit_event(WorkflowEvent::SubtaskStarted {
            index,
            title: title.to_string(),
            timestamp: Utc::now(),
        });

        // Create subtask context with per-subtask knowledge
        let mut subtask_ctx = SubtaskContext::new(index, title);

        // Fetch subtask-specific knowledge
        if let Some(preprocessed) = self.preprocessed_context.read().await.as_ref() {
            // Filter relevant knowledge for this subtask
            subtask_ctx.relevant_knowledge = preprocessed.relevant_knowledge
                .iter()
                .filter(|k| self.is_relevant_to_subtask(k, title))
                .cloned()
                .collect();

            subtask_ctx.past_mistakes = preprocessed.past_mistakes
                .iter()
                .filter(|k| self.is_relevant_to_subtask(k, title))
                .cloned()
                .collect();
        }

        Ok(subtask_ctx)
    }

    /// Complete a subtask
    pub async fn complete_subtask(&self, index: usize, title: &str) -> Result<()> {
        self.emit_event(WorkflowEvent::SubtaskCompleted {
            index,
            title: title.to_string(),
            timestamp: Utc::now(),
        });

        // Update context
        if let Some(ctx) = self.execution_context.write().await.as_mut() {
            ctx.advance_subtask();
        }

        Ok(())
    }

    /// Record a file read
    pub async fn on_file_read(&self, path: &str) -> Result<()> {
        self.security.write().await.on_file_read(path)?;

        if let Some(ctx) = self.execution_context.write().await.as_mut() {
            ctx.record_file_read(path.into());
        }

        self.emit_event(WorkflowEvent::FileRead {
            timestamp: Utc::now(),
        });

        Ok(())
    }

    /// Check if file edit is allowed
    pub async fn check_file_edit(&self, path: &str) -> Result<()> {
        self.security.read().await.check_can_modify()?;
        self.security.read().await.file_tracker().check_edit(path)?;
        Ok(())
    }

    /// Record a file modification
    pub async fn on_file_modified(&self, path: &str) -> Result<()> {
        self.security.write().await.on_file_edit(path)?;

        if let Some(ctx) = self.execution_context.write().await.as_mut() {
            ctx.record_file_modified(path.into());
        }

        self.emit_event(WorkflowEvent::FileModified {
            timestamp: Utc::now(),
        });

        Ok(())
    }

    /// Record a learning
    pub async fn record_learning(&self, learning: Learning) -> Result<()> {
        self.emit_event(WorkflowEvent::LearningSaved {
            title: learning.title.clone(),
            category: learning.category.clone(),
            timestamp: Utc::now(),
        });

        if let Some(ctx) = self.execution_context.write().await.as_mut() {
            ctx.add_learning(learning);
        }
        Ok(())
    }

    /// Record a mistake
    pub async fn record_mistake(&self, mistake: Mistake) -> Result<()> {
        self.emit_event(WorkflowEvent::MistakeRecorded {
            title: mistake.what_failed.clone(),
            timestamp: Utc::now(),
        });

        if let Some(ctx) = self.execution_context.write().await.as_mut() {
            ctx.add_mistake(mistake);
        }
        Ok(())
    }

    /// Begin enrichment phase
    pub async fn begin_enrichment(&self) -> Result<()> {
        // Validate state
        {
            let state = self.state.read().await;
            if state.current() != WorkflowState::Executing {
                return Err(WorkflowError::InvalidState {
                    current_state: state.current().to_string(),
                    action: "begin enrichment".to_string(),
                });
            }
        }

        // Transition to Enriching
        {
            let mut state = self.state.write().await;
            let transition = state.transition(WorkflowState::Enriching)
                .map_err(|e| WorkflowError::InvalidState {
                    current_state: e.from.to_string(),
                    action: "begin enrichment".to_string(),
                })?;

            self.emit_event(WorkflowEvent::StateChanged {
                from: transition.from,
                to: transition.to,
                timestamp: Utc::now(),
            });
        }

        // Run enrichment
        self.run_enrichment().await
    }

    /// Phase 4: Enrichment
    async fn run_enrichment(&self) -> Result<()> {
        if !self.config.auto_enrich() {
            return self.complete_workflow().await;
        }

        let ctx_guard = self.execution_context.read().await;
        let ctx = ctx_guard.as_ref().ok_or_else(|| WorkflowError::Internal {
            message: "No execution context".to_string(),
        })?;

        // Save learnings to knowledge base
        for learning in &ctx.learnings {
            let category = match learning.category.as_str() {
                "architecture" => KnowledgeCategory::Architecture,
                "api" => KnowledgeCategory::Api,
                "deployment" => KnowledgeCategory::Deployment,
                "testing" => KnowledgeCategory::Testing,
                "security" => KnowledgeCategory::Security,
                "performance" => KnowledgeCategory::Performance,
                "workflow" => KnowledgeCategory::Workflow,
                "conventions" => KnowledgeCategory::Conventions,
                "troubleshooting" => KnowledgeCategory::Troubleshooting,
                _ => KnowledgeCategory::Other,
            };

            let create = CreateKnowledge {
                title: learning.title.clone(),
                content: learning.content.clone(),
                category,
                tags: learning.tags.clone(),
                project_id: Some(ctx.project_id.clone()),
                source: Some("erold-agent".to_string()),
                source_url: None,
                source_type: None,
                ttl_days: None, // Learnings don't expire
                auto_refresh: false,
            };

            if let Err(e) = self.repository.save_knowledge(&create).await {
                tracing::warn!("Failed to save learning: {}", e);
            }
        }

        // Save mistakes as troubleshooting knowledge
        for mistake in &ctx.mistakes {
            let content = format!(
                "## What Failed\n{}\n\n## Problem\n{}\n\n## Wrong Approach\n{}\n\n## Why It Failed\n{}\n\n## Correct Approach\n{}",
                mistake.what_failed,
                mistake.problem,
                mistake.wrong_approach,
                mistake.why_failed,
                mistake.correct_approach
            );

            let create = CreateKnowledge {
                title: format!("Mistake: {}", mistake.what_failed),
                content,
                category: KnowledgeCategory::Troubleshooting,
                tags: mistake.tags.clone(),
                project_id: Some(ctx.project_id.clone()),
                source: Some("erold-agent".to_string()),
                source_url: None,
                source_type: None,
                ttl_days: None, // Mistakes don't expire
                auto_refresh: false,
            };

            if let Err(e) = self.repository.save_knowledge(&create).await {
                tracing::warn!("Failed to save mistake: {}", e);
            }
        }

        drop(ctx_guard);
        self.complete_workflow().await
    }

    /// Complete the workflow
    async fn complete_workflow(&self) -> Result<()> {
        let duration = self.start_time.read().await
            .map(|t| t.elapsed().as_secs())
            .unwrap_or(0);

        let (task_id, learnings_count) = {
            let ctx = self.execution_context.read().await;
            let ctx = ctx.as_ref().ok_or_else(|| WorkflowError::Internal {
                message: "No execution context".to_string(),
            })?;
            (ctx.task_id.clone(), ctx.learnings.len())
        };

        // Complete task in repository
        self.repository.complete_task(&task_id, Some("Workflow completed successfully")).await?;

        // Transition to Completed
        {
            let mut state = self.state.write().await;
            let transition = state.transition(WorkflowState::Completed)
                .map_err(|e| WorkflowError::InvalidState {
                    current_state: e.from.to_string(),
                    action: "complete workflow".to_string(),
                })?;

            self.emit_event(WorkflowEvent::StateChanged {
                from: transition.from,
                to: transition.to,
                timestamp: Utc::now(),
            });
        }

        self.emit_event(WorkflowEvent::Completed {
            task_id,
            duration_secs: duration,
            learnings_count,
            timestamp: Utc::now(),
        });

        Ok(())
    }

    /// Fail the workflow
    pub async fn fail(&self, error: &str, recoverable: bool) -> Result<()> {
        {
            let mut state = self.state.write().await;
            let transition = state.fail(error);

            self.emit_event(WorkflowEvent::StateChanged {
                from: transition.from,
                to: transition.to,
                timestamp: Utc::now(),
            });
        }

        self.emit_event(WorkflowEvent::Failed {
            error: error.to_string(),
            recoverable,
            timestamp: Utc::now(),
        });

        Ok(())
    }

    /// Cancel the workflow
    pub async fn cancel(&self, reason: &str) -> Result<()> {
        let mut state = self.state.write().await;
        state.cancel(reason)
            .map_err(|e| WorkflowError::InvalidState {
                current_state: e.from.to_string(),
                action: "cancel workflow".to_string(),
            })?;
        Ok(())
    }

    /// Reset the workflow to Idle
    pub async fn reset(&self) -> Result<()> {
        {
            let mut state = self.state.write().await;
            state.reset()
                .map_err(|e| WorkflowError::InvalidState {
                    current_state: e.from.to_string(),
                    action: "reset workflow".to_string(),
                })?;
        }

        // Clear contexts
        *self.execution_context.write().await = None;
        *self.preprocessed_context.write().await = None;
        *self.start_time.write().await = None;

        // Reset security gate
        self.security.write().await.reset();

        Ok(())
    }

    /// Extract keywords from text (simple implementation)
    fn extract_keywords(&self, text: &str) -> Vec<String> {
        // Simple keyword extraction - split on whitespace and filter
        text.split_whitespace()
            .filter(|w| w.len() > 3)
            .filter(|w| !STOP_WORDS.contains(&w.to_lowercase().as_str()))
            .map(|w| w.to_lowercase())
            .collect()
    }

    /// Check if knowledge is relevant to a subtask
    fn is_relevant_to_subtask(&self, knowledge: &Knowledge, subtask_title: &str) -> bool {
        let title_lower = subtask_title.to_lowercase();
        let knowledge_title = knowledge.title.to_lowercase();

        // Check title overlap
        if title_lower.split_whitespace()
            .any(|word| knowledge_title.contains(word))
        {
            return true;
        }

        // Check tag overlap
        for tag in &knowledge.tags {
            if title_lower.contains(&tag.to_lowercase()) {
                return true;
            }
        }

        false
    }
}

/// Common stop words to filter from keyword extraction
const STOP_WORDS: &[&str] = &[
    "the", "and", "for", "are", "but", "not", "you", "all", "can", "had",
    "her", "was", "one", "our", "out", "has", "have", "been", "from", "this",
    "that", "with", "they", "will", "would", "there", "their", "what", "about",
    "which", "when", "make", "like", "time", "just", "know", "take", "into",
];

/// Builder for WorkflowEngine
pub struct WorkflowBuilder<R: WorkflowRepository> {
    repository: Arc<R>,
    config: WorkflowConfig,
    event_handlers: Vec<Arc<dyn WorkflowEventHandler>>,
    file_tracker: Option<FileTracker>,
}

impl<R: WorkflowRepository> WorkflowBuilder<R> {
    /// Create a new builder
    pub fn new(repository: Arc<R>) -> Self {
        Self {
            repository,
            config: WorkflowConfig::default(),
            event_handlers: Vec::new(),
            file_tracker: None,
        }
    }

    /// Set configuration
    #[must_use]
    pub fn config(mut self, config: WorkflowConfig) -> Self {
        self.config = config;
        self
    }

    /// Add an event handler
    #[must_use]
    pub fn add_event_handler(mut self, handler: Arc<dyn WorkflowEventHandler>) -> Self {
        self.event_handlers.push(handler);
        self
    }

    /// Set custom file tracker
    #[must_use]
    pub fn file_tracker(mut self, tracker: FileTracker) -> Self {
        self.file_tracker = Some(tracker);
        self
    }

    /// Add logging event handler
    #[must_use]
    pub fn with_logging(self) -> Self {
        self.add_event_handler(Arc::new(LoggingEventHandler))
    }

    /// Build the workflow engine
    #[must_use]
    pub fn build(self) -> WorkflowEngine<R> {
        let security_gate = match self.file_tracker {
            Some(tracker) => SecurityGate::new().with_file_tracker(tracker),
            None => SecurityGate::new(),
        };

        WorkflowEngine {
            config: self.config,
            repository: self.repository,
            state: RwLock::new(StateMachine::new()),
            security: RwLock::new(security_gate),
            event_handlers: self.event_handlers,
            execution_context: RwLock::new(None),
            preprocessed_context: RwLock::new(None),
            start_time: RwLock::new(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::testing::InMemoryRepository;

    #[tokio::test]
    async fn test_workflow_state_progression() {
        let repo = Arc::new(InMemoryRepository::new());
        let engine = WorkflowEngine::builder(repo).build();

        assert_eq!(engine.state().await, WorkflowState::Idle);

        // Start workflow
        engine.start("Test task", "project_1").await.unwrap();
        assert_eq!(engine.state().await, WorkflowState::Planning);
    }

    #[tokio::test]
    async fn test_cannot_start_while_running() {
        let repo = Arc::new(InMemoryRepository::new());
        let engine = WorkflowEngine::builder(repo).build();

        engine.start("Test task", "project_1").await.unwrap();

        // Try to start again
        let result = engine.start("Another task", "project_1").await;
        assert!(matches!(result, Err(WorkflowError::AlreadyRunning)));
    }

    #[tokio::test]
    async fn test_plan_creation() {
        let repo = Arc::new(InMemoryRepository::new());
        let config = WorkflowConfig::builder()
            .max_subtasks(5)
            .build();
        let engine = WorkflowEngine::builder(repo)
            .config(config)
            .build();

        engine.start("Test task", "project_1").await.unwrap();

        let subtask_titles = vec![
            "Subtask 1".to_string(),
            "Subtask 2".to_string(),
        ];

        let task = engine.create_plan("Test", "Description", "project_1", subtask_titles).await.unwrap();
        assert_eq!(task.subtasks.len(), 2);
        assert_eq!(engine.state().await, WorkflowState::AwaitingApproval);
    }

    #[tokio::test]
    async fn test_empty_plan_rejected() {
        let repo = Arc::new(InMemoryRepository::new());
        let engine = WorkflowEngine::builder(repo).build();

        engine.start("Test task", "project_1").await.unwrap();

        let result = engine.create_plan("Test", "Description", "project_1", vec![]).await;
        assert!(matches!(result, Err(WorkflowError::EmptyPlan)));
    }

    #[tokio::test]
    async fn test_plan_too_large() {
        let repo = Arc::new(InMemoryRepository::new());
        let config = WorkflowConfig::builder()
            .max_subtasks(2)
            .build();
        let engine = WorkflowEngine::builder(repo)
            .config(config)
            .build();

        engine.start("Test task", "project_1").await.unwrap();

        let subtask_titles: Vec<_> = (0..5)
            .map(|i| format!("Subtask {}", i))
            .collect();

        let result = engine.create_plan("Test", "Description", "project_1", subtask_titles).await;
        assert!(matches!(result, Err(WorkflowError::PlanTooLarge { .. })));
    }
}
