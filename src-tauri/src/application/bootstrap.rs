use std::sync::Arc;

use tauri::{AppHandle, Manager};

use crate::infrastructure::{
    chat_repository::ChatRepository,
    keychain::load_or_create_master_key,
    metadata_repository::MetadataRepository,
    ocr_repository::{LocalOcrEngine, OcrRepository},
    sqlite_document_repository::SqliteDocumentRepository,
    storage_error::StorageError,
    upload_repository::UploadRepository,
    vector_repository::LanceVectorRepository,
};

use super::{chat_service::ChatService, document_service::DocumentService, metadata_service::MetadataService, ocr_service::OcrService, upload_service::UploadService, vector_service::VectorService};

/// Application composition root. The encryption key stays inside the repository and is never managed by Tauri state.
pub struct ApplicationBootstrap {
    pub documents: DocumentService,
    pub uploads: UploadService,
    pub ocr: OcrService,
    pub metadata: MetadataService,
    pub vectors: VectorService,
    pub chat: ChatService,
}

impl ApplicationBootstrap {
    pub fn initialize(app: &AppHandle) -> Result<Self, StorageError> {
        let application_data_directory = app.path().app_data_dir().map_err(|_| StorageError::InvalidStoragePath)?;
        let master_key = Arc::new(load_or_create_master_key()?);
        let documents = SqliteDocumentRepository::open(&application_data_directory, Arc::clone(&master_key))?;
        let uploads = UploadRepository::open(&application_data_directory, Arc::clone(&master_key))?;
        let ocr_repository = OcrRepository::open(&application_data_directory, Arc::clone(&master_key))?;
        let chat_repository = ChatRepository::open(&application_data_directory, Arc::clone(&master_key))?;
        let metadata_repository = MetadataRepository::open(&application_data_directory, master_key)?;
        let vector_repository = LanceVectorRepository::open(&application_data_directory)?;
        let ocr = OcrService::new(ocr_repository, LocalOcrEngine::new(&application_data_directory));
        Ok(Self {
            documents: DocumentService::new(documents),
            uploads: UploadService::new(uploads),
            ocr,
            metadata: MetadataService::new(metadata_repository),
            vectors: VectorService::new(vector_repository),
            chat: ChatService::new(chat_repository),
        })
    }
}
