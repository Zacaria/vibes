use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::AppState;

pub fn render_right(frame: &mut Frame<'_>, area: Rect, state: &AppState) {
    let mut text = Text::default();
    if let Ok(profile) = state.active_profile() {
        text.push_line(Line::from(vec![Span::styled(
            "Profile",
            Style::default().fg(Color::Green),
        )]));
        text.push_line(Line::from(format!("Name: {}", profile.name)));
        text.push_line(Line::from(format!("Model: {}", profile.model)));
        text.push_line(Line::from(format!(
            "Provider: {}",
            profile.provider_string()
        )));
        if let Some(system) = profile.system_prompt {
            text.push_line(Line::from(""));
            text.push_line(Line::from(vec![Span::styled(
                "System",
                Style::default().fg(Color::Blue),
            )]));
            text.push_line(Line::from(system));
        }
    }
    let usage = state.usage();
    text.push_line(Line::from(""));
    text.push_line(Line::from(vec![Span::styled(
        "Usage",
        Style::default().fg(Color::Yellow),
    )]));
    text.push_line(Line::from(format!("Prompt: {}", usage.prompt_tokens)));
    text.push_line(Line::from(format!(
        "Completion: {}",
        usage.completion_tokens
    )));
    text.push_line(Line::from(format!("Total: {}", usage.total_tokens)));

    let block = Block::default().title("Context").borders(Borders::ALL);
    frame.render_widget(Paragraph::new(text).block(block), area);
}

pub fn render_bottom(frame: &mut Frame<'_>, area: Rect, state: &AppState) {
    let block = Block::default().title("Status").borders(Borders::ALL);
    let paragraph = Paragraph::new(Line::from(state.status_line())).block(block);
    frame.render_widget(paragraph, area);
}
