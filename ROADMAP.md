# Erold Code Roadmap

> Build a workflow-first AI coding agent with mandatory human-in-the-loop approval.

---

## Next Priority

| Feature | Status |
|---------|--------|
| Remove all web browsing tools | TODO |
| Add Guidelines API tool | TODO |
| Auto-fetch guidelines before task execution | TODO |
| Implement full Erold workflow enforcement | TODO |
| End-to-end workflow testing | TODO |

### Guidelines Integration
The model will automatically:
1. Detect language/framework from context
2. Fetch relevant guidelines from `erold.dev/api/v1/guidelines`
3. Apply guidelines before writing any code

### Testing Checklist
| Test | Status |
|------|--------|
| Guidelines fetch on startup | TODO |
| Workflow phase transitions | TODO |
| Task auto-creation | TODO |
| Knowledge base save/retrieve | TODO |
| Full end-to-end coding session | TODO |

---

## Vision

A **learning development agent** that:
- **Gets smarter over time** - saves learnings, avoids past mistakes
- **Always uses fresh knowledge** - TTL-based expiration, auto-refresh from internet
- **Per-subtask context** - fetches relevant knowledge for each subtask, not just task-level
- **Tracks everything** - decisions, tools used, outcomes for visibility
- **Human stays in control** - plan approval is mandatory, no loopholes

---

## The Learning Loop

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         CONTINUOUS LEARNING AGENT                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  USER REQUEST: "Add OAuth authentication"                                    │
│       │                                                                      │
│       ▼                                                                      │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │ TASK-LEVEL PREPROCESSING                                             │    │
│  │  • Fetch all knowledge from Erold                                    │    │
│  │  • Refresh stale knowledge from internet (if TTL expired)            │    │
│  │  • Filter for relevance to "OAuth authentication"                    │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│       │                                                                      │
│       ▼                                                                      │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │ PLANNING → Create subtasks → WAIT FOR HUMAN APPROVAL                 │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│       │                                                                      │
│       ▼                                                                      │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │ FOR EACH SUBTASK:                                                    │    │
│  │  ┌───────────────────────────────────────────────────────────────┐  │    │
│  │  │ 1. SUBTASK PREPROCESSING                                       │  │    │
│  │  │    • Search knowledge by subtask keywords                      │  │    │
│  │  │    • Fetch past mistakes (troubleshooting) to AVOID            │  │    │
│  │  │    • Refresh stale knowledge from internet                     │  │    │
│  │  ├───────────────────────────────────────────────────────────────┤  │    │
│  │  │ 2. EXECUTE                                                     │  │    │
│  │  │    • Inject past mistakes as warnings                          │  │    │
│  │  │    • Track files read/edited                                   │  │    │
│  │  │    • Log decisions made (chose X because Y)                    │  │    │
│  │  ├───────────────────────────────────────────────────────────────┤  │    │
│  │  │ 3. MINI-ENRICH                                                 │  │    │
│  │  │    • Save subtask notes, keywords used, knowledge used         │  │    │
│  │  └───────────────────────────────────────────────────────────────┘  │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│       │                                                                      │
│       ▼                                                                      │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │ FINAL ENRICHMENT                                                     │    │
│  │  • Save learnings (what worked) → knowledge base                     │    │
│  │  • Save mistakes (what failed) → troubleshooting category            │    │
│  │  • Save decisions (why we chose X) → workflow category               │    │
│  │  • Complete task with summary                                        │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│       │                                                                      │
│       ▼                                                                      │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │ KNOWLEDGE BASE GROWS → Agent gets smarter → Next task benefits       │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Knowledge Freshness

```
┌─────────────────────────────────────────────────────────────────┐
│                    KNOWLEDGE TTL SYSTEM                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  KNOWLEDGE ENTRY                                                 │
│  ├─ title: "Next.js 15 App Router Patterns"                     │
│  ├─ ttlDays: 30                                                 │
│  ├─ sourceUrl: "https://nextjs.org/docs"                        │
│  ├─ lastRefreshedAt: "2024-12-01"                               │
│  └─ autoRefresh: true                                           │
│                                                                  │
│  ON ACCESS:                                                      │
│  ├─ Is (today - lastRefreshedAt) > ttlDays?                     │
│  │   │                                                          │
│  │   ├─ NO  → Use cached version                                │
│  │   │                                                          │
│  │   └─ YES → Fetch sourceUrl                                   │
│  │            ├─ Parse content                                  │
│  │            ├─ Update knowledge in Erold                      │
│  │            └─ Use fresh version                              │
│                                                                  │
│  TTL GUIDELINES:                                                 │
│  ├─ Library versions:     7-14 days                             │
│  ├─ Framework guides:     30 days                               │
│  ├─ Security practices:   14 days                               │
│  ├─ Project conventions:  ∞ (no expiry)                         │
│  └─ Learnings/Mistakes:   ∞ (no expiry)                         │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                              EROLD CLI                                          │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                 │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐      │
│  │   CLI UI    │───▶│  WORKFLOW   │───▶│    LLM      │───▶│   TOOLS     │      │
│  │  (ratatui)  │    │   ENGINE    │    │  (Claude)   │    │  (handlers) │      │
│  └─────────────┘    └──────┬──────┘    └─────────────┘    └─────────────┘      │
│                            │                                                    │
│                            ▼                                                    │
│                    ┌─────────────┐                                              │
│                    │  EROLD API  │                                              │
│                    │   CLIENT    │                                              │
│                    └─────────────┘                                              │
│                                                                                 │
└─────────────────────────────────────────────────────────────────────────────────┘
```

---

## Phase 1: Foundation (Week 1)

### 1.1 Project Structure

```
erold-cli/
├── Cargo.toml                    # Workspace definition
├── crates/
│   ├── erold-api/                # Erold API client
│   │   ├── src/
│   │   │   ├── client.rs         # HTTP client
│   │   │   ├── models.rs         # Task, Knowledge, etc.
│   │   │   └── error.rs
│   │   └── Cargo.toml
│   │
│   ├── erold-config/             # Configuration
│   │   ├── src/
│   │   │   ├── loader.rs         # Config loading
│   │   │   ├── credentials.rs    # API key management
│   │   │   └── types.rs          # Config structs
│   │   └── Cargo.toml
│   │
│   ├── erold-workflow/           # Workflow engine
│   │   ├── src/
│   │   │   ├── engine.rs         # Main orchestrator
│   │   │   ├── phases/
│   │   │   │   ├── preprocess.rs # Phase 1: Fetch context
│   │   │   │   ├── plan.rs       # Phase 2: Create & approve plan
│   │   │   │   ├── execute.rs    # Phase 3: Run with gates
│   │   │   │   └── enrich.rs     # Phase 4: Save learnings
│   │   │   ├── state.rs          # Workflow state machine
│   │   │   └── error.rs
│   │   └── Cargo.toml
│   │
│   ├── erold-tools/              # Tool implementations
│   │   ├── src/
│   │   │   ├── registry.rs       # Tool registry
│   │   │   ├── read.rs           # Read file
│   │   │   ├── write.rs          # Write file
│   │   │   ├── shell.rs          # Run commands
│   │   │   ├── search.rs         # Grep/glob
│   │   │   └── plan.rs           # Update plan tool
│   │   └── Cargo.toml
│   │
│   ├── erold-llm/                # LLM integration
│   │   ├── src/
│   │   │   ├── client.rs         # Claude API client
│   │   │   ├── messages.rs       # Message formatting
│   │   │   └── tools.rs          # Tool call handling
│   │   └── Cargo.toml
│   │
│   ├── erold-web/                # Web fetching (for knowledge refresh)
│   │   ├── src/
│   │   │   ├── fetcher.rs        # HTTP fetching
│   │   │   ├── parsers/          # Content parsers
│   │   │   │   ├── docs.rs       # Documentation sites
│   │   │   │   ├── npm.rs        # npm registry
│   │   │   │   └── github.rs     # GitHub repos/releases
│   │   │   └── cache.rs          # Response caching
│   │   └── Cargo.toml
│   │
│   └── erold-tui/                # Terminal UI
│       ├── src/
│       │   ├── app.rs            # Main app state
│       │   ├── views/
│       │   │   ├── chat.rs       # Chat view
│       │   │   ├── plan.rs       # Plan approval view
│       │   │   └── progress.rs   # Progress view
│       │   └── input.rs          # Input handling
│       └── Cargo.toml
│
├── src/
│   └── main.rs                   # CLI entry point
│
└── docs/
    ├── WORKFLOW.md
    └── WORKFLOW_EXECUTION_GRAPH.txt
```

### 1.2 Core Dependencies

```toml
# Cargo.toml (workspace)
[workspace]
members = [
    "crates/erold-api",
    "crates/erold-config",
    "crates/erold-workflow",
    "crates/erold-tools",
    "crates/erold-llm",
    "crates/erold-web",
    "crates/erold-tui",
]

[workspace.dependencies]
tokio = { version = "1.0", features = ["full"] }
reqwest = { version = "0.12", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
ratatui = "0.28"
crossterm = "0.28"
```

### 1.3 Deliverables

- [ ] Initialize Rust workspace
- [ ] Set up crate structure
- [ ] Basic error types
- [ ] Logging infrastructure

---

## Phase 2: Erold API Client (Week 1-2)

### 2.1 API Client

```rust
// erold-api/src/client.rs
pub struct EroldClient {
    http: reqwest::Client,
    base_url: String,
    api_key: String,
    project_id: String,
}

impl EroldClient {
    // Context
    pub async fn get_context(&self) -> Result<ProjectContext>;

    // Tasks
    pub async fn create_task(&self, task: &CreateTask) -> Result<Task>;
    pub async fn get_task(&self, id: &str) -> Result<Task>;
    pub async fn update_task(&self, id: &str, update: &UpdateTask) -> Result<Task>;
    pub async fn approve_task(&self, id: &str) -> Result<Task>;
    pub async fn reject_task(&self, id: &str, reason: &str) -> Result<Task>;

    // Knowledge
    pub async fn list_knowledge(&self) -> Result<Vec<Knowledge>>;
    pub async fn search_knowledge(&self, query: &str) -> Result<Vec<Knowledge>>;
    pub async fn save_knowledge(&self, knowledge: &CreateKnowledge) -> Result<Knowledge>;

    // Tech Info
    pub async fn get_tech_info(&self) -> Result<TechInfo>;
    pub async fn update_tech_info(&self, info: &UpdateTechInfo) -> Result<TechInfo>;
}
```

### 2.2 Models

```rust
// erold-api/src/models.rs
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub subtasks: Vec<Subtask>,
    pub progress_percent: Option<i32>,
}

pub enum TaskStatus {
    PendingApproval,
    Approved,
    InProgress,
    Completed,
    Rejected,
    Blocked,
}

pub struct Subtask {
    pub id: String,
    pub title: String,
    pub status: SubtaskStatus,
    pub order: i32,

    // Per-subtask learning context
    pub keywords: Vec<String>,             // For knowledge search
    pub knowledge_used: Vec<String>,       // IDs of knowledge that helped
    pub notes: Option<String>,             // Agent observations
    pub decisions: Vec<Decision>,          // Decisions made during this subtask
}

pub struct Decision {
    pub description: String,               // What was decided
    pub chose: String,                     // What was chosen
    pub reason: String,                    // Why
    pub alternatives: Vec<String>,         // What was considered
}

pub struct Knowledge {
    pub id: String,
    pub title: String,
    pub content: String,
    pub category: String,
    pub tags: Vec<String>,

    // Freshness (for auto-refresh)
    pub ttl_days: Option<u32>,           // How long until stale
    pub source_url: Option<String>,       // Where to refresh from
    pub source_type: Option<SourceType>,  // docs, npm, github, manual
    pub last_refreshed_at: Option<DateTime<Utc>>,
    pub auto_refresh: bool,
}

pub enum SourceType {
    Docs,      // Official documentation
    Npm,       // npm registry
    Crates,    // crates.io
    GitHub,    // GitHub repo/releases
    Manual,    // Human-entered, no auto-refresh
}
```

### 2.3 Deliverables

- [ ] HTTP client with retry logic
- [ ] All API endpoints implemented
- [ ] Model serialization/deserialization
- [ ] Integration tests against real API

---

## Phase 3: Configuration (Week 2)

### 3.1 Config Structure

```rust
// erold-config/src/types.rs
pub struct EroldConfig {
    pub api: ApiConfig,
    pub workflow: WorkflowConfig,
    pub llm: LlmConfig,
}

pub struct ApiConfig {
    pub url: String,
    pub timeout_secs: u64,
}

pub struct WorkflowConfig {
    // ALL TRUE BY DEFAULT - no opt-in required
    pub require_plan: bool,              // default: true
    pub require_approval: bool,          // default: true
    pub require_read_before_edit: bool,  // default: true
    pub auto_enrich: bool,               // default: true

    // Timeouts
    pub approval_timeout_secs: u64,      // default: 300
    pub approval_poll_secs: u64,         // default: 5
}

pub struct LlmConfig {
    pub model: String,                   // default: claude-sonnet-4-20250514
    pub max_tokens: u32,
    pub temperature: f32,
}
```

### 3.2 Credentials

```rust
// erold-config/src/credentials.rs
pub struct Credentials {
    pub erold_api_key: String,
    pub anthropic_api_key: String,
}

impl Credentials {
    pub fn load() -> Result<Self>;  // From ~/.erold/credentials.toml
}
```

### 3.3 Deliverables

- [ ] Config file loading (~/.erold/config.toml)
- [ ] Credentials loading (~/.erold/credentials.toml)
- [ ] Project linking (.erold/project.json)
- [ ] Environment variable overrides

---

## Phase 4: Workflow Engine (Week 2-3)

### 4.1 State Machine

```rust
// erold-workflow/src/state.rs
pub enum WorkflowState {
    /// Initial state - waiting for user input
    Idle,

    /// Phase 1: Fetching context from Erold
    Preprocessing {
        message: String,
    },

    /// Phase 2: Plan created, waiting for approval
    AwaitingApproval {
        task_id: String,
        plan: Vec<PlanItem>,
    },

    /// Phase 3: Executing approved plan
    Executing {
        task_id: String,
        current_subtask: usize,
        total_subtasks: usize,
    },

    /// Phase 4: Saving learnings
    Enriching {
        task_id: String,
        learnings: Vec<Learning>,
    },

    /// Workflow complete
    Completed {
        task_id: String,
        summary: String,
    },

    /// Error state
    Failed {
        error: String,
        recoverable: bool,
    },
}
```

### 4.2 Engine

```rust
// erold-workflow/src/engine.rs
pub struct WorkflowEngine {
    state: WorkflowState,
    erold: Arc<EroldClient>,
    llm: Arc<LlmClient>,
    tools: Arc<ToolRegistry>,
    config: WorkflowConfig,
}

impl WorkflowEngine {
    /// Process user message through the full workflow
    pub async fn process(&mut self, message: &str) -> Result<WorkflowResult> {
        // 1. PREPROCESS - Always runs, no exceptions
        let context = self.preprocess(message).await?;

        // 2. PLAN - Create plan, save to Erold
        let plan = self.create_plan(message, &context).await?;

        // 3. APPROVE - Wait for human approval (MANDATORY)
        self.wait_for_approval(&plan.task_id).await?;

        // 4. EXECUTE - Run plan with gates
        let result = self.execute(&plan).await?;

        // 5. ENRICH - Save learnings
        self.enrich(&result).await?;

        Ok(result)
    }
}
```

### 4.3 Phase 1: Preprocessing (Task-Level)

```rust
// erold-workflow/src/phases/preprocess.rs
pub struct Preprocessor {
    erold: Arc<EroldClient>,
    llm: Arc<LlmClient>,
    web: Arc<WebFetcher>,  // For refreshing stale knowledge
}

impl Preprocessor {
    /// Task-level preprocessing - runs once at start
    pub async fn run(&self, message: &str) -> Result<PreprocessedContext> {
        // 1. Fetch ALL from Erold
        let context = self.erold.get_context().await?;
        let knowledge = self.erold.list_knowledge().await?;
        let tech_info = self.erold.get_tech_info().await?;
        let tasks = self.erold.list_tasks().await?;

        // 2. Check & refresh expired knowledge
        let refreshed_knowledge = self.refresh_stale_knowledge(knowledge).await?;

        // 3. LLM filters relevance
        let relevance = self.llm.filter_relevance(message, &FetchedContext {
            context,
            knowledge: refreshed_knowledge,
            tech_info,
            tasks,
        }).await?;

        Ok(PreprocessedContext {
            relevant_knowledge: relevance.knowledge,
            relevant_tasks: relevance.tasks,
            tech_info: if relevance.needs_tech { Some(tech_info) } else { None },
            complexity: relevance.complexity,
        })
    }

    /// Subtask-level preprocessing - runs before EACH subtask
    pub async fn run_for_subtask(
        &self,
        subtask: &Subtask,
        task_context: &PreprocessedContext,
    ) -> Result<SubtaskContext> {
        // 1. Extract keywords from subtask title
        let keywords = self.llm.extract_keywords(&subtask.title).await?;

        // 2. Search knowledge by subtask keywords
        let subtask_knowledge = self.erold
            .search_knowledge(&keywords.join(" "))
            .await?;

        // 3. Check for past mistakes (troubleshooting category)
        let past_mistakes = self.erold
            .search_knowledge_by_category("troubleshooting", &keywords)
            .await?;

        // 4. Refresh any stale knowledge
        let fresh_knowledge = self.refresh_stale_knowledge(subtask_knowledge).await?;

        Ok(SubtaskContext {
            keywords,
            relevant_knowledge: fresh_knowledge,
            past_mistakes,
            inherited_context: task_context.clone(),
        })
    }

    /// Check and refresh stale knowledge from internet
    async fn refresh_stale_knowledge(&self, knowledge: Vec<Knowledge>) -> Result<Vec<Knowledge>> {
        let mut refreshed = Vec::new();

        for k in knowledge {
            if k.is_expired() && k.auto_refresh && k.source_url.is_some() {
                // Fetch fresh content from source
                match self.web.fetch(&k.source_url.unwrap()).await {
                    Ok(fresh_content) => {
                        // Update knowledge in Erold
                        let updated = self.erold.update_knowledge(&k.id, &UpdateKnowledge {
                            content: Some(fresh_content),
                            last_refreshed_at: Some(Utc::now()),
                        }).await?;
                        refreshed.push(updated);
                    }
                    Err(e) => {
                        // Log warning, use stale version
                        tracing::warn!("Failed to refresh knowledge {}: {}", k.id, e);
                        refreshed.push(k);
                    }
                }
            } else {
                refreshed.push(k);
            }
        }

        Ok(refreshed)
    }
}
```

### 4.4 Phase 2: Planning

```rust
// erold-workflow/src/phases/plan.rs
pub struct Planner {
    erold: Arc<EroldClient>,
    llm: Arc<LlmClient>,
}

impl Planner {
    pub async fn create_plan(
        &self,
        message: &str,
        context: &PreprocessedContext,
    ) -> Result<Plan> {
        // 1. LLM creates plan
        let plan_items = self.llm.create_plan(message, context).await?;

        // 2. Create task in Erold with subtasks
        let task = self.erold.create_task(&CreateTask {
            title: self.extract_title(message),
            description: Some(message.to_string()),
            status: "pending_approval".to_string(),
            subtasks: plan_items.iter().enumerate().map(|(i, item)| {
                Subtask {
                    title: item.step.clone(),
                    status: "pending".to_string(),
                    order: i as i32,
                }
            }).collect(),
        }).await?;

        Ok(Plan {
            task_id: task.id,
            items: plan_items,
        })
    }

    pub async fn wait_for_approval(&self, task_id: &str) -> Result<ApprovalResult> {
        let start = Instant::now();
        let timeout = Duration::from_secs(self.config.approval_timeout_secs);
        let poll_interval = Duration::from_secs(self.config.approval_poll_secs);

        loop {
            if start.elapsed() > timeout {
                return Err(WorkflowError::ApprovalTimeout);
            }

            let task = self.erold.get_task(task_id).await?;

            match task.status {
                TaskStatus::Approved | TaskStatus::InProgress => {
                    return Ok(ApprovalResult::Approved);
                }
                TaskStatus::Rejected => {
                    return Err(WorkflowError::PlanRejected {
                        reason: task.rejection_reason,
                    });
                }
                _ => {
                    // Still pending, wait and poll again
                    tokio::time::sleep(poll_interval).await;
                }
            }
        }
    }
}
```

### 4.5 Phase 3: Execution (Per-Subtask Loop)

```rust
// erold-workflow/src/phases/execute.rs
pub struct Executor {
    erold: Arc<EroldClient>,
    llm: Arc<LlmClient>,
    tools: Arc<ToolRegistry>,
    preprocessor: Arc<Preprocessor>,  // For per-subtask preprocessing
    read_files: HashSet<PathBuf>,
    decisions: Vec<Decision>,         // Track all decisions
}

impl Executor {
    pub async fn execute(
        &mut self,
        plan: &Plan,
        task_context: &PreprocessedContext,
    ) -> Result<ExecutionResult> {
        let mut learnings = Vec::new();
        let mut all_decisions = Vec::new();

        for (index, subtask) in plan.subtasks.iter().enumerate() {
            // ═══════════════════════════════════════════════════════════
            // PER-SUBTASK PREPROCESSING - Fetch knowledge for THIS subtask
            // ═══════════════════════════════════════════════════════════
            let subtask_context = self.preprocessor
                .run_for_subtask(subtask, task_context)
                .await?;

            // Log what knowledge we're using
            tracing::info!(
                "Subtask '{}' using {} knowledge items, {} past mistakes to avoid",
                subtask.title,
                subtask_context.relevant_knowledge.len(),
                subtask_context.past_mistakes.len()
            );

            // Mark subtask as in_progress
            self.update_subtask_status(&plan.task_id, index, "in_progress").await?;

            // ═══════════════════════════════════════════════════════════
            // EXECUTE with subtask-specific context
            // ═══════════════════════════════════════════════════════════
            let result = self.execute_subtask(subtask, &subtask_context).await?;

            // Collect learnings and decisions
            learnings.extend(result.learnings);
            all_decisions.extend(result.decisions.clone());

            // Update subtask with what we learned
            self.erold.update_subtask(&plan.task_id, index, &UpdateSubtask {
                status: Some("completed".to_string()),
                keywords: Some(subtask_context.keywords),
                knowledge_used: Some(
                    subtask_context.relevant_knowledge.iter().map(|k| k.id.clone()).collect()
                ),
                notes: result.notes,
                decisions: Some(result.decisions),
            }).await?;
        }

        Ok(ExecutionResult {
            task_id: plan.task_id.clone(),
            learnings,
            decisions: all_decisions,
        })
    }

    /// Execute single subtask with its context
    async fn execute_subtask(
        &mut self,
        subtask: &Subtask,
        context: &SubtaskContext,
    ) -> Result<SubtaskResult> {
        // Inject past mistakes as warnings
        let system_prompt = self.build_system_prompt(context);

        // Run LLM with tools
        let result = self.llm.chat_with_tools(
            &system_prompt,
            &subtask.title,
            &self.tools,
        ).await?;

        // Extract any decisions made
        let decisions = self.extract_decisions(&result);

        Ok(SubtaskResult {
            learnings: result.learnings,
            decisions,
            notes: result.notes,
        })
    }

    /// Build system prompt including past mistakes to avoid
    fn build_system_prompt(&self, context: &SubtaskContext) -> String {
        let mut prompt = String::new();

        if !context.past_mistakes.is_empty() {
            prompt.push_str("\n⚠️ AVOID THESE PAST MISTAKES:\n");
            for mistake in &context.past_mistakes {
                prompt.push_str(&format!("- {}: {}\n", mistake.title, mistake.content));
            }
        }

        if !context.relevant_knowledge.is_empty() {
            prompt.push_str("\n📚 RELEVANT KNOWLEDGE:\n");
            for k in &context.relevant_knowledge {
                prompt.push_str(&format!("- {}: {}\n", k.title, k.content));
            }
        }

        prompt
    }

    /// Gate: Must read before edit
    pub fn check_can_edit(&self, path: &Path) -> Result<()> {
        if !self.read_files.contains(path) {
            return Err(WorkflowError::MustReadBeforeEdit {
                path: path.to_path_buf(),
            });
        }
        Ok(())
    }

    pub fn on_file_read(&mut self, path: &Path) {
        self.read_files.insert(path.to_path_buf());
    }
}
```

### 4.6 Phase 4: Enrichment (Learning Loop)

```rust
// erold-workflow/src/phases/enrich.rs
pub struct Enricher {
    erold: Arc<EroldClient>,
    llm: Arc<LlmClient>,
}

impl Enricher {
    pub async fn enrich(&self, result: &ExecutionResult) -> Result<()> {
        // ═══════════════════════════════════════════════════════════
        // 1. SAVE LEARNINGS - What worked, new discoveries
        // ═══════════════════════════════════════════════════════════
        for learning in &result.learnings {
            self.erold.save_knowledge(&CreateKnowledge {
                title: learning.title.clone(),
                content: learning.content.clone(),
                category: learning.category.clone(),
                tags: learning.tags.clone(),
                source: "agent".to_string(),
                // No TTL for learnings - they don't expire
                ttl_days: None,
                auto_refresh: false,
            }).await?;
        }

        // ═══════════════════════════════════════════════════════════
        // 2. SAVE MISTAKES - What didn't work (troubleshooting)
        // ═══════════════════════════════════════════════════════════
        for mistake in &result.mistakes {
            self.erold.save_knowledge(&CreateKnowledge {
                title: format!("Don't: {}", mistake.what_failed),
                content: format!(
                    "## Problem\n{}\n\n## Wrong Approach\n{}\n\n## Why It Failed\n{}\n\n## Correct Approach\n{}",
                    mistake.problem,
                    mistake.wrong_approach,
                    mistake.why_failed,
                    mistake.correct_approach,
                ),
                category: "troubleshooting".to_string(),
                tags: [
                    vec!["mistake".to_string(), "dont-repeat".to_string()],
                    mistake.tags.clone(),
                ].concat(),
                source: "agent".to_string(),
                ttl_days: None,  // Mistakes don't expire
                auto_refresh: false,
            }).await?;
        }

        // ═══════════════════════════════════════════════════════════
        // 3. SAVE DECISIONS - Why we chose what we chose
        // ═══════════════════════════════════════════════════════════
        if !result.decisions.is_empty() {
            let decisions_content = result.decisions.iter()
                .map(|d| format!(
                    "### {}\n- **Chose**: {}\n- **Reason**: {}\n- **Alternatives**: {}",
                    d.description, d.chose, d.reason, d.alternatives.join(", ")
                ))
                .collect::<Vec<_>>()
                .join("\n\n");

            self.erold.save_knowledge(&CreateKnowledge {
                title: format!("Decisions for task: {}", result.task_title),
                content: decisions_content,
                category: "workflow".to_string(),
                tags: vec!["decisions".to_string(), "rationale".to_string()],
                source: "agent".to_string(),
                ttl_days: None,
                auto_refresh: false,
            }).await?;
        }

        // ═══════════════════════════════════════════════════════════
        // 4. DETECT & UPDATE TECH INFO
        // ═══════════════════════════════════════════════════════════
        let detected_tech = self.detect_tech_changes().await?;
        if !detected_tech.is_empty() {
            self.erold.update_tech_info(&UpdateTechInfo {
                dependencies: detected_tech,
            }).await?;
        }

        // ═══════════════════════════════════════════════════════════
        // 5. COMPLETE THE TASK
        // ═══════════════════════════════════════════════════════════
        self.erold.update_task(&result.task_id, &UpdateTask {
            status: Some("done".to_string()),
            progress_percent: Some(100),
            completion_summary: Some(self.generate_summary(result).await?),
        }).await?;

        Ok(())
    }

    /// Generate summary of what was accomplished
    async fn generate_summary(&self, result: &ExecutionResult) -> Result<String> {
        self.llm.summarize(&format!(
            "Task: {}\nLearnings: {:?}\nDecisions: {:?}",
            result.task_title,
            result.learnings,
            result.decisions,
        )).await
    }
}
```

### 4.7 Deliverables

- [ ] State machine implementation
- [ ] Preprocessor (fetch + filter + inject)
- [ ] Planner (create + approval wait)
- [ ] Executor (gates + progress tracking)
- [ ] Enricher (save learnings)
- [ ] Integration tests

---

## Phase 5: Tool Implementations (Week 3)

### 5.1 Tool Registry

```rust
// erold-tools/src/registry.rs
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters(&self) -> serde_json::Value;

    async fn execute(
        &self,
        params: serde_json::Value,
        ctx: &mut ToolContext,
    ) -> Result<ToolOutput>;
}

pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        let mut registry = Self { tools: HashMap::new() };

        // Register all tools
        registry.register(Arc::new(ReadFileTool));
        registry.register(Arc::new(WriteFileTool));
        registry.register(Arc::new(ShellTool));
        registry.register(Arc::new(GlobTool));
        registry.register(Arc::new(GrepTool));
        registry.register(Arc::new(UpdatePlanTool));

        registry
    }
}
```

### 5.2 Tool Implementations

```rust
// erold-tools/src/read.rs
pub struct ReadFileTool;

impl Tool for ReadFileTool {
    fn name(&self) -> &str { "read_file" }

    async fn execute(&self, params: Value, ctx: &mut ToolContext) -> Result<ToolOutput> {
        let path: PathBuf = serde_json::from_value(params["path"].clone())?;

        // Read file
        let content = tokio::fs::read_to_string(&path).await?;

        // Track that file was read (for read-before-edit gate)
        ctx.executor.on_file_read(&path);

        Ok(ToolOutput::text(content))
    }
}

// erold-tools/src/write.rs
pub struct WriteFileTool;

impl Tool for WriteFileTool {
    fn name(&self) -> &str { "write_file" }

    async fn execute(&self, params: Value, ctx: &mut ToolContext) -> Result<ToolOutput> {
        let path: PathBuf = serde_json::from_value(params["path"].clone())?;
        let content: String = serde_json::from_value(params["content"].clone())?;

        // GATE: Must have read file first
        ctx.executor.check_can_edit(&path)?;

        // GATE: Must have approved plan
        if !ctx.has_approved_plan() {
            return Err(ToolError::NoPlanApproved);
        }

        // Write file
        tokio::fs::write(&path, &content).await?;

        Ok(ToolOutput::text(format!("Wrote {} bytes to {}", content.len(), path.display())))
    }
}
```

### 5.3 Deliverables

- [ ] read_file tool
- [ ] write_file tool (with gates)
- [ ] shell tool
- [ ] glob tool
- [ ] grep tool
- [ ] update_plan tool

---

## Phase 6: LLM Integration (Week 3-4)

### 6.1 Claude Client

```rust
// erold-llm/src/client.rs
pub struct LlmClient {
    http: reqwest::Client,
    api_key: String,
    model: String,
}

impl LlmClient {
    pub async fn chat(
        &self,
        messages: Vec<Message>,
        tools: &[ToolDefinition],
    ) -> Result<LlmResponse> {
        let response = self.http
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&ChatRequest {
                model: &self.model,
                messages,
                tools,
                max_tokens: 8192,
            })
            .send()
            .await?;

        // Parse response, handle tool calls
        Ok(response.json().await?)
    }

    pub async fn filter_relevance(
        &self,
        message: &str,
        context: &FetchedContext,
    ) -> Result<RelevanceFilter> {
        // Structured output for relevance filtering
        todo!()
    }

    pub async fn create_plan(
        &self,
        message: &str,
        context: &PreprocessedContext,
    ) -> Result<Vec<PlanItem>> {
        // Structured output for plan creation
        todo!()
    }
}
```

### 6.2 Deliverables

- [ ] Claude API client
- [ ] Message formatting
- [ ] Tool call handling
- [ ] Structured output parsing
- [ ] Streaming support

---

## Phase 7: Terminal UI (Week 4)

### 7.1 App Structure

```rust
// erold-tui/src/app.rs
pub struct App {
    state: AppState,
    workflow: WorkflowEngine,
    input: InputState,
}

pub enum AppState {
    /// Normal chat input
    Chat {
        messages: Vec<ChatMessage>,
    },

    /// Showing plan for approval
    PlanApproval {
        task_id: String,
        plan: Vec<PlanItem>,
    },

    /// Executing with progress
    Executing {
        task_id: String,
        current_step: usize,
        total_steps: usize,
        logs: Vec<String>,
    },
}
```

### 7.2 Views

```rust
// erold-tui/src/views/plan.rs
pub fn render_plan_approval(f: &mut Frame, plan: &[PlanItem], task_id: &str) {
    // Show plan items
    // [A]pprove / [R]eject buttons
    // Link to Erold web UI
}

// erold-tui/src/views/progress.rs
pub fn render_progress(f: &mut Frame, current: usize, total: usize, logs: &[String]) {
    // Progress bar
    // Current step highlighted
    // Scrollable logs
}
```

### 7.3 Deliverables

- [ ] Basic TUI layout
- [ ] Chat view
- [ ] Plan approval view
- [ ] Progress view
- [ ] Input handling
- [ ] Keyboard shortcuts

---

## Phase 8: CLI Entry Point (Week 4)

### 8.1 Main

```rust
// src/main.rs
#[tokio::main]
async fn main() -> Result<()> {
    // 1. Parse args
    let args = Args::parse();

    // 2. Load config
    let config = EroldConfig::load()?;

    // 3. Load credentials (REQUIRED)
    let credentials = Credentials::load()
        .context("Erold credentials required. Run 'erold login' first.")?;

    // 4. Load project (REQUIRED)
    let project = Project::load()
        .context("Project not linked. Run 'erold link' first.")?;

    // 5. Initialize clients
    let erold = EroldClient::new(&config.api, &credentials, &project.id);
    let llm = LlmClient::new(&config.llm, &credentials.anthropic_api_key);

    // 6. Initialize workflow engine
    let workflow = WorkflowEngine::new(erold, llm, config.workflow);

    // 7. Run TUI
    let app = App::new(workflow);
    app.run().await
}
```

### 8.2 Commands

```bash
# Main command - interactive mode
erold

# Setup commands
erold login              # Configure credentials
erold link               # Link to Erold project
erold config             # Edit config

# One-shot mode
erold run "Add authentication"
```

### 8.3 Deliverables

- [ ] Argument parsing (clap)
- [ ] Login command
- [ ] Link command
- [ ] Interactive mode
- [ ] One-shot mode

---

## Success Criteria

### Mandatory Requirements

| Requirement | Implementation |
|-------------|----------------|
| Plan before edit | WorkflowState blocks execution until approved |
| Human approval | wait_for_approval() polls Erold, blocks until approved/rejected |
| Read before edit | check_can_edit() fails if file not in read_files set |
| No loopholes | No file count exceptions, no silent degradation |
| Erold required | Credentials::load() fails if not configured |

### Quality Gates

- [ ] All workflow states have tests
- [ ] API client has integration tests
- [ ] TUI has basic interaction tests
- [ ] E2E test: full workflow from message to completion

---

## Timeline Summary

| Phase | Duration | Deliverables |
|-------|----------|--------------|
| 1. Foundation | 3 days | Project structure, logging |
| 2. API Client | 4 days | Full Erold API client |
| 3. Config | 2 days | Config + credentials |
| 4. Workflow | 5 days | 4-phase engine |
| 5. Tools | 3 days | All tool handlers |
| 6. LLM | 4 days | Claude integration |
| 7. TUI | 4 days | Terminal UI |
| 8. CLI | 2 days | Entry point + commands |

**Total: ~4 weeks**

---

## Next Steps

1. Initialize Rust workspace with crate structure
2. Implement erold-api client first (foundation for everything)
3. Build workflow engine with state machine
4. Add tools with gates
5. Integrate LLM
6. Build TUI
7. Polish and test
