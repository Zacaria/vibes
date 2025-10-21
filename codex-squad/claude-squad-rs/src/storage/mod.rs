pub mod export;
pub mod import;

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use parking_lot::Mutex;
use rusqlite::{params, Connection, OptionalExtension};
use std::convert::TryInto;
use std::str::FromStr;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::domain::{ConversationSummary, Message, MessageRole};
use crate::util;

#[derive(Clone)]
pub struct Storage {
    inner: Arc<StorageInner>,
}

struct StorageInner {
    conn: Mutex<Connection>,
}

impl Storage {
    pub fn try_new(path: PathBuf) -> Result<Self> {
        util::ensure_parent(&path)?;
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL").ok();
        conn.pragma_update(None, "foreign_keys", "ON").ok();
        conn.execute_batch(include_str!("../../migrations/sqlite/0001_init.sql"))?;
        Ok(Self {
            inner: Arc::new(StorageInner {
                conn: Mutex::new(conn),
            }),
        })
    }

    pub fn health_check(&self) -> String {
        match self
            .conn()
            .query_row("select count(*) from conversations", [], |row| {
                row.get::<_, i64>(0)
            }) {
            Ok(count) => format!("{} conversations", count),
            Err(err) => format!("error: {}", err),
        }
    }

    fn conn(&self) -> parking_lot::MutexGuard<'_, Connection> {
        self.inner.conn.lock()
    }

    pub fn upsert_conversation(&self, title: &str) -> Result<Uuid> {
        let id = Uuid::new_v4();
        let now = util::format_time(util::now());
        self.conn().execute(
            "insert into conversations(id, title, created_at, updated_at) values(?1, ?2, ?3, ?3)",
            params![id.to_string(), title, now],
        )?;
        Ok(id)
    }

    pub fn append_message(&self, msg: &Message) -> Result<()> {
        let conn = self.conn();
        conn.execute(
            "insert into messages(id, conversation_id, role, sender, body, created_at) values(?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                msg.id.to_string(),
                msg.conversation_id.to_string(),
                format_role(msg.role),
                msg.sender,
                msg.body,
                util::format_time(msg.created_at),
            ],
        )?;
        conn.execute(
            "update conversations set updated_at = ?2 where id = ?1",
            params![
                msg.conversation_id.to_string(),
                util::format_time(msg.created_at)
            ],
        )?;
        Ok(())
    }

    pub fn load_messages(&self, conversation_id: &Uuid) -> Result<Vec<Message>> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare("select id, sender, role, body, created_at from messages where conversation_id = ?1 order by created_at asc")?;
        let rows = stmt
            .query_map(params![conversation_id.to_string()], |row| {
                let created_at: String = row.get(4)?;
                Ok(Message {
                    id: Uuid::parse_str(row.get::<_, String>(0)?.as_str())
                        .unwrap_or_else(|_| Uuid::nil()),
                    conversation_id: *conversation_id,
                    sender: row.get(1)?,
                    role: parse_role(row.get::<_, String>(2)?.as_str()),
                    body: row.get(3)?,
                    created_at: OffsetDateTime::parse(
                        &created_at,
                        &time::format_description::well_known::Rfc3339,
                    )
                    .unwrap_or_else(|_| util::now()),
                    attachments: Vec::new(),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn list_conversations(&self, filter: Option<&str>) -> Result<Vec<ConversationSummary>> {
        let conn = self.conn();
        let mut query = "select id, title, updated_at, (select count(*) from messages m where m.conversation_id = c.id) as message_count from conversations c".to_string();
        if filter.is_some() {
            query.push_str(" where title like ?1");
        }
        query.push_str(" order by updated_at desc limit 100");
        let mut stmt = conn.prepare(&query)?;
        let mut rows = Vec::new();
        let filter_param = filter.map(|f| format!("%{}%", f));
        let mut rows_iter = if let Some(param) = filter_param.as_ref() {
            stmt.query(params![param])?
        } else {
            stmt.query([])?
        };
        while let Some(row) = rows_iter.next()? {
            let updated_at: String = row.get(2)?;
            rows.push(ConversationSummary {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_else(|_| Uuid::nil()),
                title: row.get(1)?,
                updated_at: OffsetDateTime::parse(
                    &updated_at,
                    &time::format_description::well_known::Rfc3339,
                )
                .unwrap_or_else(|_| util::now()),
                message_count: row.get::<_, i64>(3)?.try_into().unwrap_or_default(),
            });
        }
        Ok(rows)
    }

    pub fn conversation_title(&self, id: &Uuid) -> Result<Option<String>> {
        self.conn()
            .query_row(
                "select title from conversations where id = ?1",
                params![id.to_string()],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(Into::into)
    }
}

fn format_role(role: MessageRole) -> String {
    role.to_string()
}

fn parse_role(role: &str) -> MessageRole {
    MessageRole::from_str(role).unwrap_or(MessageRole::User)
}
