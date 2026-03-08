//! Workflow context types
//!
//! Immutable context objects passed between phases.

use erold_api::{Knowledge, Task, Decision, Guideline};
use std::collections::HashSet;
use std::path::PathBuf;

/// Context from task-level preprocessing
#[derive(Debug, Clone, Default)]
pub struct PreprocessedContext {
    /// Knowledge articles relevant to the task
    pub relevant_knowledge: Vec<Knowledge>,
    /// Related tasks for context
    pub relevant_tasks: Vec<Task>,
    /// Past mistakes to avoid (troubleshooting category)
    pub past_mistakes: Vec<Knowledge>,
    /// Extracted keywords for the task
    pub keywords: Vec<String>,
    /// Coding guidelines from erold.dev
    pub guidelines: Vec<Guideline>,
}

impl PreprocessedContext {
    /// Create an empty context
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    /// Check if context has any knowledge
    #[must_use]
    pub fn has_knowledge(&self) -> bool {
        !self.relevant_knowledge.is_empty()
    }

    /// Check if there are past mistakes to avoid
    #[must_use]
    pub fn has_past_mistakes(&self) -> bool {
        !self.past_mistakes.is_empty()
    }

    /// Check if there are guidelines available
    #[must_use]
    pub fn has_guidelines(&self) -> bool {
        !self.guidelines.is_empty()
    }

    /// Get total knowledge count
    #[must_use]
    pub fn knowledge_count(&self) -> usize {
        self.relevant_knowledge.len() + self.past_mistakes.len()
    }

    /// Get guidelines count
    #[must_use]
    pub fn guidelines_count(&self) -> usize {
        self.guidelines.len()
    }
}

/// Context for a specific subtask
#[derive(Debug, Clone)]
pub struct SubtaskContext {
    /// Subtask index (0-based)
    pub index: usize,
    /// Subtask title
    pub title: String,
    /// Keywords extracted from subtask
    pub keywords: Vec<String>,
    /// Knowledge relevant to this subtask
    pub relevant_knowledge: Vec<Knowledge>,
    /// Past mistakes relevant to this subtask
    pub past_mistakes: Vec<Knowledge>,
    /// Coding guidelines relevant to this subtask
    pub guidelines: Vec<Guideline>,
    /// IDs of knowledge used (for tracking)
    pub knowledge_used_ids: Vec<String>,
}

impl SubtaskContext {
    /// Create a new subtask context
    #[must_use]
    pub fn new(index: usize, title: impl Into<String>) -> Self {
        Self {
            index,
            title: title.into(),
            keywords: Vec::new(),
            relevant_knowledge: Vec::new(),
            past_mistakes: Vec::new(),
            guidelines: Vec::new(),
            knowledge_used_ids: Vec::new(),
        }
    }

    /// Record that a knowledge article was used
    pub fn mark_knowledge_used(&mut self, knowledge_id: impl Into<String>) {
        self.knowledge_used_ids.push(knowledge_id.into());
    }

    /// Build system prompt additions from context
    #[must_use]
    pub fn build_prompt_additions(&self) -> String {
        let mut additions = String::new();

        // Add coding guidelines (highest priority - these are rules to follow)
        if !self.guidelines.is_empty() {
            additions.push_str("\n📋 CODING GUIDELINES (MUST FOLLOW):\n");
            for guideline in &self.guidelines {
                // Use AI metadata if available for concise prompt
                if let Some(ref ai) = guideline.ai {
                    additions.push_str(&format!(
                        "- [{}] **{}**: {}\n",
                        format!("{:?}", ai.priority).to_uppercase(),
                        guideline.title,
                        ai.prompt_snippet
                    ));
                } else if let Some(ref desc) = guideline.description {
                    additions.push_str(&format!("- **{}**: {}\n", guideline.title, desc));
                } else {
                    additions.push_str(&format!("- **{}**\n", guideline.title));
                }
            }
        }

        // Add warnings about past mistakes
        if !self.past_mistakes.is_empty() {
            additions.push_str("\n⚠️ AVOID THESE PAST MISTAKES:\n");
            for mistake in &self.past_mistakes {
                additions.push_str(&format!("- **{}**\n", mistake.title));
                // Truncate long content
                let content = if mistake.content.len() > 500 {
                    format!("{}...", &mistake.content[..500])
                } else {
                    mistake.content.clone()
                };
                additions.push_str(&format!("  {}\n", content.replace('\n', "\n  ")));
            }
        }

        // Add relevant knowledge
        if !self.relevant_knowledge.is_empty() {
            additions.push_str("\n📚 RELEVANT KNOWLEDGE:\n");
            for knowledge in &self.relevant_knowledge {
                additions.push_str(&format!("- **{}** [{:?}]\n", knowledge.title, knowledge.category));
                // Truncate long content
                let content = if knowledge.content.len() > 500 {
                    format!("{}...", &knowledge.content[..500])
                } else {
                    knowledge.content.clone()
                };
                additions.push_str(&format!("  {}\n", content.replace('\n', "\n  ")));
            }
        }

        additions
    }
}

/// Execution context tracking state during execution
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// Task ID being executed
    pub task_id: String,
    /// Task title
    pub task_title: String,
    /// Project ID
    pub project_id: String,
    /// Files that have been read (for security gate)
    read_files: HashSet<PathBuf>,
    /// Files that have been modified
    modified_files: HashSet<PathBuf>,
    /// Whether plan has been approved
    plan_approved: bool,
    /// Current subtask index
    pub current_subtask: usize,
    /// Total subtasks
    pub total_subtasks: usize,
    /// Collected decisions
    pub decisions: Vec<Decision>,
    /// Collected learnings
    pub learnings: Vec<Learning>,
    /// Collected mistakes
    pub mistakes: Vec<Mistake>,
}

/// A learning discovered during execution
#[derive(Debug, Clone)]
pub struct Learning {
    pub title: String,
    pub content: String,
    pub category: String,
    pub tags: Vec<String>,
}

/// A mistake made during execution
#[derive(Debug, Clone)]
pub struct Mistake {
    pub what_failed: String,
    pub problem: String,
    pub wrong_approach: String,
    pub why_failed: String,
    pub correct_approach: String,
    pub tags: Vec<String>,
}

impl ExecutionContext {
    /// Create a new execution context
    #[must_use]
    pub fn new(
        task_id: impl Into<String>,
        task_title: impl Into<String>,
        project_id: impl Into<String>,
        total_subtasks: usize,
    ) -> Self {
        Self {
            task_id: task_id.into(),
            task_title: task_title.into(),
            project_id: project_id.into(),
            read_files: HashSet::new(),
            modified_files: HashSet::new(),
            plan_approved: false,
            current_subtask: 0,
            total_subtasks,
            decisions: Vec::new(),
            learnings: Vec::new(),
            mistakes: Vec::new(),
        }
    }

    /// Mark plan as approved
    pub fn approve_plan(&mut self) {
        self.plan_approved = true;
    }

    /// Check if plan is approved
    #[must_use]
    pub fn is_plan_approved(&self) -> bool {
        self.plan_approved
    }

    /// Record that a file was read
    pub fn record_file_read(&mut self, path: PathBuf) {
        self.read_files.insert(path);
    }

    /// Check if a file can be edited (must be read first)
    #[must_use]
    pub fn can_edit_file(&self, path: &PathBuf) -> bool {
        self.read_files.contains(path)
    }

    /// Record that a file was modified
    pub fn record_file_modified(&mut self, path: PathBuf) {
        self.modified_files.insert(path);
    }

    /// Get count of read files
    #[must_use]
    pub fn read_files_count(&self) -> usize {
        self.read_files.len()
    }

    /// Get count of modified files
    #[must_use]
    pub fn modified_files_count(&self) -> usize {
        self.modified_files.len()
    }

    /// Add a decision
    pub fn add_decision(&mut self, decision: Decision) {
        self.decisions.push(decision);
    }

    /// Add a learning
    pub fn add_learning(&mut self, learning: Learning) {
        self.learnings.push(learning);
    }

    /// Add a mistake
    pub fn add_mistake(&mut self, mistake: Mistake) {
        self.mistakes.push(mistake);
    }

    /// Advance to next subtask
    pub fn advance_subtask(&mut self) {
        if self.current_subtask < self.total_subtasks {
            self.current_subtask += 1;
        }
    }

    /// Calculate progress percentage
    #[must_use]
    pub fn progress_percent(&self) -> u8 {
        if self.total_subtasks == 0 {
            return 100;
        }
        let progress = (self.current_subtask as f64 / self.total_subtasks as f64) * 100.0;
        progress.round() as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_context_file_tracking() {
        let mut ctx = ExecutionContext::new("task_1", "Test", "proj_1", 3);

        let file = PathBuf::from("/src/main.rs");

        // Can't edit before reading
        assert!(!ctx.can_edit_file(&file));

        // Read the file
        ctx.record_file_read(file.clone());

        // Now can edit
        assert!(ctx.can_edit_file(&file));
    }

    #[test]
    fn test_execution_context_progress() {
        let mut ctx = ExecutionContext::new("task_1", "Test", "proj_1", 4);

        assert_eq!(ctx.progress_percent(), 0);

        ctx.advance_subtask();
        assert_eq!(ctx.progress_percent(), 25);

        ctx.advance_subtask();
        assert_eq!(ctx.progress_percent(), 50);

        ctx.advance_subtask();
        ctx.advance_subtask();
        assert_eq!(ctx.progress_percent(), 100);
    }

    #[test]
    fn test_subtask_context_prompt() {
        let mut ctx = SubtaskContext::new(0, "Add authentication");

        ctx.past_mistakes.push(Knowledge {
            id: "k1".to_string(),
            title: "Don't use MD5 for passwords".to_string(),
            content: "MD5 is insecure for password hashing.".to_string(),
            category: erold_api::KnowledgeCategory::Troubleshooting,
            tags: vec!["security".to_string()],
            project_id: None,
            source: None,
            agent_id: None,
            agent_name: None,
            ttl_days: None,
            source_url: None,
            source_type: None,
            last_refreshed_at: None,
            auto_refresh: false,
            created_at: None,
            updated_at: None,
            created_by: None,
        });

        let prompt = ctx.build_prompt_additions();
        assert!(prompt.contains("AVOID THESE PAST MISTAKES"));
        assert!(prompt.contains("MD5"));
    }
}
