use indexmap::IndexMap;
use serde_json::json;
use tauri_plugin_sql::DbPool;
use time::OffsetDateTime;

use super::GameState;
use crate::errors::{AppError, AppResult};

pub async fn save_slot(db: &DbPool, slot_id: i64, name: &str, state: &GameState) -> AppResult<()> {
    let data = serde_json::to_string(state).map_err(|e| AppError::Serialization(e.to_string()))?;
    let created_at = OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap();
    db.execute(
        "INSERT INTO save_slots (id, name, created_at, data) VALUES ($1, $2, $3, $4)\n        ON CONFLICT(id) DO UPDATE SET name=$2, created_at=$3, data=$4",
        vec![json!(slot_id), json!(name), json!(created_at), json!(data)],
    )
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;
    Ok(())
}

pub async fn load_slot(db: &DbPool, slot_id: i64) -> AppResult<Option<GameState>> {
    let rows = db
        .select(
            "SELECT data FROM save_slots WHERE id = $1",
            vec![json!(slot_id)],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    if let Some(row) = rows.into_iter().next() {
        if let Some(value) = row.get("data") {
            let text = value
                .as_str()
                .ok_or_else(|| AppError::Serialization("invalid data column".into()))?;
            let state: GameState =
                serde_json::from_str(text).map_err(|e| AppError::Serialization(e.to_string()))?;
            return Ok(Some(state));
        }
    }
    Ok(None)
}

pub async fn list_slots(db: &DbPool) -> AppResult<Vec<IndexMap<String, serde_json::Value>>> {
    let rows = db
        .select(
            "SELECT id, name, created_at FROM save_slots ORDER BY id",
            vec![],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    Ok(rows)
}

pub async fn record_telemetry(db: &DbPool, key: &str, value: f64) -> AppResult<()> {
    let ts = OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap();
    db.execute(
        "INSERT INTO telemetry (ts, k, v) VALUES ($1, $2, $3)",
        vec![json!(ts), json!(key), json!(value)],
    )
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;
    Ok(())
}
