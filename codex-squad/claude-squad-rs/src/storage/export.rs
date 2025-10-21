use anyhow::{anyhow, Result};
use uuid::Uuid;

use crate::domain::{ConversationExport, ConversationSummary};
use crate::storage::Storage;

#[derive(Clone, Copy, Debug)]
pub enum ExportFormat {
    Json,
    Md,
}

pub fn export_conversation(
    storage: Storage,
    conversation_id: &str,
    format: ExportFormat,
) -> Result<String> {
    let id = Uuid::parse_str(conversation_id).map_err(|_| anyhow!("invalid conversation id"))?;
    let title = storage
        .conversation_title(&id)?
        .ok_or_else(|| anyhow!("conversation not found"))?;
    let messages = storage.load_messages(&id)?;
    let summary = ConversationSummary {
        id,
        title,
        updated_at: messages
            .last()
            .map(|m| m.created_at)
            .unwrap_or_else(crate::util::now),
        message_count: messages.len() as u64,
    };
    let export = ConversationExport { summary, messages };
    match format {
        ExportFormat::Json => Ok(serde_json::to_string_pretty(&export)?),
        ExportFormat::Md => Ok(render_markdown(&export)),
    }
}

fn render_markdown(export: &ConversationExport) -> String {
    let mut out = String::new();
    out.push_str(&format!("# {}\n\n", escape_markdown(&export.summary.title)));
    for message in &export.messages {
        let role = message.role.to_string();
        out.push_str(&format!(
            "## {} ({})\n\n",
            escape_markdown(&message.sender),
            role
        ));
        out.push_str(&render_body(&message.body));
    }
    out
}

fn render_body(body: &str) -> String {
    if body.contains("```") {
        format!("~~~~\n{}\n~~~~\n\n", body)
    } else {
        format!("```text\n{}\n```\n\n", body)
    }
}

fn escape_markdown(input: &str) -> String {
    input
        .chars()
        .flat_map(|c| match c {
            '\\' => vec!['\\', '\\'],
            '`' | '*' | '_' | '{' | '}' | '[' | ']' | '(' | ')' | '#' | '+' | '-' | '!' | '>' => {
                vec!['\\', c]
            }
            other => vec![other],
        })
        .collect()
}
