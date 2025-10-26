use std::path::PathBuf;

use anyhow::{Context, Result};
use directories::ProjectDirs;
use parking_lot::Mutex;
use rusqlite::{Connection, OpenFlags};
use std::sync::Arc;

use crate::data::migrations;

#[derive(Clone)]
pub struct AppDatabase {
    conn: Arc<Mutex<Connection>>,
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub path: PathBuf,
}

impl DatabaseConfig {
    pub fn resolve() -> Result<Self> {
        let proj = ProjectDirs::from("com", "OpenAI", "cli-twitter")
            .context("unable to resolve project dirs")?;
        let data_dir = proj.data_dir();
        std::fs::create_dir_all(data_dir).context("creating data dir")?;
        let db_path = data_dir.join("app.db");
        Ok(Self { path: db_path })
    }
}

impl AppDatabase {
    pub fn open(cfg: &DatabaseConfig) -> Result<Self> {
        let mut conn = Connection::open_with_flags(
            &cfg.path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
        )?;
        conn.pragma_update(None, "foreign_keys", &1)?;
        migrations::apply(&mut conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn connection(&self) -> parking_lot::MutexGuard<'_, Connection> {
        self.conn.lock()
    }
}
