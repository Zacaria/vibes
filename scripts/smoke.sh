#!/usr/bin/env bash
set -euo pipefail
if [ $# -lt 2 ]; then
  echo "usage: scripts/smoke.sh <email> <password>" >&2
  exit 1
fi
EMAIL="$1"
PASSWORD="$2"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/.."
CMD_LOGIN="/login email:${EMAIL} pw:${PASSWORD}"
CMD_POST="/post \"smoke test $(date +%s)\" audience:public"
cargo run -- --run "$CMD_LOGIN"
cargo run -- --run "$CMD_POST"
