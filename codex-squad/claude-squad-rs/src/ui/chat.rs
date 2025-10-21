use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::AppState;
use crate::domain::MessageRole;

pub fn render(frame: &mut Frame<'_>, area: Rect, state: &AppState) {
    let mut text = Text::default();
    for message in state.messages() {
        let (label, style) = match message.role {
            MessageRole::User => (
                "You",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            MessageRole::Assistant => ("Assistant", Style::default().fg(Color::Yellow)),
            MessageRole::System => ("System", Style::default().fg(Color::Gray)),
            MessageRole::Tool => ("Tool", Style::default().fg(Color::Green)),
        };
        text.push_line(Line::from(vec![Span::styled(label, style)]));
        text.push_line(Line::from(message.body.clone()));
        text.push_line(Line::from(""));
    }
    if text.lines.is_empty() {
        text.push_line(Line::from("Start chatting by typing a message."));
    }
    let block = Block::default().title("Conversation").borders(Borders::ALL);
    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}
