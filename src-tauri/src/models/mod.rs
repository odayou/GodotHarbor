use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
    #[serde(default)]
    pub is_favorite: bool,
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
            is_favorite: false,
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
    MissingSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub project_id: String,
    pub name: String,
    pub path: String,
    pub godot_version: String,
    #[serde(default)]
    pub icon_path: String,
    #[serde(default)]
    pub group: String,
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
            group: String::new(),
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
    #[serde(default = "default_true")]
    pub auto_scan_on_startup: bool,
    #[serde(default)]
    pub sidebar_collapsed: bool,
}

fn default_true() -> bool { true }

impl Default for Settings {
    fn default() -> Self {
        Self {
            scan_directories: Vec::new(),
            mount_strategy: MountStrategy::Symlink,
            language: "zh-CN".to_string(),
            theme: "light".to_string(),
            auto_scan_on_startup: true,
            sidebar_collapsed: false,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EngineType {
    Godot3,
    Godot4,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Engine {
    pub engine_id: String,
    pub name: String,
    pub path: String,
    pub engine_type: EngineType,
    pub version: String,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Engine {
    pub fn new(name: String, path: String, engine_type: EngineType, version: String) -> Self {
        let now = Utc::now();
        Self {
            engine_id: Uuid::new_v4().to_string(),
            name,
            path,
            engine_type,
            version,
            is_default: false,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectEngineBinding {
    pub project_id: String,
    pub engine_id: String,
    pub custom_args: String,
    pub created_at: DateTime<Utc>,
}

impl ProjectEngineBinding {
    pub fn new(project_id: String, engine_id: String, custom_args: String) -> Self {
        Self {
            project_id,
            engine_id,
            custom_args,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependency {
    pub plugin_id: String,
    pub version_constraint: String,
    pub is_optional: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginUpdateInfo {
    pub plugin_id: String,
    pub current_version: String,
    pub latest_version: String,
    pub update_available: bool,
    pub release_notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamSharedConfig {
    pub config_id: String,
    pub name: String,
    pub description: String,
    pub bindings: Vec<ProjectBinding>,
    pub engine_bindings: Vec<ProjectEngineBinding>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TeamSharedConfig {
    pub fn new(name: String, description: String) -> Self {
        let now = Utc::now();
        Self {
            config_id: Uuid::new_v4().to_string(),
            name,
            description,
            bindings: Vec::new(),
            engine_bindings: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchResult {
    pub success: bool,
    pub pid: Option<u32>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStats {
    pub project_count: usize,
    pub plugin_count: usize,
    pub binding_count: usize,
    pub engine_count: usize,
    pub recent_projects: Vec<Project>,
}
