# Test Plan

## Automated

- `cargo test` – unit and integration tests.
  - `tests/config_precedence.rs` validates profile selection precedence across CLI, environment, and codex config.
- `cargo clippy -- -D warnings` – linting gate.
- `scripts/smoke.sh` – end-to-end headless run ensuring `--profile` is honored and the active profile/model are surfaced.

## Manual

1. Launch `cargo run -- chat` and verify the Ratatui interface renders with sidebar, chat timeline, context panel, and command bar.
2. Use `/profile pro` to switch profiles; observe status line and right panel update.
3. Send messages with and without API keys set to observe real vs. offline echo responses.
4. Run `cargo run -- list profiles` to confirm CLI output matches configuration files.
5. Execute `cargo run -- export <conversation>` after a chat session to generate JSON or Markdown exports.
