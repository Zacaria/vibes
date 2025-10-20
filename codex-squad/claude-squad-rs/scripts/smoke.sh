#!/usr/bin/env bash
set -euo pipefail

tmpdir=$(mktemp -d)
trap 'rm -rf "$tmpdir"' EXIT

cat >"$tmpdir/profiles.yaml" <<'YAML'
- name: default
  provider: anthropic
  model: claude-3-haiku
  default: true
- name: pro
  provider: openai
  model: gpt-4o
  metadata:
    api_key_env: OPENAI_API_KEY
YAML

cargo run --quiet -- chat --config "$tmpdir" --profile pro --no-ansi | tee "$tmpdir/out.txt"
if grep -q "profile=pro" "$tmpdir/out.txt" && grep -q "model=gpt-4o" "$tmpdir/out.txt"; then
  echo "PASS"
else
  echo "FAIL" >&2
  exit 1
fi
