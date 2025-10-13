#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."
if [[ -z "${ANDROID_HOME:-}" ]]; then
  echo "ANDROID_HOME is not set."
  exit 1
fi

cargo test --manifest-path src-tauri/Cargo.toml --test smoke -- --nocapture
cargo tauri android dev --emulator --ci --target aarch64-linux-android -- --no-window &
PID=$!
sleep 20
kill $PID || true
