use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Report {
    pub id: i64,
    pub task_id: Option<i64>,
    pub path: String,
    pub summary: String,
    pub created_at: OffsetDateTime,
}
