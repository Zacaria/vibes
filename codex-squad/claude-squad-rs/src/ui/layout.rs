use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::Frame;

use crate::app::AppState;

use super::{chat, command, sidebar, status};

pub fn render(frame: &mut Frame<'_>, state: &AppState) {
    let size = frame.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(3)].as_ref())
        .split(size);
    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Length(24),
                Constraint::Min(40),
                Constraint::Length(28),
            ]
            .as_ref(),
        )
        .split(chunks[0]);
    sidebar::render(frame, top[0], state);
    chat::render(frame, top[1], state);
    status::render_right(frame, top[2], state);
    let bottom = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(2)].as_ref())
        .split(chunks[1]);
    status::render_bottom(frame, bottom[0], state);
    command::render(frame, bottom[1], state);
}
