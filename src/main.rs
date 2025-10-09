use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

use cli_twitter::{
    app, cfg,
    commands::{parse_command, CommandExecutor},
    data::DbPool,
    supabase::{SessionStore, SupabaseClient, SupabaseConfig},
    telemetry,
};

#[derive(Parser, Debug)]
#[command(author, version, about = "CLI Twitter TUI client")]
struct Cli {
    #[arg(long)]
    config: Option<PathBuf>,
    #[arg(long, help = "run a single slash command and exit")]
    run: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    telemetry::init()?;

    let config = cfg::AppConfig::load(cli.config.as_deref())?;
    if let Some(supabase_cfg) = config.supabase() {
        if let Some(url) = &supabase_cfg.url {
            std::env::set_var("SUPABASE_URL", url);
        }
        if let Some(key) = &supabase_cfg.anon_key {
            std::env::set_var("SUPABASE_ANON_KEY", key);
        }
    }

    let supabase_config = SupabaseConfig::from_env()?;
    let supabase_client = SupabaseClient::new(supabase_config.clone())?;
    let session_store = SessionStore::new(config.app_name())?;
    let mut session = session_store.load()?;
    let db = DbPool::new(config.app_name())?;
    let reports_dir = config.reports_dir()?;

    let executor = CommandExecutor::new(
        supabase_client.clone(),
        session_store.clone(),
        db.clone(),
        supabase_config.clone(),
        reports_dir.clone(),
    );

    if let Some(cmd) = cli.run.as_deref() {
        let command = parse_command(cmd)?;
        let outcome = executor.execute(&mut session, command).await?;
        println!("{}", outcome.status);
        if let Some(feed) = outcome.feed {
            for post in feed {
                println!("{}: {}", post.author, post.body);
            }
        }
        if let Some(tasks) = outcome.tasks {
            for task in tasks {
                println!("[{}] {} - {}", task.id, task.title, task.status);
            }
        }
        session_store.save(&session)?;
        return Ok(());
    }

    app::run_app(executor, session).await
}
