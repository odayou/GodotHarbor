pub mod models;
pub mod commands;
pub mod storage;
pub mod scanner;
pub mod plugin_manager;
pub mod linker;
pub mod operation_log;
pub mod engine;
pub mod godot_resolver;

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

            let handle = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                let _ = commands::auto_scan_projects(handle).await;
            });

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
            commands::log_client_error,
            commands::toggle_plugin_favorite,
            commands::update_project_group,
            commands::get_project_groups,
            commands::backup_data,
            commands::restore_data,
            commands::register_engine,
            commands::get_engines,
            commands::remove_engine,
            commands::set_default_engine,
            commands::bind_project_engine,
            commands::unbind_project_engine,
            commands::get_project_engine_binding,
            commands::launch_project_with_engine,
            commands::check_plugin_updates,
            commands::export_team_config,
            commands::get_team_configs,
            commands::import_team_config,
            commands::delete_team_config,
            commands::resolve_plugin_dependencies,
            commands::search_asset_library,
            commands::import_from_asset_library,
            commands::get_dashboard_stats,
            commands::auto_scan_projects,
            commands::relocate_project,
            commands::detect_moved_projects,
            commands::confirm_project_relocation,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
