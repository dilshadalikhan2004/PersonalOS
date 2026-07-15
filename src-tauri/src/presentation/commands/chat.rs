use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

use crate::{application::bootstrap::ApplicationBootstrap, infrastructure::chat_repository::ChatMessage};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AskChatRequest {
    pub request_id: String,
    pub question: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatTokenEvent {
    pub request_id: String,
    pub token: String,
}

#[tauri::command]
pub fn get_chat_history(state: State<'_, ApplicationBootstrap>) -> Result<Vec<ChatMessage>, String> {
    state.chat.history().map_err(|_| "Chat history could not be opened.".to_owned())
}

#[tauri::command]
pub fn ask_chat(app: AppHandle, state: State<'_, ApplicationBootstrap>, request: AskChatRequest) -> Result<ChatMessage, String> {
    let request_id = request.request_id;
    state
        .chat
        .answer(request.question, &state.ocr, &state.vectors, |token| {
            let _ = app.emit("chat-token", ChatTokenEvent { request_id: request_id.clone(), token: token.to_owned() });
        })
        .map_err(|_| "I don't know.".to_owned())
}
