use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::AppState;

pub fn render(frame: &mut Frame<'_>, area: Rect, state: &AppState) {
    let mut text = Text::default();
    let active = state.active_profile().ok();
    if let Ok(profiles) = state.execution().load_profiles() {
        text.push_line(Line::from(vec![Span::styled(
            "Profiles",
            Style::default().fg(Color::Green),
        )]));
        for profile in profiles {
            let indicator = if active
                .as_ref()
                .map(|p| p.name == profile.name)
                .unwrap_or(false)
            {
                "*"
            } else {
                " "
            };
            text.push_line(Line::from(format!(
                "{} {} ({})",
                indicator, profile.name, profile.model
            )));
        }
    }
    if let Ok(squads) = state.execution().load_squads() {
        text.push_line(Line::from(""));
        text.push_line(Line::from(vec![Span::styled(
            "Squads",
            Style::default().fg(Color::Blue),
        )]));
        for squad in squads {
            text.push_line(Line::from(format!(
                "- {} ({} members)",
                squad.name,
                squad.members.len()
            )));
        }
    }
    if text.lines.is_empty() {
        text.push_line(Line::from("No profiles configured."));
    }
    let block = Block::default().title("Navigator").borders(Borders::ALL);
    frame.render_widget(Paragraph::new(text).block(block), area);
}
