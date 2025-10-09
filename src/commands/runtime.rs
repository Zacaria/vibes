use std::{fs, path::PathBuf};

use anyhow::{anyhow, Context, Result};
use rusqlite;
use time::OffsetDateTime;
use tokio::task;

use crate::{
    data::{
        cache::CacheDao, reports::write_report_file, reports::ReportsDao, tasks::TaskDao, DbPool,
    },
    domain::{now_rfc3339, FeedFilter, Post, Session, Task, TaskStatus},
    supabase::{passkey, SessionStore, SupabaseClient, SupabaseConfig},
};

use super::Command;

#[derive(Clone)]
pub struct CommandExecutor {
    supabase: SupabaseClient,
    session_store: SessionStore,
    db: DbPool,
    config: SupabaseConfig,
    reports_dir: PathBuf,
}

impl CommandExecutor {
    pub fn new(
        supabase: SupabaseClient,
        session_store: SessionStore,
        db: DbPool,
        config: SupabaseConfig,
        reports_dir: PathBuf,
    ) -> Self {
        Self {
            supabase,
            session_store,
            db,
            config,
            reports_dir,
        }
    }

    pub async fn execute(&self, session: &mut Session, command: Command) -> Result<CommandOutcome> {
        match command {
            Command::Login { email, password } => {
                let new_session = self
                    .supabase
                    .login_email_password(&email, &password)
                    .await
                    .context("login via Supabase")?;
                self.session_store.save(&new_session)?;
                *session = new_session.clone();
                let feed = self
                    .fetch_and_cache_feed(&new_session, FeedFilter::Global)
                    .await?;
                Ok(CommandOutcome::with_feed(
                    format!("logged in as {}", email),
                    feed,
                ))
            }
            Command::Passkey => {
                let new_session =
                    passkey::run_passkey_flow(&self.config, &self.supabase, &self.session_store)
                        .await
                        .context("passkey flow")?;
                *session = new_session.clone();
                let feed = self
                    .fetch_and_cache_feed(&new_session, FeedFilter::Global)
                    .await?;
                Ok(CommandOutcome::with_feed("passkey login complete", feed))
            }
            Command::Post { body, audience } => {
                self.ensure_auth(session)?;
                let post = self
                    .supabase
                    .create_post(session, &body, audience.clone())
                    .await?;
                self.cache_posts(vec![post.clone()]).await?;
                Ok(CommandOutcome::new(format!(
                    "posted {} characters to {}",
                    body.chars().count(),
                    audience
                )))
            }
            Command::Feed { filter } => {
                self.ensure_auth(session)?;
                let feed = self.fetch_and_cache_feed(session, filter.clone()).await?;
                Ok(CommandOutcome::with_feed(
                    format!("refreshed {} feed", filter.as_str()),
                    feed,
                ))
            }
            Command::Follow { handle } => {
                self.ensure_auth(session)?;
                self.supabase.follow(session, &handle).await?;
                Ok(CommandOutcome::new(format!("followed {}", handle)))
            }
            Command::Like { post_id } => {
                self.ensure_auth(session)?;
                self.supabase.like(session, &post_id).await?;
                Ok(CommandOutcome::new(format!("liked post {}", post_id)))
            }
            Command::WhoAmI => {
                self.ensure_auth(session)?;
                let profile = self.supabase.current_user(session).await?;
                Ok(CommandOutcome::new(format!(
                    "you are {} ({})",
                    profile.display_name.as_deref().unwrap_or(&profile.handle),
                    profile.handle
                )))
            }
            Command::Logout => {
                if session.is_authenticated() {
                    self.supabase.logout(session).await.ok();
                }
                self.session_store.clear()?;
                *session = Session::default();
                Ok(CommandOutcome::new("logged out"))
            }
            Command::TasksAdd { title, description } => {
                self.add_task(&title, &description).await?;
                let tasks = self.list_tasks(None).await?;
                Ok(CommandOutcome::with_tasks(
                    "created task".to_string(),
                    tasks,
                ))
            }
            Command::TasksList { status } => {
                let tasks = self.list_tasks(status.clone()).await?;
                Ok(CommandOutcome::with_tasks(
                    format!("listed {} tasks", tasks.len()),
                    tasks,
                ))
            }
            Command::TasksDone { id } => {
                let before = self.cache_counts().await?;
                let task = self.complete_task(id).await?;
                let after = self.cache_counts().await?;
                let report_path = self
                    .write_task_report(&task, before, after)
                    .await
                    .context("write task report")?;
                let tasks = self.list_tasks(None).await?;
                Ok(CommandOutcome::with_tasks(
                    format!(
                        "task {} done; report saved to {}",
                        id,
                        report_path.display()
                    ),
                    tasks,
                ))
            }
            Command::ReportSync => {
                let path = self.sync_reports().await?;
                Ok(CommandOutcome::new(format!(
                    "report synced: {}",
                    path.display()
                )))
            }
            Command::SettingsShow => {
                let settings = self.read_settings().await?;
                let mut lines = Vec::new();
                for (k, v) in settings {
                    lines.push(format!("{} = {}", k, v));
                }
                Ok(CommandOutcome::new(lines.join("\n")))
            }
            Command::SettingsSet { key, value } => {
                self.write_setting(&key, &value).await?;
                Ok(CommandOutcome::new(format!("setting {} updated", key)))
            }
            Command::Help => Ok(CommandOutcome::new(self.help_text())),
        }
    }

    async fn fetch_and_cache_feed(
        &self,
        session: &Session,
        filter: FeedFilter,
    ) -> Result<Vec<Post>> {
        let posts = self
            .supabase
            .fetch_feed(filter.clone(), session)
            .await
            .context("fetch feed")?;
        self.cache_posts(posts.clone()).await?;
        Ok(posts)
    }

    async fn cache_posts(&self, posts: Vec<Post>) -> Result<()> {
        if posts.is_empty() {
            return Ok(());
        }
        let db = self.db.clone();
        let posts_clone = posts;
        task::spawn_blocking(move || {
            db.with_conn(|conn| {
                let dao = CacheDao::new(conn);
                dao.cache_posts(&posts_clone)
            })
        })
        .await??;
        Ok(())
    }

    pub async fn cached_feed(&self, limit: usize) -> Result<Vec<Post>> {
        let db = self.db.clone();
        let posts = task::spawn_blocking(move || {
            db.with_conn(|conn| CacheDao::new(conn).list_cached_posts(limit))
        })
        .await??;
        Ok(posts)
    }

    async fn add_task(&self, title: &str, description: &str) -> Result<Task> {
        let title = title.to_string();
        let description = description.to_string();
        let db = self.db.clone();
        let task = task::spawn_blocking(move || {
            db.with_conn(|conn| TaskDao::new(conn).add_task(&title, &description))
        })
        .await??;
        Ok(task)
    }

    async fn list_tasks(&self, status: Option<TaskStatus>) -> Result<Vec<Task>> {
        let db = self.db.clone();
        let status_clone = status.clone();
        let tasks = task::spawn_blocking(move || {
            db.with_conn(|conn| TaskDao::new(conn).list_tasks(status_clone))
        })
        .await??;
        Ok(tasks)
    }

    async fn complete_task(&self, id: i64) -> Result<Task> {
        let db = self.db.clone();
        let task = task::spawn_blocking(move || {
            db.with_conn(|conn| {
                TaskDao::new(conn)
                    .mark_done(id)?
                    .ok_or_else(|| anyhow!("task {} not found", id))
            })
        })
        .await??;
        Ok(task)
    }

    async fn cache_counts(&self) -> Result<CacheSnapshot> {
        let db = self.db.clone();
        let snapshot = task::spawn_blocking(move || {
            db.with_conn(|conn| {
                let dao = CacheDao::new(conn);
                dao.counts().map(CacheSnapshot::from)
            })
        })
        .await??;
        Ok(snapshot)
    }

    async fn write_task_report(
        &self,
        task: &Task,
        before: CacheSnapshot,
        after: CacheSnapshot,
    ) -> Result<PathBuf> {
        let filename = format!(
            "{}_task-{}.md",
            OffsetDateTime::now_utc()
                .format(&time::macros::format_description!("%Y%m%d_%H%M"))
                .unwrap(),
            task.id
        );
        let diff = after.diff(&before);
        let markdown = format!(
            "# Task Completion Report\n\n- Task ID: {}\n- Title: {}\n- Completed At: {}\n- Description: {}\n\n## Cache Delta\n{}\n",
            task.id,
            task.title,
            task.done_at.clone().unwrap_or_else(now_rfc3339),
            task.description,
            diff
        );
        let path = write_report_file(&self.reports_dir, &filename, &markdown)?;
        let db = self.db.clone();
        let path_string = path.to_string_lossy().to_string();
        let summary = format!(
            "Task {} done; cache delta {}",
            task.id,
            diff.replace('\n', ", ")
        );
        let task_id = task.id;
        task::spawn_blocking(move || {
            db.with_conn(|conn| {
                ReportsDao::new(conn).record_report(Some(task_id), &path_string, &summary)
            })
        })
        .await??;
        Ok(path)
    }

    async fn sync_reports(&self) -> Result<PathBuf> {
        let db = self.db.clone();
        let reports_dir = self.reports_dir.clone();
        let report = task::spawn_blocking(move || {
            db.with_conn(|conn| {
                let dao = ReportsDao::new(conn);
                let reports = dao.list_reports()?;
                let summary_path = reports_dir.join(format!(
                    "{}_sync.md",
                    OffsetDateTime::now_utc()
                        .format(&time::macros::format_description!("%Y%m%d_%H%M%S"))
                        .unwrap()
                ));
                let mut content = String::from("# Report Sync\n\n");
                for report in reports {
                    content.push_str(&format!("- [{}]({})\n", report.summary, report.path));
                }
                fs::create_dir_all(&reports_dir)?;
                fs::write(&summary_path, content)?;
                Ok::<PathBuf, anyhow::Error>(summary_path)
            })
        })
        .await??;
        Ok(report)
    }

    async fn read_settings(&self) -> Result<Vec<(String, String)>> {
        let db = self.db.clone();
        let settings = task::spawn_blocking(move || {
            db.with_conn(|conn| {
                let mut stmt = conn.prepare("SELECT key, value FROM settings ORDER BY key")?;
                let rows = stmt
                    .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
                    .collect::<rusqlite::Result<Vec<(String, String)>>>()?;
                Ok(rows)
            })
        })
        .await??;
        Ok(settings)
    }

    async fn write_setting(&self, key: &str, value: &str) -> Result<()> {
        let key = key.to_string();
        let value = value.to_string();
        let db = self.db.clone();
        task::spawn_blocking(move || {
            db.with_conn(|conn| {
                conn.execute(
                    "INSERT INTO settings(key, value) VALUES(?1, ?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
                    (key.as_str(), value.as_str()),
                )?;
                Ok(())
            })
        })
        .await??;
        Ok(())
    }

    fn ensure_auth(&self, session: &Session) -> Result<()> {
        if session.is_authenticated() {
            Ok(())
        } else {
            Err(anyhow!("please /login first"))
        }
    }

    fn help_text(&self) -> String {
        vec![
            "Available commands:",
            "/login email:<addr> pw:<secret>",
            "/passkey",
            "/post \"text\" audience:public|restrained|private",
            "/feed [global|following|me]",
            "/follow @handle",
            "/like <post_id>",
            "/whoami",
            "/logout",
            "/tasks add \"title\" \"desc\"",
            "/tasks ls [open|done|all]",
            "/tasks done <id>",
            "/report sync",
            "/settings show",
            "/settings set key=value",
        ]
        .join("\n")
    }
}

#[derive(Debug, Clone)]
pub struct CommandOutcome {
    pub status: String,
    pub feed: Option<Vec<Post>>,
    pub tasks: Option<Vec<Task>>,
}

impl CommandOutcome {
    pub fn new<S: Into<String>>(status: S) -> Self {
        Self {
            status: status.into(),
            feed: None,
            tasks: None,
        }
    }

    pub fn with_feed<S: Into<String>>(status: S, feed: Vec<Post>) -> Self {
        Self {
            status: status.into(),
            feed: Some(feed),
            tasks: None,
        }
    }

    pub fn with_tasks<S: Into<String>>(status: S, tasks: Vec<Task>) -> Self {
        Self {
            status: status.into(),
            feed: None,
            tasks: Some(tasks),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct CacheSnapshot {
    profiles: i64,
    posts: i64,
    follows: i64,
    likes: i64,
}

impl From<crate::data::cache::CacheCounts> for CacheSnapshot {
    fn from(value: crate::data::cache::CacheCounts) -> Self {
        Self {
            profiles: value.profiles,
            posts: value.posts,
            follows: value.follows,
            likes: value.likes,
        }
    }
}

impl CacheSnapshot {
    fn diff(&self, before: &CacheSnapshot) -> String {
        format!(
            "profiles: {:+}, posts: {:+}, follows: {:+}, likes: {:+}",
            self.profiles - before.profiles,
            self.posts - before.posts,
            self.follows - before.follows,
            self.likes - before.likes,
        )
    }
}
