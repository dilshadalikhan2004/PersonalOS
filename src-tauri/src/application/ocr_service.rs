use std::{path::Path, sync::Mutex};

use crate::infrastructure::{ocr_repository::{LocalOcrEngine, OcrRepository, OcrResult}, storage_error::StorageError};

pub struct OcrService { repository: Mutex<OcrRepository>, engine: LocalOcrEngine }

impl OcrService {
    pub fn new(repository: OcrRepository, engine: LocalOcrEngine) -> Self { Self { repository: Mutex::new(repository), engine } }
    pub fn extract_and_store(&self, upload_id: String, source: &Path, languages: &str, progress: impl Fn(u8, &'static str)) -> Result<OcrResult, StorageError> {
        let (text, source_kind) = self.engine.extract(source, languages, progress)?;
        let result = OcrResult { upload_id, text, languages: languages.to_owned(), source: source_kind, created_at_unix_ms: now_ms()? };
        self.repository.lock().map_err(|_| StorageError::LockUnavailable)?.save(&result)?; Ok(result)
    }
    pub fn get(&self, upload_id: &str) -> Result<Option<OcrResult>, StorageError> { self.repository.lock().map_err(|_| StorageError::LockUnavailable)?.get(upload_id) }
    pub fn list(&self, limit: usize) -> Result<Vec<OcrResult>, StorageError> { self.repository.lock().map_err(|_| StorageError::LockUnavailable)?.list(limit) }
}

fn now_ms() -> Result<i64, StorageError> { std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_err(|error| StorageError::Io(std::io::Error::other(error))).map(|duration| duration.as_millis().try_into().unwrap_or(i64::MAX)) }
