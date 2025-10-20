#!/usr/bin/env bash
set -euo pipefail

cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test --all
