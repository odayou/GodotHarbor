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
        exported: config.plugins.len() as u32,
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

    let config_upgraded = if config.version < 2 {
        config.upgrade_to_v2()
    } else {
        config
    };

    for plugin_config in &config_upgraded.plugins {
        let source_url = if plugin_config.url.is_empty() {
            match plugin_config.source.as_str() {
                "git" | "asset-store" | "url" => continue,
                _ => continue,
            }
        } else {
            &plugin_config.url
        };

        if plugin_config.source == "local" {
            errors.push(format!("插件 \"{}\" 为本地导入类型，无法通过配置文件自动安装，需手动导入", plugin_config.name));
            skipped += 1;
            continue;
        }

        let existing_plugin = plugins.iter().find(|p| {
            let source_match = match p.source.source_type {
                crate::models::SourceType::Git => p.source.url == *source_url,
                crate::models::SourceType::AssetLibrary => {
                    let id = source_url.trim_start_matches("asset-library:");
                    p.source.url == format!("asset-library://{}", id) || p.source.url == *source_url
                }
                _ => p.source.url == *source_url,
            };
            source_match || p.name.to_lowercase() == plugin_config.name.to_lowercase()
        });

        let plugin_id = if let Some(plugin) = existing_plugin {
            plugin.plugin_id.clone()
        } else {
            let source_type = if plugin_config.source == "asset-store" || source_url.starts_with("asset-library:") {
                crate::models::SourceType::AssetLibrary
            } else if plugin_config.source == "git" || source_url.ends_with(".git") || source_url.contains("github.com") {
                crate::models::SourceType::Git
            } else {
                crate::models::SourceType::Url
            };

            let plugin_source = crate::models::PluginSource {
                source_type: source_type.clone(),
                url: source_url.clone(),
                git_ref: plugin_config.r#ref.clone(),
                imported_at: chrono::Utc::now(),
            };

            let mut plugin = Plugin::new(plugin_config.name.clone(), plugin_source);
            plugin.asset_type = plugin_config.asset_type.clone();

            match source_type {
                crate::models::SourceType::Git => {
                    let manager = get_plugin_manager(&app);
                    let git_ref = if plugin_config.r#ref.is_empty() { None } else { Some(plugin_config.r#ref.as_str()) };
                    match manager.import_from_git(source_url, git_ref, &app) {
                        Ok(imported_plugin) => {
                            plugin = imported_plugin;
                            imported += 1;
                        }
                        Err(e) => {
                            errors.push(format!("导入 {} 失败: {}", plugin_config.name, e));
                            skipped += 1;
                            continue;
                        }
                    }
                }
                _ => {
                    let version_id = uuid::Uuid::new_v4().to_string();
                    let version = crate::models::PluginVersion {
                        version_id: version_id.clone(),
                        version: plugin_config.version.clone(),
                        path: source_url.clone(),
                        created_at: chrono::Utc::now(),
                        units: vec![crate::models::PluginUnit {
                            unit_id: uuid::Uuid::new_v4().to_string(),
                            name: plugin_config.name.clone(),
                            dir_name: plugin_config.name.clone(),
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

        let plugin_ref = plugins.iter().find(|p| p.plugin_id == plugin_id);
        let mount_path = plugin_ref
            .and_then(|p| p.versions.first())
            .and_then(|v| v.units.first())
            .map(|u| {
                if u.subdirectory.is_empty() {
                    format!("addons/{}", u.dir_name)
                } else {
                    u.subdirectory.clone()
                }
            })
            .unwrap_or_else(|| format!("addons/{}", plugin_config.name));
        let already_bound = bindings.iter().any(|b| b.project_id == project_id && b.plugin_id == plugin_id);
        let mount_conflict = bindings.iter().any(|b| {
            b.project_id == project_id && b.plugin_id != plugin_id && b.mount_path == mount_path
        });
        if mount_conflict {
            errors.push(format!(
                "插件 \"{}\" 的挂载路径 {} 已被其他插件占用，跳过绑定",
                plugin_config.name, mount_path
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
                    mount_path,
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

#[tauri::command]
pub fn check_project_drift(app: AppHandle, project_id: String) -> Result<crate::models::DriftReport, String> {
    let storage = get_storage(&app);
    let projects: Vec<crate::models::Project> = storage.load_or_default("projects.json");
    let project = projects.iter().find(|p| p.project_id == project_id)
        .ok_or("项目不存在".to_string())?;

    let config = harbor_config::read_harbor_config_from_project(&project.path)
        .map_err(|e| format!("读取 .harbor.yml 失败: {}", e))?;

    let plugins: Vec<Plugin> = storage.load_or_default("plugins.json");
    let bindings: Vec<crate::models::ProjectBinding> = storage.load_or_default("bindings.json");
    let engines: Vec<crate::models::Engine> = storage.load_or_default("engines.json");
    let project_bindings: Vec<&crate::models::ProjectBinding> = bindings.iter()
        .filter(|b| b.project_id == project_id)
        .collect();

    let mut items = Vec::new();

    if let Some(ref config) = config {
        let config_upgraded = if config.version < 2 { config.upgrade_to_v2() } else { config.clone() };

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
            if let Some(engine) = engine_match {
                if engine.version != godot_cfg.version {
                    items.push(crate::models::DriftItem {
                        item_type: "engine".to_string(),
                        name: "Godot".to_string(),
                        status: crate::models::DriftStatus::VersionMismatch,
                        expected: godot_cfg.version.clone(),
                        actual: engine.version.clone(),
                        message: format!("引擎版本不一致：声明 {}，实际 {}", godot_cfg.version, engine.version),
                    });
                }
            } else {
                items.push(crate::models::DriftItem {
                    item_type: "engine".to_string(),
                    name: "Godot".to_string(),
                    status: crate::models::DriftStatus::Missing,
                    expected: godot_cfg.version.clone(),
                    actual: "未安装".to_string(),
                    message: format!("声明的引擎 Godot {} 未安装", godot_cfg.version),
                });
            }
        }

        for plugin_cfg in &config_upgraded.plugins {
            let binding = project_bindings.iter().find(|b| {
                let plugin = plugins.iter().find(|p| p.plugin_id == b.plugin_id);
                plugin.map_or(false, |p| p.name.to_lowercase() == plugin_cfg.name.to_lowercase())
            });

            if let Some(binding) = binding {
                let plugin = plugins.iter().find(|p| p.plugin_id == binding.plugin_id);
                if let Some(plugin) = plugin {
                    if !plugin_cfg.version.is_empty() {
                        if let Some(version) = plugin.versions.first() {
                            if version.version != plugin_cfg.version {
                                items.push(crate::models::DriftItem {
                                    item_type: "plugin".to_string(),
                                    name: plugin_cfg.name.clone(),
                                    status: crate::models::DriftStatus::VersionMismatch,
                                    expected: plugin_cfg.version.clone(),
                                    actual: version.version.clone(),
                                    message: format!("插件 {} 版本不一致：声明 {}，实际 {}", plugin_cfg.name, plugin_cfg.version, version.version),
                                });
                            }
                        }
                    }
                }
            } else {
                items.push(crate::models::DriftItem {
                    item_type: "plugin".to_string(),
                    name: plugin_cfg.name.clone(),
                    status: crate::models::DriftStatus::Missing,
                    expected: plugin_cfg.version.clone(),
                    actual: "未安装".to_string(),
                    message: format!("声明的插件 {} 未安装", plugin_cfg.name),
                });
            }
        }

        for binding in &project_bindings {
            let plugin = plugins.iter().find(|p| p.plugin_id == binding.plugin_id);
            let plugin_name = plugin.map_or("unknown", |p| &p.name);
            let in_config = config_upgraded.plugins.iter().any(|pc| pc.name.to_lowercase() == plugin_name.to_lowercase());
            if !in_config {
                items.push(crate::models::DriftItem {
                    item_type: "plugin".to_string(),
                    name: plugin_name.to_string(),
                    status: crate::models::DriftStatus::Unexpected,
                    expected: "未声明".to_string(),
                    actual: "已安装".to_string(),
                    message: format!("插件 {} 已安装但未在 .harbor.yml 中声明", plugin_name),
                });
            }
        }
    } else {
        items.push(crate::models::DriftItem {
            item_type: "config".to_string(),
            name: ".harbor.yml".to_string(),
            status: crate::models::DriftStatus::Missing,
            expected: "存在".to_string(),
            actual: "不存在".to_string(),
            message: "项目缺少 .harbor.yml 配置文件".to_string(),
        });
    }

    let has_drift = items.iter().any(|i| i.status != crate::models::DriftStatus::InSync);

    Ok(crate::models::DriftReport {
        project_id: project.project_id.clone(),
        project_name: project.name.clone(),
        items,
        checked_at: chrono::Utc::now(),
        has_drift,
    })
}

#[tauri::command]
pub async fn check_all_drifts(app: AppHandle) -> Result<Vec<crate::models::DriftReport>, String> {
    let app_clone = app.clone();
    tokio::task::spawn_blocking(move || {
        let storage = get_storage(&app_clone);
        let projects: Vec<crate::models::Project> = storage.load_or_default("projects.json");
        let mut reports = Vec::new();
        for project in &projects {
            let report = check_project_drift(app_clone.clone(), project.project_id.clone())?;
            reports.push(report);
        }
        Ok(reports)
    })
    .await
    .map_err(|e| format!("任务执行失败: {}", e))?
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncPreview {
    pub project_id: String,
    pub actions: Vec<SyncAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncAction {
    pub action_type: String,
    pub item_type: String,
    pub name: String,
    pub detail: String,
}

#[tauri::command]
pub fn preview_sync(app: AppHandle, project_id: String) -> Result<SyncPreview, String> {
    let report = check_project_drift(app.clone(), project_id.clone())?;
    let mut actions = Vec::new();

    for item in &report.items {
        match item.status {
            crate::models::DriftStatus::Missing => {
                actions.push(SyncAction {
                    action_type: "install".to_string(),
                    item_type: item.item_type.clone(),
                    name: item.name.clone(),
                    detail: format!("安装 {} ({})", item.name, item.expected),
                });
            }
            crate::models::DriftStatus::VersionMismatch => {
                actions.push(SyncAction {
                    action_type: "update".to_string(),
                    item_type: item.item_type.clone(),
                    name: item.name.clone(),
                    detail: format!("{} {} → {}", item.name, item.actual, item.expected),
                });
            }
            crate::models::DriftStatus::Unexpected => {
                actions.push(SyncAction {
                    action_type: "remove".to_string(),
                    item_type: item.item_type.clone(),
                    name: item.name.clone(),
                    detail: format!("移除未声明的 {}", item.name),
                });
            }
            crate::models::DriftStatus::InSync => {}
        }
    }

    Ok(SyncPreview {
        project_id,
        actions,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncEnvironmentResult {
    pub synced: u32,
    pub failed: u32,
    pub skipped: u32,
    pub details: Vec<String>,
}

#[tauri::command]
pub fn sync_project_environment(app: AppHandle, project_id: String, only_items: Option<Vec<String>>) -> Result<SyncEnvironmentResult, String> {
    let report = check_project_drift(app.clone(), project_id.clone())?;
    let storage = get_storage(&app);
    let projects: Vec<crate::models::Project> = storage.load_or_default("projects.json");
    let project = projects.iter().find(|p| p.project_id == project_id)
        .ok_or("项目不存在".to_string())?;

    let config = harbor_config::read_harbor_config_from_project(&project.path)
        .map_err(|e| format!("读取 .harbor.yml 失败: {}", e))?
        .ok_or("项目缺少 .harbor.yml".to_string())?;

    let config_upgraded = if config.version < 2 { config.upgrade_to_v2() } else { config };

    let mut synced = 0u32;
    let mut failed = 0u32;
    let mut skipped = 0u32;
    let mut details = Vec::new();

    for item in &report.items {
        if let Some(ref items) = only_items {
            let item_key = format!("{}:{}", item.item_type, item.name);
            if !items.contains(&item_key) {
                continue;
            }
        }
        match item.status {
            crate::models::DriftStatus::Missing => {
                if item.item_type == "plugin" {
                    let plugin_cfg = config_upgraded.plugins.iter().find(|p| p.name == item.name);
                    if let Some(pc) = plugin_cfg {
                        if pc.source == "local" {
                            details.push(format!("插件 {} 为本地来源，需手动安装", pc.name));
                            skipped += 1;
                            continue;
                        }
                        details.push(format!("正在安装插件 {}...", pc.name));
                    } else {
                        details.push(format!("未找到插件 {} 的配置信息", item.name));
                        failed += 1;
                        continue;
                    }
                } else if item.item_type == "engine" {
                    details.push(format!("引擎 {} 需手动下载或通过模板实例化安装", item.expected));
                    skipped += 1;
                    continue;
                } else if item.item_type == "config" {
                    let result = write_harbor_config(app.clone(), project_id.clone())?;
                    details.push(format!("已生成 .harbor.yml（导出 {} 个插件配置）", result.exported));
                    synced += 1;
                    continue;
                }
                synced += 1;
            }
            crate::models::DriftStatus::VersionMismatch => {
                if item.item_type == "engine" {
                    details.push(format!("引擎版本不一致（{} → {}），请手动下载对应版本", item.actual, item.expected));
                    skipped += 1;
                } else if item.item_type == "plugin" {
                    let plugin_cfg = config_upgraded.plugins.iter().find(|p| p.name == item.name);
                    if let Some(pc) = plugin_cfg {
                        if pc.source == "local" {
                            details.push(format!("插件 {} 为本地来源，版本不一致需手动更新", pc.name));
                            skipped += 1;
                        } else {
                            details.push(format!("插件 {} 版本不一致（{} → {}），将重新导入", item.name, item.actual, item.expected));
                            synced += 1;
                        }
                    } else {
                        details.push(format!("{} 版本不一致（{} → {}），需手动处理", item.name, item.actual, item.expected));
                        skipped += 1;
                    }
                } else {
                    details.push(format!("{} 版本不一致（{} → {}），需手动处理", item.name, item.actual, item.expected));
                    skipped += 1;
                }
            }
            crate::models::DriftStatus::Unexpected => {
                details.push(format!("将插件 {} 添加到 .harbor.yml 声明中", item.name));
                let mut updated = config_upgraded.clone();
                let plugins: Vec<Plugin> = storage.load_or_default("plugins.json");
                let bindings: Vec<crate::models::ProjectBinding> = storage.load_or_default("bindings.json");
                let binding = bindings.iter().find(|b| {
                    b.project_id == project_id && plugins.iter()
                        .find(|p| p.plugin_id == b.plugin_id)
                        .map_or(false, |p| p.name.to_lowercase() == item.name.to_lowercase())
                });
                if let Some(binding) = binding {
                    let plugin = plugins.iter().find(|p| p.plugin_id == binding.plugin_id);
                    let version = plugin.and_then(|p| p.versions.first()).map(|v| v.version.clone()).unwrap_or_default();
                    let source = plugin.map(|p| match p.source.source_type {
                        crate::models::SourceType::Git => "git",
                        crate::models::SourceType::AssetLibrary => "asset-store",
                        crate::models::SourceType::Url => "url",
                        crate::models::SourceType::Local => "local",
                    }).unwrap_or("local").to_string();
                    let url = plugin.map(|p| p.source.url.clone()).unwrap_or_default();
                    let git_ref = plugin.map(|p| p.source.git_ref.clone()).unwrap_or_default();
                    updated.plugins.push(harbor_config::HarborPlugin {
                        name: item.name.clone(),
                        version,
                        source,
                        url,
                        r#ref: git_ref,
                        asset_type: plugin.map(|p| p.asset_type.clone()).unwrap_or_default(),
                    });
                    let _ = harbor_config::write_harbor_config_to_project(&project.path, &updated);
                    synced += 1;
                } else {
                    skipped += 1;
                }
            }
            crate::models::DriftStatus::InSync => {}
        }
    }

    if report.items.iter().any(|i| i.status == crate::models::DriftStatus::Missing && i.item_type == "plugin") {
        let sync_result = sync_harbor_config(app.clone(), project_id.clone())?;
        synced += sync_result.imported + sync_result.bound;
        failed += sync_result.skipped;
        for err in &sync_result.errors {
            details.push(err.clone());
        }
    }

    Ok(SyncEnvironmentResult {
        synced,
        failed,
        skipped,
        details,
    })
}
