mod application;
mod domain;
mod infrastructure;
mod presentation;

use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let bootstrap = application::bootstrap::ApplicationBootstrap::initialize(&app.handle())?;
            app.manage(bootstrap);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            presentation::commands::health::health_check,
            presentation::commands::documents::create_document,
            presentation::commands::documents::get_document,
            presentation::commands::documents::list_documents,
            presentation::commands::documents::update_document,
            presentation::commands::documents::delete_document,
            presentation::commands::uploads::upload_document,
            presentation::commands::ocr::get_ocr_text,
            presentation::commands::metadata::get_structured_metadata,
            presentation::commands::chat::get_chat_history,
            presentation::commands::chat::ask_chat,
            presentation::commands::dashboard::get_dashboard_summary,
            presentation::commands::library::list_library_documents,
            presentation::commands::timeline::get_life_timeline,
            presentation::commands::reminders::get_local_reminders
        ])
        .run(tauri::generate_context!())
        .expect("failed to run LifeOS desktop application");
}
