use anyhow::{Context, Result};
use rusqlite::{params, OptionalExtension};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use crate::domain::{Task, TaskStatus};

use super::AppDatabase;

pub struct TaskDao<'a> {
    db: &'a AppDatabase,
}

impl<'a> TaskDao<'a> {
    pub fn new(db: &'a AppDatabase) -> Self {
        Self { db }
    }

    pub fn add(&self, title: &str, description: &str) -> Result<Task> {
        let created_at = OffsetDateTime::now_utc();
        let conn = self.db.connection();
        conn.execute(
            "INSERT INTO tasks(title, description, status, created_at) VALUES(?1, ?2, 'open', ?3)",
            params![
                title,
                description,
                created_at
                    .format(&Rfc3339)
                    .unwrap_or_else(|_| created_at.to_string())
            ],
        )?;
        let id = conn.last_insert_rowid();
        Ok(Task {
            id,
            title: title.to_string(),
            description: description.to_string(),
            status: TaskStatus::Open,
            created_at,
            done_at: None,
        })
    }

    pub fn list(&self, filter: Option<TaskStatus>) -> Result<Vec<Task>> {
        let conn = self.db.connection();
        let mut stmt = match filter {
            Some(TaskStatus::Open) => conn.prepare("SELECT id, title, description, status, created_at, done_at FROM tasks WHERE status='open' ORDER BY created_at DESC")?,
            Some(TaskStatus::Done) => conn.prepare("SELECT id, title, description, status, created_at, done_at FROM tasks WHERE status='done' ORDER BY done_at DESC")?,
            None => conn.prepare("SELECT id, title, description, status, created_at, done_at FROM tasks ORDER BY created_at DESC")?,
        };
        let tasks = stmt
            .query_map([], |row| {
                let status: String = row.get(3)?;
                let created_at: String = row.get(4)?;
                let done_at_raw: Option<String> = row.get(5)?;
                Ok(Task {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    description: row.get(2)?,
                    status: match status.as_str() {
                        "done" => TaskStatus::Done,
                        _ => TaskStatus::Open,
                    },
                    created_at: OffsetDateTime::parse(&created_at, &Rfc3339)
                        .unwrap_or_else(|_| OffsetDateTime::now_utc()),
                    done_at: done_at_raw.and_then(|s| OffsetDateTime::parse(&s, &Rfc3339).ok()),
                })
            })?
            .filter_map(Result::ok)
            .collect();
        Ok(tasks)
    }

    pub fn mark_done(&self, id: i64) -> Result<Option<Task>> {
        let done_at = OffsetDateTime::now_utc();
        let rows = {
            let conn = self.db.connection();
            conn.execute(
                "UPDATE tasks SET status='done', done_at=?2 WHERE id=?1",
                params![
                    id,
                    done_at
                        .format(&Rfc3339)
                        .unwrap_or_else(|_| done_at.to_string())
                ],
            )?
        };
        if rows == 0 {
            return Ok(None);
        }
        self.get(id).map(Some)
    }

    pub fn get(&self, id: i64) -> Result<Task> {
        let conn = self.db.connection();
        conn.query_row(
            "SELECT id, title, description, status, created_at, done_at FROM tasks WHERE id=?1",
            params![id],
            |row| {
                let status: String = row.get(3)?;
                let created_at: String = row.get(4)?;
                let done_at_raw: Option<String> = row.get(5)?;
                Ok(Task {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    description: row.get(2)?,
                    status: match status.as_str() {
                        "done" => TaskStatus::Done,
                        _ => TaskStatus::Open,
                    },
                    created_at: OffsetDateTime::parse(&created_at, &Rfc3339)
                        .unwrap_or_else(|_| OffsetDateTime::now_utc()),
                    done_at: done_at_raw.and_then(|s| OffsetDateTime::parse(&s, &Rfc3339).ok()),
                })
            },
        )
        .optional()
        .context("task lookup")?
        .context("task not found")
    }
}
