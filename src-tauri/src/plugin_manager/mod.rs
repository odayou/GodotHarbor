use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use uuid::Uuid;
use walkdir::WalkDir;
use crate::utils::{copy_dir_all, should_skip_dir};
use rayon::prelude::*;
use tauri::{AppHandle, Emitter};
use crate::models::{Plugin, PluginSource, PluginVersion, PluginUnit, SourceType, Compatibility, Project, compute_dir_hash, ScannedPlugin};

const PLUGIN_SCAN_MAX_DEPTH: usize = 5;
const COMPAT_SCAN_MAX_DEPTH: usize = 5;

pub struct PluginManager {
    plugins_dir: PathBuf,
}

impl PluginManager {
    pub fn new(plugins_dir: PathBuf) -> Self {
        fs::create_dir_all(&plugins_dir).ok();
        Self { plugins_dir }
    }

    pub fn scan_project_plugins(&self, projects: &[Project]) -> Result<Vec<ScannedPlugin>> {
        let results: Vec<ScannedPlugin> = projects
            .par_iter()
            .filter_map(|project| {
                let project_path = Path::new(&project.path);
                let addons_dir = project_path.join("addons");

                if !addons_dir.exists() || !addons_dir.is_dir() {
                    return None;
                }

                let plugin_entries: Vec<ScannedPlugin> = WalkDir::new(&addons_dir)
                    .max_depth(1)
                    .follow_links(false)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| {
                        let path = e.path();
                        path.is_dir() && path != addons_dir && path.join("plugin.cfg").exists()
                    })
                    .filter_map(|e| {
                        let path = e.into_path();
                        let cfg_path = path.join("plugin.cfg");
                        let plugin_name = fs::read_to_string(&cfg_path)
                            .ok()
                            .and_then(|content| {
                                content.lines()
                                    .find(|l| l.trim().starts_with("name="))
                                    .map(|l| l[5..].trim_matches('"').trim().to_string())
                                    .filter(|s| !s.is_empty())
                            })
                            .unwrap_or_else(|| path.file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_default());
                        Some(ScannedPlugin {
                            path: path.to_string_lossy().to_string(),
                            plugin_name,
                            project_name: project.name.clone(),
                        })
                    })
                    .collect();

                if plugin_entries.is_empty() {
                    None
                } else {
                    Some(plugin_entries)
                }
            })
            .flatten()
            .collect();

        let mut seen_names = std::collections::HashSet::new();
        let deduped: Vec<ScannedPlugin> = results
            .into_iter()
            .filter(|sp| seen_names.insert(sp.plugin_name.to_lowercase()))
            .collect();

        Ok(deduped)
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

        let mut plugin = Plugin::new(plugin_name.clone(), plugin_source);

        let version_id = Uuid::new_v4().to_string();
        let version_dir = self.plugins_dir.join(&plugin.plugin_id).join(&version_id);
        let payload_dir = version_dir.join("payload");

        fs::create_dir_all(&payload_dir)
            .context("Failed to create version directory")?;

        if let Err(e) = copy_dir_all(source, &payload_dir).map_err(|e| anyhow::anyhow!(e)) {
            let _ = fs::remove_dir_all(&version_dir);
            return Err(e.context("Failed to copy plugin files, cleaned up partial import"));
        }

        self.finalize_import(&mut plugin, &payload_dir, &version_id, &plugin_name)?;

        Ok(plugin)
    }

    fn finalize_import(&self, plugin: &mut Plugin, payload_dir: &Path, version_id: &str, plugin_name: &str) -> Result<()> {
        let units = match self.parse_plugin_units(payload_dir) {
            Ok(u) => u,
            Err(e) => {
                let version_dir = payload_dir.parent().unwrap_or(payload_dir);
                let _ = fs::remove_dir_all(version_dir);
                return Err(e.context("Failed to parse plugin units, cleaned up partial import"));
            }
        };

        let compatibility = self.detect_compatibility(payload_dir);
        self.write_harbor_marker(payload_dir);
        let content_hash = compute_dir_hash(payload_dir).unwrap_or_default();

        let (unit_version, unit_name, unit_description, unit_author) =
            if let Some(first_unit) = units.first() {
                (
                    if first_unit.version.is_empty() { "1.0.0".to_string() } else { first_unit.version.clone() },
                    if first_unit.name.is_empty() { plugin_name.to_string() } else { first_unit.name.clone() },
                    first_unit.description.clone(),
                    first_unit.author.clone(),
                )
            } else {
                ("1.0.0".to_string(), plugin_name.to_string(), String::new(), String::new())
            };

        let plugin_version = PluginVersion {
            version_id: version_id.to_string(),
            version: unit_version,
            path: payload_dir.to_string_lossy().to_string(),
            created_at: chrono::Utc::now(),
            units,
        };

        plugin.versions.push(plugin_version);
        plugin.compatibility = compatibility;
        plugin.name = unit_name;
        plugin.description = unit_description;
        plugin.author = unit_author;
        plugin.content_hash = content_hash;

        Ok(())
    }

    pub fn import_from_git(&self, git_url: &str, app_handle: &AppHandle) -> Result<Plugin> {
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

        let mut plugin = Plugin::new(plugin_name.clone(), plugin_source);

        let version_id = Uuid::new_v4().to_string();
        let version_dir = self.plugins_dir.join(&plugin.plugin_id).join(&version_id);
        let payload_dir = version_dir.join("payload");
        let git_store_dir = version_dir.join("git");

        fs::create_dir_all(&payload_dir)
            .context("Failed to create version directory")?;

        let mut callbacks = git2::RemoteCallbacks::new();
        let app_handle_clone = app_handle.clone();
        callbacks.transfer_progress(move |progress| {
            let received = progress.received_objects();
            let total = progress.total_objects();
            let percentage = if total > 0 {
                (received as f64 / total as f64 * 100.0) as u32
            } else {
                0
            };
            let _ = app_handle_clone.emit("git-clone-progress", serde_json::json!({
                "received_objects": received,
                "total_objects": total,
                "percentage": percentage,
            }));
            true
        });

        let mut fetch_options = git2::FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        let mut builder = git2::build::RepoBuilder::new();
        builder.fetch_options(fetch_options);

        if let Err(e) = builder.clone(git_url, &payload_dir) {
            let _ = fs::remove_dir_all(&version_dir);
            return Err(anyhow::anyhow!("Failed to clone git repository, cleaned up partial clone: {}", e));
        }

        let git_dir = payload_dir.join(".git");
        if git_dir.exists() {
            if !git_store_dir.exists() {
                fs::create_dir_all(&git_store_dir).ok();
            }
            if let Err(e) = copy_dir_all(&git_dir, &git_store_dir) {
                eprintln!("Warning: failed to backup .git directory: {}", e);
            }
            fs::remove_dir_all(&git_dir).ok();
        }

        self.finalize_import(&mut plugin, &payload_dir, &version_id, &plugin_name)?;

        Ok(plugin)
    }

    pub fn parse_plugin_units(&self, plugin_dir: &Path) -> Result<Vec<PluginUnit>> {
        let mut units = Vec::new();

        for entry in WalkDir::new(plugin_dir)
            .follow_links(false)
            .max_depth(PLUGIN_SCAN_MAX_DEPTH)
            .into_iter()
            .filter_entry(|e| {
                if e.file_type().is_dir() {
                    return !should_skip_dir(&e.file_name().to_string_lossy());
                }
                true
            })
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
        let mut version = String::new();

        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("name=") {
                name = line[5..].trim_matches('"').to_string();
            } else if line.starts_with("description=") {
                description = line[12..].trim_matches('"').to_string();
            } else if line.starts_with("author=") {
                author = line[7..].trim_matches('"').to_string();
            } else if line.starts_with("version=") {
                version = line[8..].trim_matches('"').to_string();
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
            description,
            author,
            version,
            subdirectory,
            plugin_cfg_path: cfg_path.to_string_lossy().to_string(),
        })
    }

    pub fn detect_compatibility(&self, plugin_dir: &Path) -> Compatibility {
        let files: Vec<PathBuf> = WalkDir::new(plugin_dir)
            .follow_links(false)
            .max_depth(COMPAT_SCAN_MAX_DEPTH)
            .into_iter()
            .filter_entry(|e| {
                if e.file_type().is_dir() {
                    return !should_skip_dir(&e.file_name().to_string_lossy());
                }
                true
            })
            .filter_map(|e| e.ok())
            .filter(|e| {
                if let Some(ext) = e.path().extension() {
                    ext == "gd" || ext == "tscn" || ext == "tres"
                } else {
                    false
                }
            })
            .map(|e| e.into_path())
            .collect();

        let (godot4_signals, godot3_signals) = files
            .par_iter()
            .map(|path| {
                let mut g4 = 0i32;
                let mut g3 = 0i32;

                if let Ok(content) = fs::read_to_string(path) {
                    if let Some(ext) = path.extension() {
                        if ext == "gd" {
                            if content.contains("@export") { g4 += 2; }
                            if content.contains("class_name") { g4 += 1; }
                            if content.contains("await ") { g4 += 1; }
                            if content.contains("var ") && content.contains(": ") { g4 += 1; }
                            if content.contains("enum ") && content.contains(":") { g4 += 1; }

                            if content.contains("export(") { g3 += 2; }
                            if content.contains("export(PackedScene)") { g3 += 1; }
                            if content.contains(".set_deferred(") { g3 += 1; }
                        }
                        if ext == "tscn" || ext == "tres" {
                            if content.contains("[node]") && content.contains("script/signal") { g3 += 1; }
                            if content.contains("metadata/_edit_lock_") { g4 += 1; }
                            if content.contains("uid://") { g4 += 2; }
                        }
                    }
                }

                (g4, g3)
            })
            .reduce(
                || (0i32, 0i32),
                |(g4a, g3a), (g4b, g3b)| (g4a + g4b, g3a + g3b),
            );

        if godot4_signals > 0 && godot3_signals == 0 {
            Compatibility::Godot4
        } else if godot3_signals > 0 && godot4_signals == 0 {
            Compatibility::Godot3
        } else if godot4_signals > 0 && godot3_signals > 0 {
            Compatibility::Both
        } else {
            Compatibility::Unknown
        }
    }

    fn write_harbor_marker(&self, payload_dir: &Path) {
        let marker_path = payload_dir.join(".harbor-managed");
        let marker_content = serde_json::json!({
            "managed_by": "godot-harbor",
            "version": "1.0",
            "created_at": chrono::Utc::now().to_rfc3339()
        });
        if let Ok(content) = serde_json::to_string_pretty(&marker_content) {
            let _ = fs::write(&marker_path, content);
        }
    }
}
