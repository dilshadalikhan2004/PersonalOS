use std::{path::Path, sync::Arc, time::{SystemTime, UNIX_EPOCH}};

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use super::{crypto, keychain::MasterKey, ollama_client::LocalOllamaClient, storage_error::StorageError};

const MAX_OCR_CHARACTERS: usize = 30_000;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuredMetadata {
    pub upload_id: String, pub title: Option<String>, pub document_type: String, pub expiry_date: Option<String>, pub document_date: Option<String>, pub addresses: Vec<String>, pub names: Vec<String>, pub phone_numbers: Vec<String>, pub emails: Vec<String>, pub companies: Vec<String>, pub amounts: Vec<Amount>, pub important_ids: Vec<ImportantId>, pub created_at_unix_ms: i64,
}
#[derive(Clone, Serialize, Deserialize)] pub struct Amount { pub value: String, pub currency: Option<String>, pub description: Option<String> }
#[derive(Clone, Serialize, Deserialize)] pub struct ImportantId { pub label: String, pub value: String }

pub struct MetadataRepository { connection: Connection, master_key: Arc<MasterKey> }
impl MetadataRepository {
    pub fn open(app_data: &Path, master_key: Arc<MasterKey>) -> Result<Self, StorageError> { let connection = Connection::open(app_data.join("lifeos.sqlite3"))?; connection.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA secure_delete = ON; CREATE TABLE IF NOT EXISTS structured_metadata (upload_id TEXT PRIMARY KEY NOT NULL, encrypted_payload BLOB NOT NULL);")?; Ok(Self { connection, master_key }) }
    pub fn save(&mut self, metadata: &StructuredMetadata) -> Result<(), StorageError> { let encrypted = crypto::encrypt(&self.master_key, &serde_json::to_vec(metadata)?)?; self.connection.execute("INSERT INTO structured_metadata (upload_id, encrypted_payload) VALUES (?1, ?2) ON CONFLICT(upload_id) DO UPDATE SET encrypted_payload = excluded.encrypted_payload", params![metadata.upload_id, encrypted])?; Ok(()) }
    pub fn get(&self, upload_id: &str) -> Result<Option<StructuredMetadata>, StorageError> { let payload: Option<Vec<u8>> = self.connection.query_row("SELECT encrypted_payload FROM structured_metadata WHERE upload_id = ?1", params![upload_id], |row| row.get(0)).optional()?; payload.map(|value| serde_json::from_slice(&crypto::decrypt(&self.master_key, &value)?).map_err(Into::into)).transpose() }
    pub fn list(&self) -> Result<Vec<StructuredMetadata>, StorageError> {
        let mut statement = self.connection.prepare("SELECT encrypted_payload FROM structured_metadata")?;
        let rows = statement.query_map([], |row| row.get::<_, Vec<u8>>(0))?;
        let mut metadata = Vec::new();
        for row in rows {
            let payload = row?;
            metadata.push(serde_json::from_slice(&crypto::decrypt(&self.master_key, &payload)?)?);
        }
        Ok(metadata)
    }
}

pub struct LocalOllamaClassifier;
impl LocalOllamaClassifier {
    pub fn classify(upload_id: String, extracted_text: &str) -> Result<StructuredMetadata, StorageError> {
        let response = LocalOllamaClient::generate_json(&format!("Extract only facts present in the document text. Unknown values must be null or empty arrays. Return valid JSON matching the provided schema. Do not infer or fabricate data.\n\nDOCUMENT TEXT:\n{}", truncate(extracted_text)), schema())?;
        let mut extracted: ModelMetadata = serde_json::from_str(&response).map_err(|_| StorageError::InvalidModelResponse)?;
        extracted.normalize();
        Ok(StructuredMetadata { upload_id, title: extracted.title, document_type: extracted.document_type.unwrap_or_else(|| "other".to_owned()), expiry_date: extracted.expiry_date, document_date: extracted.document_date, addresses: extracted.addresses, names: extracted.names, phone_numbers: extracted.phone_numbers, emails: extracted.emails, companies: extracted.companies, amounts: extracted.amounts, important_ids: extracted.important_ids, created_at_unix_ms: now_ms()? })
    }
}

#[derive(Deserialize)] struct ModelMetadata { title: Option<String>, document_type: Option<String>, expiry_date: Option<String>, document_date: Option<String>, #[serde(default)] addresses: Vec<String>, #[serde(default)] names: Vec<String>, #[serde(default)] phone_numbers: Vec<String>, #[serde(default)] emails: Vec<String>, #[serde(default)] companies: Vec<String>, #[serde(default)] amounts: Vec<Amount>, #[serde(default)] important_ids: Vec<ImportantId> }
impl ModelMetadata { fn normalize(&mut self) { self.title = clean(self.title.take()); self.expiry_date = clean(self.expiry_date.take()); self.document_date = clean(self.document_date.take()); self.document_type = clean(self.document_type.take()); self.addresses = compact(&self.addresses); self.names = compact(&self.names); self.phone_numbers = compact(&self.phone_numbers); self.emails = compact(&self.emails); self.companies = compact(&self.companies); self.amounts.truncate(32); self.important_ids.truncate(32); } }
fn clean(value: Option<String>) -> Option<String> { value.and_then(|value| { let value = value.trim().to_owned(); (!value.is_empty() && value.len() <= 512).then_some(value) }) }
fn compact(values: &[String]) -> Vec<String> { values.iter().filter_map(|value| clean(Some(value.clone()))).take(32).collect() }
fn truncate(value: &str) -> String { value.chars().take(MAX_OCR_CHARACTERS).collect() }
fn schema() -> serde_json::Value { serde_json::json!({"type":"object","properties":{"title":{"type":["string","null"]},"document_type":{"type":["string","null"]},"expiry_date":{"type":["string","null"]},"document_date":{"type":["string","null"]},"addresses":{"type":"array","items":{"type":"string"}},"names":{"type":"array","items":{"type":"string"}},"phone_numbers":{"type":"array","items":{"type":"string"}},"emails":{"type":"array","items":{"type":"string"}},"companies":{"type":"array","items":{"type":"string"}},"amounts":{"type":"array","items":{"type":"object","properties":{"value":{"type":"string"},"currency":{"type":["string","null"]},"description":{"type":["string","null"]}},"required":["value","currency","description"]}},"important_ids":{"type":"array","items":{"type":"object","properties":{"label":{"type":"string"},"value":{"type":"string"}},"required":["label","value"]}}},"required":["title","document_type","expiry_date","document_date","addresses","names","phone_numbers","emails","companies","amounts","important_ids"]}) }
fn now_ms() -> Result<i64, StorageError> { SystemTime::now().duration_since(UNIX_EPOCH).map_err(|error| StorageError::Io(std::io::Error::other(error))).map(|duration| duration.as_millis().try_into().unwrap_or(i64::MAX)) }
