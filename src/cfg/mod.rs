use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct RawConfig {
    #[serde(default = "default_app_name")]
    pub app_name: String,
    pub reports_dir: Option<PathBuf>,
    pub supabase: Option<SupabaseSection>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct SupabaseSection {
    pub url: Option<String>,
    pub anon_key: Option<String>,
}

fn default_app_name() -> String {
    "cli-twitter".to_string()
}

pub struct AppConfig {
    inner: RawConfig,
}

impl AppConfig {
    pub fn load(path: Option<&Path>) -> Result<Self> {
        let mut inner = RawConfig::default();
        if let Some(path) = path {
            if path.exists() {
                let text = fs::read_to_string(path)
                    .with_context(|| format!("read config {}", path.display()))?;
                inner = toml::from_str(&text).context("parse config")?;
            }
        } else {
            let default = Path::new("config/config.toml");
            if default.exists() {
                let text = fs::read_to_string(default).context("read default config")?;
                inner = toml::from_str(&text).context("parse default config")?;
            }
        }
        Ok(Self { inner })
    }

    pub fn app_name(&self) -> &str {
        &self.inner.app_name
    }

    pub fn reports_dir(&self) -> Result<PathBuf> {
        if let Some(dir) = &self.inner.reports_dir {
            fs::create_dir_all(dir).context("create reports dir")?;
            return Ok(dir.clone());
        }
        let project_dirs = ProjectDirs::from("dev", "PromptOps", &self.inner.app_name)
            .context("project dirs for reports")?;
        let dir = project_dirs.data_dir().join("reports");
        fs::create_dir_all(&dir).context("create default reports dir")?;
        Ok(dir)
    }

    pub fn supabase(&self) -> Option<&SupabaseSection> {
        self.inner.supabase.as_ref()
    }
}
