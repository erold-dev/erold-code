//! Progress view
//!
//! Shows task/subtask progress during execution.

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Gauge, List, ListItem},
};

/// Subtask status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubtaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

/// A subtask in the progress view
#[derive(Debug, Clone)]
pub struct Subtask {
    pub title: String,
    pub status: SubtaskStatus,
}

/// Progress view state
#[derive(Debug, Default)]
pub struct ProgressView {
    /// Task title
    pub task_title: String,
    /// Subtasks
    pub subtasks: Vec<Subtask>,
    /// Current tool being executed
    pub current_tool: Option<String>,
}

impl ProgressView {
    /// Create a new progress view
    #[must_use]
    pub fn new(task_title: impl Into<String>, subtasks: Vec<String>) -> Self {
        Self {
            task_title: task_title.into(),
            subtasks: subtasks
                .into_iter()
                .map(|title| Subtask {
                    title,
                    status: SubtaskStatus::Pending,
                })
                .collect(),
            current_tool: None,
        }
    }

    /// Get completion percentage
    #[must_use]
    pub fn completion_percent(&self) -> u16 {
        if self.subtasks.is_empty() {
            return 0;
        }
        let completed = self
            .subtasks
            .iter()
            .filter(|s| s.status == SubtaskStatus::Completed)
            .count();
        ((completed * 100) / self.subtasks.len()) as u16
    }

    /// Start a subtask
    pub fn start_subtask(&mut self, index: usize) {
        if let Some(subtask) = self.subtasks.get_mut(index) {
            subtask.status = SubtaskStatus::InProgress;
        }
    }

    /// Complete a subtask
    pub fn complete_subtask(&mut self, index: usize) {
        if let Some(subtask) = self.subtasks.get_mut(index) {
            subtask.status = SubtaskStatus::Completed;
        }
    }

    /// Fail a subtask
    pub fn fail_subtask(&mut self, index: usize) {
        if let Some(subtask) = self.subtasks.get_mut(index) {
            subtask.status = SubtaskStatus::Failed;
        }
    }

    /// Set current tool
    pub fn set_tool(&mut self, tool: Option<String>) {
        self.current_tool = tool;
    }
}

/// Render the progress view
pub fn render(progress: &ProgressView, frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Progress bar
            Constraint::Min(5),    // Subtasks
            Constraint::Length(3), // Current action
        ])
        .split(area);

    render_progress_bar(progress, frame, chunks[0]);
    render_subtasks(progress, frame, chunks[1]);
    render_current_action(progress, frame, chunks[2]);
}

fn render_progress_bar(progress: &ProgressView, frame: &mut Frame, area: Rect) {
    let gauge = Gauge::default()
        .block(
            Block::default()
                .title(format!(" {} ", progress.task_title))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .gauge_style(Style::default().fg(Color::Green))
        .percent(progress.completion_percent())
        .label(format!("{}%", progress.completion_percent()));

    frame.render_widget(gauge, area);
}

fn render_subtasks(progress: &ProgressView, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(" Subtasks ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let items: Vec<ListItem> = progress
        .subtasks
        .iter()
        .enumerate()
        .map(|(i, subtask)| {
            let (icon, style) = match subtask.status {
                SubtaskStatus::Pending => ("○", Style::default().fg(Color::DarkGray)),
                SubtaskStatus::InProgress => ("◐", Style::default().fg(Color::Yellow)),
                SubtaskStatus::Completed => ("●", Style::default().fg(Color::Green)),
                SubtaskStatus::Failed => ("✗", Style::default().fg(Color::Red)),
            };

            ListItem::new(format!("{icon} {}. {}", i + 1, subtask.title)).style(style)
        })
        .collect();

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

fn render_current_action(progress: &ProgressView, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(" Current Action ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let text = match &progress.current_tool {
        Some(tool) => format!("Executing: {tool}"),
        None => "Idle".to_string(),
    };

    let paragraph = ratatui::widgets::Paragraph::new(text)
        .block(block)
        .style(Style::default().fg(Color::Cyan));

    frame.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_view() {
        let mut view = ProgressView::new(
            "Test Task",
            vec!["Step 1".to_string(), "Step 2".to_string()],
        );

        assert_eq!(view.completion_percent(), 0);

        view.start_subtask(0);
        assert_eq!(view.subtasks[0].status, SubtaskStatus::InProgress);

        view.complete_subtask(0);
        assert_eq!(view.subtasks[0].status, SubtaskStatus::Completed);
        assert_eq!(view.completion_percent(), 50);

        view.complete_subtask(1);
        assert_eq!(view.completion_percent(), 100);
    }
}
