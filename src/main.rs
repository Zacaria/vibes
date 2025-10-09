mod app;
mod cfg;
mod commands;
mod data;
mod domain;
mod reports;
mod supabase;
mod telemetry;
mod ui;

use anyhow::Result;
use clap::Parser;
use tracing::info;

use app::App;
use data::{AppDatabase, DatabaseConfig};
use supabase::{SessionStore, SupabaseClient, SupabaseConfig};

#[derive(Parser, Debug)]
#[command(author, version, about = "Terminal Twitter client powered by Supabase")]
struct Cli {
    /// Run command in non-interactive mode
    #[arg(long)]
    command: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    telemetry::init();
    let cli = Cli::parse();
    let cfg = cfg::load_config()?;
    if let Some(feed) = cfg.default_feed.as_deref() {
        info!("default_feed" = %feed, "configuration loaded");
    }
    let db_cfg = DatabaseConfig::resolve()?;
    let db = AppDatabase::open(&db_cfg)?;
    let session_store = SessionStore::new()?;
    let sb_cfg = SupabaseConfig::from_env()?;
    let supabase = SupabaseClient::new(sb_cfg, session_store)?;

    if let Some(cmd) = cli.command {
        if cmd.starts_with('/') {
            let command = commands::parse_command(&cmd)?;
            let output =
                commands::execute(commands::CommandContext::new(&db, &supabase), command).await?;
            println!("{}", output.message);
            return Ok(());
        }
    }

    let app = App::new(&db, &supabase)?;
    app.run().await
}
