use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use super::AudienceScope;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Post {
    pub id: Uuid,
    pub author: Uuid,
    pub body: String,
    pub audience: AudienceScope,
    pub created_at: OffsetDateTime,
    pub author_handle: Option<String>,
    pub liked: bool,
    pub like_count: i64,
}

impl Post {
    pub fn new(author: Uuid, body: impl Into<String>, audience: AudienceScope) -> Self {
        Self {
            id: Uuid::new_v4(),
            author,
            body: body.into(),
            audience,
            created_at: OffsetDateTime::now_utc(),
            author_handle: None,
            liked: false,
            like_count: 0,
        }
    }
}
