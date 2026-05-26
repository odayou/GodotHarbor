use crate::models::{Plugin, SourceType, Project, AssetType};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HarborConfig {
    pub version: u32,
    #[serde(default)]
    pub bindings: Vec<HarborBinding>,
    #[serde(default)]
    pub godot: Option<HarborGodot>,
    #[serde(default)]
    pub plugins: Vec<HarborPlugin>,
    #[serde(default)]
    pub export_presets: Vec<HarborExportPreset>,
    #[serde(default)]
    pub ci: Option<HarborCI>,
    #[serde(default)]
    pub settings: HarborSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HarborGodot {
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub mono: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HarborPlugin {
    pub name: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub r#ref: String,
    #[serde(default = "default_mount")]
    pub mount: String,
    #[serde(default)]
    pub asset_type: AssetType,
}

fn default_mount() -> String {
    "copy".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HarborExportPreset {
    pub platform: String,
    pub name: String,
    #[serde(default)]
    pub config: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HarborCI {
    #[serde(default)]
    pub provider: String,
    #[serde(default)]
    pub platforms: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HarborSettings {
    #[serde(default = "default_mount_strategy")]
    pub mount_strategy: String,
    #[serde(default = "default_true")]
    pub auto_sync: bool,
    #[serde(default)]
    pub drift_check_on_startup: bool,
}

fn default_mount_strategy() -> String {
    "copy".to_string()
}
fn default_true() -> bool { true }

impl Default for HarborSettings {
    fn default() -> Self {
        Self {
            mount_strategy: "copy".to_string(),
            auto_sync: true,
            drift_check_on_startup: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HarborBinding {
    pub name: String,
    pub source: String,
    #[serde(default)]
    pub r#ref: String,
    pub mount_path: String,
    #[serde(default)]
    pub asset_type: AssetType,
}

impl Default for HarborConfig {
    fn default() -> Self {
        Self {
            version: 2,
            bindings: Vec::new(),
            godot: None,
            plugins: Vec::new(),
            export_presets: Vec::new(),
            ci: None,
            settings: HarborSettings::default(),
        }
    }
}

impl HarborConfig {
    pub fn to_yaml(&self) -> Result<String> {
        if self.version >= 2 {
            self.to_yaml_v2()
        } else {
            self.to_yaml_v1()
        }
    }

    fn to_yaml_v2(&self) -> Result<String> {
        let mut lines = Vec::new();
        lines.push("# Harbor Configuration".to_string());
        lines.push("# Commit this file to version control for team collaboration".to_string());
        lines.push(format!("version: {}", self.version));
        lines.push(String::new());

        if let Some(godot) = &self.godot {
            lines.push("godot:".to_string());
            lines.push(format!("  version: \"{}\"", godot.version));
            lines.push(format!("  mono: {}", godot.mono));
            lines.push(String::new());
        }

        if self.plugins.is_empty() {
            lines.push("plugins: []".to_string());
        } else {
            lines.push("plugins:".to_string());
            for p in &self.plugins {
                lines.push(format!("  - name: {}", p.name));
                if !p.version.is_empty() {
                    lines.push(format!("    version: \"{}\"", p.version));
                }
                if !p.source.is_empty() {
                    lines.push(format!("    source: {}", p.source));
                }
                if !p.url.is_empty() {
                    lines.push(format!("    url: \"{}\"", p.url));
                }
                if !p.r#ref.is_empty() {
                    lines.push(format!("    ref: \"{}\"", p.r#ref));
                }
                lines.push(format!("    mount: {}", p.mount));
                if p.asset_type != AssetType::Plugin {
                    lines.push(format!("    asset_type: {:?}", p.asset_type));
                }
            }
            lines.push(String::new());
        }

        if !self.export_presets.is_empty() {
            lines.push("export_presets:".to_string());
            for ep in &self.export_presets {
                lines.push(format!("  - platform: \"{}\"", ep.platform));
                lines.push(format!("    name: \"{}\"", ep.name));
                if !ep.config.is_null() && ep.config.as_object().map_or(true, |o| !o.is_empty()) {
                    lines.push(format!("    config: {}", serde_json::to_string(&ep.config).unwrap_or_default()));
                }
            }
            lines.push(String::new());
        }

        if let Some(ci) = &self.ci {
            lines.push("ci:".to_string());
            lines.push(format!("  provider: \"{}\"", ci.provider));
            if !ci.platforms.is_empty() {
                let platforms_str: Vec<String> = ci.platforms.iter().map(|p| format!("\"{}\"", p)).collect();
                lines.push(format!("  platforms: [{}]", platforms_str.join(", ")));
            }
            lines.push(String::new());
        }

        lines.push("settings:".to_string());
        lines.push(format!("  mount_strategy: {}", self.settings.mount_strategy));
        lines.push(format!("  auto_sync: {}", self.settings.auto_sync));
        lines.push(format!("  drift_check_on_startup: {}", self.settings.drift_check_on_startup));

        Ok(lines.join("\n"))
    }

    fn to_yaml_v1(&self) -> Result<String> {
        let mut lines = Vec::new();
        lines.push("# Harbor Plugin Manager Configuration".to_string());
        lines.push("# Commit this file to version control for team collaboration".to_string());
        lines.push(format!("version: {}", self.version));
        lines.push(String::new());

        if self.bindings.is_empty() {
            lines.push("bindings: []".to_string());
        } else {
            lines.push("bindings:".to_string());
            for b in &self.bindings {
                lines.push(format!("  - name: {}", b.name));
                lines.push(format!("    source: {}", b.source));
                if !b.r#ref.is_empty() {
                    lines.push(format!("    ref: {}", b.r#ref));
                }
                lines.push(format!("    mount_path: {}", b.mount_path));
                if b.asset_type != AssetType::Plugin {
                    lines.push(format!("    asset_type: {:?}", b.asset_type));
                }
            }
        }

        Ok(lines.join("\n"))
    }

    pub fn from_yaml(content: &str) -> Result<Self> {
        let raw: serde_yaml::Value = serde_yaml::from_str(content)
            .context("Failed to parse .harbor.yml")?;
        let version = raw.get("version")
            .and_then(|v| v.as_u64())
            .unwrap_or(1) as u32;

        match version {
            1 => Self::from_yaml_v1(content),
            2 => Self::from_yaml_v2(content),
            _ => anyhow::bail!("Unsupported .harbor.yml version: {}", version),
        }
    }

    fn from_yaml_v1(content: &str) -> Result<Self> {
        let config: HarborConfigV1 = serde_yaml::from_str(content)
            .context("Failed to parse .harbor.yml v1")?;
        Ok(HarborConfig {
            version: 1,
            bindings: config.bindings,
            godot: None,
            plugins: Vec::new(),
            export_presets: Vec::new(),
            ci: None,
            settings: HarborSettings::default(),
        })
    }

    fn from_yaml_v2(content: &str) -> Result<Self> {
        let config: HarborConfig = serde_yaml::from_str(content)
            .context("Failed to parse .harbor.yml v2")?;
        if config.version != 2 {
            anyhow::bail!("Expected .harbor.yml version 2, got {}", config.version);
        }
        Ok(config)
    }

    pub fn upgrade_to_v2(&self) -> HarborConfig {
        if self.version >= 2 {
            return self.clone();
        }
        let mut plugins = Vec::new();
        for b in &self.bindings {
            let (source, url, git_ref) = parse_source_v1(&b.source);
            plugins.push(HarborPlugin {
                name: b.name.clone(),
                version: String::new(),
                source,
                url,
                r#ref: git_ref,
                mount: self.settings.mount_strategy.clone(),
                asset_type: b.asset_type.clone(),
            });
        }
        HarborConfig {
            version: 2,
            bindings: Vec::new(),
            godot: None,
            plugins,
            export_presets: Vec::new(),
            ci: None,
            settings: self.settings.clone(),
        }
    }
}

fn parse_source_v1(source: &str) -> (String, String, String) {
    if source.starts_with("asset-library:") {
        ("asset-store".to_string(), source.to_string(), String::new())
    } else if source.contains("github.com") || source.contains("gitlab.com") || source.ends_with(".git") {
        ("git".to_string(), source.to_string(), String::new())
    } else if source.starts_with("http") {
        ("url".to_string(), source.to_string(), String::new())
    } else {
        ("local".to_string(), source.to_string(), String::new())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HarborConfigV1 {
    pub version: u32,
    #[serde(default)]
    pub bindings: Vec<HarborBinding>,
}

pub fn get_harbor_config_path(project_path: &str) -> std::path::PathBuf {
    Path::new(project_path).join(".harbor.yml")
}

pub fn read_harbor_config_from_project(project_path: &str) -> Result<Option<HarborConfig>> {
    let config_path = get_harbor_config_path(project_path);
    if !config_path.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(&config_path)
        .with_context(|| format!("Failed to read {}", config_path.to_string_lossy()))?;
    let config = HarborConfig::from_yaml(&content)?;
    Ok(Some(config))
}

pub fn write_harbor_config_to_project(project_path: &str, config: &HarborConfig) -> Result<()> {
    let config_path = get_harbor_config_path(project_path);
    let content = config.to_yaml()?;
    std::fs::write(&config_path, content)
        .with_context(|| format!("Failed to write {}", config_path.to_string_lossy()))?;
    Ok(())
}

pub fn generate_config_from_bindings(
    project: &Project,
    plugins: &[Plugin],
    all_bindings: &[crate::models::ProjectBinding],
) -> (HarborConfig, Vec<String>) {
    let project_bindings: Vec<&crate::models::ProjectBinding> = all_bindings
        .iter()
        .filter(|b| b.project_id == project.project_id)
        .collect();

    let mut harbor_plugins = Vec::new();
    let mut skipped_local = Vec::new();
    for pb in &project_bindings {
        if let Some(plugin) = plugins.iter().find(|p| p.plugin_id == pb.plugin_id) {
            if plugin.source.source_type == SourceType::Local {
                skipped_local.push(plugin.name.clone());
                continue;
            }
            let (source, url) = match plugin.source.source_type {
                SourceType::Git => ("git".to_string(), plugin.source.url.clone()),
                SourceType::AssetLibrary => {
                    let id = plugin.source.url.trim_start_matches("asset-library://");
                    ("asset-store".to_string(), format!("asset-library:{}", id))
                }
                SourceType::Url => ("url".to_string(), plugin.source.url.clone()),
                _ => ("local".to_string(), plugin.source.url.clone()),
            };
            let git_ref = if !plugin.source.git_ref.is_empty() {
                plugin.source.git_ref.clone()
            } else {
                plugin.versions.iter()
                    .find(|v| v.version_id == pb.version_id)
                    .and_then(|v| {
                        let git_dir = std::path::PathBuf::from(&v.path).parent()
                            .unwrap_or(Path::new(""))
                            .join("git");
                        if git_dir.exists() {
                            Some(read_git_head_ref(&git_dir).unwrap_or_default())
                        } else {
                            Some(String::new())
                        }
                    })
                    .unwrap_or_default()
            };

            harbor_plugins.push(HarborPlugin {
                name: plugin.name.clone(),
                version: plugin.versions.iter()
                    .find(|v| v.version_id == pb.version_id)
                    .map(|v| v.version.clone())
                    .unwrap_or_default(),
                source,
                url,
                r#ref: git_ref,
                mount: "copy".to_string(),
                asset_type: plugin.asset_type.clone(),
            });
        }
    }

    (HarborConfig {
        version: 2,
        bindings: Vec::new(),
        godot: Some(HarborGodot {
            version: project.godot_version.clone(),
            mono: false,
        }),
        plugins: harbor_plugins,
        export_presets: Vec::new(),
        ci: None,
        settings: HarborSettings::default(),
    }, skipped_local)
}

fn read_git_head_ref(git_dir: &Path) -> Option<String> {
    let head_path = git_dir.join("HEAD");
    let head_content = std::fs::read_to_string(&head_path).ok()?;
    if head_content.starts_with("ref: ") {
        let ref_path = head_content.trim_start_matches("ref: ").trim();
        let resolved = std::fs::read_to_string(git_dir.join(ref_path)).ok()?;
        Some(resolved.trim().to_string())
    } else {
        Some(head_content.trim().to_string())
    }
}
