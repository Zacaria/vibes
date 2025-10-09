use std::{fs, path::Path};

use anyhow::{Context, Result};
use rusqlite::{params, Connection};

use crate::domain::{now_rfc3339, Report};

pub struct ReportsDao<'a> {
    conn: &'a Connection,
}

impl<'a> ReportsDao<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn record_report(&self, task_id: Option<i64>, path: &str, summary: &str) -> Result<Report> {
        let created_at = now_rfc3339();
        self.conn
            .execute(
                "INSERT INTO reports(task_id, path, summary, created_at) VALUES(?1, ?2, ?3, ?4)",
                params![task_id, path, summary, &created_at],
            )
            .with_context(|| format!("insert report record for {}", path))?;
        let id = self.conn.last_insert_rowid();
        Ok(Report {
            id,
            task_id,
            path: path.to_string(),
            summary: summary.to_string(),
            created_at,
        })
    }

    pub fn list_reports(&self) -> Result<Vec<Report>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, task_id, path, summary, created_at FROM reports ORDER BY id DESC",
        )?;
        let rows = stmt
            .query_map([], |row| {
                Ok(Report {
                    id: row.get(0)?,
                    task_id: row.get(1)?,
                    path: row.get(2)?,
                    summary: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }
}

pub fn write_report_file<P: AsRef<Path>>(
    reports_dir: P,
    filename: &str,
    content: &str,
) -> Result<std::path::PathBuf> {
    let dir = reports_dir.as_ref();
    fs::create_dir_all(dir).context("create reports directory")?;
    let path = dir.join(filename);
    fs::write(&path, content).with_context(|| format!("write report {}", path.display()))?;
    Ok(path)
}
