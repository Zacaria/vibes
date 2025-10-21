use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::app::{AppState, InputMode};

pub fn render(f: &mut Frame<'_>, app: &AppState) {
    let size = f.size();

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(3)].as_ref())
        .split(size);

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(20),
                Constraint::Percentage(55),
                Constraint::Percentage(25),
            ]
            .as_ref(),
        )
        .split(vertical[0]);

    draw_nav(f, columns[0], app);
    draw_feed(f, columns[1], app);
    draw_tasks(f, columns[2], app);
    draw_command(f, vertical[1], app);
}

fn draw_nav(f: &mut Frame<'_>, area: Rect, _app: &AppState) {
    let items = vec![
        ListItem::new("/login email: pw:"),
        ListItem::new("/passkey"),
        ListItem::new("/post \"hello\" audience:public"),
        ListItem::new("/feed [global|following|me]"),
        ListItem::new("/tasks add/ls/done"),
        ListItem::new("Esc toggles command mode"),
    ];
    let list = List::new(items)
        .block(Block::default().title("Commands").borders(Borders::ALL))
        .style(Style::default().fg(Color::Gray));
    f.render_widget(list, area);
}

fn draw_feed(f: &mut Frame<'_>, area: Rect, app: &AppState) {
    let mut items = Vec::new();
    for post in &app.feed {
        let header = post
            .author_handle
            .as_ref()
            .map(|h| format!("@{}", h))
            .unwrap_or_else(|| post.author.to_string());
        let text = format!("{}\n{}\n{}", header, post.body, post.created_at);
        items.push(ListItem::new(text));
    }
    if items.is_empty() {
        items.push(ListItem::new("No posts yet. Try /feed or /post."));
    }
    let block = Block::default()
        .title(format!("Feed ({})", app.feed_filter.as_str()))
        .borders(Borders::ALL);
    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().bg(Color::Blue).fg(Color::Black));
    f.render_widget(list, area);
}

fn draw_tasks(f: &mut Frame<'_>, area: Rect, app: &AppState) {
    let mut items = Vec::new();
    for task in &app.tasks {
        let status = format!("[#{}] {} ({})", task.id, task.title, task.status);
        items.push(ListItem::new(status));
    }
    if items.is_empty() {
        items.push(ListItem::new("No tasks. Use /tasks add."));
    }
    let block = Block::default()
        .title("Project Tasks")
        .borders(Borders::ALL);
    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

fn draw_command(f: &mut Frame<'_>, area: Rect, app: &AppState) {
    let prompt = match app.mode {
        InputMode::Command => ">",
        InputMode::Navigation => "⟲",
    };
    let text = format!("{} {}", prompt, app.command_input);
    let paragraph = Paragraph::new(text)
        .style(match app.mode {
            InputMode::Command => Style::default().fg(Color::Yellow),
            InputMode::Navigation => Style::default().fg(Color::Gray),
        })
        .block(
            Block::default()
                .title(app.status_line.clone())
                .borders(Borders::ALL),
        )
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}
