use anyhow::Error;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Profile {
    pub name: String,
    pub provider: ProviderKind,
    pub model: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub top_p: Option<f32>,
    #[serde(default)]
    pub system_prompt: Option<String>,
    #[serde(default)]
    pub tools: Vec<ToolConfig>,
    #[serde(default)]
    pub attachments: AttachmentPolicy,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub default: bool,
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Squad {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub members: Vec<Member>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Member {
    pub name: String,
    pub role: String,
    pub profile: String,
    #[serde(default)]
    pub system_prompt: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProviderKind {
    #[default]
    Anthropic,
    OpenAi,
    OpenAiCompat,
}

impl FromStr for ProviderKind {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "anthropic" | "claude" => ProviderKind::Anthropic,
            "openai" => ProviderKind::OpenAi,
            "openai_compat" | "compat" => ProviderKind::OpenAiCompat,
            other => return Err(Error::msg(format!("unknown provider: {}", other))),
        })
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum AttachmentPolicy {
    #[default]
    None,
    Upload,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ToolConfig {
    pub name: String,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub config: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Message {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub sender: String,
    pub role: MessageRole,
    pub body: String,
    pub created_at: OffsetDateTime,
    #[serde(default)]
    pub attachments: Vec<Attachment>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Attachment {
    pub id: Uuid,
    pub kind: AttachmentKind,
    pub name: String,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum AttachmentKind {
    #[default]
    File,
    Url,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ProviderModel {
    pub provider: ProviderKind,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConversationSummary {
    pub id: Uuid,
    pub title: String,
    pub updated_at: OffsetDateTime,
    pub message_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UsageMetrics {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Keymap {
    pub bindings: HashMap<String, String>,
}

impl Keymap {
    #[allow(dead_code)]
    pub fn lookup(&self, action: &str) -> Option<&String> {
        self.bindings.get(action)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConversationExport {
    pub summary: ConversationSummary,
    pub messages: Vec<Message>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ConversationImport {
    pub title: String,
    pub messages: Vec<Message>,
}
