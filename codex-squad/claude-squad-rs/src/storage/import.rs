use anyhow::Result;
use uuid::Uuid;

use crate::domain::{ConversationImport, Message};
use crate::storage::Storage;

pub fn import_conversation(storage: Storage, contents: &str) -> Result<Uuid> {
    let import: ConversationImport = if contents.trim_start().starts_with('{') {
        serde_json::from_str(contents)?
    } else {
        serde_yaml::from_str(contents)?
    };
    let conversation_id = storage.upsert_conversation(&import.title)?;
    for message in import.messages {
        let msg = Message {
            conversation_id,
            ..message
        };
        storage.append_message(&msg)?;
    }
    Ok(conversation_id)
}
