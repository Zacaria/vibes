use anyhow::{anyhow, Result};
use clap::ValueEnum;
use uuid::Uuid;

use crate::domain::{ConversationExport, ConversationSummary};
use crate::storage::Storage;

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum ExportFormat {
    Json,
    Md,
}

pub fn export_conversation(
    storage: &Storage,
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
        message_count: messages.len(),
    };
    let export = ConversationExport { summary, messages };
    match format {
        ExportFormat::Json => Ok(serde_json::to_string_pretty(&export)?),
        ExportFormat::Md => Ok(render_markdown(&export)),
    }
}

fn render_markdown(export: &ConversationExport) -> String {
    let mut out = String::new();
    out.push_str(&format!("# {}\n\n", export.summary.title));
    for message in &export.messages {
        out.push_str(&format!(
            "## {} ({:?})\n\n{}\n\n",
            message.sender, message.role, message.body
        ));
    }
    out
}
