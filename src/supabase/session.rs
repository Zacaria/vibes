use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use base64::{engine::general_purpose, Engine};
use directories::ProjectDirs;
use keyring::Entry;
use sha2::{Digest, Sha256};

use crate::domain::Session;

#[derive(Clone)]
pub struct SessionStore {
    path: PathBuf,
    service: String,
    username: String,
}

impl SessionStore {
    pub fn new(app_name: &str) -> Result<Self> {
        let project_dirs = ProjectDirs::from("dev", "PromptOps", app_name)
            .context("project dirs for session store")?;
        let config_dir = project_dirs.config_dir();
        fs::create_dir_all(config_dir).context("create config dir")?;
        let path = config_dir.join("sessions.json");
        let service = format!("{}-sessions", app_name);
        let username = whoami::username();
        Ok(Self {
            path,
            service,
            username,
        })
    }

    fn entry(&self) -> Result<Entry> {
        Entry::new(&self.service, &self.username).context("build keyring entry")
    }

    pub fn load(&self) -> Result<Session> {
        if let Ok(entry) = self.entry() {
            if let Ok(secret) = entry.get_password() {
                if let Ok(session) = serde_json::from_str::<Session>(&secret) {
                    return Ok(session);
                }
            }
        }
        if self.path.exists() {
            let encoded = fs::read_to_string(&self.path).context("read session file")?;
            let decoded = self.decode(&encoded)?;
            let session: Session =
                serde_json::from_slice(&decoded).context("decode session json")?;
            Ok(session)
        } else {
            Ok(Session::default())
        }
    }

    pub fn save(&self, session: &Session) -> Result<()> {
        let json = serde_json::to_string(session).context("serialize session")?;
        if let Ok(entry) = self.entry() {
            if let Err(err) = entry.set_password(&json) {
                tracing::warn!(error = %err, "failed to write keyring; falling back to file");
            } else {
                return Ok(());
            }
        }
        let encoded = self.encode(json.as_bytes());
        fs::write(&self.path, encoded).context("write session file")?;
        Ok(())
    }

    pub fn clear(&self) -> Result<()> {
        if let Ok(entry) = self.entry() {
            if let Err(err) = entry.delete_password() {
                tracing::warn!(error = %err, "failed to clear keyring session");
            }
        }
        if self.path.exists() {
            fs::remove_file(&self.path).ok();
        }
        Ok(())
    }

    fn encode(&self, data: &[u8]) -> String {
        let key = self.derive_key();
        let encrypted: Vec<u8> = data
            .iter()
            .zip(key.iter().cycle())
            .map(|(b, k)| b ^ k)
            .collect();
        general_purpose::STANDARD_NO_PAD.encode(encrypted)
    }

    fn decode(&self, encoded: &str) -> Result<Vec<u8>> {
        let key = self.derive_key();
        let bytes = general_purpose::STANDARD_NO_PAD
            .decode(encoded)
            .context("decode base64 session")?;
        Ok(bytes
            .iter()
            .zip(key.iter().cycle())
            .map(|(b, k)| b ^ k)
            .collect())
    }

    fn derive_key(&self) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(self.service.as_bytes());
        hasher.update(self.username.as_bytes());
        let host = whoami::fallible::hostname().unwrap_or_else(|_| "unknown-host".to_string());
        hasher.update(host.as_bytes());
        hasher.update(b"cli-twitter-session");
        hasher.finalize().to_vec()
    }
}
