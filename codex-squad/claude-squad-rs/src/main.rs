mod app;
mod commands;
mod config;
mod domain;
mod errors;
mod integrate;
mod providers;
mod storage;
mod telemetry;
mod ui;
mod util;

use std::path::PathBuf;

use anyhow::Result;
use clap::{ArgAction, Parser, Subcommand, ValueEnum};
use config::ConfigLoader;
use storage::export::ExportFormat;
use telemetry::TelemetryGuard;
use tracing::{debug, info};

#[derive(Parser, Debug)]
#[command(
    name = "claude-squad-rs",
    version,
    about = "Claude Squad TUI client in Rust"
)]
struct Cli {
    /// Optional configuration directory override.
    #[arg(long = "config", value_name = "DIR", global = true)]
    config: Option<PathBuf>,

    /// Profile override.
    #[arg(long = "profile", value_name = "NAME", global = true)]
    profile: Option<String>,

    /// Run inside codex-cli context.
    #[arg(long = "codex", global = true, action = ArgAction::SetTrue)]
    codex: bool,

    /// Log level override.
    #[arg(long = "log-level", value_enum, default_value = "info", global = true)]
    log_level: LogLevel,

    /// Disable ANSI output for non-TUI commands.
    #[arg(long = "no-ansi", global = true, action = ArgAction::SetTrue)]
    no_ansi: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(ValueEnum, Copy, Clone, Debug, Default)]
enum LogLevel {
    #[default]
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Info => "info",
            LogLevel::Debug => "debug",
            LogLevel::Trace => "trace",
        }
    }
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Launch the interactive chat UI (default when no subcommand specified).
    Chat(ChatArgs),
    /// List entities from the configuration.
    List {
        #[command(subcommand)]
        kind: ListKind,
    },
    /// Export a stored conversation.
    Export {
        conversation_id: String,
        #[arg(long, value_enum)]
        format: ExportFormat,
    },
    /// Import a conversation history file.
    Import { file: PathBuf },
    /// Inspect stored history.
    History {
        #[arg(long)]
        filter: Option<String>,
    },
    /// Configuration helpers.
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Parser, Debug, Default)]
struct ChatArgs {
    #[arg(long)]
    profile: Option<String>,
    #[arg(long)]
    model: Option<String>,
    #[arg(long)]
    provider: Option<String>,
    #[arg(long)]
    system: Option<String>,
    #[arg(long = "system-file")]
    system_file: Option<PathBuf>,
    #[arg(long, action = ArgAction::SetTrue, default_value_t = true)]
    stream: bool,
}

#[derive(Subcommand, Debug)]
enum ListKind {
    Profiles,
    Squads,
    Models,
}

#[derive(Subcommand, Debug)]
enum ConfigAction {
    Path,
    Doctor,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install().expect("color-eyre install");
    let cli = Cli::parse();
    let _guard = TelemetryGuard::init(cli.log_level.as_str())?;

    let loader = ConfigLoader::discover(cli.config.clone())?;
    let execution = config::ExecutionContext::new(
        loader,
        config::ExecutionOverrides {
            profile: cli.profile.clone(),
            codex_profile: std::env::var("CODEX_PROFILE").ok(),
            codex_enabled: cli.codex,
        },
    );

    let mut app_state = app::AppState::new(execution.clone());

    let no_ansi = cli.no_ansi;

    match cli.command.unwrap_or(Commands::Chat(ChatArgs::default())) {
        Commands::Chat(args) => {
            debug!(?args, "launching chat UI");
            if let Some(profile) = args.profile {
                app_state.set_profile_override(profile);
            }
            if let Some(model) = args.model {
                app_state.set_model_override(model);
            }
            if let Some(provider) = args.provider {
                app_state.set_provider_override(provider);
            }
            if let Some(system) = args.system {
                app_state.set_system_override(system);
            }
            if let Some(system_file) = args.system_file {
                let text = std::fs::read_to_string(system_file)?;
                app_state.set_system_override(text);
            }
            app_state.set_streaming(args.stream);
            if no_ansi {
                app::run_headless(app_state).await?;
            } else {
                app::run_tui(app_state).await?;
            }
        }
        Commands::List { kind } => match kind {
            ListKind::Profiles => {
                let profiles = execution.load_profiles()?;
                for profile in profiles {
                    println!(
                        "{}\t{}\t{}",
                        profile.name,
                        profile.provider_string(),
                        profile.model
                    );
                }
            }
            ListKind::Squads => {
                let squads = execution.load_squads()?;
                for squad in squads {
                    println!("{}\t{} members", squad.name, squad.members.len());
                }
            }
            ListKind::Models => {
                for model in execution.list_models()? {
                    println!("{}", model);
                }
            }
        },
        Commands::Export {
            conversation_id,
            format,
        } => {
            let exported = storage::export::export_conversation(
                &execution.storage(),
                &conversation_id,
                format,
            )?;
            println!("{}", exported);
        }
        Commands::Import { file } => {
            let contents = std::fs::read_to_string(&file)?;
            let conversation_id =
                storage::import::import_conversation(&execution.storage(), &contents)?;
            println!("Imported conversation {}", conversation_id);
        }
        Commands::History { filter } => {
            let conversations = execution.storage().list_conversations(filter.as_deref())?;
            for convo in conversations {
                println!("{}\t{}\t{}", convo.id, convo.title, convo.updated_at);
            }
        }
        Commands::Config { action } => match action {
            ConfigAction::Path => {
                println!("{}", execution.loader().config_dir().display());
            }
            ConfigAction::Doctor => {
                let report = execution.diagnose()?;
                println!("{}", report);
            }
        },
    }

    info!("shutdown");
    Ok(())
}
