# Erold Workflow System - Architecture & Specification

> **Status**: Final Design
> **Last Updated**: 2025-12-25

---

## Vision

Erold is the **persistent brain** for AI agents - providing full context management and project management.

**Two sides of the same system:**

| Erold Platform (Humans) | Erold CLI (Agent) |
|-------------------------|-------------------|
| See all agent actions | Know current tasks |
| See task progress | Have relevant knowledge |
| See knowledge created | Know project setup |
| Full audit trail | Never start blind |

**Core Principle**: Erold is the single source of truth. Not agent memory.

---

## Design Principles

### 1. Extensibility First
- New workflows can be added without modifying core
- Prompts and schemas are configuration, not code
- Hooks can be composed and chained

### 2. Clean Architecture
- Domain logic separate from infrastructure
- Dependency inversion - core doesn't depend on Erold API directly
- Testable in isolation

### 3. Single Responsibility
- Each component does one thing well
- Workflows orchestrate, they don't implement
- Hooks are atomic operations

### 4. Open/Closed Principle
- Open for extension (new workflows, hooks)
- Closed for modification (core engine stable)

---

## Design Patterns Used

### 1. Strategy Pattern - Workflows

Different workflows implement the same interface but with different behavior.

```rust
/// Core workflow trait - all workflows implement this
pub trait Workflow: Send + Sync {
    /// Unique identifier for this workflow
    fn id(&self) -> &str;

    /// Human-readable name
    fn name(&self) -> &str;

    /// Analyze user input and determine task/knowledge needs
    async fn analyze_input(&self, ctx: &WorkflowContext) -> Result<TaskAnalysis>;

    /// Extract outcomes after work is complete
    async fn extract_outcomes(&self, ctx: &WorkflowContext) -> Result<WorkOutcome>;

    /// Get workflow-specific configuration
    fn config(&self) -> &WorkflowConfig;
}

// Different strategies
pub struct CodingWorkflow { ... }
pub struct ResearchWorkflow { ... }
pub struct DocumentationWorkflow { ... }
pub struct BugfixWorkflow { ... }
pub struct CustomWorkflow { config: WorkflowConfig }
```

### 2. Chain of Responsibility - Hooks

Hooks can be chained and composed. Each hook can pass to the next or stop the chain.

```rust
/// Hook trait - atomic operations in the workflow
#[async_trait]
pub trait Hook: Send + Sync {
    /// Execute this hook
    async fn execute(&self, ctx: &mut HookContext) -> Result<HookResult>;

    /// Hook priority (lower = runs first)
    fn priority(&self) -> i32 { 0 }

    /// Can this hook be skipped?
    fn skippable(&self) -> bool { false }
}

pub enum HookResult {
    Continue,           // Pass to next hook
    Stop,               // Stop chain, but not an error
    Error(HookError),   // Stop chain with error
}

/// Hook chain - executes hooks in order
pub struct HookChain {
    hooks: Vec<Box<dyn Hook>>,
}

impl HookChain {
    pub fn add(&mut self, hook: impl Hook + 'static) -> &mut Self;
    pub async fn execute(&self, ctx: &mut HookContext) -> Result<()>;
}
```

### 3. Observer Pattern - Events

Components can subscribe to workflow events without tight coupling.

```rust
/// Events emitted during workflow execution
pub enum WorkflowEvent {
    SessionStarted { context: Context },
    TaskStarted { task_id: String },
    FileRead { path: String },
    FileEdited { path: String },
    ToolCalled { tool: String, args: Value },
    TaskCompleted { task_id: String, summary: String },
    KnowledgeSaved { article_id: String },
    WorkflowError { error: WorkflowError },
}

/// Observer trait
#[async_trait]
pub trait WorkflowObserver: Send + Sync {
    async fn on_event(&self, event: &WorkflowEvent);
}

/// Observable workflow engine
pub struct WorkflowEngine {
    observers: Vec<Arc<dyn WorkflowObserver>>,
}
```

### 4. Repository Pattern - Data Access

Abstract Erold API access for testability.

```rust
/// Repository trait for Erold operations
#[async_trait]
pub trait EroldRepository: Send + Sync {
    // Context
    async fn get_context(&self, project_id: Option<&str>) -> Result<Context>;

    // Tasks
    async fn get_task(&self, task_id: &str) -> Result<Option<Task>>;
    async fn create_task(&self, task: &CreateTask) -> Result<Task>;
    async fn start_task(&self, task_id: &str) -> Result<Task>;
    async fn complete_task(&self, task_id: &str, summary: &str) -> Result<Task>;
    async fn block_task(&self, task_id: &str, reason: &str) -> Result<Task>;

    // Knowledge
    async fn search_knowledge(&self, query: &str) -> Result<Vec<Knowledge>>;
    async fn save_knowledge(&self, knowledge: &CreateKnowledge) -> Result<Knowledge>;

    // Activity
    async fn log_activity(&self, activity: &Activity) -> Result<()>;
}

/// Real implementation using Erold API
pub struct EroldApiRepository {
    client: EroldClient,
}

/// Mock implementation for testing
pub struct MockEroldRepository {
    tasks: HashMap<String, Task>,
    knowledge: Vec<Knowledge>,
}
```

### 5. Factory Pattern - Workflow Creation

Create workflows based on configuration or detection.

```rust
/// Factory for creating workflows
pub struct WorkflowFactory {
    registry: HashMap<String, Box<dyn Fn() -> Box<dyn Workflow>>>,
}

impl WorkflowFactory {
    /// Register a workflow type
    pub fn register<W: Workflow + Default + 'static>(&mut self, id: &str);

    /// Create workflow by ID
    pub fn create(&self, id: &str) -> Option<Box<dyn Workflow>>;

    /// Create workflow from config file
    pub fn from_config(&self, config: &WorkflowConfig) -> Box<dyn Workflow>;
}
```

### 6. Template Method - Workflow Execution

Define the skeleton, let subclasses fill in steps.

```rust
/// Base workflow with template method
pub struct BaseWorkflow {
    config: WorkflowConfig,
    hooks: HookChain,
}

impl BaseWorkflow {
    /// Template method - defines the workflow skeleton
    pub async fn execute(&self, ctx: &mut WorkflowContext) -> Result<()> {
        // 1. Pre-processing (can be overridden)
        self.pre_process(ctx).await?;

        // 2. Analyze input (abstract - must be implemented)
        let analysis = self.analyze_input(ctx).await?;

        // 3. Validate (can be overridden)
        self.validate(&analysis, ctx).await?;

        // 4. Execute hooks
        self.hooks.execute(ctx).await?;

        // 5. Post-processing (can be overridden)
        self.post_process(ctx).await?;

        Ok(())
    }

    // Hook points for customization
    async fn pre_process(&self, ctx: &mut WorkflowContext) -> Result<()> { Ok(()) }
    async fn validate(&self, analysis: &TaskAnalysis, ctx: &WorkflowContext) -> Result<()>;
    async fn post_process(&self, ctx: &mut WorkflowContext) -> Result<()> { Ok(()) }
}
```

### 7. Builder Pattern - Configuration

Fluent API for building workflows and configs.

```rust
/// Workflow builder
pub struct WorkflowBuilder {
    id: String,
    name: String,
    config: WorkflowConfig,
    hooks: Vec<Box<dyn Hook>>,
    prompt_templates: PromptTemplates,
}

impl WorkflowBuilder {
    pub fn new(id: &str) -> Self;

    pub fn name(mut self, name: &str) -> Self;
    pub fn enforce_read_before_edit(mut self, enforce: bool) -> Self;
    pub fn require_planning(mut self, require: bool) -> Self;
    pub fn planning_threshold(mut self, threshold: u8) -> Self;
    pub fn auto_test(mut self, auto: bool) -> Self;

    pub fn add_hook(mut self, hook: impl Hook + 'static) -> Self;
    pub fn input_prompt(mut self, template: &str) -> Self;
    pub fn outcome_prompt(mut self, template: &str) -> Self;

    pub fn build(self) -> Result<Box<dyn Workflow>>;
}

// Usage:
let workflow = WorkflowBuilder::new("coding")
    .name("Standard Coding Workflow")
    .enforce_read_before_edit(true)
    .require_planning(true)
    .planning_threshold(5)
    .auto_test(true)
    .add_hook(ReadBeforeEditHook::new())
    .add_hook(ActivityLoggingHook::new())
    .input_prompt(include_str!("prompts/coding_input.txt"))
    .outcome_prompt(include_str!("prompts/coding_outcome.txt"))
    .build()?;
```

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              WORKFLOW ENGINE                                 │
│                                                                             │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐  │
│  │   Coding    │    │  Research   │    │    Docs     │    │   Custom    │  │
│  │  Workflow   │    │  Workflow   │    │  Workflow   │    │  Workflow   │  │
│  └──────┬──────┘    └──────┬──────┘    └──────┬──────┘    └──────┬──────┘  │
│         │                  │                  │                  │          │
│         └──────────────────┴──────────────────┴──────────────────┘          │
│                                    │                                        │
│                                    ▼                                        │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                         BASE WORKFLOW                                 │  │
│  │                                                                       │  │
│  │   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌────────────┐  │  │
│  │   │ Pre-Process │─►│  Analyze    │─►│  Validate   │─►│ Post-Proc  │  │  │
│  │   └─────────────┘  └─────────────┘  └─────────────┘  └────────────┘  │  │
│  │                           │                                          │  │
│  └───────────────────────────┼──────────────────────────────────────────┘  │
│                              ▼                                              │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                          HOOK CHAIN                                   │  │
│  │                                                                       │  │
│  │   ┌──────────┐   ┌──────────┐   ┌──────────┐   ┌──────────┐         │  │
│  │   │ Context  │──►│ Activity │──►│ Read     │──►│ Validate │──► ...  │  │
│  │   │ Loader   │   │ Logger   │   │ Tracker  │   │ Task     │         │  │
│  │   └──────────┘   └──────────┘   └──────────┘   └──────────┘         │  │
│  │                                                                       │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                              │                                              │
│                              ▼                                              │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                       MODEL ADAPTER                                   │  │
│  │                                                                       │  │
│  │   ┌───────────────┐    ┌───────────────┐    ┌───────────────┐       │  │
│  │   │ Prompt        │    │ Structured    │    │ Response      │       │  │
│  │   │ Templates     │───►│ Output        │───►│ Validation    │       │  │
│  │   └───────────────┘    └───────────────┘    └───────────────┘       │  │
│  │                                                                       │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                              │                                              │
│                              ▼                                              │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                      EROLD REPOSITORY                                 │  │
│  │                                                                       │  │
│  │   ┌────────┐  ┌────────┐  ┌────────┐  ┌────────┐  ┌────────┐        │  │
│  │   │Context │  │ Tasks  │  │Knowledge│  │ Vault  │  │Activity│        │  │
│  │   └────────┘  └────────┘  └────────┘  └────────┘  └────────┘        │  │
│  │                                                                       │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                              │                                              │
└──────────────────────────────┼──────────────────────────────────────────────┘
                               ▼
                        ┌─────────────┐
                        │  EROLD API  │
                        └─────────────┘
```

---

## Module Structure

```
erold-tools/src/
│
├── lib.rs                      # Public exports
│
├── workflow/
│   ├── mod.rs                  # Module exports
│   │
│   ├── traits/
│   │   ├── mod.rs
│   │   ├── workflow.rs         # Workflow trait
│   │   ├── hook.rs             # Hook trait
│   │   └── observer.rs         # Observer trait
│   │
│   ├── engine/
│   │   ├── mod.rs
│   │   ├── engine.rs           # WorkflowEngine
│   │   ├── context.rs          # WorkflowContext
│   │   └── chain.rs            # HookChain
│   │
│   ├── workflows/
│   │   ├── mod.rs
│   │   ├── base.rs             # BaseWorkflow
│   │   ├── coding.rs           # CodingWorkflow
│   │   ├── research.rs         # ResearchWorkflow
│   │   ├── documentation.rs    # DocumentationWorkflow
│   │   ├── bugfix.rs           # BugfixWorkflow
│   │   └── custom.rs           # CustomWorkflow (from config)
│   │
│   ├── hooks/
│   │   ├── mod.rs
│   │   ├── context_loader.rs   # Load context on session start
│   │   ├── activity_logger.rs  # Log all actions
│   │   ├── read_tracker.rs     # Track file reads
│   │   ├── edit_guard.rs       # Enforce read-before-edit
│   │   ├── task_validator.rs   # Validate task exists
│   │   └── knowledge_search.rs # Search knowledge base
│   │
│   ├── factory.rs              # WorkflowFactory
│   └── builder.rs              # WorkflowBuilder
│
├── model/
│   ├── mod.rs
│   ├── adapter.rs              # ModelAdapter trait
│   ├── prompts.rs              # PromptTemplates
│   └── schemas.rs              # TaskAnalysis, WorkOutcome
│
├── repository/
│   ├── mod.rs
│   ├── traits.rs               # EroldRepository trait
│   ├── api.rs                  # EroldApiRepository
│   └── mock.rs                 # MockEroldRepository (testing)
│
├── config/
│   ├── mod.rs
│   ├── workflow_config.rs      # WorkflowConfig
│   └── loader.rs               # Load from TOML/YAML
│
└── prompts/
    ├── coding_input.txt
    ├── coding_outcome.txt
    ├── research_input.txt
    ├── research_outcome.txt
    └── ...
```

---

## Configuration Format

Workflows can be defined in configuration files:

```toml
# .erold/workflows/coding.toml

[workflow]
id = "coding"
name = "Standard Coding Workflow"
description = "Full enforcement for coding tasks"

[enforcement]
read_before_edit = true
require_planning = true
planning_threshold = 5
auto_test = true
block_commit_on_test_failure = true

[hooks]
# Hooks execute in order
enabled = [
    "context_loader",
    "activity_logger",
    "read_tracker",
    "edit_guard",
    "task_validator",
    "knowledge_search",
]

[prompts]
input_analysis = """
You are analyzing a user request to determine task and knowledge needs.

Active tasks: {my_tasks}
Projects: {projects}
User message: {user_message}

Return JSON:
{
  "is_actionable_task": boolean,
  "existing_task_id": string | null,
  "new_task": { "title": string, "project_id": string } | null,
  "knowledge_domains": string[]
}
"""

work_outcome = """
Based on the work just completed, extract outcomes.

Return JSON:
{
  "task_complete": boolean,
  "completion_summary": string | null,
  "learnings": [{ "title": string, "category": string, "content": string }],
  "blocked": { "is_blocked": boolean, "reason": string | null }
}
"""
```

---

## Schemas

### TaskAnalysis

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskAnalysis {
    /// Is this something that requires work?
    pub is_actionable_task: bool,

    /// If continuing existing task, which one?
    #[serde(skip_serializing_if = "Option::is_none")]
    pub existing_task_id: Option<String>,

    /// If new task needed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_task: Option<NewTask>,

    /// Domains to search in knowledge base
    #[serde(default)]
    pub knowledge_domains: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct NewTask {
    pub title: String,
    pub project_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<TaskPriority>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}
```

### WorkOutcome

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkOutcome {
    /// Is the current task complete?
    pub task_complete: bool,

    /// Summary of what was accomplished
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_summary: Option<String>,

    /// New knowledge to save
    #[serde(default)]
    pub learnings: Vec<Learning>,

    /// If work is blocked
    #[serde(default)]
    pub blocked: BlockedStatus,

    /// Progress update (0-100)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress_percent: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Learning {
    pub title: String,
    pub category: KnowledgeCategory,
    pub content: String,
    #[serde(default = "default_scope")]
    pub scope: KnowledgeScope,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct BlockedStatus {
    pub is_blocked: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}
```

---

## Event System

```rust
/// All events in the workflow system
#[derive(Debug, Clone)]
pub enum WorkflowEvent {
    // Session events
    SessionStarted { context: Context },
    SessionEnded { duration_secs: u64 },

    // Task events
    TaskAnalyzed { analysis: TaskAnalysis },
    TaskCreated { task: Task },
    TaskStarted { task_id: String },
    TaskProgressed { task_id: String, percent: u8 },
    TaskCompleted { task_id: String, summary: String },
    TaskBlocked { task_id: String, reason: String },

    // File events
    FileRead { path: String },
    FileEditAttempted { path: String, allowed: bool },
    FileEdited { path: String },

    // Knowledge events
    KnowledgeSearched { query: String, results: usize },
    KnowledgeSaved { article: Knowledge },

    // Tool events
    ToolCalled { tool: String, args: Value },
    ToolCompleted { tool: String, success: bool },

    // Workflow events
    WorkflowSwitched { from: String, to: String },
    HookExecuted { hook: String, result: HookResult },

    // Error events
    ValidationFailed { error: String },
    WorkflowError { error: WorkflowError },
}
```

---

## Usage Examples

### Basic Usage

```rust
use erold_tools::workflow::{WorkflowEngine, CodingWorkflow};
use erold_tools::repository::EroldApiRepository;

// Create repository
let repository = EroldApiRepository::new(client);

// Create engine with default coding workflow
let engine = WorkflowEngine::builder()
    .repository(repository)
    .default_workflow(CodingWorkflow::new())
    .build()?;

// Start session - automatically loads context
engine.start_session().await?;

// Process user input
let analysis = engine.analyze_input("Implement user authentication").await?;

// After work is done
let outcome = engine.extract_outcomes().await?;
```

### Custom Workflow

```rust
use erold_tools::workflow::{WorkflowBuilder, hooks::*};

let my_workflow = WorkflowBuilder::new("my-workflow")
    .name("My Custom Workflow")
    .enforce_read_before_edit(true)
    .require_planning(false)
    .add_hook(ContextLoaderHook::new())
    .add_hook(ActivityLoggerHook::new())
    .add_hook(CustomHook::new(|ctx| {
        // Custom logic
        Ok(HookResult::Continue)
    }))
    .input_prompt("Analyze: {user_message}")
    .build()?;

engine.register_workflow(my_workflow);
engine.set_workflow("my-workflow");
```

### Multiple Workflows

```rust
// Register multiple workflows
engine.register_workflow(CodingWorkflow::new());
engine.register_workflow(ResearchWorkflow::new());
engine.register_workflow(DocumentationWorkflow::new());

// Switch based on task
engine.set_workflow("research");
```

### Event Observation

```rust
struct MetricsObserver;

#[async_trait]
impl WorkflowObserver for MetricsObserver {
    async fn on_event(&self, event: &WorkflowEvent) {
        match event {
            WorkflowEvent::TaskCompleted { task_id, .. } => {
                metrics::increment("tasks_completed");
            }
            WorkflowEvent::FileEdited { path } => {
                metrics::increment("files_edited");
            }
            _ => {}
        }
    }
}

engine.add_observer(Arc::new(MetricsObserver));
```

---

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::MockEroldRepository;

    #[tokio::test]
    async fn test_task_analysis_validates_existing_task() {
        let mut mock = MockEroldRepository::new();
        mock.add_task(Task { id: "task-1".into(), ... });

        let engine = WorkflowEngine::builder()
            .repository(mock)
            .build()?;

        // Valid task ID
        let analysis = TaskAnalysis {
            existing_task_id: Some("task-1".into()),
            ..Default::default()
        };
        assert!(engine.validate_analysis(&analysis).await.is_ok());

        // Invalid task ID
        let analysis = TaskAnalysis {
            existing_task_id: Some("nonexistent".into()),
            ..Default::default()
        };
        assert!(engine.validate_analysis(&analysis).await.is_err());
    }

    #[tokio::test]
    async fn test_read_before_edit_enforcement() {
        let engine = create_test_engine();

        // Edit without read should fail
        let result = engine.check_edit("src/main.rs").await;
        assert!(!result.allowed);

        // Read the file
        engine.on_file_read("src/main.rs").await;

        // Now edit should succeed
        let result = engine.check_edit("src/main.rs").await;
        assert!(result.allowed);
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_full_workflow_cycle() {
    let client = EroldClient::from_env()?;
    let repository = EroldApiRepository::new(client);
    let engine = WorkflowEngine::new(repository);

    // Start session
    engine.start_session().await?;

    // Analyze input
    let analysis = engine.analyze_input("Fix login bug").await?;
    assert!(analysis.is_actionable_task);

    // Simulate work
    engine.on_file_read("src/auth.rs").await;
    engine.on_file_edit("src/auth.rs").await?;

    // Extract outcomes
    let outcome = engine.extract_outcomes().await?;
    assert!(outcome.task_complete || !outcome.blocked.is_blocked);
}
```

---

## Implementation Phases

### Phase 1: Core Traits & Types
1. Define `Workflow` trait
2. Define `Hook` trait
3. Define `WorkflowObserver` trait
4. Define `EroldRepository` trait
5. Create schemas (`TaskAnalysis`, `WorkOutcome`)

### Phase 2: Base Infrastructure
1. Implement `HookChain`
2. Implement `WorkflowContext`
3. Implement `WorkflowEngine`
4. Implement `EroldApiRepository`
5. Implement `MockEroldRepository`

### Phase 3: Hooks
1. `ContextLoaderHook`
2. `ActivityLoggerHook`
3. `ReadTrackerHook`
4. `EditGuardHook`
5. `TaskValidatorHook`
6. `KnowledgeSearchHook`

### Phase 4: Workflows
1. `BaseWorkflow` (template method)
2. `CodingWorkflow`
3. `ResearchWorkflow`
4. `DocumentationWorkflow`
5. `BugfixWorkflow`

### Phase 5: Configuration
1. `WorkflowConfig` structure
2. TOML/YAML loader
3. `WorkflowBuilder`
4. `WorkflowFactory`

### Phase 6: Integration
1. Wire into `core/src/codex.rs`
2. Wire into tool handlers
3. Add model adapter
4. End-to-end testing

### Phase 7: Cleanup
1. Remove old keyword-based code
2. Update existing tests
3. Documentation
4. Examples

---

## Success Criteria

| Requirement | How We Achieve It |
|-------------|-------------------|
| Extensible | Workflow/Hook traits, factory pattern |
| Testable | Repository pattern, mock implementations |
| Configurable | TOML/YAML config, builder pattern |
| Observable | Event system, observer pattern |
| Multiple workflows | Strategy pattern, workflow registry |
| Easy to update | Prompts in files, schemas separate |
| Type-safe | Rust types, structured output validation |
| Reliable | Erold validation, no guessing |
