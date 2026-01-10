# Erold API Capabilities

> Summary of all Erold API endpoints for CLI implementation.

## Base URL & Authentication

```
Base: https://api.erold.dev/v1

Headers:
  Authorization: Bearer <erold_api_key>
  X-Tenant-ID: <tenant_id>
  Content-Type: application/json
```

---

## API Endpoints Summary

### Projects

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/projects` | List all projects |
| `GET` | `/projects/:id` | Get single project |
| `POST` | `/projects` | Create project |
| `PATCH` | `/projects/:id` | Update project |
| `DELETE` | `/projects/:id` | Delete project |
| `GET` | `/projects/:id/stats` | Get project statistics |

**Project Model:**
```json
{
  "id": "proj_abc123",
  "name": "Backend API",
  "slug": "backend-api",
  "description": "REST API for the application",
  "status": "active",
  "taskCount": 45,
  "completedTasks": 32,
  "createdAt": "2024-01-15T10:30:00Z",
  "updatedAt": "2024-03-20T14:45:00Z"
}
```

**Project Statuses:** `planning`, `active`, `completed`, `archived`

---

### Tasks

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/tasks` | List tasks (with filters) |
| `GET` | `/tasks/:id` | Get single task |
| `POST` | `/projects/:projectId/tasks` | Create task |
| `PATCH` | `/tasks/:id` | Update task |
| `DELETE` | `/tasks/:id` | Delete task |
| `POST` | `/tasks/:id/start` | Start task |
| `POST` | `/tasks/:id/complete` | Complete task |
| `POST` | `/tasks/:id/block` | Block task |
| `POST` | `/tasks/:id/log` | Log time |
| `GET` | `/tasks/:id/comments` | List comments |
| `POST` | `/tasks/:id/comments` | Add comment |
| `GET` | `/tasks/search?q=` | Search tasks |
| `GET` | `/tasks/mine` | Get my tasks |
| `GET` | `/tasks/blocked` | Get blocked tasks |

**Task Model:**
```json
{
  "id": "task_xyz789",
  "title": "Implement OAuth",
  "description": "Add OAuth 2.0 authentication",
  "status": "in_progress",
  "priority": "high",
  "projectId": "proj_abc123",
  "projectName": "Backend API",
  "assignedTo": "user_123",
  "assigneeName": "John Doe",
  "dueDate": "2024-04-01T00:00:00Z",
  "tags": ["auth", "security"],
  "progress": 60,
  "timeEstimate": 16,
  "timeLogged": 10,
  "createdAt": "2024-03-15T09:00:00Z",
  "updatedAt": "2024-03-20T16:30:00Z"
}
```

**Task Statuses:** `todo`, `in_progress`, `done`, `blocked`
**Priorities:** `low`, `medium`, `high`, `critical`

**Query Filters:**
- `projectId` - Filter by project
- `status` - Filter by status
- `priority` - Filter by priority
- `assignee` - Filter by assignee (`me` for current user)
- `limit` / `offset` - Pagination

---

### Knowledge Base

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/knowledge` | List articles |
| `GET` | `/knowledge/:id` | Get article |
| `POST` | `/knowledge` | Create article |
| `PATCH` | `/knowledge/:id` | Update article |
| `DELETE` | `/knowledge/:id` | Delete article |
| `GET` | `/knowledge?search=` | Search knowledge |

**Knowledge Model:**
```json
{
  "id": "know_123",
  "title": "API Rate Limiting Guide",
  "category": "api",
  "content": "# Rate Limiting\n\nOur API uses...",
  "projectId": "proj_abc123",
  "createdAt": "2024-01-15T10:30:00Z",
  "updatedAt": "2024-03-20T14:45:00Z"
}
```

**Categories:**
- `architecture`
- `api`
- `deployment`
- `testing`
- `security`
- `performance`
- `workflow`
- `conventions`
- `troubleshooting`
- `other`

**Query Filters:**
- `category` - Filter by category
- `projectId` - Filter by project
- `scope` - `all`, `global`, `project`, `combined`
- `search` - Full-text search

---

### Vault (Secrets)

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/projects/:projectId/vault` | List secrets (metadata only) |
| `GET` | `/projects/:projectId/vault/:id` | Get secret value |
| `POST` | `/projects/:projectId/vault` | Create secret |
| `PATCH` | `/projects/:projectId/vault/:id` | Update secret |
| `DELETE` | `/projects/:projectId/vault/:id` | Delete secret |

**Vault Entry Model:**
```json
{
  "id": "vault_123",
  "key": "DATABASE_URL",
  "value": "postgres://...",
  "scope": "shared",
  "category": "database",
  "environment": "production",
  "description": "Primary database connection"
}
```

**Scopes:** `personal`, `shared`
**Categories:** `database`, `api`, `cloud`, `service`, `credential`, `other`
**Environments:** `all`, `production`, `staging`, `development`

---

### Context & Dashboard

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/context` | Get AI context (optimized for assistants) |
| `GET` | `/dashboard` | Get dashboard data |
| `GET` | `/stats` | Get statistics |
| `GET` | `/workload` | Get team workload |

**Context Response:**
```json
{
  "success": true,
  "data": {
    // Workspace context optimized for AI
  }
}
```

**Dashboard Response:**
```json
{
  "projectCount": 5,
  "taskCount": 120,
  "openTasks": 45,
  "blockedTasks": 3,
  "myTasks": [...],
  "upcomingDue": [...],
  "recentCompleted": [...]
}
```

---

### Team

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/members` | List team members |
| `GET` | `/members/:uid` | Get member details |

---

## Webhooks

**Events:**
- `task.created` - New task created
- `task.updated` - Task modified
- `task.completed` - Task marked complete
- `task.blocked` - Task blocked
- `project.created` - New project created
- `comment.added` - Comment added to task

**Payload:**
```json
{
  "event": "task.completed",
  "timestamp": "2024-03-20T16:30:00Z",
  "data": {
    "task": { ... },
    "user": { ... }
  }
}
```

---

## Rate Limits

| Plan | Requests/minute | Requests/day |
|------|-----------------|--------------|
| Free | 60 | 1,000 |
| Pro | 300 | 10,000 |
| Team | 1,000 | 100,000 |

**Headers:**
```
X-RateLimit-Limit: 60
X-RateLimit-Remaining: 45
X-RateLimit-Reset: 1702569600
```

---

## Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `UNAUTHORIZED` | 401 | Invalid or missing API key |
| `FORBIDDEN` | 403 | Insufficient permissions |
| `NOT_FOUND` | 404 | Resource not found |
| `VALIDATION_ERROR` | 400 | Invalid request data |
| `RATE_LIMITED` | 429 | Too many requests |
| `SERVER_ERROR` | 500 | Internal server error |

---

## What We Can Do with This API

### For the Workflow Engine:

| Capability | API Endpoints | Use Case |
|------------|---------------|----------|
| **Fetch Context** | `GET /context`, `GET /knowledge`, `GET /tasks` | Phase 1: Preprocessing |
| **Create Plan** | `POST /projects/:id/tasks` | Phase 2: Create task with subtasks |
| **Wait Approval** | `GET /tasks/:id` (poll status) | Phase 2: Check task approval |
| **Track Progress** | `PATCH /tasks/:id` | Phase 3: Update subtask status |
| **Save Learnings** | `POST /knowledge` | Phase 4: Store knowledge |
| **Complete Task** | `POST /tasks/:id/complete` | Phase 4: Mark done |

### Supported (Verified from Source):

| Feature | Status | Details |
|---------|--------|---------|
| **Subtasks** | ✅ Supported | `subtasks: [{id, title, completed, order}]` |
| **Agent Execution** | ✅ Supported | `agentExecution: {status, startedAt, completedAt, error}` |
| **Execution Log** | ✅ Supported | Array of `{timestamp, type, message, percent}` |
| **Tools Used** | ✅ Supported | Array of tool names |
| **Block Reason** | ✅ Supported | `blockReason` field |
| **Completion Summary** | ✅ Supported | `completionSummary` field |
| **Agent Notes** | ✅ Supported | `agentNotes` field |
| **Progress Percent** | ✅ Supported | `progressPercent` (0-100) |

### Agent Execution Model (from firestore-agent-tasks.test.js):

```json
{
  "assigneeType": "agent",
  "agentId": "claude-code",
  "agentName": "Claude Code",
  "agentExecution": {
    "status": "pending | running | completed | failed",
    "startedAt": "2024-03-20T10:00:00Z",
    "completedAt": "2024-03-20T10:30:00Z",
    "error": null
  },
  "executionLog": [
    {"timestamp": "...", "type": "progress", "message": "Reading files", "percent": 25}
  ],
  "toolsUsed": ["read_file", "write_file", "run_command"],
  "progressPercent": 75,
  "agentNotes": "Found existing patterns in src/hooks",
  "blockReason": "Missing API credentials",
  "completionSummary": "Implemented feature X with tests"
}
```

### Missing (Need to Add for Workflow):

| Feature | Current State | Recommendation |
|---------|---------------|----------------|
| **Approval Status** | Uses `analysis` → `todo` flow | Use `analysis` for pending, `todo` for approved, `blocked` for rejected |
| **Knowledge TTL** | No expiration field | Add `expiresAt` to knowledge model (optional - can implement client-side)

---

## CLI API Client Methods

Based on this API, our client should implement:

```rust
impl EroldClient {
    // Projects
    pub async fn list_projects(&self) -> Result<Vec<Project>>;
    pub async fn get_project(&self, id: &str) -> Result<Project>;
    pub async fn create_project(&self, data: &CreateProject) -> Result<Project>;

    // Tasks
    pub async fn list_tasks(&self, filters: &TaskFilters) -> Result<Vec<Task>>;
    pub async fn get_task(&self, id: &str) -> Result<Task>;
    pub async fn create_task(&self, project_id: &str, data: &CreateTask) -> Result<Task>;
    pub async fn update_task(&self, id: &str, data: &UpdateTask) -> Result<Task>;
    pub async fn start_task(&self, id: &str) -> Result<Task>;
    pub async fn complete_task(&self, id: &str, summary: Option<&str>) -> Result<Task>;
    pub async fn block_task(&self, id: &str, reason: &str) -> Result<Task>;

    // Knowledge
    pub async fn list_knowledge(&self, filters: &KnowledgeFilters) -> Result<Vec<Knowledge>>;
    pub async fn get_knowledge(&self, id: &str) -> Result<Knowledge>;
    pub async fn create_knowledge(&self, data: &CreateKnowledge) -> Result<Knowledge>;
    pub async fn search_knowledge(&self, query: &str) -> Result<Vec<Knowledge>>;

    // Context
    pub async fn get_context(&self) -> Result<Context>;
    pub async fn get_dashboard(&self) -> Result<Dashboard>;

    // Vault
    pub async fn list_vault(&self, project_id: &str) -> Result<Vec<VaultEntry>>;
    pub async fn get_secret(&self, project_id: &str, id: &str) -> Result<VaultEntry>;
    pub async fn create_secret(&self, project_id: &str, data: &CreateSecret) -> Result<VaultEntry>;
}
```

---

## Next Steps

1. **Confirm subtasks support** - Does the API support inline subtasks on tasks?
2. **Add approval status** - Need `pending_approval`, `approved`, `rejected` statuses
3. **Add rejection reason** - Need field for why a task was rejected
4. **Add knowledge TTL** - Need `expiresAt` field for knowledge expiration
