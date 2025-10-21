use std::{io, time::Duration};

use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tokio::{self, sync::mpsc, time::interval};

use crate::{
    commands::{parse_command, Command, CommandError, CommandExecutor},
    domain::Session,
    ui,
};

mod state;

pub use state::{AppState, InputMode};

pub async fn run_app(executor: CommandExecutor, session: Session) -> Result<()> {
    let mut stdout = io::stdout();
    enable_raw_mode().context("enable raw mode")?;
    execute!(stdout, EnterAlternateScreen).context("enter alternate screen")?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend).context("init terminal")?;

    let guard = CleanupGuard;
    let result = run_loop(&mut terminal, executor, session).await;
    drop(guard);
    result
}

struct CleanupGuard;

impl Drop for CleanupGuard {
    fn drop(&mut self) {
        disable_raw_mode().ok();
        execute!(io::stdout(), LeaveAlternateScreen).ok();
    }
}

async fn run_loop(
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>,
    executor: CommandExecutor,
    session: Session,
) -> Result<()> {
    let mut app = AppState::new(session);
    if let Ok(outcome) = executor
        .execute(&mut app.session, Command::TasksList { status: None })
        .await
    {
        if let Some(tasks) = outcome.tasks {
            app.update_tasks(tasks);
        }
    }
    if let Ok(posts) = executor.cached_feed(50).await {
        app.update_feed(posts);
    }

    let (event_tx, mut event_rx) = mpsc::unbounded_channel();
    std::thread::spawn(move || {
        while let Ok(ev) = event::read() {
            if event_tx.send(ev).is_err() {
                break;
            }
        }
    });

    let mut tick = interval(Duration::from_millis(100));

    loop {
        terminal.draw(|f| ui::render(f, &app)).context("draw ui")?;
        tokio::select! {
            _ = tick.tick() => {},
            Some(ev) = event_rx.recv() => {
                if handle_event(ev, &mut app, &executor).await? {
                    break;
                }
            }
            else => break,
        }
    }

    Ok(())
}

async fn handle_event(
    event: Event,
    app: &mut AppState,
    executor: &CommandExecutor,
) -> Result<bool> {
    match event {
        Event::Key(key) => handle_key(key, app, executor).await,
        Event::Resize(_, _) => Ok(false),
        _ => Ok(false),
    }
}

async fn handle_key(key: KeyEvent, app: &mut AppState, executor: &CommandExecutor) -> Result<bool> {
    if key.modifiers.contains(KeyModifiers::CONTROL) && matches!(key.code, KeyCode::Char('c')) {
        return Ok(true);
    }
    match app.mode {
        InputMode::Command => handle_command_mode(key, app, executor).await,
        InputMode::Navigation => handle_nav_mode(key, app),
    }
}

async fn handle_command_mode(
    key: KeyEvent,
    app: &mut AppState,
    executor: &CommandExecutor,
) -> Result<bool> {
    match key.code {
        KeyCode::Esc => {
            app.mode = InputMode::Navigation;
            app.set_status("Navigation mode. Press / to enter command mode.");
        }
        KeyCode::Enter => {
            let line = app.command_input.trim().to_string();
            if line.is_empty() {
                return Ok(false);
            }
            match parse_command(&line) {
                Ok(command) => match executor.execute(&mut app.session, command).await {
                    Ok(outcome) => {
                        if let Some(feed) = outcome.feed {
                            app.update_feed(feed);
                        }
                        if let Some(tasks) = outcome.tasks {
                            app.update_tasks(tasks);
                        }
                        app.set_status(outcome.status);
                        app.command_input.clear();
                    }
                    Err(err) => {
                        app.set_status(format!("error: {}", err));
                    }
                },
                Err(CommandError::NotACommand) => {
                    app.set_status("Commands must start with /");
                }
                Err(err) => {
                    app.set_status(format!("{}", err));
                }
            }
        }
        KeyCode::Backspace => {
            app.command_input.pop();
        }
        KeyCode::Char(c) => {
            app.command_input.push(c);
        }
        KeyCode::Left => {
            // ignore
        }
        KeyCode::Right => {}
        KeyCode::Up => {}
        KeyCode::Down => {}
        _ => {}
    }
    Ok(false)
}

fn handle_nav_mode(key: KeyEvent, app: &mut AppState) -> Result<bool> {
    match key.code {
        KeyCode::Esc => {
            app.mode = InputMode::Command;
            app.set_status("Command mode");
        }
        KeyCode::Char('/') => {
            app.mode = InputMode::Command;
            app.command_input.clear();
            app.command_input.push('/');
        }
        KeyCode::Char('q') => return Ok(true),
        _ => {}
    }
    Ok(false)
}
