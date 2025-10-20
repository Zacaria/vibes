use crate::domain::{Message, MessageRole, Profile, UsageMetrics};
use crate::providers::{offline_stream, ProviderEvent, ProviderParams, ProviderStream};
use anyhow::Result;
use async_stream::try_stream;
use futures::StreamExt;
use reqwest::Client;
use serde::Serialize;
use serde_json::Value;

#[derive(Serialize)]
struct AnthropicRequest<'a> {
    model: &'a str,
    #[serde(rename = "max_tokens")]
    max_tokens: i32,
    messages: Vec<AnthropicMessage<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<&'a str>,
    stream: bool,
}

#[derive(Serialize)]
struct AnthropicMessage<'a> {
    role: &'a str,
    content: &'a str,
}

pub async fn stream(
    profile: &Profile,
    messages: &[Message],
    params: ProviderParams,
) -> Result<ProviderStream> {
    let api_key = std::env::var(
        profile
            .metadata
            .get("api_key_env")
            .and_then(|v| v.as_str())
            .unwrap_or("ANTHROPIC_API_KEY"),
    )
    .ok();
    if api_key.is_none() {
        return Ok(offline_stream(messages));
    }
    let api_key = api_key.unwrap();
    let client = Client::builder().build()?;
    let body = AnthropicRequest {
        model: &profile.model,
        max_tokens: 1024,
        messages: messages
            .iter()
            .map(|m| AnthropicMessage {
                role: map_role(m.role),
                content: &m.body,
            })
            .collect(),
        system: params
            .system_prompt
            .as_deref()
            .or(profile.system_prompt.as_deref()),
        stream: params.stream,
    };
    let resp = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&body)
        .send()
        .await?;
    if params.stream {
        let mut lines = resp.bytes_stream();
        Ok(Box::pin(try_stream! {
            let mut buffer = String::new();
            while let Some(chunk) = lines.next().await {
                let chunk = chunk?;
                buffer.push_str(&String::from_utf8_lossy(&chunk));
                while let Some(idx) = buffer.find('\n') {
                    let mut line = buffer.drain(..=idx).collect::<String>();
                    if line.ends_with('\n') {
                        line.pop();
                    }
                    if line.ends_with('\r') {
                        line.pop();
                    }
                    if line.starts_with("data:") {
                        let token = line.trim_start_matches("data:").trim();
                        if token == "[DONE]" {
                            yield ProviderEvent { delta: None, done: true, usage: None };
                        } else if !token.is_empty() {
                            match serde_json::from_str::<Value>(token) {
                                Ok(value) => {
                                    if let Some(text) = value
                                        .pointer("/delta/text")
                                        .and_then(|v| v.as_str())
                                    {
                                        if !text.is_empty() {
                                            yield ProviderEvent { delta: Some(text.to_string()), done: false, usage: None };
                                        }
                                    }
                                }
                                Err(_) => {
                                    yield ProviderEvent { delta: Some(token.to_string()), done: false, usage: None };
                                }
                            }
                        }
                    }
                }
            }
            if !buffer.trim().is_empty() {
                if buffer.starts_with("data:") {
                    let token = buffer.trim_start_matches("data:").trim();
                    if token == "[DONE]" {
                        yield ProviderEvent { delta: None, done: true, usage: None };
                    } else if !token.is_empty() {
                        if let Ok(value) = serde_json::from_str::<Value>(token) {
                            if let Some(text) = value
                                .pointer("/delta/text")
                                .and_then(|v| v.as_str())
                            {
                                if !text.is_empty() {
                                    yield ProviderEvent { delta: Some(text.to_string()), done: false, usage: None };
                                }
                            }
                        }
                    }
                }
            }
            yield ProviderEvent { delta: None, done: true, usage: None };
        }))
    } else {
        let value: serde_json::Value = resp.json().await?;
        let text = value
            .pointer("/content/0/text")
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
