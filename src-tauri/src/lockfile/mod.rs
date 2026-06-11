use serde::{Deserialize, Serialize};
use std::path::Path;
use anyhow::{Context, Result};
use sha2::{Sha256, Digest};

use crate::models::{compute_dir_hash, Engine, Plugin, Project, ProjectBinding, SourceType};

// ─── Data Models ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarborLock {
    pub version: String,
    pub locked_at: String,
    pub project_name: String,
    pub project_path: String,
    pub godot_version: String,
    pub engine: Option<LockedEngine>,
    pub plugins: Vec<LockedPlugin>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedEngine {
    pub engine_id: String,
    pub version: String,
    pub engine_type: String,
    pub path_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedPlugin {
    pub plugin_id: String,
    pub plugin_name: String,
    pub version_id: String,
    pub version: String,
    pub unit_id: String,
    pub unit_name: String,
    pub mount_path: String,
    pub subdirectory: String,
    pub content_hash: String,
    pub source_type: String,
    pub source_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockDiff {
    pub added: Vec<LockedPlugin>,
    pub removed: Vec<LockedPlugin>,
    pub changed: Vec<LockChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockChange {
    pub plugin_name: String,
    pub field: String,
    pub old_value: String,
    pub new_value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockVerifyResult {
    pub is_valid: bool,
    pub mismatches: Vec<LockMismatch>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockMismatch {
    pub plugin_name: String,
    pub expected_hash: String,
    pub actual_hash: String,
    pub expected_version: String,
    pub actual_version: String,
}

// ─── Core Functions ───

fn compute_path_hash(path: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(path.as_bytes());
    format!("sha256:{:x}", hasher.finalize())
}

pub fn get_lock_path(project_path: &str) -> std::path::PathBuf {
    Path::new(project_path).join("harbor.lock")
}

pub fn generate_lock(
    project: &Project,
    bindings: &[ProjectBinding],
    plugins: &[Plugin],
    engines: &[Engine],
    engine_bindings: &[(String, String)],
) -> HarborLock {
    let project_bindings: Vec<&ProjectBinding> = bindings
        .iter()
        .filter(|b| b.project_id == project.project_id)
        .collect();

    let locked_engine = engine_bindings
        .iter()
        .find(|(pid, _)| pid == &project.project_id)
        .and_then(|(_, engine_id)| engines.iter().find(|e| e.engine_id == *engine_id))
        .map(|e| LockedEngine {
            engine_id: e.engine_id.clone(),
            version: e.version.clone(),
            engine_type: e.engine_type.to_string(),
            path_hash: compute_path_hash(&e.path),
        });

    let mut locked_plugins = Vec::new();
    for binding in &project_bindings {
        if let Some(plugin) = plugins.iter().find(|p| p.plugin_id == binding.plugin_id) {
            let version = plugin.versions.iter()
                .find(|v| v.version_id == binding.version_id);
            let unit = version
                .and_then(|v| v.units.iter().find(|u| u.unit_id == binding.unit_id));

            let version_str = version.map(|v| v.version.clone()).unwrap_or_default();
            let unit_name = unit.map(|u| u.name.clone()).unwrap_or_default();

            let content_hash = if let Some(ver) = version {
                let plugin_dir = Path::new(&ver.path);
                if plugin_dir.exists() {
                    compute_dir_hash(plugin_dir).unwrap_or_default()
                } else {
                    plugin.content_hash.clone()
                }
            } else {
                plugin.content_hash.clone()
            };

            let source_type = match plugin.source.source_type {
                SourceType::Git => "Git".to_string(),
                SourceType::Local => "Local".to_string(),
                SourceType::AssetLibrary => "AssetLibrary".to_string(),
                SourceType::Url => "Url".to_string(),
            };

            locked_plugins.push(LockedPlugin {
                plugin_id: plugin.plugin_id.clone(),
                plugin_name: plugin.name.clone(),
                version_id: binding.version_id.clone(),
                version: version_str,
                unit_id: binding.unit_id.clone(),
                unit_name,
                mount_path: binding.mount_path.clone(),
                subdirectory: binding.subdirectory.clone(),
                content_hash,
                source_type,
                source_url: plugin.source.url.clone(),
            });
        }
    }

    HarborLock {
        version: "1".to_string(),
        locked_at: chrono::Utc::now().to_rfc3339(),
        project_name: project.name.clone(),
        project_path: project.path.clone(),
        godot_version: project.godot_version.clone(),
        engine: locked_engine,
        plugins: locked_plugins,
    }
}

pub fn write_lock(project_path: &str, lock: &HarborLock) -> Result<()> {
    let lock_path = get_lock_path(project_path);
    let content = serde_yaml::to_string(lock)
        .context("序列化 harbor.lock 失败")?;
    std::fs::write(&lock_path, content)
        .with_context(|| format!("写入 harbor.lock 失败: {}", lock_path.to_string_lossy()))?;
    Ok(())
}

pub fn read_lock(project_path: &str) -> Result<Option<HarborLock>> {
    let lock_path = get_lock_path(project_path);
    if !lock_path.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(&lock_path)
        .with_context(|| format!("读取 harbor.lock 失败: {}", lock_path.to_string_lossy()))?;
    let lock: HarborLock = serde_yaml::from_str(&content)
        .context("解析 harbor.lock 失败")?;
    Ok(Some(lock))
}

pub fn verify_lock(
    _project_path: &str,
    lock: &HarborLock,
    plugins: &[Plugin],
) -> LockVerifyResult {
    let mut mismatches = Vec::new();

    for locked in &lock.plugins {
        if let Some(plugin) = plugins.iter().find(|p| p.plugin_id == locked.plugin_id) {
            let version = plugin.versions.iter()
                .find(|v| v.version_id == locked.version_id);

            if let Some(ver) = version {
                let actual_hash = compute_dir_hash(Path::new(&ver.path))
                    .unwrap_or_default();

                if actual_hash != locked.content_hash {
                    let actual_version = ver.version.clone();
                    mismatches.push(LockMismatch {
                        plugin_name: locked.plugin_name.clone(),
                        expected_hash: locked.content_hash.clone(),
                        actual_hash,
                        expected_version: locked.version.clone(),
                        actual_version,
                    });
                }
            } else {
                mismatches.push(LockMismatch {
                    plugin_name: locked.plugin_name.clone(),
                    expected_hash: locked.content_hash.clone(),
                    actual_hash: String::new(),
                    expected_version: locked.version.clone(),
                    actual_version: "未找到版本".to_string(),
                });
            }
        } else {
            mismatches.push(LockMismatch {
                plugin_name: locked.plugin_name.clone(),
                expected_hash: locked.content_hash.clone(),
                actual_hash: String::new(),
                expected_version: locked.version.clone(),
                actual_version: "未安装".to_string(),
            });
        }
    }

    LockVerifyResult {
        is_valid: mismatches.is_empty(),
        mismatches,
    }
}

pub fn diff_locks(old: &HarborLock, new: &HarborLock) -> LockDiff {
    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut changed = Vec::new();

    for new_plugin in &new.plugins {
        if let Some(old_plugin) = old.plugins.iter().find(|p| p.plugin_id == new_plugin.plugin_id) {
            if old_plugin.version != new_plugin.version {
                changed.push(LockChange {
                    plugin_name: new_plugin.plugin_name.clone(),
                    field: "version".to_string(),
                    old_value: old_plugin.version.clone(),
                    new_value: new_plugin.version.clone(),
                });
            }
            if old_plugin.content_hash != new_plugin.content_hash {
                changed.push(LockChange {
                    plugin_name: new_plugin.plugin_name.clone(),
                    field: "content_hash".to_string(),
                    old_value: old_plugin.content_hash.clone(),
                    new_value: new_plugin.content_hash.clone(),
                });
            }
            if old_plugin.mount_path != new_plugin.mount_path {
                changed.push(LockChange {
                    plugin_name: new_plugin.plugin_name.clone(),
                    field: "mount_path".to_string(),
                    old_value: old_plugin.mount_path.clone(),
                    new_value: new_plugin.mount_path.clone(),
                });
            }
        } else {
            added.push(new_plugin.clone());
        }
    }

    for old_plugin in &old.plugins {
        if !new.plugins.iter().any(|p| p.plugin_id == old_plugin.plugin_id) {
            removed.push(old_plugin.clone());
        }
    }

    LockDiff { added, removed, changed }
}

pub fn sync_from_lock(
    _project_path: &str,
    lock: &HarborLock,
    plugins: &[Plugin],
    bindings: &mut Vec<ProjectBinding>,
    project_id: &str,
    strict: bool,
) -> Result<Vec<String>> {
    let mut messages = Vec::new();

    for locked in &lock.plugins {
        let existing_binding = bindings.iter()
            .find(|b| b.project_id == project_id && b.plugin_id == locked.plugin_id);

        if let Some(plugin) = plugins.iter().find(|p| p.plugin_id == locked.plugin_id) {
            let version = plugin.versions.iter()
                .find(|v| v.version_id == locked.version_id);

            if let Some(ver) = version {
                if strict {
                    let actual_hash = compute_dir_hash(Path::new(&ver.path))
                        .unwrap_or_default();
                    if actual_hash != locked.content_hash {
                        messages.push(format!(
                            "插件 {} 内容哈希不匹配（严格模式），跳过同步",
                            locked.plugin_name
                        ));
                        continue;
                    }
                }

                if let Some(existing) = existing_binding {
                    if existing.version_id != locked.version_id
                        || existing.mount_path != locked.mount_path
                    {
                        if let Some(idx) = bindings.iter().position(|b| {
                            b.project_id == project_id && b.plugin_id == locked.plugin_id
                        }) {
                            bindings[idx] = ProjectBinding::new(
                                project_id.to_string(),
                                locked.plugin_id.clone(),
                                locked.version_id.clone(),
                                locked.unit_id.clone(),
                                locked.mount_path.clone(),
                                locked.subdirectory.clone(),
                            );
                            messages.push(format!("已更新插件 {} 的绑定", locked.plugin_name));
                        }
                    }
                } else {
                    bindings.push(ProjectBinding::new(
                        project_id.to_string(),
                        locked.plugin_id.clone(),
                        locked.version_id.clone(),
                        locked.unit_id.clone(),
                        locked.mount_path.clone(),
                        locked.subdirectory.clone(),
                    ));
                    messages.push(format!("已添加插件 {} 的绑定", locked.plugin_name));
                }
            } else {
                messages.push(format!(
                    "插件 {} 的版本 {} 未找到，跳过同步",
                    locked.plugin_name, locked.version
                ));
            }
        } else {
            messages.push(format!(
                "插件 {} 未安装，无法从锁文件同步",
                locked.plugin_name
            ));
        }
    }

    Ok(messages)
}
