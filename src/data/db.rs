use std::{fs, path::PathBuf, sync::Arc};

use anyhow::{Context, Result};
use directories::ProjectDirs;
use parking_lot::Mutex;
use rusqlite::{Connection, OpenFlags};

pub struct DbPool {
    conn: Arc<Mutex<Connection>>,
    path: PathBuf,
}

impl Clone for DbPool {
    fn clone(&self) -> Self {
        Self {
            conn: Arc::clone(&self.conn),
            path: self.path.clone(),
        }
    }
}

impl DbPool {
    pub fn new(app_name: &str) -> Result<Self> {
        let project_dirs = ProjectDirs::from("dev", "PromptOps", app_name)
            .context("unable to resolve project directories")?;
        let data_dir = project_dirs.data_dir();
        fs::create_dir_all(data_dir).context("failed to create app data dir")?;
        let db_path = data_dir.join("app.db");
        let conn = Connection::open_with_flags(
            &db_path,
            OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_FULL_MUTEX,
        )
        .with_context(|| format!("opening sqlite at {}", db_path.display()))?;
        let pool = Self {
            conn: Arc::new(Mutex::new(conn)),
            path: db_path,
        };
        pool.apply_migrations()?;
        Ok(pool)
    }

    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory().context("open in-memory sqlite")?;
        let pool = Self {
            conn: Arc::new(Mutex::new(conn)),
            path: PathBuf::from(":memory:"),
        };
        pool.apply_migrations()?;
        Ok(pool)
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn connection(&self) -> parking_lot::MutexGuard<'_, Connection> {
        self.conn.lock()
    }

    pub fn with_conn<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&Connection) -> Result<R>,
    {
        let conn = self.connection();
        f(&conn)
    }

    fn apply_migrations(&self) -> Result<()> {
        static MIGRATIONS: &[(&str, &str)] = &[(
            "0001_init.sql",
            include_str!("../../migrations/sqlite/0001_init.sql"),
        )];

        let conn = self.connection();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS __migrations (name TEXT PRIMARY KEY, applied_at TEXT NOT NULL)",
            [],
        )
        .context("create migrations table")?;

        for (name, sql) in MIGRATIONS {
            let already: Option<String> = conn
                .query_row(
                    "SELECT name FROM __migrations WHERE name = ?1",
                    [name],
                    |row| row.get(0),
                )
                .optional()
                .context("check migration")?;
            if already.is_some() {
                continue;
            }
            tracing::info!(migration = %name, "applying migration");
            conn.execute_batch(sql)
                .with_context(|| format!("apply migration {}", name))?;
            conn.execute(
                "INSERT INTO __migrations(name, applied_at) VALUES(?1, datetime('now'))",
                [name],
            )
            .context("insert migration record")?;
        }

        Ok(())
    }
}

trait OptionalRow {
    fn optional(self) -> Result<Option<String>>;
}

impl OptionalRow for rusqlite::Result<String> {
    fn optional(self) -> Result<Option<String>> {
        match self {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}
