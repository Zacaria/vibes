use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AudienceScope {
    Public,
    Restrained,
    Private,
}

impl AudienceScope {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::Restrained => "restrained",
            Self::Private => "private",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseAudienceScopeError;

impl fmt::Display for ParseAudienceScopeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid audience scope")
    }
}

impl std::error::Error for ParseAudienceScopeError {}

impl FromStr for AudienceScope {
    type Err = ParseAudienceScopeError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "public" => Ok(Self::Public),
            "restrained" => Ok(Self::Restrained),
            "private" => Ok(Self::Private),
            _ => Err(ParseAudienceScopeError),
        }
    }
}

impl fmt::Display for AudienceScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FeedFilter {
    Global,
    Following,
    Me,
}

impl FeedFilter {
    pub fn from_optional_str(input: Option<&str>) -> Self {
        input
            .and_then(|s| s.parse::<FeedFilter>().ok())
            .unwrap_or(FeedFilter::Global)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            FeedFilter::Global => "global",
            FeedFilter::Following => "following",
            FeedFilter::Me => "me",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseFeedFilterError;

impl fmt::Display for ParseFeedFilterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid feed filter")
    }
}

impl std::error::Error for ParseFeedFilterError {}

impl FromStr for FeedFilter {
    type Err = ParseFeedFilterError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.to_ascii_lowercase().as_str() {
            "global" => Ok(FeedFilter::Global),
            "following" => Ok(FeedFilter::Following),
            "me" => Ok(FeedFilter::Me),
            _ => Err(ParseFeedFilterError),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub id: Uuid,
    pub handle: String,
    pub display_name: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: Uuid,
    pub author: Uuid,
    pub body: String,
    pub audience: AudienceScope,
    pub created_at: String,
    #[serde(default)]
    pub author_handle: Option<String>,
    #[serde(default)]
    pub like_count: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Session {
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub expires_at: Option<OffsetDateTime>,
    pub user_id: Option<Uuid>,
    #[serde(default)]
    pub email: Option<String>,
}

impl Session {
    pub fn is_authenticated(&self) -> bool {
        self.access_token.is_some() && self.user_id.is_some()
    }

    pub fn mask(&self) -> String {
        let access = self
            .access_token
            .as_ref()
            .map(|token| format!("{}…", &token[..token.len().min(4)]))
            .unwrap_or_else(|| "<none>".to_string());
        format!("Session(access={})", access)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    pub created_at: String,
    pub done_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Open,
    Done,
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskStatus::Open => write!(f, "open"),
            TaskStatus::Done => write!(f, "done"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub id: i64,
    pub task_id: Option<i64>,
    pub path: String,
    pub summary: String,
    pub created_at: String,
}

pub fn now_rfc3339() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}
