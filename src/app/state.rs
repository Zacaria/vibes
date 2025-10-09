use crate::domain::{FeedFilter, Post, Session, Task};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Navigation,
    Command,
}

pub struct AppState {
    pub session: Session,
    pub feed: Vec<Post>,
    pub tasks: Vec<Task>,
    pub command_input: String,
    pub status_line: String,
    pub mode: InputMode,
    pub feed_filter: FeedFilter,
    pub last_sync: Option<String>,
}

impl AppState {
    pub fn new(session: Session) -> Self {
        Self {
            session,
            feed: Vec::new(),
            tasks: Vec::new(),
            command_input: String::new(),
            status_line: "Type /help for commands".to_string(),
            mode: InputMode::Command,
            feed_filter: FeedFilter::Global,
            last_sync: None,
        }
    }

    pub fn set_status(&mut self, status: impl Into<String>) {
        self.status_line = status.into();
    }

    pub fn update_feed(&mut self, posts: Vec<Post>) {
        self.feed = posts;
        self.last_sync = Some(crate::domain::now_rfc3339());
    }

    pub fn update_tasks(&mut self, tasks: Vec<Task>) {
        self.tasks = tasks;
    }
}
