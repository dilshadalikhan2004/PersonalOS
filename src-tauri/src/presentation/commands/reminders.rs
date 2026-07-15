use tauri::State;

use crate::application::{
    bootstrap::ApplicationBootstrap,
    reminder_service::{LocalReminder, ReminderService},
};

#[tauri::command]
pub fn get_local_reminders(state: State<'_, ApplicationBootstrap>) -> Result<Vec<LocalReminder>, String> {
    ReminderService::detect(&state.metadata)
        .map_err(|_| "Local reminders could not be generated.".to_owned())
}
