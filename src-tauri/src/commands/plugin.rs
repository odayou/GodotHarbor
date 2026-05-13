use std::path::Path;
use std::fs;
use serde::{Serialize, Deserialize};
use tauri::{AppHandle, Emitter};
use crate::models::*;
use crate::storage::Storage;
use crate::linker::Linker;
use uuid::Uuid;
use futures::future::join_all;
use crate::utils::{create_http_client, no_window_cmd};
use super::utils::*;
use super::update::record_update_history;

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

    let existing_hash_map: std::collections::HashMap<String, String> = plugins.iter()
        .filter(|p| !p.content_hash.is_empty())
        .map(|p| (p.content_hash.clone(), p.plugin_id.clone()))
        .collect();

    let existing_name_map: std::collections::HashMap<String, String> = plugins.iter()
        .map(|p| (p.name.to_lowercase(), p.plugin_id.clone()))
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
        let mut pending_bindings: Vec<ProjectBinding> = Vec::new();
        let mut seen_paths = std::collections::HashSet::new();

        for scanned in &scanned_plugins_clone {
            let path_str = scanned.path.clone();
            let path_lower = path_str.replace('\\', "/").to_lowercase();

            if seen_paths.contains(&path_lower) {
                continue;
            }
            seen_paths.insert(path_lower);

            let source_path = Path::new(&path_str);
            let content_hash = compute_dir_hash(source_path).unwrap_or_default();

            if let Some(existing_plugin_id) = existing_hash_map.get(&content_hash).cloned() {
                if !content_hash.is_empty() {
                    if let Some(proj) = find_project_by_id_or_path(&projects_clone, &scanned.project_id, &path_str) {
                        let mount_path = compute_mount_path(&path_str, &proj.path);
                        let existing_plugin = local_plugins.iter().find(|p| p.plugin_id == existing_plugin_id);
                        let version = existing_plugin.and_then(|p| p.versions.first());
                        let version_id = version.map(|v| v.version_id.clone()).unwrap_or_default();
                        let unit_id = version
                            .and_then(|v| v.units.first())
                            .map(|u| u.unit_id.clone())
                            .unwrap_or_default();

                        pending_bindings.push(ProjectBinding::new(
                            proj.project_id.clone(),
                            existing_plugin_id,
                            version_id,
                            unit_id,
                            mount_path,
                            String::new(),
                        ));
                    }
                    continue;
                }
            }

            if let Some(existing_plugin_id) = existing_name_map.get(&scanned.plugin_name.to_lowercase()).cloned() {
                if content_hash.is_empty() {
                    if let Some(proj) = find_project_by_id_or_path(&projects_clone, &scanned.project_id, &path_str) {
                        let mount_path = compute_mount_path(&path_str, &proj.path);
                        let existing_plugin = local_plugins.iter().find(|p| p.plugin_id == existing_plugin_id);
                        let version = existing_plugin.and_then(|p| p.versions.first());
                        let version_id = version.map(|v| v.version_id.clone()).unwrap_or_default();
                        let unit_id = version
                            .and_then(|v| v.units.first())
                            .map(|u| u.unit_id.clone())
                            .unwrap_or_default();

                        pending_bindings.push(ProjectBinding::new(
                            proj.project_id.clone(),
                            existing_plugin_id,
                            version_id,
                            unit_id,
                            mount_path,
                            String::new(),
                        ));
                    }
                    continue;
                }
            }

            match import_mode_clone.as_str() {
                "copy" => {
                    match manager_clone.import_from_local(&path_str) {
                        Ok(plugin) => {
                            let plugin_id = plugin.plugin_id.clone();
                            let version = plugin.versions.first();
                            let version_id = version.map(|v| v.version_id.clone()).unwrap_or_default();
                            let unit_id = version
                                .and_then(|v| v.units.first())
                                .map(|u| u.unit_id.clone())
                                .unwrap_or_default();

                            local_imported.push(plugin.clone());
                            local_plugins.push(plugin);

                            if let Some(proj) = find_project_by_id_or_path(&projects_clone, &scanned.project_id, &path_str) {
                                let mount_path = compute_mount_path(&path_str, &proj.path);
                                pending_bindings.push(ProjectBinding::new(
                                    proj.project_id.clone(),
                                    plugin_id,
                                    version_id,
                                    unit_id,
                                    mount_path,
                                    String::new(),
                                ));
                            }
                        }
                        Err(e) => eprintln!("Failed to import plugin from {}: {}", path_str, e),
                    }
                }
                "move" => {
                    match manager_clone.import_from_local(&path_str) {
                        Ok(mut plugin) => {
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
                            let unit_id = version
                                .and_then(|v| v.units.first())
                                .map(|u| u.unit_id.clone())
                                .unwrap_or_default();

                            if let Err(e) = replace_with_symlink(source_path, &payload_path) {
                                eprintln!("Warning: failed to replace with symlink: {}", e);
                            }

                            local_imported.push(plugin.clone());
                            local_plugins.push(plugin);

                            if let Some(proj) = find_project_by_id_or_path(&projects_clone, &scanned.project_id, &path_str) {
                                let mount_path = compute_mount_path(&path_str, &proj.path);
                                pending_bindings.push(ProjectBinding::new(
                                    proj.project_id.clone(),
                                    plugin_id,
                                    version_id,
                                    unit_id,
                                    mount_path,
                                    String::new(),
                                ));
                            }
                        }
                        Err(e) => eprintln!("Failed to import plugin from {}: {}", path_str, e),
                    }
                }
                "reference" => {
                    let plugin_name = scanned.plugin_name.clone();
                    let plugin_source = PluginSource {
                        source_type: SourceType::Local,
                        url: path_str.clone(),
                        imported_at: chrono::Utc::now(),
                    };
                    let mut plugin = Plugin::new(plugin_name.clone(), plugin_source);
                    plugin.content_hash = if content_hash.is_empty() {
                        compute_dir_hash(source_path).unwrap_or_default()
                    } else {
                        content_hash
                    };

                    match manager_clone.parse_plugin_units(source_path) {
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
                                version_id: version_id.clone(),
                                version: unit_version,
                                path: path_str.clone(),
                                created_at: chrono::Utc::now(),
                                units,
                            };
                            plugin.versions.push(plugin_version);
                            plugin.compatibility = manager_clone.detect_compatibility(source_path);
                            plugin.name = unit_name;
                            plugin.description = unit_description;
                            plugin.author = unit_author;

                            let plugin_id = plugin.plugin_id.clone();
                            let unit_id = plugin.versions.first()
                                .and_then(|v| v.units.first())
                                .map(|u| u.unit_id.clone())
                                .unwrap_or_default();

                            if let Some(proj) = find_project_by_id_or_path(&projects_clone, &scanned.project_id, &path_str) {
                                let mount_path = compute_mount_path(&path_str, &proj.path);
                                pending_bindings.push(ProjectBinding::new(
                                    proj.project_id.clone(),
                                    plugin_id,
                                    version_id,
                                    unit_id,
                                    mount_path,
                                    String::new(),
                                ));
                            }

                            local_imported.push(plugin.clone());
                            local_plugins.push(plugin);
                        }
                        Err(_) => {
                            let version_id = Uuid::new_v4().to_string();
                            let plugin_version = PluginVersion {
                                version_id: version_id.clone(),
                                version: "1.0.0".to_string(),
                                path: path_str.clone(),
                                created_at: chrono::Utc::now(),
                                units: Vec::new(),
                            };
                            plugin.versions.push(plugin_version);

                            let plugin_id = plugin.plugin_id.clone();

                            if let Some(proj) = find_project_by_id_or_path(&projects_clone, &scanned.project_id, &path_str) {
                                let mount_path = compute_mount_path(&path_str, &proj.path);
                                pending_bindings.push(ProjectBinding::new(
                                    proj.project_id.clone(),
                                    plugin_id,
                                    version_id,
                                    String::new(),
                                    mount_path,
                                    String::new(),
                                ));
                            }

                            local_imported.push(plugin.clone());
                            local_plugins.push(plugin);
                        }
                    }
                }
                _ => {}
            }
        }

        if !pending_bindings.is_empty() {
            let mut bindings: Vec<ProjectBinding> = storage_clone.load_or_default("bindings.json");
            for new_binding in &pending_bindings {
                bindings.retain(|b| !(b.project_id == new_binding.project_id && b.plugin_id == new_binding.plugin_id));
            }
            bindings.extend(pending_bindings);
            let _ = storage_clone.save("bindings.json", &bindings);
        }

        (local_plugins, local_imported)
    }).await.map_err(|e| format!("导入任务失败: {}", e))?;

    let (updated_plugins, imported) = import_result;
    plugins = updated_plugins;
    let imported_plugins = imported;

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

fn find_project_by_id_or_path<'a>(projects: &'a [Project], project_id: &str, plugin_path: &str) -> Option<&'a Project> {
    if !project_id.is_empty() {
        if let Some(proj) = projects.iter().find(|p| p.project_id == project_id) {
            return Some(proj);
        }
    }
    projects.iter().find(|p| plugin_path.starts_with(&p.path))
}

fn compute_mount_path(plugin_path: &str, project_path: &str) -> String {
    plugin_path
        .replace(&format!("{}/", project_path.replace('\\', "/")), "")
        .replace(&format!("{}\\", project_path), "")
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

pub const APP_GITHUB_OWNER: &str = "odayou";
pub const APP_GITHUB_REPO: &str = "GodotHarbor";


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

fn parse_packed_string_array(value: &str) -> Vec<String> {
    let inner = value.trim()
        .strip_prefix("PackedStringArray(")
        .and_then(|s| s.strip_suffix(")"))
        .unwrap_or("");
    if inner.is_empty() {
        return vec![];
    }
    let mut result = Vec::new();
    let mut current = String::new();
    let mut in_string = false;
    for ch in inner.chars() {
        if ch == '"' {
            in_string = !in_string;
            if !in_string && !current.is_empty() {
                result.push(current.clone());
                current.clear();
            }
        } else if in_string {
            current.push(ch);
        }
    }
    result
}

fn extract_plugin_dir_name(entry: &str) -> String {
    let mut result = entry.to_string();
    loop {
        let next = result
            .strip_prefix("res://addons/")
            .and_then(|s| s.strip_suffix("/plugin.cfg"))
            .unwrap_or(&result)
            .to_string();
        if next == result {
            break;
        }
        result = next;
    }
    if result.contains('/') || result.contains('\\') || result.contains("res://") {
        result
            .replace('\\', "/")
            .split('/')
            .last()
            .unwrap_or(&result)
            .to_string()
    } else {
        result
    }
}

fn parse_enabled_entries(value: &str) -> Vec<String> {
    let raw = parse_packed_string_array(value);
    raw.iter().map(|e| extract_plugin_dir_name(e)).collect()
}

fn build_packed_string_array(entries: &[String]) -> String {
    if entries.is_empty() {
        return "enabled=PackedStringArray()".to_string();
    }
    let items: Vec<String> = entries.iter().map(|e| format!("\"res://addons/{}/plugin.cfg\"", e)).collect();
    format!("enabled=PackedStringArray({})", items.join(", "))
}

fn modify_editor_plugins(
    project_path: &str,
    plugin_dir_name: &str,
    enable: bool,
) -> Result<bool, String> {
    if plugin_dir_name.is_empty()
        || plugin_dir_name.contains('/')
        || plugin_dir_name.contains('\\')
        || plugin_dir_name.contains("res://")
        || plugin_dir_name.contains("plugin.cfg")
    {
        return Err(format!("Invalid plugin_dir_name: '{}', must be a simple directory name", plugin_dir_name));
    }

    let project_godot = Path::new(project_path).join("project.godot");

    if !project_godot.exists() {
        return Err("project.godot not found".to_string());
    }

    let content = fs::read_to_string(&project_godot)
        .map_err(|e| format!("Failed to read project.godot: {}", e))?;

    if !content.lines().any(|l| l.trim().starts_with('[') && l.trim().ends_with(']')) {
        return Err("project.godot appears to be corrupted (no INI sections found), skipping plugin enable".to_string());
    }

    let backup_path = project_godot.with_extension("godot.harborbak");
    fs::write(&backup_path, &content)
        .map_err(|e| format!("Failed to backup project.godot: {}", e))?;

    let line_ending = if content.contains("\r\n") { "\r\n" } else { "\n" };
    let has_trailing_newline = content.ends_with('\n');
    let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();

    let mut section_idx: Option<usize> = None;
    let mut next_section_idx: Option<usize> = None;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed == "[editor_plugins]" {
            section_idx = Some(i);
        } else if trimmed.starts_with('[') && trimmed.ends_with(']') && trimmed != "[editor_plugins]" {
            if section_idx.is_some() && next_section_idx.is_none() {
                next_section_idx = Some(i);
            }
        }
    }

    let mut install_entries: Vec<String> = Vec::new();
    let mut enabled_entries: Vec<String> = Vec::new();

    if let Some(idx) = section_idx {
        let end = next_section_idx.unwrap_or(lines.len());
        for i in idx..end {
            let trimmed = lines[i].trim();
            if trimmed.starts_with('[') || trimmed.is_empty() || trimmed.starts_with(';') {
                continue;
            }
            if let Some(eq_pos) = trimmed.find('=') {
                let key = trimmed[..eq_pos].trim();
                let val = trimmed[eq_pos + 1..].trim();
                if key == "enabled" {
                    enabled_entries = parse_enabled_entries(val);
                } else if val == "true" {
                    install_entries.push(key.to_string());
                }
            }
        }
    }

    if enable {
        if !install_entries.iter().any(|e| e == plugin_dir_name) {
            install_entries.push(plugin_dir_name.to_string());
        }
        let mut seen = std::collections::HashSet::new();
        enabled_entries.retain(|e| seen.insert(e.clone()));
        if !seen.contains(plugin_dir_name) {
            enabled_entries.push(plugin_dir_name.to_string());
        }
    } else {
        install_entries.retain(|e| e != plugin_dir_name);
        let mut seen = std::collections::HashSet::new();
        enabled_entries.retain(|e| seen.insert(e.clone()));
        enabled_entries.retain(|e| e != plugin_dir_name);
    }

    if install_entries.is_empty() && enabled_entries.is_empty() {
        if let Some(idx) = section_idx {
            let end = next_section_idx.unwrap_or(lines.len());
            lines.drain(idx..end);
            if idx > 0 && lines.get(idx - 1).map(|l| l.trim().is_empty()).unwrap_or(false) {
                lines.remove(idx - 1);
            }
        }
    } else {
        let mut new_section_lines: Vec<String> = vec!["[editor_plugins]".to_string(), String::new()];
        for entry in &install_entries {
            new_section_lines.push(format!("{}=true", entry));
        }
        if !enabled_entries.is_empty() {
            new_section_lines.push(build_packed_string_array(&enabled_entries));
        }
        new_section_lines.push(String::new());

        if let Some(idx) = section_idx {
            let end = next_section_idx.unwrap_or(lines.len());
            lines.splice(idx..end, new_section_lines);
        } else {
            let insert_pos = lines.iter().position(|l| l.trim().starts_with('[')).unwrap_or(0);
            new_section_lines.push(String::new());
            lines.splice(insert_pos..insert_pos, new_section_lines);
        }
    }

    let mut new_content = lines.join(line_ending);
    if has_trailing_newline && !new_content.ends_with(line_ending) {
        new_content.push_str(line_ending);
    }
    match fs::write(&project_godot, &new_content) {
        Ok(_) => {
            let verify = fs::read_to_string(&project_godot).unwrap_or_default();
            if !verify.lines().any(|l| l.trim().starts_with('[') && l.trim().ends_with(']')) {
                let _ = fs::write(&project_godot, &content);
                return Err("Verification failed: project.godot appears corrupted after write, reverted".to_string());
            }
            let _ = fs::remove_file(&backup_path);
            Ok(true)
        }
        Err(e) => {
            let _ = fs::write(&project_godot, &content);
            Err(format!("Failed to write project.godot: {}", e))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    fn create_project_godot(dir: &std::path::Path, content: &str) -> std::path::PathBuf {
        fs::write(dir.join("project.godot"), content).unwrap();
        dir.join("project.godot")
    }

    fn read_project_godot(dir: &std::path::Path) -> String {
        fs::read_to_string(dir.join("project.godot")).unwrap()
    }

    #[test]
    fn test_parse_packed_string_array_empty() {
        assert_eq!(parse_packed_string_array("PackedStringArray()"), Vec::<String>::new());
    }

    #[test]
    fn test_parse_packed_string_array_single() {
        let result = parse_packed_string_array(
            r#"PackedStringArray("res://addons/foo/plugin.cfg")"#
        );
        assert_eq!(result, vec!["res://addons/foo/plugin.cfg"]);
    }

    #[test]
    fn test_parse_packed_string_array_multiple() {
        let result = parse_packed_string_array(
            r#"PackedStringArray("res://addons/foo/plugin.cfg", "res://addons/bar/plugin.cfg")"#
        );
        assert_eq!(result, vec![
            "res://addons/foo/plugin.cfg",
            "res://addons/bar/plugin.cfg"
        ]);
    }

    #[test]
    fn test_parse_packed_string_array_with_spaces() {
        let result = parse_packed_string_array(
            r#"PackedStringArray( "res://addons/foo/plugin.cfg" , "res://addons/bar/plugin.cfg" )"#
        );
        assert_eq!(result, vec![
            "res://addons/foo/plugin.cfg",
            "res://addons/bar/plugin.cfg"
        ]);
    }

    #[test]
    fn test_extract_plugin_dir_name_simple() {
        assert_eq!(
            extract_plugin_dir_name("res://addons/my_plugin/plugin.cfg"),
            "my_plugin"
        );
    }

    #[test]
    fn test_extract_plugin_dir_name_nested() {
        assert_eq!(
            extract_plugin_dir_name("res://addons/sub/my_plugin/plugin.cfg"),
            "my_plugin"
        );
    }

    #[test]
    fn test_extract_plugin_dir_name_bare() {
        assert_eq!(extract_plugin_dir_name("my_plugin"), "my_plugin");
    }

    #[test]
    fn test_build_packed_string_array_empty() {
        assert_eq!(build_packed_string_array(&[]), "enabled=PackedStringArray()");
    }

    #[test]
    fn test_build_packed_string_array_single() {
        let entries = vec!["foo".to_string()];
        assert_eq!(
            build_packed_string_array(&entries),
            r#"enabled=PackedStringArray("res://addons/foo/plugin.cfg")"#
        );
    }

    #[test]
    fn test_build_packed_string_array_multiple() {
        let entries = vec!["foo".to_string(), "bar".to_string()];
        assert_eq!(
            build_packed_string_array(&entries),
            r#"enabled=PackedStringArray("res://addons/foo/plugin.cfg", "res://addons/bar/plugin.cfg")"#
        );
    }

    #[test]
    fn test_modify_editor_plugins_enable_no_existing_section() {
        let dir = TempDir::new().unwrap();
        let content = "[application]\nconfig/name=\"Test\"\n";
        create_project_godot(dir.path(), content);

        let result = modify_editor_plugins(dir.path().to_str().unwrap(), "my_plugin", true);
        assert!(result.is_ok());

        let new_content = read_project_godot(dir.path());
        assert!(new_content.contains("[editor_plugins]"));
        assert!(new_content.contains("my_plugin=true"));
        assert!(new_content.contains("res://addons/my_plugin/plugin.cfg"));
    }

    #[test]
    fn test_modify_editor_plugins_enable_existing_section() {
        let dir = TempDir::new().unwrap();
        let content = "[editor_plugins]\nfoo=true\nenabled=PackedStringArray(\"res://addons/foo/plugin.cfg\")\n\n[application]\nconfig/name=\"Test\"\n";
        create_project_godot(dir.path(), content);

        let result = modify_editor_plugins(dir.path().to_str().unwrap(), "bar", true);
        assert!(result.is_ok());

        let new_content = read_project_godot(dir.path());
        assert!(new_content.contains("bar=true"));
        assert!(new_content.contains("res://addons/bar/plugin.cfg"));
        assert!(new_content.contains("res://addons/foo/plugin.cfg"));
    }

    #[test]
    fn test_modify_editor_plugins_disable_removes_entry() {
        let dir = TempDir::new().unwrap();
        let content = "[editor_plugins]\nfoo=true\nbar=true\nenabled=PackedStringArray(\"res://addons/foo/plugin.cfg\", \"res://addons/bar/plugin.cfg\")\n\n[application]\nconfig/name=\"Test\"\n";
        create_project_godot(dir.path(), content);

        let result = modify_editor_plugins(dir.path().to_str().unwrap(), "bar", false);
        assert!(result.is_ok());

        let new_content = read_project_godot(dir.path());
        assert!(!new_content.contains("bar=true"));
        assert!(!new_content.contains("res://addons/bar/plugin.cfg"));
        assert!(new_content.contains("foo=true"));
    }

    #[test]
    fn test_modify_editor_plugins_disable_last_removes_section() {
        let dir = TempDir::new().unwrap();
        let content = "[editor_plugins]\nfoo=true\nenabled=PackedStringArray(\"res://addons/foo/plugin.cfg\")\n\n[application]\nconfig/name=\"Test\"\n";
        create_project_godot(dir.path(), content);

        let result = modify_editor_plugins(dir.path().to_str().unwrap(), "foo", false);
        assert!(result.is_ok());

        let new_content = read_project_godot(dir.path());
        assert!(!new_content.contains("[editor_plugins]"));
    }

    #[test]
    fn test_modify_editor_plugins_invalid_dir_name_with_slash() {
        let dir = TempDir::new().unwrap();
        create_project_godot(dir.path(), "[application]\n");

        let result = modify_editor_plugins(dir.path().to_str().unwrap(), "foo/bar", true);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid plugin_dir_name"));
    }

    #[test]
    fn test_modify_editor_plugins_invalid_dir_name_empty() {
        let dir = TempDir::new().unwrap();
        create_project_godot(dir.path(), "[application]\n");

        let result = modify_editor_plugins(dir.path().to_str().unwrap(), "", true);
        assert!(result.is_err());
    }

    #[test]
    fn test_modify_editor_plugins_invalid_dir_name_with_plugin_cfg() {
        let dir = TempDir::new().unwrap();
        create_project_godot(dir.path(), "[application]\n");

        let result = modify_editor_plugins(dir.path().to_str().unwrap(), "plugin.cfg", true);
        assert!(result.is_err());
    }

    #[test]
    fn test_modify_editor_plugins_no_project_godot() {
        let dir = TempDir::new().unwrap();

        let result = modify_editor_plugins(dir.path().to_str().unwrap(), "foo", true);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("project.godot not found"));
    }

    #[test]
    fn test_modify_editor_plugins_corrupted_file() {
        let dir = TempDir::new().unwrap();
        create_project_godot(dir.path(), "this is not a valid godot file\nno sections here\n");

        let result = modify_editor_plugins(dir.path().to_str().unwrap(), "foo", true);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("corrupted"));
    }

    #[test]
    fn test_modify_editor_plugins_preserves_line_endings() {
        let dir = TempDir::new().unwrap();
        let content = "[application]\r\nconfig/name=\"Test\"\r\n";
        create_project_godot(dir.path(), content);

        let result = modify_editor_plugins(dir.path().to_str().unwrap(), "foo", true);
        assert!(result.is_ok());

        let new_content = read_project_godot(dir.path());
        assert!(new_content.contains("\r\n"));
    }

    #[test]
    fn test_modify_editor_plugins_enable_idempotent() {
        let dir = TempDir::new().unwrap();
        let content = "[editor_plugins]\nfoo=true\nenabled=PackedStringArray(\"res://addons/foo/plugin.cfg\")\n\n[application]\n";
        create_project_godot(dir.path(), content);

        let result = modify_editor_plugins(dir.path().to_str().unwrap(), "foo", true);
        assert!(result.is_ok());

        let new_content = read_project_godot(dir.path());
        let foo_count = new_content.matches("foo=true").count();
        assert_eq!(foo_count, 1);
    }

    #[test]
    fn test_modify_editor_plugins_deduplicates_enabled() {
        let dir = TempDir::new().unwrap();
        let content = "[editor_plugins]\nfoo=true\nenabled=PackedStringArray(\"res://addons/foo/plugin.cfg\", \"res://addons/foo/plugin.cfg\")\n\n[application]\n";
        create_project_godot(dir.path(), content);

        let result = modify_editor_plugins(dir.path().to_str().unwrap(), "bar", true);
        assert!(result.is_ok());

        let new_content = read_project_godot(dir.path());
        let foo_count = new_content.matches("res://addons/foo/plugin.cfg").count();
        assert_eq!(foo_count, 1);
    }

    #[test]
    fn test_derive_plugin_dir_name_simple() {
        let binding = ProjectBinding::new(
            "p1".into(), "pl1".into(), "v1".into(), "u1".into(),
            "addons/my_plugin".into(), String::new(),
        );
        assert_eq!(derive_plugin_dir_name(&binding), "my_plugin");
    }

    #[test]
    fn test_derive_plugin_dir_name_with_res_prefix() {
        let binding = ProjectBinding::new(
            "p1".into(), "pl1".into(), "v1".into(), "u1".into(),
            "res://addons/my_plugin".into(), String::new(),
        );
        assert_eq!(derive_plugin_dir_name(&binding), "my_plugin");
    }

    #[test]
    fn test_derive_plugin_dir_name_nested() {
        let binding = ProjectBinding::new(
            "p1".into(), "pl1".into(), "v1".into(), "u1".into(),
            "addons/sub/my_plugin".into(), String::new(),
        );
        assert_eq!(derive_plugin_dir_name(&binding), "my_plugin");
    }
}

#[tauri::command]
pub fn get_enabled_plugins(app: AppHandle, project_id: String) -> Result<Vec<String>, String> {
    let storage = get_storage(&app);
    let projects: Vec<Project> = storage.load_or_default("projects.json");
    let project = projects.iter().find(|p| p.project_id == project_id)
        .ok_or_else(|| "Project not found".to_string())?;

    let project_godot = Path::new(&project.path).join("project.godot");
    if !project_godot.exists() {
        return Ok(vec![]);
    }

    let content = fs::read_to_string(&project_godot)
        .map_err(|e| format!("Failed to read project.godot: {}", e))?;

    let mut enabled = Vec::new();
    let mut in_section = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "[editor_plugins]" {
            in_section = true;
            continue;
        }
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_section = false;
            continue;
        }
        if in_section && !trimmed.is_empty() && !trimmed.starts_with(';') {
            if let Some(eq_pos) = trimmed.find('=') {
                let key = trimmed[..eq_pos].trim();
                let val = trimmed[eq_pos + 1..].trim();
                if key == "enabled" && val.starts_with("PackedStringArray") {
                    let entries = parse_enabled_entries(val);
                    enabled.extend(entries);
                }
            }
        }
    }

    Ok(enabled)
}

fn derive_plugin_dir_name(binding: &ProjectBinding) -> String {
    let dir_name = binding.mount_path
        .replace('\\', "/")
        .trim_end_matches('/')
        .split('/')
        .last()
        .unwrap_or(&binding.mount_path)
        .to_string();
    if dir_name.is_empty() || dir_name.contains("res://") || dir_name.contains("plugin.cfg") {
        binding.mount_path.replace('\\', "/")
            .split('/')
            .filter(|s| !s.is_empty() && *s != "addons" && !s.starts_with("res:"))
            .last()
            .unwrap_or(&binding.mount_path)
            .to_string()
    } else {
        dir_name
    }
}

#[tauri::command]
pub fn enable_plugin_in_project(
    app: AppHandle,
    project_id: String,
    plugin_id: String,
) -> Result<bool, String> {
    let storage = get_storage(&app);
    let projects: Vec<Project> = storage.load_or_default("projects.json");
    let project = projects.iter().find(|p| p.project_id == project_id)
        .ok_or_else(|| "Project not found".to_string())?;

    let plugins: Vec<Plugin> = storage.load_or_default("plugins.json");
    let plugin = plugins.iter().find(|p| p.plugin_id == plugin_id)
        .ok_or_else(|| "Plugin not found".to_string())?;

    let bindings: Vec<ProjectBinding> = storage.load_or_default("bindings.json");
    let binding = bindings.iter().find(|b| b.project_id == project_id && b.plugin_id == plugin_id)
        .ok_or_else(|| "Binding not found".to_string())?;

    let plugin_dir_name = derive_plugin_dir_name(binding);

    match modify_editor_plugins(&project.path, &plugin_dir_name, true) {
        Ok(result) => {
            log_operation(&app, "enable_plugin", &project_id,
                &format!("Enabled plugin {} in project {}", plugin.name, project.name));
            Ok(result)
        }
        Err(e) => {
            log_operation(&app, "enable_plugin_failed", &project_id,
                &format!("Failed to enable plugin {} in project {}: {}", plugin.name, project.name, e));
            Err(e)
        }
    }
}

#[tauri::command]
pub fn disable_plugin_in_project(
    app: AppHandle,
    project_id: String,
    plugin_id: String,
) -> Result<bool, String> {
    let storage = get_storage(&app);
    let projects: Vec<Project> = storage.load_or_default("projects.json");
    let project = projects.iter().find(|p| p.project_id == project_id)
        .ok_or_else(|| "Project not found".to_string())?;

    let plugins: Vec<Plugin> = storage.load_or_default("plugins.json");
    let plugin = plugins.iter().find(|p| p.plugin_id == plugin_id)
        .ok_or_else(|| "Plugin not found".to_string())?;

    let bindings: Vec<ProjectBinding> = storage.load_or_default("bindings.json");
    let binding = bindings.iter().find(|b| b.project_id == project_id && b.plugin_id == plugin_id)
        .ok_or_else(|| "Binding not found".to_string())?;

    let plugin_dir_name = derive_plugin_dir_name(binding);

    match modify_editor_plugins(&project.path, &plugin_dir_name, false) {
        Ok(result) => {
            log_operation(&app, "disable_plugin", &project_id,
                &format!("Disabled plugin {} in project {}", plugin.name, project.name));
            Ok(result)
        }
        Err(e) => {
            log_operation(&app, "disable_plugin_failed", &project_id,
                &format!("Failed to disable plugin {} in project {}: {}", plugin.name, project.name, e));
            Err(e)
        }
    }
}


