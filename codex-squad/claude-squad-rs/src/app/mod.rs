use std::fs;
use std::io::{self};
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::ExecutableCommand;
use futures::StreamExt;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use scopeguard::guard;
use tokio::sync::mpsc::{self, Sender};
use tui_input::backend::crossterm::EventHandler;
use uuid::Uuid;

use crate::commands::{self, Command};
use crate::config::ExecutionContext;
use crate::domain::{Message, MessageRole, Profile, ProviderKind, UsageMetrics};
use crate::providers::{self, ProviderEvent, ProviderParams};
use crate::ui;
use crate::util;

pub struct AppState {
    execution: ExecutionContext,
    profile_override: Option<String>,
    model_override: Option<String>,
    provider_override: Option<String>,
    system_override: Option<String>,
    streaming: bool,
    input: tui_input::Input,
    messages: Vec<Message>,
    status: String,
    conversation_id: Uuid,
    pending_assistant: Option<usize>,
    usage: UsageMetrics,
}

impl AppState {
    pub fn new(execution: ExecutionContext) -> Result<Self> {
        let conversation_id = execution
            .storage()
            .upsert_conversation("New Conversation")?;
        Ok(Self {
            execution,
            profile_override: None,
            model_override: None,
            provider_override: None,
            system_override: None,
            streaming: true,
            input: tui_input::Input::default(),
            messages: Vec::new(),
            status: "Ready".into(),
            conversation_id,
            pending_assistant: None,
            usage: UsageMetrics::default(),
        })
    }

    pub fn set_profile_override(&mut self, profile: String) {
        self.profile_override = Some(profile);
    }

    pub fn set_model_override(&mut self, model: String) {
        self.model_override = Some(model);
    }

    pub fn set_provider_override(&mut self, provider: String) {
        self.provider_override = Some(provider);
    }

    pub fn set_system_override(&mut self, system: String) {
        self.system_override = Some(system);
    }

    pub fn set_streaming(&mut self, streaming: bool) {
        self.streaming = streaming;
    }

    pub fn active_profile(&self) -> Result<Profile> {
        let mut profile = self
            .execution
            .active_profile(self.profile_override.as_deref())?;
        if let Some(model) = &self.model_override {
            profile.model = model.clone();
        }
        if let Some(provider) = &self.provider_override {
            if let Ok(kind) = provider.parse::<ProviderKind>() {
                profile.provider = kind;
            }
        }
        if let Some(system) = &self.system_override {
            profile.system_prompt = Some(system.clone());
        }
        Ok(profile)
    }

    pub fn status_line(&self) -> String {
        let profile = self.active_profile().unwrap_or_else(|_| Profile::default());
        format!(
            "{} | {} | streaming={} | {}",
            profile.name,
            profile.model,
            if self.streaming { "on" } else { "off" },
            self.status
        )
    }

    pub fn messages(&self) -> &[Message] {
        &self.messages
    }

    pub fn input(&self) -> &tui_input::Input {
        &self.input
    }

    pub fn execution(&self) -> &ExecutionContext {
        &self.execution
    }

    pub fn usage(&self) -> &UsageMetrics {
        &self.usage
    }

    fn append_message(&mut self, message: Message) {
        self.messages.push(message);
    }

    fn reset_input(&mut self) {
        self.input.reset();
    }

    fn update_status(&mut self, status: impl Into<String>) {
        self.status = status.into();
    }

    fn start_assistant_message(&mut self) {
        let msg = Message {
            id: Uuid::new_v4(),
            conversation_id: self.conversation_id,
            sender: "assistant".into(),
            role: MessageRole::Assistant,
            body: String::new(),
            created_at: util::now(),
            attachments: Vec::new(),
        };
        self.pending_assistant = Some(self.messages.len());
        self.messages.push(msg);
    }

    fn apply_delta(&mut self, delta: &str) {
        if let Some(idx) = self.pending_assistant {
            if let Some(message) = self.messages.get_mut(idx) {
                message.body.push_str(delta);
                message.created_at = util::now();
            }
        }
    }

    fn finalize_assistant_message(&mut self) {
        if let Some(idx) = self.pending_assistant.take() {
            if let Some(message) = self.messages.get(idx).cloned() {
                if let Err(err) = self.execution.storage().append_message(&message) {
                    self.update_status(format!("persist error: {}", err));
                }
            }
        }
    }
}

pub async fn run_tui(mut state: AppState) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    let mut terminal = guard(terminal, |mut term| {
        let _ = term.show_cursor();
        let _ = term
            .backend_mut()
            .execute(crossterm::event::DisableMouseCapture);
        let _ = term
            .backend_mut()
            .execute(crossterm::terminal::LeaveAlternateScreen);
        let _ = disable_raw_mode();
    });
    terminal.hide_cursor()?;

    let (tx, mut rx) = mpsc::channel(1024);

    loop {
        terminal.draw(|frame| {
            ui::layout::render(frame, &state);
        })?;

        while let Ok(event) = rx.try_recv() {
            match event {
                InternalEvent::Provider(event) => match (event.done, event.delta) {
                    (false, Some(delta)) => {
                        state.apply_delta(&delta);
                    }
                    (true, _) => {
                        state.finalize_assistant_message();
                        if let Some(usage) = event.usage {
                            state.usage = usage;
                        }
                        state.update_status("response complete");
                    }
                    _ => {}
                },
                InternalEvent::Status(status) => state.update_status(status),
            }
        }

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && handle_key_event(&mut state, key, &tx).await?
                {
                    break;
                }
            }
        }
    }

    Ok(())
}

pub async fn run_headless(state: AppState) -> Result<()> {
    let profile = state.active_profile()?;
    println!(
        "whoami: profile={} model={} provider={:?}",
        profile.name, profile.model, profile.provider
    );
    Ok(())
}

async fn handle_key_event(
    state: &mut AppState,
    key: KeyEvent,
    tx: &Sender<InternalEvent>,
) -> Result<bool> {
    match key.code {
        KeyCode::Esc => return Ok(true),
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Ok(true),
        KeyCode::Char('/') => {
            let ev = Event::Key(KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE));
            state.input.handle_event(&ev);
        }
        KeyCode::Enter => {
            let value = state.input.value().to_string();
            if value.starts_with('/') {
                handle_command(state, &value).await?;
            } else if !value.trim().is_empty() {
                submit_message(state, value, tx.clone()).await?;
            }
            state.reset_input();
        }
        KeyCode::Backspace
        | KeyCode::Char(_)
        | KeyCode::Tab
        | KeyCode::Left
        | KeyCode::Right
        | KeyCode::Up
        | KeyCode::Down => {
            let ev = Event::Key(key);
            state.input.handle_event(&ev);
        }
        _ => {}
    }
    Ok(false)
}

async fn handle_command(state: &mut AppState, command_text: &str) -> Result<()> {
    match commands::parse(command_text) {
        Ok(Command::Profile(name)) => {
            state.set_profile_override(name.clone());
            state.update_status(format!("Switched to profile {}", name));
        }
        Ok(Command::Model(model)) => {
            state.set_model_override(model.clone());
            state.update_status(format!("Switched to model {}", model));
        }
        Ok(Command::NewConversation) => {
            state.messages.clear();
            state.conversation_id = state
                .execution
                .storage()
                .upsert_conversation("New Conversation")?;
            state.update_status("Started new conversation");
        }
        Ok(Command::SystemPrompt(prompt)) => {
            state.set_system_override(prompt.clone());
            state.update_status("System prompt updated");
        }
        Ok(Command::ToggleStreaming) => {
            state.streaming = !state.streaming;
            state.update_status(format!("Streaming: {}", state.streaming));
        }
        Ok(Command::Help) => {
            state.update_status("Commands: /profile, /model, /new, /sys, /stream");
        }
        Ok(Command::Export(format)) => {
            let storage = state.execution.storage();
            match crate::storage::export::export_conversation(
                storage,
                &state.conversation_id.to_string(),
                format,
            ) {
                Ok(output) => {
                    let export_dir = state.execution.loader().config_dir().join("exports");
                    if let Err(err) = fs::create_dir_all(&export_dir) {
                        state.update_status(format!("export failed: {}", err));
                    } else {
                        let path =
                            export_dir.join(format!("conversation-{}.txt", state.conversation_id));
                        match fs::write(&path, output) {
                            Ok(()) => {
                                state.update_status(format!("Exported to {}", path.display()))
                            }
                            Err(err) => state.update_status(format!("export failed: {}", err)),
                        }
                    }
                }
                Err(err) => state.update_status(format!("Export failed: {}", err)),
            }
        }
        Ok(Command::Unknown(text)) => {
            state.update_status(format!("Unknown command: {}", text));
        }
        Err(err) => {
            state.update_status(format!("Command error: {}", err));
        }
    }
    Ok(())
}

async fn submit_message(
    state: &mut AppState,
    text: String,
    tx: Sender<InternalEvent>,
) -> Result<()> {
    let profile = state.active_profile()?;
    let message = Message {
        id: Uuid::new_v4(),
        conversation_id: state.conversation_id,
        sender: "you".into(),
        role: MessageRole::User,
        body: text.clone(),
        created_at: util::now(),
        attachments: Vec::new(),
    };
    state.append_message(message.clone());
    state.execution.storage().append_message(&message)?;
    let history = state.messages.clone();
    state.start_assistant_message();
    state.update_status("Awaiting provider...");
    let streaming = state.streaming;
    let system_override = state.system_override.clone();

    tokio::spawn(async move {
        if let Err(err) =
            stream_from_provider(profile, tx.clone(), history, streaming, system_override).await
        {
            let _ = tx
                .send(InternalEvent::Status(format!("Provider error: {}", err)))
                .await;
        }
    });
    Ok(())
}

async fn stream_from_provider(
    profile: Profile,
    tx: Sender<InternalEvent>,
    history: Vec<Message>,
    streaming: bool,
    system_override: Option<String>,
) -> Result<()> {
    let params = ProviderParams {
        stream: streaming,
        system_prompt: system_override,
    };
    let stream = providers::stream_chat(&profile, &history, params).await?;
    futures::pin_mut!(stream);
    while let Some(event) = stream.next().await {
        match event {
            Ok(ev) => {
                let done = ev.done;
                if tx.send(InternalEvent::Provider(ev)).await.is_err() {
                    break;
                }
                if done {
                    break;
                }
            }
            Err(err) => {
                return Err(err);
            }
        }
    }
    Ok(())
}

enum InternalEvent {
    Provider(ProviderEvent),
    Status(String),
}
