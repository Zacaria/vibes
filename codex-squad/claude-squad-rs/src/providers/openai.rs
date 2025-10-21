use anyhow::Result;
use async_stream::try_stream;
use futures::StreamExt;
use reqwest::Client;
use serde::Serialize;
use serde_json::Value;

use crate::domain::{Message, MessageRole, Profile, UsageMetrics};
use crate::providers::{offline_stream, ProviderEvent, ProviderParams, ProviderStream};

#[derive(Serialize)]
struct ChatCompletionRequest<'a> {
    model: &'a str,
    messages: Vec<ChatCompletionMessage<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream_options: Option<StreamOptions>,
}

#[derive(Serialize)]
struct ChatCompletionMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Serialize)]
struct StreamOptions {
    include_usage: bool,
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
    let client = Client::builder().timeout(Duration::from_secs(60)).build()?;
    let mut request_messages: Vec<ChatCompletionMessage<'_>> = Vec::new();
    if let Some(system_prompt) = params
        .system_prompt
        .as_deref()
        .or(profile.system_prompt.as_deref())
    {
        request_messages.push(ChatCompletionMessage {
            role: "system",
            content: system_prompt,
        });
    }
    request_messages.extend(messages.iter().map(|m| ChatCompletionMessage {
        role: map_role(m.role),
        content: &m.body,
    }));
    let body = ChatCompletionRequest {
        model: &profile.model,
        messages: request_messages,
        temperature: profile.temperature,
        stream: params.stream,
        stream_options: params.stream.then_some(StreamOptions {
            include_usage: true,
        }),
    };
    let url = base_url.unwrap_or_else(|| "https://api.openai.com/v1/chat/completions".to_string());
    let resp = client
        .post(url)
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await?
        .error_for_status()?;
    if params.stream {
        let mut stream = resp.bytes_stream();
        Ok(Box::pin(try_stream! {
            let mut buffer = String::new();
            let mut done_emitted = false;
            while let Some(chunk) = stream.next().await {
                let chunk = chunk?;
                for line in String::from_utf8_lossy(&chunk).split('\n') {
                    if line.starts_with("data:") {
                        let token = line.trim_start_matches("data:").trim();
                        if token == "[DONE]" {
                            yield ProviderEvent { delta: None, done: true, usage: None };
                        } else if !token.is_empty() {
                            match serde_json::from_str::<Value>(token) {
                                Ok(value) => {
                                    if let Some(text) = value
                                        .pointer("/choices/0/delta/content")
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
                        yield evt;
                    }
                    done_emitted |= saw_done;
                }
            }
            if !buffer.trim().is_empty() {
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
            .pointer("/choices/0/message/content")
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
            if let Some(text) = value
                .pointer("/choices/0/delta/content")
                .and_then(|v| v.as_str())
            {
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
            .get("prompt_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or_default(),
        completion_tokens: usage
            .get("completion_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or_default(),
        total_tokens: usage
            .get("total_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or_default(),
    })
}
