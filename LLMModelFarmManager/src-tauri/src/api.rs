use indexmap::IndexMap;
use tauri::{AppHandle, State};
use tauri_plugin_sql::DbInstances;
use uuid::Uuid;

use crate::errors::AppError;
use crate::game::persistence;
use crate::game::{GameEngine, GameSpeed, GameState, Gpu};

const DB_URL: &str = "sqlite:game.db";

#[tauri::command]
pub async fn get_state(engine: State<'_, GameEngine>) -> GameState {
    engine.snapshot()
}

#[tauri::command]
pub async fn set_speed(engine: State<'_, GameEngine>, speed: GameSpeed) -> GameState {
    engine.set_speed(speed);
    engine.snapshot()
}

#[tauri::command]
pub async fn set_paused(engine: State<'_, GameEngine>, paused: bool) -> GameState {
    engine.set_paused(paused);
    engine.snapshot()
}

#[tauri::command]
pub async fn purchase_gpu(
    engine: State<'_, GameEngine>,
    rack_id: String,
    template: Gpu,
) -> Result<GameState, String> {
    let rack_uuid = Uuid::parse_str(&rack_id).map_err(|e| e.to_string())?;
    engine
        .purchase_gpu(rack_uuid, &template)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_game(
    engine: State<'_, GameEngine>,
    instances: State<'_, DbInstances>,
    slot_id: i64,
    name: String,
) -> Result<(), String> {
    let guard = instances.0.read().await;
    let pool = guard
        .get(DB_URL)
        .ok_or_else(|| AppError::Database("database not loaded".into()))?;
    persistence::save_slot(pool, slot_id, &name, &engine.snapshot())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn load_game(
    engine: State<'_, GameEngine>,
    instances: State<'_, DbInstances>,
    slot_id: i64,
) -> Result<GameState, String> {
    let guard = instances.0.read().await;
    let pool = guard
        .get(DB_URL)
        .ok_or_else(|| AppError::Database("database not loaded".into()))?;
    if let Some(state) = persistence::load_slot(pool, slot_id)
        .await
        .map_err(|e| e.to_string())?
    {
        let mut current = engine.state().write();
        *current = state.clone();
        return Ok(state);
    }
    Err("slot not found".into())
}

#[tauri::command]
pub async fn list_saves(
    instances: State<'_, DbInstances>,
) -> Result<Vec<IndexMap<String, serde_json::Value>>, String> {
    let guard = instances.0.read().await;
    let pool = guard
        .get(DB_URL)
        .ok_or_else(|| AppError::Database("database not loaded".into()))?;
    persistence::list_slots(pool)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn emit_tutorial(app: AppHandle) -> Result<(), String> {
    app.emit_all(
        "game://tutorial",
        &serde_json::json!({
            "steps": [
                "Welcome to LLM Model Farm Manager!",
                "Monitor KPIs on the dashboard.",
                "Use the market to expand capacity.",
                "Balance power, cooling, and workloads to stay profitable."
            ]
        }),
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn record_metric(
    instances: State<'_, DbInstances>,
    key: String,
    value: f64,
) -> Result<(), String> {
    let guard = instances.0.read().await;
    let pool = guard
        .get(DB_URL)
        .ok_or_else(|| AppError::Database("database not loaded".into()))?;
    persistence::record_telemetry(pool, &key, value)
        .await
        .map_err(|e| e.to_string())
}
