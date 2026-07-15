use tauri::State;

use crate::application::{
    bootstrap::ApplicationBootstrap,
    timeline_service::{LifeTimelineEvent, TimelineService},
};

#[tauri::command]
pub fn get_life_timeline(state: State<'_, ApplicationBootstrap>) -> Result<Vec<LifeTimelineEvent>, String> {
    TimelineService::generate(&state.uploads, &state.metadata)
        .map_err(|_| "Life timeline could not be generated locally.".to_owned())
}
