use serde::Deserialize;
use tauri::State;

use crate::application::bootstrap::ApplicationBootstrap;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetOcrTextRequest { pub upload_id: String }

#[tauri::command]
pub fn get_ocr_text(state: State<'_, ApplicationBootstrap>, request: GetOcrTextRequest) -> Result<Option<crate::infrastructure::ocr_repository::OcrResult>, String> {
    state.ocr.get(&request.upload_id).map_err(|_| "The extracted text could not be accessed securely.".to_owned())
}
