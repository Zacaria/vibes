use anyhow::Result;
use rusqlite::params;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use crate::domain::Report;

use super::AppDatabase;

pub struct ReportDao<'a> {
    db: &'a AppDatabase,
}

impl<'a> ReportDao<'a> {
    pub fn new(db: &'a AppDatabase) -> Self {
        Self { db }
    }

    pub fn insert(&self, report: &Report) -> Result<()> {
        let conn = self.db.connection();
        conn.execute(
            "INSERT OR IGNORE INTO reports(id, task_id, path, summary, created_at) VALUES(?1, ?2, ?3, ?4, ?5)",
            params![
                report.id,
                report.task_id,
                &report.path,
                &report.summary,
                report
                    .created_at
                    .format(&Rfc3339)
                    .unwrap_or_else(|_| report.created_at.to_string())
            ],
        )?;
        Ok(())
    }

    pub fn latest(&self, limit: usize) -> Result<Vec<Report>> {
        let conn = self.db.connection();
        let mut stmt = conn.prepare(
            "SELECT id, task_id, path, summary, created_at FROM reports ORDER BY created_at DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            let created_at: String = row.get(4)?;
            Ok(Report {
                id: row.get(0)?,
                task_id: row.get(1)?,
                path: row.get(2)?,
                summary: row.get(3)?,
                created_at: OffsetDateTime::parse(&created_at, &Rfc3339)
                    .unwrap_or_else(|_| OffsetDateTime::now_utc()),
            })
        })?;
        Ok(rows.filter_map(Result::ok).collect())
    }
}
