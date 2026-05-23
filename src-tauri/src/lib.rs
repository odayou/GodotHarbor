pub mod models;
pub mod commands;
pub mod storage;
pub mod scanner;
pub mod plugin_manager;
pub mod linker;
pub mod operation_log;
pub mod engine;
pub mod engine_downloader;
pub mod godot_resolver;
pub mod version_checker;
pub mod watcher;
pub mod update_scheduler;
pub mod hot_update;
pub mod utils;
pub mod featured;
pub mod harbor_config;

use tauri::{Emitter, Manager};
use tauri_plugin_notification::NotificationExt;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

pub struct AppState {
    pub fs_watcher: Mutex<watcher::FsWatcher>,
}

static WINDOW_CLOSED: AtomicBool = AtomicBool::new(false);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_notification::init())
        .register_uri_scheme_protocol("hotupdate", move |ctx, request| {
            let uri = request.uri().to_string();
            let path = uri.trim_start_matches("hotupdate://localhost/");
            let path = percent_encoding::percent_decode_str(path).decode_utf8_lossy().to_string();

            let app_handle = ctx.app_handle();
            let data_dir = app_handle.path().app_data_dir()
                .expect("Failed to get app data directory");
            let overlay_path = data_dir.join("hotupdate_overlay").join(&path);

            if overlay_path.exists() && overlay_path.is_file() {
                let data = std::fs::read(&overlay_path).unwrap_or_default();
                let mime = if path.ends_with(".html") { "text/html" }
                    else if path.ends_with(".js") { "application/javascript" }
                    else if path.ends_with(".css") { "text/css" }
                    else if path.ends_with(".json") { "application/json" }
                    else if path.ends_with(".png") { "image/png" }
                    else if path.ends_with(".svg") { "image/svg+xml" }
                    else if path.ends_with(".ico") { "image/x-icon" }
                    else if path.ends_with(".woff") { "font/woff" }
                    else if path.ends_with(".woff2") { "font/woff2" }
                    else { "application/octet-stream" };
                tauri::http::Response::builder()
                    .status(200)
                    .header("Content-Type", mime)
                    .body(data)
                    .unwrap()
            } else {
                tauri::http::Response::builder()
                    .status(404)
                    .body(Vec::new())
                    .unwrap()
            }
        })
        .manage(AppState {
            fs_watcher: Mutex::new(watcher::FsWatcher::new(5)),
        })
        .setup(|app| {
            #[cfg(desktop)]
            {
                app.handle().plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                        let _ = window.set_always_on_top(true);
                        let win = window.clone();
                        tauri::async_runtime::spawn(async move {
                            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                            let _ = win.set_always_on_top(false);
                        });
                    }
                }))?;
            }

            let app_handle = app.handle();
            let config_dir = app_handle.path().app_data_dir()
                .expect("Failed to get app data directory");
            std::fs::create_dir_all(&config_dir)
                .expect("Failed to create app data directory");

            let config_storage = crate::storage::Storage::new(config_dir.clone());
            let settings: crate::models::Settings = config_storage.load_or_default("settings.json");
            let data_dir = if settings.custom_data_dir.is_empty() {
                config_dir
            } else {
                std::path::PathBuf::from(&settings.custom_data_dir)
            };
            std::fs::create_dir_all(&data_dir)
                .expect("Failed to create data directory");

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

            let discover_handle = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
                let _ = commands::auto_discover_engines(discover_handle).await;
            });

            let scheduler_handle = app_handle.clone();
            update_scheduler::start_update_scheduler(scheduler_handle);

            let ping_handle = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                let _ = featured::report_usage_ping(ping_handle).await;
            });

            let watcher_handle = app_handle.clone();
            let watcher_app = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                let storage = {
                    let data_dir = watcher_app.path().app_data_dir()
                        .expect("Failed to get app data directory");
                    storage::Storage::new(data_dir)
                };
                let settings: models::Settings = storage.load_or_default("settings.json");
                let dirs = if settings.scan_directories.is_empty() {
                    commands::get_default_scan_dirs()
                } else {
                    settings.scan_directories
                };

                let state = watcher_handle.state::<AppState>();
                let result = {
                    let guard = state.fs_watcher.lock();
                    if let Ok(guard) = guard {
                        guard.start(watcher_handle.clone(), dirs)
                    } else {
                        Err("获取监听状态锁失败".to_string())
                    }
                };
                drop(result);
            });

            let show_handle = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                if !WINDOW_CLOSED.load(Ordering::SeqCst) {
                    if let Some(window) = show_handle.get_webview_window("main") {
                        let _ = window.maximize();
                        let _ = window.show();
                        let _ = window.set_always_on_top(true);
                        let win = window.clone();
                        tauri::async_runtime::spawn(async move {
                            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                            let _ = win.set_always_on_top(false);
                        });
                    }
                }
            });

            // 处理窗口关闭事件，改为隐藏而不是退出
            let app_clone = app_handle.clone();
            if let Some(window) = app.get_webview_window("main") {
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        WINDOW_CLOSED.store(true, Ordering::SeqCst);
                        if let Some(window) = app_clone.get_webview_window("main") {
                            let _ = window.hide();
                        }
                    }
                });
            }

            // 创建系统托盘图标和菜单
            use tauri::menu::{MenuBuilder, MenuItemBuilder};
            use tauri::tray::{TrayIconBuilder, MouseButton, MouseButtonState, TrayIconEvent};

            let show_item = MenuItemBuilder::with_id("show", "显示窗口")
                .accelerator("Alt+Space")
                .build(app)?;
            
            let version_item = MenuItemBuilder::with_id("version", format!("版本: {}", commands::get_app_version(app_handle.clone()).unwrap_or("未知".to_string())))
                .build(app)?;
            
            let check_update_item = MenuItemBuilder::with_id("check_update", "检查更新")
                .build(app)?;
            
            let quit_item = MenuItemBuilder::with_id("quit", "退出")
                .build(app)?;
            
            let menu = MenuBuilder::new(app)
                .item(&show_item)
                .separator()
                .item(&version_item)
                .item(&check_update_item)
                .separator()
                .item(&quit_item)
                .build()?;

            let tray_icon = tauri::image::Image::from_bytes(include_bytes!("../icons/StoreLogo.png"))
                .expect("Failed to load tray icon");
            let _app_handle = app.handle().clone();
            let _tray = TrayIconBuilder::new()
                .icon(tray_icon)
                .menu(&menu)
                .tooltip("Godot Harbor")
                .on_menu_event(move |app, event| {
                    match event.id().as_ref() {
                        "show" => {
                            WINDOW_CLOSED.store(false, Ordering::SeqCst);
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                                let _ = window.set_always_on_top(true);
                                let win = window.clone();
                                tauri::async_runtime::spawn(async move {
                                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                                    let _ = win.set_always_on_top(false);
                                });
                            }
                        }
                        "check_update" => {
                            let app_clone = app.clone();
                            tauri::async_runtime::spawn(async move {
                                let _ = app_clone.emit("tray-check-update-start", ());

                                let mut has_updates = false;
                                let mut update_parts: Vec<String> = Vec::new();

                                if let Ok(result) = commands::check_all_updates(app_clone.clone(), None).await {
                                    if !result.plugin_updates.is_empty() {
                                        has_updates = true;
                                        update_parts.push(format!("{} 个插件更新", result.plugin_updates.len()));
                                        let _ = app_clone.emit("plugin-updates-available", &result.plugin_updates);
                                    }
                                    if !result.engine_updates.is_empty() {
                                        has_updates = true;
                                        update_parts.push(format!("{} 个引擎更新", result.engine_updates.len()));
                                        let _ = app_clone.emit("engine-updates-available", &result.engine_updates);
                                    }
                                }

                                if let Ok(Some(update)) = commands::check_app_update(app_clone.clone(), None).await {
                                    has_updates = true;
                                    update_parts.push(format!("应用更新 v{}", update.latest_version));
                                    let _ = app_clone.emit("app-update-available", &update);
                                }

                                if let Ok(Some(hot_update)) = commands::check_hot_update(app_clone.clone(), None).await {
                                    has_updates = true;
                                    update_parts.push(format!("热更新 {}", hot_update.version));
                                    let _ = app_clone.emit("hot-update-available", &hot_update);
                                }

                                if has_updates {
                                    let _ = app_clone.emit("updates-available", ());
                                    let _ = app_clone.emit("tray-check-update-result", serde_json::json!({
                                        "has_updates": true,
                                        "message": update_parts.join(", ")
                                    }));

                                    if let Some(window) = app_clone.get_webview_window("main") {
                                        let _ = window.show();
                                        let _ = window.set_focus();
                                    }

                                    let _ = app_clone.notification().builder()
                                        .title("发现更新")
                                        .body(&update_parts.join(", "))
                                        .show();
                                } else {
                                    let _ = app_clone.emit("tray-check-update-result", serde_json::json!({
                                        "has_updates": false,
                                        "message": "当前已是最新版本"
                                    }));

                                    let _ = app_clone.notification().builder()
                                        .title("Godot Harbor")
                                        .body("当前已是最新版本")
                                        .show();
                                }
                            });
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(move |tray, event| {
                    match event {
                        TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } => {
                            WINDOW_CLOSED.store(false, Ordering::SeqCst);
                            let app = tray.app_handle();
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                                let _ = window.set_always_on_top(true);
                                let win = window.clone();
                                tauri::async_runtime::spawn(async move {
                                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                                    let _ = win.set_always_on_top(false);
                                });
                            }
                        }
                        TrayIconEvent::DoubleClick { button: MouseButton::Left, .. } => {
                            WINDOW_CLOSED.store(false, Ordering::SeqCst);
                            let app = tray.app_handle();
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                                let _ = window.set_always_on_top(true);
                                let win = window.clone();
                                tauri::async_runtime::spawn(async move {
                                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                                    let _ = win.set_always_on_top(false);
                                });
                            }
                        }
                        _ => {}
                    }
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler!(
            commands::get_settings,
            commands::save_settings,
            commands::get_default_scan_dirs,
            commands::scan_projects,
            commands::get_projects,
            commands::add_project,
            commands::import_project_from_git,
            commands::remove_project,
            commands::import_plugin_from_local,
            commands::import_plugin_from_git,
            commands::list_git_refs,
            commands::import_plugin_from_url,
            commands::get_plugins,
            commands::remove_plugin,
            commands::bind_plugin,
            commands::unbind_plugin,
            commands::apply_changes,
            commands::list_addon_backups,
            commands::restore_addon_backup,
            commands::save_as_template,
            commands::list_templates,
            commands::delete_template,
            commands::apply_template_to_project,
            commands::get_project_bindings,
            commands::get_all_project_bindings,
            commands::scan_project_plugins,
            commands::import_plugins_from_projects,
            commands::get_operation_logs,
            commands::log_client_error,
            commands::toggle_plugin_favorite,
            commands::update_project_group,
            commands::get_project_groups,
            commands::backup_data,
            commands::restore_data,
            commands::reset_data,
            commands::register_engine,
            commands::get_engines,
            commands::remove_engine,
            commands::check_plugin_updates,
            commands::resolve_plugin_dependencies,
            commands::search_asset_library,
            commands::import_from_asset_library,
            commands::get_asset_library_configure,
            commands::get_asset_detail,
            commands::import_from_asset_library_with_progress,
            commands::import_project_from_asset_library,
            commands::read_harbor_config,
            commands::read_harbor_config_raw,
            commands::write_harbor_config,
            commands::delete_harbor_config,
            commands::sync_harbor_config,
            commands::check_harbor_configs,
            commands::check_uid_conflicts,
            commands::get_dashboard_stats,
            commands::auto_scan_projects,
            commands::relocate_project,
            commands::detect_moved_projects,
            commands::confirm_project_relocation,
            commands::sync_projects,
            commands::restart_fs_watcher,
            commands::auto_discover_engines,
            commands::check_godot_updates,
            commands::batch_remove_projects,
            commands::batch_remove_plugins,
            commands::batch_bind_plugins,
            commands::batch_unbind_plugins,
            commands::batch_apply_changes,
            commands::enable_plugin_in_project,
            commands::disable_plugin_in_project,
            commands::get_enabled_plugins,
            commands::get_plugin_storage_stats,
            commands::remove_plugin_version,
            commands::get_plugin_bindings,
            commands::check_binding_health,
            commands::repair_binding,
            commands::check_plugin_duplicate,
            commands::get_total_storage_stats,
            commands::cleanup_orphaned_plugin_dirs,
            commands::update_git_plugin,
            commands::batch_update_plugins,
            commands::skip_app_version,
            commands::check_app_update,
            commands::install_app_update,
            commands::check_all_updates,
            commands::get_app_version,
            commands::check_hot_update,
            commands::install_hot_update,
            commands::rollback_hot_update,
            commands::get_current_hot_update_version,
            commands::get_update_history,
            commands::clear_update_history,
            commands::check_engine_health,
            commands::rename_engine,
            commands::launch_engine,
            commands::find_matching_engines,
            commands::set_project_default_engine,
            commands::open_in_file_manager,
            commands::read_file_as_base64,
            commands::check_auto_setup_needed,
            commands::mark_auto_setup_done,
            commands::fetch_remote_engine_versions,
            commands::download_engine,
            commands::download_engine_from_url,
            commands::cancel_engine_download,
            commands::get_active_downloads,
            commands::cleanup_download_temp,
            commands::get_storage_paths,
            commands::migrate_data_dir,
            featured::get_featured_plugins,
            featured::report_usage_ping,
            featured::record_plugin_install,
        ))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}