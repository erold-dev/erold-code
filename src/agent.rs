//! Agent runtime - Orchestrates the full workflow
//!
//! Ties together all crates:
//! - erold-api: Task/Knowledge management
//! - erold-workflow: State machine, security gates
//! - erold-llm: Claude API
//! - erold-tools: Tool execution
//! - erold-config: Configuration

use std::sync::Arc;
use tokio::sync::RwLock;

use erold_api::EroldClient;
use erold_config::{Credentials, EroldConfig};
use erold_llm::{ChatSession, LlmClient, ContentBlock, Tool, models::StopReason};
use erold_tools::{ToolRegistry, ToolContext, ToolOutput};
use erold_workflow::{
    WorkflowEngine, WorkflowConfig, WorkflowRepository, LiveWorkflowRepository,
    SecurityGate,
};

use crate::ui::{AgentUI, ConsoleUI};

/// Agent configuration
#[derive(Debug, Clone)]
pub struct AgentConfig {
    #[allow(dead_code)]
    pub model: String,
    pub max_tokens: u32,
    pub max_iterations: usize,
    pub require_approval: bool,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            model: "gpt-4o".to_string(),
            max_tokens: 8192,
            max_iterations: 50,
            require_approval: true,
        }
    }
}

impl From<&EroldConfig> for AgentConfig {
    fn from(config: &EroldConfig) -> Self {
        Self {
            model: config.llm.model.clone(),
            max_tokens: config.llm.max_tokens,
            max_iterations: 50,
            require_approval: config.workflow.require_approval,
        }
    }
}

/// The main agent that executes tasks
pub struct Agent<R: WorkflowRepository + 'static> {
    /// Configuration
    config: AgentConfig,
    /// LLM client
    llm: LlmClient,
    /// Tool registry
    tools: ToolRegistry,
    /// Workflow engine
    workflow: Arc<WorkflowEngine<R>>,
    /// Security gate (shared with workflow)
    security: Arc<RwLock<SecurityGate>>,
    /// Working directory
    working_dir: std::path::PathBuf,
    /// Project ID
    project_id: String,
    /// UI handler
    ui: Box<dyn AgentUI>,
}

impl Agent<LiveWorkflowRepository> {
    /// Create an agent from credentials and config
    pub fn from_credentials(
        creds: &Credentials,
        config: &EroldConfig,
        project_id: &str,
        working_dir: std::path::PathBuf,
    ) -> anyhow::Result<Self> {
        // Create API client
        let api_client = EroldClient::new(&config.api.url, &creds.erold_api_key, &creds.tenant_id)?
            .with_project(project_id);

        // Create workflow repository
        let repository = Arc::new(LiveWorkflowRepository::new(
            api_client,
            project_id.to_string(),
        ));

        // Create workflow config (security settings are always on by default)
        let workflow_config = WorkflowConfig::builder()
            .approval_timeout(std::time::Duration::from_secs(config.workflow.approval_timeout_secs))
            .approval_poll_interval(std::time::Duration::from_secs(config.workflow.approval_poll_secs))
            .build();

        // Create workflow engine
        let workflow = Arc::new(
            WorkflowEngine::builder(repository)
                .config(workflow_config)
                .with_logging()
                .build()
        );

        // Create LLM client
        let llm = LlmClient::with_model(&creds.openai_api_key, &config.llm.model)?;

        // Create security gate (shared)
        let security = Arc::new(RwLock::new(SecurityGate::new()));

        // Create tool registry
        let tools = ToolRegistry::with_defaults();

        Ok(Self {
            config: AgentConfig::from(config),
            llm,
            tools,
            workflow,
            security,
            working_dir,
            project_id: project_id.to_string(),
            ui: Box::new(ConsoleUI::new()),
        })
    }

    /// Set custom UI handler
    pub fn with_ui(mut self, ui: Box<dyn AgentUI>) -> Self {
        self.ui = ui;
        self
    }
}

impl<R: WorkflowRepository + 'static> Agent<R> {
    /// Run a task through the full workflow
    pub async fn run(&self, task_description: &str) -> anyhow::Result<()> {
        self.ui.task_started(task_description);

        // Phase 1: Preprocessing
        self.ui.phase_started("Preprocessing");
        self.workflow.start(task_description, &self.project_id).await?;
        self.ui.phase_completed("Preprocessing");

        // Phase 2: Planning - Use LLM to create a plan
        self.ui.phase_started("Planning");
        let plan = self.create_plan(task_description).await?;
        self.ui.plan_created(&plan);

        // Create task with subtasks in API
        let task = self.workflow.create_plan(
            task_description,
            &format!("Task: {task_description}"),
            &self.project_id,
            plan.clone(),
        ).await?;

        // Wait for approval if required
        if self.config.require_approval {
            self.ui.awaiting_approval();

            // For CLI, we'll auto-approve for now (TUI would wait for user input)
            let approved = self.ui.wait_for_approval(&plan).await;

            if approved {
                self.workflow.approve_plan().await?;
                self.ui.plan_approved();
            } else {
                self.workflow.reject_plan(Some("User rejected".to_string())).await?;
                self.ui.plan_rejected("User rejected");
                return Ok(());
            }
        } else {
            // Auto-approve if not required
            self.security.write().await.approve_plan();
        }

        // Phase 3: Execution
        self.ui.phase_started("Execution");
        self.execute_subtasks(&task.subtasks.iter().map(|s| s.title.clone()).collect::<Vec<_>>()).await?;
        self.ui.phase_completed("Execution");

        // Phase 4: Enrichment
        self.ui.phase_started("Enrichment");
        self.workflow.begin_enrichment().await?;
        self.ui.phase_completed("Enrichment");

        self.ui.task_completed();
        Ok(())
    }

    /// Use LLM to create a plan (subtask list)
    async fn create_plan(&self, task_description: &str) -> anyhow::Result<Vec<String>> {
        let system_prompt = r#"You are a software development planning assistant.
Given a task description, create a clear, actionable plan broken into subtasks.

IMPORTANT:
- Each subtask should be a single, focused action
- Subtasks should be in logical order
- Include reading files before modifying them
- Include testing/verification steps

Respond with ONLY a JSON array of subtask titles, nothing else.
Example: ["Read the main.rs file", "Add error handling to parse function", "Run tests to verify"]"#;

        let mut session = ChatSession::new(self.llm.clone())
            .system(system_prompt)
            .max_tokens(1024)
            .temperature(0.0);

        let response = session.send(format!("Create a plan for: {task_description}")).await?;

        // Parse the response as JSON array (strip markdown code fences if present)
        let text = response.text();
        let json_text = text
            .trim()
            .strip_prefix("```json")
            .or_else(|| text.trim().strip_prefix("```"))
            .unwrap_or(&text)
            .trim()
            .strip_suffix("```")
            .unwrap_or(&text)
            .trim();

        let subtasks: Vec<String> = serde_json::from_str(json_text)
            .map_err(|e| anyhow::anyhow!("Failed to parse plan: {e}\nResponse: {text}"))?;

        if subtasks.is_empty() {
            anyhow::bail!("LLM returned empty plan");
        }

        Ok(subtasks)
    }

    /// Execute each subtask using the LLM agent loop
    async fn execute_subtasks(&self, subtasks: &[String]) -> anyhow::Result<()> {
        for (index, subtask_title) in subtasks.iter().enumerate() {
            self.ui.subtask_started(index, subtask_title);

            // Get subtask context from workflow
            let subtask_ctx = self.workflow.start_subtask(index, subtask_title).await?;

            // Build context prompt with relevant knowledge
            let context = self.build_subtask_context(&subtask_ctx);

            // Execute the subtask with agent loop
            self.execute_agent_loop(subtask_title, &context).await?;

            // Mark subtask complete
            self.workflow.complete_subtask(index, subtask_title).await?;
            self.ui.subtask_completed(index, subtask_title);
        }

        Ok(())
    }

    /// Build context string from subtask context
    fn build_subtask_context(&self, ctx: &erold_workflow::SubtaskContext) -> String {
        let mut context = String::new();

        if !ctx.relevant_knowledge.is_empty() {
            context.push_str("## Relevant Knowledge\n\n");
            for k in &ctx.relevant_knowledge {
                context.push_str(&format!("### {}\n{}\n\n", k.title, k.content));
            }
        }

        if !ctx.past_mistakes.is_empty() {
            context.push_str("## Past Mistakes to Avoid\n\n");
            for m in &ctx.past_mistakes {
                context.push_str(&format!("### {}\n{}\n\n", m.title, m.content));
            }
        }

        context
    }

    /// Run the agent loop for a single subtask
    async fn execute_agent_loop(&self, subtask: &str, context: &str) -> anyhow::Result<()> {
        let system_prompt = self.build_system_prompt(context);
        let tool_defs = self.build_tool_definitions();

        let mut session = ChatSession::new(self.llm.clone())
            .system(&system_prompt)
            .tools(tool_defs)
            .max_tokens(self.config.max_tokens)
            .temperature(0.0);

        // Initial message
        session.user(format!("Complete this subtask: {subtask}"));

        let tool_context = ToolContext::new(
            Arc::clone(&self.security),
            self.working_dir.clone(),
        );

        // Agent loop
        for iteration in 0..self.config.max_iterations {
            let response = session.complete().await?;

            // Check for tool use
            let tool_uses: Vec<_> = response.content.iter()
                .filter_map(|c| match c {
                    ContentBlock::ToolUse { id, name, input } => Some((id.clone(), name.clone(), input.clone())),
                    _ => None,
                })
                .collect();

            if tool_uses.is_empty() {
                // No tool use - check if done
                if matches!(response.stop_reason, Some(StopReason::EndTurn)) {
                    self.ui.agent_message(&response.text());
                    break;
                }
            }

            // Execute tools
            let mut tool_results = Vec::new();
            for (id, name, input) in &tool_uses {
                self.ui.tool_started(name);

                let result = self.execute_tool(name, input.clone(), &tool_context).await;

                let (output, is_error) = match result {
                    Ok(output) => {
                        let text = output.to_string();
                        self.ui.tool_completed(name, &text);
                        (text, output.is_error())
                    }
                    Err(e) => {
                        let err = format!("Error: {e}");
                        self.ui.tool_error(name, &err);
                        (err, true)
                    }
                };

                tool_results.push(ContentBlock::ToolResult {
                    tool_use_id: id.clone(),
                    content: output,
                    is_error: Some(is_error),
                });
            }

            // Add tool results to conversation
            session.tool_results(tool_results);

            // Check for stop condition
            if matches!(response.stop_reason, Some(StopReason::EndTurn)) {
                break;
            }

            if iteration >= self.config.max_iterations - 1 {
                self.ui.agent_message("Max iterations reached, moving on");
            }
        }

        Ok(())
    }

    /// Execute a single tool
    async fn execute_tool(
        &self,
        name: &str,
        input: serde_json::Value,
        context: &ToolContext,
    ) -> anyhow::Result<ToolOutput> {
        // Track file reads/edits with workflow engine
        if name == "read_file" {
            if let Some(path) = input.get("path").and_then(|v| v.as_str()) {
                self.workflow.on_file_read(path).await?;
            }
        } else if name == "write_file" || name == "edit_file" {
            if let Some(path) = input.get("path").and_then(|v| v.as_str()) {
                self.workflow.check_file_edit(path).await?;
            }
        }

        let output = self.tools.execute(name, input, context).await?;

        Ok(output)
    }

    /// Build the system prompt for the agent
    fn build_system_prompt(&self, context: &str) -> String {
        format!(r#"You are an expert software developer assistant. You complete tasks by using the available tools.

RULES:
1. ALWAYS read a file before modifying it
2. Make focused, minimal changes
3. Explain what you're doing before each tool use
4. After completing the task, provide a brief summary

WORKING DIRECTORY: {}

{context}

Use the available tools to complete the task. When done, say "Task complete" and summarize what you did."#,
            self.working_dir.display()
        )
    }

    /// Build tool definitions for the LLM
    fn build_tool_definitions(&self) -> Vec<Tool> {
        self.tools.definitions()
            .into_iter()
            .map(|def| Tool {
                name: def.name,
                description: def.description,
                input_schema: def.parameters,
            })
            .collect()
    }
}
