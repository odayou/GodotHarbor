use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use uuid::Uuid;
use walkdir::WalkDir;
use crate::utils::{copy_dir_all, should_skip_dir};
use rayon::prelude::*;
use tauri::{AppHandle, Emitter};
use crate::models::{Plugin, PluginSource, PluginVersion, PluginUnit, SourceType, Compatibility, Project, compute_dir_hash, ScannedPlugin, AssetType};

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

fn detect_plugin_source(plugin_dir: &Path) -> (crate::models::SourceType, String, String) {
    let git_dir = plugin_dir.join(".git");
    if !git_dir.exists() {
        return (crate::models::SourceType::Local, String::new(), String::new());
    }

    let repo = match git2::Repository::open(plugin_dir) {
        Ok(r) => r,
        Err(_) => return (crate::models::SourceType::Local, String::new(), String::new()),
    };

    let remote_url = repo.find_remote("origin")
        .ok()
        .and_then(|r| r.url().map(|u| u.to_string()))
        .unwrap_or_default();

    if remote_url.is_empty() {
        return (crate::models::SourceType::Local, String::new(), String::new());
    }

    let git_ref = repo.head()
        .ok()
        .and_then(|h| h.shorthand().map(|s| s.to_string()))
        .unwrap_or_default();

    (crate::models::SourceType::Git, remote_url, git_ref)
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

                let project_id = project.project_id.clone();
                let project_name = project.name.clone();
                let project_path_str = project.path.clone();

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

                        let (detected_source_type, detected_source_url, detected_git_ref) =
                            Self::detect_plugin_source(&path);

                        Some(ScannedPlugin {
                            path: path.to_string_lossy().to_string(),
                            plugin_name,
                            project_name: project_name.clone(),
                            project_id: project_id.clone(),
                            project_path: project_path_str.clone(),
                            detected_source_type,
                            detected_source_url,
                            detected_git_ref,
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

        Ok(results)
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
            git_ref: String::new(),
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
        let (mut units, asset_type) = self.analyze_asset_type(payload_dir, plugin_name);

        for unit in &mut units {
            if unit.dir_name.is_empty() || unit.dir_name == "payload" {
                unit.dir_name = plugin_name.to_string();
            }
        }

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
        plugin.asset_type = asset_type;

        Ok(())
    }

    pub fn import_from_url(&self, url: &str, app_handle: &AppHandle) -> Result<Plugin> {
        let url_path = url.split('?').next().unwrap_or(url);
        let file_name = url_path
            .split('/')
            .last()
            .unwrap_or("plugin")
            .to_string();

        let plugin_name = if file_name.contains('.') {
            file_name.rsplitn(2, '.').last().unwrap_or("plugin").to_string()
        } else {
            file_name.clone()
        };

        let plugin_source = PluginSource {
            source_type: SourceType::Url,
            url: url.to_string(),
            git_ref: String::new(),
            imported_at: chrono::Utc::now(),
        };

        let mut plugin = Plugin::new(plugin_name.clone(), plugin_source);

        let version_id = Uuid::new_v4().to_string();
        let version_dir = self.plugins_dir.join(&plugin.plugin_id).join(&version_id);
        let payload_dir = version_dir.join("payload");

        fs::create_dir_all(&version_dir)
            .context("Failed to create version directory")?;

        let download_dir = version_dir.join("download");
        let archive_path = download_dir.join(&file_name);

        let rt = tokio::runtime::Runtime::new()
            .context("Failed to create tokio runtime")?;
        rt.block_on(async {
            let client = crate::utils::create_http_client(None)
                .map_err(|e| anyhow::anyhow!(e))?;
            let resp = client.get(url).send().await
                .map_err(|e| anyhow::anyhow!("下载失败: {}", e))?;
            if !resp.status().is_success() {
                return Err(anyhow::anyhow!("下载失败，HTTP 状态码: {}", resp.status()));
            }
            fs::create_dir_all(&download_dir)
                .context("Failed to create download directory")?;
            let bytes = resp.bytes().await
                .map_err(|e| anyhow::anyhow!("读取响应内容失败: {}", e))?;
            fs::write(&archive_path, &bytes)
                .context("Failed to write downloaded file")?;
            Ok(())
        }).map_err(|e: anyhow::Error| {
            let _ = fs::remove_dir_all(&version_dir);
            e
        })?;

        let is_archive = file_name.ends_with(".zip")
            || file_name.ends_with(".tar.gz")
            || file_name.ends_with(".tgz")
            || file_name.ends_with(".tar.bz2")
            || file_name.ends_with(".gz");

        if is_archive {
            fs::create_dir_all(&payload_dir)
                .context("Failed to create payload directory")?;

            let extract_result = if file_name.ends_with(".zip") {
                Self::extract_zip(&archive_path, &payload_dir)
            } else {
                Self::extract_tar(&archive_path, &payload_dir)
            };

            let _ = fs::remove_dir_all(&download_dir);

            if let Err(e) = extract_result {
                let _ = fs::remove_dir_all(&version_dir);
                return Err(e.context("解压插件文件失败，已清理"));
            }

            let actual_payload = if let Some(single_dir) = Self::find_single_subdir(&payload_dir) {
                single_dir
            } else {
                payload_dir.clone()
            };

            self.finalize_import(&mut plugin, &actual_payload, &version_id, &plugin_name)?;
        } else {
            let _ = fs::remove_dir_all(&download_dir);
            let _ = fs::remove_dir_all(&version_dir);
            return Err(anyhow::anyhow!("不支持的文件格式，请提供 .zip 或 .tar.gz 压缩包"));
        }

        let _ = app_handle.emit("plugin-import-progress", serde_json::json!({
            "status": "complete",
            "plugin_name": plugin.name,
        }));

        Ok(plugin)
    }

    fn extract_zip(archive_path: &Path, target_dir: &Path) -> Result<()> {
        let file = fs::File::open(archive_path)
            .context("Failed to open zip archive")?;
        let mut archive = zip::ZipArchive::new(file)
            .context("Failed to read zip archive")?;
        archive.extract(target_dir)
            .map_err(|e| anyhow::anyhow!("解压 zip 失败: {}", e))?;
        Ok(())
    }

    fn extract_tar(archive_path: &Path, target_dir: &Path) -> Result<()> {
        let file = fs::File::open(archive_path)
            .context("Failed to open tar archive")?;
        let file_name = archive_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        if file_name.ends_with(".tar.gz") || file_name.ends_with(".tgz") {
            let gz = flate2::read::GzDecoder::new(file);
            let mut archive = tar::Archive::new(gz);
            archive.unpack(target_dir)
                .map_err(|e| anyhow::anyhow!("解压 tar.gz 失败: {}", e))?;
        } else if file_name.ends_with(".tar.bz2") {
            let bz = bzip2::read::BzDecoder::new(file);
            let mut archive = tar::Archive::new(bz);
            archive.unpack(target_dir)
                .map_err(|e| anyhow::anyhow!("解压 tar.bz2 失败: {}", e))?;
        } else if file_name.ends_with(".gz") {
            let gz = flate2::read::GzDecoder::new(file);
            let mut archive = tar::Archive::new(gz);
            archive.unpack(target_dir)
                .map_err(|e| anyhow::anyhow!("解压 gz 失败: {}", e))?;
        } else {
            let mut archive = tar::Archive::new(file);
            archive.unpack(target_dir)
                .map_err(|e| anyhow::anyhow!("解压 tar 失败: {}", e))?;
        }
        Ok(())
    }

    fn find_single_subdir(dir: &Path) -> Option<PathBuf> {
        if let Ok(entries) = fs::read_dir(dir) {
            let subdirs: Vec<PathBuf> = entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir())
                .map(|e| e.path())
                .collect();
            if subdirs.len() == 1 {
                return Some(subdirs.into_iter().next().unwrap());
            }
        }
        None
    }

    pub fn import_from_git(&self, git_url: &str, git_ref: Option<&str>, app_handle: &AppHandle) -> Result<Plugin> {
        let plugin_name = git_url
            .split('/')
            .last()
            .unwrap_or("unknown")
            .trim_end_matches(".git")
            .to_string();

        let resolved_ref = git_ref.map(|r| r.to_string());

        let plugin_source = PluginSource {
            source_type: SourceType::Git,
            url: git_url.to_string(),
            git_ref: resolved_ref.clone().unwrap_or_default(),
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

        if let Some(git_ref) = git_ref {
            builder.branch(git_ref);
        }

        if let Err(e) = builder.clone(git_url, &payload_dir) {
            let _ = fs::remove_dir_all(&version_dir);
            return Err(anyhow::anyhow!("Failed to clone git repository, cleaned up partial clone: {}", e));
        }

        let git_dir = payload_dir.join(".git");
        if git_dir.exists() {
            if !git_store_dir.exists() {
                fs::create_dir_all(&git_store_dir).ok();
            }

            let actual_ref = if resolved_ref.is_none() {
                git2::Repository::open(&payload_dir)
                    .ok()
                    .and_then(|repo| {
                        let head = repo.head().ok()?;
                        head.target().map(|oid| oid.to_string())
                    })
                    .unwrap_or_default()
            } else {
                String::new()
            };

            if let Err(e) = copy_dir_all(&git_dir, &git_store_dir) {
                eprintln!("Warning: failed to backup .git directory: {}", e);
            }
            fs::remove_dir_all(&git_dir).ok();

            if !actual_ref.is_empty() {
                plugin.source.git_ref = actual_ref;
            }
        }

        self.finalize_import(&mut plugin, &payload_dir, &version_id, &plugin_name)?;

        Ok(plugin)
    }

    pub fn parse_plugin_units(&self, plugin_dir: &Path) -> Vec<PluginUnit> {
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

        units
    }

    pub fn analyze_asset_type(&self, payload_dir: &Path, fallback_name: &str) -> (Vec<PluginUnit>, AssetType) {
        let has_project_godot = WalkDir::new(payload_dir)
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
            .any(|e| {
                e.file_name() == "project.godot"
            });

        if has_project_godot {
            return (Vec::new(), AssetType::Project);
        }

        let units = self.parse_plugin_units(payload_dir);
        if !units.is_empty() {
            (units, AssetType::Plugin)
        } else {
            let virtual_unit = PluginUnit {
                unit_id: Uuid::new_v4().to_string(),
                name: fallback_name.to_string(),
                dir_name: fallback_name.to_string(),
                description: String::new(),
                author: String::new(),
                version: String::new(),
                subdirectory: String::new(),
                plugin_cfg_path: String::new(),
                is_virtual: true,
            };
            (vec![virtual_unit], AssetType::AssetPack)
        }
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

        let dir_name = cfg_path.parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or(&name)
            .to_string();

        Ok(PluginUnit {
            unit_id: Uuid::new_v4().to_string(),
            name,
            dir_name,
            description,
            author,
            version,
            subdirectory,
            plugin_cfg_path: cfg_path.to_string_lossy().to_string(),
            is_virtual: false,
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

    pub fn scan_uid_list(&self, payload_dir: &Path) -> Vec<String> {
        let mut uids = Vec::new();
        if let Ok(entries) = WalkDir::new(payload_dir)
            .follow_links(false)
            .max_depth(PLUGIN_SCAN_MAX_DEPTH)
            .into_iter()
            .filter_entry(|e| {
                if e.file_type().is_dir() {
                    return !should_skip_dir(&e.file_name().to_string_lossy());
                }
                true
            })
            .collect::<Result<Vec<_>, _>>()
        {
            for entry in entries {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    if ext == "uid" {
                        if let Ok(content) = fs::read_to_string(path) {
                            for line in content.lines() {
                                let trimmed = line.trim();
                                if trimmed.starts_with("uid://") {
                                    uids.push(trimmed.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
        uids.sort();
        uids.dedup();
        uids
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_plugin(dir: &Path, name: &str, version: &str) -> PathBuf {
        let plugin_dir = dir.join(name);
        fs::create_dir_all(&plugin_dir).unwrap();
        let cfg_content = format!(
            "name=\"{}\"\ndescription=\"test\"\nauthor=\"test\"\nversion=\"{}\"",
            name, version
        );
        fs::write(plugin_dir.join("plugin.cfg"), cfg_content).unwrap();
        fs::write(plugin_dir.join("script.gd"), "extends Node").unwrap();
        plugin_dir
    }

    fn create_test_project(dir: &Path, name: &str, plugin_names: &[&str]) -> Project {
        let project_dir = dir.join(name);
        fs::create_dir_all(&project_dir).unwrap();
        fs::write(project_dir.join("project.godot"), "[application]\nconfig/name=\"test\"\n").unwrap();

        let addons_dir = project_dir.join("addons");
        fs::create_dir_all(&addons_dir).unwrap();

        for plugin_name in plugin_names {
            let plugin_dir = addons_dir.join(plugin_name);
            fs::create_dir_all(&plugin_dir).unwrap();
            fs::write(
                plugin_dir.join("plugin.cfg"),
                format!("name=\"{}\"\nversion=\"1.0.0\"", plugin_name),
            ).unwrap();
        }

        Project::new(
            name.to_string(),
            project_dir.to_string_lossy().to_string(),
            "4.2".to_string(),
            String::new(),
        )
    }

    #[test]
    fn test_scan_project_plugins_single_project() {
        let dir = TempDir::new().unwrap();
        let project = create_test_project(dir.path(), "my_game", &["plugin_a", "plugin_b"]);

        let manager = PluginManager::new(dir.path().join("plugins"));
        let results = manager.scan_project_plugins(&[project]).unwrap();

        assert_eq!(results.len(), 2);
        let names: Vec<&str> = results.iter().map(|r| r.plugin_name.as_str()).collect();
        assert!(names.contains(&"plugin_a"));
        assert!(names.contains(&"plugin_b"));
    }

    #[test]
    fn test_scan_project_plugins_multiple_projects() {
        let dir = TempDir::new().unwrap();
        let project1 = create_test_project(dir.path(), "game1", &["plugin_a"]);
        let project2 = create_test_project(dir.path(), "game2", &["plugin_b"]);

        let manager = PluginManager::new(dir.path().join("plugins"));
        let results = manager.scan_project_plugins(&[project1, project2]).unwrap();

        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_scan_project_plugins_same_plugin_different_projects() {
        let dir = TempDir::new().unwrap();
        let project1 = create_test_project(dir.path(), "game1", &["shared_plugin"]);
        let project2 = create_test_project(dir.path(), "game2", &["shared_plugin"]);

        let manager = PluginManager::new(dir.path().join("plugins"));
        let results = manager.scan_project_plugins(&[project1, project2]).unwrap();

        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.plugin_name == "shared_plugin"));
        assert!(results[0].project_id != results[1].project_id);
    }

    #[test]
    fn test_scan_project_plugins_no_addons() {
        let dir = TempDir::new().unwrap();
        let project_dir = dir.path().join("empty_game");
        fs::create_dir_all(&project_dir).unwrap();
        fs::write(project_dir.join("project.godot"), "[application]\n").unwrap();

        let project = Project::new(
            "empty_game".to_string(),
            project_dir.to_string_lossy().to_string(),
            "4.2".to_string(),
            String::new(),
        );

        let manager = PluginManager::new(dir.path().join("plugins"));
        let results = manager.scan_project_plugins(&[project]).unwrap();

        assert!(results.is_empty());
    }

    #[test]
    fn test_scan_project_plugins_no_plugin_cfg() {
        let dir = TempDir::new().unwrap();
        let project_dir = dir.path().join("game");
        fs::create_dir_all(&project_dir).unwrap();
        fs::write(project_dir.join("project.godot"), "[application]\n").unwrap();

        let addons_dir = project_dir.join("addons");
        let no_cfg_dir = addons_dir.join("no_cfg_plugin");
        fs::create_dir_all(&no_cfg_dir).unwrap();
        fs::write(no_cfg_dir.join("readme.txt"), "not a plugin").unwrap();

        let project = Project::new(
            "game".to_string(),
            project_dir.to_string_lossy().to_string(),
            "4.2".to_string(),
            String::new(),
        );

        let manager = PluginManager::new(dir.path().join("plugins"));
        let results = manager.scan_project_plugins(&[project]).unwrap();

        assert!(results.is_empty());
    }

    #[test]
    fn test_import_from_local() {
        let dir = TempDir::new().unwrap();
        let plugin_dir = create_test_plugin(dir.path(), "my_plugin", "1.0.0");

        let manager = PluginManager::new(dir.path().join("store"));
        let result = manager.import_from_local(&plugin_dir.to_string_lossy()).unwrap();

        assert_eq!(result.name, "my_plugin");
        assert!(!result.versions.is_empty());
        assert!(!result.content_hash.is_empty());
        assert_eq!(result.asset_type, AssetType::Plugin);
    }

    #[test]
    fn test_import_from_local_nonexistent() {
        let dir = TempDir::new().unwrap();
        let manager = PluginManager::new(dir.path().join("store"));
        let result = manager.import_from_local("/nonexistent/path");

        assert!(result.is_err());
    }

    #[test]
    fn test_import_from_local_copies_files() {
        let dir = TempDir::new().unwrap();
        let plugin_dir = create_test_plugin(dir.path(), "copy_test", "1.0.0");

        let store_dir = dir.path().join("store");
        let manager = PluginManager::new(store_dir.clone());
        let result = manager.import_from_local(&plugin_dir.to_string_lossy()).unwrap();

        let version = result.versions.first().unwrap();
        let payload_path = Path::new(&version.path);
        assert!(payload_path.exists());
        assert!(payload_path.join("plugin.cfg").exists());
        assert!(payload_path.join("script.gd").exists());
    }

    #[test]
    fn test_import_from_local_generates_hash() {
        let dir = TempDir::new().unwrap();
        let plugin_dir = create_test_plugin(dir.path(), "hash_test", "1.0.0");

        let manager = PluginManager::new(dir.path().join("store"));
        let result = manager.import_from_local(&plugin_dir.to_string_lossy()).unwrap();

        assert!(!result.content_hash.is_empty());
    }

    #[test]
    fn test_analyze_asset_type_plugin() {
        let dir = TempDir::new().unwrap();
        create_test_plugin(dir.path(), "test_plugin", "1.0.0");

        let manager = PluginManager::new(dir.path().join("store"));
        let (units, asset_type) = manager.analyze_asset_type(dir.path(), "test_plugin");

        assert_eq!(asset_type, AssetType::Plugin);
        assert!(!units.is_empty());
        assert!(!units[0].is_virtual);
    }

    #[test]
    fn test_analyze_asset_type_project() {
        let dir = TempDir::new().unwrap();
        fs::create_dir_all(dir.path()).unwrap();
        fs::write(dir.path().join("project.godot"), "[application]\n").unwrap();

        let manager = PluginManager::new(dir.path().join("store"));
        let (units, asset_type) = manager.analyze_asset_type(dir.path(), "test_project");

        assert_eq!(asset_type, AssetType::Project);
        assert!(units.is_empty());
    }

    #[test]
    fn test_analyze_asset_type_asset_pack() {
        let dir = TempDir::new().unwrap();
        fs::create_dir_all(dir.path()).unwrap();
        fs::write(dir.path().join("model.obj"), "v 0 0 0").unwrap();

        let manager = PluginManager::new(dir.path().join("store"));
        let (units, asset_type) = manager.analyze_asset_type(dir.path(), "test_assets");

        assert_eq!(asset_type, AssetType::AssetPack);
        assert!(!units.is_empty());
        assert!(units[0].is_virtual);
    }

    #[test]
    fn test_parse_plugin_cfg() {
        let dir = TempDir::new().unwrap();
        let cfg_path = dir.path().join("plugin.cfg");
        fs::write(&cfg_path, "name=\"MyPlugin\"\ndescription=\"A test\"\nauthor=\"TestAuthor\"\nversion=\"2.0.0\"").unwrap();

        let manager = PluginManager::new(dir.path().join("store"));
        let unit = manager.parse_plugin_cfg(&cfg_path, dir.path()).unwrap();

        assert_eq!(unit.name, "MyPlugin");
        assert_eq!(unit.description, "A test");
        assert_eq!(unit.author, "TestAuthor");
        assert_eq!(unit.version, "2.0.0");
        assert!(!unit.is_virtual);
    }

    #[test]
    fn test_parse_plugin_cfg_subdirectory() {
        let dir = TempDir::new().unwrap();
        let subdir = dir.path().join("addons").join("my_plugin");
        fs::create_dir_all(&subdir).unwrap();
        let cfg_path = subdir.join("plugin.cfg");
        fs::write(&cfg_path, "name=\"SubPlugin\"").unwrap();

        let manager = PluginManager::new(dir.path().join("store"));
        let unit = manager.parse_plugin_cfg(&cfg_path, dir.path()).unwrap();

        assert_eq!(unit.subdirectory.replace('\\', "/"), "addons/my_plugin");
    }

    #[test]
    fn test_detect_compatibility_godot4() {
        let dir = TempDir::new().unwrap();
        let script = dir.path().join("test.gd");
        fs::write(&script, "extends Node\n\n@export var speed: float = 1.0\n\nfunc _ready():\n\tawait get_tree().create_timer(1.0).timeout").unwrap();

        let manager = PluginManager::new(dir.path().join("store"));
        let compat = manager.detect_compatibility(dir.path());

        assert!(matches!(compat, Compatibility::Godot4 | Compatibility::Both));
    }

    #[test]
    fn test_detect_compatibility_godot3() {
        let dir = TempDir::new().unwrap();
        let script = dir.path().join("test.gd");
        fs::write(&script, "extends Node\n\nexport(float) var speed = 1.0").unwrap();

        let manager = PluginManager::new(dir.path().join("store"));
        let compat = manager.detect_compatibility(dir.path());

        assert!(matches!(compat, Compatibility::Godot3 | Compatibility::Both));
    }

    #[test]
    fn test_detect_compatibility_unknown() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("readme.txt"), "just a readme").unwrap();

        let manager = PluginManager::new(dir.path().join("store"));
        let compat = manager.detect_compatibility(dir.path());

        assert!(matches!(compat, Compatibility::Unknown));
    }

    #[test]
    fn test_import_from_local_same_content_same_hash() {
        let dir = TempDir::new().unwrap();
        let plugin1 = create_test_plugin(dir.path(), "same_plugin", "1.0.0");

        let dir2 = TempDir::new().unwrap();
        let plugin2 = create_test_plugin(dir2.path(), "same_plugin", "1.0.0");

        let manager = PluginManager::new(dir.path().join("store"));
        let result1 = manager.import_from_local(&plugin1.to_string_lossy()).unwrap();
        let result2 = manager.import_from_local(&plugin2.to_string_lossy()).unwrap();

        assert_eq!(result1.content_hash, result2.content_hash);
    }

    #[test]
    fn test_import_from_local_different_content_different_hash() {
        let dir = TempDir::new().unwrap();
        let plugin1 = create_test_plugin(dir.path(), "plugin_v1", "1.0.0");

        let dir2 = TempDir::new().unwrap();
        let plugin2_dir = dir2.path().join("plugin_v2");
        fs::create_dir_all(&plugin2_dir).unwrap();
        fs::write(plugin2_dir.join("plugin.cfg"), "name=\"plugin_v2\"\nversion=\"2.0.0\"").unwrap();
        fs::write(plugin2_dir.join("different.gd"), "extends Node2D").unwrap();

        let manager = PluginManager::new(dir.path().join("store"));
        let result1 = manager.import_from_local(&plugin1.to_string_lossy()).unwrap();
        let result2 = manager.import_from_local(&plugin2_dir.to_string_lossy()).unwrap();

        assert_ne!(result1.content_hash, result2.content_hash);
    }

    #[test]
    fn test_scan_uid_list_empty() {
        let dir = TempDir::new().unwrap();
        let manager = PluginManager::new(dir.path().join("store"));
        let uids = manager.scan_uid_list(dir.path());
        assert!(uids.is_empty());
    }

    #[test]
    fn test_scan_uid_list_with_uids() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("test.uid"), "uid://abc123\nuid://def456").unwrap();

        let manager = PluginManager::new(dir.path().join("store"));
        let uids = manager.scan_uid_list(dir.path());

        assert_eq!(uids.len(), 2);
        assert!(uids.contains(&"uid://abc123".to_string()));
        assert!(uids.contains(&"uid://def456".to_string()));
    }

    #[test]
    fn test_scan_uid_list_dedup() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("a.uid"), "uid://abc123").unwrap();
        fs::write(dir.path().join("b.uid"), "uid://abc123").unwrap();

        let manager = PluginManager::new(dir.path().join("store"));
        let uids = manager.scan_uid_list(dir.path());

        assert_eq!(uids.len(), 1);
    }

    #[test]
    fn test_find_single_subdir_single() {
        let dir = TempDir::new().unwrap();
        fs::create_dir_all(dir.path().join("only_child")).unwrap();

        let result = PluginManager::find_single_subdir(dir.path());
        assert!(result.is_some());
        assert_eq!(result.unwrap().file_name().unwrap(), "only_child");
    }

    #[test]
    fn test_find_single_subdir_multiple() {
        let dir = TempDir::new().unwrap();
        fs::create_dir_all(dir.path().join("child1")).unwrap();
        fs::create_dir_all(dir.path().join("child2")).unwrap();

        let result = PluginManager::find_single_subdir(dir.path());
        assert!(result.is_none());
    }

    #[test]
    fn test_find_single_subdir_empty() {
        let dir = TempDir::new().unwrap();

        let result = PluginManager::find_single_subdir(dir.path());
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_plugin_cfg_dir_name_differs_from_name() {
        let dir = TempDir::new().unwrap();
        let plugin_dir = dir.path().join("addons").join("godot_mcp");
        fs::create_dir_all(&plugin_dir).unwrap();
        let cfg_path = plugin_dir.join("plugin.cfg");
        fs::write(&cfg_path, "name=\"Godot MCP\"\ndescription=\"MCP plugin\"\nauthor=\"test\"\nversion=\"1.0.0\"").unwrap();

        let manager = PluginManager::new(dir.path().join("store"));
        let unit = manager.parse_plugin_cfg(&cfg_path, dir.path()).unwrap();

        assert_eq!(unit.name, "Godot MCP");
        assert_eq!(unit.dir_name, "godot_mcp");
        assert_ne!(unit.name, unit.dir_name);
    }

    #[test]
    fn test_import_from_local_dir_name_preserved() {
        let dir = TempDir::new().unwrap();
        let plugin_dir = dir.path().join("my_awesome_plugin");
        fs::create_dir_all(&plugin_dir).unwrap();
        fs::write(plugin_dir.join("plugin.cfg"), "name=\"My Awesome Plugin\"\nversion=\"1.0.0\"").unwrap();
        fs::write(plugin_dir.join("script.gd"), "extends Node").unwrap();

        let manager = PluginManager::new(dir.path().join("store"));
        let result = manager.import_from_local(&plugin_dir.to_string_lossy()).unwrap();

        assert_eq!(result.name, "My Awesome Plugin");
        let version = result.versions.first().unwrap();
        let unit = version.units.first().unwrap();
        assert_eq!(unit.dir_name, "my_awesome_plugin");
        assert_eq!(unit.name, "My Awesome Plugin");
    }

    #[test]
    fn test_analyze_asset_type_dir_name_from_folder() {
        let dir = TempDir::new().unwrap();
        let plugin_dir = dir.path().join("addons").join("cool_plugin");
        fs::create_dir_all(&plugin_dir).unwrap();
        fs::write(plugin_dir.join("plugin.cfg"), "name=\"Cool Plugin Display Name\"").unwrap();

        let manager = PluginManager::new(dir.path().join("store"));
        let (units, asset_type) = manager.analyze_asset_type(dir.path(), "cool_plugin");

        assert_eq!(asset_type, AssetType::Plugin);
        assert!(!units.is_empty());
        assert_eq!(units[0].name, "Cool Plugin Display Name");
        assert_eq!(units[0].dir_name, "cool_plugin");
    }
}
