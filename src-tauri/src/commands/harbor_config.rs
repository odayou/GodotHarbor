use crate::harbor_config::{self, HarborConfig};
use crate::models::Plugin;
use crate::commands::utils::{get_storage, get_plugin_manager};
use tauri::AppHandle;
use serde::{Deserialize, Serialize};

#[tauri::command]
pub fn read_harbor_config(app: AppHandle, project_id: String) -> Result<Option<HarborConfig>, String> {
    let storage = get_storage(&app);
    let projects: Vec<crate::models::Project> = storage.load_or_default("projects.json");
    let project = projects.iter().find(|p| p.project_id == project_id)
        .ok_or("项目不存在".to_string())?;

    harbor_config::read_harbor_config_from_project(&project.path)
        .map_err(|e| format!("读取 .harbor.yml 失败: {}", e))
}

#[tauri::command]
pub fn read_harbor_config_raw(app: AppHandle, project_id: String) -> Result<Option<String>, String> {
    let storage = get_storage(&app);
    let projects: Vec<crate::models::Project> = storage.load_or_default("projects.json");
    let project = projects.iter().find(|p| p.project_id == project_id)
        .ok_or("项目不存在".to_string())?;

    let config_path = harbor_config::get_harbor_config_path(&project.path);
    if !config_path.exists() {
        return Ok(None);
    }
    std::fs::read_to_string(&config_path)
        .map(Some)
        .map_err(|e| format!("读取 .harbor.yml 失败: {}", e))
}

#[tauri::command]
pub fn delete_harbor_config(app: AppHandle, project_id: String) -> Result<(), String> {
    let storage = get_storage(&app);
    let projects: Vec<crate::models::Project> = storage.load_or_default("projects.json");
    let project = projects.iter().find(|p| p.project_id == project_id)
        .ok_or("项目不存在".to_string())?;

    let config_path = harbor_config::get_harbor_config_path(&project.path);
    if config_path.exists() {
        std::fs::remove_file(&config_path)
            .map_err(|e| format!("删除 .harbor.yml 失败: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
pub fn check_harbor_configs(app: AppHandle, project_ids: Vec<String>) -> Result<std::collections::HashMap<String, bool>, String> {
    let storage = get_storage(&app);
    let projects: Vec<crate::models::Project> = storage.load_or_default("projects.json");
    let mut result = std::collections::HashMap::new();
    for pid in &project_ids {
        if let Some(project) = projects.iter().find(|p| &p.project_id == pid) {
            let config_path = harbor_config::get_harbor_config_path(&project.path);
            result.insert(pid.clone(), config_path.exists());
        }
    }
    Ok(result)
}

#[tauri::command]
pub fn write_harbor_config(app: AppHandle, project_id: String) -> Result<ExportResult, String> {
    let storage = get_storage(&app);
    let projects: Vec<crate::models::Project> = storage.load_or_default("projects.json");
    let project = projects.iter().find(|p| p.project_id == project_id)
        .ok_or("项目不存在".to_string())?;

    let plugins: Vec<Plugin> = storage.load_or_default("plugins.json");
    let bindings: Vec<crate::models::ProjectBinding> = storage.load_or_default("bindings.json");
    let (config, skipped_local) = harbor_config::generate_config_from_bindings(project, &plugins, &bindings);

    harbor_config::write_harbor_config_to_project(&project.path, &config)
        .map_err(|e| format!("写入 .harbor.yml 失败: {}", e))?;

    Ok(ExportResult {
        exported: config.bindings.len() as u32,
        skipped_local,
    })
}

#[tauri::command]
pub fn sync_harbor_config(app: AppHandle, project_id: String) -> Result<SyncResult, String> {
    let storage = get_storage(&app);
    let projects: Vec<crate::models::Project> = storage.load_or_default("projects.json");
    let project = projects.iter().find(|p| p.project_id == project_id)
        .ok_or("项目不存在".to_string())?;

    let config = harbor_config::read_harbor_config_from_project(&project.path)
        .map_err(|e| format!("读取 .harbor.yml 失败: {}", e))?
        .ok_or("项目根目录下未找到 .harbor.yml 文件".to_string())?;

    let mut plugins: Vec<Plugin> = storage.load_or_default("plugins.json");
    let mut bindings: Vec<crate::models::ProjectBinding> = storage.load_or_default("bindings.json");
    let mut imported = 0u32;
    let mut bound = 0u32;
    let mut skipped = 0u32;
    let mut errors: Vec<String> = Vec::new();

    for binding_config in &config.bindings {
        if binding_config.source.starts_with("local:") {
            errors.push(format!("插件 \"{}\" 为本地导入类型，无法通过配置文件自动安装，需手动导入", binding_config.name));
            skipped += 1;
            continue;
        }

        let existing_plugin = plugins.iter().find(|p| {
            let source_match = match p.source.source_type {
                crate::models::SourceType::Git => p.source.url == binding_config.source,
                crate::models::SourceType::AssetLibrary => {
                    let id = binding_config.source.trim_start_matches("asset-library:");
                    p.source.url == format!("asset-library://{}", id) || p.source.url == binding_config.source
                }
                _ => p.source.url == binding_config.source,
            };
            source_match || p.name.to_lowercase() == binding_config.name.to_lowercase()
        });

        let plugin_id = if let Some(plugin) = existing_plugin {
            plugin.plugin_id.clone()
        } else {
            let source_type = if binding_config.source.starts_with("asset-library:") {
                crate::models::SourceType::AssetLibrary
            } else if binding_config.source.ends_with(".git") || binding_config.source.contains("github.com") {
                crate::models::SourceType::Git
            } else {
                crate::models::SourceType::Url
            };

            let plugin_source = crate::models::PluginSource {
                source_type: source_type.clone(),
                url: binding_config.source.clone(),
                git_ref: binding_config.r#ref.clone(),
                imported_at: chrono::Utc::now(),
            };

            let mut plugin = Plugin::new(binding_config.name.clone(), plugin_source);
            plugin.asset_type = binding_config.asset_type.clone();

            match source_type {
                crate::models::SourceType::Git => {
                    let manager = get_plugin_manager(&app);
                    let git_ref = if binding_config.r#ref.is_empty() { None } else { Some(binding_config.r#ref.as_str()) };
                    match manager.import_from_git(&binding_config.source, git_ref, &app) {
                        Ok(imported_plugin) => {
                            plugin = imported_plugin;
                            imported += 1;
                        }
                        Err(e) => {
                            errors.push(format!("导入 {} 失败: {}", binding_config.name, e));
                            skipped += 1;
                            continue;
                        }
                    }
                }
                _ => {
                    let version_id = uuid::Uuid::new_v4().to_string();
                    let version = crate::models::PluginVersion {
                        version_id: version_id.clone(),
                        version: "1.0.0".to_string(),
                        path: binding_config.source.clone(),
                        created_at: chrono::Utc::now(),
                        units: vec![crate::models::PluginUnit {
                            unit_id: uuid::Uuid::new_v4().to_string(),
                            name: binding_config.name.clone(),
                            dir_name: binding_config.name.clone(),
                            description: String::new(),
                            author: String::new(),
                            version: String::new(),
                            subdirectory: String::new(),
                            plugin_cfg_path: String::new(),
                            is_virtual: true,
                        }],
                    };
                    plugin.versions.push(version);
                    imported += 1;
                }
            }

            let pid = plugin.plugin_id.clone();
            plugins.push(plugin);
            pid
        };

        let already_bound = bindings.iter().any(|b| b.project_id == project_id && b.plugin_id == plugin_id);
        let mount_conflict = bindings.iter().any(|b| {
            b.project_id == project_id && b.plugin_id != plugin_id && b.mount_path == binding_config.mount_path
        });
        if mount_conflict {
            errors.push(format!(
                "插件 \"{}\" 的挂载路径 {} 已被其他插件占用，跳过绑定",
                binding_config.name, binding_config.mount_path
            ));
            skipped += 1;
            continue;
        }
        if !already_bound {
            let plugin = plugins.iter().find(|p| p.plugin_id == plugin_id);
            let version = plugin.and_then(|p| p.versions.first());
            let unit = version.and_then(|v| v.units.first());

            if let (Some(version), Some(unit)) = (version, unit) {
                bindings.push(crate::models::ProjectBinding::new(
                    project_id.clone(),
                    plugin_id.clone(),
                    version.version_id.clone(),
                    unit.unit_id.clone(),
                    binding_config.mount_path.clone(),
                    String::new(),
                ));
                bound += 1;
            }
        }
    }

    storage.save("plugins.json", &plugins)
        .map_err(|e| format!("保存插件列表失败: {}", e))?;
    storage.save("bindings.json", &bindings)
        .map_err(|e| format!("保存绑定列表失败: {}", e))?;

    Ok(SyncResult {
        imported,
        bound,
        skipped,
        errors,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResult {
    pub exported: u32,
    pub skipped_local: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub imported: u32,
    pub bound: u32,
    pub skipped: u32,
    pub errors: Vec<String>,
}
