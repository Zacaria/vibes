# Claude Squad RS

Claude Squad RS is a Rust-native port of the [claude-squad](https://github.com/smtg-ai/claude-squad) assistant manager. It provides a terminal user interface built with [`ratatui`](https://github.com/ratatui-org/ratatui) and multi-provider chat orchestration.

## Features

- Multi-profile chat with provider/model overrides and persistent history.
- Squad roster management with per-member system prompts.
- Streaming assistant responses with offline echo fallback when API keys are absent.
- Slash commands for common actions (`/profile`, `/model`, `/new`, `/sys`, `/stream`, `/export`).
- Codex CLI integration with explicit precedence: CLI `--profile` > `CODEX_PROFILE` env > codex config active profile > local default.
- SQLite persistence (via `rusqlite`) including export/import utilities.
- Headless execution (`--no-ansi`) for scripting and smoke testing.

## Getting Started

```bash
cargo build
```

To launch the TUI:

```bash
cargo run -- chat
```

To run the headless smoke test harness:

```bash
scripts/smoke.sh
```

## Configuration

Configuration files live in `~/.config/claude-squad-rs` by default and may be overridden with `--config <dir>` or the `CLAUDE_SQUAD_CONFIG` environment variable. Example files are provided in `configs/examples/`.

- `profiles.yaml` – profile definitions with provider/model and optional metadata such as `api_key_env`.
- `squads.yaml` – named squads referencing profile names.
- `keymaps.toml` – custom key bindings for navigation.

Secrets are read from environment variables or the operating system keychain. API keys are **never** written to disk.

## CLI Overview

```text
claude-squad-rs [FLAGS] <SUBCOMMAND>

Flags:
  --config <DIR>   Override configuration directory
  --profile <NAME> Override profile (highest precedence)
  --codex          Enable codex-cli integration
  --log-level      info|debug|trace
  --no-ansi        Run chat in headless mode (no TUI)

Subcommands:
  chat     Launch the TUI/headless chat experience
  list     List profiles, squads, or models
  export   Export a conversation to JSON or Markdown
  import   Import a conversation from JSON/YAML
  history  Show stored conversations with optional filtering
  config   Show config path or run a health check
```

## Keyboard Shortcuts

- `j` / `k` or `↑` / `↓` – scroll conversation
- `g` / `G` – jump to top/bottom
- `/` – focus command line
- `Enter` – send message
- `:` – (future) command palette placeholder
- `F2` – switch profile (via `/profile` command)
- `F3` – switch model (via `/model` command)
- `F5` – new chat (`/new`)
- `F10` – export (`/export`)

## Development

Use the included helper scripts to keep quality gates passing:

```bash
scripts/dev.sh
```

Continuous integration runs build, clippy (`-D warnings`), tests, and the smoke script on every push.
