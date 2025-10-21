use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::AppState;

pub fn render(frame: &mut Frame<'_>, area: Rect, state: &AppState) {
    let block = Block::default().title("Command").borders(Borders::ALL);
    let value = state.input().value().to_string();
    let mut line = Line::from(value);
    if line.spans.is_empty() {
        line = Line::from(vec![Span::styled(
            "/ to focus commands, Enter to send",
            Style::default().fg(Color::DarkGray),
        )]);
    }
    frame.render_widget(Paragraph::new(line).block(block), area);
}
