use crate::domain::{Message, MessageRole, Profile, UsageMetrics};
use crate::providers::{offline_stream, ProviderEvent, ProviderParams, ProviderStream};
use anyhow::Result;
use async_stream::try_stream;
use futures::StreamExt;
use reqwest::Client;
use serde::Serialize;
use serde_json::Value;
use std::time::Duration;

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
    let client = Client::builder().timeout(Duration::from_secs(60)).build()?;
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
        .await?
        .error_for_status()?;
    if params.stream {
        let mut lines = resp.bytes_stream();
        Ok(Box::pin(try_stream! {
            let mut buffer = String::new();
            let mut done_emitted = false;
            while let Some(chunk) = lines.next().await {
                let chunk = chunk?;
                buffer.push_str(&String::from_utf8_lossy(&chunk));
                while let Some(idx) = buffer.find("\n\n") {
                    let event: String = buffer.drain(..idx + 2).collect();
                    let (events, saw_done) = parse_sse_event(&event);
                    for evt in events {
                        if evt.done {
                            done_emitted = true;
                        }
                        yield evt;
                    }
                    done_emitted |= saw_done;
                }
            }
            if !buffer.is_empty() {
                let (events, saw_done) = parse_sse_event(&buffer);
                for evt in events {
                    if evt.done {
                        done_emitted = true;
                    }
                    yield evt;
                }
                done_emitted |= saw_done;
            }
            if !done_emitted {
                yield ProviderEvent { delta: None, done: true, usage: None };
            }
        }))
    } else {
        let value: Value = resp.json().await?;
        let text = value
            .pointer("/content/0/text")
            .and_then(|v| v.as_str())
            .unwrap_or("(no response)")
            .to_string();
        let usage = parse_usage(&value);
        Ok(Box::pin(try_stream! {
            yield ProviderEvent { delta: Some(text), done: false, usage: None };
            yield ProviderEvent { delta: None, done: true, usage };
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

fn parse_sse_event(event: &str) -> (Vec<ProviderEvent>, bool) {
    let mut events = Vec::new();
    let mut saw_done = false;
    for line in event.lines() {
        if !line.starts_with("data:") {
            continue;
        }
        let data = line.trim_start_matches("data:").trim();
        if data.is_empty() {
            continue;
        }
        if data == "[DONE]" {
            events.push(ProviderEvent {
                delta: None,
                done: true,
                usage: None,
            });
            saw_done = true;
            continue;
        }
        if let Ok(value) = serde_json::from_str::<Value>(data) {
            if let Some(text) = value.pointer("/delta/text").and_then(|v| v.as_str()) {
                if !text.is_empty() {
                    events.push(ProviderEvent {
                        delta: Some(text.to_string()),
                        done: false,
                        usage: None,
                    });
                }
            }
            if let Some(usage) = parse_usage(&value) {
                events.push(ProviderEvent {
                    delta: None,
                    done: false,
                    usage: Some(usage),
                });
            }
        }
    }
    (events, saw_done)
}

fn parse_usage(value: &Value) -> Option<UsageMetrics> {
    let usage = value.get("usage")?;
    Some(UsageMetrics {
        prompt_tokens: usage
            .get("input_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or_default(),
        completion_tokens: usage
            .get("output_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or_default(),
        total_tokens: usage
            .get("total_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or_default(),
    })
}
