use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboxItem {
    pub id: i64,
    pub kind: String,
    pub payload: String,
    pub created_at: String,
    pub attempts: i64,
}

pub struct OutboxDao<'a> {
    conn: &'a Connection,
}

impl<'a> OutboxDao<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn enqueue(&self, kind: &str, payload: &str) -> Result<()> {
        self.conn
            .execute(
                "INSERT INTO outbox(kind, payload, created_at, attempts) VALUES(?1, ?2, datetime('now'), 0)",
                params![kind, payload],
            )
            .context("insert into outbox")?;
        Ok(())
    }

    pub fn list_pending(&self, limit: usize) -> Result<Vec<OutboxItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, kind, payload, created_at, attempts FROM outbox ORDER BY id ASC LIMIT ?1",
        )?;
        let rows = stmt
            .query_map([limit as i64], |row| {
                Ok(OutboxItem {
                    id: row.get(0)?,
                    kind: row.get(1)?,
                    payload: row.get(2)?,
                    created_at: row.get(3)?,
                    attempts: row.get(4)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }

    pub fn increment_attempts(&self, id: i64) -> Result<()> {
        self.conn
            .execute(
                "UPDATE outbox SET attempts = attempts + 1 WHERE id=?1",
                params![id],
            )
            .context("increment outbox attempts")?;
        Ok(())
    }

    pub fn remove(&self, id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM outbox WHERE id=?1", params![id])
            .context("delete outbox item")?;
        Ok(())
    }
}
