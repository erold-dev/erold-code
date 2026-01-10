# Erold Workflow Enforcement - Official Schema

## Overview

This document defines the official workflow for the Erold CLI. The workflow enforces:
- Fixed preprocessing (always fetch, filter, inject)
- Plan-before-implement with task breakdown
- Read-before-edit gates
- Automatic enrichment (write-back to Erold)

---

## Complete Workflow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           USER MESSAGE                                       │
│                  "Add authentication to the app"                            │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  STEP 1: PREPROCESSING (Fixed - Always Runs)                                │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  1.1 Fetch ALL from Erold:                                                  │
│      ├── Project context (what project is this?)                            │
│      ├── Existing tasks (is there already a task for this?)                 │
│      ├── Knowledge base (auth guidelines, security best practices)          │
│      ├── Tech info (what stack? React? FastAPI? What auth libs exist?)      │
│      └── Vault keys (do we have API keys for auth providers?)               │
│                                                                             │
│  1.2 LLM Filters relevance:                                                 │
│      "Which of these items are relevant to 'Add authentication'?"           │
│      → Returns: security knowledge, existing auth tasks, JWT libs, etc.     │
│                                                                             │
│  1.3 Assess complexity & workflow type:                                     │
│      → Complexity: 7/10 (new feature, multiple files, security critical)    │
│      → Workflow: Coding                                                     │
│      → Requires planning: YES                                               │
│                                                                             │
│  1.4 Inject filtered context into conversation                              │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  STEP 2: PLANNING (If Required)                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  2.1 LLM creates plan:                                                      │
│      ┌─────────────────────────────────────────────────────────────────┐   │
│      │ Plan: Add Authentication                                         │   │
│      ├─────────────────────────────────────────────────────────────────┤   │
│      │ 1. Set up auth provider (NextAuth/Auth.js)                      │   │
│      │ 2. Create user model and database schema                        │   │
│      │ 3. Implement login/register API endpoints                       │   │
│      │ 4. Create login/register UI components                          │   │
│      │ 5. Add protected route middleware                               │   │
│      │ 6. Test authentication flow                                     │   │
│      └─────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  2.2 Save to Erold (Task with Subtasks):                                   │
│                                                                             │
│      ┌─────────────────────────────────────────────────────────────────┐   │
│      │ TASK                                                            │   │
│      │ Title: "Add authentication to the app"                          │   │
│      │ Status: in_progress                                             │   │
│      │                                                                 │   │
│      │ Subtasks (inline checklist):                                    │   │
│      │   □ 1. Set up auth provider (NextAuth/Auth.js)                  │   │
│      │   □ 2. Create user model and database schema                    │   │
│      │   □ 3. Implement login/register API endpoints                   │   │
│      │   □ 4. Create login/register UI components                      │   │
│      │   □ 5. Add protected route middleware                           │   │
│      │   □ 6. Test authentication flow                                 │   │
│      └─────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  2.3 Wait for approval (if configured)                                     │
│      → User approves in Erold or locally                                   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  STEP 3: EXECUTION (Per Task)                                               │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  For each subtask in order:                                                    │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ Subtask 1: Set up auth provider                                     │   │
│  ├─────────────────────────────────────────────────────────────────────┤   │
│  │                                                                     │   │
│  │  3.1 START SUBTASK                                                  │   │
│  │      → Mark subtask status: in_progress                             │   │
│  │      → Task progress: 0/6 → 1/6 in progress                         │   │
│  │                                                                     │   │
│  │  3.2 ENFORCE READ-BEFORE-EDIT                                       │   │
│  │      → Before editing package.json, MUST read it first              │   │
│  │      → Blocked if not read                                          │   │
│  │                                                                     │   │
│  │  3.3 EXECUTE WORK                                                   │   │
│  │      → Read package.json ✓                                          │   │
│  │      → Edit package.json (add next-auth) ✓                          │   │
│  │      → Create src/auth.ts ✓                                         │   │
│  │      → Create src/app/api/auth/[...nextauth]/route.ts ✓            │   │
│  │                                                                     │   │
│  │  3.4 COMPLETE SUBTASK                                               │   │
│  │      → Mark subtask status: completed                               │   │
│  │      → Task progress: 1/6 completed                                 │   │
│  │      → Capture learning: "Used NextAuth v5 with credentials"        │   │
│  │                                                                     │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  Repeat for Subtask 2, 3, 4, 5, 6...                                          │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  STEP 4: POST-PROCESSING (Enrichment)                                       │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  4.1 Update Erold with learnings:                                          │
│      ├── Knowledge: "Auth setup pattern for Next.js 15"                    │
│      ├── Knowledge: "JWT token rotation best practice"                     │
│      └── Solution: "Fixed CORS issue with credentials: include"           │
│                                                                             │
│  4.2 Update Tech Info:                                                     │
│      ├── Added: next-auth (auth library)                                   │
│      ├── Added: bcrypt (password hashing)                                  │
│      └── Added: jose (JWT handling)                                        │
│                                                                             │
│  4.3 Update Vault (if needed):                                             │
│      ├── Suggested: AUTH_SECRET (for JWT signing)                          │
│      └── Suggested: GOOGLE_CLIENT_ID (for OAuth)                           │
│                                                                             │
│  4.4 Complete Task:                                                        │
│      → All 6 subtasks completed                                            │
│      → Task status: completed                                              │
│      → Summary saved                                                       │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Step 1: Preprocessing (Fixed Workflow)

The preprocessing step ALWAYS runs, with no LLM discretion.

### 1.1 Fetch ALL from Erold

```rust
FetchedEroldContext {
    context: Option<EroldContext>,      // Project info, active tasks, blockers
    tech_info: Option<TechInfo>,        // Stack, commands, deployment
    knowledge: Vec<Knowledge>,          // All knowledge articles
    tasks: Vec<Task>,                   // All tasks
    vault_keys: Vec<VaultEntry>,        // Vault entries (keys only, no values)
}
```

### 1.2 LLM Filters Relevance

Send summary to LLM with structured output:

```rust
RelevanceFilter {
    relevant_knowledge_ids: Vec<String>,  // Which knowledge articles are relevant
    relevant_task_ids: Vec<String>,       // Which tasks are relevant
    needs_tech_info: bool,                // Is tech stack info needed?
    needs_vault_keys: Vec<String>,        // Which vault keys might be needed?
    complexity: u8,                       // 1-10 complexity score
    needs_planning: bool,                 // Does this need a plan?
}
```

### 1.3 Assess Complexity & Workflow Type

| Complexity | Description | Planning Required |
|------------|-------------|-------------------|
| 1-3 | Simple (typo fix, small change) | No |
| 4-6 | Moderate (new function, bug fix) | Maybe |
| 7-10 | Complex (new feature, refactor) | Yes |

| Workflow Type | Triggers | Gates |
|---------------|----------|-------|
| Coding | "add", "implement", "create" | Full enforcement |
| Bugfix | "fix", "bug", "error" | Read-before-edit |
| Research | "research", "explain", "how" | Minimal |
| Documentation | "document", "readme" | None |
| Refactoring | "refactor", "restructure" | Full + testing |

### 1.4 Inject Filtered Context

Build context injection string with:
- Relevant knowledge articles (full content)
- Relevant tasks
- Tech info (if needed)
- Vault key hints (names only)

---

## Step 2: Planning

### 2.1 Plan Structure

When `update_plan` tool is called, the plan contains:

```rust
UpdatePlanArgs {
    explanation: Option<String>,  // Overall goal
    plan: Vec<PlanItem>,          // Steps
}

PlanItem {
    step: String,                 // Step description
    status: PlanStatus,           // pending | in_progress | completed
}
```

### 2.2 Task Creation in Erold

**Create Task with Subtasks:**
```rust
CreateTask {
    title: "Add authentication to the app",
    description: "Implementation plan with 6 steps",
    status: "in_progress",
    priority: "medium",
    subtasks: vec![
        Subtask { title: "Set up auth provider (NextAuth/Auth.js)", status: "pending", order: 0 },
        Subtask { title: "Create user model and database schema", status: "pending", order: 1 },
        Subtask { title: "Implement login/register API endpoints", status: "pending", order: 2 },
        Subtask { title: "Create login/register UI components", status: "pending", order: 3 },
        Subtask { title: "Add protected route middleware", status: "pending", order: 4 },
        Subtask { title: "Test authentication flow", status: "pending", order: 5 },
    ],
}
```

### 2.3 Approval Flow

If `require_plan_approval` is enabled:

1. Create tasks in Erold with status `pending_approval`
2. Poll task status until approved/rejected
3. If approved → proceed with execution
4. If rejected → return error to model with reason

---

## Step 3: Execution

### 3.1 Subtask Lifecycle

```
pending → in_progress → completed
                     ↘ blocked (if issue found)
```

### 3.2 Read-Before-Edit Enforcement

Before any file edit:
1. Check if file was read in this session
2. If not → block edit, return error
3. If yes → allow edit

```rust
// In apply_patch handler
let enforcer = session.services.workflow_enforcer.read().await;
if !enforcer.can_edit(&file_path) {
    return Err("Must read file before editing: {file_path}");
}
```

### 3.3 Progress Tracking

After each subtask completion:
1. Mark subtask as `completed`
2. Calculate task progress: `completed_subtasks / total_subtasks`
3. Update task progressPercent field

---

## Step 4: Enrichment (Write-Back)

### 4.1 Knowledge Capture

On task completion, capture:
- What was done
- Any problems solved
- Patterns discovered

```rust
CapturedLearning {
    title: "Auth setup with NextAuth v5",
    category: "conventions",
    content: "Step-by-step guide...",
    tags: vec!["auth", "nextauth", "next.js"],
    task_id: Some(task.id),
}
```

### 4.2 Tech Info Update

Detect new technologies from:
- package.json dependencies
- Cargo.toml dependencies
- pyproject.toml dependencies
- Import statements

```rust
DetectedTech {
    name: "next-auth",
    version: Some("5.0.0"),
    category: TechCategory::Backend,
}
```

### 4.3 Vault Suggestions

When credentials are needed:
1. Check if key exists in vault
2. If not → suggest adding it
3. Never store actual values automatically

---

## Data Flow Summary

```
User Message
    │
    ▼
┌─────────────────┐
│ PREPROCESS      │ ◄── Erold API (READ)
│ Fetch → Filter  │     - get_context()
│ → Inject        │     - get_tech_info()
└────────┬────────┘     - list_knowledge()
         │              - list_tasks()
         ▼              - list_vault()
┌─────────────────┐
│ PLAN            │ ◄── Erold API (WRITE)
│ Create task     │     - create_task() [with subtasks]
│ Wait approval   │     - get_task() [poll approval]
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ EXECUTE         │ ◄── Erold API (WRITE)
│ Per-subtask     │     - update_task() [subtask status]
│ Read-before-    │     - update_task() [progressPercent]
│ edit gates      │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ ENRICH          │ ◄── Erold API (WRITE)
│ Save learnings  │     - save_knowledge()
│ Update tech     │     - update_tech_info()
│ Suggest vault   │     - create_vault_entry()
└─────────────────┘
```

---

## Configuration

```toml
[workflow]
# Enforcement gates
enforce_read_before_edit = true
require_plan_approval = false

# Thresholds
plan_complexity_threshold = 5      # Require plan if complexity >= this
plan_approval_timeout_secs = 300   # 5 minutes
plan_approval_poll_secs = 5

# Auto-enrichment
auto_capture_learnings = true
auto_detect_tech = true
auto_suggest_vault = true
```

---

## API Requirements

### Task Model with Subtasks

```rust
Task {
    id: String,
    title: String,
    description: Option<String>,
    status: String,                 // pending | in_progress | completed | blocked
    priority: String,
    progressPercent: Option<i32>,   // 0-100, calculated from subtasks
    subtasks: Vec<Subtask>,         // Inline checklist
    // ... other fields
}

Subtask {
    id: String,                     // Auto-generated UUID
    title: String,
    status: String,                 // pending | in_progress | completed
    order: i32,                     // Display order
}

CreateTask {
    title: String,
    description: Option<String>,
    status: String,
    priority: String,
    subtasks: Option<Vec<Subtask>>, // Optional inline subtasks
}
```

### API Endpoints (Already Implemented)

| Endpoint | Purpose |
|----------|---------|
| `POST /tasks` | Create task with subtasks |
| `GET /tasks/{id}` | Get task with subtasks |
| `PATCH /tasks/{id}` | Update task and/or subtasks |
| `POST /tasks/{id}/approve` | Approve a pending task |
| `POST /tasks/{id}/reject` | Reject a pending task |

---

## Implementation Checklist

- [x] Add `subtasks` array to Task model (Erold API)
- [x] Update plan handler to create task with subtasks
- [x] Add WorkflowEngine to SessionServices
- [x] Implement knowledge expiration check (TTL filter in KnowledgeSearchHook)
- [x] Wire file read/edit hooks (on_file_read, check_edit)
- [x] Create SubtaskTracker for progress tracking
- [x] Implement task approval waiting/polling
- [x] Wire enrichment (WorkflowEnricher complete with all methods)
- [x] Implement preprocessing (fetch → filter → inject) - wired in message_preprocessor.rs
- [x] Wire KnowledgeSearchHook in message preprocessing - calls engine.process_user_message()
- [x] WorkflowEngine coexists with legacy WorkflowEnforcer (graceful fallback)
- [x] Test complete workflow end-to-end - all 675 core tests pass
