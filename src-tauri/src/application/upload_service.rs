use std::{path::Path, sync::Mutex};

use crate::infrastructure::{storage_error::StorageError, upload_repository::{UploadProgress, UploadRepository, UploadedFile}};

pub struct UploadService { repository: Mutex<UploadRepository> }

impl UploadService {
    pub fn new(repository: UploadRepository) -> Self { Self { repository: Mutex::new(repository) } }
    pub fn ingest(&self, source_path: &Path, progress: impl FnMut(UploadProgress)) -> Result<UploadedFile, StorageError> {
        self.repository.lock().map_err(|_| StorageError::LockUnavailable)?.ingest(source_path, progress)
    }

    pub fn recent(&self, limit: usize) -> Result<Vec<UploadedFile>, StorageError> {
        self.repository.lock().map_err(|_| StorageError::LockUnavailable)?.list_recent(limit)
    }
}
