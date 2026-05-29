================================================================================
| Software: GodotHarbor, Version: v2.2.4 | Page 1 |
================================================================================
// File: src-tauri\src\featured.rs
use serde::{Serialize, Deserialize};
use std::fs;
use crate::storage::Storage;
use crate::models::Settings;
use tauri::Manager;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeaturedPlugin {
    pub name: String,
    pub description: String,
    pub author: String,
    pub source_url: String,
    pub compatibility: String,
    pub tags: Vec<String>,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeaturedPluginsList {
    pub version: u32,
    pub updated_at: String,
    pub plugins: Vec<FeaturedPlugin>,
}

impl FeaturedPluginsList {
    pub fn builtin() -> Self {
        Self {
            version: 1,
            updated_at: "2025-05-14".to_string(),
            plugins: vec![
                FeaturedPlugin {
                    name: "GodotSteam".to_string(),
                    description: "Steam API integration for Godot".to_string(),
                    author: "Gramps".to_string(),
                    source_url: "<url>".to_string(),
                    compatibility: "Godot4".to_string(),
                    tags: vec!["steam".into(), "multiplayer".into(), "sdk".into()],
                    category: "integration".to_string(),
                },
                FeaturedPlugin {
                    name: "Dialogic".to_string(),
                    description: "Create dialog systems and timelines".to_string(),
                    author: "Jowan".to_string(),
                    source_url: "<url>".to_string(),
                    compatibility: "Godot4".to_string(),
                    tags: vec!["dialog".into(), "narrative".into(), "visual-novel".into()],
                    category: "gameplay".to_string(),
                },

================================================================================
| Software: GodotHarbor, Version: v2.2.4 | Page 2 |
================================================================================
                FeaturedPlugin {
                    name: "PhantomCamera".to_string(),
                    description: "Powerful camera system for Godot".to_string(),
                    author: "Marcel".to_string(),
                    source_url: "<url>".to_string(),
                    compatibility: "Godot4".to_string(),
                    tags: vec!["camera".into(), "cinematic".into()],
                    category: "gameplay".to_string(),
                },
                FeaturedPlugin {
                    name: "GUT".to_string(),
                    description: "Godot Unit Testing framework".to_string(),
                    author: "Butch".to_string(),
                    source_url: "<url>".to_string(),
                    compatibility: "Both".to_string(),
                    tags: vec!["testing".into(), "unit-test".into()],
                    category: "development".to_string(),
                },
                FeaturedPlugin {
                    name: "LimboAI".to_string(),
                    description: "Behavior trees and state machines for Godot".to_string(),
                    author: "LimboAI".to_string(),
                    source_url: "<url>".to_string(),
                    compatibility: "Godot4".to_string(),
                    tags: vec!["ai".into(), "behavior-tree".into(), "state-machine".into()],
                    category: "gameplay".to_string(),
                },
                FeaturedPlugin {
                    name: "Aseprite Wizard".to_string(),
                    description: "Import Aseprite animations into Godot".to_string(),
                    author: "Vinicius".to_string(),
                    source_url: "<url>".to_string(),
                    compatibility: "Godot4".to_string(),
                    tags: vec!["animation".into(), "aseprite".into(), "sprite".into()],
                    category: "art".to_string(),
                },
                FeaturedPlugin {
                    name: "Terrain3D".to_string(),
                    description: "High performance terrain system for Godot".to_string(),
                    author: "Cory".to_string(),
                    source_url: "<url>".to_string(),
                    compatibility: "Godot4".to_string(),
                    tags: vec!["terrain".into(), "3d".into(), "landscape".into()],
                    category: "3d".to_string(),
                },
                FeaturedPlugin {
                    name: "Godot SQLite".to_string(),
                    description: "SQLite database integration for Godot".to_string(),
                    author: "Khairul".to_string(),
                    source_url: "<url>".to_string(),

================================================================================
| Software: GodotHarbor, Version: v2.2.4 | Page 3 |
================================================================================
                    compatibility: "Both".to_string(),
                    tags: vec!["database".into(), "sqlite".into(), "storage".into()],
                    category: "data".to_string(),
                },
                FeaturedPlugin {
                    name: "Kuhltimate Pixelizer".to_string(),
                    description: "Real-time pixel art post-processing".to_string(),
                    author: "Kuhltimate".to_string(),
                    source_url: "<url>".to_string(),
                    compatibility: "Godot4".to_string(),
                    tags: vec!["pixel-art".into(), "shader".into(), "post-processing".into()],
                    category: "art".to_string(),
                },
                FeaturedPlugin {
                    name: "GodotTIE".to_string(),
                    description: "Text Interface Engine for dialog and text effects".to_string(),
                    author: "Jan".to_string(),
                    source_url: "<url>".to_string(),
                    compatibility: "Godot4".to_string(),
                    tags: vec!["text".into(), "dialog".into(), "typewriter".into()],
                    category: "ui".to_string(),
                },
            ],
        }
    }
}

#[tauri::command]
pub fn get_featured_plugins(app: tauri::AppHandle) -> Result<FeaturedPluginsList, String> {
    let data_dir = crate::commands::get_data_dir(&app);
    let cache_path = data_dir.join("cache").join("featured_plugins.json");

    if cache_path.exists() {
        if let Ok(content) = fs::read_to_string(&cache_path) {
            if let Ok(list) = serde_json::from_str::<FeaturedPluginsList>(&content) {
                return Ok(list);
            }
        }
    }

    Ok(FeaturedPluginsList::builtin())
}

#[tauri::command]
pub async fn report_usage_ping(app: tauri::AppHandle) -> Result<(), String> {
    let config_dir = app.path().app_data_dir()
        .map_err(|e| format!("获取配置目录失败: {}", e))?;
    let config_storage = Storage::new(config_dir);
    let mut settings: Settings = config_storage.load_or_default("settings.json");


================================================================================
| Software: GodotHarbor, Version: v2.2.4 | Page 4 |
================================================================================
    if !settings.enable_anonymous_usage_stats {
        return Ok(());
    }

    if settings.anonymous_user_id.is_empty() {
        settings.anonymous_user_id = Uuid::new_v4().to_string();
        config_storage.save("settings.json", &settings)
            .map_err(|e| format!("保存匿名ID失败: {}", e))?;
    }

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let data_dir = crate::commands::get_data_dir(&app);
    let last_ping_file = data_dir.join(".last_usage_ping");
    if last_ping_file.exists() {
        if let Ok(last_ping) = std::fs::read_to_string(&last_ping_file) {
            if last_ping.trim() == today {
                return Ok(());
            }
        }
    }

    let version = crate::commands::get_app_version(app.clone()).unwrap_or_default();
    let os = std::env::consts::OS.to_string();
    let timestamp = chrono::Utc::now().to_rfc3339();

    let uuid = settings.anonymous_user_id.clone();
    let ping_url = format!(
        "<url>?u={}&v={}&o={}&t={}",
        urlencoding::encode(&uuid),
        urlencoding::encode(&version),
        urlencoding::encode(&os),
        urlencoding::encode(&timestamp)
    );

    let client = crate::utils::create_http_client(Some(std::time::Duration::from_secs(3)))?;

    let _ = client.get(&ping_url)
        .header("User-Agent", "GodotHarbor")
        .timeout(std::time::Duration::from_secs(3))
        .send()
        .await;

    let _ = std::fs::write(&last_ping_file, &today);

    Ok(())
}

#[tauri::command]
pub fn record_plugin_install(app: tauri::AppHandle, plugin_id: String) -> Result<(), String> {
    let storage = crate::commands::get_storage(&app);

================================================================================
| Software: GodotHarbor, Version: v2.2.4 | Page 5 |
================================================================================
    let mut plugins: Vec<crate::models::Plugin> = storage.load_or_default("plugins.json");

    if let Some(plugin) = plugins.iter_mut().find(|p| p.plugin_id == plugin_id) {
        plugin.install_count += 1;
        storage.save("plugins.json", &plugins)
            .map_err(|e| format!("保存插件统计失败: {}", e))?;
    }

    Ok(())
}

// File: src-tauri\src\lib.rs
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
pub mod asset_store;
pub mod mcp;

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

================================================================================
| Software: GodotHarbor, Version: v2.2.4 | Page 6 |
================================================================================
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

================================================================================
| Software: GodotHarbor, Version: v2.2.4 | Page 7 |
================================================================================
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

            let builtin_handle = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                let _ = commands::ensure_builtin_templates(builtin_handle);

================================================================================
| Software: GodotHarbor, Version: v2.2.4 | Page 8 |
================================================================================
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

================================================================================
| Software: GodotHarbor, Version: v2.2.4 | Page 9 |
================================================================================
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

================================================================================
| Software: GodotHarbor, Version: v2.2.4 | Page 10 |
================================================================================
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

================================================================================
| Software: GodotHarbor, Version: v2.2.4 | Page 11 |
================================================================================
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

================================================================================
| Software: GodotHarbor, Version: v2.2.4 | Page 12 |
================================================================================
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

================================================================================
| Software: GodotHarbor, Version: v2.2.4 | Page 13 |
================================================================================
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
            commands::search_assets,
            commands::get_asset_detail_v2,
            commands::get_asset_store_categories,
            commands::check_asset_api_availability,
            commands::list_hub_templates,
            commands::get_hub_template,
            commands::save_hub_template,
            commands::delete_hub_template,
            commands::import_template_from_url,
            commands::instantiate_template,
            commands::generate_template_from_project,
            commands::ensure_builtin_templates,
            commands::read_harbor_config,
            commands::read_harbor_config_raw,
            commands::write_harbor_config,
            commands::delete_harbor_config,
            commands::sync_harbor_config,
            commands::check_harbor_configs,
            commands::check_project_drift,
            commands::check_all_drifts,
            commands::preview_sync,
            commands::sync_project_environment,
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

================================================================================
| Software: GodotHarbor, Version: v2.2.4 | Page 14 |
================================================================================
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
            commands::list_export_templates,
            commands::download_export_template,
            commands::delete_export_template,
            commands::list_export_presets,
            commands::apply_export_preset,
            commands::save_export_preset_to_harbor,
            commands::build_project,
            commands::get_build_records,
            commands::delete_build_record,

================================================================================
| Software: GodotHarbor, Version: v2.2.4 | Page 15 |
================================================================================
            commands::generate_github_actions,
            commands::generate_gitlab_ci,
            commands::write_ci_config,
            commands::get_builtin_export_presets,
            commands::export_preset_to_json,
            commands::import_preset_from_json,
            commands::start_mcp_server,
        ))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// File: src-tauri\src\main.rs
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    godot_harbor_lib::run()
}

// File: src-tauri\src\operation_log.rs
use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub action: String,
    pub target: String,
    pub detail: String,
}

pub struct OperationLogger {
    log_dir: PathBuf,
}

impl OperationLogger {
    pub fn new(data_dir: PathBuf) -> Self {
        let log_dir = data_dir.join("logs");
        fs::create_dir_all(&log_dir).ok();
        Self { log_dir }
    }

    fn write_entry(&self, entry: &LogEntry) -> Result<()> {
        let date = entry.timestamp.format("%Y-%m-%d").to_string();
        let log_file = self.log_dir.join(format!("{}.jsonl", date));


================================================================================
| Software: GodotHarbor, Version: v2.2.4 | Page 16 |
================================================================================
        let mut line = serde_json::to_string(entry)?;
        line.push('\n');

        use std::io::Write;
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file)?;
        file.write_all(line.as_bytes())?;

        Ok(())
    }

    pub fn log(&self, action: &str, target: &str, detail: &str) -> Result<()> {
        let entry = LogEntry {
            timestamp: Utc::now(),
            level: "success".to_string(),
            action: action.to_string(),
            target: target.to_string(),
            detail: detail.to_string(),
        };
        self.write_entry(&entry)
    }

    pub fn log_error(&self, action: &str, target: &str, error: &str) -> Result<()> {
        let entry = LogEntry {
            timestamp: Utc::now(),
            level: "error".to_string(),
            action: action.to_string(),
            target: target.to_string(),
            detail: error.to_string(),
        };
        self.write_entry(&entry)
    }

    pub fn get_logs(&self, limit: usize) -> Result<Vec<LogEntry>> {
        let mut entries = Vec::new();

        if !self.log_dir.exists() {
            return Ok(entries);
        }

        let mut files: Vec<_> = fs::read_dir(&self.log_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path().extension().map(|ext| ext == "jsonl").unwrap_or(false)
            })
            .collect();

        files.sort_by(|a, b| b.file_name().cmp(&a.file_name()));

================================================================================
| Software: GodotHarbor, Version: v2.2.4 | Page 17 |
================================================================================

        for file in files {
            let content = fs::read_to_string(file.path())?;
            for line in content.lines().rev() {
                if let Ok(entry) = serde_json::from_str::<LogEntry>(line) {
                    entries.push(entry);
                    if entries.len() >= limit {
                        entries.reverse();
                        return Ok(entries);
                    }
                }
            }
        }

        entries.reverse();
        Ok(entries)
    }
}

// File: src-tauri\src\asset_store\mod.rs
use crate::utils::create_http_client;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

const ASSET_STORE_BASE: &str = "<url>";
const ASSET_STORE_API_BASE: &str = "<url>";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetStoreSearchResult {
    pub results: Vec<AssetStoreAsset>,
    pub page: u32,
    pub pages: u32,
    pub total: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetStoreAsset {
    pub asset_id: String,
    pub title: String,
    pub author: String,
    pub author_id: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub godot_version_min: String,
    #[serde(default)]
    pub godot_version_max: String,
    #[serde(default)]

================================================================================
| Software: GodotHarbor, Version: v2.2.4 | Page 18 |
================================================================================
    pub license: String,
    #[serde(default)]
    pub cost: String,
    #[serde(default)]
    pub icon_url: String,
    #[serde(default)]
    pub rating: f64,
    #[serde(default)]
    pub review_count: u32,
    #[serde(default)]
    pub download_url: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub version_string: String,
    #[serde(default)]
    pub modify_date: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub source_url: String,
    #[serde(default)]
    pub store_url: String,
    #[serde(default)]
    pub asset_type: String,
    #[serde(default)]
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetStoreRelease {
    pub version_id: String,
    pub version: String,
    #[serde(default)]
    pub changelog: String,
    #[serde(default)]
    pub download_url: String,
    #[serde(default)]
    pub sha256: String,
    #[serde(default)]
    pub godot_version_min: String,
    #[serde(default)]
    pub godot_version_max: String,
    #[serde(default)]
    pub file_size: u64,
    #[serde(default)]
    pub published_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]

================================================================================
| Software: GodotHarbor, Version: v2.2.4 | Page 19 |
================================================================================
pub struct AssetStoreDetail {
    pub asset_id: String,
    pub title: String,
    pub author: String,
    pub author_id: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub godot_version_min: String,
    #[serde(default)]
    pub godot_version_max: String,
    #[serde(default)]
    pub license: String,
    #[serde(default)]
    pub cost: String,
    #[serde(default)]
    pub icon_url: String,
    #[serde(default)]
    pub rating: f64,
    #[serde(default)]
    pub review_count: u32,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub source_url: String,
    #[serde(default)]
    pub store_url: String,
    #[serde(default)]
    pub asset_type: String,
    #[serde(default)]
    pub verified: bool,
    #[serde(default)]
    pub releases: Vec<AssetStoreRelease>,
    #[serde(default)]
    pub preview_images: Vec<String>,
}

pub struct AssetStoreClient {
    client: reqwest::Client,
    base_url: String,
}

impl AssetStoreClient {
    pub fn new(app: &AppHandle) -> Result<Self, String> {
        let client = create_http_client(Some(std::time::Duration::from_secs(60)))?;
        let base_url = {
            let storage = crate::commands::get_storage(app);
            let settings: crate::models::Settings = storage.load_or_default("settings.json");

================================================================================
| Software: GodotHarbor, Version: v2.2.4 | Page 20 |
================================================================================
            if !settings.asset_library_mirror.is_empty() {
                settings.asset_library_mirror.trim_end_matches('/').to_string()
            } else {
                ASSET_STORE_API_BASE.to_string()
            }
        };
        Ok(Self { client, base_url })
    }

    pub fn store_page_url(&self, publisher: &str, asset_slug: &str) -> String {
        format!("{}/asset/{}/{}", ASSET_STORE_BASE, publisher, asset_slug)
    }

    pub fn download_url(&self, publisher: &str, asset_slug: &str, version_id: &str) -> String {
        format!("{}/asset/{}/{}/download/{}", ASSET_STORE_BASE, publisher, asset_slug, version_id)
    }

    pub async fn search(
        &self,
        query: &str,
        page: u32,
        max_results: u32,
        sort: &str,
        category: Option<&str>,
        godot_version: Option<&str>,
        tags: Option<&[String]>,
    ) -> Result<serde_json::Value, String> {
        let mut params = vec![
            ("query", query.to_string()),
            ("page", page.to_string()),
            ("max_results", max_results.to_string()),
            ("sort", sort.to_string()),
        ];

        if let Some(cat) = category {
            params.push(("category", cat.to_string()));
        }
        if let Some(gv) = godot_version {
            params.push(("godot_version", gv.to_string()));
        }
        if let Some(tag_list) = tags {
            for tag in tag_list {
                params.push(("tag", tag.clone()));
            }
        }

        let url = format!("{}/asset", self.base_url);
        let resp = self.client.get(&url)
            .query(&params)
            .send()

================================================================================
| Software: GodotHarbor, Version: v2.2.4 | Page 21 |
================================================================================
            .await
            .map_err(|e| format!("请求 Asset Store 搜索失败: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("Asset Store 搜索返回错误: {}", resp.status()));
        }

        let text = resp.text().await
            .map_err(|e| format!("读取 Asset Store 响应失败: {}", e))?;

        if text.trim().starts_with('<') {
            return Err("Asset Store 返回了非 JSON 响应，可能服务暂时不可用，请尝试切换到 Legacy 模式".to_string());
        }

        serde_json::from_str(&text)
            .map_err(|e| format!("解析 Asset Store 搜索结果失败: {} (响应前200字符: {})", e, &text[..text.len().min(200)]))
    }

    pub async fn get_detail(&self, asset_id: &str) -> Result<serde_json::Value, String> {
        let url = format!("{}/asset/{}", self.base_url, asset_id);
        let resp = self.client.get(&url)
            .send()
            .await
            .map_err(|e| format!("请求 Asset Store 详情失败: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("Asset Store 详情返回错误: {}", resp.status()));
        }

        let text = resp.text().await
            .map_err(|e| format!("读取 Asset Store 详情响应失败: {}", e))?;
        if text.trim().starts_with('<') {
            return Err("Asset Store 返回了非 JSON 响应，可能服务暂时不可用".to_string());
        }
        serde_json::from_str(&text)
            .map_err(|e| format!("解析 Asset Store 详情失败: {}", e))
    }

    pub async fn get_releases(&self, asset_id: &str) -> Result<serde_json::Value, String> {
        let url = format!("{}/asset/{}/releases", self.base_url, asset_id);
        let resp = self.client.get(&url)
            .send()
            .await
            .map_err(|e| format!("请求 Asset Store 版本列表失败: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("Asset Store 版本列表返回错误: {}", resp.status()));
        }

        let text = resp.text().await

================================================================================
| Software: GodotHarbor, Version: v2.2.4 | Page 22 |
================================================================================
            .map_err(|e| format!("读取 Asset Store 版本列表响应失败: {}", e))?;
        if text.trim().starts_with('<') {
            return Err("Asset Store 返回了非 JSON 响应，可能服务暂时不可用".to_string());
        }
        serde_json::from_str(&text)
            .map_err(|e| format!("解析 Asset Store 版本列表失败: {}", e))
    }

    pub async fn get_categories(&self) -> Result<serde_json::Value, String> {
        let url = format!("{}/category", self.base_url);
        let resp = self.client.get(&url)
            .send()
            .await
            .map_err(|e| format!("请求 Asset Store 分类失败: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("Asset Store 分类返回错误: {}", resp.status()));
        }

        let text = resp.text().await
            .map_err(|e| format!("读取 Asset Store 分类响应失败: {}", e))?;
        if text.trim().starts_with('<') {
            return Err("Asset Store 返回了非 JSON 响应，可能服务暂时不可用".to_string());
        }
        serde_json::from_str(&text)
            .map_err(|e| format!("解析 Asset Store 分类失败: {}", e))
    }

    pub async fn check_available(&self) -> bool {
        let url = format!("{}/asset", self.base_url);
        match self.client.get(&url)
            .query(&[("max_results", "1")])
            .send()
            .await
        {
            Ok(resp) => {
                if !resp.status().is_success() {
                    return false;
                }
                match resp.text().await {
                    Ok(text) => !text.trim().starts_with('<'),
                    Err(_) => false,
                }
            }
            Err(_) => false,
        }
    }
}

pub fn get_asset_store_base(app: &tauri::AppHandle) -> String {

================================================================================
| Software: GodotHarbor, Version: v2.2.4 | Page 23 |
================================================================================
    let storage = crate::commands::get_storage(app);
    let settings: crate::models::Settings = storage.load_or_default("settings.json");
    if !settings.asset_library_mirror.is_empty() {
        settings.asset_library_mirror.trim_end_matches('/').to_string()
    } else {
        ASSET_STORE_API_BASE.to_string()
    }
}

// File: src-tauri\src\bin\mcp_server.rs
fn main() {
    godot_harbor_lib::mcp::server::run_mcp_server();
}

// File: src-tauri\src\commands\asset.rs
use std::fs;
use serde::{Serialize, Deserialize};
use tauri::{AppHandle, Emitter};
use crate::models::*;
use uuid::Uuid;
use crate::utils::create_http_client;
use super::utils::*;
use super::plugin::AssetLibrarySearchParams;

#[tauri::command]
pub async fn search_asset_library(app: AppHandle, params: AssetLibrarySearchParams) -> Result<serde_json::Value, String> {
    let mut url_params = vec![];

    if let Some(f) = &params.filter {
        url_params.push(format!("filter={}", urlencoding::encode(f)));
    } else {
        url_params.push("filter=".to_string());
    }

    url_params.push(format!("type={}", params.asset_type.as_deref().unwrap_or("any")));

    if let Some(c) = &params.category {
        url_params.push(format!("category={}", c));
    }
    if let Some(s) = &params.support {
        url_params.push(format!("support={}", s));
    }
    if let Some(c) = &params.cost {
        url_params.push(format!("cost={}", c));
    }

    url_params.push(format!("godot_version={}", params.godot_version.as_deref().unwrap_or("any")));
    url_params.push(format!("max_results={}", params.max_results.unwrap_or(20)));

    if let Some(p) = params.page {

================================================================================
| Software: GodotHarbor, Version: v2.2.4 | Page 24 |
================================================================================
        url_params.push(format!("page={}", p));
    }

    url_params.push(format!("sort={}", params.sort.as_deref().unwrap_or("updated")));

    if params.reverse.unwrap_or(false) {
        url_params.push("reverse".to_string());
    }

    let asset_lib_base = crate::utils::get_asset_library_base(&app);
    let url = format!("{}/asset?{}", asset_lib_base, url_params.join("&"));

    let client = create_http_client(None)?;

    let resp = client.get(&url).send().await
        .map_err(|e| format!("请求 Asset Library 失败: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Asset Library 返回错误状态: {}", resp.status()));
    }

    let text = resp.text().await
        .map_err(|e| format!("读取 Asset Library 响应失败: {}", e))?;

    let json: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| format!("解析 Asset Library 响应失败: {} (响应前100字符: {})", e, &text[..text.len().min(100)]))?;

    let filter_str = params.filter.as_deref().unwrap_or("");
    log_operation(&app, "search_asset_library", "", &format!("搜索 Asset Library: {}", filter_str));
    Ok(json)
}

#[tauri::command]
pub async fn get_asset_library_configure(app: AppHandle) -> Result<serde_json::Value, String> {
    let asset_lib_base = crate::utils::get_asset_library_base(&app);
    let url = format!("{}/configure?type=any", asset_lib_base);

    let client = create_http_client(None)?;

    let resp = client.get(url).send().await
        .map_err(|e| format!("请求 Asset Library 配置失败: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Asset Library 返回错误状态: {}", resp.status()));
    }

    let json: serde_json::Value = resp.json().await
        .map_err(|e| format!("解析 Asset Library 配置失败: {}", e))?;

    log_operation(&app, "get_asset_library_configure", "", "获取 Asset Library 配置");

================================================================================
| Software: GodotHarbor, Version: v2.2.4 | Page 25 |
================================================================================
    Ok(json)
}

#[tauri::command]
pub async fn get_asset_detail(app: AppHandle, asset_id: String) -> Result<serde_json::Value, String> {
    let asset_lib_base = crate::utils::get_asset_library_base(&app);
    let url = format!(
        "{}/asset/{}",
        asset_lib_base, asset_id
    );

    let client = create_http_client(None)?;

    let resp = client.get(&url).send().await
        .map_err(|e| format!("请求 Asset Library 失败: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Asset Library 返回错误状态: {}", resp.status()));
    }

    let json: serde_json::Value = resp.json().await
        .map_err(|e| format!("解析 Asset Library 响应失败: {}", e))?;

    log_operation(&app, "get_asset_detail", &asset_id, &format!("获取资产详情: {}", asset_id));
    Ok(json)
}

#[tauri::command]
pub async fn import_from_asset_library(app: AppHandle, asset_id: String) -> Result<Plugin, String> {
    let asset_lib_base = crate::utils::get_asset_library_base(&app);
    let url = format!(
        "{}/asset/{}",
        asset_lib_base, asset_id
    );

    let client = create_http_client(None)?;

    let resp = client.get(&url).send().await
        .map_err(|e| format!("请求 Asset Library 失败: {}", e))?;

    let asset: serde_json::Value = resp.json().await
        .map_err(|e| format!("解析 Asset Library 响应失败: {}", e))?;

    let download_url = asset.get("download_url")
        .and_then(|v| v.as_str())
        .ok_or("未找到下载链接")?;

    let asset_name = asset.get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")

================================================================================
| Software: GodotHarbor, Version: v2.2.4 | Page 26 |
================================================================================
        .to_string();

    let author_name = asset.get("author")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let desc = asset.get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let plugin_source = PluginSource {
        source_type: SourceType::AssetLibrary,
        url: format!("asset-library://{}", asset_id),
        git_ref: String::new(),
        imported_at: chrono::Utc::now(),
    };

    let mut plugin = Plugin::new(asset_name.clone(), plugin_source);
    plugin.description = desc;
    plugin.author = author_name;

    let version_id = Uuid::new_v4().to_string();
    let version_dir = get_data_dir(&app).join("plugins").join(&plugin.plugin_id).join(&version_id);
    let payload_dir = version_dir.join("payload");

    fs::create_dir_all(&payload_dir)
        .map_err(|e| format!("创建目录失败: {}", e))?;

    let temp_zip = version_dir.join("download.zip");
    let resp = client.get(download_url).send().await
        .map_err(|e| format!("下载资源失败: {}", e))?;

    let bytes = resp.bytes().await
        .map_err(|e| format!("读取下载数据失败: {}", e))?;

    fs::write(&temp_zip, &bytes)
        .map_err(|e| format!("写入文件失败: {}", e))?;

    let file = std::fs::File::open(&temp_zip)
        .map_err(|e| format!("打开压缩文件失败: {}", e))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| format!("解压失败: {}", e))?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| format!("读取压缩条目失败: {}", e))?;
        let outpath = match entry.enclosed_name() {
            Some(path) => payload_dir.join(path),
            None => continue,

================================================================================
| Software: GodotHarbor, Version: v2.2.4 | Page 27 |
================================================================================
        };
        if entry.is_dir() {
            std::fs::create_dir_all(&outpath).ok();
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(p).ok();
                }
            }
            let mut outfile = std::fs::File::create(&outpath)
                .map_err(|e| format!("创建文件失败: {}", e))?;
            std::io::copy(&mut entry, &mut outfile)
                .map_err(|e| format!("写入文件失败: {}", e))?;
        }
    }

    let _ = std::fs::remove_file(&temp_zip);

    let manager = get_plugin_manager(&app);
    let (units, asset_type) = manager.analyze_asset_type(&payload_dir, &asset_name);

    let compatibility = manager.detect_compatibility(&payload_dir);

    let content_hash = crate::models::compute_dir_hash(&payload_dir).unwrap_or_default();

    let (unit_version, unit_name) = if let Some(first_unit) = units.first() {
        (
            if first_unit.version.is_empty() { "1.0.0".to_string() } else { first_unit.version.clone() },
            if first_unit.name.is_empty() { asset_name.clone() } else { first_unit.name.clone() },
        )
    } else {
        ("1.0.0".to_string(), asset_name.clone())
    };

    let plugin_version = PluginVersion {
        version_id: version_id.clone(),
        version: unit_version,
        path: payload_dir.to_string_lossy().to_string(),
        created_at: chrono::Utc::now(),
        units,
    };

    plugin.versions.push(plugin_version);
    plugin.compatibility = compatibility;
    plugin.name = unit_name;
    plugin.content_hash = content_hash;
    plugin.asset_type = asset_type;

    upsert_plugin(&app, &plugin, "import_asset_library", &asset_id.to_string())
}

================================================================================
| Software: GodotHarbor, Version: v2.2.4 | Page 28 |
================================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetImportProgressPayload {
    pub asset_id: String,
    pub stage: String,
    pub progress: f64,
    pub message: String,
}

#[tauri::command]
pub async fn import_from_asset_library_with_progress(app: AppHandle, asset_id: String) -> Result<Plugin, String> {
    let _ = app.emit("asset-import-progress", AssetImportProgressPayload {
        asset_id: asset_id.clone(),
        stage: "downloading".to_string(),
        progress: 0.0,
        message: "正在获取资产信息...".to_string(),
    });

    let asset_lib_base = crate::utils::get_asset_library_base(&app);
    let url = format!(
        "{}/asset/{}",
        asset_lib_base, asset_id
    );

    let client = create_http_client(None)?;

    let resp = client.get(&url).send().await
        .map_err(|e| format!("请求 Asset Library 失败: {}", e))?;

    let asset: serde_json::Value = resp.json().await
        .map_err(|e| format!("解析 Asset Library 响应失败: {}", e))?;

    let download_url = asset.get("download_url")
        .and_then(|v| v.as_str())
        .ok_or("未找到下载链接")?;

    let asset_name = asset.get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let author_name = asset.get("author")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let desc = asset.get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

================================================================================
| Software: GodotHarbor, Version: v2.2.4 | Page 29 |
================================================================================

    let plugin_source = PluginSource {
        source_type: SourceType::AssetLibrary,
        url: format!("asset-library://{}", asset_id),
        git_ref: String::new(),
        imported_at: chrono::Utc::now(),
    };

    let mut plugin = Plugin::new(asset_name.clone(), plugin_source);
    plugin.description = desc;
    plugin.author = author_name;

    let version_id = Uuid::new_v4().to_string();
    let version_dir = get_data_dir(&app).join("plugins").join(&plugin.plugin_id).join(&version_id);
    let payload_dir = version_dir.join("payload");

    fs::create_dir_all(&payload_dir)
        .map_err(|e| format!("创建目录失败: {}", e))?;

    let _ = app.emit("asset-import-progress", AssetImportProgressPayload {
        asset_id: asset_id.clone(),
        stage: "downloading".to_string(),
        progress: 0.1,
        message: format!("正在下载 {}...", asset_name),
    });

    let temp_zip = version_dir.join("download.zip");
    let resp = client.get(download_url).send().await
        .map_err(|e| format!("下载资源失败: {}", e))?;

    let total_size = resp.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;
    let mut file = std::fs::File::create(&temp_zip)
        .map_err(|e| format!("创建临时文件失败: {}", e))?;
    let mut stream = resp.bytes_stream();
    use futures::StreamExt;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("读取下载数据失败: {}", e))?;
        std::io::Write::write_all(&mut file, &chunk)
            .map_err(|e| format!("写入文件失败: {}", e))?;
        downloaded += chunk.len() as u64;
        if total_size > 0 {
            let progress = 0.1 + 0.6 * (downloaded as f64 / total_size as f64);
            let _ = app.emit("asset-import-progress", AssetImportProgressPayload {
                asset_id: asset_id.clone(),
                stage: "downloading".to_string(),
                progress,
                message: format!("正在下载 {} ({:.0}/{:.0} KB)...", asset_name, downloaded as f64 / 1024.0, total_size as f64 / 1024.0),
            });

================================================================================
| Software: GodotHarbor, Version: v2.2.4 | Page 30 |
================================================================================
        }
    }
    drop(file);

    let _ = app.emit("asset-import-progress", AssetImportProgressPayload {
        asset_id: asset_id.clone(),
        stage: "extracting".to_string(),
        progress: 0.7,
        message: format!("正在解压 {}...", asset_name),
    });

    let file = std::fs::File::open(&temp_zip)
        .map_err(|e| format!("打开压缩文件失败: {}", e))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| format!("解压失败: {}", e))?;

    let total_entries = archive.len();
    for i in 0..total_entries {
        let mut entry = archive.by_index(i).map_err(|e| format!("读取压缩条目失败: {}", e))?;
        let outpath = match entry.enclosed_name() {
            Some(path) => payload_dir.join(path),
            None => continue,
        };
        if entry.is_dir() {
            std::fs::create_dir_all(&outpath).ok();
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(p).ok();
                }
            }
            let mut outfile = std::fs::File::create(&outpath)
                .map_err(|e| format!("创建文件失败: {}", e))?;
            std::io::copy(&mut entry, &mut outfile)
                .map_err(|e| format!("写入文件失败: {}", e))?;
        }
        let progress = 0.7 + 0.2 * ((i + 1) as f64 / total_entries as f64);
        let _ = app.emit("asset-import-progress", AssetImportProgressPayload {
            asset_id: asset_id.clone(),
            stage: "extracting".to_string(),
            progress,
            message: format!("正在解压 {} ({}/{})...", asset_name, i + 1, total_entries),
        });
    }

    let _ = std::fs::remove_file(&temp_zip);

    let _ = app.emit("asset-import-progress", AssetImportProgressPayload {
        asset_id: asset_id.clone(),
        stage: "parsing".to_string(),


