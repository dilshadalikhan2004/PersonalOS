use serde::Serialize;
use tauri::State;

use crate::application::bootstrap::ApplicationBootstrap;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryDocument {
    pub id: String,
    pub file_name: String,
    pub media_type: String,
    pub size_bytes: u64,
    pub created_at_unix_ms: i64,
    pub title: Option<String>,
    pub document_type: Option<String>,
    pub expiry_date: Option<String>,
    pub document_date: Option<String>,
}

#[tauri::command]
pub fn list_library_documents(state: State<'_, ApplicationBootstrap>) -> Result<Vec<LibraryDocument>, String> {
    let uploads = state
        .uploads
        .recent(250)
        .map_err(|_| "Documents could not be loaded securely.".to_owned())?;
    let metadata = state
        .metadata
        .list()
        .map_err(|_| "Documents could not be loaded securely.".to_owned())?;
    Ok(uploads
        .into_iter()
        .map(|upload| {
            let meta = metadata.iter().find(|item| item.upload_id == upload.id);
            LibraryDocument {
                id: upload.id,
                file_name: upload.file_name,
                media_type: upload.media_type,
                size_bytes: upload.size_bytes,
                created_at_unix_ms: upload.created_at_unix_ms,
                title: meta.and_then(|item| item.title.clone()),
                document_type: meta.map(|item| item.document_type.clone()),
                expiry_date: meta.and_then(|item| item.expiry_date.clone()),
                document_date: meta.and_then(|item| item.document_date.clone()),
            }
        })
        .collect())
}
