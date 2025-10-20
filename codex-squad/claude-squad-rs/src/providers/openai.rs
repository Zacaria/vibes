use anyhow::Result;
use async_stream::try_stream;
use futures::StreamExt;
use reqwest::Client;
use serde::Serialize;

use crate::domain::{Message, MessageRole, Profile, UsageMetrics};
use crate::providers::{offline_stream, ProviderEvent, ProviderParams, ProviderStream};

#[derive(Serialize)]
struct ChatCompletionRequest<'a> {
    model: &'a str,
    messages: Vec<ChatCompletionMessage<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    stream: bool,
}

#[derive(Serialize)]
struct ChatCompletionMessage<'a> {
    role: &'a str,
    content: &'a str,
}

pub async fn stream(
    profile: &Profile,
    messages: &[Message],
    params: ProviderParams,
    base_url: Option<String>,
) -> Result<ProviderStream> {
    let env_var = profile
        .metadata
        .get("api_key_env")
        .and_then(|v| v.as_str())
        .unwrap_or("OPENAI_API_KEY");
    let api_key = std::env::var(env_var).ok();
    if api_key.is_none() {
        return Ok(offline_stream(messages));
    }
    let api_key = api_key.unwrap();
    let client = Client::builder().build()?;
    let body = ChatCompletionRequest {
        model: &profile.model,
        messages: messages
            .iter()
            .map(|m| ChatCompletionMessage {
                role: map_role(m.role),
                content: &m.body,
            })
            .collect(),
        temperature: profile.temperature,
        stream: params.stream,
    };
    let url = base_url.unwrap_or_else(|| "https://api.openai.com/v1/chat/completions".to_string());
    let resp = client
        .post(url)
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await?;
    if params.stream {
        let mut stream = resp.bytes_stream();
        Ok(Box::pin(try_stream! {
            while let Some(chunk) = stream.next().await {
                let chunk = chunk?;
                for line in String::from_utf8_lossy(&chunk).split('\n') {
                    if line.starts_with("data:") {
                        let token = line.trim_start_matches("data:").trim();
                        if token == "[DONE]" {
                            yield ProviderEvent { delta: None, done: true, usage: None };
                        } else if !token.is_empty() {
                            yield ProviderEvent { delta: Some(token.to_string()), done: false, usage: None };
                        }
                    }
                }
            }
            yield ProviderEvent { delta: None, done: true, usage: None };
        }))
    } else {
        let value: serde_json::Value = resp.json().await?;
        let text = value
            .pointer("/choices/0/message/content")
            .and_then(|v| v.as_str())
            .unwrap_or("(no response)")
            .to_string();
        Ok(Box::pin(try_stream! {
            yield ProviderEvent { delta: Some(text), done: false, usage: None };
            yield ProviderEvent { delta: None, done: true, usage: Some(UsageMetrics::default()) };
        }))
    }
}

fn map_role(role: MessageRole) -> &'static str {
    match role {
        MessageRole::User => "user",
        MessageRole::Assistant => "assistant",
        MessageRole::System => "system",
        MessageRole::Tool => "tool",
    }
}
