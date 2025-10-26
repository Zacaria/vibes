use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: OffsetDateTime,
}

impl SessionTokens {
    pub fn is_expired(&self) -> bool {
        self.expires_at <= OffsetDateTime::now_utc()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Session {
    pub user_id: String,
    pub email: String,
    pub tokens: SessionTokens,
}

impl Session {
    pub fn is_valid(&self) -> bool {
        !self.tokens.is_expired()
    }
}
