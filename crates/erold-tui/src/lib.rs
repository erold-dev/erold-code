//! Terminal UI for the Erold CLI
//!
//! Provides a rich terminal interface for interacting with the assistant.

mod error;
mod terminal;
mod events;

pub mod views;

pub use error::{TuiError, Result};
pub use terminal::{init, restore, install_panic_hook, Tui};
pub use events::{AppEvent, EventHandler, Action};
pub use views::plan::PlanView;
pub use views::progress::{ProgressView, SubtaskStatus, Subtask};

/// Application state
#[derive(Debug, Default)]
pub struct App {
    /// Whether the app should quit
    pub should_quit: bool,
    /// Current input text
    pub input: String,
    /// Chat history
    pub messages: Vec<ChatMessage>,
    /// Current view mode
    pub mode: ViewMode,
    /// Plan view state (when showing plan)
    pub plan: Option<PlanView>,
    /// Progress view state (when executing)
    pub progress: Option<ProgressView>,
}

/// Current view mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewMode {
    /// Chat interface
    #[default]
    Chat,
    /// Plan approval
    Plan,
    /// Execution progress
    Progress,
}

/// Chat message in the UI
#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
}

/// Message role for display
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

impl App {
    /// Create a new app
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a user message
    pub fn add_user_message(&mut self, content: impl Into<String>) {
        self.messages.push(ChatMessage {
            role: MessageRole::User,
            content: content.into(),
        });
    }

    /// Add an assistant message
    pub fn add_assistant_message(&mut self, content: impl Into<String>) {
        self.messages.push(ChatMessage {
            role: MessageRole::Assistant,
            content: content.into(),
        });
    }

    /// Add a system message
    pub fn add_system_message(&mut self, content: impl Into<String>) {
        self.messages.push(ChatMessage {
            role: MessageRole::System,
            content: content.into(),
        });
    }

    /// Handle a key action
    pub fn handle_action(&mut self, action: Action) {
        match action {
            Action::Quit => self.should_quit = true,
            Action::Char(c) => self.input.push(c),
            Action::Backspace => { self.input.pop(); }
            Action::Clear => self.input.clear(),
            Action::ScrollUp => {
                if let Some(ref mut plan) = self.plan {
                    plan.previous();
                }
            }
            Action::ScrollDown => {
                if let Some(ref mut plan) = self.plan {
                    plan.next();
                }
            }
            _ => {}
        }
    }

    /// Show plan for approval
    pub fn show_plan(&mut self, steps: Vec<String>) {
        self.plan = Some(PlanView::new(steps));
        self.mode = ViewMode::Plan;
    }

    /// Show progress view
    pub fn show_progress(&mut self, task: impl Into<String>, subtasks: Vec<String>) {
        self.progress = Some(ProgressView::new(task, subtasks));
        self.mode = ViewMode::Progress;
    }

    /// Return to chat view
    pub fn show_chat(&mut self) {
        self.mode = ViewMode::Chat;
        self.plan = None;
    }
}

/// Render the current view
pub fn render(app: &App, frame: &mut ratatui::Frame) {
    let area = frame.area();

    match app.mode {
        ViewMode::Chat => views::chat::render(app, frame, area),
        ViewMode::Plan => {
            if let Some(ref plan) = app.plan {
                views::plan::render(plan, frame, area);
            }
        }
        ViewMode::Progress => {
            if let Some(ref progress) = app.progress {
                views::progress::render(progress, frame, area);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_messages() {
        let mut app = App::new();
        assert!(app.messages.is_empty());

        app.add_user_message("Hello");
        assert_eq!(app.messages.len(), 1);
        assert_eq!(app.messages[0].role, MessageRole::User);

        app.add_assistant_message("Hi there!");
        assert_eq!(app.messages.len(), 2);
        assert_eq!(app.messages[1].role, MessageRole::Assistant);
    }

    #[test]
    fn test_app_input() {
        let mut app = App::new();

        app.handle_action(Action::Char('H'));
        app.handle_action(Action::Char('i'));
        assert_eq!(app.input, "Hi");

        app.handle_action(Action::Backspace);
        assert_eq!(app.input, "H");

        app.handle_action(Action::Clear);
        assert!(app.input.is_empty());
    }

    #[test]
    fn test_view_modes() {
        let mut app = App::new();
        assert_eq!(app.mode, ViewMode::Chat);

        app.show_plan(vec!["Step 1".to_string()]);
        assert_eq!(app.mode, ViewMode::Plan);
        assert!(app.plan.is_some());

        app.show_chat();
        assert_eq!(app.mode, ViewMode::Chat);
        assert!(app.plan.is_none());
    }
}
