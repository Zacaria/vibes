#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."
if [[ $(uname) != "Darwin" ]]; then
  echo "iOS smoke test must run on macOS."
  exit 1
fi

cargo test --manifest-path src-tauri/Cargo.toml --test smoke -- --nocapture
cargo tauri ios dev --ci --target aarch64-apple-ios-sim &
PID=$!
sleep 20
kill $PID || true
