use crate::models::{Plugin, SourceType, Project, AssetType};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HarborConfig {
    pub version: u32,
    #[serde(default)]
    pub bindings: Vec<HarborBinding>,
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
            version: 1,
            bindings: Vec::new(),
        }
    }
}

impl HarborConfig {
    pub fn to_yaml(&self) -> Result<String> {
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
        let config: HarborConfig = serde_yaml::from_str(content)
            .context("Failed to parse .harbor.yml")?;
        if config.version != 1 {
            anyhow::bail!("Unsupported .harbor.yml version: {}", config.version);
        }
        Ok(config)
    }
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

    let mut bindings = Vec::new();
    let mut skipped_local = Vec::new();
    for pb in &project_bindings {
        if let Some(plugin) = plugins.iter().find(|p| p.plugin_id == pb.plugin_id) {
            if plugin.source.source_type == SourceType::Local {
                skipped_local.push(plugin.name.clone());
                continue;
            }
            let source_str = match plugin.source.source_type {
                SourceType::Git => plugin.source.url.clone(),
                SourceType::AssetLibrary => {
                    let id = plugin.source.url.trim_start_matches("asset-library://");
                    format!("asset-library:{}", id)
                }
                SourceType::Url => plugin.source.url.clone(),
                _ => plugin.source.url.clone(),
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

            bindings.push(HarborBinding {
                name: plugin.name.clone(),
                source: source_str,
                r#ref: git_ref,
                mount_path: pb.mount_path.clone(),
                asset_type: plugin.asset_type.clone(),
            });
        }
    }

    (HarborConfig {
        version: 1,
        bindings,
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
