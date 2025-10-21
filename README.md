# cli-twitter

A terminal-first, Supabase-backed micro-blogging client written in Rust using Ratatui. It provides a slash-command driven interface for authentication, posting, reading the feed, and managing a local "project plan" stored in SQLite with automatic Markdown reporting.

## Features

- Ratatui-based TUI with command palette, feed, and task sidebar.
- Supabase authentication via email/password or passkey (WebAuthn) flow.
- Audience-aware posting (`public`, `restrained`, `private`).
- Slash commands inspired by Claude/Codex CLIs.
- Local SQLite cache for posts/profiles and project planning tasks.
- Reports generated on task completion and stored under `reports/`.
- Persisted sessions using system keyring or encrypted file fallback.
- Structured logging via `tracing`.

## Prerequisites

- Rust 1.80+
- Supabase project with GoTrue/PostgREST enabled
- Supabase CLI (for applying SQL and policies)
- `cargo`, `sqlite3` (optional for inspection)

## Supabase Setup

1. Export your Supabase credentials:
   ```bash
   export SUPABASE_URL=https://<project>.supabase.co
   export SUPABASE_ANON_KEY=<anon key>
   export SUPABASE_SERVICE_ROLE=<service role key>
   ```
2. Apply the schema and RLS policies:
   ```bash
   supabase db push --file supabase/remote.sql
   supabase db push --file supabase/rls.sql
   ```
3. Configure GoTrue email templates or providers as needed.
4. Ensure passkey authentication is enabled in your Supabase dashboard (Auth > Settings > Passwordless > Passkeys).

## Local Configuration

Copy the configuration templates:

```bash
cp .env.example .env
cp config/config.toml.example config/config.toml
```

Edit `config/config.toml` to match your environment. The application resolves its data directory via [`directories`](https://docs.rs/directories).

## Development

Install dependencies and run the TUI:

```bash
scripts/dev.sh
```

Run formatting, linting, and tests:

```bash
scripts/test.sh
```

### Slash Command Reference

- `/login email:<addr> pw:<secret>`
- `/passkey`
- `/post "message" audience:public|restrained|private`
- `/feed [global|following|me]`
- `/follow @handle`
- `/like <post_id>`
- `/whoami`
- `/logout`
- `/tasks add "Title" "Description"`
- `/tasks ls [open|done|all]`
- `/tasks done <id>`
- `/report sync`
- `/settings show`
- `/settings set key=value`

Reports are written to the directory returned by `config.reports_dir()` (defaults to `$APPDIR/reports`).

## Smoke Test

A non-interactive smoke test can log in and post via the `--run` flag:

```bash
cargo run -- --run '/login email:you@example.com pw:supersecret'
cargo run -- --run '/post "hello from CI" audience:public'
```

## Build & Run

The project builds as a standard cargo binary:

```bash
cargo build
cargo run
```

Invoke the TUI and interact using the slash command prompt at the bottom. Press `Esc` to toggle between navigation and command mode. `Ctrl+C` or `q` exits.

## Licensing

Released under the MIT License. See [`LICENSE`](LICENSE).
