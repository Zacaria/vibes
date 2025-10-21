use llm_model_farm_manager::game::balance::{
    DifficultyPreset, EnergyPreset, EventsPreset, FinancePreset, MarketPreset,
};
use llm_model_farm_manager::game::{persistence, GameState};
use sqlx::SqlitePool;
use tauri_plugin_sql::DbPool;

fn preset() -> DifficultyPreset {
    DifficultyPreset {
        name: "Test".into(),
        finance: FinancePreset {
            starting_cash: 5_000_000.0,
            success_cash_target: 10_000_000.0,
            sla_tolerance: 3,
        },
        energy: EnergyPreset {
            base_price_per_kwh: 0.18,
            carbon_target: 280.0,
        },
        market: MarketPreset {
            base_token_price: 40.0,
            demand_growth: 0.1,
        },
        events: EventsPreset {
            failure_multiplier: 1.0,
            random_seed: 1,
        },
    }
}

#[tokio::test]
async fn save_load_roundtrip() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    sqlx::query(
        "CREATE TABLE save_slots (id INTEGER PRIMARY KEY, name TEXT, created_at TEXT, data TEXT);",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query("CREATE TABLE telemetry (ts TEXT, k TEXT, v REAL);")
        .execute(&pool)
        .await
        .unwrap();
    let pool = DbPool::Sqlite(pool);

    let preset = preset();
    let state = GameState::bootstrap(&preset);
    persistence::save_slot(&pool, 1, "Test", &state)
        .await
        .unwrap();

    let loaded = persistence::load_slot(&pool, 1).await.unwrap().unwrap();
    assert_eq!(loaded.workloads.len(), state.workloads.len());
}
