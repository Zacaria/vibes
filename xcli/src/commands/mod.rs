use anyhow::{anyhow, Result};
use regex::Regex;
use std::collections::HashMap;
use std::str::FromStr;

use crate::data::{cache::CacheDao, reports::ReportDao, tasks::TaskDao, AppDatabase};
use crate::domain::{AudienceScope, FeedFilter, TaskStatus};
use crate::supabase::SupabaseClient;

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Login {
        email: String,
        password: String,
    },
    Passkey,
    Post {
        text: String,
        audience: AudienceScope,
    },
    Feed {
        filter: FeedFilter,
    },
    Follow {
        handle: String,
    },
    Like {
        post_id: String,
    },
    WhoAmI,
    Logout,
    TasksAdd {
        title: String,
        description: String,
    },
    TasksList {
        filter: Option<TaskStatus>,
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
}

pub fn parse_command(input: &str) -> Result<Command> {
    let input = input.trim();
    if !input.starts_with('/') {
        return Err(anyhow!("commands must start with '/'"));
    }
    let mut parts = input[1..].split_whitespace();
    let cmd = parts.next().ok_or_else(|| anyhow!("missing command"))?;
    match cmd {
        "login" => {
            let mut email = None;
            let mut password = None;
            for chunk in parts {
                if let Some(rest) = chunk.strip_prefix("email:") {
                    email = Some(rest.to_string());
                } else if let Some(rest) = chunk.strip_prefix("pw:") {
                    password = Some(rest.to_string());
                }
            }
            Ok(Command::Login {
                email: email.ok_or_else(|| anyhow!("email missing"))?,
                password: password.ok_or_else(|| anyhow!("pw missing"))?,
            })
        }
        "passkey" => Ok(Command::Passkey),
        "post" => {
            let quoted =
                extract_quoted(input).ok_or_else(|| anyhow!("post requires quoted text"))?;
            let audience = extract_named(input, "audience")
                .map(|s| AudienceScope::from_str(&s))
                .transpose()?
                .unwrap_or_default();
            Ok(Command::Post {
                text: quoted,
                audience,
            })
        }
        "feed" => {
            let filter = parts
                .next()
                .map(|s| FeedFilter::from_str(s))
                .transpose()?
                .unwrap_or_default();
            Ok(Command::Feed { filter })
        }
        "follow" => Ok(Command::Follow {
            handle: parts
                .next()
                .ok_or_else(|| anyhow!("missing handle"))?
                .trim_start_matches('@')
                .to_string(),
        }),
        "like" => Ok(Command::Like {
            post_id: parts
                .next()
                .ok_or_else(|| anyhow!("missing post id"))?
                .to_string(),
        }),
        "whoami" => Ok(Command::WhoAmI),
        "logout" => Ok(Command::Logout),
        "tasks" => parse_tasks(parts.collect::<Vec<_>>().as_slice()),
        "report" => Ok(Command::ReportSync),
        "settings" => parse_settings(parts.collect::<Vec<_>>().as_slice()),
        _ => Err(anyhow!("unknown command")),
    }
}

fn parse_tasks(args: &[&str]) -> Result<Command> {
    match args.first().copied() {
        Some("add") => {
            let text = args[1..].join(" ");
            let captures = extract_two_quotes(&text)
                .ok_or_else(|| anyhow!("usage: /tasks add \"title\" \"desc\""))?;
            Ok(Command::TasksAdd {
                title: captures.0,
                description: captures.1,
            })
        }
        Some("ls") => {
            let filter = args.get(1).and_then(|s| TaskStatus::from_str(s).ok());
            Ok(Command::TasksList { filter })
        }
        Some("done") => {
            let id = args
                .get(1)
                .ok_or_else(|| anyhow!("missing id"))?
                .parse::<i64>()?;
            Ok(Command::TasksDone { id })
        }
        _ => Err(anyhow!("unknown tasks command")),
    }
}

fn parse_settings(args: &[&str]) -> Result<Command> {
    match args.first().copied() {
        Some("show") => Ok(Command::SettingsShow),
        Some("set") => {
            let mut iter = args.iter().skip(1);
            let pair = iter
                .next()
                .ok_or_else(|| anyhow!("usage: /settings set key=value"))?;
            let mut split = pair.splitn(2, '=');
            let key = split.next().unwrap().to_string();
            let value = split
                .next()
                .ok_or_else(|| anyhow!("missing value"))?
                .to_string();
            Ok(Command::SettingsSet { key, value })
        }
        _ => Err(anyhow!("unknown settings command")),
    }
}

fn extract_quoted(input: &str) -> Option<String> {
    let re = Regex::new(r#""([^"]+)""#).ok()?;
    re.captures(input)
        .and_then(|cap| cap.get(1).map(|m| m.as_str().to_string()))
}

fn extract_named(input: &str, key: &str) -> Option<String> {
    let pattern = format!("{}:([\\w-]+)", key);
    let re = Regex::new(&pattern).ok()?;
    re.captures(input)
        .and_then(|cap| cap.get(1).map(|m| m.as_str().to_string()))
}

fn extract_two_quotes(input: &str) -> Option<(String, String)> {
    let re = Regex::new(r#""([^"]+)"\s+"([^"]+)""#).ok()?;
    re.captures(input).and_then(|cap| {
        let first = cap.get(1)?.as_str().to_string();
        let second = cap.get(2)?.as_str().to_string();
        Some((first, second))
    })
}

impl FromStr for TaskStatus {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "open" => Ok(TaskStatus::Open),
            "done" => Ok(TaskStatus::Done),
            _ => Err(anyhow!("invalid status")),
        }
    }
}

#[derive(Clone, Copy)]
pub struct CommandContext<'a> {
    pub db: &'a AppDatabase,
    pub supabase: &'a SupabaseClient,
}

impl<'a> CommandContext<'a> {
    pub fn new(db: &'a AppDatabase, supabase: &'a SupabaseClient) -> Self {
        Self { db, supabase }
    }
}

#[derive(Debug, Default)]
pub struct CommandOutput {
    pub message: String,
    pub feed: Option<Vec<crate::domain::Post>>,
}

pub async fn execute(ctx: CommandContext<'_>, command: Command) -> Result<CommandOutput> {
    match command {
        Command::Login { email, password } => {
            let session = ctx.supabase.login_email(&email, &password).await?;
            Ok(CommandOutput {
                message: format!("Logged in as {}", session.email),
                ..Default::default()
            })
        }
        Command::Passkey => {
            passkey_flow().await?;
            Ok(CommandOutput {
                message: "Passkey flow completed".to_string(),
                ..Default::default()
            })
        }
        Command::Post { text, audience } => {
            let session = ctx
                .supabase
                .ensure_session()
                .await?
                .ok_or_else(|| anyhow!("login required"))?;
            let post = ctx.supabase.create_post(&session, &text, audience).await?;
            let cache = CacheDao::new(ctx.db);
            cache.upsert_post(&post)?;
            Ok(CommandOutput {
                message: format!("Posted {}", post.id),
                ..Default::default()
            })
        }
        Command::Feed { filter } => {
            let session = ctx
                .supabase
                .ensure_session()
                .await?
                .ok_or_else(|| anyhow!("login required"))?;
            let posts = ctx.supabase.fetch_feed(&session, filter).await?;
            let cache = CacheDao::new(ctx.db);
            for post in &posts {
                cache.upsert_post(post)?;
            }
            Ok(CommandOutput {
                message: format!("Fetched {} posts", posts.len()),
                feed: Some(posts),
            })
        }
        Command::Follow { handle } => Ok(CommandOutput {
            message: format!("Follow request queued for @{}", handle),
            ..Default::default()
        }),
        Command::Like { post_id } => Ok(CommandOutput {
            message: format!("Like queued for {}", post_id),
            ..Default::default()
        }),
        Command::WhoAmI => {
            if let Some(session) = ctx.supabase.restore_session().await? {
                Ok(CommandOutput {
                    message: format!("Signed in as {}", session.email),
                    ..Default::default()
                })
            } else {
                Ok(CommandOutput {
                    message: "Not signed in".to_string(),
                    ..Default::default()
                })
            }
        }
        Command::Logout => {
            ctx.supabase.sessions.clear().await?;
            Ok(CommandOutput {
                message: "Session cleared".to_string(),
                ..Default::default()
            })
        }
        Command::TasksAdd { title, description } => {
            let dao = TaskDao::new(ctx.db);
            let task = dao.add(&title, &description)?;
            Ok(CommandOutput {
                message: format!("Task {} created", task.id),
                ..Default::default()
            })
        }
        Command::TasksList { filter } => {
            let dao = TaskDao::new(ctx.db);
            let tasks = dao.list(filter)?;
            let text = tasks
                .iter()
                .map(|task| format!("#{} [{}] {}", task.id, task.status.as_str(), task.title))
                .collect::<Vec<_>>()
                .join("\n");
            Ok(CommandOutput {
                message: text,
                ..Default::default()
            })
        }
        Command::TasksDone { id } => {
            let dao = TaskDao::new(ctx.db);
            if let Some(task) = dao.mark_done(id)? {
                let report = crate::reports::generate_report(ctx.db, &task)?;
                let dao = ReportDao::new(ctx.db);
                dao.insert(&report)?;
                Ok(CommandOutput {
                    message: format!("Task {} done; report: {}", task.id, report.path),
                    ..Default::default()
                })
            } else {
                Ok(CommandOutput {
                    message: "Task not found".to_string(),
                    ..Default::default()
                })
            }
        }
        Command::ReportSync => {
            let dao = ReportDao::new(ctx.db);
            let reports = dao.latest(10)?;
            Ok(CommandOutput {
                message: format!("Reports: {}", reports.len()),
                ..Default::default()
            })
        }
        Command::SettingsShow => {
            let conn = ctx.db.connection();
            let mut stmt = conn.prepare("SELECT key, value FROM settings ORDER BY key")?;
            let rows = stmt
                .query_map([], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })?
                .filter_map(Result::ok)
                .collect::<Vec<_>>();
            let mut map = HashMap::new();
            for (k, v) in rows {
                map.insert(k, v);
            }
            Ok(CommandOutput {
                message: format!("Settings: {:?}", map),
                ..Default::default()
            })
        }
        Command::SettingsSet { key, value } => {
            let conn = ctx.db.connection();
            conn.execute(
                "INSERT INTO settings(key, value) VALUES(?1, ?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
                rusqlite::params![key, value],
            )?;
            Ok(CommandOutput {
                message: "Setting saved".to_string(),
                ..Default::default()
            })
        }
    }
}

async fn passkey_flow() -> Result<()> {
    use tokio::net::TcpListener;

    let listener = TcpListener::bind(("127.0.0.1", 0)).await?;
    let port = listener.local_addr()?.port();
    let url = format!("http://localhost:{}/callback", port);
    tracing::info!(%url, "passkey callback listening");
    if let Err(err) = open::that_detached(&url) {
        tracing::warn!(?err, "failed to open browser");
    }
    let (mut socket, _) = listener.accept().await?;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let mut buf = vec![0u8; 4096];
    let n = socket.read(&mut buf).await?;
    let request = String::from_utf8_lossy(&buf[..n]);
    let token = request
        .split("session=")
        .nth(1)
        .and_then(|part| part.split_whitespace().next())
        .ok_or_else(|| anyhow!("no session token"))?;
    tracing::info!("received session {}", token.len());
    socket
        .write_all(b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nPasskey captured. You may close this window.")
        .await?;
    Ok(())
}
