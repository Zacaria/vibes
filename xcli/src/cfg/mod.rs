use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub default_feed: Option<String>,
    pub supabase_project: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            default_feed: Some("global".into()),
            supabase_project: None,
        }
    }
}

pub fn load_config() -> Result<AppConfig> {
    dotenvy::dotenv().ok();
    if let Ok(path) = std::env::var("CLI_TWITTER_CONFIG") {
        return read_file(PathBuf::from(path));
    }
    let proj = ProjectDirs::from("com", "OpenAI", "cli-twitter").context("project dirs")?;
    let config_dir = proj.config_dir();
    std::fs::create_dir_all(config_dir).ok();
    let path = config_dir.join("config.toml");
    if path.exists() {
        read_file(path)
    } else {
        Ok(AppConfig::default())
    }
}

fn read_file(path: PathBuf) -> Result<AppConfig> {
    let mut buf = String::new();
    File::open(&path)
        .with_context(|| format!("opening config {}", path.display()))?
        .read_to_string(&mut buf)?;
    let cfg = toml::from_str(&buf)?;
    Ok(cfg)
}
