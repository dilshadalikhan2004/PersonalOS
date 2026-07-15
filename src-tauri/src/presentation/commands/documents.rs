use serde::{Deserialize, Serialize};
use tauri::State;

use crate::{application::bootstrap::ApplicationBootstrap, domain::{Document, DocumentId, DomainError, NewDocument, UpdateDocument}, infrastructure::storage_error::StorageError};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDocumentRequest {
    pub title: String,
    pub content: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDocumentRequest {
    pub title: String,
    pub content: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentResponse {
    pub id: String,
    pub title: String,
    pub content: String,
    pub created_at_unix_ms: i64,
    pub updated_at_unix_ms: i64,
}

#[tauri::command]
pub fn create_document(state: State<'_, ApplicationBootstrap>, request: CreateDocumentRequest) -> Result<DocumentResponse, String> {
    let new_document = NewDocument { title: request.title, content: request.content }.validate().map_err(validation_error)?;
    state.documents.create(new_document).map(DocumentResponse::from).map_err(storage_error)
}

#[tauri::command]
pub fn get_document(state: State<'_, ApplicationBootstrap>, id: String) -> Result<DocumentResponse, String> {
    let id = DocumentId::parse(&id).map_err(validation_error)?;
    state.documents.get(id).map(DocumentResponse::from).map_err(storage_error)
}

#[tauri::command]
pub fn list_documents(state: State<'_, ApplicationBootstrap>) -> Result<Vec<DocumentResponse>, String> {
    state.documents.list().map(|documents| documents.into_iter().map(DocumentResponse::from).collect()).map_err(storage_error)
}

#[tauri::command]
pub fn update_document(state: State<'_, ApplicationBootstrap>, id: String, request: UpdateDocumentRequest) -> Result<DocumentResponse, String> {
    let id = DocumentId::parse(&id).map_err(validation_error)?;
    let update = UpdateDocument { title: request.title, content: request.content }.validate().map_err(validation_error)?;
    state.documents.update(id, update).map(DocumentResponse::from).map_err(storage_error)
}

#[tauri::command]
pub fn delete_document(state: State<'_, ApplicationBootstrap>, id: String) -> Result<(), String> {
    let id = DocumentId::parse(&id).map_err(validation_error)?;
    state.documents.delete(id).map_err(storage_error)
}

impl From<Document> for DocumentResponse {
    fn from(document: Document) -> Self {
        Self { id: document.id.as_str().to_owned(), title: document.title, content: document.content, created_at_unix_ms: document.created_at_unix_ms, updated_at_unix_ms: document.updated_at_unix_ms }
    }
}

fn validation_error(error: DomainError) -> String {
    error.to_string()
}

// Do not disclose paths, driver errors, cryptographic details, or keychain state to the UI.
fn storage_error(error: StorageError) -> String {
    match error {
        StorageError::NotFound => "Document was not found.".to_owned(),
        _ => "The document could not be accessed securely.".to_owned(),
    }
}
