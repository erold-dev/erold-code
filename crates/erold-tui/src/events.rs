//! Event handling
//!
//! Keyboard and terminal events.

use std::time::Duration;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use tokio::sync::mpsc;

/// Application events
#[derive(Debug, Clone)]
pub enum AppEvent {
    /// Keyboard input
    Key(KeyEvent),
    /// Terminal tick for updates
    Tick,
    /// User submitted input
    Submit(String),
    /// User requested quit
    Quit,
    /// Assistant response chunk (for streaming)
    AssistantChunk(String),
    /// Assistant response complete
    AssistantDone,
    /// Tool call started
    ToolStart { name: String, id: String },
    /// Tool call completed
    ToolDone { id: String, result: String },
    /// Plan needs approval
    PlanReady { plan: Vec<String> },
    /// Plan was approved
    PlanApproved,
    /// Plan was rejected
    PlanRejected,
}

/// Event handler that polls for keyboard events
pub struct EventHandler {
    rx: mpsc::UnboundedReceiver<AppEvent>,
}

impl EventHandler {
    /// Create a new event handler
    pub fn new(tick_rate: Duration) -> (Self, mpsc::UnboundedSender<AppEvent>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let event_tx = tx.clone();

        // Spawn event polling task
        tokio::spawn(async move {
            loop {
                if event::poll(tick_rate).unwrap_or(false) {
                    if let Ok(Event::Key(key)) = event::read() {
                        if event_tx.send(AppEvent::Key(key)).is_err() {
                            break;
                        }
                    }
                }
                if event_tx.send(AppEvent::Tick).is_err() {
                    break;
                }
            }
        });

        (Self { rx }, tx)
    }

    /// Get the next event
    pub async fn next(&mut self) -> Option<AppEvent> {
        self.rx.recv().await
    }
}

/// Convert key event to action
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// No action
    None,
    /// Quit the application
    Quit,
    /// Submit current input
    Submit,
    /// Add character to input
    Char(char),
    /// Delete character
    Backspace,
    /// Clear input
    Clear,
    /// Scroll up
    ScrollUp,
    /// Scroll down
    ScrollDown,
    /// Approve plan
    Approve,
    /// Reject plan
    Reject,
}

impl From<KeyEvent> for Action {
    fn from(key: KeyEvent) -> Self {
        match key.code {
            // Quit shortcuts
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::Quit,
            KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::Quit,
            KeyCode::Esc => Action::Quit,

            // Submit
            KeyCode::Enter => Action::Submit,

            // Edit
            KeyCode::Backspace => Action::Backspace,
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::Clear,

            // Scroll
            KeyCode::Up => Action::ScrollUp,
            KeyCode::Down => Action::ScrollDown,
            KeyCode::PageUp => Action::ScrollUp,
            KeyCode::PageDown => Action::ScrollDown,

            // Plan approval (y/n)
            KeyCode::Char('y') | KeyCode::Char('Y') => Action::Approve,
            KeyCode::Char('n') | KeyCode::Char('N') => Action::Reject,

            // Regular characters
            KeyCode::Char(c) => Action::Char(c),

            _ => Action::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_from_key() {
        let key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert_eq!(Action::from(key), Action::Quit);

        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::empty());
        assert_eq!(Action::from(key), Action::Submit);

        let key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty());
        assert_eq!(Action::from(key), Action::Char('a'));
    }
}
