use std::sync::Mutex;

use crate::infrastructure::{metadata_repository::{LocalOllamaClassifier, MetadataRepository, StructuredMetadata}, storage_error::StorageError};

pub struct MetadataService { repository: Mutex<MetadataRepository> }
impl MetadataService {
    pub fn new(repository: MetadataRepository) -> Self { Self { repository: Mutex::new(repository) } }
    pub fn classify_and_store(&self, upload_id: String, text: &str) -> Result<StructuredMetadata, StorageError> { let metadata = LocalOllamaClassifier::classify(upload_id, text)?; self.repository.lock().map_err(|_| StorageError::LockUnavailable)?.save(&metadata)?; Ok(metadata) }
    pub fn get(&self, upload_id: &str) -> Result<Option<StructuredMetadata>, StorageError> { self.repository.lock().map_err(|_| StorageError::LockUnavailable)?.get(upload_id) }
    pub fn list(&self) -> Result<Vec<StructuredMetadata>, StorageError> { self.repository.lock().map_err(|_| StorageError::LockUnavailable)?.list() }
}
