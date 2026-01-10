//! Workflow repository (Repository pattern)
//!
//! Abstract data access layer for workflow operations.
//! Enables testing with mock implementations.

use async_trait::async_trait;
use erold_api::{Knowledge, Task, CreateKnowledge, CreateTask, UpdateTask, UpdateKnowledge, Guideline, GuidelinesFilter};
use crate::error::Result;

/// Repository trait for workflow data access
///
/// Abstracts all external data operations, enabling:
/// - Unit testing with mock implementations
/// - Swapping storage backends
/// - Caching layer insertion
#[async_trait]
pub trait WorkflowRepository: Send + Sync {
    // =========================================================================
    // Knowledge Operations
    // =========================================================================

    /// Search for relevant knowledge
    async fn search_knowledge(&self, query: &str) -> Result<Vec<Knowledge>>;

    /// Get knowledge by ID
    async fn get_knowledge(&self, id: &str) -> Result<Option<Knowledge>>;

    /// Save new knowledge (learning)
    async fn save_knowledge(&self, knowledge: &CreateKnowledge) -> Result<Knowledge>;

    /// Update existing knowledge
    async fn update_knowledge(&self, id: &str, update: &UpdateKnowledge) -> Result<Knowledge>;

    /// Get expired knowledge that needs refresh
    async fn get_expired_knowledge(&self) -> Result<Vec<Knowledge>>;

    // =========================================================================
    // Task Operations
    // =========================================================================

    /// Get task by ID
    async fn get_task(&self, id: &str) -> Result<Option<Task>>;

    /// Create a new task with subtasks
    async fn create_task(&self, project_id: &str, task: &CreateTask) -> Result<Task>;

    /// Update task
    async fn update_task(&self, id: &str, update: &UpdateTask) -> Result<Task>;

    /// Start task (set to in-progress)
    async fn start_task(&self, id: &str) -> Result<Task>;

    /// Complete a task
    async fn complete_task(&self, id: &str, summary: Option<&str>) -> Result<Task>;

    /// Block a task
    async fn block_task(&self, id: &str, reason: &str) -> Result<Task>;

    // =========================================================================
    // Guidelines Operations
    // =========================================================================

    /// Fetch coding guidelines from erold.dev
    async fn fetch_guidelines(&self, filter: Option<GuidelinesFilter>) -> Result<Vec<Guideline>>;

    /// Fetch guidelines by topic (e.g., "frontend", "backend", "security")
    async fn fetch_guidelines_by_topic(&self, topic: &str) -> Result<Vec<Guideline>>;
}

/// Live implementation using Erold API client
pub struct LiveWorkflowRepository {
    client: erold_api::EroldClient,
    guidelines_client: erold_api::GuidelinesClient,
    project_id: String,
}

impl LiveWorkflowRepository {
    /// Create a new live repository
    ///
    /// # Errors
    /// Returns error if guidelines client creation fails
    pub fn new(client: erold_api::EroldClient, project_id: impl Into<String>) -> Result<Self> {
        let guidelines_client = erold_api::GuidelinesClient::new()?;
        Ok(Self {
            client,
            guidelines_client,
            project_id: project_id.into(),
        })
    }

    /// Get the project ID
    #[must_use]
    pub fn project_id(&self) -> &str {
        &self.project_id
    }
}

#[async_trait]
impl WorkflowRepository for LiveWorkflowRepository {
    async fn search_knowledge(&self, query: &str) -> Result<Vec<Knowledge>> {
        let results = self.client.search_knowledge(query).await?;
        Ok(results)
    }

    async fn get_knowledge(&self, id: &str) -> Result<Option<Knowledge>> {
        match self.client.get_knowledge(id).await {
            Ok(k) => Ok(Some(k)),
            Err(erold_api::ApiError::NotFound(_)) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn save_knowledge(&self, knowledge: &CreateKnowledge) -> Result<Knowledge> {
        let created = self.client.create_knowledge(knowledge).await?;
        Ok(created)
    }

    async fn update_knowledge(&self, id: &str, update: &UpdateKnowledge) -> Result<Knowledge> {
        let updated = self.client.update_knowledge(id, update).await?;
        Ok(updated)
    }

    async fn get_expired_knowledge(&self) -> Result<Vec<Knowledge>> {
        // Get all knowledge and filter expired ones client-side
        let all = self.client.list_knowledge(None, Some(&self.project_id)).await?;
        let expired: Vec<Knowledge> = all
            .into_iter()
            .filter(|k| k.is_expired())
            .collect();
        Ok(expired)
    }

    async fn get_task(&self, id: &str) -> Result<Option<Task>> {
        match self.client.get_task(id).await {
            Ok(t) => Ok(Some(t)),
            Err(erold_api::ApiError::NotFound(_)) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn create_task(&self, project_id: &str, task: &CreateTask) -> Result<Task> {
        let created = self.client.create_task(project_id, task).await?;
        Ok(created)
    }

    async fn update_task(&self, id: &str, update: &UpdateTask) -> Result<Task> {
        let updated = self.client.update_task(id, update).await?;
        Ok(updated)
    }

    async fn start_task(&self, id: &str) -> Result<Task> {
        let started = self.client.start_task(id).await?;
        Ok(started)
    }

    async fn complete_task(&self, id: &str, summary: Option<&str>) -> Result<Task> {
        let completed = self.client.complete_task(id, summary).await?;
        Ok(completed)
    }

    async fn block_task(&self, id: &str, reason: &str) -> Result<Task> {
        let blocked = self.client.block_task(id, reason).await?;
        Ok(blocked)
    }

    async fn fetch_guidelines(&self, filter: Option<GuidelinesFilter>) -> Result<Vec<Guideline>> {
        let guidelines = self.guidelines_client.fetch(filter).await?;
        Ok(guidelines)
    }

    async fn fetch_guidelines_by_topic(&self, topic: &str) -> Result<Vec<Guideline>> {
        let guidelines = self.guidelines_client.fetch_by_topic(topic).await?;
        Ok(guidelines)
    }
}

/// In-memory repository for testing
#[cfg(test)]
pub mod testing {
    use super::*;
    use erold_api::{TaskStatus, TaskPriority, Subtask, CreateSubtask};
    use std::collections::HashMap;
    use std::sync::RwLock;

    pub struct InMemoryRepository {
        knowledge: RwLock<HashMap<String, Knowledge>>,
        tasks: RwLock<HashMap<String, Task>>,
        next_id: RwLock<usize>,
    }

    impl InMemoryRepository {
        pub fn new() -> Self {
            Self {
                knowledge: RwLock::new(HashMap::new()),
                tasks: RwLock::new(HashMap::new()),
                next_id: RwLock::new(1),
            }
        }

        fn next_id(&self) -> String {
            let mut id = self.next_id.write().unwrap();
            let current = *id;
            *id += 1;
            format!("test_{}", current)
        }

        pub fn with_knowledge(self, knowledge: Vec<Knowledge>) -> Self {
            let mut map = self.knowledge.write().unwrap();
            for k in knowledge {
                map.insert(k.id.clone(), k);
            }
            drop(map);
            self
        }
    }

    impl Default for InMemoryRepository {
        fn default() -> Self {
            Self::new()
        }
    }

    #[async_trait]
    impl WorkflowRepository for InMemoryRepository {
        async fn search_knowledge(&self, query: &str) -> Result<Vec<Knowledge>> {
            let map = self.knowledge.read().unwrap();
            let query_lower = query.to_lowercase();

            let results: Vec<Knowledge> = map
                .values()
                .filter(|k| {
                    query.is_empty()
                        || k.title.to_lowercase().contains(&query_lower)
                        || k.content.to_lowercase().contains(&query_lower)
                })
                .take(50)
                .cloned()
                .collect();

            Ok(results)
        }

        async fn get_knowledge(&self, id: &str) -> Result<Option<Knowledge>> {
            let map = self.knowledge.read().unwrap();
            Ok(map.get(id).cloned())
        }

        async fn save_knowledge(&self, create: &CreateKnowledge) -> Result<Knowledge> {
            let mut map = self.knowledge.write().unwrap();
            let id = self.next_id();
            let knowledge = Knowledge {
                id: id.clone(),
                title: create.title.clone(),
                content: create.content.clone(),
                category: create.category.clone(),
                tags: create.tags.clone(),
                project_id: create.project_id.clone(),
                source: create.source.clone(),
                agent_id: None,
                agent_name: None,
                ttl_days: create.ttl_days,
                source_url: create.source_url.clone(),
                source_type: create.source_type.clone(),
                last_refreshed_at: None,
                auto_refresh: create.auto_refresh,
                created_at: Some(chrono::Utc::now()),
                updated_at: Some(chrono::Utc::now()),
                created_by: None,
            };
            map.insert(id, knowledge.clone());
            Ok(knowledge)
        }

        async fn update_knowledge(&self, id: &str, update: &UpdateKnowledge) -> Result<Knowledge> {
            let mut map = self.knowledge.write().unwrap();
            if let Some(k) = map.get_mut(id) {
                if let Some(ref title) = update.title {
                    k.title = title.clone();
                }
                if let Some(ref content) = update.content {
                    k.content = content.clone();
                }
                if let Some(ref category) = update.category {
                    k.category = category.clone();
                }
                if let Some(ref tags) = update.tags {
                    k.tags = tags.clone();
                }
                if let Some(ref last_refreshed_at) = update.last_refreshed_at {
                    k.last_refreshed_at = Some(*last_refreshed_at);
                }
                k.updated_at = Some(chrono::Utc::now());
                Ok(k.clone())
            } else {
                Err(crate::error::WorkflowError::KnowledgeNotFound {
                    knowledge_id: id.to_string(),
                })
            }
        }

        async fn get_expired_knowledge(&self) -> Result<Vec<Knowledge>> {
            let map = self.knowledge.read().unwrap();
            let expired: Vec<Knowledge> = map
                .values()
                .filter(|k| k.is_expired())
                .cloned()
                .collect();
            Ok(expired)
        }

        async fn get_task(&self, id: &str) -> Result<Option<Task>> {
            let map = self.tasks.read().unwrap();
            Ok(map.get(id).cloned())
        }

        async fn create_task(&self, project_id: &str, create: &CreateTask) -> Result<Task> {
            let mut map = self.tasks.write().unwrap();
            let id = self.next_id();
            let task = Task {
                id: id.clone(),
                title: create.title.clone(),
                description: create.description.clone(),
                project_id: project_id.to_string(),
                project_title: None,
                status: create.status.clone().unwrap_or(TaskStatus::Todo),
                priority: create.priority.clone().unwrap_or(TaskPriority::Medium),
                assigned_to: None,
                assignee_type: create.assignee_type.clone(),
                agent_id: create.agent_id.clone(),
                agent_name: create.agent_name.clone(),
                agent_execution: None,
                execution_log: Vec::new(),
                tools_used: Vec::new(),
                progress_percent: Some(0),
                subtasks: create.subtasks.iter().enumerate().map(|(i, s)| {
                    Subtask {
                        id: format!("{}_{}", id, i),
                        title: s.title.clone(),
                        completed: s.completed,
                        order: s.order,
                        keywords: Vec::new(),
                        knowledge_used: Vec::new(),
                        notes: None,
                        decisions: Vec::new(),
                    }
                }).collect(),
                block_reason: None,
                blocked_by: Vec::new(),
                completion_summary: None,
                agent_notes: None,
                tags: create.tags.clone(),
                due_date: None,
                estimated_hours: None,
                actual_hours: None,
                created_at: Some(chrono::Utc::now()),
                updated_at: Some(chrono::Utc::now()),
                created_by: None,
            };
            map.insert(id, task.clone());
            Ok(task)
        }

        async fn update_task(&self, id: &str, update: &UpdateTask) -> Result<Task> {
            let mut map = self.tasks.write().unwrap();
            if let Some(t) = map.get_mut(id) {
                if let Some(ref title) = update.title {
                    t.title = title.clone();
                }
                if let Some(ref description) = update.description {
                    t.description = Some(description.clone());
                }
                if let Some(ref status) = update.status {
                    t.status = status.clone();
                }
                if let Some(ref priority) = update.priority {
                    t.priority = priority.clone();
                }
                if let Some(progress) = update.progress_percent {
                    t.progress_percent = Some(progress);
                }
                t.updated_at = Some(chrono::Utc::now());
                Ok(t.clone())
            } else {
                Err(crate::error::WorkflowError::TaskNotFound {
                    task_id: id.to_string(),
                })
            }
        }

        async fn start_task(&self, id: &str) -> Result<Task> {
            let mut map = self.tasks.write().unwrap();
            if let Some(t) = map.get_mut(id) {
                t.status = TaskStatus::InProgress;
                t.updated_at = Some(chrono::Utc::now());
                Ok(t.clone())
            } else {
                Err(crate::error::WorkflowError::TaskNotFound {
                    task_id: id.to_string(),
                })
            }
        }

        async fn complete_task(&self, id: &str, summary: Option<&str>) -> Result<Task> {
            let mut map = self.tasks.write().unwrap();
            if let Some(t) = map.get_mut(id) {
                t.status = TaskStatus::Done;
                t.completion_summary = summary.map(String::from);
                t.updated_at = Some(chrono::Utc::now());
                Ok(t.clone())
            } else {
                Err(crate::error::WorkflowError::TaskNotFound {
                    task_id: id.to_string(),
                })
            }
        }

        async fn block_task(&self, id: &str, reason: &str) -> Result<Task> {
            let mut map = self.tasks.write().unwrap();
            if let Some(t) = map.get_mut(id) {
                t.status = TaskStatus::Blocked;
                t.block_reason = Some(reason.to_string());
                t.updated_at = Some(chrono::Utc::now());
                Ok(t.clone())
            } else {
                Err(crate::error::WorkflowError::TaskNotFound {
                    task_id: id.to_string(),
                })
            }
        }

        async fn fetch_guidelines(&self, _filter: Option<GuidelinesFilter>) -> Result<Vec<Guideline>> {
            // Return empty guidelines for testing
            // Tests can mock specific guidelines if needed
            Ok(Vec::new())
        }

        async fn fetch_guidelines_by_topic(&self, _topic: &str) -> Result<Vec<Guideline>> {
            Ok(Vec::new())
        }
    }
}
