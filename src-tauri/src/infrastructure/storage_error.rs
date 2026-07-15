use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("secure OS keychain is unavailable; encrypted storage cannot be opened")]
    KeychainUnavailable,
    #[error("the OS keychain contains an invalid LifeOS encryption key")]
    InvalidKeychainKey,
    #[error("encrypted data could not be authenticated")]
    AuthenticationFailed,
    #[error("encrypted data has an unsupported format")]
    UnsupportedEncryptedFormat,
    #[error("the selected file is not a supported document or exceeds the size limit")]
    UnsupportedUpload,
    #[error("a thumbnail could not be generated")]
    ThumbnailGenerationFailed,
    #[error("OCR is not available locally")]
    OcrUnavailable,
    #[error("local OCR processing failed")]
    OcrFailed,
    #[error("OCR does not support this file type")]
    OcrUnsupportedFile,
    #[error("OCR language configuration is invalid")]
    InvalidOcrLanguage,
    #[error("local Ollama is unavailable")]
    OllamaUnavailable,
    #[error("local Ollama returned invalid structured metadata")]
    InvalidModelResponse,
    #[error("vector index operation failed")]
    Vector(String),
    #[error("document was not found")]
    NotFound,
    #[error("storage path is invalid")]
    InvalidStoragePath,
    #[error("storage is temporarily unavailable")]
    LockUnavailable,
    #[error("database operation failed")]
    Database(#[source] rusqlite::Error),
    #[error("file operation failed")]
    Io(#[source] io::Error),
    #[error("encrypted document could not be serialized")]
    Serialization(#[source] serde_json::Error),
}

impl From<rusqlite::Error> for StorageError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Database(error)
    }
}

impl From<io::Error> for StorageError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for StorageError {
    fn from(error: serde_json::Error) -> Self {
        Self::Serialization(error)
    }
}
