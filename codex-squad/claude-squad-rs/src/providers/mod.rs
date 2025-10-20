use std::pin::Pin;

use crate::domain::{Message, MessageRole, Profile, ProviderKind, UsageMetrics};
use anyhow::Result;
use async_stream::try_stream;
use futures::Stream;

pub mod anthropic;
pub mod openai;

pub type ProviderStream = Pin<Box<dyn Stream<Item = Result<ProviderEvent>> + Send>>;

#[derive(Debug, Clone)]
pub struct ProviderEvent {
    pub delta: Option<String>,
    pub done: bool,
    pub usage: Option<UsageMetrics>,
}

#[derive(Debug, Clone, Default)]
pub struct ProviderParams {
    pub stream: bool,
    pub system_prompt: Option<String>,
}

pub async fn stream_chat(
    profile: &Profile,
    messages: &[Message],
    params: ProviderParams,
) -> Result<ProviderStream> {
    match profile.provider {
        ProviderKind::Anthropic => anthropic::stream(profile, messages, params).await,
        ProviderKind::OpenAi => openai::stream(profile, messages, params, None).await,
        ProviderKind::OpenAiCompat => {
            let base_url = profile
                .metadata
                .get("base_url")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            openai::stream(profile, messages, params, base_url).await
        }
    }
}

fn build_offline_stream(messages: &[Message]) -> ProviderStream {
    let last_user = messages
        .iter()
        .rev()
        .find(|m| matches!(m.role, MessageRole::User));
    let response = last_user
        .map(|m| format!("Echo: {}", m.body))
        .unwrap_or_else(|| "No input".to_string());
    Box::pin(try_stream! {
        for chunk in response.split_whitespace() {
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            yield ProviderEvent { delta: Some(format!("{} ", chunk)), done: false, usage: None };
        }
        yield ProviderEvent { delta: None, done: true, usage: Some(UsageMetrics { prompt_tokens: 0, completion_tokens: 0, total_tokens: 0 }) };
    })
}

pub(crate) fn offline_stream(messages: &[Message]) -> ProviderStream {
    build_offline_stream(messages)
}
