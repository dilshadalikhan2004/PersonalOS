use std::sync::Mutex;

use crate::{domain::{Document, DocumentId, NewDocument, UpdateDocument}, infrastructure::{sqlite_document_repository::SqliteDocumentRepository, storage_error::StorageError}};

/// Application layer façade. Commands call this service rather than infrastructure directly.
pub struct DocumentService {
    repository: Mutex<SqliteDocumentRepository>,
}

impl DocumentService {
    pub fn new(repository: SqliteDocumentRepository) -> Self { Self { repository: Mutex::new(repository) } }

    pub fn create(&self, new_document: NewDocument) -> Result<Document, StorageError> {
        self.repository.lock().map_err(|_| StorageError::LockUnavailable)?.create(new_document)
    }
    pub fn get(&self, id: DocumentId) -> Result<Document, StorageError> {
        self.repository.lock().map_err(|_| StorageError::LockUnavailable)?.get(&id)
    }
    pub fn list(&self) -> Result<Vec<Document>, StorageError> {
        self.repository.lock().map_err(|_| StorageError::LockUnavailable)?.list()
    }
    pub fn update(&self, id: DocumentId, update: UpdateDocument) -> Result<Document, StorageError> {
        self.repository.lock().map_err(|_| StorageError::LockUnavailable)?.update(&id, update)
    }
    pub fn delete(&self, id: DocumentId) -> Result<(), StorageError> {
        self.repository.lock().map_err(|_| StorageError::LockUnavailable)?.delete(&id)
    }
}
