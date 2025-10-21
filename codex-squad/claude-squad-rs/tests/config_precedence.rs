use std::fs;
use std::path::Path;

use assert_fs::TempDir;
use claude_squad_rs::config::{ConfigLoader, ExecutionContext, ExecutionOverrides};
use serial_test::serial;

fn write_profiles(dir: &Path) {
    let profiles = r#"- name: default
  provider: anthropic
  model: claude-3-haiku
  default: true
- name: pro
  provider: open_ai
  model: gpt-4o
"#;
    fs::write(dir.join("profiles.yaml"), profiles).unwrap();
}

fn write_codex_config(path: &Path) {
    let cfg = r#"active_profile: default
profiles:
  default:
    provider: anthropic
    model: claude-3-opus
  pro:
    provider: open_ai
    model: gpt-4o
"#;
    fs::write(path, cfg).unwrap();
}

#[serial]
#[test]
fn cli_override_wins_over_env_and_codex() {
    let temp = TempDir::new().unwrap();
    write_profiles(temp.path());
    let codex_path = temp.path().join("codex.yaml");
    write_codex_config(&codex_path);
    std::env::set_var("CODEX_CONFIG", codex_path.to_str().unwrap());
    let loader = ConfigLoader::discover(Some(temp.path().to_path_buf())).unwrap();
    let overrides = ExecutionOverrides {
        profile: Some("pro".into()),
        codex_profile: Some("default".into()),
        codex_enabled: true,
    };
    let ctx = ExecutionContext::new(loader, overrides).unwrap();
    let profile = ctx.active_profile(None).unwrap();
    assert_eq!(profile.name, "pro");
    assert_eq!(profile.model, "gpt-4o");
    std::env::remove_var("CODEX_CONFIG");
}

#[serial]
#[test]
fn env_override_wins_over_codex_config() {
    let temp = TempDir::new().unwrap();
    write_profiles(temp.path());
    let codex_path = temp.path().join("codex.yaml");
    write_codex_config(&codex_path);
    std::env::set_var("CODEX_CONFIG", codex_path.to_str().unwrap());
    let loader = ConfigLoader::discover(Some(temp.path().to_path_buf())).unwrap();
    let overrides = ExecutionOverrides {
        profile: None,
        codex_profile: Some("pro".into()),
        codex_enabled: true,
    };
    let ctx = ExecutionContext::new(loader, overrides).unwrap();
    let profile = ctx.active_profile(None).unwrap();
    assert_eq!(profile.name, "pro");
    std::env::remove_var("CODEX_CONFIG");
}

#[serial]
#[test]
fn codex_config_used_when_enabled() {
    let temp = TempDir::new().unwrap();
    write_profiles(temp.path());
    let codex_path = temp.path().join("codex.yaml");
    write_codex_config(&codex_path);
    std::env::set_var("CODEX_CONFIG", codex_path.to_str().unwrap());
    let loader = ConfigLoader::discover(Some(temp.path().to_path_buf())).unwrap();
    let overrides = ExecutionOverrides {
        profile: None,
        codex_profile: None,
        codex_enabled: true,
    };
    let ctx = ExecutionContext::new(loader, overrides).unwrap();
    let profile = ctx.active_profile(None).unwrap();
    assert_eq!(profile.name, "default");
    assert_eq!(profile.model, "claude-3-opus");
    std::env::remove_var("CODEX_CONFIG");
}
