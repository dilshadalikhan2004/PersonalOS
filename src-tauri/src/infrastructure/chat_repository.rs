use std::{path::Path, sync::Arc, time::{SystemTime, UNIX_EPOCH}};

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{crypto, keychain::MasterKey, storage_error::StorageError};

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub id: String,
    pub role: ChatRole,
    pub content: String,
    pub citations: Vec<ChatCitation>,
    pub created_at_unix_ms: i64,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ChatRole {
    User,
    Assistant,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatCitation {
    pub upload_id: String,
    pub chunk_index: i32,
    pub excerpt: String,
}

pub struct ChatRepository {
    connection: Connection,
    master_key: Arc<MasterKey>,
}

impl ChatRepository {
    pub fn open(app_data: &Path, master_key: Arc<MasterKey>) -> Result<Self, StorageError> {
        let connection = Connection::open(app_data.join("lifeos.sqlite3"))?;
        connection.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA secure_delete = ON; CREATE TABLE IF NOT EXISTS chat_messages (id TEXT PRIMARY KEY NOT NULL, created_at_unix_ms INTEGER NOT NULL, encrypted_payload BLOB NOT NULL);")?;
        Ok(Self { connection, master_key })
    }

    pub fn append(&mut self, role: ChatRole, content: String, citations: Vec<ChatCitation>) -> Result<ChatMessage, StorageError> {
        let message = ChatMessage { id: Uuid::new_v4().to_string(), role, content, citations, created_at_unix_ms: now_ms()? };
        let encrypted_payload = crypto::encrypt(&self.master_key, &serde_json::to_vec(&message)?)?;
        self.connection.execute("INSERT INTO chat_messages (id, created_at_unix_ms, encrypted_payload) VALUES (?1, ?2, ?3)", params![message.id, message.created_at_unix_ms, encrypted_payload])?;
        Ok(message)
    }

    pub fn recent(&self, limit: usize) -> Result<Vec<ChatMessage>, StorageError> {
        let limit = limit.min(200);
        let mut statement = self.connection.prepare("SELECT encrypted_payload FROM chat_messages ORDER BY created_at_unix_ms DESC LIMIT ?1")?;
        let rows = statement.query_map(params![limit as i64], |row| row.get::<_, Vec<u8>>(0))?;
        let mut messages = Vec::new();
        for row in rows {
            let payload = row?;
            messages.push(serde_json::from_slice(&crypto::decrypt(&self.master_key, &payload)?)?);
        }
        messages.reverse();
        Ok(messages)
    }
}

fn now_ms() -> Result<i64, StorageError> {
    SystemTime::now().duration_since(UNIX_EPOCH).map_err(|error| StorageError::Io(std::io::Error::other(error))).map(|duration| duration.as_millis().try_into().unwrap_or(i64::MAX))
}
