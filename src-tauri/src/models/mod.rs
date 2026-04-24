use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SourceType {
    Git,
    Local,
    AssetLibrary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSource {
    pub source_type: SourceType,
    pub url: String,
    pub imported_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginUnit {
    pub unit_id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub version: String,
    pub subdirectory: String,
    pub plugin_cfg_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginVersion {
    pub version_id: String,
    pub version: String,
    pub path: String,
    pub created_at: DateTime<Utc>,
    pub units: Vec<PluginUnit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Compatibility {
    Godot3,
    Godot4,
    Both,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plugin {
    pub plugin_id: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub source: PluginSource,
    pub versions: Vec<PluginVersion>,
    pub compatibility: Compatibility,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Plugin {
    pub fn new(name: String, source: PluginSource) -> Self {
        let now = Utc::now();
        Self {
            plugin_id: Uuid::new_v4().to_string(),
            name,
            description: String::new(),
            author: String::new(),
            source,
            versions: Vec::new(),
            compatibility: Compatibility::Unknown,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProjectStatus {
    Ready,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub project_id: String,
    pub name: String,
    pub path: String,
    pub godot_version: String,
    #[serde(default)]
    pub icon_path: String,
    pub status: ProjectStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Project {
    pub fn new(name: String, path: String, godot_version: String, icon_path: String) -> Self {
        let now = Utc::now();
        Self {
            project_id: Uuid::new_v4().to_string(),
            name,
            path,
            godot_version,
            icon_path,
            status: ProjectStatus::Ready,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectBinding {
    pub project_id: String,
    pub plugin_id: String,
    pub version_id: String,
    pub unit_id: String,
    pub mount_path: String,
    pub created_at: DateTime<Utc>,
}

impl ProjectBinding {
    pub fn new(
        project_id: String,
        plugin_id: String,
        version_id: String,
        unit_id: String,
        mount_path: String,
    ) -> Self {
        Self {
            project_id,
            plugin_id,
            version_id,
            unit_id,
            mount_path,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MountStrategy {
    Symlink,
    Junction,
    Copy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub scan_directories: Vec<String>,
    pub mount_strategy: MountStrategy,
    pub language: String,
    pub theme: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            scan_directories: Vec::new(),
            mount_strategy: MountStrategy::Symlink,
            language: "zh-CN".to_string(),
            theme: "light".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyResult {
    pub success: bool,
    pub created: Vec<String>,
    pub removed: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictInfo {
    pub conflict_type: String,
    pub path: String,
    pub message: String,
}
