use anyhow::{Context, Result};
use rusqlite::{Connection, TransactionBehavior};

pub fn apply(conn: &mut Connection) -> Result<()> {
    let current_version: i64 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
    if current_version >= 1 {
        return Ok(());
    }
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    tx.execute_batch(include_str!("../../migrations/sqlite/001_init.sql"))
        .context("applying 001_init.sql")?;
    tx.pragma_update(None, "user_version", &1)?;
    tx.commit()?;
    Ok(())
}
