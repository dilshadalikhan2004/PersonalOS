use serde::Serialize;

/// Non-sensitive process health response; it contains no user information.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthCheckResponse {
    pub status: &'static str,
}

#[tauri::command]
pub fn health_check() -> HealthCheckResponse {
    HealthCheckResponse { status: "ok" }
}

