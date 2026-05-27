use std::path::Path;
use std::fs;
use tauri::{AppHandle, State};
use crate::models::*;
use crate::AppState;
use crate::operation_log::LogEntry;
use crate::utils::no_window_cmd;
use super::utils::*;
use super::plugin::TotalStorageStats;

#[tauri::command]
pub fn read_file_as_base64(path: String) -> Result<String, String> {
    use std::io::Read;
    let mut file = std::fs::File::open(&path).map_err(|e| format!("打开文件失败: {}", e))?;
    let mut data = Vec::new();
    file.read_to_end(&mut data).map_err(|e| format!("读取文件失败: {}", e))?;
    Ok(base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &data))
}

#[tauri::command]
pub fn get_default_scan_dirs() -> Vec<String> {
    let mut dirs = Vec::new();
    if cfg!(windows) {
        if let Some(userprofile) = std::env::var("USERPROFILE").ok() {
            dirs.push(format!("{}\\Documents", userprofile));
            dirs.push(format!("{}\\Desktop", userprofile));
        }
        for drive in ['D', 'E', 'F'] {
            let drive_path = format!("{}:\\", drive);
            if std::path::Path::new(&drive_path).exists() {
                dirs.push(drive_path);
            }
        }
    } else {
        if let Some(home) = std::env::var("HOME").ok() {
            dirs.push(format!("{}/Documents", home));
            dirs.push(format!("{}/projects", home));
        }
    }
    dirs
}












#[tauri::command]
pub fn restart_fs_watcher(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let settings = load_settings(&app);

    let dirs = if settings.scan_directories.is_empty() {
        get_default_scan_dirs()
    } else {
        settings.scan_directories
    };

    {
        let guard = state.fs_watcher.lock().map_err(|e| format!("获取监听状态锁失败: {}", e))?;
        guard.start(app.clone(), dirs)?;
    }

    log_operation(&app, "restart_fs_watcher", "", "文件系统监听已重启");
    Ok(())
}


#[tauri::command]
pub async fn get_total_storage_stats(app: AppHandle) -> Result<TotalStorageStats, String> {
    let storage = get_storage(&app);
    let plugins: Vec<Plugin> = storage.load_or_default("plugins.json");
    let bindings: Vec<ProjectBinding> = storage.load_or_default("bindings.json");

    let total_versions: usize = plugins.iter().map(|p| p.versions.len()).sum();

    let data_dir = get_data_dir(&app);
    let plugins_dir = data_dir.join("plugins");
    let plugin_ids: Vec<String> = plugins.iter().map(|p| p.plugin_id.clone()).collect();

    let (total_size_bytes, orphaned_size) = tokio::task::spawn_blocking(move || {
        let total_size = dir_size(&plugins_dir);
        let mut orphaned: u64 = 0;
        if plugins_dir.exists() {
            if let Ok(entries) = fs::read_dir(&plugins_dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let entry_path = entry.path();
                    if entry_path.is_dir() {
                        let dir_name = entry_path.file_name().unwrap_or_default().to_string_lossy().to_string();
                        if !plugin_ids.iter().any(|id| id == &dir_name) {
                            orphaned += dir_size(&entry_path);
                        }
                    }
                }
            }
        }
        (total_size, orphaned)
    }).await.map_err(|e| format!("计算存储统计失败: {}", e))?;

    let mut hash_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for p in &plugins {
        if !p.content_hash.is_empty() {
            *hash_counts.entry(p.content_hash.clone()).or_insert(0) += 1;
        }
    }
    let duplicate_hash_count = hash_counts.values().filter(|&&c| c > 1).count();

    Ok(TotalStorageStats {
        total_plugins: plugins.len(),
        total_versions,
        total_bindings: bindings.len(),
        total_size_bytes,
        total_size_display: format_size(total_size_bytes),
        orphaned_size_bytes: orphaned_size,
        orphaned_size_display: format_size(orphaned_size),
        duplicate_hash_count,
    })
}

#[tauri::command]
pub async fn cleanup_orphaned_plugin_dirs(app: AppHandle) -> Result<u64, String> {
    let storage = get_storage(&app);
    let plugins: Vec<Plugin> = storage.load_or_default("plugins.json");
    let plugins_dir = get_data_dir(&app).join("plugins");
    let plugin_ids: Vec<String> = plugins.iter().map(|p| p.plugin_id.clone()).collect();

    let cleaned = tokio::task::spawn_blocking(move || {
        let mut count = 0u64;
        if plugins_dir.exists() {
            if let Ok(entries) = fs::read_dir(&plugins_dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let entry_path = entry.path();
                    if entry_path.is_dir() {
                        let dir_name = entry_path.file_name().unwrap_or_default().to_string_lossy().to_string();
                        if !plugin_ids.iter().any(|id| id == &dir_name) {
                            if let Ok(()) = fs::remove_dir_all(&entry_path) {
                                count += 1;
                            }
                        }
                    }
                }
            }
        }
        count
    }).await.map_err(|e| format!("清理孤立目录失败: {}", e))?;

    log_operation(&app, "cleanup_orphaned", "", &format!("清理了 {} 个孤立目录", cleaned));
    Ok(cleaned)
}


#[tauri::command]
pub fn get_operation_logs(app: AppHandle, limit: Option<usize>) -> Result<Vec<LogEntry>, String> {
    let logger = get_logger(&app);
    logger.get_logs(limit.unwrap_or(100))
        .map_err(|e| format!("获取操作日志失败: {}", e))
}

#[tauri::command]
pub fn log_client_error(app: AppHandle, source: String, error: String) -> Result<(), String> {
    log_error(&app, &source, "", &error);
    Ok(())
}


#[tauri::command]
pub fn get_dashboard_stats(app: AppHandle) -> Result<DashboardStats, String> {
    let storage = get_storage(&app);

    let projects: Vec<Project> = storage.load_or_default("projects.json");
    let plugins: Vec<Plugin> = storage.load_or_default("plugins.json");
    let engines: Vec<Engine> = storage.load_or_default("engines.json");
    let bindings: Vec<ProjectBinding> = storage.load_or_default("bindings.json");

    let mut total_bindings = 0;
    for project in &projects {
        let project_bindings: Vec<ProjectBinding> = bindings.iter()
            .filter(|b| b.project_id == project.project_id)
            .cloned()
            .collect();
        total_bindings += project_bindings.len();
    }

    let mut recent_projects = projects.clone();
    recent_projects.sort_by(|a, b| {
        match (a.last_opened_at, b.last_opened_at) {
            (Some(a_time), Some(b_time)) => b_time.cmp(&a_time),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => b.updated_at.cmp(&a.updated_at),
        }
    });
    recent_projects.truncate(8);

    let drift_count = projects.iter().filter(|p| {
        let config_path = crate::harbor_config::get_harbor_config_path(&p.path);
        if !config_path.exists() {
            return false;
        }
        if let Ok(Some(config)) = crate::harbor_config::read_harbor_config_from_project(&p.path) {
            let config_upgraded = if config.version < 2 { config.upgrade_to_v2() } else { config };
            let project_bindings: Vec<&ProjectBinding> = bindings.iter()
                .filter(|b| b.project_id == p.project_id)
                .collect();
            let mut has_drift = false;
            if let Some(ref godot_cfg) = config_upgraded.godot {
                let engine_match = engines.iter().find(|e| {
                    let ev: Vec<&str> = e.version.split('.').collect();
                    let tv: Vec<&str> = godot_cfg.version.split('.').collect();
                    if ev.len() >= 2 && tv.len() >= 2 {
                        ev[0] == tv[0] && ev[1] == tv[1] && e.is_mono == godot_cfg.mono
                    } else {
                        e.version == godot_cfg.version && e.is_mono == godot_cfg.mono
                    }
                });
                if engine_match.is_none() {
                    has_drift = true;
                }
            }
            if !has_drift {
                for pc in &config_upgraded.plugins {
                    let binding_exists = project_bindings.iter().any(|b| {
                        plugins.iter().find(|p| p.plugin_id == b.plugin_id)
                            .map_or(false, |p| p.name.to_lowercase() == pc.name.to_lowercase())
                    });
                    if !binding_exists {
                        has_drift = true;
                        break;
                    }
                }
            }
            has_drift
        } else {
            false
        }
    }).count();

    Ok(DashboardStats {
        project_count: projects.len(),
        plugin_count: plugins.len(),
        binding_count: total_bindings,
        engine_count: engines.len(),
        recent_projects,
        drift_count,
    })
}

#[tauri::command]
pub fn open_in_file_manager(path: String) -> Result<(), String> {
    let p = Path::new(&path);
    if !p.exists() {
        return Err(format!("路径不存在: {}", path));
    }
    #[cfg(target_os = "windows")]
    {
        let target = if p.is_dir() { path.clone() } else { p.parent().map(|d| d.to_string_lossy().to_string()).unwrap_or(path.clone()) };
        no_window_cmd("explorer")
            .arg(&target)
            .spawn()
            .map_err(|e| format!("打开文件管理器失败: {}", e))?;
    }
    #[cfg(target_os = "macos")]
    {
        let mut cmd = std::process::Command::new("open");
        if !p.is_dir() {
            cmd.arg("-R");
        }
        cmd.arg(&path)
            .spawn()
            .map_err(|e| format!("打开 Finder 失败: {}", e))?;
    }
    #[cfg(target_os = "linux")]
    {
        let target = if p.is_dir() { path.clone() } else { p.parent().map(|d| d.to_string_lossy().to_string()).unwrap_or(path.clone()) };
        std::process::Command::new("xdg-open")
            .arg(&target)
            .spawn()
            .map_err(|e| format!("打开文件管理器失败: {}", e))?;
    }
    Ok(())
}

