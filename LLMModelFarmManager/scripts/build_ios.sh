#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."
pnpm -C app install
pnpm -C app build
cargo tauri ios build
