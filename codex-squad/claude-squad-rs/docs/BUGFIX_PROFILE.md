# Codex Profile Bugfix

## Summary

The legacy project ignored the selected profile when launched via `codex-cli`. Regardless of the `CODEX_PROFILE` environment variable or CLI flag, the first profile in `profiles.yaml` was applied, causing mismatched providers/models.

## Root Cause

- Profile resolution was split across the CLI wrapper and the UI bootstrapper, with the codex integration executing after the local configuration was already chosen.
- Precedence rules were implicit and fell back to `profiles[0]` before codex overrides could run.

## Fix

- Added `config::ExecutionContext` to centralize profile resolution.
- Implemented explicit precedence (`CLI > ENV > codex config > default`) and codex profile loading in `integrate::codex_cli`.
- Exposed the logic through `ExecutionOverrides`, ensuring both the CLI and TUI use the same resolution path.

## Verification

- New integration tests (`tests/config_precedence.rs`) cover CLI and environment overrides as well as codex config-only scenarios.
- `scripts/smoke.sh` runs a headless chat session with `--profile pro` and asserts the resulting profile/model pair, preventing regressions.
