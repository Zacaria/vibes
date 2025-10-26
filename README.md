# cli-twitter

`cli-twitter` is a cross-platform Rust terminal application that offers a Twitter-like experience in your terminal. It integrates with Supabase for authentication and social features while keeping a local SQLite project planner to track tasks and reports.

## Features

- Supabase email/password and passkey authentication
- Ratatui-based TUI with command palette inspired by slash commands
- Offline cache of posts, profiles, follows, and likes
- Local SQLite task planner with automatic report generation
- Structured logging with `tracing`
- Configurable via environment variables and TOML config files

## Requirements

- Rust 1.80+
- Supabase project with the schema in `supabase/remote.sql` and policies in `supabase/rls.sql`
- `supabase` CLI for managing remote database

## Setup

1. Copy `.env.example` to `.env` and fill in your Supabase credentials.
2. Apply the SQL schema and RLS policies:

```bash
supabase db push --file supabase/remote.sql
supabase db push --file supabase/rls.sql
```

3. Install Rust dependencies and build:

```bash
cargo build
```

4. Run the TUI:

```bash
cargo run
```

Slash command examples:

- `/login email:me@example.com pw:mypassword`
- `/post "Hello world" audience:public`
- `/feed global`
- `/tasks add "Implement offline mode" "Ensure cache handles network outages"`
- `/tasks done 1`

## Reports

Task completion generates markdown files under `reports/` and tracks them in the SQLite database.

## Configuration

Place a TOML file in `$CONFIG_DIR/cli-twitter/config.toml` or set `CLI_TWITTER_CONFIG` with fields shown in `config/config.toml.example`.

## Tests

Run unit tests with:

```bash
cargo test
```

## License

This project is licensed under the MIT License. See `LICENSE` for details.
