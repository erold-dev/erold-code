//! UI abstraction for the agent
//!
//! Allows the agent to communicate progress through different UIs:
//! - ConsoleUI: Simple text output for CLI mode
//! - TuiUI: Rich terminal UI for interactive mode

use std::io::{self, Write};
use async_trait::async_trait;

/// UI trait for agent progress updates
#[async_trait]
pub trait AgentUI: Send + Sync {
    /// Task started
    fn task_started(&self, description: &str);

    /// Phase started
    fn phase_started(&self, phase: &str);

    /// Phase completed
    fn phase_completed(&self, phase: &str);

    /// Plan created
    fn plan_created(&self, subtasks: &[String]);

    /// Awaiting approval
    fn awaiting_approval(&self);

    /// Wait for user to approve/reject the plan
    async fn wait_for_approval(&self, subtasks: &[String]) -> bool;

    /// Plan approved
    fn plan_approved(&self);

    /// Plan rejected
    fn plan_rejected(&self, reason: &str);

    /// Subtask started
    fn subtask_started(&self, index: usize, title: &str);

    /// Subtask completed
    fn subtask_completed(&self, index: usize, title: &str);

    /// Tool execution started
    fn tool_started(&self, name: &str);

    /// Tool completed successfully
    fn tool_completed(&self, name: &str, output: &str);

    /// Tool error
    fn tool_error(&self, name: &str, error: &str);

    /// Agent message (thinking/status)
    fn agent_message(&self, message: &str);

    /// Task completed
    fn task_completed(&self);
}

/// Simple console UI for non-interactive mode
pub struct ConsoleUI {
    verbose: bool,
}

impl ConsoleUI {
    pub fn new() -> Self {
        Self { verbose: true }
    }

    #[allow(dead_code)]
    pub fn quiet() -> Self {
        Self { verbose: false }
    }

    fn print_status(&self, prefix: &str, message: &str) {
        println!("{prefix} {message}");
    }
}

impl Default for ConsoleUI {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AgentUI for ConsoleUI {
    fn task_started(&self, description: &str) {
        println!("\n{}", "=".repeat(60));
        println!("Task: {description}");
        println!("{}\n", "=".repeat(60));
    }

    fn phase_started(&self, phase: &str) {
        self.print_status("[>]", &format!("Starting {phase}..."));
    }

    fn phase_completed(&self, phase: &str) {
        self.print_status("[+]", &format!("{phase} completed"));
    }

    fn plan_created(&self, subtasks: &[String]) {
        println!("\nPlan ({} steps):", subtasks.len());
        for (i, step) in subtasks.iter().enumerate() {
            println!("  {}. {step}", i + 1);
        }
        println!();
    }

    fn awaiting_approval(&self) {
        print!("Approve this plan? [Y/n]: ");
        let _ = io::stdout().flush();
    }

    async fn wait_for_approval(&self, _subtasks: &[String]) -> bool {
        // Read from stdin
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            return false;
        }
        let input = input.trim().to_lowercase();
        input.is_empty() || input == "y" || input == "yes"
    }

    fn plan_approved(&self) {
        self.print_status("[+]", "Plan approved");
    }

    fn plan_rejected(&self, reason: &str) {
        self.print_status("[!]", &format!("Plan rejected: {reason}"));
    }

    fn subtask_started(&self, index: usize, title: &str) {
        println!("\n[{}/...] {title}", index + 1);
    }

    fn subtask_completed(&self, _index: usize, title: &str) {
        if self.verbose {
            self.print_status("  [+]", &format!("Completed: {title}"));
        }
    }

    fn tool_started(&self, name: &str) {
        if self.verbose {
            print!("  -> {name}... ");
            let _ = io::stdout().flush();
        }
    }

    fn tool_completed(&self, _name: &str, output: &str) {
        if self.verbose {
            // Truncate long outputs
            let display = if output.len() > 100 {
                format!("{}...", &output[..100])
            } else {
                output.to_string()
            };
            println!("OK");
            if !display.is_empty() && !display.trim().is_empty() {
                for line in display.lines().take(5) {
                    println!("     {line}");
                }
            }
        }
    }

    fn tool_error(&self, name: &str, error: &str) {
        println!("ERROR");
        eprintln!("  [!] {name}: {error}");
    }

    fn agent_message(&self, message: &str) {
        if self.verbose && !message.is_empty() {
            // Print assistant's message, truncated if very long
            let display = if message.len() > 500 {
                format!("{}...", &message[..500])
            } else {
                message.to_string()
            };
            println!("\n{display}\n");
        }
    }

    fn task_completed(&self) {
        println!("\n{}", "=".repeat(60));
        println!("Task completed successfully!");
        println!("{}", "=".repeat(60));
    }
}

/// Auto-approve UI for non-interactive mode
pub struct AutoApproveUI {
    inner: ConsoleUI,
}

impl AutoApproveUI {
    pub fn new() -> Self {
        Self {
            inner: ConsoleUI::new(),
        }
    }
}

impl Default for AutoApproveUI {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AgentUI for AutoApproveUI {
    fn task_started(&self, description: &str) {
        self.inner.task_started(description);
    }

    fn phase_started(&self, phase: &str) {
        self.inner.phase_started(phase);
    }

    fn phase_completed(&self, phase: &str) {
        self.inner.phase_completed(phase);
    }

    fn plan_created(&self, subtasks: &[String]) {
        self.inner.plan_created(subtasks);
    }

    fn awaiting_approval(&self) {
        println!("[Auto-approving plan]");
    }

    async fn wait_for_approval(&self, _subtasks: &[String]) -> bool {
        true // Always approve
    }

    fn plan_approved(&self) {
        self.inner.plan_approved();
    }

    fn plan_rejected(&self, reason: &str) {
        self.inner.plan_rejected(reason);
    }

    fn subtask_started(&self, index: usize, title: &str) {
        self.inner.subtask_started(index, title);
    }

    fn subtask_completed(&self, index: usize, title: &str) {
        self.inner.subtask_completed(index, title);
    }

    fn tool_started(&self, name: &str) {
        self.inner.tool_started(name);
    }

    fn tool_completed(&self, name: &str, output: &str) {
        self.inner.tool_completed(name, output);
    }

    fn tool_error(&self, name: &str, error: &str) {
        self.inner.tool_error(name, error);
    }

    fn agent_message(&self, message: &str) {
        self.inner.agent_message(message);
    }

    fn task_completed(&self) {
        self.inner.task_completed();
    }
}
