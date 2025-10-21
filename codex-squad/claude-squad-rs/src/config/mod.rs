use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{anyhow, Result};
use directories::ProjectDirs;
use parking_lot::RwLock;

use crate::domain::{Keymap, Profile, ProviderKind, Squad};
use crate::integrate::codex_cli;
use crate::storage::Storage;

#[derive(Clone)]
pub struct ConfigLoader {
    config_dir: Arc<PathBuf>,
}

impl ConfigLoader {
    pub fn discover(config_override: Option<PathBuf>) -> Result<Self> {
        let config_dir = if let Some(path) = config_override {
            path
        } else if let Ok(env) = std::env::var("CLAUDE_SQUAD_CONFIG") {
            PathBuf::from(env)
        } else {
            default_config_dir()?
        };
        std::fs::create_dir_all(&config_dir)?;
        Ok(Self {
            config_dir: Arc::new(config_dir),
        })
    }

    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }

    pub fn profiles_file(&self) -> PathBuf {
        self.config_dir.join("profiles.yaml")
    }

    pub fn squads_file(&self) -> PathBuf {
        self.config_dir.join("squads.yaml")
    }

    #[allow(dead_code)]
    pub fn keymaps_file(&self) -> PathBuf {
        self.config_dir.join("keymaps.toml")
    }

    pub fn load_profiles(&self) -> Result<Vec<Profile>> {
        read_yaml(self.profiles_file())
    }

    pub fn load_squads(&self) -> Result<Vec<Squad>> {
        read_yaml(self.squads_file())
    }

    #[allow(dead_code)]
    pub fn load_keymap(&self) -> Result<Option<Keymap>> {
        let path = self.keymaps_file();
        if path.exists() {
            let data = std::fs::read_to_string(path)?;
            let map: Keymap = toml::from_str(&data)?;
            Ok(Some(map))
        } else {
            Ok(None)
        }
    }
}

fn read_yaml<T>(path: PathBuf) -> Result<Vec<T>>
where
    T: serde::de::DeserializeOwned,
{
    if !path.exists() {
        return Ok(Vec::new());
    }
    let contents = std::fs::read_to_string(path)?;
    if contents.trim().is_empty() {
        return Ok(Vec::new());
    }
    let value = serde_yaml::from_str(&contents)?;
    Ok(value)
}

fn default_config_dir() -> Result<PathBuf> {
    if let Some(dirs) = ProjectDirs::from("ai", "codex", "claude-squad-rs") {
        Ok(dirs.config_dir().to_path_buf())
    } else {
        Err(anyhow!("unable to determine configuration directory"))
    }
}

#[derive(Clone, Default)]
pub struct ExecutionOverrides {
    pub profile: Option<String>,
    pub codex_profile: Option<String>,
    pub codex_enabled: bool,
}

#[derive(Clone)]
pub struct ExecutionContext {
    loader: ConfigLoader,
    overrides: ExecutionOverrides,
    storage: Storage,
    cached_profiles: Arc<RwLock<Vec<Profile>>>,
}

impl ExecutionContext {
    pub fn new(loader: ConfigLoader, overrides: ExecutionOverrides) -> Result<Self> {
        let data_dir = loader.config_dir().join("data");
        std::fs::create_dir_all(&data_dir)?;
        let storage = Storage::try_new(data_dir.join("app.db"))?;
        Ok(Self {
            loader,
            overrides,
            storage,
            cached_profiles: Arc::new(RwLock::new(Vec::new())),
        })
    }

    pub fn loader(&self) -> &ConfigLoader {
        &self.loader
    }

    pub fn storage(&self) -> Storage {
        self.storage.clone()
    }

    pub fn load_profiles(&self) -> Result<Vec<Profile>> {
        let mut guard = self.cached_profiles.write();
        if guard.is_empty() {
            *guard = self.loader.load_profiles()?;
        }
        Ok(guard.clone())
    }

    #[allow(dead_code)]
    pub fn reload_profiles(&self) -> Result<()> {
        let profiles = self.loader.load_profiles()?;
        *self.cached_profiles.write() = profiles;
        Ok(())
    }

    pub fn load_squads(&self) -> Result<Vec<Squad>> {
        self.loader.load_squads()
    }

    #[allow(dead_code)]
    pub fn load_keymap(&self) -> Result<Option<Keymap>> {
        self.loader.load_keymap()
    }

    pub fn list_models(&self) -> Result<Vec<String>> {
        let mut models = self
            .load_profiles()?
            .into_iter()
            .map(|p| format!("{}:{}", p.provider_string(), p.model))
            .collect::<Vec<_>>();
        models.sort();
        models.dedup();
        Ok(models)
    }

    pub fn active_profile(&self, runtime_override: Option<&str>) -> Result<Profile> {
        let profiles = self.load_profiles()?;
        if let Some(name) = runtime_override {
            if let Some(profile) = self.fetch_profile(&profiles, name)? {
                return Ok(profile);
            }
        }
        if let Some(profile) = self.resolve_profile_from_overrides(&profiles)? {
            return Ok(profile);
        }
        profiles
            .first()
            .cloned()
            .ok_or_else(|| anyhow!("no profiles configured"))
    }

    fn resolve_profile_from_overrides(&self, profiles: &[Profile]) -> Result<Option<Profile>> {
        if let Some(name) = self.overrides.profile.clone() {
            if let Some(profile) = self.fetch_profile(profiles, &name)? {
                return Ok(Some(profile));
            }
        }
        if let Some(name) = self.overrides.codex_profile.clone() {
            if let Some(profile) = self.fetch_profile(profiles, &name)? {
                return Ok(Some(profile));
            }
        }
        if self.overrides.codex_enabled {
            if let Some(name) = codex_cli::detect_codex_profile_name(None, None, true) {
                if let Some(profile) = self.fetch_profile(profiles, &name)? {
                    return Ok(Some(profile));
                }
            }
        }
        if let Some(profile) = profiles.iter().find(|p| p.default) {
            return Ok(Some(profile.clone()));
        }
        Ok(None)
    }

    fn fetch_profile(&self, profiles: &[Profile], name: &str) -> Result<Option<Profile>> {
        if self.overrides.codex_enabled {
            if let Some(profile) = codex_cli::load_codex_profile(name)? {
                return Ok(Some(profile));
            }
        }
        Ok(find_profile(profiles, name))
    }

    pub fn diagnose(&self) -> Result<String> {
        let mut report = String::new();
        let profiles = self.load_profiles()?;
        report.push_str(&format!("Profiles: {}\n", profiles.len()));
        for profile in &profiles {
            report.push_str(&format!(
                " - {} ({:?} -> {})\n",
                profile.name, profile.provider, profile.model
            ));
        }
        let squads = self.load_squads().unwrap_or_default();
        report.push_str(&format!("Squads: {}\n", squads.len()));
        let db_state = self.storage.health_check();
        report.push_str(&format!("Storage: {}\n", db_state));
        Ok(report)
    }

    #[allow(dead_code)]
    pub fn overrides(&self) -> &ExecutionOverrides {
        &self.overrides
    }
}

fn find_profile(profiles: &[Profile], name: &str) -> Option<Profile> {
    profiles.iter().find(|p| p.name == name).cloned()
}

impl Profile {
    pub fn provider_string(&self) -> String {
        match self.provider {
            ProviderKind::Anthropic => "anthropic".into(),
            ProviderKind::OpenAi => "openai".into(),
            ProviderKind::OpenAiCompat => "openai_compat".into(),
        }
    }
}
