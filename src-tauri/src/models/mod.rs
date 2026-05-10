use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::fs;
use crate::utils::should_skip_dir;

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
            if should_skip_dir(&file_name.to_string_lossy()) {
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
    Url,
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
    #[serde(default)]
    pub is_healthy: Option<bool>,
    #[serde(default)]
    pub subdirectory: String,
}

impl ProjectBinding {
    pub fn new(
        project_id: String,
        plugin_id: String,
        version_id: String,
        unit_id: String,
        mount_path: String,
        subdirectory: String,
    ) -> Self {
        Self {
            project_id,
            plugin_id,
            version_id,
            unit_id,
            mount_path,
            created_at: Utc::now(),
            is_healthy: None,
            subdirectory,
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
pub struct EngineMirrorConfig {
    pub id: String,
    pub name: String,
    pub base_url: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub is_official: bool,
    #[serde(default = "default_mirror_type")]
    pub mirror_type: String,
}

fn default_mirror_type() -> String {
    "github_api".to_string()
}

impl EngineMirrorConfig {
    pub fn official() -> Self {
        Self {
            id: "official".to_string(),
            name: "GitHub Official".to_string(),
            base_url: "https://api.github.com".to_string(),
            enabled: true,
            is_official: true,
            mirror_type: "github_api".to_string(),
        }
    }

    pub fn your_objectstorage() -> Self {
        Self {
            id: "your-objectstorage".to_string(),
            name: "Your ObjectStorage (CN) - Unavailable".to_string(),
            base_url: "https://godot-releases.nbg1.your-objectstorage.com".to_string(),
            enabled: false,
            is_official: false,
            mirror_type: "direct".to_string(),
        }
    }
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
    pub auto_check_plugin_updates: bool,
    #[serde(default = "default_true")]
    pub auto_check_app_updates: bool,
    #[serde(default = "default_true")]
    pub auto_check_engine_updates: bool,
    #[serde(default = "default_four")]
    pub update_check_interval_hours: u32,
    #[serde(default)]
    pub skipped_app_version: String,
    #[serde(default = "default_engine_mirrors")]
    pub engine_mirrors: Vec<EngineMirrorConfig>,
    #[serde(default)]
    pub custom_data_dir: String,
    #[serde(default)]
    pub selected_mirror_id: String,
    #[serde(default)]
    pub known_engine_paths: Vec<String>,
    #[serde(default)]
    pub auto_apply: bool,
    #[serde(default)]
    pub github_api_proxy: String,
    #[serde(default)]
    pub asset_library_mirror: String,
}

fn default_true() -> bool { true }
fn default_four() -> u32 { 4 }
fn default_engine_mirrors() -> Vec<EngineMirrorConfig> {
    vec![EngineMirrorConfig::official()]
}

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
            auto_check_plugin_updates: false,
            auto_check_app_updates: true,
            auto_check_engine_updates: true,
            update_check_interval_hours: 4,
            skipped_app_version: String::new(),
            engine_mirrors: default_engine_mirrors(),
            custom_data_dir: String::new(),
            selected_mirror_id: String::new(),
            known_engine_paths: Vec::new(),
            auto_apply: true,
            github_api_proxy: String::new(),
            asset_library_mirror: String::new(),
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
            created_at: now,
            updated_at: now,
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
    #[serde(default)]
    pub subdirectory: String,
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
    pub download_url: Option<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EngineReleaseChannel {
    Stable,
    Rc,
    Beta,
    Alpha,
    Dev,
}

impl std::fmt::Display for EngineReleaseChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineReleaseChannel::Stable => write!(f, "stable"),
            EngineReleaseChannel::Rc => write!(f, "rc"),
            EngineReleaseChannel::Beta => write!(f, "beta"),
            EngineReleaseChannel::Alpha => write!(f, "alpha"),
            EngineReleaseChannel::Dev => write!(f, "dev"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteEngineVersion {
    pub version: String,
    pub tag_name: String,
    pub channel: EngineReleaseChannel,
    #[serde(default)]
    pub channel_number: u32,
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub is_stable: bool,
    #[serde(default)]
    pub is_lts: bool,
    pub published_at: String,
    pub release_url: String,
    pub release_notes: String,
    pub download_url: String,
    pub file_name: String,
    pub file_size: u64,
    pub is_installed: bool,
    #[serde(default = "default_variant")]
    pub variant: String,
}

fn default_variant() -> String {
    "standard".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineDownloadProgress {
    pub version: String,
    pub variant: String,
    pub stage: String,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub progress: f64,
    pub message: String,
    pub speed: f64,
    pub eta: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadEngineResult {
    pub success: bool,
    pub cancelled: bool,
    pub engine: Option<Engine>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedRemoteVersions {
    #[serde(default)]
    pub cache_version: u32,
    pub cached_at: String,
    pub mirror_id: String,
    pub versions: Vec<RemoteEngineVersion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedPluginUpdates {
    #[serde(default)]
    pub cache_version: u32,
    pub cached_at: String,
    pub updates: Vec<PluginUpdateInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedAppUpdate {
    #[serde(default)]
    pub cache_version: u32,
    pub cached_at: String,
    pub update_info: Option<AppUpdateInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoragePaths {
    pub app_data_dir: String,
    pub plugins_dir: String,
    pub engines_dir: String,
    pub cache_dir: String,
    pub logs_dir: String,
    pub hot_updates_dir: String,
    pub settings_file: String,
    pub projects_file: String,
    pub engines_file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateBinding {
    pub plugin_id: String,
    pub plugin_name: String,
    pub version_id: String,
    pub unit_id: String,
    pub unit_name: String,
    pub mount_path: String,
    pub subdirectory: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectTemplate {
    pub template_id: String,
    pub name: String,
    pub bindings: Vec<TemplateBinding>,
    pub created_at: String,
}
