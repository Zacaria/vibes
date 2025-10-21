use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::domain::Post;

pub fn nav_panel() -> Paragraph<'static> {
    let nav_text = Text::from(vec![
        Line::from("Commands:"),
        Line::from("/login email: pw:"),
        Line::from("/post \"text\" audience:public"),
        Line::from("/feed global"),
        Line::from("/tasks add \"title\" \"desc\""),
    ]);
    Paragraph::new(nav_text).block(Block::default().title("Help").borders(Borders::ALL))
}

pub fn feed_panel(posts: &[Post]) -> Paragraph<'_> {
    let feed_lines = posts
        .iter()
        .map(|post| {
            Line::from(vec![
                Span::styled(
                    format!(
                        "@{}",
                        post.author_handle
                            .clone()
                            .unwrap_or_else(|| post.author.to_string())
                    ),
                    Style::default().fg(Color::Cyan),
                ),
                Span::raw(" "),
                Span::raw(post.body.clone()),
                Span::raw(format!(" [{}]", post.audience)),
            ])
        })
        .collect::<Vec<_>>();
    Paragraph::new(feed_lines)
        .wrap(Wrap { trim: true })
        .block(Block::default().title("Feed").borders(Borders::ALL))
}

pub fn status_panel(lines: Vec<String>) -> Paragraph<'static> {
    let status_lines = lines.into_iter().map(Line::from).collect::<Vec<_>>();
    Paragraph::new(status_lines).block(Block::default().title("Status").borders(Borders::ALL))
}

pub fn command_panel<'a>(input: &'a str, editing: bool) -> Paragraph<'a> {
    let style = if editing {
        Style::default().fg(Color::Green)
    } else {
        Style::default()
    };
    Paragraph::new(input.to_string())
        .style(style)
        .block(Block::default().title("Command").borders(Borders::ALL))
}
