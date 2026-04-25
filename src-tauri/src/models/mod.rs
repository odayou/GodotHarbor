use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::fs;

pub fn compute_dir_hash(dir: &Path) -> Result<String, String> {
    let mut hasher = DefaultHasher::new();
    compute_dir_hash_recursive(dir, &mut hasher)?;
    Ok(format!("{:016x}", hasher.finish()))
}

fn compute_dir_hash_recursive(dir: &Path, hasher: &mut DefaultHasher) -> Result<(), String> {
    if !dir.exists() {
        return Ok(());
    }
    let mut entries: Vec<_> = fs::read_dir(dir)
        .map_err(|e| format!("读取目录失败: {}", e))?
        .filter_map(|e| e.ok())
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        let file_name = entry.file_name();
        file_name.to_string_lossy().hash(hasher);

        if path.is_dir() {
            let name_lower = file_name.to_string_lossy().to_lowercase();
            if [".git", ".svn", ".hg", "node_modules", "__pycache__", ".godot", ".import", "build", "dist", ".cache"]
                .iter().any(|s| name_lower == *s) {
                continue;
            }
            compute_dir_hash_recursive(&path, hasher)?;
        } else {
            let file_name_str = file_name.to_string_lossy();
            if file_name_str == ".harbor-managed" {
                continue;
            }
            if let Ok(content) = fs::read(&path) {
                content.len().hash(hasher);
                if content.len() <= 65536 {
                    content.hash(hasher);
                } else {
                    content[..65536].hash(hasher);
                    let mid = content.len() / 2;
                    content[mid..mid.min(mid + 65536)].hash(hasher);
                    let start = content.len().saturating_sub(65536);
                    content[start..].hash(hasher);
                }
            }
        }
    }
    Ok(())
}

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
pub struct ScannedPlugin {
    pub path: String,
    pub plugin_name: String,
    pub project_name: String,
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
    #[serde(default)]
    pub content_hash: String,
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
            content_hash: String::new(),
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
    #[serde(default)]
    pub last_synced_at: Option<DateTime<Utc>>,
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
            last_synced_at: None,
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
    #[serde(default = "default_true")]
    pub auto_discover_engines: bool,
    #[serde(default)]
    pub onboarding_completed: bool,
    #[serde(default)]
    pub plugin_storage_path: String,
    #[serde(default)]
    pub auto_check_plugin_updates: bool,
    #[serde(default = "default_true")]
    pub auto_check_app_updates: bool,
    #[serde(default = "default_true")]
    pub auto_check_engine_updates: bool,
    #[serde(default = "default_four")]
    pub update_check_interval_hours: u32,
    #[serde(default)]
    pub skipped_app_version: String,
}

fn default_true() -> bool { true }
fn default_four() -> u32 { 4 }

impl Default for Settings {
    fn default() -> Self {
        Self {
            scan_directories: Vec::new(),
            mount_strategy: MountStrategy::Symlink,
            language: "zh-CN".to_string(),
            theme: "light".to_string(),
            auto_scan_on_startup: true,
            sidebar_collapsed: false,
            auto_discover_engines: true,
            onboarding_completed: false,
            plugin_storage_path: String::new(),
            auto_check_plugin_updates: false,
            auto_check_app_updates: true,
            auto_check_engine_updates: true,
            update_check_interval_hours: 4,
            skipped_app_version: String::new(),
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

impl std::fmt::Display for EngineType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineType::Godot3 => write!(f, "Godot3"),
            EngineType::Godot4 => write!(f, "Godot4"),
            EngineType::Unknown => write!(f, "Unknown"),
        }
    }
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
    pub plugin_name: String,
    pub current_version: String,
    pub latest_version: String,
    pub update_available: bool,
    pub release_notes: String,
    pub source_url: String,
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
pub struct BatchResult {
    pub success_count: usize,
    pub failed_count: usize,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchBindingRequest {
    pub project_id: String,
    pub plugin_id: String,
    pub version_id: String,
    pub unit_id: String,
    pub mount_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchApplyResult {
    pub results: Vec<ProjectApplyResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectApplyResult {
    pub project_id: String,
    pub project_name: String,
    pub success: bool,
    pub created: Vec<String>,
    pub removed: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStats {
    pub project_count: usize,
    pub plugin_count: usize,
    pub binding_count: usize,
    pub engine_count: usize,
    pub recent_projects: Vec<Project>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppUpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub release_notes: String,
    pub pub_date: String,
    pub download_size: Option<u64>,
    pub is_hot_update: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotUpdateInfo {
    pub version: String,
    pub min_compatible_app_version: String,
    pub max_compatible_app_version: String,
    pub release_notes: String,
    pub pub_date: String,
    pub download_size: u64,
    pub checksum: String,
    pub download_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCheckResult {
    pub app_update: Option<AppUpdateInfo>,
    pub hot_update: Option<HotUpdateInfo>,
    pub plugin_updates: Vec<PluginUpdateInfo>,
    pub engine_updates: Vec<crate::version_checker::VersionUpdateInfo>,
    pub checked_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProgress {
    pub update_type: String,
    pub target_id: String,
    pub stage: String,
    pub progress: u32,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateHistoryEntry {
    pub id: String,
    pub update_type: String,
    pub target_name: String,
    pub from_version: String,
    pub to_version: String,
    pub status: String,
    pub applied_at: String,
    pub notes: String,
}
