//! Plan approval view
//!
//! Shows the proposed plan and allows user to approve or reject.

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

/// Plan state for approval
#[derive(Debug, Default)]
pub struct PlanView {
    /// Plan steps
    pub steps: Vec<String>,
    /// Currently selected step
    pub selected: usize,
    /// Whether waiting for approval
    pub awaiting_approval: bool,
}

impl PlanView {
    /// Create a new plan view
    #[must_use]
    pub fn new(steps: Vec<String>) -> Self {
        Self {
            steps,
            selected: 0,
            awaiting_approval: true,
        }
    }

    /// Select next step
    pub fn next(&mut self) {
        if self.selected < self.steps.len().saturating_sub(1) {
            self.selected += 1;
        }
    }

    /// Select previous step
    pub fn previous(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }
}

/// Render the plan view
pub fn render(plan: &PlanView, frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),    // Plan steps
            Constraint::Length(3), // Instructions
        ])
        .split(area);

    render_steps(plan, frame, chunks[0]);
    render_instructions(frame, chunks[1]);
}

fn render_steps(plan: &PlanView, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(" Proposed Plan - Review Required ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let items: Vec<ListItem> = plan
        .steps
        .iter()
        .enumerate()
        .map(|(i, step)| {
            let style = if i == plan.selected {
                Style::default().fg(Color::Yellow).bold()
            } else {
                Style::default().fg(Color::White)
            };

            let prefix = if i == plan.selected { "▶ " } else { "  " };
            ListItem::new(format!("{prefix}{i}. {step}")).style(style)
        })
        .collect();

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

fn render_instructions(frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let instructions = Paragraph::new(Line::from(vec![
        Span::styled("[Y]", Style::default().fg(Color::Green).bold()),
        Span::raw(" Approve  "),
        Span::styled("[N]", Style::default().fg(Color::Red).bold()),
        Span::raw(" Reject  "),
        Span::styled("[↑/↓]", Style::default().fg(Color::Cyan)),
        Span::raw(" Navigate"),
    ]))
    .block(block)
    .alignment(Alignment::Center);

    frame.render_widget(instructions, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_navigation() {
        let mut plan = PlanView::new(vec![
            "Step 1".to_string(),
            "Step 2".to_string(),
            "Step 3".to_string(),
        ]);

        assert_eq!(plan.selected, 0);

        plan.next();
        assert_eq!(plan.selected, 1);

        plan.next();
        assert_eq!(plan.selected, 2);

        plan.next(); // Should not go beyond last
        assert_eq!(plan.selected, 2);

        plan.previous();
        assert_eq!(plan.selected, 1);
    }
}
