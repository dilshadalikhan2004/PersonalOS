use tauri::State;

use crate::{application::{bootstrap::ApplicationBootstrap, dashboard_service::{DashboardService, DashboardSummary}}};

#[tauri::command]
pub fn get_dashboard_summary(state: State<'_, ApplicationBootstrap>) -> Result<DashboardSummary, String> {
    DashboardService::summary(&state.uploads, &state.metadata).map_err(|_| "Dashboard data could not be loaded.".to_owned())
}
