# Migration Notes

This document summarizes how the original TypeScript-based `claude-squad` project maps onto the Rust implementation.

## Entry Point

| Original (TS) | Rust Equivalent |
| ------------- | ---------------- |
| `src/index.ts` CLI parser using Commander | `src/main.rs` built with `clap` |
| `src/app.tsx` React TUI | `src/app/mod.rs` Ratatui state machine |

## Configuration

- YAML profile and squad files remain structurally identical. The loader moved from ad-hoc filesystem calls to `config::ConfigLoader` with `directories` crate support.
- Codex CLI integration now lives in `integrate::codex_cli`, centralizing precedence logic and providing typed Profile conversion.

## State & Storage

- Conversations that were previously stored in JSON flat files are now persisted in SQLite via `storage::Storage` and migrations in `migrations/sqlite/`.
- Message streaming uses async providers and `tokio` tasks instead of Node streams.

## UI Layout

- React components map to Ratatui widgets:
  - Sidebar ➜ `ui::sidebar`
  - Chat timeline ➜ `ui::chat`
  - Status/Context panel ➜ `ui::status`
  - Command line ➜ `ui::command`

## Providers

- Provider adapters live in `providers::anthropic` and `providers::openai`. They implement streaming when credentials exist and gracefully fall back to local echo responses otherwise, preserving the “always respond” behaviour from the original JS project.

## Commands & Tools

- Slash command parser moved from regex-based parsing to `commands::parse`, returning typed `Command` variants handled inside the TUI state machine.

## Headless Mode

- The Rust version introduces `app::run_headless` (invoked via `--no-ansi`), replacing the previous Node CLI’s dependence on TTY detection for scripted smoke tests.

Overall parity is maintained while leveraging Rust’s safety, async runtime, and native terminal UI libraries.
