use std::path::{PathBuf, Path};
use std::fs;
use std::io::{Read, Write};
use serde::{Serialize, Deserialize};
use tauri::{AppHandle, Manager, Emitter, State};
use crate::models::*;
use crate::storage::Storage;
use crate::scanner::ProjectScanner;
use crate::plugin_manager::PluginManager;
use crate::linker::Linker;
use crate::operation_log::{OperationLogger, LogEntry};
use crate::AppState;
use uuid::Uuid;

fn get_data_dir(app: &AppHandle) -> PathBuf {
    app.path().app_data_dir()
        .expect("Failed to get app data directory")
}

fn get_storage(app: &AppHandle) -> Storage {
    let data_dir = get_data_dir(app);
    Storage::new(data_dir)
}

fn get_plugin_manager(app: &AppHandle) -> PluginManager {
    let data_dir = get_data_dir(app);
    PluginManager::new(data_dir.join("plugins"))
}

fn get_logger(app: &AppHandle) -> OperationLogger {
    let data_dir = get_data_dir(app);
    OperationLogger::new(data_dir)
}

fn log_operation(app: &AppHandle, action: &str, target: &str, detail: &str) {
    let logger = get_logger(app);
    if let Err(e) = logger.log(action, target, detail) {
        eprintln!("Failed to write operation log: {}", e);
    }
}

fn log_error(app: &AppHandle, action: &str, target: &str, error: &str) {
    let logger = get_logger(app);
    if let Err(e) = logger.log_error(action, target, error) {
        eprintln!("Failed to write error log: {}", e);
    }
}

#[tauri::command]
pub fn get_settings(app: AppHandle) -> Result<Settings, String> {
    let storage = get_storage(&app);
    let settings: Settings = storage.load_or_default("settings.json");
    Ok(settings)
}

#[tauri::command]
pub fn save_settings(app: AppHandle, settings: Settings) -> Result<(), String> {
    let storage = get_storage(&app);
    storage.save("settings.json", &settings)
        .map_err(|e| format!("保存设置失败: {}", e))?;
    log_operation(&app, "save_settings", "settings.json", "设置已保存");
    Ok(())
}

#[tauri::command]
pub fn scan_projects(app: AppHandle, root_dirs: Vec<String>) -> Result<Vec<Project>, String> {
    if root_dirs.is_empty() {
        log_error(&app, "scan_projects", "", "未提供扫描目录");
        return Err("请至少指定一个扫描目录".to_string());
    }

    let valid_dirs: Vec<String> = root_dirs.iter()
        .filter(|d| std::path::Path::new(d).exists())
        .cloned()
        .collect();

    if valid_dirs.is_empty() {
        return Err("所有指定的目录均不存在".to_string());
    }

    let all_projects = ProjectScanner::scan_directories_parallel(&valid_dirs)
        .map_err(|e| format!("扫描失败: {}", e))?;

    let storage = get_storage(&app);
    let mut existing_projects: Vec<Project> = storage.load_or_default("projects.json");

    for project in &all_projects {
        if let Some(index) = existing_projects.iter().position(|p| p.path == project.path) {
            let mut existing = existing_projects[index].clone();
            existing.name = project.name.clone();
            existing.godot_version = project.godot_version.clone();
            existing_projects[index] = existing;
        } else {
            existing_projects.push(project.clone());
        }
    }

    storage.save("projects.json", &existing_projects)
        .map_err(|e| format!("保存项目列表失败: {}", e))?;

    log_operation(&app, "scan_projects", &valid_dirs.join(", "),
        &format!("扫描完成，发现 {} 个项目", all_projects.len()));

    Ok(all_projects)
}

#[tauri::command]
pub fn get_projects(app: AppHandle) -> Result<Vec<Project>, String> {
    let storage = get_storage(&app);
    let mut projects: Vec<Project> = storage.load_or_default("projects.json");

    for project in projects.iter_mut() {
        let project_path = std::path::Path::new(&project.path);
        if !project_path.exists() || !project_path.join("project.godot").exists() {
            project.status = ProjectStatus::MissingSource;
        }
    }

    Ok(projects)
}

#[tauri::command]
pub fn add_project(app: AppHandle, path: String) -> Result<Project, String> {
    let project_path = std::path::Path::new(&path);

    if !project_path.exists() {
        return Err("指定的路径不存在".to_string());
    }

    let project_godot = project_path.join("project.godot");

    if !project_godot.exists() {
        return Err("指定路径下未找到 project.godot 文件，请确认是否为 Godot 项目目录".to_string());
    }

    let project = ProjectScanner::parse_project(&project_godot)
        .map_err(|e| format!("解析项目失败: {}", e))?;

    let storage = get_storage(&app);
    let mut projects: Vec<Project> = storage.load_or_default("projects.json");

    if projects.iter().any(|p| p.path == project.path) {
        return Err("该项目已存在，请勿重复添加".to_string());
    }

    let project_name = project.name.clone();
    projects.push(project.clone());
    storage.save("projects.json", &projects)
        .map_err(|e| format!("保存项目失败: {}", e))?;

    log_operation(&app, "add_project", &path, &format!("已添加项目: {}", project_name));
    Ok(project)
}

#[tauri::command]
pub fn remove_project(app: AppHandle, project_id: String) -> Result<(), String> {
    let storage = get_storage(&app);
    let mut projects: Vec<Project> = storage.load_or_default("projects.json");

    let project = projects.iter().find(|p| p.project_id == project_id)
        .ok_or("未找到指定项目".to_string())?;
    let project_name = project.name.clone();

    projects.retain(|p| p.project_id != project_id);

    storage.save("projects.json", &projects)
        .map_err(|e| format!("保存项目列表失败: {}", e))?;

    log_operation(&app, "remove_project", &project_id, &format!("已删除项目: {}", project_name));
    Ok(())
}

#[tauri::command]
pub fn import_plugin_from_local(app: AppHandle, path: String) -> Result<Plugin, String> {
    if !std::path::Path::new(&path).exists() {
        return Err("指定的插件路径不存在".to_string());
    }

    let manager = get_plugin_manager(&app);
    let new_plugin = manager.import_from_local(&path)
        .map_err(|e| format!("导入本地插件失败: {}", e))?;

    let storage = get_storage(&app);
    let mut plugins: Vec<Plugin> = storage.load_or_default("plugins.json");

    let plugin_name = new_plugin.name.clone();
    let existing_idx = plugins.iter().position(|p| p.source.url == new_plugin.source.url);
    if let Some(idx) = existing_idx {
        plugins[idx].versions.extend(new_plugin.versions);
        let result = plugins[idx].clone();
        storage.save("plugins.json", &plugins)
            .map_err(|e| format!("保存插件列表失败: {}", e))?;
        log_operation(&app, "import_plugin", &path, &format!("已为插件 {} 添加新版本", plugin_name));
        Ok(result)
    } else {
        plugins.push(new_plugin.clone());
        storage.save("plugins.json", &plugins)
            .map_err(|e| format!("保存插件列表失败: {}", e))?;
        log_operation(&app, "import_plugin", &path, &format!("已导入插件: {}", plugin_name));
        Ok(new_plugin)
    }
}

#[tauri::command]
pub fn import_plugin_from_git(app: AppHandle, url: String) -> Result<Plugin, String> {
    if url.is_empty() {
        return Err("请输入 Git 仓库地址".to_string());
    }

    let manager = get_plugin_manager(&app);
    let new_plugin = manager.import_from_git(&url, &app)
        .map_err(|e| format!("从 Git 导入插件失败: {}，请检查仓库地址是否正确", e))?;

    let storage = get_storage(&app);
    let mut plugins: Vec<Plugin> = storage.load_or_default("plugins.json");

    let plugin_name = new_plugin.name.clone();
    let existing_idx = plugins.iter().position(|p| p.source.url == new_plugin.source.url);
    if let Some(idx) = existing_idx {
        plugins[idx].versions.extend(new_plugin.versions);
        let result = plugins[idx].clone();
        storage.save("plugins.json", &plugins)
            .map_err(|e| format!("保存插件列表失败: {}", e))?;
        log_operation(&app, "import_plugin_git", &url, &format!("已为插件 {} 添加新版本", plugin_name));
        Ok(result)
    } else {
        plugins.push(new_plugin.clone());
        storage.save("plugins.json", &plugins)
            .map_err(|e| format!("保存插件列表失败: {}", e))?;
        log_operation(&app, "import_plugin_git", &url, &format!("已从 Git 导入插件: {}", plugin_name));
        Ok(new_plugin)
    }
}

#[tauri::command]
pub fn get_plugins(app: AppHandle) -> Result<Vec<Plugin>, String> {
    let storage = get_storage(&app);
    let plugins: Vec<Plugin> = storage.load_or_default("plugins.json");
    Ok(plugins)
}

#[tauri::command]
pub fn remove_plugin(app: AppHandle, plugin_id: String) -> Result<(), String> {
    let storage = get_storage(&app);
    let mut plugins: Vec<Plugin> = storage.load_or_default("plugins.json");

    let plugin = plugins.iter().find(|p| p.plugin_id == plugin_id)
        .ok_or("未找到指定插件".to_string())?;
    let plugin_name = plugin.name.clone();

    let plugin_dir = get_data_dir(&app).join("plugins").join(&plugin_id);
    if plugin_dir.exists() {
        fs::remove_dir_all(&plugin_dir)
            .map_err(|e| format!("删除插件文件失败: {}", e))?;
    }

    let mut bindings: Vec<ProjectBinding> = storage.load_or_default("bindings.json");
    bindings.retain(|b| b.plugin_id != plugin_id);
    storage.save("bindings.json", &bindings)
        .map_err(|e| format!("保存绑定关系失败: {}", e))?;

    plugins.retain(|p| p.plugin_id != plugin_id);

    storage.save("plugins.json", &plugins)
        .map_err(|e| format!("保存插件列表失败: {}", e))?;

    log_operation(&app, "remove_plugin", &plugin_id, &format!("已删除插件: {}", plugin_name));
    Ok(())
}

#[tauri::command]
pub fn bind_plugin(
    app: AppHandle,
    project_id: String,
    plugin_id: String,
    version_id: String,
    unit_id: String,
    mount_path: String,
) -> Result<(), String> {
    if mount_path.is_empty() {
        return Err("挂载路径不能为空".to_string());
    }

    let storage = get_storage(&app);

    let projects: Vec<Project> = storage.load_or_default("projects.json");
    if !projects.iter().any(|p| p.project_id == project_id) {
        return Err("未找到指定项目".to_string());
    }

    let plugins: Vec<Plugin> = storage.load_or_default("plugins.json");
    if !plugins.iter().any(|p| p.plugin_id == plugin_id) {
        log_error(&app, "bind_plugin", &project_id, "未找到指定插件");
        return Err("未找到指定插件".to_string());
    }

    let binding = ProjectBinding::new(project_id.clone(), plugin_id, version_id, unit_id, mount_path);

    let mut bindings: Vec<ProjectBinding> = storage.load_or_default("bindings.json");

    bindings.retain(|b| !(b.project_id == binding.project_id && b.plugin_id == binding.plugin_id));
    bindings.push(binding);

    storage.save("bindings.json", &bindings)
        .map_err(|e| format!("保存绑定关系失败: {}", e))?;

    log_operation(&app, "bind_plugin", &project_id, "已绑定插件到项目");
    Ok(())
}

#[tauri::command]
pub fn unbind_plugin(app: AppHandle, project_id: String, plugin_id: String) -> Result<(), String> {
    let storage = get_storage(&app);
    let mut bindings: Vec<ProjectBinding> = storage.load_or_default("bindings.json");

    let binding = bindings.iter()
        .find(|b| b.project_id == project_id && b.plugin_id == plugin_id);

    if binding.is_none() {
        log_error(&app, "unbind_plugin", &project_id, "未找到指定的绑定关系");
        return Err("未找到指定的绑定关系".to_string());
    }

    let binding = binding.unwrap();
    let mount_path = binding.mount_path.clone();

    let projects: Vec<Project> = storage.load_or_default("projects.json");
    if let Some(project) = projects.iter().find(|p| p.project_id == project_id) {
        let addons_dir = std::path::Path::new(&project.path).join("addons");
        if addons_dir.exists() {
            let plugin_path = addons_dir.join(&mount_path);

            if plugin_path.exists() {
                let metadata = std::fs::symlink_metadata(&plugin_path);
                let is_link = metadata.as_ref().map(|m| m.file_type().is_symlink()).unwrap_or(false);
                let is_junction = if cfg!(windows) {
                    use std::os::windows::fs::MetadataExt;
                    metadata.as_ref().map(|m| m.file_attributes() & 0x400 != 0).unwrap_or(false)
                } else {
                    false
                };

                if is_link || is_junction {
                    if let Err(e) = std::fs::remove_dir(&plugin_path) {
                        eprintln!("Failed to remove symlink/junction: {}", e);
                    }
                } else if let Err(e) = std::fs::remove_dir_all(&plugin_path) {
                    eprintln!("Failed to remove plugin directory: {}", e);
                }
            }
        }
    }

    bindings.retain(|b| !(b.project_id == project_id && b.plugin_id == plugin_id));

    storage.save("bindings.json", &bindings)
        .map_err(|e| format!("保存绑定关系失败: {}", e))?;

    log_operation(&app, "unbind_plugin", &project_id, "已取消插件绑定");
    Ok(())
}

#[tauri::command]
pub fn apply_changes(app: AppHandle, project_id: String) -> Result<ApplyResult, String> {
    let storage = get_storage(&app);

    let projects: Vec<Project> = storage.load_or_default("projects.json");
    let project = projects.iter()
        .find(|p| p.project_id == project_id)
        .ok_or_else(|| {
            log_error(&app, "apply_changes", &project_id, "未找到指定项目");
            "未找到指定项目".to_string()
        })?;

    let bindings: Vec<ProjectBinding> = storage.load_or_default("bindings.json");
    let desired_bindings: Vec<ProjectBinding> = bindings.iter()
        .filter(|b| b.project_id == project_id)
        .cloned()
        .collect();

    if desired_bindings.is_empty() {
        log_error(&app, "apply_changes", &project_id, "该项目没有绑定任何插件");
        return Err("该项目没有绑定任何插件".to_string());
    }

    let settings: Settings = storage.load_or_default("settings.json");
    let linker = Linker::new(settings.mount_strategy);

    let data_dir = get_data_dir(&app);
    let plugin_base_path = data_dir.join("plugins");

    let current_bindings: Vec<ProjectBinding> = Vec::new();

    let result = linker.apply_bindings(
        &project.path,
        &current_bindings,
        &desired_bindings,
        &plugin_base_path.to_string_lossy()
    ).map_err(|e| format!("应用变更失败: {}", e))?;

    log_operation(&app, "apply_changes", &project_id,
        &format!("应用变更完成: 创建 {} 项, 移除 {} 项, 错误 {} 项",
            result.created.len(), result.removed.len(), result.errors.len()));

    Ok(result)
}

#[tauri::command]
pub fn restart_fs_watcher(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let storage = get_storage(&app);
    let settings: Settings = storage.load_or_default("settings.json");

    let dirs = if settings.scan_directories.is_empty() {
        let mut default_dirs = Vec::new();
        if cfg!(windows) {
            if let Some(userprofile) = std::env::var("USERPROFILE").ok() {
                default_dirs.push(format!("{}\\Documents", userprofile));
                default_dirs.push(format!("{}\\Desktop", userprofile));
            }
            for drive in ['D', 'E', 'F'] {
                let drive_path = format!("{}:\\", drive);
                if std::path::Path::new(&drive_path).exists() {
                    default_dirs.push(drive_path);
                }
            }
        } else {
            if let Some(home) = std::env::var("HOME").ok() {
                default_dirs.push(format!("{}/Documents", home));
                default_dirs.push(format!("{}/projects", home));
            }
        }
        default_dirs
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
pub fn get_project_bindings(app: AppHandle, project_id: String) -> Result<Vec<ProjectBinding>, String> {
    let storage = get_storage(&app);
    let bindings: Vec<ProjectBinding> = storage.load_or_default("bindings.json");

    Ok(bindings.into_iter()
        .filter(|b| b.project_id == project_id)
        .collect())
}

#[tauri::command]
pub fn scan_project_plugins(app: AppHandle) -> Result<Vec<crate::models::ScannedPlugin>, String> {
    let storage = get_storage(&app);
    let projects: Vec<Project> = storage.load_or_default("projects.json");

    if projects.is_empty() {
        return Err("暂无项目，请先添加项目".to_string());
    }

    let manager = get_plugin_manager(&app);
    let scanned_plugins = manager.scan_project_plugins(&projects)
        .map_err(|e| format!("扫描项目插件失败: {}", e))?;

    Ok(scanned_plugins)
}

#[tauri::command]
pub fn import_plugins_from_projects(app: AppHandle) -> Result<Vec<Plugin>, String> {
    let storage = get_storage(&app);
    let projects: Vec<Project> = storage.load_or_default("projects.json");

    if projects.is_empty() {
        return Err("暂无项目，请先添加项目".to_string());
    }

    let mut plugins: Vec<Plugin> = storage.load_or_default("plugins.json");

    let manager = get_plugin_manager(&app);
    let scanned_plugins = manager.scan_project_plugins(&projects)
        .map_err(|e| format!("扫描项目插件失败: {}", e))?;

    if scanned_plugins.is_empty() {
        return Err("未在项目中发现可导入的插件".to_string());
    }

    let mut imported_plugins = Vec::new();
    let mut seen_names: std::collections::HashSet<String> = plugins.iter()
        .map(|p| p.name.to_lowercase())
        .collect();

    for scanned in scanned_plugins {
        let path_str = scanned.path.clone();

        let already_imported = plugins.iter()
            .any(|p| p.source.url == path_str);

        if already_imported {
            continue;
        }

        match manager.import_from_local(&path_str) {
            Ok(plugin) => {
                let plugin_name_lower = plugin.name.to_lowercase();
                if seen_names.contains(&plugin_name_lower) {
                    if let Some(idx) = plugins.iter().position(|p| p.name.to_lowercase() == plugin_name_lower) {
                        plugins[idx].versions.extend(plugin.versions);
                        if !plugin.content_hash.is_empty() {
                            plugins[idx].content_hash = plugin.content_hash.clone();
                        }
                        let result = plugins[idx].clone();
                        imported_plugins.push(result);
                    }
                } else {
                    seen_names.insert(plugin_name_lower);
                    imported_plugins.push(plugin.clone());
                    plugins.push(plugin);
                }
            }
            Err(e) => eprintln!("Failed to import plugin from {}: {}", path_str, e),
        }
    }

    storage.save("plugins.json", &plugins)
        .map_err(|e| format!("保存插件列表失败: {}", e))?;

    log_operation(&app, "import_plugins_from_projects", "", 
        &format!("从项目导入了 {} 个插件", imported_plugins.len()));

    Ok(imported_plugins)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginStorageStats {
    pub total_size_bytes: u64,
    pub total_size_display: String,
    pub version_count: usize,
    pub binding_count: usize,
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;
    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

fn dir_size(path: &Path) -> u64 {
    walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter_map(|e| e.metadata().ok())
        .filter(|m| m.is_file())
        .map(|m| m.len())
        .sum()
}

#[tauri::command]
pub fn get_plugin_storage_stats(app: AppHandle, plugin_id: String) -> Result<PluginStorageStats, String> {
    let storage = get_storage(&app);
    let plugins: Vec<Plugin> = storage.load_or_default("plugins.json");
    let plugin = plugins.iter().find(|p| p.plugin_id == plugin_id)
        .ok_or("未找到指定插件".to_string())?;

    let plugin_dir = get_data_dir(&app).join("plugins").join(&plugin_id);
    let total_size_bytes = if plugin_dir.exists() { dir_size(&plugin_dir) } else { 0 };

    let bindings: Vec<ProjectBinding> = storage.load_or_default("bindings.json");
    let binding_count = bindings.iter().filter(|b| b.plugin_id == plugin_id).count();

    Ok(PluginStorageStats {
        total_size_bytes,
        total_size_display: format_size(total_size_bytes),
        version_count: plugin.versions.len(),
        binding_count,
    })
}

#[tauri::command]
pub fn remove_plugin_version(app: AppHandle, plugin_id: String, version_id: String) -> Result<(), String> {
    let storage = get_storage(&app);
    let mut plugins: Vec<Plugin> = storage.load_or_default("plugins.json");

    let plugin = plugins.iter_mut()
        .find(|p| p.plugin_id == plugin_id)
        .ok_or("未找到指定插件".to_string())?;

    if plugin.versions.len() <= 1 {
        return Err("插件至少需要保留一个版本，如需删除请直接删除插件".to_string());
    }

    let version_dir = get_data_dir(&app).join("plugins").join(&plugin_id).join(&version_id);
    if version_dir.exists() {
        fs::remove_dir_all(&version_dir)
            .map_err(|e| format!("删除版本文件失败: {}", e))?;
    }

    let plugin_name = plugin.name.clone();
    plugin.versions.retain(|v| v.version_id != version_id);
    plugin.updated_at = chrono::Utc::now();

    storage.save("plugins.json", &plugins)
        .map_err(|e| format!("保存插件列表失败: {}", e))?;

    log_operation(&app, "remove_plugin_version", &plugin_id, 
        &format!("已删除插件 {} 的版本 {}", plugin_name, version_id));

    Ok(())
}

#[tauri::command]
pub fn get_plugin_bindings(app: AppHandle, plugin_id: String) -> Result<Vec<ProjectBinding>, String> {
    let storage = get_storage(&app);
    let bindings: Vec<ProjectBinding> = storage.load_or_default("bindings.json");
    Ok(bindings.into_iter().filter(|b| b.plugin_id == plugin_id).collect())
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
pub fn toggle_plugin_favorite(app: AppHandle, plugin_id: String) -> Result<bool, String> {
    let storage = get_storage(&app);
    let mut plugins: Vec<Plugin> = storage.load_or_default("plugins.json");

    let plugin = plugins.iter_mut()
        .find(|p| p.plugin_id == plugin_id)
        .ok_or("未找到指定插件".to_string())?;

    plugin.is_favorite = !plugin.is_favorite;
    let new_state = plugin.is_favorite;

    storage.save("plugins.json", &plugins)
        .map_err(|e| format!("保存插件状态失败: {}", e))?;

    log_operation(&app, "toggle_favorite", &plugin_id,
        &format!("插件收藏状态: {}", if new_state { "已收藏" } else { "已取消" }));
    Ok(new_state)
}

#[tauri::command]
pub fn update_project_group(app: AppHandle, project_id: String, group: String) -> Result<(), String> {
    let storage = get_storage(&app);
    let mut projects: Vec<Project> = storage.load_or_default("projects.json");

    let project = projects.iter_mut()
        .find(|p| p.project_id == project_id)
        .ok_or("未找到指定项目".to_string())?;

    project.group = group.clone();

    storage.save("projects.json", &projects)
        .map_err(|e| format!("保存项目分组失败: {}", e))?;

    log_operation(&app, "update_project_group", &project_id, &format!("项目分组已更新: {}", group));
    Ok(())
}

#[tauri::command]
pub fn get_project_groups(app: AppHandle) -> Result<Vec<String>, String> {
    let storage = get_storage(&app);
    let projects: Vec<Project> = storage.load_or_default("projects.json");

    let mut groups: Vec<String> = projects.iter()
        .filter(|p| !p.group.is_empty())
        .map(|p| p.group.clone())
        .collect();
    groups.sort();
    groups.dedup();

    Ok(groups)
}

#[tauri::command]
pub fn backup_data(app: AppHandle, backup_path: String) -> Result<String, String> {
    let data_dir = get_data_dir(&app);
    let backup_dir = std::path::Path::new(&backup_path);

    std::fs::create_dir_all(backup_dir)
        .map_err(|e| format!("创建备份目录失败: {}", e))?;

    let files = ["settings.json", "projects.json", "plugins.json", "bindings.json", "engines.json", "engine_bindings.json", "team_configs.json"];
    let mut backup_info = Vec::new();

    for filename in &files {
        let src = data_dir.join(filename);
        if src.exists() {
            let dst = backup_dir.join(filename);
            std::fs::copy(&src, &dst)
                .map_err(|e| format!("备份文件 {} 失败: {}", filename, e))?;
            backup_info.push(filename.to_string());
        }
    }

    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let backup_file = backup_dir.join(format!("backup_{}.json", timestamp));

    let backup_data = serde_json::json!({
        "version": "1.0",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "files": backup_info
    });

    std::fs::write(&backup_file, serde_json::to_string_pretty(&backup_data).unwrap())
        .map_err(|e| format!("创建备份信息文件失败: {}", e))?;

    log_operation(&app, "backup_data", &backup_path, &format!("数据备份成功，共备份 {} 个文件", backup_info.len()));
    Ok(format!("备份成功，共备份 {} 个文件", backup_info.len()))
}

#[tauri::command]
pub fn restore_data(app: AppHandle, backup_path: String) -> Result<String, String> {
    let data_dir = get_data_dir(&app);
    let backup_dir = std::path::Path::new(&backup_path);

    if !backup_dir.exists() {
        return Err("备份目录不存在".to_string());
    }

    let files = ["settings.json", "projects.json", "plugins.json", "bindings.json", "engines.json", "engine_bindings.json", "team_configs.json"];
    let mut restore_info = Vec::new();

    for filename in &files {
        let src = backup_dir.join(filename);
        if src.exists() {
            let dst = data_dir.join(filename);
            std::fs::copy(&src, &dst)
                .map_err(|e| format!("恢复文件 {} 失败: {}", filename, e))?;
            restore_info.push(filename.to_string());
        }
    }

    log_operation(&app, "restore_data", &backup_path, &format!("数据恢复成功，共恢复 {} 个文件", restore_info.len()));
    Ok(format!("恢复成功，共恢复 {} 个文件", restore_info.len()))
}

#[tauri::command]
pub fn register_engine(app: AppHandle, path: String, name: String) -> Result<Engine, String> {
    if path.is_empty() {
        return Err("引擎路径不能为空".to_string());
    }

    if !crate::engine::EngineManager::validate_engine_path(&path) {
        log_error(&app, "register_engine", &path, "引擎路径无效或找不到可执行文件");
        return Err("引擎路径无效或找不到 Godot 可执行文件".to_string());
    }

    let engine = crate::engine::EngineManager::get_engine_info(&path)
        .map_err(|e| format!("获取引擎信息失败: {}", e))?;

    let mut registered_engine = engine;
    registered_engine.name = if name.is_empty() { registered_engine.name.clone() } else { name };

    let storage = get_storage(&app);
    let mut engines: Vec<Engine> = storage.load_or_default("engines.json");

    if engines.iter().any(|e| e.path == registered_engine.path) {
        return Err("该引擎已被注册".to_string());
    }

    if engines.is_empty() {
        registered_engine.is_default = true;
    }

    engines.push(registered_engine.clone());
    storage.save("engines.json", &engines)
        .map_err(|e| format!("保存引擎信息失败: {}", e))?;

    log_operation(&app, "register_engine", &path, &format!("已注册引擎: {}", registered_engine.name));
    Ok(registered_engine)
}

#[tauri::command]
pub fn get_engines(app: AppHandle) -> Result<Vec<Engine>, String> {
    let storage = get_storage(&app);
    let engines: Vec<Engine> = storage.load_or_default("engines.json");
    Ok(engines)
}

#[tauri::command]
pub fn remove_engine(app: AppHandle, engine_id: String) -> Result<(), String> {
    let storage = get_storage(&app);
    let mut engines: Vec<Engine> = storage.load_or_default("engines.json");

    let engine = engines.iter().find(|e| e.engine_id == engine_id)
        .ok_or("未找到指定引擎".to_string())?;
    let engine_name = engine.name.clone();
    let was_default = engine.is_default;

    engines.retain(|e| e.engine_id != engine_id);

    if was_default && !engines.is_empty() {
        engines[0].is_default = true;
    }

    storage.save("engines.json", &engines)
        .map_err(|e| format!("保存引擎列表失败: {}", e))?;

    log_operation(&app, "remove_engine", &engine_id, &format!("已删除引擎: {}", engine_name));
    Ok(())
}

#[tauri::command]
pub fn set_default_engine(app: AppHandle, engine_id: String) -> Result<(), String> {
    let storage = get_storage(&app);
    let mut engines: Vec<Engine> = storage.load_or_default("engines.json");

    let engine_name = {
        let engine = engines.iter()
            .find(|e| e.engine_id == engine_id)
            .ok_or("未找到指定引擎".to_string())?;
        engine.name.clone()
    };

    for e in engines.iter_mut() {
        e.is_default = e.engine_id == engine_id;
    }

    storage.save("engines.json", &engines)
        .map_err(|e| format!("保存引擎设置失败: {}", e))?;

    log_operation(&app, "set_default_engine", &engine_id, &format!("已将 {} 设为默认引擎", engine_name));
    Ok(())
}

#[tauri::command]
pub fn bind_project_engine(
    app: AppHandle,
    project_id: String,
    engine_id: String,
    custom_args: String,
) -> Result<(), String> {
    let storage = get_storage(&app);

    let projects: Vec<Project> = storage.load_or_default("projects.json");
    if !projects.iter().any(|p| p.project_id == project_id) {
        return Err("未找到指定项目".to_string());
    }

    let engines: Vec<Engine> = storage.load_or_default("engines.json");
    if !engines.iter().any(|e| e.engine_id == engine_id) {
        return Err("未找到指定引擎".to_string());
    }

    let binding = ProjectEngineBinding::new(project_id.clone(), engine_id, custom_args);

    let mut engine_bindings: Vec<ProjectEngineBinding> = storage.load_or_default("engine_bindings.json");
    engine_bindings.retain(|b| b.project_id != binding.project_id);
    engine_bindings.push(binding);

    storage.save("engine_bindings.json", &engine_bindings)
        .map_err(|e| format!("保存引擎绑定失败: {}", e))?;

    log_operation(&app, "bind_project_engine", &project_id, "已绑定项目到引擎");
    Ok(())
}

#[tauri::command]
pub fn unbind_project_engine(app: AppHandle, project_id: String) -> Result<(), String> {
    let storage = get_storage(&app);
    let mut engine_bindings: Vec<ProjectEngineBinding> = storage.load_or_default("engine_bindings.json");

    engine_bindings.retain(|b| b.project_id != project_id);

    storage.save("engine_bindings.json", &engine_bindings)
        .map_err(|e| format!("保存引擎绑定失败: {}", e))?;

    log_operation(&app, "unbind_project_engine", &project_id, "已解除项目引擎绑定");
    Ok(())
}

#[tauri::command]
pub fn get_project_engine_binding(app: AppHandle, project_id: String) -> Result<Option<ProjectEngineBinding>, String> {
    let storage = get_storage(&app);
    let engine_bindings: Vec<ProjectEngineBinding> = storage.load_or_default("engine_bindings.json");

    Ok(engine_bindings.into_iter().find(|b| b.project_id == project_id))
}

#[tauri::command]
pub fn launch_project_with_engine(
    app: AppHandle,
    project_id: String,
    engine_id: Option<String>,
    custom_args: Option<String>,
) -> Result<LaunchResult, String> {
    let storage = get_storage(&app);

    let projects: Vec<Project> = storage.load_or_default("projects.json");
    let project = projects.iter()
        .find(|p| p.project_id == project_id)
        .ok_or("未找到指定项目".to_string())?;

    let project_path = std::path::Path::new(&project.path);
    if !project_path.exists() || !project_path.join("project.godot").exists() {
        log_error(&app, "launch_project", &project_id, "项目路径不存在");
        return Ok(LaunchResult {
            success: false,
            pid: None,
            error: Some("项目路径不存在或不是有效的 Godot 项目".to_string()),
        });
    }

    let engines: Vec<Engine> = storage.load_or_default("engines.json");

    let engine = if let Some(eid) = engine_id {
        engines.iter().find(|e| e.engine_id == eid).cloned()
    } else {
        let engine_bindings: Vec<ProjectEngineBinding> = storage.load_or_default("engine_bindings.json");
        let binding = engine_bindings.iter().find(|b| b.project_id == project_id);
        if let Some(b) = binding {
            engines.iter().find(|e| e.engine_id == b.engine_id).cloned()
        } else {
            engines.iter().find(|e| e.is_default).cloned()
        }
    };

    let engine = engine.ok_or_else(|| {
        log_error(&app, "launch_project", &project_id, "未找到可用的引擎");
        "未找到可用的引擎，请先注册引擎".to_string()
    })?;

    let exe_name = if cfg!(windows) { "godot.exe" } else { "godot" };
    let exe_path = std::path::Path::new(&engine.path).join(exe_name);
    let actual_exe = if exe_path.exists() {
        exe_path
    } else {
        std::path::Path::new(&engine.path).join(format!("bin/{}", exe_name))
    };

    if !actual_exe.exists() {
        log_error(&app, "launch_project", &project_id, "引擎可执行文件不存在");
        return Ok(LaunchResult {
            success: false,
            pid: None,
            error: Some("引擎可执行文件不存在".to_string()),
        });
    }

    let mut cmd = std::process::Command::new(&actual_exe);
    cmd.current_dir(&project.path);

    if let Some(args) = custom_args {
        if !args.is_empty() {
            cmd.arg("--").args(args.split_whitespace());
        }
    } else {
        let engine_bindings: Vec<ProjectEngineBinding> = storage.load_or_default("engine_bindings.json");
        if let Some(binding) = engine_bindings.iter().find(|b| b.project_id == project_id) {
            if !binding.custom_args.is_empty() {
                cmd.arg("--").args(binding.custom_args.split_whitespace());
            }
        }
    }
    cmd.arg("--path").arg(&project.path);

    match cmd.spawn() {
        Ok(child) => {
            log_operation(&app, "launch_project", &project_id,
                &format!("使用 {} 启动项目成功，PID: {}", engine.name, child.id()));
            Ok(LaunchResult {
                success: true,
                pid: Some(child.id()),
                error: None,
            })
        }
        Err(e) => {
            log_error(&app, "launch_project", &project_id, &e.to_string());
            Ok(LaunchResult {
                success: false,
                pid: None,
                error: Some(format!("启动失败: {}", e)),
            })
        }
    }
}

#[tauri::command]
pub fn check_plugin_updates(app: AppHandle) -> Result<Vec<PluginUpdateInfo>, String> {
    let storage = get_storage(&app);
    let plugins: Vec<Plugin> = storage.load_or_default("plugins.json");

    let mut update_infos = Vec::new();

    for plugin in &plugins {
        let current_version = plugin.versions.last()
            .map(|v| v.version.clone())
            .unwrap_or_else(|| "0.0.0".to_string());

        let mut latest_version = current_version.clone();
        let mut release_notes = String::new();

        if plugin.source.source_type == SourceType::Git && !plugin.source.url.is_empty() {
            let url = &plugin.source.url;
            if url.contains("github.com") {
                let api_url = url.trim_end_matches(".git")
                    .replace("git@github.com:", "https://api.github.com/repos/")
                    .replace("https://github.com/", "https://api.github.com/repos/");
                let releases_url = format!("{}/releases/latest", api_url);

                if let Ok(client) = reqwest::blocking::ClientBuilder::new()
                    .user_agent("GodotHarbor")
                    .build() {
                    if let Ok(resp) = client.get(&releases_url).send() {
                        if resp.status().is_success() {
                            if let Ok(json) = resp.json::<serde_json::Value>() {
                                if let Some(tag) = json.get("tag_name").and_then(|t| t.as_str()) {
                                    let tag = tag.trim_start_matches('v');
                                    if compare_versions(&current_version, tag) < 0 {
                                        latest_version = tag.to_string();
                                    }
                                }
                                if let Some(notes) = json.get("body").and_then(|b| b.as_str()) {
                                    release_notes = notes.chars().take(500).collect();
                                }
                            }
                        }
                    }
                }
            }
        }

        let update_available = compare_versions(&current_version, &latest_version) < 0;

        update_infos.push(PluginUpdateInfo {
            plugin_id: plugin.plugin_id.clone(),
            current_version,
            latest_version,
            update_available,
            release_notes,
        });
    }

    log_operation(&app, "check_plugin_updates", "", &format!("检查了 {} 个插件的更新", update_infos.len()));
    Ok(update_infos)
}

fn compare_versions(current: &str, latest: &str) -> i32 {
    let c_parts: Vec<u32> = current.split('.').filter_map(|s| s.parse().ok()).collect();
    let l_parts: Vec<u32> = latest.split('.').filter_map(|s| s.parse().ok()).collect();

    let max_len = c_parts.len().max(l_parts.len());

    for i in 0..max_len {
        let c = c_parts.get(i).unwrap_or(&0);
        let l = l_parts.get(i).unwrap_or(&0);
        if c < l {
            return -1;
        } else if c > l {
            return 1;
        }
    }
    0
}

#[tauri::command]
pub fn export_team_config(app: AppHandle, name: String, description: String, project_ids: Vec<String>) -> Result<TeamSharedConfig, String> {
    let storage = get_storage(&app);

    let bindings: Vec<ProjectBinding> = storage.load_or_default("bindings.json");
    let engine_bindings: Vec<ProjectEngineBinding> = storage.load_or_default("engine_bindings.json");

    let selected_bindings: Vec<ProjectBinding> = bindings.into_iter()
        .filter(|b| project_ids.contains(&b.project_id))
        .collect();

    let selected_engine_bindings: Vec<ProjectEngineBinding> = engine_bindings.into_iter()
        .filter(|b| project_ids.contains(&b.project_id))
        .collect();

    let mut config = TeamSharedConfig::new(name, description);
    config.bindings = selected_bindings;
    config.engine_bindings = selected_engine_bindings;

    let mut configs: Vec<TeamSharedConfig> = storage.load_or_default("team_configs.json");
    configs.push(config.clone());

    storage.save("team_configs.json", &configs)
        .map_err(|e| format!("保存团队配置失败: {}", e))?;

    log_operation(&app, "export_team_config", &config.config_id, &format!("已导出团队配置: {}", config.name));
    Ok(config)
}

#[tauri::command]
pub fn get_team_configs(app: AppHandle) -> Result<Vec<TeamSharedConfig>, String> {
    let storage = get_storage(&app);
    let configs: Vec<TeamSharedConfig> = storage.load_or_default("team_configs.json");
    Ok(configs)
}

#[tauri::command]
pub fn import_team_config(app: AppHandle, config_id: String, target_project_ids: Vec<String>) -> Result<(), String> {
    let storage = get_storage(&app);
    let configs: Vec<TeamSharedConfig> = storage.load_or_default("team_configs.json");

    let config = configs.iter()
        .find(|c| c.config_id == config_id)
        .ok_or("未找到指定的团队配置".to_string())?;

    let mut bindings: Vec<ProjectBinding> = storage.load_or_default("bindings.json");
    let mut engine_bindings: Vec<ProjectEngineBinding> = storage.load_or_default("engine_bindings.json");

    let mut imported_count = 0;

    for binding in &config.bindings {
        if target_project_ids.contains(&binding.project_id) {
            bindings.retain(|b| !(b.project_id == binding.project_id && b.plugin_id == binding.plugin_id));
            let mut new_binding = binding.clone();
            new_binding.created_at = chrono::Utc::now();
            bindings.push(new_binding);
            imported_count += 1;
        }
    }

    for engine_binding in &config.engine_bindings {
        if target_project_ids.contains(&engine_binding.project_id) {
            engine_bindings.retain(|b| b.project_id != engine_binding.project_id);
            let mut new_binding = engine_binding.clone();
            new_binding.created_at = chrono::Utc::now();
            engine_bindings.push(new_binding);
        }
    }

    storage.save("bindings.json", &bindings)
        .map_err(|e| format!("保存绑定关系失败: {}", e))?;
    storage.save("engine_bindings.json", &engine_bindings)
        .map_err(|e| format!("保存引擎绑定失败: {}", e))?;

    log_operation(&app, "import_team_config", &config_id, &format!("导入了 {} 个绑定到目标项目", imported_count));
    Ok(())
}

#[tauri::command]
pub fn delete_team_config(app: AppHandle, config_id: String) -> Result<(), String> {
    let storage = get_storage(&app);
    let mut configs: Vec<TeamSharedConfig> = storage.load_or_default("team_configs.json");

    let config = configs.iter().find(|c| c.config_id == config_id)
        .ok_or("未找到指定的团队配置".to_string())?;
    let config_name = config.name.clone();

    configs.retain(|c| c.config_id != config_id);

    storage.save("team_configs.json", &configs)
        .map_err(|e| format!("删除团队配置失败: {}", e))?;

    log_operation(&app, "delete_team_config", &config_id, &format!("已删除团队配置: {}", config_name));
    Ok(())
}

#[tauri::command]
pub fn resolve_plugin_dependencies(app: AppHandle, plugin_id: String) -> Result<Vec<PluginDependency>, String> {
    let storage = get_storage(&app);
    let plugins: Vec<Plugin> = storage.load_or_default("plugins.json");

    let plugin = plugins.iter()
        .find(|p| p.plugin_id == plugin_id)
        .ok_or("未找到指定插件".to_string())?;

    let mut dependencies = Vec::new();

    if let Some(version) = plugin.versions.last() {
        for unit in &version.units {
            let cfg_path = std::path::Path::new(&unit.plugin_cfg_path);
            if cfg_path.exists() {
                if let Ok(content) = std::fs::read_to_string(cfg_path) {
                    for line in content.lines() {
                        let line = line.trim();
                        if line.starts_with("depends=") {
                            let deps_str = line[8..].trim_matches('"');
                            for dep_entry in deps_str.split(',') {
                                let dep_entry = dep_entry.trim();
                                if dep_entry.is_empty() { continue; }
                                let parts: Vec<&str> = dep_entry.splitn(2, ':').collect();
                                let dep_name = parts[0].trim();
                                let version_constraint = parts.get(1).map(|s| s.trim()).unwrap_or("*").to_string();

                                let matched_plugin = plugins.iter().find(|p| {
                                    p.name.to_lowercase() == dep_name.to_lowercase()
                                        || p.versions.iter().any(|v|
                                            v.units.iter().any(|u| u.name.to_lowercase() == dep_name.to_lowercase()))
                                });

                                dependencies.push(PluginDependency {
                                    plugin_id: matched_plugin.map(|p| p.plugin_id.clone()).unwrap_or_default(),
                                    version_constraint,
                                    is_optional: false,
                                });
                            }
                        }
                    }
                }
            }

            if let Some(parent) = cfg_path.parent() {
                let dep_file = parent.join(".dependencies");
                if dep_file.exists() {
                    if let Ok(content) = std::fs::read_to_string(&dep_file) {
                        for line in content.lines() {
                            let line = line.trim();
                            if line.is_empty() || line.starts_with('#') { continue; }
                            let parts: Vec<&str> = line.splitn(2, '=').collect();
                            if parts.len() == 2 {
                                let dep_name = parts[0].trim();
                                let version_constraint = parts[1].trim().to_string();
                                let matched_plugin = plugins.iter().find(|p| {
                                    p.name.to_lowercase() == dep_name.to_lowercase()
                                });
                                dependencies.push(PluginDependency {
                                    plugin_id: matched_plugin.map(|p| p.plugin_id.clone()).unwrap_or_default(),
                                    version_constraint,
                                    is_optional: false,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    let dep_str = format!("插件 {} 的依赖解析完成，共发现 {} 个依赖项", plugin.name, dependencies.len());
    log_operation(&app, "resolve_dependencies", &plugin_id, &dep_str);

    Ok(dependencies)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetLibrarySearchParams {
    pub filter: Option<String>,
    pub asset_type: Option<String>,
    pub category: Option<String>,
    pub support: Option<String>,
    pub cost: Option<String>,
    pub godot_version: Option<String>,
    pub max_results: Option<u32>,
    pub page: Option<u32>,
    pub sort: Option<String>,
    pub reverse: Option<bool>,
}

#[tauri::command]
pub fn search_asset_library(app: AppHandle, params: AssetLibrarySearchParams) -> Result<serde_json::Value, String> {
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
        url_params.push(format!("page={}", p));
    }

    url_params.push(format!("sort={}", params.sort.as_deref().unwrap_or("updated")));

    if params.reverse.unwrap_or(false) {
        url_params.push("reverse".to_string());
    }

    let url = format!("https://godotengine.org/asset-library/api/asset?{}", url_params.join("&"));

    let client = reqwest::blocking::ClientBuilder::new()
        .user_agent("GodotHarbor")
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let resp = client.get(&url).send()
        .map_err(|e| format!("请求 Asset Library 失败: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Asset Library 返回错误状态: {}", resp.status()));
    }

    let text = resp.text()
        .map_err(|e| format!("读取 Asset Library 响应失败: {}", e))?;

    let json: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| format!("解析 Asset Library 响应失败: {} (响应前100字符: {})", e, &text[..text.len().min(100)]))?;

    let filter_str = params.filter.as_deref().unwrap_or("");
    log_operation(&app, "search_asset_library", "", &format!("搜索 Asset Library: {}", filter_str));
    Ok(json)
}

#[tauri::command]
pub fn get_asset_library_configure(app: AppHandle) -> Result<serde_json::Value, String> {
    let url = "https://godotengine.org/asset-library/api/configure?type=any";

    let client = reqwest::blocking::ClientBuilder::new()
        .user_agent("GodotHarbor")
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let resp = client.get(url).send()
        .map_err(|e| format!("请求 Asset Library 配置失败: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Asset Library 返回错误状态: {}", resp.status()));
    }

    let json: serde_json::Value = resp.json()
        .map_err(|e| format!("解析 Asset Library 配置失败: {}", e))?;

    log_operation(&app, "get_asset_library_configure", "", "获取 Asset Library 配置");
    Ok(json)
}

#[tauri::command]
pub fn get_asset_detail(app: AppHandle, asset_id: String) -> Result<serde_json::Value, String> {
    let url = format!(
        "https://godotengine.org/asset-library/api/asset/{}",
        asset_id
    );

    let client = reqwest::blocking::ClientBuilder::new()
        .user_agent("GodotHarbor")
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let resp = client.get(&url).send()
        .map_err(|e| format!("请求 Asset Library 失败: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Asset Library 返回错误状态: {}", resp.status()));
    }

    let json: serde_json::Value = resp.json()
        .map_err(|e| format!("解析 Asset Library 响应失败: {}", e))?;

    log_operation(&app, "get_asset_detail", &asset_id, &format!("获取资产详情: {}", asset_id));
    Ok(json)
}

#[tauri::command]
pub fn import_from_asset_library(app: AppHandle, asset_id: String) -> Result<Plugin, String> {
    let url = format!(
        "https://godotengine.org/asset-library/api/asset/{}",
        asset_id
    );

    let client = reqwest::blocking::ClientBuilder::new()
        .user_agent("GodotHarbor")
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let resp = client.get(&url).send()
        .map_err(|e| format!("请求 Asset Library 失败: {}", e))?;

    let asset: serde_json::Value = resp.json()
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

    let plugin_source = PluginSource {
        source_type: SourceType::AssetLibrary,
        url: format!("asset-library://{}", asset_id),
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
    let mut resp = client.get(download_url).send()
        .map_err(|e| format!("下载资源失败: {}", e))?;

    let mut file = std::fs::File::create(&temp_zip)
        .map_err(|e| format!("创建临时文件失败: {}", e))?;

    resp.copy_to(&mut file)
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
    let units = match manager.parse_plugin_units(&payload_dir) {
        Ok(u) => u,
        Err(e) => {
            let _ = std::fs::remove_dir_all(&version_dir);
            return Err(format!("解析插件失败: {}，已清理下载文件", e));
        }
    };

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

    let storage = get_storage(&app);
    let mut plugins: Vec<Plugin> = storage.load_or_default("plugins.json");

    let existing_idx = plugins.iter().position(|p| p.source.url == plugin.source.url);
    if let Some(idx) = existing_idx {
        plugins[idx].versions.extend(plugin.versions);
        let result = plugins[idx].clone();
        storage.save("plugins.json", &plugins)
            .map_err(|e| format!("保存插件列表失败: {}", e))?;
        log_operation(&app, "import_asset_library", &asset_id.to_string(), &format!("已为插件 {} 添加新版本", result.name));
        Ok(result)
    } else {
        plugins.push(plugin.clone());
        storage.save("plugins.json", &plugins)
            .map_err(|e| format!("保存插件列表失败: {}", e))?;
        log_operation(&app, "import_asset_library", &asset_id.to_string(), &format!("已从 Asset Library 导入插件: {}", plugin.name));
        Ok(plugin)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetImportProgressPayload {
    pub asset_id: String,
    pub stage: String,
    pub progress: f64,
    pub message: String,
}

#[tauri::command]
pub fn import_from_asset_library_with_progress(app: AppHandle, asset_id: String) -> Result<Plugin, String> {
    let _ = app.emit("asset-import-progress", AssetImportProgressPayload {
        asset_id: asset_id.clone(),
        stage: "downloading".to_string(),
        progress: 0.0,
        message: "正在获取资产信息...".to_string(),
    });

    let url = format!(
        "https://godotengine.org/asset-library/api/asset/{}",
        asset_id
    );

    let client = reqwest::blocking::ClientBuilder::new()
        .user_agent("GodotHarbor")
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let resp = client.get(&url).send()
        .map_err(|e| format!("请求 Asset Library 失败: {}", e))?;

    let asset: serde_json::Value = resp.json()
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

    let plugin_source = PluginSource {
        source_type: SourceType::AssetLibrary,
        url: format!("asset-library://{}", asset_id),
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
    let mut resp = client.get(download_url).send()
        .map_err(|e| format!("下载资源失败: {}", e))?;

    let total_size = resp.content_length().unwrap_or(0);
    let mut file = std::fs::File::create(&temp_zip)
        .map_err(|e| format!("创建临时文件失败: {}", e))?;

    let mut downloaded: u64 = 0;
    let mut buffer = [0u8; 8192];
    loop {
        let bytes_read = resp.read(&mut buffer)
            .map_err(|e| format!("读取下载数据失败: {}", e))?;
        if bytes_read == 0 { break; }
        file.write_all(&buffer[..bytes_read])
            .map_err(|e| format!("写入文件失败: {}", e))?;
        downloaded += bytes_read as u64;
        if total_size > 0 {
            let progress = 0.1 + 0.6 * (downloaded as f64 / total_size as f64);
            let _ = app.emit("asset-import-progress", AssetImportProgressPayload {
                asset_id: asset_id.clone(),
                stage: "downloading".to_string(),
                progress,
                message: format!("正在下载 {} ({:.0}%)...", asset_name, progress * 100.0),
            });
        }
    }

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
        progress: 0.9,
        message: format!("正在解析插件 {}...", asset_name),
    });

    let manager = get_plugin_manager(&app);
    let units = match manager.parse_plugin_units(&payload_dir) {
        Ok(u) => u,
        Err(e) => {
            let _ = std::fs::remove_dir_all(&version_dir);
            let _ = app.emit("asset-import-progress", AssetImportProgressPayload {
                asset_id: asset_id.clone(),
                stage: "error".to_string(),
                progress: 0.0,
                message: format!("解析插件失败: {}", e),
            });
            return Err(format!("解析插件失败: {}，已清理下载文件", e));
        }
    };

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

    let storage = get_storage(&app);
    let mut plugins: Vec<Plugin> = storage.load_or_default("plugins.json");

    let existing_idx = plugins.iter().position(|p| p.source.url == plugin.source.url);
    let result = if let Some(idx) = existing_idx {
        plugins[idx].versions.extend(plugin.versions);
        let result = plugins[idx].clone();
        storage.save("plugins.json", &plugins)
            .map_err(|e| format!("保存插件列表失败: {}", e))?;
        log_operation(&app, "import_asset_library", &asset_id.to_string(), &format!("已为插件 {} 添加新版本", result.name));
        result
    } else {
        plugins.push(plugin.clone());
        storage.save("plugins.json", &plugins)
            .map_err(|e| format!("保存插件列表失败: {}", e))?;
        log_operation(&app, "import_asset_library", &asset_id.to_string(), &format!("已从 Asset Library 导入插件: {}", plugin.name));
        plugin
    };

    let _ = app.emit("asset-import-progress", AssetImportProgressPayload {
        asset_id: asset_id.clone(),
        stage: "complete".to_string(),
        progress: 1.0,
        message: format!("{} 导入完成", result.name),
    });

    Ok(result)
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
    recent_projects.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    recent_projects.truncate(5);

    Ok(DashboardStats {
        project_count: projects.len(),
        plugin_count: plugins.len(),
        binding_count: total_bindings,
        engine_count: engines.len(),
        recent_projects,
    })
}

#[tauri::command]
pub async fn auto_scan_projects(app: AppHandle) -> Result<Vec<Project>, String> {
    let settings: Settings = {
        let storage = get_storage(&app);
        storage.load_or_default("settings.json")
    };

    if !settings.auto_scan_on_startup {
        return Ok(Vec::new());
    }

    let scan_dirs = if settings.scan_directories.is_empty() {
        let mut default_dirs = Vec::new();

        if cfg!(windows) {
            if let Some(userprofile) = std::env::var("USERPROFILE").ok() {
                default_dirs.push(format!("{}\\Documents", userprofile));
                default_dirs.push(format!("{}\\Desktop", userprofile));
            }
            for drive in ['D', 'E', 'F'] {
                let drive_path = format!("{}:\\", drive);
                if std::path::Path::new(&drive_path).exists() {
                    default_dirs.push(drive_path);
                }
            }
        } else {
            if let Some(home) = std::env::var("HOME").ok() {
                default_dirs.push(format!("{}/Documents", home));
                default_dirs.push(format!("{}/projects", home));
                default_dirs.push(format!("{}/Documents/godot", home));
            }
        }

        default_dirs
    } else {
        settings.scan_directories
    };

    let mut all_new_projects = Vec::new();
    let storage = get_storage(&app);
    let mut existing_projects: Vec<Project> = storage.load_or_default("projects.json");
    let existing_paths: Vec<String> = existing_projects.iter().map(|p| p.path.clone()).collect();

    for dir in &scan_dirs {
        if !std::path::Path::new(dir).exists() {
            continue;
        }

        match ProjectScanner::scan_directory(dir) {
            Ok(scanned) => {
                for project in scanned {
                    if !existing_paths.contains(&project.path) {
                        existing_projects.push(project.clone());
                        all_new_projects.push(project);
                    }
                }
            }
            Err(_) => continue,
        }
    }

    if !all_new_projects.is_empty() {
        storage.save("projects.json", &existing_projects)
            .map_err(|e| format!("保存项目失败: {}", e))?;

        log_operation(&app, "auto_scan", "", &format!("自动扫描发现 {} 个新项目", all_new_projects.len()));
    }

    let _ = app.emit("scan-complete", &all_new_projects);

    Ok(all_new_projects)
}

#[tauri::command]
pub fn relocate_project(app: AppHandle, project_id: String, new_path: String) -> Result<Project, String> {
    let new_project_path = std::path::Path::new(&new_path);
    if !new_project_path.exists() {
        return Err("指定的新路径不存在".to_string());
    }
    if !new_project_path.join("project.godot").exists() {
        return Err("指定路径不是有效的 Godot 项目".to_string());
    }

    let storage = get_storage(&app);
    let mut projects: Vec<Project> = storage.load_or_default("projects.json");

    let project = projects.iter_mut()
        .find(|p| p.project_id == project_id)
        .ok_or("未找到指定项目".to_string())?;

    let old_path = project.path.clone();
    project.path = new_path.clone();
    project.status = ProjectStatus::Ready;

    let updated_project = project.clone();

    storage.save("projects.json", &projects)
        .map_err(|e| format!("保存项目失败: {}", e))?;

    log_operation(&app, "relocate_project", &project_id,
        &format!("项目路径已从 {} 更新为 {}", old_path, new_path));

    Ok(updated_project)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovedProjectCandidate {
    pub project_id: String,
    pub old_path: String,
    pub old_name: String,
    pub new_path: String,
    pub new_name: String,
}

#[tauri::command]
pub fn detect_moved_projects(app: AppHandle) -> Result<Vec<MovedProjectCandidate>, String> {
    let storage = get_storage(&app);
    let projects: Vec<Project> = storage.load_or_default("projects.json");

    let missing_projects: Vec<&Project> = projects.iter()
        .filter(|p| !std::path::Path::new(&p.path).exists())
        .collect();

    if missing_projects.is_empty() {
        return Ok(Vec::new());
    }

    let settings: Settings = storage.load_or_default("settings.json");
    let scan_dirs = if settings.scan_directories.is_empty() {
        let mut default_dirs = Vec::new();
        if cfg!(windows) {
            if let Some(userprofile) = std::env::var("USERPROFILE").ok() {
                default_dirs.push(format!("{}\\Documents", userprofile));
                default_dirs.push(format!("{}\\Desktop", userprofile));
            }
        } else {
            if let Some(home) = std::env::var("HOME").ok() {
                default_dirs.push(format!("{}/Documents", home));
                default_dirs.push(format!("{}/projects", home));
            }
        }
        default_dirs
    } else {
        settings.scan_directories
    };

    let mut all_scanned = Vec::new();
    for dir in &scan_dirs {
        if std::path::Path::new(dir).exists() {
            if let Ok(scanned) = ProjectScanner::scan_directory(dir) {
                all_scanned.extend(scanned);
            }
        }
    }

    let existing_paths: Vec<String> = projects.iter().map(|p| p.path.clone()).collect();
    let new_projects: Vec<&Project> = all_scanned.iter()
        .filter(|p| !existing_paths.contains(&p.path))
        .collect();

    let mut candidates = Vec::new();

    for missing in &missing_projects {
        for new_proj in &new_projects {
            if missing.name == new_proj.name {
                candidates.push(MovedProjectCandidate {
                    project_id: missing.project_id.clone(),
                    old_path: missing.path.clone(),
                    old_name: missing.name.clone(),
                    new_path: new_proj.path.clone(),
                    new_name: new_proj.name.clone(),
                });
            }
        }
    }

    Ok(candidates)
}

#[tauri::command]
pub fn confirm_project_relocation(app: AppHandle, project_id: String, new_path: String) -> Result<Project, String> {
    relocate_project(app, project_id, new_path)
}

#[tauri::command]
pub fn sync_projects(app: AppHandle) -> Result<Vec<Project>, String> {
    let storage = get_storage(&app);
    let mut projects: Vec<Project> = storage.load_or_default("projects.json");
    let now = chrono::Utc::now();
    let mut synced_count = 0;

    for project in projects.iter_mut() {
        let project_path = std::path::Path::new(&project.path);
        let project_godot = project_path.join("project.godot");

        if !project_path.exists() || !project_godot.exists() {
            project.status = ProjectStatus::MissingSource;
            project.last_synced_at = Some(now);
            synced_count += 1;
            continue;
        }

        match ProjectScanner::parse_project(&project_godot) {
            Ok(scanned) => {
                project.name = scanned.name;
                project.godot_version = scanned.godot_version;
                project.icon_path = scanned.icon_path;
                project.status = scanned.status;
                project.updated_at = now;
                project.last_synced_at = Some(now);
                synced_count += 1;
            }
            Err(_) => {
                project.status = ProjectStatus::Warning;
                project.last_synced_at = Some(now);
                synced_count += 1;
            }
        }
    }

    storage.save("projects.json", &projects)
        .map_err(|e| format!("保存项目列表失败: {}", e))?;

    log_operation(&app, "sync_projects", "",
        &format!("增量同步完成，共同步 {} 个项目", synced_count));

    Ok(projects)
}

#[tauri::command]
pub fn check_godot_updates(app: AppHandle) -> Result<crate::version_checker::GodotVersionCheckResult, String> {
    let storage = get_storage(&app);
    let engines: Vec<Engine> = storage.load_or_default("engines.json");

    let local_engines: Vec<crate::version_checker::LocalEngineVersion> = engines.iter()
        .map(|e| crate::version_checker::LocalEngineVersion {
            engine_id: e.engine_id.clone(),
            name: e.name.clone(),
            version: e.version.clone(),
            engine_type: format!("{:?}", e.engine_type),
        })
        .collect();

    let data_dir = get_data_dir(&app);
    let cache_dir = data_dir.join("cache");
    let checker = crate::version_checker::VersionChecker::new(cache_dir);

    let result = checker.check_for_updates(local_engines)?;

    if !result.updates_available.is_empty() {
        let _ = app.emit("godot-update-available", &result.updates_available);
    }

    log_operation(&app, "check_godot_updates", "",
        &format!("Godot 版本检查完成，发现 {} 个可用更新", result.updates_available.len()));

    Ok(result)
}

#[tauri::command]
pub fn batch_remove_projects(app: AppHandle, project_ids: Vec<String>) -> Result<BatchResult, String> {
    let storage = get_storage(&app);
    let mut projects: Vec<Project> = storage.load_or_default("projects.json");
    let mut success_count = 0;
    let mut failed_count = 0;
    let mut errors = Vec::new();

    for project_id in &project_ids {
        if projects.iter().any(|p| p.project_id == *project_id) {
            projects.retain(|p| p.project_id != *project_id);
            success_count += 1;
        } else {
            failed_count += 1;
            errors.push(format!("未找到项目: {}", project_id));
        }
    }

    storage.save("projects.json", &projects)
        .map_err(|e| format!("保存项目列表失败: {}", e))?;

    log_operation(&app, "batch_remove_projects", "",
        &format!("批量删除项目: 成功 {}, 失败 {}", success_count, failed_count));

    Ok(BatchResult { success_count, failed_count, errors })
}

#[tauri::command]
pub fn batch_remove_plugins(app: AppHandle, plugin_ids: Vec<String>) -> Result<BatchResult, String> {
    let storage = get_storage(&app);
    let mut plugins: Vec<Plugin> = storage.load_or_default("plugins.json");
    let mut success_count = 0;
    let mut failed_count = 0;
    let mut errors = Vec::new();

    let plugins_base_dir = get_data_dir(&app).join("plugins");

    for plugin_id in &plugin_ids {
        if let Some(_plugin) = plugins.iter().find(|p| p.plugin_id == *plugin_id) {
            let plugin_dir = plugins_base_dir.join(plugin_id);
            if plugin_dir.exists() {
                if let Err(e) = fs::remove_dir_all(&plugin_dir) {
                    errors.push(format!("删除插件 {} 文件失败: {}", plugin_id, e));
                    failed_count += 1;
                    continue;
                }
            }
            plugins.retain(|p| p.plugin_id != *plugin_id);
            success_count += 1;
        } else {
            failed_count += 1;
            errors.push(format!("未找到插件: {}", plugin_id));
        }
    }

    let mut bindings: Vec<ProjectBinding> = storage.load_or_default("bindings.json");
    bindings.retain(|b| !plugin_ids.contains(&b.plugin_id));
    storage.save("bindings.json", &bindings)
        .map_err(|e| format!("保存绑定关系失败: {}", e))?;

    storage.save("plugins.json", &plugins)
        .map_err(|e| format!("保存插件列表失败: {}", e))?;

    log_operation(&app, "batch_remove_plugins", "",
        &format!("批量删除插件: 成功 {}, 失败 {}", success_count, failed_count));

    Ok(BatchResult { success_count, failed_count, errors })
}

#[tauri::command]
pub fn batch_bind_plugins(app: AppHandle, bindings: Vec<BatchBindingRequest>) -> Result<BatchResult, String> {
    let storage = get_storage(&app);
    let projects: Vec<Project> = storage.load_or_default("projects.json");
    let plugins: Vec<Plugin> = storage.load_or_default("plugins.json");
    let mut all_bindings: Vec<ProjectBinding> = storage.load_or_default("bindings.json");

    let mut success_count = 0;
    let mut failed_count = 0;
    let mut errors = Vec::new();

    for req in &bindings {
        if !projects.iter().any(|p| p.project_id == req.project_id) {
            failed_count += 1;
            errors.push(format!("未找到项目: {}", req.project_id));
            continue;
        }
        if !plugins.iter().any(|p| p.plugin_id == req.plugin_id) {
            failed_count += 1;
            errors.push(format!("未找到插件: {}", req.plugin_id));
            continue;
        }
        if req.mount_path.is_empty() {
            failed_count += 1;
            errors.push(format!("挂载路径为空: 项目 {} 插件 {}", req.project_id, req.plugin_id));
            continue;
        }

        all_bindings.retain(|b| !(b.project_id == req.project_id && b.plugin_id == req.plugin_id));
        let binding = ProjectBinding::new(
            req.project_id.clone(),
            req.plugin_id.clone(),
            req.version_id.clone(),
            req.unit_id.clone(),
            req.mount_path.clone(),
        );
        all_bindings.push(binding);
        success_count += 1;
    }

    storage.save("bindings.json", &all_bindings)
        .map_err(|e| format!("保存绑定关系失败: {}", e))?;

    log_operation(&app, "batch_bind_plugins", "",
        &format!("批量绑定插件: 成功 {}, 失败 {}", success_count, failed_count));

    Ok(BatchResult { success_count, failed_count, errors })
}

#[tauri::command]
pub fn batch_unbind_plugins(app: AppHandle, project_id: String, plugin_ids: Vec<String>) -> Result<BatchResult, String> {
    let storage = get_storage(&app);
    let mut bindings: Vec<ProjectBinding> = storage.load_or_default("bindings.json");

    let mut success_count = 0;
    let mut failed_count = 0;
    let mut errors = Vec::new();

    let projects: Vec<Project> = storage.load_or_default("projects.json");
    let project = match projects.iter().find(|p| p.project_id == project_id) {
        Some(p) => p,
        None => return Err("未找到指定项目".to_string()),
    };

    for plugin_id in &plugin_ids {
        let binding = bindings.iter()
            .find(|b| b.project_id == project_id && b.plugin_id == *plugin_id);

        if let Some(binding) = binding {
            let mount_path = binding.mount_path.clone();
            let addons_dir = std::path::Path::new(&project.path).join("addons");
            if addons_dir.exists() {
                let plugin_path = addons_dir.join(&mount_path);
                if plugin_path.exists() {
                    let metadata = std::fs::symlink_metadata(&plugin_path);
                    let is_link = metadata.as_ref().map(|m| m.file_type().is_symlink()).unwrap_or(false);
                    let is_junction = if cfg!(windows) {
                        use std::os::windows::fs::MetadataExt;
                        metadata.as_ref().map(|m| m.file_attributes() & 0x400 != 0).unwrap_or(false)
                    } else {
                        false
                    };
                    if is_link || is_junction {
                        let _ = std::fs::remove_dir(&plugin_path);
                    } else {
                        let _ = std::fs::remove_dir_all(&plugin_path);
                    }
                }
            }
            bindings.retain(|b| !(b.project_id == project_id && b.plugin_id == *plugin_id));
            success_count += 1;
        } else {
            failed_count += 1;
            errors.push(format!("未找到绑定关系: 插件 {}", plugin_id));
        }
    }

    storage.save("bindings.json", &bindings)
        .map_err(|e| format!("保存绑定关系失败: {}", e))?;

    log_operation(&app, "batch_unbind_plugins", &project_id,
        &format!("批量解绑插件: 成功 {}, 失败 {}", success_count, failed_count));

    Ok(BatchResult { success_count, failed_count, errors })
}

#[tauri::command]
pub fn batch_apply_changes(app: AppHandle, project_ids: Vec<String>) -> Result<BatchApplyResult, String> {
    let storage = get_storage(&app);
    let projects: Vec<Project> = storage.load_or_default("projects.json");
    let all_bindings: Vec<ProjectBinding> = storage.load_or_default("bindings.json");
    let settings: Settings = storage.load_or_default("settings.json");
    let linker = Linker::new(settings.mount_strategy);
    let data_dir = get_data_dir(&app);
    let plugin_base_path = data_dir.join("plugins");

    let mut results = Vec::new();

    for project_id in &project_ids {
        let project = match projects.iter().find(|p| p.project_id == *project_id) {
            Some(p) => p,
            None => {
                results.push(ProjectApplyResult {
                    project_id: project_id.clone(),
                    project_name: String::new(),
                    success: false,
                    created: Vec::new(),
                    removed: Vec::new(),
                    errors: vec![format!("未找到项目: {}", project_id)],
                });
                continue;
            }
        };

        let desired_bindings: Vec<ProjectBinding> = all_bindings.iter()
            .filter(|b| b.project_id == *project_id)
            .cloned()
            .collect();

        if desired_bindings.is_empty() {
            results.push(ProjectApplyResult {
                project_id: project_id.clone(),
                project_name: project.name.clone(),
                success: true,
                created: Vec::new(),
                removed: Vec::new(),
                errors: Vec::new(),
            });
            continue;
        }

        let current_bindings: Vec<ProjectBinding> = Vec::new();

        match linker.apply_bindings(
            &project.path,
            &current_bindings,
            &desired_bindings,
            &plugin_base_path.to_string_lossy()
        ) {
            Ok(apply_result) => {
                results.push(ProjectApplyResult {
                    project_id: project_id.clone(),
                    project_name: project.name.clone(),
                    success: apply_result.success,
                    created: apply_result.created,
                    removed: apply_result.removed,
                    errors: apply_result.errors,
                });
            }
            Err(e) => {
                results.push(ProjectApplyResult {
                    project_id: project_id.clone(),
                    project_name: project.name.clone(),
                    success: false,
                    created: Vec::new(),
                    removed: Vec::new(),
                    errors: vec![format!("应用变更失败: {}", e)],
                });
            }
        }
    }

    log_operation(&app, "batch_apply_changes", "",
        &format!("批量应用变更完成，共处理 {} 个项目", results.len()));

    Ok(BatchApplyResult { results })
}

#[tauri::command]
pub fn auto_discover_engines(app: AppHandle) -> Result<Vec<Engine>, String> {
    let settings: Settings = {
        let storage = get_storage(&app);
        storage.load_or_default("settings.json")
    };

    if !settings.auto_discover_engines {
        log_operation(&app, "auto_discover_engines", "", "自动发现已关闭");
        return Ok(Vec::new());
    }

    let storage = get_storage(&app);
    let mut engines: Vec<Engine> = storage.load_or_default("engines.json");

    let mut removed_count = 0;
    engines.retain(|e| {
        let valid = std::path::Path::new(&e.path).exists();
        if !valid {
            removed_count += 1;
        }
        valid
    });
    if removed_count > 0 {
        let _ = storage.save("engines.json", &engines);
        log_operation(&app, "auto_discover_engines", "",
            &format!("清理 {} 个失效引擎", removed_count));
    }

    let existing_paths: Vec<String> = engines.iter().map(|e| e.path.clone()).collect();

    log_operation(&app, "auto_discover_engines", "", "开始自动发现引擎");

    let discovered = if settings.scan_directories.is_empty() {
        crate::engine::EngineManager::discover_engines(&existing_paths)
    } else {
        crate::engine::EngineManager::discover_engines_with_custom_paths(
            &existing_paths,
            &settings.scan_directories,
        )
    };

    if discovered.is_empty() {
        log_operation(&app, "auto_discover_engines", "", "未发现新引擎");
        return Ok(Vec::new());
    }

    let discovered_count = discovered.len();
    let first_engine = engines.is_empty();
    for engine in &discovered {
        if first_engine && engines.is_empty() {
            let mut e = engine.clone();
            e.is_default = true;
            engines.push(e);
        } else {
            engines.push(engine.clone());
        }
    }

    storage.save("engines.json", &engines)
        .map_err(|e| format!("保存引擎列表失败: {}", e))?;

    let _ = app.emit("engines-discovered", &discovered);

    log_operation(&app, "auto_discover_engines", "",
        &format!("自动发现 {} 个 Godot 引擎", discovered_count));

    Ok(discovered)
}
