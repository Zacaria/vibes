use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::Frame;

use crate::domain::Post;
use crate::ui;

pub struct AppState {
    input: String,
    editing: bool,
    status: Vec<String>,
    feed: Vec<Post>,
    size: Rect,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            editing: false,
            status: vec!["Press / to enter command mode".to_string()],
            feed: Vec::new(),
            size: Rect::default(),
        }
    }

    pub fn draw(&mut self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Length(3)].as_ref())
            .split(f.size());
        self.size = f.size();
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Length(25),
                    Constraint::Min(40),
                    Constraint::Length(25),
                ]
                .as_ref(),
            )
            .split(chunks[0]);

        let nav = ui::nav_panel();
        f.render_widget(nav, main_chunks[0]);

        let feed = ui::feed_panel(&self.feed);
        f.render_widget(feed, main_chunks[1]);

        let status_lines = self
            .status
            .iter()
            .rev()
            .take(10)
            .cloned()
            .collect::<Vec<_>>();
        let status = ui::status_panel(status_lines);
        f.render_widget(status, main_chunks[2]);

        let input = ui::command_panel(&self.input, self.editing);
        f.render_widget(input, chunks[1]);
    }

    pub fn toggle_input_mode(&mut self) {
        self.editing = !self.editing;
        if !self.editing {
            self.input.clear();
        }
    }

    pub fn start_editing(&mut self) {
        self.editing = true;
        self.input.clear();
    }

    pub fn push_char(&mut self, c: char) {
        self.input.push(c);
    }

    pub fn backspace(&mut self) {
        self.input.pop();
    }

    pub fn take_command(&mut self) -> Option<String> {
        if self.input.trim().is_empty() {
            self.input.clear();
            return None;
        }
        let cmd = self.input.clone();
        self.input.clear();
        self.editing = false;
        Some(cmd)
    }

    pub fn push_status(&mut self, msg: String) {
        self.status.push(msg);
    }

    pub fn set_size(&mut self, width: u16, height: u16) {
        self.size = Rect::new(0, 0, width, height);
    }

    pub fn tick(&mut self) {}

    pub fn is_editing(&self) -> bool {
        self.editing
    }

    pub fn update_feed(&mut self, posts: Vec<Post>) {
        self.feed = posts;
    }
}
