use serde::Deserialize;
use tauri::State;

use crate::application::bootstrap::ApplicationBootstrap;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetMetadataRequest { pub upload_id: String }

#[tauri::command]
pub fn get_structured_metadata(state: State<'_, ApplicationBootstrap>, request: GetMetadataRequest) -> Result<Option<crate::infrastructure::metadata_repository::StructuredMetadata>, String> { state.metadata.get(&request.upload_id).map_err(|_| "Structured metadata could not be accessed securely.".to_owned()) }
