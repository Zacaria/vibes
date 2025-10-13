#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."
pnpm -C app install
cargo install tauri-cli --locked || true
cargo tauri dev
