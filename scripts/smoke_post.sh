#!/usr/bin/env bash
set -euo pipefail

EMAIL=${1:?"email required"}
PASSWORD=${2:?"password required"}
MESSAGE=${3:-"Hello from smoke test"}
AUDIENCE=${4:-public}

cmd="/login email:${EMAIL} pw:${PASSWORD}"
cargo run -- --command "$cmd"
cmd2="/post \"${MESSAGE}\" audience:${AUDIENCE}"
cargo run -- --command "$cmd2"
