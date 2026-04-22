pub mod models;
pub mod commands;
pub mod storage;
pub mod scanner;
pub mod plugin_manager;
pub mod linker;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let app_handle = app.handle();
            
            // 初始化数据目录
            let data_dir = app_handle.path().app_data_dir()
                .expect("Failed to get app data directory");
            std::fs::create_dir_all(&data_dir)
                .expect("Failed to create app data directory");
            
            // 初始化插件存储目录
            let plugins_dir = data_dir.join("plugins");
            std::fs::create_dir_all(&plugins_dir)
                .expect("Failed to create plugins directory");
            
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
