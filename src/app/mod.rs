mod state;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::{self, Stdout};
use std::time::{Duration, Instant};

use crate::commands::{self, CommandContext};
use crate::data::AppDatabase;
use crate::supabase::SupabaseClient;

use state::AppState;

pub struct App<'a> {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    state: AppState,
    ctx: CommandContext<'a>,
}

impl<'a> App<'a> {
    pub fn new(db: &'a AppDatabase, supabase: &'a SupabaseClient) -> Result<Self> {
        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        let state = AppState::new();
        Ok(Self {
            terminal,
            state,
            ctx: CommandContext::new(db, supabase),
        })
    }

    pub async fn run(mut self) -> Result<()> {
        enable_raw_mode()?;
        let mut last_tick = Instant::now();
        let tick_rate = Duration::from_millis(200);
        self.terminal.clear()?;
        loop {
            self.terminal.draw(|f| self.state.draw(f))?;
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));
            if event::poll(timeout)? {
                match event::read()? {
                    Event::Key(key) => {
                        if self.handle_key(key).await? {
                            break;
                        }
                    }
                    Event::Resize(width, height) => {
                        self.state.set_size(width, height);
                    }
                    _ => {}
                }
            }
            if last_tick.elapsed() >= tick_rate {
                self.state.tick();
                last_tick = Instant::now();
            }
        }
        disable_raw_mode()?;
        self.terminal.show_cursor()?;
        Ok(())
    }

    async fn handle_key(&mut self, key: KeyEvent) -> Result<bool> {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return Ok(true);
        }
        match key.code {
            KeyCode::Esc => {
                self.state.toggle_input_mode();
            }
            KeyCode::Char('/') if !self.state.is_editing() => {
                self.state.start_editing();
                self.state.push_char('/');
            }
            KeyCode::Enter => {
                if let Some(command) = self.state.take_command() {
                    match commands::parse_command(&command) {
                        Ok(cmd) => match commands::execute(self.ctx, cmd).await {
                            Ok(output) => {
                                if let Some(feed) = output.feed {
                                    self.state.update_feed(feed);
                                }
                                self.state.push_status(output.message);
                            }
                            Err(err) => self.state.push_status(format!("Error: {}", err)),
                        },
                        Err(err) => self.state.push_status(format!("Parse error: {}", err)),
                    }
                }
            }
            KeyCode::Backspace => {
                self.state.backspace();
            }
            KeyCode::Char(c) => {
                if self.state.is_editing() {
                    self.state.push_char(c);
                }
            }
            _ => {}
        }
        Ok(false)
    }
}
