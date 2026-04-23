use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use uuid::Uuid;
use walkdir::WalkDir;
use crate::models::{Plugin, PluginSource, PluginVersion, PluginUnit, SourceType, Compatibility, Project};

pub struct PluginManager {
    plugins_dir: PathBuf,
}

impl PluginManager {
    pub fn new(plugins_dir: PathBuf) -> Self {
        fs::create_dir_all(&plugins_dir).ok();
        Self { plugins_dir }
    }
    
    /// 扫描所有项目中的插件目录，返回发现的插件路径列表
    pub fn scan_project_plugins(&self, projects: &[Project]) -> Result<Vec<PathBuf>> {
        let mut plugin_paths = Vec::new();
        
        for project in projects {
            let project_path = Path::new(&project.path);
            let addons_dir = project_path.join("addons");
            
            if addons_dir.exists() && addons_dir.is_dir() {
                for entry in WalkDir::new(&addons_dir)
                    .max_depth(1)
                    .into_iter()
                    .filter_map(|e| e.ok())
                {
                    let path = entry.path();
                    if path.is_dir() && path != addons_dir {
                        if path.join("plugin.cfg").exists() {
                            plugin_paths.push(path.to_path_buf());
                        }
                    }
                }
            }
        }
        
        Ok(plugin_paths)
    }

    pub fn import_from_local(&self, source_path: &str) -> Result<Plugin> {
        let source = Path::new(source_path);
        
        if !source.exists() {
            anyhow::bail!("Source path does not exist: {}", source_path);
        }

        let plugin_name = source.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let plugin_source = PluginSource {
            source_type: SourceType::Local,
            url: source_path.to_string(),
            imported_at: chrono::Utc::now(),
        };

        let mut plugin = Plugin::new(plugin_name, plugin_source);
        
        let version_id = Uuid::new_v4().to_string();
        let version_dir = self.plugins_dir.join(&plugin.plugin_id).join(&version_id);
        let payload_dir = version_dir.join("payload");
        
        fs::create_dir_all(&payload_dir)
            .context("Failed to create version directory")?;
        
        self.copy_dir_recursive(source, &payload_dir)
            .context("Failed to copy plugin files")?;
        
        let units = self.parse_plugin_units(&payload_dir)
            .context("Failed to parse plugin units")?;
        
        let compatibility = self.detect_compatibility(&payload_dir);
        
        let plugin_version = PluginVersion {
            version_id: version_id.clone(),
            version: "1.0.0".to_string(),
            path: payload_dir.to_string_lossy().to_string(),
            created_at: chrono::Utc::now(),
            units,
        };
        
        plugin.versions.push(plugin_version);
        plugin.compatibility = compatibility;
        
        if let Some(first_unit) = plugin.versions[0].units.first() {
            plugin.name = first_unit.name.clone();
        }
        
        Ok(plugin)
    }

    pub fn import_from_git(&self, git_url: &str) -> Result<Plugin> {
        let plugin_name = git_url
            .split('/')
            .last()
            .unwrap_or("unknown")
            .trim_end_matches(".git")
            .to_string();

        let plugin_source = PluginSource {
            source_type: SourceType::Git,
            url: git_url.to_string(),
            imported_at: chrono::Utc::now(),
        };

        let mut plugin = Plugin::new(plugin_name, plugin_source);
        
        let version_id = Uuid::new_v4().to_string();
        let version_dir = self.plugins_dir.join(&plugin.plugin_id).join(&version_id);
        let payload_dir = version_dir.join("payload");
        
        fs::create_dir_all(&payload_dir)
            .context("Failed to create version directory")?;
        
        git2::Repository::clone(git_url, &payload_dir)
            .context("Failed to clone git repository")?;
        
        let git_dir = payload_dir.join(".git");
        if git_dir.exists() {
            fs::remove_dir_all(&git_dir).ok();
        }
        
        let units = self.parse_plugin_units(&payload_dir)
            .context("Failed to parse plugin units")?;
        
        let compatibility = self.detect_compatibility(&payload_dir);
        
        let plugin_version = PluginVersion {
            version_id: version_id.clone(),
            version: "1.0.0".to_string(),
            path: payload_dir.to_string_lossy().to_string(),
            created_at: chrono::Utc::now(),
            units,
        };
        
        plugin.versions.push(plugin_version);
        plugin.compatibility = compatibility;
        
        if let Some(first_unit) = plugin.versions[0].units.first() {
            plugin.name = first_unit.name.clone();
        }
        
        Ok(plugin)
    }

    fn parse_plugin_units(&self, plugin_dir: &Path) -> Result<Vec<PluginUnit>> {
        let mut units = Vec::new();
        
        for entry in walkdir::WalkDir::new(plugin_dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            
            if path.file_name().map(|f| f == "plugin.cfg").unwrap_or(false) {
                if let Ok(unit) = self.parse_plugin_cfg(path, plugin_dir) {
                    units.push(unit);
                }
            }
        }
        
        if units.is_empty() {
            anyhow::bail!("No valid plugin.cfg found in plugin directory");
        }
        
        Ok(units)
    }

    fn parse_plugin_cfg(&self, cfg_path: &Path, plugin_dir: &Path) -> Result<PluginUnit> {
        let content = fs::read_to_string(cfg_path)
            .context("Failed to read plugin.cfg")?;
        
        let mut name = String::new();
        let mut description = String::new();
        let mut author = String::new();
        
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("name=") {
                name = line[5..].trim_matches('"').to_string();
            } else if line.starts_with("description=") {
                description = line[12..].trim_matches('"').to_string();
            } else if line.starts_with("author=") {
                author = line[7..].trim_matches('"').to_string();
            }
        }
        
        let subdirectory = cfg_path.parent()
            .and_then(|p| p.strip_prefix(plugin_dir).ok())
            .and_then(|p| p.to_str())
            .unwrap_or("")
            .to_string();
        
        Ok(PluginUnit {
            unit_id: Uuid::new_v4().to_string(),
            name,
            subdirectory,
            plugin_cfg_path: cfg_path.to_string_lossy().to_string(),
        })
    }

    fn detect_compatibility(&self, plugin_dir: &Path) -> Compatibility {
        for entry in walkdir::WalkDir::new(plugin_dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            
            if let Some(ext) = path.extension() {
                if ext == "gd" {
                    if let Ok(content) = fs::read_to_string(path) {
                        if content.contains("@export") || content.contains("class_name") {
                            return Compatibility::Godot4;
                        }
                        if content.contains("export(PackedScene)") || content.contains("tool") {
                            return Compatibility::Godot3;
                        }
                    }
                }
            }
        }
        
        Compatibility::Unknown
    }

    fn copy_dir_recursive(&self, src: &Path, dst: &Path) -> Result<()> {
        if !dst.exists() {
            fs::create_dir_all(dst)?;
        }
        
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());
            
            if src_path.is_dir() {
                self.copy_dir_recursive(&src_path, &dst_path)?;
            } else {
                fs::copy(&src_path, &dst_path)?;
            }
        }
        
        Ok(())
    }
}
