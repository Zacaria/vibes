use anyhow::{Context, Result};
use directories::ProjectDirs;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Read;
use std::path::PathBuf;
use std::sync::Arc;
use time::OffsetDateTime;
use tokio::task;

use crate::domain::Session;

pub struct SessionStore {
    path: PathBuf,
    mutex: Arc<Mutex<()>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PersistedSession {
    session: Session,
    saved_at: OffsetDateTime,
}

impl SessionStore {
    pub fn new() -> Result<Self> {
        let proj = ProjectDirs::from("com", "OpenAI", "cli-twitter").context("project dirs")?;
        let dir = proj.data_dir();
        std::fs::create_dir_all(dir).context("session dir")?;
        let path = dir.join("sessions.json");
        Ok(Self {
            path,
            mutex: Arc::new(Mutex::new(())),
        })
    }

    pub async fn save(&self, session: &Session) -> Result<()> {
        let path = self.path.clone();
        let session = session.clone();
        let guard = self.mutex.clone();
        task::spawn_blocking(move || {
            let _lock = guard.lock();
            let persisted = PersistedSession {
                session,
                saved_at: OffsetDateTime::now_utc(),
            };
            let file = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&path)?;
            serde_json::to_writer_pretty(file, &persisted)?;
            Ok::<_, anyhow::Error>(())
        })
        .await??;
        Ok(())
    }

    pub async fn load(&self) -> Result<Option<Session>> {
        let path = self.path.clone();
        let guard = self.mutex.clone();
        let result = task::spawn_blocking(move || {
            let _lock = guard.lock();
            if !path.exists() {
                return Ok::<Option<Session>, anyhow::Error>(None);
            }
            let mut buf = Vec::new();
            let mut file = OpenOptions::new().read(true).open(&path)?;
            file.read_to_end(&mut buf)?;
            if buf.is_empty() {
                return Ok(None);
            }
            let persisted: PersistedSession = serde_json::from_slice(&buf)?;
            Ok(Some(persisted.session))
        })
        .await??;
        Ok(result)
    }

    pub async fn clear(&self) -> Result<()> {
        let path = self.path.clone();
        let guard = self.mutex.clone();
        task::spawn_blocking(move || {
            let _lock = guard.lock();
            if path.exists() {
                std::fs::remove_file(path).ok();
            }
            Ok::<_, anyhow::Error>(())
        })
        .await??;
        Ok(())
    }
}
