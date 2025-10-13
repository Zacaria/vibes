#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod api;
mod errors;
mod game;
mod telemetry;

use api::{
    emit_tutorial, get_state, list_saves, load_game, purchase_gpu, record_metric, save_game,
    set_paused, set_speed,
};
use game::balance::DifficultyPreset;
use game::GameEngine;
use tauri::Manager;
use tauri_plugin_log::LogTarget;
use tauri_plugin_sql::{Builder as SqlBuilder, Migration, MigrationKind};
use telemetry::init as init_tracing;

fn load_difficulty() -> DifficultyPreset {
    DifficultyPreset::load_default().unwrap_or_else(|_| DifficultyPreset {
        name: "Default".into(),
        finance: game::balance::FinancePreset {
            starting_cash: 5_000_000.0,
            success_cash_target: 20_000_000.0,
            sla_tolerance: 3,
        },
        energy: game::balance::EnergyPreset {
            base_price_per_kwh: 0.18,
            carbon_target: 280.0,
        },
        market: game::balance::MarketPreset {
            base_token_price: 35.0,
            demand_growth: 0.08,
        },
        events: game::balance::EventsPreset {
            failure_multiplier: 1.0,
            random_seed: 42,
        },
    })
}

fn main() {
    init_tracing();

    let difficulty = load_difficulty();
    let migrations = vec![Migration {
        version: 1,
        description: "init",
        sql: include_str!("../../migrations/sqlite/001_init.sql"),
        kind: MigrationKind::Up,
    }];

    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::default()
                .targets([LogTarget::Stdout, LogTarget::LogDir])
                .build(),
        )
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(
            SqlBuilder::default()
                .add_migrations("sqlite:game.db", migrations)
                .build(),
        )
        .setup(move |app| {
            let engine = GameEngine::new(difficulty.clone());
            engine.start(app.handle());
            app.manage(engine);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_state,
            set_speed,
            set_paused,
            purchase_gpu,
            save_game,
            load_game,
            list_saves,
            emit_tutorial,
            record_metric
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
