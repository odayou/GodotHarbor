use std::path::{PathBuf, Path};
use std::fs;
use std::io::Write;
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
use futures::future::join_all;

use crate::utils::{copy_dir_all, create_http_client, no_window_cmd};

pub fn get_config_dir(app: &AppHandle) -> PathBuf {
    app.path().app_data_dir()
        .expect("Failed to get app data directory")
}

pub fn get_data_dir(app: &AppHandle) -> PathBuf {
    let config_dir = get_config_dir(app);
    let config_storage = Storage::new(config_dir.clone());
    let settings: Settings = config_storage.load_or_default("settings.json");
    if settings.custom_data_dir.is_empty() {
        config_dir
    } else {
        PathBuf::from(&settings.custom_data_dir)
    }
}

fn get_config_storage(app: &AppHandle) -> Storage {
    Storage::new(get_config_dir(app))
}

pub fn get_storage(app: &AppHandle) -> Storage {
    Storage::new(get_data_dir(app))
}

fn load_settings(app: &AppHandle) -> Settings {
    get_config_storage(app).load_or_default("settings.json")
}

fn save_settings_to_config(app: &AppHandle, settings: &Settings) -> Result<(), String> {
    get_config_storage(app).save("settings.json", settings)
        .map_err(|e| format!("保存设置失败: {}", e))
}

fn get_plugin_manager(app: &AppHandle) -> PluginManager {
    let plugins_dir = get_data_dir(app).join("plugins");
    PluginManager::new(plugins_dir)
}

fn get_logger(app: &AppHandle) -> OperationLogger {
    let data_dir = get_data_dir(app);
    OperationLogger::new(data_dir)
}

fn upsert_plugin(app: &AppHandle, new_plugin: &crate::models::Plugin, operation: &str, source_desc: &str) -> Result<crate::models::Plugin, String> {
    let storage = get_storage(app);
    let mut plugins: Vec<crate::models::Plugin> = storage.load_or_default("plugins.json");
    let plugin_name = new_plugin.name.clone();
    let existing_idx = plugins.iter().position(|p| p.source.url == new_plugin.source.url);
    if let Some(idx) = existing_idx {
        plugins[idx].versions.extend(new_plugin.versions.clone());
        if !new_plugin.content_hash.is_empty() {
            plugins[idx].content_hash = new_plugin.content_hash.clone();
        }
        let result = plugins[idx].clone();
        storage.save("plugins.json", &plugins)
            .map_err(|e| format!("保存插件列表失败: {}", e))?;
        log_operation(app, operation, source_desc, &format!("已为插件 {} 添加新版本", plugin_name));
        Ok(result)
    } else {
        if !new_plugin.content_hash.is_empty() {
            if let Some(dup) = plugins.iter().find(|p| !p.content_hash.is_empty() && p.content_hash == new_plugin.content_hash) {
                log_operation(app, operation, source_desc,
                    &format!("插件 {} 与已有插件 {} 内容相同(hash匹配)", plugin_name, dup.name));
            }
        }
        plugins.push(new_plugin.clone());
        storage.save("plugins.json", &plugins)
            .map_err(|e| format!("保存插件列表失败: {}", e))?;
        log_operation(app, operation, source_desc, &format!("已导入插件: {}", plugin_name));
        Ok(new_plugin.clone())
    }
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

const DATA_FILES: &[&str] = &[
    "settings.json",
    "projects.json",
    "plugins.json",
    "bindings.json",
    "engines.json",
    "operation_logs.json",
    "update_logs.json"
];

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

fn backup_addons_dir(addons_dir: &std::path::Path, backup_file: &std::path::Path) -> Result<(), String> {
    let file = std::fs::File::create(backup_file)
        .map_err(|e| format!("创建备份文件失败: {}", e))?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    let mut stack = vec![(addons_dir.to_path_buf(), String::new())];
    while let Some((dir, prefix)) = stack.pop() {
        let entries = std::fs::read_dir(&dir)
            .map_err(|e| format!("读取目录失败: {}", e))?;
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            let entry_path = if prefix.is_empty() {
                name.clone()
            } else {
                format!("{}/{}", prefix, name)
            };
            if path.is_dir() {
                zip.add_directory(&entry_path, options)
                    .map_err(|e| format!("写入目录条目失败: {}", e))?;
                stack.push((path, entry_path));
            } else {
                let data = std::fs::read(&path)
                    .map_err(|e| format!("读取文件失败: {}", e))?;
                zip.start_file(&entry_path, options)
                    .map_err(|e| format!("写入文件条目失败: {}", e))?;
                zip.write_all(&data)
                    .map_err(|e| format!("写入文件数据失败: {}", e))?;
            }
        }
    }
    zip.finish().map_err(|e| format!("完成压缩失败: {}", e))?;
    Ok(())
}

fn cleanup_old_backups(backup_dir: &std::path::Path, max_keep: usize) {
    if let Ok(entries) = std::fs::read_dir(backup_dir) {
        let mut backups: Vec<std::path::PathBuf> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| {
                p.extension().map(|e| e == "zip").unwrap_or(false)
                && p.file_name()
                    .map(|n| n.to_string_lossy().starts_with("addons_backup_"))
                    .unwrap_or(false)
            })
            .collect();
        if backups.len() > max_keep {
            backups.sort();
            let to_remove = backups.len() - max_keep;
            for old_file in backups.iter().take(to_remove) {
                let _ = std::fs::remove_file(old_file);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddonBackupInfo {
    pub file_name: String,
    pub file_path: String,
    pub file_size: u64,
    pub created_at: String,
}

#[tauri::command]
pub fn list_addon_backups(app: AppHandle, project_id: String) -> Result<Vec<AddonBackupInfo>, String> {
    let storage = get_storage(&app);
    let projects: Vec<Project> = storage.load_or_default("projects.json");
    let project = projects.iter()
        .find(|p| p.project_id == project_id)
        .ok_or_else(|| "未找到指定项目".to_string())?;

    let data_dir = get_data_dir(&app);
    let backup_dir = data_dir.join("backups").join(&project.name);

    if !backup_dir.exists() {
        return Ok(Vec::new());
    }

    let mut backups = Vec::new();
    let entries = std::fs::read_dir(&backup_dir)
        .map_err(|e| format!("读取备份目录失败: {}", e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().map(|e| e == "zip").unwrap_or(false) {
            let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
            if !file_name.starts_with("addons_backup_") {
                continue;
            }
            let file_size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            let created_at = file_name
                .strip_prefix("addons_backup_")
                .and_then(|s| s.strip_suffix(".zip"))
                .unwrap_or("unknown")
                .replace("_", " ");
            backups.push(AddonBackupInfo {
                file_name,
                file_path: path.to_string_lossy().to_string(),
                file_size,
                created_at,
            });
        }
    }

    backups.sort_by(|a, b| b.file_name.cmp(&a.file_name));
    Ok(backups)
}

#[tauri::command]
pub fn restore_addon_backup(app: AppHandle, project_id: String, backup_file: String) -> Result<(), String> {
    let storage = get_storage(&app);
    let projects: Vec<Project> = storage.load_or_default("projects.json");
    let project = projects.iter()
        .find(|p| p.project_id == project_id)
        .ok_or_else(|| "未找到指定项目".to_string())?;

    let addons_dir = std::path::Path::new(&project.path).join("addons");
    let backup_path = std::path::Path::new(&backup_file);

    if !backup_path.exists() {
        return Err("备份文件不存在".to_string());
    }

    if addons_dir.exists() {
        std::fs::remove_dir_all(&addons_dir)
            .map_err(|e| format!("删除当前 addons 目录失败: {}", e))?;
    }

    let file = std::fs::File::open(backup_path)
        .map_err(|e| format!("打开备份文件失败: {}", e))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| format!("读取备份文件失败: {}", e))?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)
            .map_err(|e| format!("读取压缩条目失败: {}", e))?;
        let out_path = addons_dir.join(entry.name());
        if entry.is_dir() {
            std::fs::create_dir_all(&out_path)
                .map_err(|e| format!("创建目录失败: {}", e))?;
        } else {
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("创建父目录失败: {}", e))?;
            }
            let mut out_file = std::fs::File::create(&out_path)
                .map_err(|e| format!("创建文件失败: {}", e))?;
            std::io::copy(&mut entry, &mut out_file)
                .map_err(|e| format!("写入文件失败: {}", e))?;
        }
    }

    log_operation(&app, "restore_addon_backup", &project_id,
        &format!("从备份恢复 addons: {}", backup_path.to_string_lossy()));

    Ok(())
}

#[tauri::command]
pub fn save_as_template(app: AppHandle, project_id: String, template_name: String) -> Result<ProjectTemplate, String> {
    let storage = get_storage(&app);
    let projects: Vec<Project> = storage.load_or_default("projects.json");
    let _project = projects.iter()
        .find(|p| p.project_id == project_id)
        .ok_or_else(|| "未找到指定项目".to_string())?;

    let bindings: Vec<ProjectBinding> = storage.load_or_default("bindings.json");
    let project_bindings: Vec<&ProjectBinding> = bindings.iter()
        .filter(|b| b.project_id == project_id)
        .collect();

    let plugins: Vec<Plugin> = storage.load_or_default("plugins.json");

    let template_bindings: Vec<TemplateBinding> = project_bindings.iter().map(|b| {
        let plugin = plugins.iter().find(|p| p.plugin_id == b.plugin_id);
        let version = plugin.and_then(|p| p.versions.iter().find(|v| v.version_id == b.version_id));
        let unit = version.and_then(|v| v.units.iter().find(|u| u.unit_id == b.unit_id));
        TemplateBinding {
            plugin_id: b.plugin_id.clone(),
            plugin_name: plugin.map(|p| p.name.clone()).unwrap_or_default(),
            version_id: b.version_id.clone(),
            unit_id: b.unit_id.clone(),
            unit_name: unit.map(|u| u.name.clone()).unwrap_or_default(),
            mount_path: b.mount_path.clone(),
            subdirectory: b.subdirectory.clone(),
        }
    }).collect();

    let template = ProjectTemplate {
        template_id: uuid::Uuid::new_v4().to_string(),
        name: template_name.clone(),
        bindings: template_bindings,
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    let data_dir = get_data_dir(&app);
    let templates_file = data_dir.join("templates.json");
    let mut templates: Vec<ProjectTemplate> = if templates_file.exists() {
        let file = std::fs::read_to_string(&templates_file)
            .map_err(|e| format!("读取模板文件失败: {}", e))?;
        serde_json::from_str(&file).unwrap_or_default()
    } else {
        Vec::new()
    };
    templates.push(template.clone());
    let json = serde_json::to_string_pretty(&templates)
        .map_err(|e| format!("序列化模板失败: {}", e))?;
    std::fs::write(&templates_file, json)
        .map_err(|e| format!("保存模板文件失败: {}", e))?;

    log_operation(&app, "save_as_template", &project_id,
        &format!("保存为模板: {} ({} 个绑定)", template_name, template.bindings.len()));

    Ok(template)
}

#[tauri::command]
pub fn list_templates(app: AppHandle) -> Result<Vec<ProjectTemplate>, String> {
    let data_dir = get_data_dir(&app);
    let templates_file = data_dir.join("templates.json");
    if !templates_file.exists() {
        return Ok(Vec::new());
    }
    let file = std::fs::read_to_string(&templates_file)
        .map_err(|e| format!("读取模板文件失败: {}", e))?;
    let templates: Vec<ProjectTemplate> = serde_json::from_str(&file).unwrap_or_default();
    Ok(templates)
}

#[tauri::command]
pub fn delete_template(app: AppHandle, template_id: String) -> Result<(), String> {
    let data_dir = get_data_dir(&app);
    let templates_file = data_dir.join("templates.json");
    if !templates_file.exists() {
        return Ok(());
    }
    let file = std::fs::read_to_string(&templates_file)
        .map_err(|e| format!("读取模板文件失败: {}", e))?;
    let mut templates: Vec<ProjectTemplate> = serde_json::from_str(&file).unwrap_or_default();
    templates.retain(|t| t.template_id != template_id);
    let json = serde_json::to_string_pretty(&templates)
        .map_err(|e| format!("序列化模板失败: {}", e))?;
    std::fs::write(&templates_file, json)
        .map_err(|e| format!("保存模板文件失败: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn apply_template_to_project(app: AppHandle, project_id: String, template_id: String) -> Result<ApplyResult, String> {
    let data_dir = get_data_dir(&app);
    let templates_file = data_dir.join("templates.json");
    let file = std::fs::read_to_string(&templates_file)
        .map_err(|e| format!("读取模板文件失败: {}", e))?;
    let templates: Vec<ProjectTemplate> = serde_json::from_str(&file).unwrap_or_default();
    let template = templates.iter()
        .find(|t| t.template_id == template_id)
        .ok_or_else(|| "未找到指定模板".to_string())?;

    let storage = get_storage(&app);
    let mut bindings: Vec<ProjectBinding> = storage.load_or_default("bindings.json");

    for tb in &template.bindings {
        if let Some(existing) = bindings.iter_mut().find(|b| b.project_id == project_id && b.plugin_id == tb.plugin_id) {
            existing.version_id = tb.version_id.clone();
            existing.unit_id = tb.unit_id.clone();
            existing.mount_path = tb.mount_path.clone();
            existing.subdirectory = tb.subdirectory.clone();
        } else {
            bindings.push(ProjectBinding {
                project_id: project_id.clone(),
                plugin_id: tb.plugin_id.clone(),
                version_id: tb.version_id.clone(),
                unit_id: tb.unit_id.clone(),
                mount_path: tb.mount_path.clone(),
                is_healthy: Some(true),
                subdirectory: tb.subdirectory.clone(),
                created_at: chrono::Utc::now(),
            });
        }
    }

    let json = serde_json::to_string_pretty(&bindings)
        .map_err(|e| format!("序列化绑定失败: {}", e))?;
    let data_dir = get_data_dir(&app);
    std::fs::write(data_dir.join("bindings.json"), json)
        .map_err(|e| format!("保存绑定文件失败: {}", e))?;

    apply_changes(app, project_id)
}

#[tauri::command]
pub fn get_settings(app: AppHandle) -> Result<Settings, String> {
    let settings = load_settings(&app);
    Ok(settings)
}

#[tauri::command]
pub fn save_settings(app: AppHandle, settings: Settings) -> Result<(), String> {
    save_settings_to_config(&app, &settings)?;
    log_operation(&app, "save_settings", "settings.json", "设置已保存");
    Ok(())
}

#[cfg(windows)]
fn detached_cmd(program: impl AsRef<std::ffi::OsStr>) -> std::process::Command {
    use std::os::windows::process::CommandExt;
    const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
    const DETACHED_PROCESS: u32 = 0x00000008;
    let mut cmd = std::process::Command::new(program);
    cmd.creation_flags(CREATE_NEW_PROCESS_GROUP | DETACHED_PROCESS);
    cmd
}

#[cfg(not(windows))]
fn detached_cmd(program: impl AsRef<std::ffi::OsStr>) -> std::process::Command {
    let mut cmd = std::process::Command::new(program);
    cmd
}

#[tauri::command]
pub fn launch_engine(app: AppHandle, engine_id: String) -> Result<(), String> {
    let storage = get_storage(&app);
    let engines: Vec<Engine> = storage.load_or_default("engines.json");

    let engine = engines.iter()
        .find(|e| e.engine_id == engine_id)
        .ok_or("未找到指定引擎".to_string())?;

    let exe_path = crate::engine::EngineManager::find_executable_in_dir(std::path::Path::new(&engine.path))
        .ok_or("未找到引擎可执行文件".to_string())?;

    detached_cmd(&exe_path)
        .spawn()
        .map_err(|e| format!("启动引擎失败: {}", e))?;

    log_operation(&app, "launch_engine", &engine_id, &format!("启动引擎: {}", engine.name));
    Ok(())
}
#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct AutoSetupState {
    completed_at: i64,
    settings_hash: String,
}

fn compute_settings_hash(settings: &Settings) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    settings.scan_directories.join(",").hash(&mut hasher);
    settings.auto_scan_on_startup.hash(&mut hasher);
    settings.auto_discover_engines.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

#[tauri::command]
pub fn check_auto_setup_needed(app: AppHandle) -> Result<bool, String> {
    let storage = get_storage(&app);
    let settings = load_settings(&app);
    let current_hash = compute_settings_hash(&settings);

    let state: Option<AutoSetupState> = storage.load("auto_setup_state.json").ok();

    match state {
        Some(s) => {
            let now = chrono::Utc::now().timestamp();
            let elapsed_hours = (now - s.completed_at) / 3600;
            if s.settings_hash == current_hash && elapsed_hours < 24 {
                Ok(false)
            } else {
                Ok(true)
            }
        }
        None => Ok(true),
    }
}

#[tauri::command]
pub fn mark_auto_setup_done(app: AppHandle) -> Result<(), String> {
    let storage = get_storage(&app);
    let settings = load_settings(&app);
    let state = AutoSetupState {
        completed_at: chrono::Utc::now().timestamp(),
        settings_hash: compute_settings_hash(&settings),
    };
    storage.save("auto_setup_state.json", &state)
        .map_err(|e| format!("保存自动设置状态失败: {}", e))
}

#[tauri::command]
pub fn read_file_as_base64(path: String) -> Result<String, String> {
    use std::io::Read;
    let mut file = std::fs::File::open(&path).map_err(|e| format!("打开文件失败: {}", e))?;
    let mut data = Vec::new();
    file.read_to_end(&mut data).map_err(|e| format!("读取文件失败: {}", e))?;
    Ok(base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &data))
}

#[tauri::command]
pub fn migrate_data_dir(app: AppHandle, new_data_dir: String) -> Result<(), String> {
    let new_path = Path::new(&new_data_dir);
    if new_path.exists() && !new_path.is_dir() {
        return Err("目标路径已存在但不是目录".to_string());
    }

    let old_data_dir = get_data_dir(&app);
    let old_str = old_data_dir.to_string_lossy().to_string();
    if old_str == new_data_dir {
        return Err("新目录与当前目录相同".to_string());
    }

    std::fs::create_dir_all(new_path)
        .map_err(|e| format!("创建目录失败: {}", e))?;

    let exclude_names: &[&str] = &["settings.json"];
    for entry in fs::read_dir(&old_data_dir)
        .map_err(|e| format!("读取源目录失败: {}", e))?
    {
        let entry = entry.map_err(|e| format!("读取目录条目失败: {}", e))?;
        let file_name = entry.file_name();
        let name_str = file_name.to_string_lossy();
        if exclude_names.iter().any(|ex| *ex == name_str) {
            continue;
        }
        let ty = entry.file_type().map_err(|e| format!("获取文件类型失败: {}", e))?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &new_path.join(&file_name))?;
        } else {
            fs::copy(&entry.path(), &new_path.join(&file_name))
                .map_err(|e| format!("复制 {} 失败: {}", name_str, e))?;
        }
    }

    let mut settings = load_settings(&app);
    settings.custom_data_dir = new_data_dir.clone();
    save_settings_to_config(&app, &settings)?;

    let new_data_dir_path = PathBuf::from(&new_data_dir);
    let new_engines_json = new_data_dir_path.join("engines.json");
    if new_engines_json.exists() {
        let new_storage = Storage::new(new_data_dir_path.clone());
        let mut engines: Vec<Engine> = new_storage.load_or_default("engines.json");
        let mut changed = false;
        for engine in &mut engines {
            if engine.path.starts_with(&old_str) {
                engine.path = engine.path.replacen(&old_str, &new_data_dir, 1);
                changed = true;
            }
        }
        if changed {
            let _ = new_storage.save("engines.json", &engines);
        }
    }

    let new_projects_json = new_data_dir_path.join("projects.json");
    if new_projects_json.exists() {
        let new_storage = Storage::new(new_data_dir_path.clone());
        let mut projects: Vec<Project> = new_storage.load_or_default("projects.json");
        let mut changed = false;
        for project in &mut projects {
            if project.path.starts_with(&old_str) {
                project.path = project.path.replacen(&old_str, &new_data_dir, 1);
                changed = true;
            }
        }
        if changed {
            let _ = new_storage.save("projects.json", &projects);
        }
    }

    let new_plugins_json = new_data_dir_path.join("plugins.json");
    if new_plugins_json.exists() {
        let new_storage = Storage::new(new_data_dir_path.clone());
        let mut plugins: Vec<Plugin> = new_storage.load_or_default("plugins.json");
        let mut changed = false;
        for plugin in &mut plugins {
            for version in &mut plugin.versions {
                if version.path.starts_with(&old_str) {
                    version.path = version.path.replacen(&old_str, &new_data_dir, 1);
                    changed = true;
                }
                for unit in &mut version.units {
                    if unit.plugin_cfg_path.starts_with(&old_str) {
                        unit.plugin_cfg_path = unit.plugin_cfg_path.replacen(&old_str, &new_data_dir, 1);
                        changed = true;
                    }
                }
            }
        }
        if changed {
            let _ = new_storage.save("plugins.json", &plugins);
        }
    }

    for entry in fs::read_dir(&old_data_dir)
        .map_err(|e| format!("读取源目录失败: {}", e))?
    {
        let entry = entry.map_err(|e| format!("读取目录条目失败: {}", e))?;
        let file_name = entry.file_name();
        let name_str = file_name.to_string_lossy();
        if exclude_names.iter().any(|ex| *ex == name_str) {
            continue;
        }
        let ty = entry.file_type().map_err(|e| format!("获取文件类型失败: {}", e))?;
        if ty.is_dir() {
            std::fs::remove_dir_all(entry.path())
                .map_err(|e| format!("删除目录 {} 失败: {}", name_str, e))?;
        } else {
            std::fs::remove_file(entry.path())
                .map_err(|e| format!("删除文件 {} 失败: {}", name_str, e))?;
        }
    }

    log_operation(&app, "migrate_data_dir", &new_data_dir,
        &format!("数据目录已迁移: {} -> {}", old_str, new_data_dir));
    Ok(())
}
#[tauri::command]
pub fn get_storage_paths(app: AppHandle) -> Result<StoragePaths, String> {
    let config_dir = get_config_dir(&app);
    let data_dir = get_data_dir(&app);
    let plugins_dir = data_dir.join("plugins");
    Ok(StoragePaths {
        app_data_dir: data_dir.to_string_lossy().to_string(),
        plugins_dir: plugins_dir.to_string_lossy().to_string(),
        engines_dir: data_dir.join("engines").to_string_lossy().to_string(),
        cache_dir: data_dir.join("cache").to_string_lossy().to_string(),
        logs_dir: data_dir.join("logs").to_string_lossy().to_string(),
        hot_updates_dir: data_dir.join("hot_updates").to_string_lossy().to_string(),
        settings_file: config_dir.join("settings.json").to_string_lossy().to_string(),
        projects_file: data_dir.join("projects.json").to_string_lossy().to_string(),
        engines_file: data_dir.join("engines.json").to_string_lossy().to_string(),
    })
}
#[tauri::command]
pub async fn fetch_remote_engine_versions(
    app: AppHandle,
    mirror_id: String,
    force_refresh: Option<bool>,
) -> Result<Vec<crate::models::RemoteEngineVersion>, String> {
    let settings = load_settings(&app);

    let mirror = settings.engine_mirrors.iter()
        .find(|m| m.id == mirror_id)
        .ok_or("未找到指定的镜像配置".to_string())?;

    if !mirror.enabled {
        return Err("该镜像已被禁用".to_string());
    }

    let force = force_refresh.unwrap_or(false);
    let current_cache_version: u32 = 2;

    let cache_dir = get_data_dir(&app).join("cache");
    let cache_file = cache_dir.join(format!("remote_versions_{}.json", mirror_id));

    let mut expired_cache: Option<crate::models::CachedRemoteVersions> = None;

    if force {
        let _ = fs::remove_file(&cache_file);
    } else if cache_file.exists() {
        if let Ok(content) = fs::read_to_string(&cache_file) {
            if let Ok(cached) = serde_json::from_str::<crate::models::CachedRemoteVersions>(&content) {
                if cached.cache_version != current_cache_version {
                    let _ = fs::remove_file(&cache_file);
                    log_operation(&app, "fetch_remote_engine_versions", &mirror_id,
                        "缓存版本不匹配，已清除旧缓存");
                } else if let Ok(cached_time) = chrono::DateTime::parse_from_rfc3339(&cached.cached_at) {
                    let elapsed = chrono::Utc::now().signed_duration_since(cached_time.with_timezone(&chrono::Utc));
                    if elapsed.num_minutes() < 30 {
                        log_operation(&app, "fetch_remote_engine_versions", &mirror_id,
                            &format!("使用缓存，共 {} 个版本", cached.versions.len()));
                        return Ok(cached.versions);
                    }
                    if !cached.versions.is_empty() {
                        expired_cache = Some(cached);
                    }
                }
            }
        }
    }

    let storage = get_storage(&app);
    let engines: Vec<Engine> = storage.load_or_default("engines.json");
    let local_versions: Vec<String> = engines.iter().map(|e| e.version.clone()).collect();

    match crate::engine_downloader::EngineDownloader::fetch_remote_versions(mirror, &local_versions).await {
        Ok(versions) => {
            if versions.is_empty() {
                if let Some(ref cached) = expired_cache {
                    log_operation(&app, "fetch_remote_engine_versions", &mirror_id,
                        "API 返回空结果，使用过期缓存");
                    return Ok(cached.versions.clone());
                }
                log_operation(&app, "fetch_remote_engine_versions", &mirror_id,
                    "API 返回空结果且无缓存");
                return Ok(versions);
            }

            let cache_dir = get_data_dir(&app).join("cache");
            let _ = fs::create_dir_all(&cache_dir);
            let cache_file = cache_dir.join(format!("remote_versions_{}.json", mirror_id));
            let cached = crate::models::CachedRemoteVersions {
                cache_version: current_cache_version,
                cached_at: chrono::Utc::now().to_rfc3339(),
                mirror_id: mirror_id.clone(),
                versions: versions.clone(),
            };
            if let Ok(json) = serde_json::to_string_pretty(&cached) {
                let _ = fs::write(&cache_file, json);
            }

            log_operation(&app, "fetch_remote_engine_versions", &mirror_id,
                &format!("获取远程引擎版本列表，共 {} 个版本", versions.len()));

            Ok(versions)
        }
        Err(e) => {
            if let Some(ref cached) = expired_cache {
                log_operation(&app, "fetch_remote_engine_versions", &mirror_id,
                    &format!("API 请求失败({}), 使用过期缓存，共 {} 个版本", e, cached.versions.len()));
                return Ok(cached.versions.clone());
            }
            Err(e)
        }
    }
}

#[tauri::command]
pub async fn download_engine(
    app: AppHandle,
    remote_version: crate::models::RemoteEngineVersion,
) -> Result<crate::models::DownloadEngineResult, String> {
    let data_dir = get_data_dir(&app);
    let engines_dir = data_dir.join("engines");
    std::fs::create_dir_all(&engines_dir)
        .map_err(|e| format!("创建引擎目录失败: {}", e))?;

    if remote_version.file_size > 0 {
        if let Ok(available) = fs2::available_space(&engines_dir) {
            let required = remote_version.file_size as u64 * 3;
            if available < required {
                let avail_mb = available as f64 / 1024.0 / 1024.0;
                let req_mb = required as f64 / 1024.0 / 1024.0;
                return Ok(crate::models::DownloadEngineResult {
                    success: false,
                    cancelled: false,
                    engine: None,
                    error: Some(format!("磁盘空间不足，可用 {:.0}MB，需要约 {:.0}MB（包含下载文件 + 解压空间 + 余量）", avail_mb, req_mb)),
                });
            }
        }
    }

    let installed_path = match crate::engine_downloader::EngineDownloader::download_and_install(
        &app, &remote_version, engines_dir,
    ).await {
        Ok(path) => path,
        Err(e) => {
            let is_cancelled = e == "下载已取消";
            if is_cancelled {
                return Ok(crate::models::DownloadEngineResult {
                    success: false,
                    cancelled: true,
                    engine: None,
                    error: Some(e),
                });
            }
            return Err(e);
        }
    };

    let path_str = installed_path.to_string_lossy().to_string();

    let engine = match crate::engine::EngineManager::get_engine_info(&path_str) {
        Ok(e) => e,
        Err(detail) => {
            let _ = std::fs::remove_dir_all(&installed_path);
            return Ok(crate::models::DownloadEngineResult {
                success: false,
                cancelled: false,
                engine: None,
                error: Some(format!("下载的引擎文件无效: {}", detail)),
            });
        }
    };

    let mut registered_engine = engine;
    registered_engine.name = if remote_version.variant == "mono" {
        format!("Godot {} (.NET)", remote_version.version)
    } else {
        format!("Godot {}", remote_version.version)
    };

    let storage = get_storage(&app);
    let mut engines: Vec<Engine> = storage.load_or_default("engines.json");

    engines.retain(|e| e.path != registered_engine.path);

    engines.push(registered_engine.clone());
    storage.save("engines.json", &engines)
        .map_err(|e| format!("保存引擎信息失败: {}", e))?;

    log_operation(&app, "download_engine", &remote_version.version,
        &format!("已下载并注册引擎: {}", registered_engine.name));

    let _ = app.emit("engines-discovered", ());

    Ok(crate::models::DownloadEngineResult {
        success: true,
        cancelled: false,
        engine: Some(registered_engine),
        error: None,
    })
}

#[tauri::command]
pub async fn download_engine_from_url(
    app: AppHandle,
    url: String,
    engine_name: Option<String>,
) -> Result<crate::models::DownloadEngineResult, String> {
    if url.is_empty() {
        return Err("请输入下载地址".to_string());
    }
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err("请输入有效的 HTTP/HTTPS 地址".to_string());
    }

    let data_dir = get_data_dir(&app);
    let engines_dir = data_dir.join("engines");
    std::fs::create_dir_all(&engines_dir)
        .map_err(|e| format!("创建引擎目录失败: {}", e))?;

    let url_path = url.split('?').next().unwrap_or(&url);
    let file_name = url_path.split('/').last().unwrap_or("engine").to_string();

    let version_key = format!("url_{}", Uuid::new_v4());
    let variant = "standard";
    crate::engine_downloader::reset_cancel(&version_key, variant);

    let download_dir = get_data_dir(&app).join("downloads");
    std::fs::create_dir_all(&download_dir)
        .map_err(|e| format!("创建下载目录失败: {}", e))?;

    let archive_path = download_dir.join(&file_name);

    let download_result = crate::engine_downloader::EngineDownloader::download_file(
        &app, &url, &archive_path, &version_key, variant, 0,
    ).await;

    if let Err(e) = download_result {
        crate::engine_downloader::cleanup_on_error(&archive_path, false, &version_key, variant);
        return Ok(crate::models::DownloadEngineResult {
            success: false,
            cancelled: e == "下载已取消",
            engine: None,
            error: Some(e),
        });
    }

    let target_dir_name = format!("custom_{}", version_key.replace('-', "_"));
    let target_dir = engines_dir.join(&target_dir_name);

    if target_dir.exists() {
        let _ = std::fs::remove_dir_all(&target_dir);
    }
    std::fs::create_dir_all(&target_dir)
        .map_err(|e| format!("创建引擎目录失败: {}", e))?;

    let extract_result = crate::engine_downloader::EngineDownloader::extract_archive(
        &app, &version_key, variant, &archive_path, &target_dir,
    );
    let _ = std::fs::remove_file(&archive_path);

    if let Err(e) = extract_result {
        crate::engine_downloader::cleanup_on_error(&target_dir, true, &version_key, variant);
        return Ok(crate::models::DownloadEngineResult {
            success: false,
            cancelled: false,
            engine: None,
            error: Some(format!("解压引擎文件失败: {}", e)),
        });
    }

    let path_str = target_dir.to_string_lossy().to_string();
    let engine = match crate::engine::EngineManager::get_engine_info(&path_str) {
        Ok(e) => e,
        Err(detail) => {
            let _ = std::fs::remove_dir_all(&target_dir);
            return Ok(crate::models::DownloadEngineResult {
                success: false,
                cancelled: false,
                engine: None,
                error: Some(format!("下载的引擎文件无效: {}", detail)),
            });
        }
    };

    let mut registered_engine = engine;
    if let Some(name) = engine_name {
        if !name.is_empty() {
            registered_engine.name = name;
        }
    }

    let storage = get_storage(&app);
    let mut engines: Vec<Engine> = storage.load_or_default("engines.json");
    engines.retain(|e| e.path != registered_engine.path);
    engines.push(registered_engine.clone());
    storage.save("engines.json", &engines)
        .map_err(|e| format!("保存引擎信息失败: {}", e))?;

    let mut settings = load_settings(&app);
    let parent = std::path::Path::new(&path_str)
        .parent()
        .map(|p| p.to_string_lossy().to_string());
    if let Some(parent_path) = parent {
        if !settings.known_engine_paths.iter().any(|p| p.to_lowercase() == parent_path.to_lowercase()) {
            settings.known_engine_paths.push(parent_path);
            let config_storage = get_config_storage(&app);
            let _ = config_storage.save("settings.json", &settings);
        }
    }

    log_operation(&app, "download_engine_from_url", &url,
        &format!("从 URL 下载并注册引擎: {}", registered_engine.name));

    let _ = app.emit("engines-discovered", ());

    Ok(crate::models::DownloadEngineResult {
        success: true,
        cancelled: false,
        engine: Some(registered_engine),
        error: None,
    })
}

#[tauri::command]
pub fn cancel_engine_download(version: String, variant: String) -> Result<(), String> {
    crate::engine_downloader::request_cancel_download(&version, &variant);
    Ok(())
}

#[tauri::command]
pub fn get_active_downloads() -> Vec<crate::models::EngineDownloadProgress> {
    crate::engine_downloader::get_active_downloads()
}

#[tauri::command]
pub fn cleanup_download_temp(app: AppHandle) -> Result<u64, String> {
    let download_dir = get_data_dir(&app).join("downloads");
    if !download_dir.exists() {
        return Ok(0);
    }
    let mut cleaned = 0u64;
    if let Ok(entries) = fs::read_dir(&download_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Ok(metadata) = fs::metadata(&path) {
                    let modified = metadata.modified().ok();
                    let age = modified.and_then(|m| m.elapsed().ok());
                    if age.map_or(false, |d| d.as_secs() > 3600) {
                        if fs::remove_file(&path).is_ok() {
                            cleaned += 1;
                        }
                    }
                }
            }
        }
    }
    Ok(cleaned)
}

#[tauri::command]
pub async fn scan_projects(app: AppHandle, root_dirs: Vec<String>) -> Result<Vec<Project>, String> {
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

    let app_clone = app.clone();
    let all_projects = tokio::task::spawn_blocking(move || {
        ProjectScanner::scan_directories_parallel(&valid_dirs)
    }).await.map_err(|e| format!("扫描任务失败: {}", e))?
    .map_err(|e| format!("扫描失败: {}", e))?;

    let storage = get_storage(&app_clone);
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

    log_operation(&app, "scan_projects", &root_dirs.join(", "),
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
pub async fn import_project_from_git(
    app: AppHandle,
    git_url: String,
    target_dir: Option<String>,
) -> Result<Project, String> {
    if git_url.is_empty() {
        return Err("请输入 Git 仓库地址".to_string());
    }

    let repo_name = git_url
        .split('/')
        .last()
        .unwrap_or("unknown")
        .trim_end_matches(".git")
        .to_string();

    let data_dir = get_data_dir(&app);
    let clone_base = if let Some(dir) = &target_dir {
        if !dir.is_empty() {
            std::path::PathBuf::from(dir)
        } else {
            data_dir.join("projects")
        }
    } else {
        data_dir.join("projects")
    };

    std::fs::create_dir_all(&clone_base)
        .map_err(|e| format!("创建项目目录失败: {}", e))?;

    let clone_target = clone_base.join(&repo_name);

    if clone_target.exists() {
        let project_godot = clone_target.join("project.godot");
        if project_godot.exists() {
            let storage = get_storage(&app);
            let projects: Vec<Project> = storage.load_or_default("projects.json");
            let existing_path = clone_target.to_string_lossy().to_string();
            if projects.iter().any(|p| p.path == existing_path) {
                return Err("该项目已存在，请勿重复添加".to_string());
            }
            let project = ProjectScanner::parse_project(&project_godot)
                .map_err(|e| format!("解析项目失败: {}", e))?;
            let project_name = project.name.clone();
            let mut all_projects: Vec<Project> = storage.load_or_default("projects.json");
            all_projects.push(project.clone());
            storage.save("projects.json", &all_projects)
                .map_err(|e| format!("保存项目失败: {}", e))?;
            log_operation(&app, "import_project_from_git", &git_url,
                &format!("从 Git 导入项目（目录已存在）: {}", project_name));
            let _ = app.emit("projects-changed", ());
            return Ok(project);
        }
        return Err(format!("目标目录已存在但不是有效的 Godot 项目: {}", clone_target.display()));
    }

    let mut callbacks = git2::RemoteCallbacks::new();
    let app_handle_clone = app.clone();
    let git_url_for_callback = git_url.clone();
    callbacks.transfer_progress(move |progress| {
        let received = progress.received_objects();
        let total = progress.total_objects();
        let percentage = if total > 0 {
            (received as f64 / total as f64 * 100.0) as u32
        } else {
            0
        };
        let _ = app_handle_clone.emit("git-clone-progress", serde_json::json!({
            "url": git_url_for_callback,
            "received_objects": received,
            "total_objects": total,
            "percentage": percentage,
        }));
        true
    });

    let mut fetch_options = git2::FetchOptions::new();
    fetch_options.remote_callbacks(callbacks);

    let mut builder = git2::build::RepoBuilder::new();
    builder.fetch_options(fetch_options);

    if let Err(e) = builder.clone(&git_url, &clone_target) {
        let _ = std::fs::remove_dir_all(&clone_target);
        return Err(format!("克隆 Git 仓库失败: {}", e));
    }

    let project_godot = clone_target.join("project.godot");
    if !project_godot.exists() {
        let _ = std::fs::remove_dir_all(&clone_target);
        return Err("克隆成功但未找到 project.godot 文件，不是有效的 Godot 项目".to_string());
    }

    let project = ProjectScanner::parse_project(&project_godot)
        .map_err(|e| {
            let _ = std::fs::remove_dir_all(&clone_target);
            format!("解析项目失败: {}", e)
        })?;

    let project_name = project.name.clone();
    let storage = get_storage(&app);
    let mut all_projects: Vec<Project> = storage.load_or_default("projects.json");

    if all_projects.iter().any(|p| p.path == project.path) {
        let _ = std::fs::remove_dir_all(&clone_target);
        return Err("该项目已存在，请勿重复添加".to_string());
    }

    all_projects.push(project.clone());
    storage.save("projects.json", &all_projects)
        .map_err(|e| format!("保存项目失败: {}", e))?;

    log_operation(&app, "import_project_from_git", &git_url,
        &format!("从 Git 导入项目: {}", project_name));

    let _ = app.emit("projects-changed", ());

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

    let mut bindings: Vec<ProjectBinding> = storage.load_or_default("bindings.json");
    let had_bindings = bindings.iter().any(|b| b.project_id == project_id);
    bindings.retain(|b| b.project_id != project_id);
    if had_bindings {
        storage.save("bindings.json", &bindings)
            .map_err(|e| format!("保存绑定列表失败: {}", e))?;
    }

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
    upsert_plugin(&app, &new_plugin, "import_plugin", &path)
}

#[tauri::command]
pub fn import_plugin_from_git(app: AppHandle, url: String) -> Result<Plugin, String> {
    if url.is_empty() {
        return Err("请输入 Git 仓库地址".to_string());
    }

    let manager = get_plugin_manager(&app);
    let new_plugin = manager.import_from_git(&url, &app)
        .map_err(|e| format!("从 Git 导入插件失败: {}，请检查仓库地址是否正确", e))?;
    upsert_plugin(&app, &new_plugin, "import_plugin_git", &url)
}

#[tauri::command]
pub fn import_plugin_from_url(app: AppHandle, url: String) -> Result<Plugin, String> {
    if url.is_empty() {
        return Err("请输入下载地址".to_string());
    }
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err("请输入有效的 HTTP/HTTPS 地址".to_string());
    }

    let manager = get_plugin_manager(&app);
    let new_plugin = manager.import_from_url(&url, &app)
        .map_err(|e| format!("从 URL 导入插件失败: {}", e))?;
    upsert_plugin(&app, &new_plugin, "import_plugin_url", &url)
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
    subdirectory: String,
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

    let binding = ProjectBinding::new(project_id.clone(), plugin_id, version_id, unit_id, mount_path, subdirectory);

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
        let plugin_path = std::path::Path::new(&project.path).join(&mount_path);

        if plugin_path.exists() {
            let metadata = std::fs::symlink_metadata(&plugin_path);
            let is_link = metadata.as_ref().map(|m| m.file_type().is_symlink()).unwrap_or(false);
            let is_junction = {
                #[cfg(windows)]
                {
                    use std::os::windows::fs::MetadataExt;
                    metadata.as_ref().map(|m| m.file_attributes() & 0x400 != 0).unwrap_or(false)
                }
                #[cfg(not(windows))]
                {
                    false
                }
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
        let result = ApplyResult {
            success: true,
            created: vec![],
            removed: vec![],
            errors: vec![],
        };
        log_operation(&app, "apply_changes", &project_id, "无绑定，跳过应用");
        return Ok(result);
    }

    let settings = load_settings(&app);
    let linker = Linker::new(settings.mount_strategy);

    let data_dir = get_data_dir(&app);
    let plugin_base_path = data_dir.join("plugins");

    let addons_dir = std::path::Path::new(&project.path).join("addons");
    if addons_dir.exists() {
        let backup_dir = data_dir.join("backups").join(&project.name);
        if let Err(e) = std::fs::create_dir_all(&backup_dir) {
            eprintln!("Failed to create backup dir: {}", e);
        } else {
            let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
            let backup_file = backup_dir.join(format!("addons_backup_{}.zip", timestamp));
            if let Err(e) = backup_addons_dir(&addons_dir, &backup_file) {
                eprintln!("Failed to backup addons: {}", e);
            } else {
                cleanup_old_backups(&backup_dir, 5);
            }
        }
    }

    let applied_dir = data_dir.join("applied_bindings");
    let applied_file = applied_dir.join(format!("{}.json", project_id));
    let current_bindings: Vec<ProjectBinding> = if applied_file.exists() {
        let applied_storage = Storage::new(applied_dir.clone());
        applied_storage.load_or_default::<Vec<ProjectBinding>>(&format!("{}.json", project_id))
    } else {
        Vec::new()
    };

    let result = linker.apply_bindings(
        &project.path,
        &current_bindings,
        &desired_bindings,
        &plugin_base_path.to_string_lossy()
    ).map_err(|e| format!("应用变更失败: {}", e))?;

    if result.success {
        if let Err(e) = std::fs::create_dir_all(&applied_dir) {
            eprintln!("Failed to create applied_bindings dir: {}", e);
        }
        let applied_storage = Storage::new(applied_dir);
        if let Err(e) = applied_storage.save(&format!("{}.json", project_id), &desired_bindings) {
            eprintln!("Failed to save applied bindings: {}", e);
        }
    }

    log_operation(&app, "apply_changes", &project_id,
        &format!("应用变更完成: 创建 {} 项, 移除 {} 项, 错误 {} 项",
            result.created.len(), result.removed.len(), result.errors.len()));

    Ok(result)
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
pub fn get_project_bindings(app: AppHandle, project_id: String) -> Result<Vec<ProjectBinding>, String> {
    let storage = get_storage(&app);
    let bindings: Vec<ProjectBinding> = storage.load_or_default("bindings.json");

    Ok(bindings.into_iter()
        .filter(|b| b.project_id == project_id)
        .collect())
}

#[tauri::command]
pub async fn scan_project_plugins(app: AppHandle) -> Result<Vec<crate::models::ScannedPlugin>, String> {
    let storage = get_storage(&app);
    let projects: Vec<Project> = storage.load_or_default("projects.json");

    if projects.is_empty() {
        return Err("暂无项目，请先添加项目".to_string());
    }

    let manager = get_plugin_manager(&app);
    let scanned_plugins = tokio::task::spawn_blocking(move || {
        manager.scan_project_plugins(&projects)
    }).await.map_err(|e| format!("扫描任务失败: {}", e))?
    .map_err(|e| format!("扫描项目插件失败: {}", e))?;

    Ok(scanned_plugins)
}

#[tauri::command]
pub async fn import_plugins_from_projects(app: AppHandle, mode: Option<String>) -> Result<Vec<Plugin>, String> {
    let import_mode = mode.unwrap_or_else(|| "copy".to_string());
    if !["copy", "move", "reference"].contains(&import_mode.as_str()) {
        return Err("无效的导入模式，支持: copy, move, reference".to_string());
    }

    let storage = get_storage(&app);
    let projects: Vec<Project> = storage.load_or_default("projects.json");

    if projects.is_empty() {
        return Err("暂无项目，请先添加项目".to_string());
    }

    let mut plugins: Vec<Plugin> = storage.load_or_default("plugins.json");

    let app_for_scan = app.clone();
    let projects_clone = projects.clone();
    let scanned_plugins = tokio::task::spawn_blocking(move || {
        let manager = get_plugin_manager(&app_for_scan);
        manager.scan_project_plugins(&projects_clone)
    }).await.map_err(|e| format!("扫描任务失败: {}", e))?
    .map_err(|e| format!("扫描项目插件失败: {}", e))?;

    if scanned_plugins.is_empty() {
        return Err("未在项目中发现可导入的插件".to_string());
    }

    let imported_plugins;
    let seen_names: std::collections::HashSet<String> = plugins.iter()
        .map(|p| p.name.to_lowercase())
        .collect();

    let import_mode_clone = import_mode.clone();
    let scanned_plugins_clone = scanned_plugins.clone();
    let plugins_clone = plugins.clone();
    let projects_clone = projects.clone();
    let manager_clone = get_plugin_manager(&app);
    let storage_clone = get_storage(&app);

    let import_result = tokio::task::spawn_blocking(move || {
        let mut local_plugins = plugins_clone;
        let mut local_imported = Vec::new();
        let mut local_seen = seen_names;
        let mut pending_bindings: Vec<ProjectBinding> = Vec::new();

        for scanned in &scanned_plugins_clone {
            let path_str = scanned.path.clone();
            let plugin_name_lower = scanned.plugin_name.to_lowercase();

            let already_imported = local_plugins.iter()
                .any(|p| p.source.url == path_str || p.name.to_lowercase() == plugin_name_lower);

            if already_imported {
                continue;
            }

            match import_mode_clone.as_str() {
                "copy" => {
                    match manager_clone.import_from_local(&path_str) {
                        Ok(plugin) => {
                            local_seen.insert(plugin.name.to_lowercase());
                            local_imported.push(plugin.clone());
                            local_plugins.push(plugin);
                        }
                        Err(e) => eprintln!("Failed to import plugin from {}: {}", path_str, e),
                    }
                }
                "move" => {
                    match manager_clone.import_from_local(&path_str) {
                        Ok(mut plugin) => {
                            let source_path = Path::new(&path_str);
                            if let Ok(metadata) = fs::symlink_metadata(source_path) {
                                if metadata.file_type().is_symlink() || is_junction_path(source_path) {
                                    if let Ok(link_target) = fs::read_link(source_path) {
                                        plugin.source.url = link_target.to_string_lossy().to_string();
                                    }
                                }
                            }
                            let plugin_id = plugin.plugin_id.clone();
                            let version = plugin.versions.first();
                            let version_id = version.map(|v| v.version_id.clone()).unwrap_or_default();
                            let payload_path = version.map(|v| v.path.clone()).unwrap_or_default();

                            if let Err(e) = replace_with_symlink(source_path, &payload_path) {
                                eprintln!("Warning: failed to replace with symlink: {}", e);
                            }

                            local_seen.insert(plugin.name.to_lowercase());
                            local_imported.push(plugin.clone());
                            local_plugins.push(plugin);

                            let project = projects_clone.iter().find(|p| path_str.starts_with(&p.path));
                            if let Some(proj) = project {
                                let mount_path = path_str.replace(&format!("{}/", proj.path.replace('\\', "/")), "")
                                    .replace(&format!("{}\\", proj.path), "");
                                pending_bindings.push(ProjectBinding::new(
                                    proj.project_id.clone(),
                                    plugin_id,
                                    version_id,
                                    String::new(),
                                    mount_path,
                                    String::new(),
                                ));
                            }
                        }
                        Err(e) => eprintln!("Failed to import plugin from {}: {}", path_str, e),
                    }
                }
                "reference" => {
                    let source = Path::new(&path_str);
                    let plugin_name = scanned.plugin_name.clone();
                    let plugin_source = PluginSource {
                        source_type: SourceType::Local,
                        url: path_str.clone(),
                        imported_at: chrono::Utc::now(),
                    };
                    let mut plugin = Plugin::new(plugin_name.clone(), plugin_source);
                    plugin.content_hash = compute_dir_hash(source).unwrap_or_default();

                    match manager_clone.parse_plugin_units(source) {
                        Ok(units) => {
                            let (unit_version, unit_name, unit_description, unit_author) =
                                if let Some(first_unit) = units.first() {
                                    (
                                        if first_unit.version.is_empty() { "1.0.0".to_string() } else { first_unit.version.clone() },
                                        if first_unit.name.is_empty() { plugin_name } else { first_unit.name.clone() },
                                        first_unit.description.clone(),
                                        first_unit.author.clone(),
                                    )
                                } else {
                                    ("1.0.0".to_string(), plugin_name, String::new(), String::new())
                                };
                            let version_id = Uuid::new_v4().to_string();
                            let plugin_version = PluginVersion {
                                version_id,
                                version: unit_version,
                                path: path_str.clone(),
                                created_at: chrono::Utc::now(),
                                units,
                            };
                            plugin.versions.push(plugin_version);
                            plugin.compatibility = manager_clone.detect_compatibility(source);
                            plugin.name = unit_name;
                            plugin.description = unit_description;
                            plugin.author = unit_author;
                        }
                        Err(_) => {
                            let version_id = Uuid::new_v4().to_string();
                            let plugin_version = PluginVersion {
                                version_id,
                                version: "1.0.0".to_string(),
                                path: path_str.clone(),
                                created_at: chrono::Utc::now(),
                                units: Vec::new(),
                            };
                            plugin.versions.push(plugin_version);
                        }
                    }

                    local_seen.insert(plugin.name.to_lowercase());
                    local_imported.push(plugin.clone());
                    local_plugins.push(plugin);
                }
                _ => {}
            }
        }

        if !pending_bindings.is_empty() {
            let mut bindings: Vec<ProjectBinding> = storage_clone.load_or_default("bindings.json");
            bindings.extend(pending_bindings);
            let _ = storage_clone.save("bindings.json", &bindings);
        }

        (local_plugins, local_imported)
    }).await.map_err(|e| format!("导入任务失败: {}", e))?;

    let (updated_plugins, imported) = import_result;
    plugins = updated_plugins;
    imported_plugins = imported;

    storage.save("plugins.json", &plugins)
        .map_err(|e| format!("保存插件列表失败: {}", e))?;

    log_operation(&app, "import_plugins_from_projects", "", 
        &format!("从项目以 {} 模式导入了 {} 个插件", import_mode, imported_plugins.len()));

    Ok(imported_plugins)
}

fn is_junction_path(path: &Path) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if let Ok(metadata) = fs::symlink_metadata(path) {
            const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
            return metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0;
        }
    }
    let _ = path;
    false
}

fn replace_with_symlink(original_path: &Path, repo_payload_path: &str) -> Result<(), String> {
    let payload = Path::new(repo_payload_path);
    if !payload.exists() {
        return Err("仓库中的插件路径不存在".to_string());
    }
    let parent = original_path.parent()
        .ok_or_else(|| "无法获取父目录".to_string())?;
    let temp_name = format!(".harbor-tmp-{}", uuid::Uuid::new_v4());
    let temp_path = parent.join(&temp_name);
    fs::rename(original_path, &temp_path)
        .map_err(|e| format!("重命名原始目录失败: {}", e))?;

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(payload, original_path)
            .map_err(|e| {
                let _ = fs::rename(&temp_path, original_path);
                format!("创建符号链接失败: {}", e)
            })?;
    }

    #[cfg(windows)]
    {
        match std::os::windows::fs::symlink_dir(payload, original_path) {
            Ok(_) => {}
            Err(_) => {
                let output = no_window_cmd("cmd")
                    .args(&["/C", "mklink", "/J"])
                    .arg(original_path)
                    .arg(payload)
                    .output()
                    .map_err(|e| format!("执行mklink失败: {}", e))?;
                if !output.status.success() {
                    let _ = fs::rename(&temp_path, original_path);
                    return Err(format!("创建Junction失败: {}", String::from_utf8_lossy(&output.stderr)));
                }
            }
        }
    }

    fs::remove_dir_all(&temp_path)
        .map_err(|e| format!("删除临时目录失败: {}", e))?;

    Ok(())
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
pub async fn get_plugin_storage_stats(app: AppHandle, plugin_id: String) -> Result<PluginStorageStats, String> {
    let storage = get_storage(&app);
    let plugins: Vec<Plugin> = storage.load_or_default("plugins.json");
    let plugin = plugins.iter().find(|p| p.plugin_id == plugin_id)
        .ok_or("未找到指定插件".to_string())?;

    let plugin_dir = get_data_dir(&app).join("plugins").join(&plugin_id);
    let total_size_bytes = tokio::task::spawn_blocking(move || {
        if plugin_dir.exists() { dir_size(&plugin_dir) } else { 0 }
    }).await.map_err(|e| format!("计算存储大小失败: {}", e))?;

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
pub fn check_binding_health(app: AppHandle, project_id: String) -> Result<Vec<ProjectBinding>, String> {
    let storage = get_storage(&app);
    let bindings: Vec<ProjectBinding> = storage.load_or_default("bindings.json");
    let project_bindings: Vec<ProjectBinding> = bindings.into_iter()
        .filter(|b| b.project_id == project_id)
        .collect();

    let projects: Vec<Project> = storage.load_or_default("projects.json");
    let project = projects.iter().find(|p| p.project_id == project_id);
    if project.is_none() {
        return Err("未找到指定项目".to_string());
    }
    let project = project.unwrap();

    let plugin_base_path = get_data_dir(&app).join("plugins");

    let mut results = Vec::new();
    for mut binding in project_bindings {
        let addons_path = Path::new(&project.path).join(&binding.mount_path);
        let is_healthy = if addons_path.exists() {
            let metadata = fs::symlink_metadata(&addons_path);
            match metadata {
                Ok(meta) => {
                    if meta.file_type().is_symlink() {
                        let link_target = fs::read_link(&addons_path);
                        match link_target {
                            Ok(target) => target.exists(),
                            Err(_) => false,
                        }
                    } else if is_junction_path(&addons_path) {
                        let payload_path = plugin_base_path
                            .join(&binding.plugin_id)
                            .join(&binding.version_id)
                            .join("payload");
                        let source_path = if binding.subdirectory.is_empty() {
                            payload_path
                        } else {
                            payload_path.join(&binding.subdirectory)
                        };
                        source_path.exists()
                    } else {
                        true
                    }
                }
                Err(_) => false,
            }
        } else {
            false
        };
        binding.is_healthy = Some(is_healthy);
        results.push(binding);
    }

    Ok(results)
}

#[tauri::command]
pub fn repair_binding(app: AppHandle, project_id: String, plugin_id: String) -> Result<(), String> {
    let storage = get_storage(&app);
    let bindings: Vec<ProjectBinding> = storage.load_or_default("bindings.json");
    let binding = bindings.iter()
        .find(|b| b.project_id == project_id && b.plugin_id == plugin_id)
        .cloned();
    if binding.is_none() {
        return Err("未找到指定的绑定关系".to_string());
    }
    let binding = binding.unwrap();

    let projects: Vec<Project> = storage.load_or_default("projects.json");
    let project = projects.iter().find(|p| p.project_id == project_id);
    if project.is_none() {
        return Err("未找到指定项目".to_string());
    }
    let project = project.unwrap();

    let settings = load_settings(&app);
    let linker = Linker::new(settings.mount_strategy.clone());
    let plugin_base_path = get_data_dir(&app).join("plugins");

    let current_bindings: Vec<ProjectBinding> = Vec::new();
    let desired_bindings = vec![binding.clone()];

    let result = linker.apply_bindings(
        &project.path,
        &current_bindings,
        &desired_bindings,
        &plugin_base_path.to_string_lossy()
    ).map_err(|e| format!("修复绑定失败: {}", e))?;

    if !result.success {
        return Err(format!("修复绑定失败: {}", result.errors.join(", ")));
    }

    log_operation(&app, "repair_binding", &project_id,
        &format!("已修复插件 {} 的符号链接", plugin_id));

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateCheckResult {
    pub is_duplicate: bool,
    pub duplicate_plugin_id: Option<String>,
    pub duplicate_plugin_name: Option<String>,
    pub content_hash: String,
}

#[tauri::command]
pub fn check_plugin_duplicate(app: AppHandle, path: String) -> Result<DuplicateCheckResult, String> {
    let source = Path::new(&path);
    if !source.exists() {
        return Err("指定路径不存在".to_string());
    }
    let content_hash = crate::models::compute_dir_hash(source)
        .map_err(|e| format!("计算内容hash失败: {}", e))?;

    let storage = get_storage(&app);
    let plugins: Vec<Plugin> = storage.load_or_default("plugins.json");

    let duplicate = plugins.iter()
        .find(|p| !p.content_hash.is_empty() && p.content_hash == content_hash);

    Ok(DuplicateCheckResult {
        is_duplicate: duplicate.is_some(),
        duplicate_plugin_id: duplicate.map(|p| p.plugin_id.clone()),
        duplicate_plugin_name: duplicate.map(|p| p.name.clone()),
        content_hash,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotalStorageStats {
    pub total_plugins: usize,
    pub total_versions: usize,
    pub total_bindings: usize,
    pub total_size_bytes: u64,
    pub total_size_display: String,
    pub orphaned_size_bytes: u64,
    pub orphaned_size_display: String,
    pub duplicate_hash_count: usize,
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
pub fn update_git_plugin(app: AppHandle, plugin_id: String) -> Result<Plugin, String> {
    let storage = get_storage(&app);
    let mut plugins: Vec<Plugin> = storage.load_or_default("plugins.json");

    let plugin = plugins.iter()
        .find(|p| p.plugin_id == plugin_id)
        .ok_or("未找到指定插件".to_string())?;

    if plugin.source.source_type != SourceType::Git {
        return Err("仅支持更新Git来源的插件".to_string());
    }

    let git_url = plugin.source.url.clone();
    let old_version = plugin.versions.first().map(|v| v.version.clone()).unwrap_or_default();
    let plugin_name = plugin.name.clone();
    let manager = get_plugin_manager(&app);
    let updated_plugin = manager.import_from_git(&git_url, &app)
        .map_err(|e| format!("更新Git插件失败: {}", e))?;

    let idx = plugins.iter().position(|p| p.plugin_id == plugin_id)
        .ok_or("未找到指定插件".to_string())?;

    plugins[idx].versions.extend(updated_plugin.versions);
    if !updated_plugin.content_hash.is_empty() {
        plugins[idx].content_hash = updated_plugin.content_hash;
    }
    plugins[idx].updated_at = chrono::Utc::now();

    let result = plugins[idx].clone();
    storage.save("plugins.json", &plugins)
        .map_err(|e| format!("保存插件列表失败: {}", e))?;

    log_operation(&app, "update_git_plugin", &plugin_id, &format!("已更新Git插件: {}", result.name));
    record_update_history(&app, "plugin", &plugin_name, &old_version, &result.versions.last().map(|v| v.version.clone()).unwrap_or_default(), "success", "");
    Ok(result)
}

const APP_GITHUB_OWNER: &str = "odayou";
const APP_GITHUB_REPO: &str = "GodotHarbor";

#[tauri::command]
pub async fn check_app_update(app: AppHandle, force_refresh: Option<bool>) -> Result<Option<AppUpdateInfo>, String> {
    let force = force_refresh.unwrap_or(false);
    let cache_version: u32 = 1;

    let cache_dir = get_data_dir(&app).join("cache");
    let cache_file = cache_dir.join("app_update.json");

    if !force && cache_file.exists() {
        if let Ok(content) = fs::read_to_string(&cache_file) {
            if let Ok(cached) = serde_json::from_str::<crate::models::CachedAppUpdate>(&content) {
                if cached.cache_version == cache_version {
                    if let Ok(cached_time) = chrono::DateTime::parse_from_rfc3339(&cached.cached_at) {
                        let elapsed = chrono::Utc::now().signed_duration_since(cached_time.with_timezone(&chrono::Utc));
                        if elapsed.num_minutes() < 30 {
                            return Ok(cached.update_info);
                        }
                    }
                } else {
                    let _ = fs::remove_file(&cache_file);
                }
            }
        }
    }

    let current_version = app.config().version.clone().unwrap_or_default();

    let mut settings = load_settings(&app);
    if !settings.skipped_app_version.is_empty() {
        let skipped = semver::Version::parse(settings.skipped_app_version.trim_start_matches('v')).ok();
        let current = semver::Version::parse(current_version.trim_start_matches('v')).ok();
        if let (Some(s), Some(c)) = (skipped, current) {
            if s <= c {
                settings.skipped_app_version = String::new();
                let _ = save_settings_to_config(&app, &settings);
            }
        }
    }

    let client = create_http_client(Some(std::time::Duration::from_secs(15)))?;

    let github_base = crate::utils::get_github_api_base(&app);
    let api_url = format!(
        "{}/repos/{}/{}/releases/latest",
        github_base, APP_GITHUB_OWNER, APP_GITHUB_REPO
    );

    let resp = client.get(&api_url).send().await
        .map_err(|e| format!("检查应用更新失败: {}", e))?;

    if !resp.status().is_success() {
        return Ok(None);
    }

    let json: serde_json::Value = resp.json().await
        .map_err(|e| format!("解析更新信息失败: {}", e))?;

    let tag = json.get("tag_name").and_then(|t| t.as_str()).unwrap_or("");
    let latest_version = tag.trim_start_matches('v').to_string();

    let current_semver = semver::Version::parse(current_version.trim_start_matches('v')).ok();
    let latest_semver = semver::Version::parse(&latest_version).ok();

    if let (Some(cur), Some(lat)) = (&current_semver, &latest_semver) {
        if lat <= cur {
            return Ok(None);
        }
    } else if latest_version == current_version {
        return Ok(None);
    }

    if !settings.skipped_app_version.is_empty() {
        if let Some(skipped) = semver::Version::parse(settings.skipped_app_version.trim_start_matches('v')).ok() {
            if let Some(lat) = &latest_semver {
                if &skipped >= lat {
                    return Ok(None);
                }
            }
        }
    }

    let release_notes = json.get("body").and_then(|b| b.as_str()).unwrap_or("").to_string();
    let pub_date = json.get("published_at").and_then(|d| d.as_str()).unwrap_or("").to_string();

    let target_ext = if cfg!(target_os = "windows") { ".nsis.zip" } else if cfg!(target_os = "macos") { ".app.tar.gz" } else { ".AppImage.tar.gz" };
    let mut download_url = None;
    let mut download_size = None;

    if let Some(assets) = json.get("assets").and_then(|a| a.as_array()) {
        for asset in assets {
            let name = asset.get("name").and_then(|n| n.as_str()).unwrap_or("");
            if name.ends_with(target_ext) {
                download_url = asset.get("browser_download_url").and_then(|u| u.as_str()).map(|s| s.to_string());
                download_size = asset.get("size").and_then(|s| s.as_u64());
                break;
            }
        }
        if download_url.is_none() {
            for asset in assets {
                let name = asset.get("name").and_then(|n| n.as_str()).unwrap_or("");
                if name.ends_with(".exe") || name.ends_with(".msi") {
                    download_url = asset.get("browser_download_url").and_then(|u| u.as_str()).map(|s| s.to_string());
                    download_size = asset.get("size").and_then(|s| s.as_u64());
                    break;
                }
            }
        }
    }

    let result = Ok(Some(AppUpdateInfo {
        current_version,
        latest_version,
        release_notes,
        pub_date,
        download_size,
        is_hot_update: false,
        download_url,
    }));

    if let Ok(ref info) = result {
        let _ = fs::create_dir_all(&cache_dir);
        let cached = crate::models::CachedAppUpdate {
            cache_version,
            cached_at: chrono::Utc::now().to_rfc3339(),
            update_info: info.clone(),
        };
        if let Ok(json) = serde_json::to_string_pretty(&cached) {
            let _ = fs::write(&cache_file, json);
        }
    }

    result
}

#[tauri::command]
pub async fn install_app_update(app: AppHandle) -> Result<(), String> {
    let update_info = check_app_update(app.clone(), Some(true)).await
        .map_err(|e| format!("检查更新失败: {}", e))?
        .ok_or("没有可用的更新".to_string())?;

    let download_url = update_info.download_url.clone()
        .ok_or("未找到下载链接".to_string())?;

    let download_url = {
        let storage = get_storage(&app);
        let settings: Settings = storage.load_or_default("settings.json");
        if !settings.github_api_proxy.is_empty() {
            download_url.replace("https://github.com/odayou/GodotHarbor/releases/download", 
                &format!("https://gitee.com/odayou/godot-harbor/releases/download"))
        } else {
            download_url
        }
    };

    let temp_dir = std::env::temp_dir().join("godot-harbor-update");
    fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("创建临时目录失败: {}", e))?;

    let file_name = download_url.split('/').last().unwrap_or("update.exe").to_string();
    let file_path = temp_dir.join(&file_name);

    let already_downloaded = if file_path.exists() {
        if let Ok(metadata) = fs::metadata(&file_path) {
            if let Some(expected_size) = update_info.download_size {
                metadata.len() == expected_size
            } else {
                metadata.len() > 0
            }
        } else {
            false
        }
    } else {
        false
    };

    if !already_downloaded {
        let client = create_http_client(None)?;

        let _ = app.emit("app-update-progress", serde_json::json!({
            "stage": "downloading",
            "progress": 0,
            "message": "正在下载更新..."
        }));

        let resp = client.get(&download_url).send().await
            .map_err(|e| format!("下载更新失败: {}", e))?;

        let total_size = resp.content_length();

        let mut file = fs::File::create(&file_path)
            .map_err(|e| format!("创建文件失败: {}", e))?;

        let mut downloaded: u64 = 0;
        let mut stream = resp.bytes_stream();
        use futures::StreamExt;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| format!("下载数据失败: {}", e))?;
            file.write_all(&chunk)
                .map_err(|e| format!("写入文件失败: {}", e))?;
            downloaded += chunk.len() as u64;

            let progress = if let Some(total) = total_size {
                ((downloaded as f64 / total as f64) * 100.0) as u32
            } else {
                0
            };

            let _ = app.emit("app-update-progress", serde_json::json!({
                "stage": "downloading",
                "progress": progress.min(100),
                "message": format!("下载中... {}%", progress.min(100))
            }));
        }
    } else {
        let _ = app.emit("app-update-progress", serde_json::json!({
            "stage": "installing",
            "progress": 100,
            "message": "安装包已就绪，正在启动安装程序..."
        }));
    }

    let _ = app.emit("app-update-progress", serde_json::json!({
        "stage": "installing",
        "progress": 100,
        "message": "下载完成，正在启动安装程序..."
    }));

    let data_dir = get_data_dir(&app);
    let hot_update_dir = data_dir.join("hot_updates");
    if hot_update_dir.exists() {
        let _ = fs::remove_dir_all(&hot_update_dir);
    }
    let overlay_dir = data_dir.join("hotupdate_overlay");
    if overlay_dir.exists() {
        let _ = fs::remove_dir_all(&overlay_dir);
    }

    if cfg!(target_os = "windows") {
        if file_name.ends_with(".nsis.zip") {
            let extract_dir = temp_dir.join("nsis_extract");
            let _ = fs::create_dir_all(&extract_dir);
            let extract_result = no_window_cmd("powershell")
                .args(["-NoProfile", "-Command", &format!(
                    "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
                    file_path.display(), extract_dir.display()
                )])
                .output();

            if let Err(e) = extract_result {
                return Err(format!("解压更新包失败: {}", e));
            }

            let installer = walk_dir_for_exe(&extract_dir);
            if let Some(installer) = installer {
                no_window_cmd(&installer)
                    .args(&["/S", "--force-run"])
                    .spawn()
                    .map_err(|e| format!("启动安装程序失败: {}", e))?;
            } else {
                open_file_in_os(&file_path)?;
            }
        } else {
            no_window_cmd(&file_path)
                .spawn()
                .map_err(|e| format!("启动安装程序失败: {}", e))?;
        }
    } else if cfg!(target_os = "macos") {
        let app_dir = std::env::current_exe()
            .ok()
            .and_then(|e| e.parent().and_then(|p| p.parent().map(|pp| pp.to_path_buf())))
            .unwrap_or_else(|| PathBuf::from("/Applications"));

        let _ = app.emit("app-update-progress", serde_json::json!({
            "stage": "installing",
            "progress": 100,
            "message": "正在解压并安装更新..."
        }));

        let extract_result = std::process::Command::new("tar")
            .args(["-xzf", &file_path.to_string_lossy(), "-C", &app_dir.to_string_lossy()])
            .output();

        if let Err(e) = extract_result {
            return Err(format!("解压更新包失败: {}", e));
        }

        let app_name = "Godot Harbor.app";
        std::process::Command::new("open")
            .arg(app_dir.join(app_name))
            .spawn()
            .map_err(|e| format!("启动应用失败: {}", e))?;
    } else {
        let _ = app.emit("app-update-progress", serde_json::json!({
            "stage": "installing",
            "progress": 100,
            "message": "正在解压并安装更新..."
        }));

        let extract_dir = temp_dir.join("appimage_extract");
        let _ = fs::create_dir_all(&extract_dir);

        let extract_result = std::process::Command::new("tar")
            .args(["-xzf", &file_path.to_string_lossy(), "-C", &extract_dir.to_string_lossy()])
            .output();

        if let Err(e) = extract_result {
            return Err(format!("解压更新包失败: {}", e));
        }

        let appimage = walk_dir_for_appimage(&extract_dir);
        if let Some(appimage_path) = appimage {
            let current_exe = std::env::current_exe().unwrap_or_default();
            let install_dir = current_exe.parent().unwrap_or(Path::new("/usr/bin"));
            let dest = install_dir.join(appimage_path.file_name().unwrap_or_default());
            let _ = fs::copy(&appimage_path, &dest);

            std::process::Command::new("chmod")
                .args(["+x", &dest.to_string_lossy()])
                .output().ok();

            std::process::Command::new(&dest)
                .spawn()
                .map_err(|e| format!("启动应用失败: {}", e))?;
        } else {
            open_file_in_os(&file_path)?;
        }
    }

    let _ = app.emit("app-update-progress", serde_json::json!({
        "stage": "complete",
        "progress": 100,
        "message": "安装程序已启动，即将重启..."
    }));

    record_update_history(&app, "app", "Godot Harbor", &update_info.current_version, &update_info.latest_version, "success", "安装程序已启动");

    app.exit(0);
    Ok(())
}

fn walk_dir_for_exe(dir: &Path) -> Option<PathBuf> {
    for entry in fs::read_dir(dir).ok()? {
        let entry = entry.ok()?;
        let path = entry.path();
        if path.is_dir() {
            if let Some(exe) = walk_dir_for_exe(&path) {
                return Some(exe);
            }
        } else if path.extension().map(|e| e == "exe").unwrap_or(false) {
            return Some(path);
        }
    }
    None
}

#[cfg(target_os = "linux")]
fn walk_dir_for_appimage(dir: &Path) -> Option<PathBuf> {
    for entry in fs::read_dir(dir).ok()? {
        let entry = entry.ok()?;
        let path = entry.path();
        if path.is_dir() {
            if let Some(f) = walk_dir_for_appimage(&path) {
                return Some(f);
            }
        } else if path.to_string_lossy().ends_with(".AppImage") {
            return Some(path);
        }
    }
    None
}

#[cfg(not(target_os = "linux"))]
fn walk_dir_for_appimage(_dir: &Path) -> Option<PathBuf> {
    None
}

fn open_file_in_os(path: &Path) -> Result<(), String> {
    if cfg!(target_os = "windows") {
        no_window_cmd("explorer")
            .arg(path)
            .spawn()
            .map_err(|e| format!("打开文件失败: {}", e))?;
    } else if cfg!(target_os = "macos") {
        std::process::Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("打开文件失败: {}", e))?;
    } else {
        std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("打开文件失败: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
pub fn batch_update_plugins(app: AppHandle, plugin_ids: Vec<String>) -> Result<BatchResult, String> {
    let storage = get_storage(&app);
    let mut plugins: Vec<Plugin> = storage.load_or_default("plugins.json");
    let manager = get_plugin_manager(&app);

    let mut success_count = 0usize;
    let mut failed_count = 0usize;
    let mut errors = Vec::new();
    let mut dirty = false;

    for plugin_id in &plugin_ids {
        let plugin = match plugins.iter().find(|p| p.plugin_id == *plugin_id) {
            Some(p) => p.clone(),
            None => {
                failed_count += 1;
                errors.push(format!("未找到插件: {}", plugin_id));
                continue;
            }
        };

        if plugin.source.source_type != SourceType::Git {
            failed_count += 1;
            errors.push(format!("插件 {} 非Git来源，不支持自动更新", plugin.name));
            continue;
        }

        let git_url = plugin.source.url.clone();
        let old_version = plugin.versions.first().map(|v| v.version.clone()).unwrap_or_default();
        match manager.import_from_git(&git_url, &app) {
            Ok(updated) => {
                let new_version = updated.versions.last().map(|v| v.version.clone()).unwrap_or_default();
                if let Some(idx) = plugins.iter().position(|p| p.plugin_id == *plugin_id) {
                    plugins[idx].versions.extend(updated.versions);
                    if !updated.content_hash.is_empty() {
                        plugins[idx].content_hash = updated.content_hash;
                    }
                    plugins[idx].updated_at = chrono::Utc::now();
                    dirty = true;
                }
                success_count += 1;
                record_update_history(&app, "plugin", &plugin.name, &old_version, &new_version, "success", "");
                let _ = app.emit("plugin-update-progress", serde_json::json!({
                    "plugin_id": plugin_id,
                    "stage": "complete",
                    "progress": 100,
                    "message": format!("插件 {} 更新完成", plugin.name)
                }));
            }
            Err(e) => {
                failed_count += 1;
                record_update_history(&app, "plugin", &plugin.name, &old_version, "", "failed", &format!("更新失败: {}", e));
                errors.push(format!("更新插件 {} 失败: {}", plugin.name, e));
                let _ = app.emit("plugin-update-progress", serde_json::json!({
                    "plugin_id": plugin_id,
                    "stage": "error",
                    "progress": 0,
                    "message": format!("更新失败: {}", e)
                }));
            }
        }
    }

    if dirty {
        storage.save("plugins.json", &plugins)
            .map_err(|e| format!("保存插件列表失败: {}", e))?;
    }

    log_operation(&app, "batch_update_plugins", "", 
        &format!("批量更新插件: 成功 {}, 失败 {}", success_count, failed_count));

    Ok(BatchResult { success_count, failed_count, errors })
}

#[tauri::command]
pub fn skip_app_version(app: AppHandle, version: String) -> Result<(), String> {
    let mut settings = load_settings(&app);
    settings.skipped_app_version = version;
    save_settings_to_config(&app, &settings)
        .map_err(|e| format!("保存设置失败: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn check_all_updates(app: AppHandle, force_refresh: Option<bool>) -> Result<UpdateCheckResult, String> {
    let force = force_refresh.unwrap_or(false);

    let cache_dir = get_data_dir(&app).join("cache");
    let rate_limit_file = cache_dir.join("last_update_check.txt");

    if !force && rate_limit_file.exists() {
        if let Ok(content) = fs::read_to_string(&rate_limit_file) {
            if let Ok(last_time) = chrono::DateTime::parse_from_rfc3339(content.trim()) {
                let elapsed = chrono::Utc::now().signed_duration_since(last_time.with_timezone(&chrono::Utc));
                if elapsed.num_minutes() < 5 {
                    return Err("更新检查过于频繁，请稍后再试".to_string());
                }
            }
        }
    }

    let _ = fs::create_dir_all(&cache_dir);
    let _ = fs::write(&rate_limit_file, chrono::Utc::now().to_rfc3339());

    let plugin_updates = check_plugin_updates(app.clone(), Some(force)).await.unwrap_or_default();

    let storage = get_storage(&app);
    let engines: Vec<Engine> = storage.load_or_default("engines.json");
    let local_engines: Vec<crate::version_checker::LocalEngineVersion> = engines.iter().map(|e| {
        crate::version_checker::LocalEngineVersion {
            engine_id: e.engine_id.clone(),
            name: e.name.clone(),
            version: e.version.clone(),
            engine_type: e.engine_type.to_string(),
        }
    }).collect();

    let data_dir = get_data_dir(&app);
    let github_base = crate::utils::get_github_api_base(&app);
    let checker = crate::version_checker::VersionChecker::new(data_dir)
        .with_github_api_base(github_base);
    let engine_result = checker.check_for_updates(local_engines).await.ok();
    let engine_updates = engine_result.map(|r| r.updates_available).unwrap_or_default();

    let app_update = check_app_update(app.clone(), Some(force)).await.ok().flatten();

    let hot_update = check_hot_update(app.clone(), None).await.ok().flatten();

    Ok(UpdateCheckResult {
        app_update,
        hot_update,
        plugin_updates,
        engine_updates,
        checked_at: chrono::Utc::now().to_rfc3339(),
    })
}

#[tauri::command]
pub fn get_app_version(app: AppHandle) -> Result<String, String> {
    Ok(app.config().version.clone().unwrap_or_default())
}

#[tauri::command]
pub async fn check_hot_update(app: AppHandle, manifest_url: Option<String>) -> Result<Option<HotUpdateInfo>, String> {
    let data_dir = get_data_dir(&app);
    let settings = load_settings(&app);
    let current_version = app.config().version.clone().unwrap_or_default();

    if !settings.skipped_app_version.is_empty() {
        let skipped_semver = semver::Version::parse(settings.skipped_app_version.trim_start_matches('v')).ok();
        let current_semver = semver::Version::parse(current_version.trim_start_matches('v')).ok();
        if let (Some(skipped), Some(current)) = (skipped_semver, current_semver) {
            if skipped > current {
                return Ok(None);
            }
        }
    }

    let url = manifest_url.unwrap_or_else(|| "https://godotharbor.odayou.workers.dev/hot-update/manifest.json".to_string());
    let manager = crate::hot_update::HotUpdateManager::new(data_dir);
    manager.check_for_hot_update(&url, &current_version).await
}

#[tauri::command]
pub async fn install_hot_update(app: AppHandle, manifest_url: Option<String>) -> Result<(), String> {
    let url = manifest_url.unwrap_or_else(|| "https://godotharbor.odayou.workers.dev/hot-update/manifest.json".to_string());
    let data_dir = get_data_dir(&app);
    let manager = crate::hot_update::HotUpdateManager::new(data_dir);
    manager.download_and_apply(&app, &url).await
}

#[tauri::command]
pub fn rollback_hot_update(app: AppHandle) -> Result<(), String> {
    let data_dir = get_data_dir(&app);
    let manager = crate::hot_update::HotUpdateManager::new(data_dir);
    manager.rollback(&app)
}

#[tauri::command]
pub fn get_current_hot_update_version(app: AppHandle) -> Result<Option<String>, String> {
    let data_dir = get_data_dir(&app);
    let manager = crate::hot_update::HotUpdateManager::new(data_dir);
    manager.get_current_hot_update_version()
}

#[tauri::command]
pub fn get_update_history(app: AppHandle) -> Result<Vec<crate::models::UpdateHistoryEntry>, String> {
    let storage = get_storage(&app);
    let history: Vec<crate::models::UpdateHistoryEntry> = storage.load_or_default("update_history.json");
    Ok(history)
}

#[tauri::command]
pub fn clear_update_history(app: AppHandle) -> Result<(), String> {
    let storage = get_storage(&app);
    let empty: Vec<crate::models::UpdateHistoryEntry> = Vec::new();
    storage.save("update_history.json", &empty)
        .map_err(|e| format!("保存更新历史失败: {}", e))?;
    Ok(())
}

pub fn record_update_history(app: &AppHandle, update_type: &str, target_name: &str, from_version: &str, to_version: &str, status: &str, notes: &str) {
    let storage = get_storage(app);
    let mut history: Vec<crate::models::UpdateHistoryEntry> = storage.load_or_default("update_history.json");
    
    history.insert(0, crate::models::UpdateHistoryEntry {
        id: uuid::Uuid::new_v4().to_string(),
        update_type: update_type.to_string(),
        target_name: target_name.to_string(),
        from_version: from_version.to_string(),
        to_version: to_version.to_string(),
        status: status.to_string(),
        applied_at: chrono::Utc::now().to_rfc3339(),
        notes: notes.to_string(),
    });

    if history.len() > 100 {
        history.truncate(100);
    }

    let _ = storage.save("update_history.json", &history);
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
    let storage = get_storage(&app);
    
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let base_backup_dir = std::path::Path::new(&backup_path).join(format!("backup_{}", timestamp));
    
    let mut backup_dir = base_backup_dir.clone();
    let mut counter = 1;
    while backup_dir.exists() {
        backup_dir = std::path::Path::new(&backup_path).join(format!("backup_{}_{}", timestamp, counter));
        counter += 1;
    }
    
    std::fs::create_dir_all(&backup_dir)
        .map_err(|e| format!("创建备份目录失败: {}", e))?;

    let files = DATA_FILES;
    let mut backup_files = Vec::new();

    for filename in files {
        let src = data_dir.join(filename);
        if src.exists() {
            let dst = backup_dir.join(filename);
            std::fs::copy(&src, &dst)
                .map_err(|e| format!("备份文件 {} 失败: {}", filename, e))?;
            backup_files.push(filename.to_string());
        }
    }

    let plugins_src_dir = data_dir.join("plugins");
    let plugins_dst_dir = backup_dir.join("plugins");
    
    if plugins_src_dir.exists() {
        copy_dir_all(&plugins_src_dir, &plugins_dst_dir)
            .map_err(|e| format!("备份插件目录失败: {}", e))?;
    }

    let projects: Vec<Project> = storage.load_or_default("projects.json");
    let plugins: Vec<Plugin> = storage.load_or_default("plugins.json");
    let bindings: Vec<ProjectBinding> = storage.load_or_default("bindings.json");

    let backup_info = serde_json::json!({
        "version": "1.0",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "app_version": env!("CARGO_PKG_VERSION"),
        "files": backup_files,
        "project_count": projects.len(),
        "plugin_count": plugins.len(),
        "binding_count": bindings.len()
    });

    std::fs::write(backup_dir.join("backup_info.json"), serde_json::to_string_pretty(&backup_info).unwrap())
        .map_err(|e| format!("创建备份信息文件失败: {}", e))?;

    log_operation(&app, "backup_data", &backup_path, &format!("数据备份成功，共备份 {} 个文件", backup_files.len()));
    Ok(format!("备份成功，备份位置: {}", backup_dir.display()))
}

#[tauri::command]
pub fn restore_data(app: AppHandle, backup_path: String) -> Result<String, String> {
    let data_dir = get_data_dir(&app);
    let backup_dir = std::path::Path::new(&backup_path);

    if !backup_dir.exists() {
        return Err("备份目录不存在".to_string());
    }

    let backup_info_path = backup_dir.join("backup_info.json");
    if !backup_info_path.exists() {
        return Err("无效的备份目录，缺少 backup_info.json".to_string());
    }

    let pre_restore_backup_dir = data_dir.join("restore_backup");
    if let Err(e) = std::fs::create_dir_all(&pre_restore_backup_dir) {
        eprintln!("Failed to create pre-restore backup dir: {}", e);
    } else {
        let files = DATA_FILES;
        for filename in files {
            let src = data_dir.join(filename);
            if src.exists() {
                let dst = pre_restore_backup_dir.join(filename);
                let _ = std::fs::copy(&src, &dst);
            }
        }
        let plugins_src = data_dir.join("plugins");
        if plugins_src.exists() {
            let plugins_dst = pre_restore_backup_dir.join("plugins");
            let _ = copy_dir_all(&plugins_src, &plugins_dst);
        }
    }

    let files = DATA_FILES;
    let mut restore_info = Vec::new();

    for filename in files {
        let src = backup_dir.join(filename);
        if src.exists() {
            let dst = data_dir.join(filename);
            std::fs::copy(&src, &dst)
                .map_err(|e| format!("恢复文件 {} 失败: {}", filename, e))?;
            restore_info.push(filename.to_string());
        }
    }

    let plugins_dst_dir = data_dir.join("plugins");
    let plugins_src_dir = backup_dir.join("plugins");
    
    if plugins_src_dir.exists() {
        if plugins_dst_dir.exists() {
            std::fs::remove_dir_all(&plugins_dst_dir)
                .map_err(|e| format!("删除现有插件目录失败: {}", e))?;
        }
        copy_dir_all(&plugins_src_dir, &plugins_dst_dir)
            .map_err(|e| format!("恢复插件目录失败: {}", e))?;
        restore_info.push("plugins/".to_string());
    }

    log_operation(&app, "restore_data", &backup_path, &format!("数据恢复成功，共恢复 {} 个项目", restore_info.len()));
    Ok(format!("恢复成功，共恢复 {} 个项目", restore_info.len()))
}

#[tauri::command]
pub fn reset_data(app: AppHandle, backup_path: String) -> Result<String, String> {
    let data_dir = get_data_dir(&app);
    let storage = get_storage(&app);
    
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let backup_dir = std::path::Path::new(&backup_path).join(format!("backup_{}", timestamp));
    
    std::fs::create_dir_all(&backup_dir)
        .map_err(|e| format!("创建备份目录失败: {}", e))?;

    let files = DATA_FILES;
    let mut backup_files = Vec::new();

    for filename in files {
        let src = data_dir.join(filename);
        if src.exists() {
            let dst = backup_dir.join(filename);
            std::fs::copy(&src, &dst)
                .map_err(|e| format!("备份文件 {} 失败: {}", filename, e))?;
            backup_files.push(filename.to_string());
        }
    }

    let plugins_src_dir = data_dir.join("plugins");
    let plugins_dst_dir = backup_dir.join("plugins");
    
    if plugins_src_dir.exists() {
        copy_dir_all(&plugins_src_dir, &plugins_dst_dir)
            .map_err(|e| format!("备份插件目录失败: {}", e))?;
    }

    let projects: Vec<Project> = storage.load_or_default("projects.json");
    let plugins: Vec<Plugin> = storage.load_or_default("plugins.json");
    let bindings: Vec<ProjectBinding> = storage.load_or_default("bindings.json");

    let backup_info = serde_json::json!({
        "version": "1.0",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "app_version": env!("CARGO_PKG_VERSION"),
        "files": backup_files,
        "project_count": projects.len(),
        "plugin_count": plugins.len(),
        "binding_count": bindings.len()
    });

    std::fs::write(backup_dir.join("backup_info.json"), serde_json::to_string_pretty(&backup_info).unwrap())
        .map_err(|e| format!("创建备份信息文件失败: {}", e))?;

    log_operation(&app, "backup_data", &backup_path, &format!("数据备份成功，共备份 {} 个文件", backup_files.len()));

    let files_to_delete = DATA_FILES;

    for filename in files_to_delete {
        let file_path = data_dir.join(filename);
        if file_path.exists() {
            std::fs::remove_file(&file_path)
                .map_err(|e| format!("删除文件 {} 失败: {}", filename, e))?;
        }
    }

    if plugins_src_dir.exists() {
        std::fs::remove_dir_all(&plugins_src_dir)
            .map_err(|e| format!("删除插件目录失败: {}", e))?;
    }

    log_operation(&app, "reset_data", backup_dir.to_str().unwrap_or(""), "数据重置成功");
    Ok(format!("数据重置成功！\n\n备份已保存至: {}\n\n请重启应用", backup_dir.display()))
}

#[tauri::command]
pub fn register_engine(app: AppHandle, path: String, name: String) -> Result<Engine, String> {
    if path.is_empty() {
        return Err("引擎路径不能为空".to_string());
    }

    let engine_dir = {
        let p = std::path::Path::new(&path);
        if p.is_file() {
            p.parent().map(|dir| dir.to_string_lossy().to_string()).unwrap_or(path.clone())
        } else {
            path.clone()
        }
    };

    if let Err(detail) = crate::engine::EngineManager::validate_engine_path_detail(&engine_dir) {
        log_error(&app, "register_engine", &path, &detail);
        return Err(format!("引擎路径无效: {}", detail));
    }

    let engine = crate::engine::EngineManager::get_engine_info(&engine_dir)
        .map_err(|e| format!("获取引擎信息失败: {}", e))?;

    let mut registered_engine = engine;
    registered_engine.name = if name.is_empty() { registered_engine.name.clone() } else { name };

    let storage = get_storage(&app);
    let mut engines: Vec<Engine> = storage.load_or_default("engines.json");

    engines.retain(|e| e.path != registered_engine.path);

    engines.push(registered_engine.clone());
    storage.save("engines.json", &engines)
        .map_err(|e| format!("保存引擎信息失败: {}", e))?;

    let mut settings = load_settings(&app);
    let parent = std::path::Path::new(&engine_dir)
        .parent()
        .map(|p| p.to_string_lossy().to_string());
    if let Some(parent_path) = parent {
        if !settings.known_engine_paths.iter().any(|p| p.to_lowercase() == parent_path.to_lowercase()) {
            settings.known_engine_paths.push(parent_path);
            let config_storage = get_config_storage(&app);
            let _ = config_storage.save("settings.json", &settings);
        }
    }

    log_operation(&app, "register_engine", &path, &format!("已注册引擎: {}", registered_engine.name));
    Ok(registered_engine)
}

#[tauri::command]
pub fn get_engines(app: AppHandle) -> Result<Vec<Engine>, String> {
    let storage = get_storage(&app);
    let mut engines: Vec<Engine> = storage.load_or_default("engines.json");

    let migrated: bool = storage.load_or_default("engine_version_migrated_v2.json");
    if !migrated {
        for engine in &mut engines {
            if let Ok((_, version)) = crate::engine::EngineManager::detect_engine(&engine.path) {
                if version != engine.version {
                    let is_mono = version.to_lowercase().contains("mono");
                    engine.name = if is_mono {
                        format!("Godot {} (.NET)", version)
                    } else {
                        format!("Godot {}", version)
                    };
                    engine.version = version;
                }
            }
        }
        let _ = storage.save("engines.json", &engines);
        let _ = storage.save("engine_version_migrated_v2.json", &true);
    }

    Ok(engines)
}

#[tauri::command]
pub fn remove_engine(app: AppHandle, engine_id: String, delete_files: bool) -> Result<(), String> {
    let storage = get_storage(&app);
    let mut engines: Vec<Engine> = storage.load_or_default("engines.json");

    let engine = engines.iter().find(|e| e.engine_id == engine_id)
        .ok_or("未找到指定引擎".to_string())?;
    let engine_name = engine.name.clone();
    let engine_path = engine.path.clone();

    engines.retain(|e| e.engine_id != engine_id);

    storage.save("engines.json", &engines)
        .map_err(|e| format!("保存引擎列表失败: {}", e))?;

    if delete_files && !engine_path.is_empty() {
        let path = std::path::Path::new(&engine_path);
        if path.exists() && path.is_dir() {
            let data_dir = get_data_dir(&app);
            let engines_dir = data_dir.join("engines");
            if path.starts_with(&engines_dir) {
                let _ = std::fs::remove_dir_all(path);
            }
        }
    }

    log_operation(&app, "remove_engine", &engine_id, &format!("已删除引擎: {}{}", engine_name, if delete_files { "（含文件）" } else { "" }));
    Ok(())
}

#[tauri::command]
pub async fn check_plugin_updates(app: AppHandle, force_refresh: Option<bool>) -> Result<Vec<PluginUpdateInfo>, String> {
    let force = force_refresh.unwrap_or(false);
    let cache_version: u32 = 1;

    let cache_dir = get_data_dir(&app).join("cache");
    let cache_file = cache_dir.join("plugin_updates.json");

    if !force && cache_file.exists() {
        if let Ok(content) = fs::read_to_string(&cache_file) {
            if let Ok(cached) = serde_json::from_str::<crate::models::CachedPluginUpdates>(&content) {
                if cached.cache_version == cache_version {
                    if let Ok(cached_time) = chrono::DateTime::parse_from_rfc3339(&cached.cached_at) {
                        let elapsed = chrono::Utc::now().signed_duration_since(cached_time.with_timezone(&chrono::Utc));
                        if elapsed.num_minutes() < 30 {
                            log_operation(&app, "check_plugin_updates", "", &format!("使用缓存，{} 个插件更新信息", cached.updates.len()));
                            return Ok(cached.updates);
                        }
                    }
                } else {
                    let _ = fs::remove_file(&cache_file);
                }
            }
        }
    }

    let storage = get_storage(&app);
    let plugins: Vec<Plugin> = storage.load_or_default("plugins.json");

    let client = create_http_client(Some(std::time::Duration::from_secs(10)))?;
    let github_base = crate::utils::get_github_api_base(&app);

    let futures = plugins.iter().map(|plugin| {
        let client = client.clone();
        let plugin_id = plugin.plugin_id.clone();
        let plugin_name = plugin.name.clone();
        let current_version = plugin.versions.last()
            .map(|v| v.version.clone())
            .unwrap_or_else(|| "0.0.0".to_string());
        let source_type = plugin.source.source_type.clone();
        let url = plugin.source.url.clone();
        let github_base = github_base.clone();

        async move {
            let mut latest_version = current_version.clone();
            let mut release_notes = String::new();

            if source_type == SourceType::Git && !url.is_empty() && url.contains("github.com") {
                let api_url = url.trim_end_matches(".git")
                    .replace("git@github.com:", "https://api.github.com/repos/")
                    .replace("https://github.com/", "https://api.github.com/repos/");
                let api_url = api_url.replace("https://api.github.com", &github_base);
                let releases_url = format!("{}/releases/latest", api_url);

                if let Ok(resp) = client.get(&releases_url).send().await {
                    if resp.status().is_success() {
                        if let Ok(json) = resp.json::<serde_json::Value>().await {
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

            let update_available = compare_versions(&current_version, &latest_version) < 0;

            PluginUpdateInfo {
                plugin_id,
                plugin_name,
                current_version,
                latest_version,
                update_available,
                release_notes,
                source_url: url,
            }
        }
    });

    let update_infos: Vec<PluginUpdateInfo> = futures::future::join_all(futures).await;

    let _ = fs::create_dir_all(&cache_dir);
    let cached = crate::models::CachedPluginUpdates {
        cache_version,
        cached_at: chrono::Utc::now().to_rfc3339(),
        updates: update_infos.clone(),
    };
    if let Ok(json) = serde_json::to_string_pretty(&cached) {
        let _ = fs::write(&cache_file, json);
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

    upsert_plugin(&app, &plugin, "import_asset_library", &asset_id.to_string())
}

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

    let result = upsert_plugin(&app, &plugin, "import_asset_library", &asset_id.to_string())?;

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
    let settings = load_settings(&app);

    if !settings.auto_scan_on_startup {
        return Ok(Vec::new());
    }

    let scan_dirs = if settings.scan_directories.is_empty() {
        let mut dirs = get_default_scan_dirs();
        if !cfg!(windows) {
            if let Some(home) = std::env::var("HOME").ok() {
                dirs.push(format!("{}/Documents/godot", home));
            }
        }
        dirs
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

    let settings = load_settings(&app);
    let scan_dirs = if settings.scan_directories.is_empty() {
        get_default_scan_dirs()
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
pub async fn check_godot_updates(app: AppHandle) -> Result<crate::version_checker::GodotVersionCheckResult, String> {
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
    let github_base = crate::utils::get_github_api_base(&app);
    let checker = crate::version_checker::VersionChecker::new(cache_dir)
        .with_github_api_base(github_base);

    let result = checker.check_for_updates(local_engines).await?;

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

    let ids_set: std::collections::HashSet<_> = project_ids.iter().cloned().collect();

    for project_id in &project_ids {
        if projects.iter().any(|p| p.project_id == *project_id) {
            projects.retain(|p| p.project_id != *project_id);
            success_count += 1;
        } else {
            failed_count += 1;
            errors.push(format!("未找到项目: {}", project_id));
        }
    }

    let mut bindings: Vec<ProjectBinding> = storage.load_or_default("bindings.json");
    let had_bindings = bindings.iter().any(|b| ids_set.contains(&b.project_id));
    bindings.retain(|b| !ids_set.contains(&b.project_id));
    if had_bindings {
        storage.save("bindings.json", &bindings)
            .map_err(|e| format!("保存绑定列表失败: {}", e))?;
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
            req.subdirectory.clone(),
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
            let plugin_path = std::path::Path::new(&project.path).join(&mount_path);
            if plugin_path.exists() {
                let metadata = std::fs::symlink_metadata(&plugin_path);
                let is_link = metadata.as_ref().map(|m| m.file_type().is_symlink()).unwrap_or(false);
                let is_junction = {
                    #[cfg(windows)]
                    {
                        use std::os::windows::fs::MetadataExt;
                        metadata.as_ref().map(|m| m.file_attributes() & 0x400 != 0).unwrap_or(false)
                    }
                    #[cfg(not(windows))]
                    {
                        false
                    }
                };
                if is_link || is_junction {
                    let _ = std::fs::remove_dir(&plugin_path);
                } else {
                    let _ = std::fs::remove_dir_all(&plugin_path);
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
pub async fn batch_apply_changes(app: AppHandle, project_ids: Vec<String>) -> Result<BatchApplyResult, String> {
    let storage = get_storage(&app);
    let projects: Vec<Project> = storage.load_or_default("projects.json");
    let all_bindings: Vec<ProjectBinding> = storage.load_or_default("bindings.json");
    let settings = load_settings(&app);
    let linker = Linker::new(settings.mount_strategy);
    let data_dir = get_data_dir(&app);
    let plugin_base_path = data_dir.join("plugins");

    let futures: Vec<_> = project_ids.iter().map(|project_id| {
        let project_id = project_id.clone();
        let projects = projects.clone();
        let all_bindings = all_bindings.clone();
        let linker = linker.clone();
        let plugin_base_path = plugin_base_path.clone();

        tokio::task::spawn_blocking(move || {
            let project = match projects.iter().find(|p| p.project_id == project_id) {
                Some(p) => p,
                None => {
                    return ProjectApplyResult {
                        project_id: project_id.clone(),
                        project_name: String::new(),
                        success: false,
                        created: Vec::new(),
                        removed: Vec::new(),
                        errors: vec![format!("未找到项目: {}", project_id)],
                    };
                }
            };

            let desired_bindings: Vec<ProjectBinding> = all_bindings.iter()
                .filter(|b| b.project_id == project_id)
                .cloned()
                .collect();

            if desired_bindings.is_empty() {
                return ProjectApplyResult {
                    project_id: project_id.clone(),
                    project_name: project.name.clone(),
                    success: true,
                    created: Vec::new(),
                    removed: Vec::new(),
                    errors: Vec::new(),
                };
            }

            let current_bindings: Vec<ProjectBinding> = Vec::new();

            match linker.apply_bindings(
                &project.path,
                &current_bindings,
                &desired_bindings,
                &plugin_base_path.to_string_lossy()
            ) {
                Ok(apply_result) => {
                    ProjectApplyResult {
                        project_id: project_id.clone(),
                        project_name: project.name.clone(),
                        success: apply_result.success,
                        created: apply_result.created,
                        removed: apply_result.removed,
                        errors: apply_result.errors,
                    }
                }
                Err(e) => {
                    ProjectApplyResult {
                        project_id: project_id.clone(),
                        project_name: project.name.clone(),
                        success: false,
                        created: Vec::new(),
                        removed: Vec::new(),
                        errors: vec![format!("应用变更失败: {}", e)],
                    }
                }
            }
        })
    }).collect();

    let results: Vec<ProjectApplyResult> = join_all(futures)
        .await
        .into_iter()
        .filter_map(|r| r.ok())
        .collect();

    log_operation(&app, "batch_apply_changes", "",
        &format!("批量应用变更完成，共处理 {} 个项目", results.len()));

    Ok(BatchApplyResult { results })
}

#[tauri::command]
pub async fn auto_discover_engines(app: AppHandle) -> Result<Vec<Engine>, String> {
    let settings = load_settings(&app);

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
    let scan_dirs = settings.scan_directories.clone();
    let known_engine_paths = settings.known_engine_paths.clone();

    log_operation(&app, "auto_discover_engines", "", "开始自动发现引擎");

    let discovered = tokio::task::spawn_blocking(move || {
        crate::engine::EngineManager::discover_engines_with_known_paths(
            &existing_paths,
            &scan_dirs,
            &known_engine_paths,
        )
    })
    .await
    .map_err(|e| format!("发现引擎任务失败: {}", e))?;

    if discovered.is_empty() {
        return Ok(Vec::new());
    }

    let discovered_count = discovered.len();
    for engine in &discovered {
        engines.push(engine.clone());
    }

    let storage = get_storage(&app);
    storage.save("engines.json", &engines)
        .map_err(|e| format!("保存引擎列表失败: {}", e))?;

    let mut settings = load_settings(&app);
    for engine in &discovered {
        let parent = std::path::Path::new(&engine.path)
            .parent()
            .map(|p| p.to_string_lossy().to_string());
        if let Some(parent_path) = parent {
            if !settings.known_engine_paths.iter().any(|p| p.to_lowercase() == parent_path.to_lowercase()) {
                settings.known_engine_paths.push(parent_path);
            }
        }
    }
    let config_storage = get_config_storage(&app);
    let _ = config_storage.save("settings.json", &settings);

    let _ = app.emit("engines-discovered", &discovered);

    log_operation(&app, "auto_discover_engines", "",
        &format!("自动发现 {} 个 Godot 引擎", discovered_count));

    Ok(discovered)
}

#[tauri::command]
pub fn check_engine_health(app: AppHandle, engine_id: String) -> Result<bool, String> {
    let storage = get_storage(&app);
    let engines: Vec<Engine> = storage.load_or_default("engines.json");

    let engine = engines.iter()
        .find(|e| e.engine_id == engine_id)
        .ok_or("未找到指定引擎".to_string())?;

    let exe_path = crate::engine::EngineManager::find_executable_in_dir(std::path::Path::new(&engine.path));
    Ok(exe_path.is_some())
}

#[tauri::command]
pub fn rename_engine(app: AppHandle, engine_id: String, new_name: String) -> Result<(), String> {
    if new_name.trim().is_empty() {
        return Err("引擎名称不能为空".to_string());
    }

    let storage = get_storage(&app);
    let mut engines: Vec<Engine> = storage.load_or_default("engines.json");

    let old_name;
    let new_engine_name;
    {
        let engine = engines.iter_mut()
            .find(|e| e.engine_id == engine_id)
            .ok_or("未找到指定引擎".to_string())?;

        old_name = engine.name.clone();
        new_engine_name = new_name.trim().to_string();
        engine.name = new_engine_name.clone();
    }

    storage.save("engines.json", &engines)
        .map_err(|e| format!("保存引擎列表失败: {}", e))?;

    log_operation(&app, "rename_engine", &engine_id, &format!("引擎重命名: {} -> {}", old_name, new_engine_name));
    Ok(())
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
