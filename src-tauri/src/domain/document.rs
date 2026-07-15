use std::str::FromStr;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::DomainError;

const MAX_TITLE_LENGTH: usize = 512;
const MAX_CONTENT_LENGTH: usize = 20 * 1024 * 1024;

/// Opaque identifier for a document. It is safe to expose, but carries no document content.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DocumentId(String);

impl DocumentId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    pub fn parse(value: &str) -> Result<Self, DomainError> {
        Uuid::from_str(value).map_err(|_| DomainError::InvalidDocumentId)?;
        Ok(Self(value.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug)]
pub struct NewDocument {
    pub title: String,
    pub content: String,
}

#[derive(Clone, Debug)]
pub struct UpdateDocument {
    pub title: String,
    pub content: String,
}

/// Plaintext document used only within application memory and Tauri IPC.
/// It must never be persisted without encryption.
#[derive(Clone, Debug)]
pub struct Document {
    pub id: DocumentId,
    pub title: String,
    pub content: String,
    pub created_at_unix_ms: i64,
    pub updated_at_unix_ms: i64,
}

/// The exact payload written to the encrypted file store.
#[derive(Serialize, Deserialize)]
pub struct DocumentPayload {
    pub title: String,
    pub content: String,
    pub created_at_unix_ms: i64,
    pub updated_at_unix_ms: i64,
}

impl NewDocument {
    pub fn validate(self) -> Result<Self, DomainError> {
        validate_document_fields(&self.title, &self.content)?;
        Ok(self)
    }
}

impl UpdateDocument {
    pub fn validate(self) -> Result<Self, DomainError> {
        validate_document_fields(&self.title, &self.content)?;
        Ok(self)
    }
}

fn validate_document_fields(title: &str, content: &str) -> Result<(), DomainError> {
    if title.trim().is_empty() {
        return Err(DomainError::EmptyTitle);
    }
    if title.chars().count() > MAX_TITLE_LENGTH {
        return Err(DomainError::TitleTooLong);
    }
    if content.len() > MAX_CONTENT_LENGTH {
        return Err(DomainError::ContentTooLarge);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::NewDocument;

    #[test]
    fn title_is_required() {
        let document = NewDocument { title: "  ".to_owned(), content: String::new() };
        assert!(document.validate().is_err());
    }
}
