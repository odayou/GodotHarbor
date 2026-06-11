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
pub struct EnvironmentDiff {
    pub project_a: String,
    pub project_a_name: String,
    pub project_b: String,
    pub project_b_name: String,
    pub only_in_a: Vec<DiffPlugin>,
    pub only_in_b: Vec<DiffPlugin>,
    pub different_version: Vec<DiffVersionChange>,
    pub same: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffPlugin {
    pub plugin_name: String,
    pub version: String,
    pub mount_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffVersionChange {
    pub plugin_name: String,
    pub version_a: String,
    pub version_b: String,
    pub mount_path_a: String,
    pub mount_path_b: String,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchProjectInitResult {
    pub results: Vec<ProjectInitResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInitResult {
    pub project_name: String,
    pub success: bool,
    pub error: Option<String>,
    pub plugins_installed: usize,
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

pub fn compare_environments(
    project_a: &Project,
    bindings_a: &[ProjectBinding],
    project_b: &Project,
    bindings_b: &[ProjectBinding],
    plugins: &[Plugin],
) -> EnvironmentDiff {
    let get_plugin_info = |binding: &ProjectBinding| -> (String, String, String) {
        let plugin = plugins.iter().find(|p| p.plugin_id == binding.plugin_id);
        let name = plugin.map(|p| p.name.clone()).unwrap_or_default();
        let version = plugin.and_then(|p| {
            p.versions.iter().find(|v| v.version_id == binding.version_id)
        }).map(|v| v.version.clone()).unwrap_or_default();
        (name, version, binding.mount_path.clone())
    };

    let a_map: std::collections::HashMap<String, (String, String, String)> = bindings_a
        .iter()
        .filter(|b| b.project_id == project_a.project_id)
        .map(|b| {
            let (name, version, mount) = get_plugin_info(b);
            (b.plugin_id.clone(), (name, version, mount))
        })
        .collect();

    let b_map: std::collections::HashMap<String, (String, String, String)> = bindings_b
        .iter()
        .filter(|b| b.project_id == project_b.project_id)
        .map(|b| {
            let (name, version, mount) = get_plugin_info(b);
            (b.plugin_id.clone(), (name, version, mount))
        })
        .collect();

    let mut only_in_a = Vec::new();
    let mut only_in_b = Vec::new();
    let mut different_version = Vec::new();
    let mut same = Vec::new();

    for (plugin_id, (name, version_a, mount_a)) in &a_map {
        if let Some((_, version_b, mount_b)) = b_map.get(plugin_id) {
            if version_a == version_b {
                same.push(name.clone());
            } else {
                different_version.push(DiffVersionChange {
                    plugin_name: name.clone(),
                    version_a: version_a.clone(),
                    version_b: version_b.clone(),
                    mount_path_a: mount_a.clone(),
                    mount_path_b: mount_b.clone(),
                });
            }
        } else {
            only_in_a.push(DiffPlugin {
                plugin_name: name.clone(),
                version: version_a.clone(),
                mount_path: mount_a.clone(),
            });
        }
    }

    for (plugin_id, (name, version_b, mount_b)) in &b_map {
        if !a_map.contains_key(plugin_id) {
            only_in_b.push(DiffPlugin {
                plugin_name: name.clone(),
                version: version_b.clone(),
                mount_path: mount_b.clone(),
            });
        }
    }

    EnvironmentDiff {
        project_a: project_a.project_id.clone(),
        project_a_name: project_a.name.clone(),
        project_b: project_b.project_id.clone(),
        project_b_name: project_b.name.clone(),
        only_in_a,
        only_in_b,
        different_version,
        same,
    }
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

    let latest_version = match plugin.versions.first() {
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

pub fn batch_init_from_template(
    template_id: &str,
    project_names: &[String],
    base_dir: &str,
    storage: &crate::storage::Storage,
) -> BatchProjectInitResult {
    let templates: Vec<Template> = storage.load_or_default("hub_templates.json");
    let template = match templates.iter().find(|t| t.template_id == template_id) {
        Some(t) => t,
        None => {
            return BatchProjectInitResult {
                results: project_names.iter().map(|name| ProjectInitResult {
                    project_name: name.clone(),
                    success: false,
                    error: Some("未找到模板".to_string()),
                    plugins_installed: 0,
                }).collect(),
            };
        }
    };

    let mut results = Vec::new();
    let base_path = PathBuf::from(base_dir);

    for name in project_names {
        let project_dir = base_path.join(name);

        if project_dir.exists() {
            results.push(ProjectInitResult {
                project_name: name.clone(),
                success: false,
                error: Some(format!("目录已存在: {}", project_dir.display())),
                plugins_installed: 0,
            });
            continue;
        }

        // Create project directory
        if let Err(e) = std::fs::create_dir_all(&project_dir) {
            results.push(ProjectInitResult {
                project_name: name.clone(),
                success: false,
                error: Some(format!("创建项目目录失败: {}", e)),
                plugins_installed: 0,
            });
            continue;
        }

        // Create project.godot
        let godot_content = format!(
            "; Engine configuration file.\n; It's best edited using the editor UI and not directly,\n; since the parameters that go here are not all obvious.\n;\n; Format:\n;   [section] ; section goes between []\n;   param=value ; assign values to parameters\n\nconfig/features=PackedStringArray(\"4.2\", \"Godot Engine\")\n\n[application]\n\nconfig/name=\"{}\"\nconfig/icon=\"res://icon.svg\"\n\n[display]\n\nwindow/size/viewport_width=1152\nwindow/size/viewport_height=648\n",
            name
        );

        if let Err(e) = std::fs::write(project_dir.join("project.godot"), &godot_content) {
            let _ = std::fs::remove_dir_all(&project_dir);
            results.push(ProjectInitResult {
                project_name: name.clone(),
                success: false,
                error: Some(format!("创建 project.godot 失败: {}", e)),
                plugins_installed: 0,
            });
            continue;
        }

        // Create directories from template
        let plugins_installed = 0;
        for dir in &template.directories {
            let dir_path = project_dir.join(&dir.path);
            let _ = std::fs::create_dir_all(&dir_path);
        }

        // Create addons directory
        let _ = std::fs::create_dir_all(project_dir.join("addons"));

        // Register project
        let project = Project::new(
            name.clone(),
            project_dir.to_string_lossy().to_string(),
            template.godot.version.clone(),
            String::new(),
        );

        let mut projects: Vec<Project> = storage.load_or_default("projects.json");
        projects.push(project);

        if let Err(e) = storage.save("projects.json", &projects) {
            results.push(ProjectInitResult {
                project_name: name.clone(),
                success: false,
                error: Some(format!("保存项目列表失败: {}", e)),
                plugins_installed: 0,
            });
            continue;
        }

        results.push(ProjectInitResult {
            project_name: name.clone(),
            success: true,
            error: None,
            plugins_installed,
        });
    }

    BatchProjectInitResult { results }
}
