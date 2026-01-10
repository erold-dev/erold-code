//! Chat view
//!
//! Renders the chat interface with message history and input.

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
};

use crate::{App, MessageRole};

/// Render the chat view
pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),    // Messages
            Constraint::Length(3), // Input
        ])
        .split(area);

    render_messages(app, frame, chunks[0]);
    render_input(app, frame, chunks[1]);
}

fn render_messages(app: &App, frame: &mut Frame, area: Rect) {
    let messages_block = Block::default()
        .title(" Chat ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = messages_block.inner(area);
    frame.render_widget(messages_block, area);

    if app.messages.is_empty() {
        let welcome = Paragraph::new("Welcome to Erold! Type a message to get started.")
            .style(Style::default().fg(Color::DarkGray))
            .wrap(Wrap { trim: true });
        frame.render_widget(welcome, inner);
        return;
    }

    // Build message text with styling
    let mut lines: Vec<Line> = Vec::new();
    for msg in &app.messages {
        let (prefix, style) = match msg.role {
            MessageRole::User => ("You: ", Style::default().fg(Color::Green)),
            MessageRole::Assistant => ("Assistant: ", Style::default().fg(Color::Blue)),
            MessageRole::System => ("System: ", Style::default().fg(Color::Yellow)),
        };

        // Split content into lines
        for (i, line) in msg.content.lines().enumerate() {
            if i == 0 {
                lines.push(Line::from(vec![
                    Span::styled(prefix, style.bold()),
                    Span::raw(line),
                ]));
            } else {
                lines.push(Line::from(format!("  {line}")));
            }
        }
        lines.push(Line::from("")); // Empty line between messages
    }

    let messages = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .scroll((app.messages.len().saturating_sub(inner.height as usize) as u16, 0));

    frame.render_widget(messages, inner);

    // Scrollbar
    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
    let mut scrollbar_state = ScrollbarState::new(app.messages.len())
        .position(app.messages.len().saturating_sub(1));
    frame.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
}

fn render_input(app: &App, frame: &mut Frame, area: Rect) {
    let input_block = Block::default()
        .title(" Input ")
        .borders(Borders::ALL)
        .border_style(if app.input.is_empty() {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default().fg(Color::Green)
        });

    let inner = input_block.inner(area);
    frame.render_widget(input_block, area);

    let input = Paragraph::new(app.input.as_str())
        .style(Style::default().fg(Color::White));

    frame.render_widget(input, inner);

    // Show cursor
    frame.set_cursor_position(Position::new(
        inner.x + app.input.len() as u16,
        inner.y,
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_app_renders() {
        let app = App::new();
        assert!(app.messages.is_empty());
    }
}
