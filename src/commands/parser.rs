use thiserror::Error;
use uuid::Uuid;

use crate::domain::{AudienceScope, FeedFilter, TaskStatus};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Login {
        email: String,
        password: String,
    },
    Passkey,
    Post {
        body: String,
        audience: AudienceScope,
    },
    Feed {
        filter: FeedFilter,
    },
    Follow {
        handle: String,
    },
    Like {
        post_id: Uuid,
    },
    WhoAmI,
    Logout,
    TasksAdd {
        title: String,
        description: String,
    },
    TasksList {
        status: Option<TaskStatus>,
    },
    TasksDone {
        id: i64,
    },
    ReportSync,
    SettingsShow,
    SettingsSet {
        key: String,
        value: String,
    },
    Help,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CommandError {
    #[error("not a command")]
    NotACommand,
    #[error("unknown command: {0}")]
    UnknownCommand(String),
    #[error("invalid arguments: {0}")]
    InvalidArguments(String),
}

pub fn parse_command(input: &str) -> Result<Command, CommandError> {
    let trimmed = input.trim();
    if !trimmed.starts_with('/') {
        return Err(CommandError::NotACommand);
    }
    let mut parts = trimmed.splitn(2, ' ');
    let head = parts.next().unwrap();
    let tail = parts.next().unwrap_or("").trim();
    match head {
        "/login" => parse_login(tail),
        "/passkey" => Ok(Command::Passkey),
        "/post" => parse_post(tail),
        "/feed" => Ok(Command::Feed {
            filter: FeedFilter::from_optional_str(if tail.is_empty() { None } else { Some(tail) }),
        }),
        "/follow" => {
            if tail.is_empty() {
                Err(CommandError::InvalidArguments("missing handle".into()))
            } else {
                Ok(Command::Follow {
                    handle: tail.to_string(),
                })
            }
        }
        "/like" => {
            let id = tail.trim();
            let uuid = Uuid::parse_str(id)
                .map_err(|_| CommandError::InvalidArguments("invalid post id".into()))?;
            Ok(Command::Like { post_id: uuid })
        }
        "/whoami" => Ok(Command::WhoAmI),
        "/logout" => Ok(Command::Logout),
        "/tasks" => parse_tasks(tail),
        "/report" => parse_report(tail),
        "/settings" => parse_settings(tail),
        "/help" => Ok(Command::Help),
        other => Err(CommandError::UnknownCommand(other.to_string())),
    }
}

fn parse_login(args: &str) -> Result<Command, CommandError> {
    let mut email = None;
    let mut password = None;
    for token in args.split_whitespace() {
        if let Some(rest) = token.strip_prefix("email:") {
            email = Some(rest.to_string());
        } else if let Some(rest) = token.strip_prefix("pw:") {
            password = Some(rest.to_string());
        }
    }
    match (email, password) {
        (Some(email), Some(password)) => Ok(Command::Login { email, password }),
        _ => Err(CommandError::InvalidArguments(
            "usage: /login email:<addr> pw:<secret>".into(),
        )),
    }
}

fn parse_post(args: &str) -> Result<Command, CommandError> {
    let (body, rest) = extract_quoted(args)
        .ok_or_else(|| CommandError::InvalidArguments("missing post body".into()))?;
    let mut audience = AudienceScope::Public;
    for token in rest.split_whitespace() {
        if let Some(val) = token.strip_prefix("audience:") {
            audience = val
                .parse()
                .map_err(|_| CommandError::InvalidArguments("invalid audience".into()))?;
        }
    }
    Ok(Command::Post { body, audience })
}

fn parse_tasks(args: &str) -> Result<Command, CommandError> {
    if args.starts_with("add ") {
        let rest = args.trim_start_matches("add").trim_start();
        let (title, rest) = extract_quoted(rest)
            .ok_or_else(|| CommandError::InvalidArguments("missing task title".into()))?;
        let (description, _) = extract_quoted(rest)
            .ok_or_else(|| CommandError::InvalidArguments("missing task description".into()))?;
        return Ok(Command::TasksAdd { title, description });
    } else if args.starts_with("ls") {
        let status = args.split_whitespace().nth(1).and_then(|s| match s {
            "open" => Some(TaskStatus::Open),
            "done" => Some(TaskStatus::Done),
            "all" => None,
            _ => None,
        });
        return Ok(Command::TasksList { status });
    } else if args.starts_with("done") {
        let id_str = args
            .split_whitespace()
            .nth(1)
            .ok_or_else(|| CommandError::InvalidArguments("usage: /tasks done <id>".into()))?;
        let id = id_str
            .parse::<i64>()
            .map_err(|_| CommandError::InvalidArguments("invalid task id".into()))?;
        return Ok(Command::TasksDone { id });
    }
    Err(CommandError::InvalidArguments(
        "usage: /tasks <add|ls|done>".into(),
    ))
}

fn parse_report(args: &str) -> Result<Command, CommandError> {
    match args.trim() {
        "sync" => Ok(Command::ReportSync),
        _ => Err(CommandError::InvalidArguments("usage: /report sync".into())),
    }
}

fn parse_settings(args: &str) -> Result<Command, CommandError> {
    let trimmed = args.trim();
    if trimmed == "show" {
        Ok(Command::SettingsShow)
    } else if let Some(rest) = trimmed.strip_prefix("set ") {
        let mut iter = rest.splitn(2, '=');
        let key = iter
            .next()
            .ok_or_else(|| CommandError::InvalidArguments("missing key".into()))?
            .trim();
        let value = iter
            .next()
            .ok_or_else(|| CommandError::InvalidArguments("missing value".into()))?
            .trim();
        Ok(Command::SettingsSet {
            key: key.to_string(),
            value: value.to_string(),
        })
    } else {
        Err(CommandError::InvalidArguments(
            "usage: /settings show | /settings set key=value".into(),
        ))
    }
}

fn extract_quoted(input: &str) -> Option<(String, &str)> {
    let trimmed = input.trim_start();
    if !trimmed.starts_with('"') {
        return None;
    }
    let mut result = String::new();
    let mut escaped = false;
    for (idx, ch) in trimmed[1..].char_indices() {
        if escaped {
            result.push(ch);
            escaped = false;
            continue;
        }
        match ch {
            '\\' => {
                escaped = true;
            }
            '"' => {
                let rest_index = idx + 2; // skip opening quote + closing quote
                let rest = &trimmed[rest_index..];
                return Some((result, rest));
            }
            other => result.push(other),
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_login_command() {
        let cmd = parse_command("/login email:foo@example.com pw:secret").unwrap();
        assert_eq!(
            cmd,
            Command::Login {
                email: "foo@example.com".into(),
                password: "secret".into()
            }
        );
    }

    #[test]
    fn parse_post_command() {
        let cmd = parse_command("/post \"hello\" audience:private").unwrap();
        assert_eq!(
            cmd,
            Command::Post {
                body: "hello".into(),
                audience: AudienceScope::Private
            }
        );
    }

    #[test]
    fn parse_tasks_add() {
        let cmd = parse_command("/tasks add \"title\" \"desc\"").unwrap();
        assert_eq!(
            cmd,
            Command::TasksAdd {
                title: "title".into(),
                description: "desc".into()
            }
        );
    }
}
