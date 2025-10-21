use anyhow::{Context, Result};
use rusqlite::{params, Connection};

use crate::domain::{now_rfc3339, Task, TaskStatus};

pub struct TaskDao<'a> {
    conn: &'a Connection,
}

impl<'a> TaskDao<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn add_task(&self, title: &str, description: &str) -> Result<Task> {
        let created_at = now_rfc3339();
        self.conn
            .execute(
                "INSERT INTO tasks(title, description, status, created_at) VALUES(?1, ?2, 'open', ?3)",
                params![title, description, &created_at],
            )
            .context("insert task")?;
        let id = self.conn.last_insert_rowid();
        Ok(Task {
            id,
            title: title.to_string(),
            description: description.to_string(),
            status: TaskStatus::Open,
            created_at,
            done_at: None,
        })
    }

    pub fn list_tasks(&self, filter: Option<TaskStatus>) -> Result<Vec<Task>> {
        let mut stmt = match filter {
            Some(TaskStatus::Open) => self
                .conn
                .prepare("SELECT id, title, description, status, created_at, done_at FROM tasks WHERE status='open' ORDER BY id DESC"),
            Some(TaskStatus::Done) => self
                .conn
                .prepare("SELECT id, title, description, status, created_at, done_at FROM tasks WHERE status='done' ORDER BY id DESC"),
            None => self
                .conn
                .prepare("SELECT id, title, description, status, created_at, done_at FROM tasks ORDER BY id DESC"),
        }?;
        let rows = stmt
            .query_map([], |row| {
                let status: String = row.get(3)?;
                Ok(Task {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    description: row.get(2)?,
                    status: if status == "done" {
                        TaskStatus::Done
                    } else {
                        TaskStatus::Open
                    },
                    created_at: row.get(4)?,
                    done_at: row.get(5)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }

    pub fn mark_done(&self, id: i64) -> Result<Option<Task>> {
        let done_at = now_rfc3339();
        let changed = self
            .conn
            .execute(
                "UPDATE tasks SET status='done', done_at=?2 WHERE id=?1",
                params![id, &done_at],
            )
            .context("update task status")?;
        if changed == 0 {
            return Ok(None);
        }
        let task = self.get_task(id)?;
        Ok(task)
    }

    pub fn get_task(&self, id: i64) -> Result<Option<Task>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, description, status, created_at, done_at FROM tasks WHERE id=?1",
        )?;
        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            let status: String = row.get(3)?;
            let task = Task {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                status: if status == "done" {
                    TaskStatus::Done
                } else {
                    TaskStatus::Open
                },
                created_at: row.get(4)?,
                done_at: row.get(5)?,
            };
            Ok(Some(task))
        } else {
            Ok(None)
        }
    }
}
