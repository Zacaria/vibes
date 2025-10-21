#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."
cargo test --manifest-path src-tauri/Cargo.toml --test smoke -- --nocapture
pnpm -C app test
