use anyhow::{anyhow, Result};

use crate::storage::export::ExportFormat;

#[derive(Debug, Clone)]
pub enum Command {
    Profile(String),
    Model(String),
    SystemPrompt(String),
    NewConversation,
    ToggleStreaming,
    Export(ExportFormat),
    Help,
    Unknown(String),
}

pub fn parse(input: &str) -> Result<Command> {
    let input = input.trim();
    if !input.starts_with('/') {
        return Err(anyhow!("not a command"));
    }
    let mut parts = input[1..].split_whitespace();
    let Some(keyword) = parts.next() else {
        return Ok(Command::Help);
    };
    let rest = parts.collect::<Vec<_>>().join(" ");
    match keyword {
        "profile" => Ok(Command::Profile(rest.trim().to_string())),
        "model" => Ok(Command::Model(rest.trim().to_string())),
        "sys" | "system" => Ok(Command::SystemPrompt(rest)),
        "new" => Ok(Command::NewConversation),
        "stream" => Ok(Command::ToggleStreaming),
        "export" => {
            let format = match rest.trim() {
                "json" => ExportFormat::Json,
                "md" | "markdown" => ExportFormat::Md,
                other => return Ok(Command::Unknown(other.to_string())),
            };
            Ok(Command::Export(format))
        }
        "help" => Ok(Command::Help),
        other => Ok(Command::Unknown(other.to_string())),
    }
}
