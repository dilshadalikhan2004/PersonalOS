use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

use crate::{application::bootstrap::ApplicationBootstrap, infrastructure::{storage_error::StorageError, upload_repository::UploadProgress}};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadDocumentRequest { pub source_path: String, pub upload_id: String, pub ocr_languages: Option<String> }

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadProgressEvent { pub upload_id: String, pub percent: u8, pub stage: &'static str }

#[tauri::command]
pub fn upload_document(app: AppHandle, state: State<'_, ApplicationBootstrap>, request: UploadDocumentRequest) -> Result<crate::infrastructure::upload_repository::UploadedFile, String> {
    let upload_id = request.upload_id;
    let source_path = secure_upload_path(&request.source_path)?;
    let uploaded = state.uploads.ingest(&source_path, |progress: UploadProgress| {
        let _ = app.emit("document-upload-progress", UploadProgressEvent { upload_id: upload_id.clone(), percent: progress.percent, stage: progress.stage });
    }).map_err(public_error)?;
    let languages = request.ocr_languages.as_deref().unwrap_or("eng");
    // OCR is best-effort: upload durability never depends on a locally missing OCR binary.
    let ocr_result = state.ocr.extract_and_store(uploaded.id.clone(), &source_path, languages, |percent, stage| {
        let _ = app.emit("document-upload-progress", UploadProgressEvent { upload_id: upload_id.clone(), percent, stage });
    });
    let final_stage = if let Ok(ocr) = ocr_result {
        let _ = app.emit("document-upload-progress", UploadProgressEvent { upload_id: upload_id.clone(), percent: 92, stage: "Classifying with local Ollama" });
        let classified = state.metadata.classify_and_store(uploaded.id.clone(), &ocr.text).is_ok();
        let _ = app.emit("document-upload-progress", UploadProgressEvent { upload_id: upload_id.clone(), percent: 96, stage: "Creating local vector index" });
        let indexed = state.vectors.index_document(&uploaded.id, &ocr.text).is_ok();
        if indexed && classified { "Ready with local AI metadata and search" } else if indexed { "Ready with semantic search" } else if classified { "Ready with local AI metadata" } else { "Ready with local OCR" }
    } else { "Stored securely" };
    let _ = app.emit("document-upload-progress", UploadProgressEvent { upload_id, percent: 100, stage: final_stage });
    Ok(uploaded)
}

fn public_error(error: StorageError) -> String {
    match error { StorageError::UnsupportedUpload => "Choose a PDF, PNG, JPEG, or DOCX file up to 50 MB.".to_owned(), _ => "The file could not be stored securely.".to_owned() }
}

fn secure_upload_path(source_path: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(source_path);
    if !path.is_absolute() {
        return Err("Choose a file from your device.".to_owned());
    }
    let canonical = path.canonicalize().map_err(|_| "Choose a file from your device.".to_owned())?;
    if !canonical.is_file() {
        return Err("Choose a file from your device.".to_owned());
    }
    Ok(canonical)
}
