pub mod models;
pub mod commands;
pub mod storage;
pub mod scanner;
pub mod plugin_manager;
pub mod linker;
pub mod operation_log;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let app_handle = app.handle();
            let data_dir = app_handle.path().app_data_dir()
                .expect("Failed to get app data directory");
            std::fs::create_dir_all(&data_dir)
                .expect("Failed to create app data directory");
            let plugins_dir = data_dir.join("plugins");
            std::fs::create_dir_all(&plugins_dir)
                .expect("Failed to create plugins directory");
            let logs_dir = data_dir.join("logs");
            std::fs::create_dir_all(&logs_dir)
                .expect("Failed to create logs directory");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::save_settings,
            commands::scan_projects,
            commands::get_projects,
            commands::add_project,
            commands::remove_project,
            commands::import_plugin_from_local,
            commands::import_plugin_from_git,
            commands::get_plugins,
            commands::remove_plugin,
            commands::bind_plugin,
            commands::unbind_plugin,
            commands::apply_changes,
            commands::get_project_bindings,
            commands::scan_project_plugins,
            commands::import_plugins_from_projects,
            commands::get_operation_logs,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
