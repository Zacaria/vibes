# Design Overview

LLM Model Farm Manager is a cross-platform management simulation built with Tauri v2 and Svelte. The architecture cleanly separates the deterministic simulation engine (Rust) from the presentation layer (Svelte) and keeps all persistence logic within the Rust backend.

## Backend (Rust)

* **Entry Point (`src-tauri/src/main.rs`)** – configures Tauri plugins (SQL, Store, Log), registers commands, and bootstraps the `GameEngine` with difficulty presets and migrations.
* **Game Engine (`src-tauri/src/game`)** – owns the authoritative `GameState`, a fixed-timestep loop, and the domain models:
  * `scheduler.rs` computes GPU utilisation per tick with inference priority.
  * `economy.rs` calculates revenue, electricity usage, depreciation, and updates KPIs.
  * `events.rs` injects random incidents (failures, demand spikes, SLA penalties) and emits Tauri events.
  * `persistence.rs` writes save slots and telemetry through `tauri-plugin-sql`.
  * `balance.rs` loads difficulty presets and encapsulates balancing numbers.
* **Telemetry (`telemetry.rs`)** – initialises structured logging via `tracing`.
* **API (`api.rs`)** – exposes commands for the frontend (state snapshots, speed toggles, purchases, saves) and reuses persistence helpers.
* **Tests** – unit and integration tests cover economy math, scheduler behaviour, persistence round-trips, and smoke-playthrough logic.

The game loop runs inside a Tokio task, advancing the simulation by 5 in-game minutes per tick and publishing snapshots over a broadcast channel. All commands manipulate the shared `GameState` through a `parking_lot::RwLock`, ensuring thread-safe access with minimal locking overhead.

## Frontend (Svelte + Vite)

* **State Management (`app/src/lib/store.ts`)** – wraps Tauri commands, subscribes to tick events, maintains autosave timers, and exposes derived KPI stores.
* **API Layer (`app/src/lib/api.ts`)** – a thin wrapper around `@tauri-apps/api` invoke/event helpers, keeping transport details out of components.
* **UI Layout (`App.svelte`)** – mobile-friendly sidebar navigation with seven primary screens (Dashboard, Farm, Workloads, Market, Finance, Events, Settings).
* **Components** – reusable KPI cards, sparklines, and hierarchical farm tree visualisations.
* **Testing** – Vitest suites mock the API layer to validate store behaviour and ensure reactive updates.

## Persistence

SQLite stores save slots, telemetry, and metadata. Migrations live under `migrations/sqlite` and are registered with the SQL plugin so they execute automatically on startup. Game state is serialised as JSON to retain compatibility across mobile platforms.

## Build & CI

The CI workflow installs Rust, pnpm, builds the frontend, runs Vitest, executes `cargo clippy -- -D warnings`, and runs Rust tests. Scripts in `scripts/` streamline local development and smoke testing on desktop, Android, and iOS.

