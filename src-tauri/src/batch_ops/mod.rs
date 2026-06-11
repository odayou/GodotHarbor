use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;
use chrono::Utc;

use crate::models::*;

// ─── Data Models ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentSnapshot {
    pub snapshot_id: String,
    pub project_id: String,
    pub project_name: String,
    pub created_at: String,
    pub godot_version: String,
    pub engine: Option<SnapshotEngine>,
    pub plugins: Vec<SnapshotPlugin>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotEngine {
    pub engine_id: String,
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotPlugin {
    pub plugin_id: String,
    pub plugin_name: String,
    pub version_id: String,
    pub version: String,
    pub unit_id: String,
    pub unit_name: String,
    pub mount_path: String,
    pub content_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalUpgradeResult {
    pub plugin_name: String,
    pub old_version: String,
    pub new_version: String,
    pub affected_projects: Vec<String>,
    pub success: bool,
    pub error: Option<String>,
}

// ─── Core Functions ───

pub fn create_environment_snapshot(
    project: &Project,
    bindings: &[ProjectBinding],
    plugins: &[Plugin],
    engines: &[Engine],
) -> EnvironmentSnapshot {
    let engine_info = project.last_used_engine_id.as_ref().and_then(|eid| {
        engines.iter().find(|e| e.engine_id == *eid).map(|e| SnapshotEngine {
            engine_id: e.engine_id.clone(),
            name: e.name.clone(),
            version: e.version.clone(),
        })
    });

    let snapshot_plugins: Vec<SnapshotPlugin> = bindings
        .iter()
        .filter(|b| b.project_id == project.project_id)
        .map(|binding| {
            let plugin = plugins.iter().find(|p| p.plugin_id == binding.plugin_id);
            let version = plugin.and_then(|p| {
                p.versions.iter().find(|v| v.version_id == binding.version_id)
            });
            let unit = version.and_then(|v| {
                v.units.iter().find(|u| u.unit_id == binding.unit_id)
            });

            SnapshotPlugin {
                plugin_id: binding.plugin_id.clone(),
                plugin_name: plugin.map(|p| p.name.clone()).unwrap_or_default(),
                version_id: binding.version_id.clone(),
                version: version.map(|v| v.version.clone()).unwrap_or_default(),
                unit_id: binding.unit_id.clone(),
                unit_name: unit.map(|u| u.name.clone()).unwrap_or_default(),
                mount_path: binding.mount_path.clone(),
                content_hash: plugin.map(|p| p.content_hash.clone()).unwrap_or_default(),
            }
        })
        .collect();

    EnvironmentSnapshot {
        snapshot_id: Uuid::new_v4().to_string(),
        project_id: project.project_id.clone(),
        project_name: project.name.clone(),
        created_at: Utc::now().to_rfc3339(),
        godot_version: project.godot_version.clone(),
        engine: engine_info,
        plugins: snapshot_plugins,
    }
}

pub fn save_snapshot(data_dir: &PathBuf, snapshot: &EnvironmentSnapshot) -> Result<(), String> {
    let snapshot_dir = data_dir.join("snapshots").join(&snapshot.project_id);
    std::fs::create_dir_all(&snapshot_dir)
        .map_err(|e| format!("创建快照目录失败: {}", e))?;

    let file_path = snapshot_dir.join(format!("{}.json", snapshot.snapshot_id));
    let content = serde_json::to_string_pretty(snapshot)
        .map_err(|e| format!("序列化快照失败: {}", e))?;

    std::fs::write(&file_path, content)
        .map_err(|e| format!("写入快照文件失败: {}", e))?;

    Ok(())
}

pub fn load_snapshots(data_dir: &PathBuf, project_id: &str) -> Result<Vec<EnvironmentSnapshot>, String> {
    let snapshot_dir = data_dir.join("snapshots").join(project_id);
    if !snapshot_dir.exists() {
        return Ok(Vec::new());
    }

    let mut snapshots = Vec::new();
    let entries = std::fs::read_dir(&snapshot_dir)
        .map_err(|e| format!("读取快照目录失败: {}", e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().map(|e| e == "json").unwrap_or(false) {
            let content = std::fs::read_to_string(&path)
                .map_err(|e| format!("读取快照文件失败: {}", e))?;
            let snapshot: EnvironmentSnapshot = serde_json::from_str(&content)
                .map_err(|e| format!("解析快照文件失败: {}", e))?;
            snapshots.push(snapshot);
        }
    }

    snapshots.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(snapshots)
}

pub fn restore_from_snapshot(
    snapshot: &EnvironmentSnapshot,
    storage: &crate::storage::Storage,
) -> Result<Vec<String>, String> {
    let mut bindings: Vec<ProjectBinding> = storage.load_or_default("bindings.json");
    let plugins: Vec<Plugin> = storage.load_or_default("plugins.json");

    // Remove all existing bindings for this project
    bindings.retain(|b| b.project_id != snapshot.project_id);

    // Recreate bindings from snapshot
    let mut restored = Vec::new();
    for sp in &snapshot.plugins {
        // Verify plugin still exists
        if let Some(_plugin) = plugins.iter().find(|p| p.plugin_id == sp.plugin_id) {
            let binding = ProjectBinding::new(
                snapshot.project_id.clone(),
                sp.plugin_id.clone(),
                sp.version_id.clone(),
                sp.unit_id.clone(),
                sp.mount_path.clone(),
                String::new(),
            );
            bindings.push(binding);
            restored.push(sp.plugin_name.clone());
        }
    }

    storage.save("bindings.json", &bindings)
        .map_err(|e| format!("保存绑定列表失败: {}", e))?;

    Ok(restored)
}

pub fn delete_snapshot_file(data_dir: &PathBuf, snapshot_id: &str, project_id: &str) -> Result<(), String> {
    let file_path = data_dir.join("snapshots").join(project_id).join(format!("{}.json", snapshot_id));
    if file_path.exists() {
        std::fs::remove_file(&file_path)
            .map_err(|e| format!("删除快照文件失败: {}", e))?;
    } else {
        return Err("快照文件不存在".to_string());
    }
    Ok(())
}

pub fn global_upgrade_plugin(
    plugin_id: &str,
    plugins: &[Plugin],
    bindings: &[ProjectBinding],
    storage: &crate::storage::Storage,
) -> Vec<GlobalUpgradeResult> {
    let plugin = match plugins.iter().find(|p| p.plugin_id == plugin_id) {
        Some(p) => p,
        None => {
            return vec![GlobalUpgradeResult {
                plugin_name: plugin_id.to_string(),
                old_version: String::new(),
                new_version: String::new(),
                affected_projects: Vec::new(),
                success: false,
                error: Some("未找到插件".to_string()),
            }];
        }
    };

    let latest_version = match plugin.versions.last() {
        Some(v) => v,
        None => {
            return vec![GlobalUpgradeResult {
                plugin_name: plugin.name.clone(),
                old_version: String::new(),
                new_version: String::new(),
                affected_projects: Vec::new(),
                success: false,
                error: Some("插件没有可用版本".to_string()),
            }];
        }
    };

    let affected_bindings: Vec<&ProjectBinding> = bindings
        .iter()
        .filter(|b| b.plugin_id == plugin_id && b.version_id != latest_version.version_id)
        .collect();

    if affected_bindings.is_empty() {
        return vec![GlobalUpgradeResult {
            plugin_name: plugin.name.clone(),
            old_version: String::new(),
            new_version: latest_version.version.clone(),
            affected_projects: Vec::new(),
            success: true,
            error: None,
        }];
    }

    let mut results = Vec::new();
    let mut all_bindings: Vec<ProjectBinding> = storage.load_or_default("bindings.json");
    let mut projects_map: std::collections::HashMap<String, String> = std::collections::HashMap::new();

    let projects: Vec<Project> = storage.load_or_default("projects.json");
    for p in &projects {
        projects_map.insert(p.project_id.clone(), p.name.clone());
    }

    for binding in &affected_bindings {
        let old_version = plugin.versions.iter()
            .find(|v| v.version_id == binding.version_id)
            .map(|v| v.version.clone())
            .unwrap_or_default();

        let project_name = projects_map.get(&binding.project_id).cloned().unwrap_or_default();

        if let Some(b) = all_bindings.iter_mut().find(|b| {
            b.project_id == binding.project_id
                && b.plugin_id == binding.plugin_id
                && b.version_id == binding.version_id
        }) {
            b.version_id = latest_version.version_id.clone();
            results.push(GlobalUpgradeResult {
                plugin_name: plugin.name.clone(),
                old_version,
                new_version: latest_version.version.clone(),
                affected_projects: vec![project_name],
                success: true,
                error: None,
            });
        }
    }

    if let Err(e) = storage.save("bindings.json", &all_bindings) {
        for r in &mut results {
            r.success = false;
            r.error = Some(format!("保存绑定列表失败: {}", e));
        }
    }

    results
}
