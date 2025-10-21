#![allow(dead_code)]

use thiserror::Error;

#[derive(Debug, Error)]
pub enum SquadError {
    #[error("configuration error: {0}")]
    Config(String),
    #[error("storage error: {0}")]
    Storage(String),
    #[error("provider error: {0}")]
    Provider(String),
    #[error("command error: {0}")]
    Command(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("yaml error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("toml error: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("sql error: {0}")]
    Sql(#[from] rusqlite::Error),
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
}

pub type SquadResult<T> = Result<T, SquadError>;
