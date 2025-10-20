use std::{collections::HashMap, fs, path::PathBuf};

use anyhow::Result;
use directories::BaseDirs;
use serde::Deserialize;

use crate::domain::{Profile, ProviderKind};

#[derive(Debug, Clone, Deserialize, Default)]
struct CodexFile {
    #[serde(default)]
    active_profile: Option<String>,
    #[serde(default)]
    profiles: HashMap<String, CodexProfile>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct CodexProfile {
    #[serde(default)]
    provider: Option<String>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    temperature: Option<f32>,
    #[serde(default)]
    top_p: Option<f32>,
    #[serde(default)]
    system_prompt: Option<String>,
}

pub fn detect_codex_profile_name(
    cli_override: Option<String>,
    env_override: Option<String>,
    explicit_codex: bool,
) -> Option<String> {
    if let Some(name) = cli_override {
        return Some(name);
    }
    if let Some(name) = env_override {
        return Some(name);
    }
    if !explicit_codex {
        return None;
    }
    load_active_profile_name().ok().flatten()
}

fn load_active_profile_name() -> Result<Option<String>> {
    for path in candidate_paths() {
        if path.exists() {
            let contents = fs::read_to_string(&path)?;
            let cfg: CodexFile = serde_yaml::from_str(&contents)?;
            if let Some(active) = cfg.active_profile {
                return Ok(Some(active));
            }
        }
    }
    Ok(None)
}

pub fn load_codex_profile(name: &str) -> Result<Option<Profile>> {
    for path in candidate_paths() {
        if path.exists() {
            let contents = fs::read_to_string(&path)?;
            let cfg: CodexFile = serde_yaml::from_str(&contents)?;
            if let Some(profile) = cfg.profiles.get(name) {
                return Ok(Some(convert_profile(name, profile.clone())));
            }
        }
    }
    Ok(None)
}

fn convert_profile(name: &str, profile: CodexProfile) -> Profile {
    let provider = profile
        .provider
        .as_deref()
        .unwrap_or("anthropic")
        .parse::<ProviderKind>()
        .unwrap_or(ProviderKind::Anthropic);
    Profile {
        name: name.to_string(),
        provider,
        model: profile
            .model
            .unwrap_or_else(|| "claude-3-sonnet".to_string()),
        description: Some("Imported from codex-cli".to_string()),
        temperature: profile.temperature,
        top_p: profile.top_p,
        system_prompt: profile.system_prompt,
        enabled: true,
        default: false,
        ..Default::default()
    }
}

fn candidate_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Some(base) = BaseDirs::new() {
        paths.push(base.config_dir().join("codex").join("config.yml"));
        paths.push(base.config_dir().join("codex").join("config.yaml"));
    }
    if let Some(home) = std::env::var_os("HOME") {
        let mut path = PathBuf::from(home);
        path.push(".config/codex/config.yml");
        paths.push(path);
    }
    if let Some(path) = std::env::var_os("CODEX_CONFIG") {
        paths.push(PathBuf::from(path));
    }
    paths
}
