use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use walkdir::WalkDir;

pub struct GodotResourceResolver {
    project_root: PathBuf,
    uid_cache: HashMap<String, String>,
}

impl GodotResourceResolver {
    pub fn new(project_root: PathBuf) -> Self {
        Self {
            project_root,
            uid_cache: HashMap::new(),
        }
    }

    pub fn build_uid_cache(&mut self) -> Result<()> {
        let import_dir = self.project_root.join(".godot").join("imported");

        if !import_dir.exists() {
            return self.build_uid_cache_from_import_files();
        }

        for entry in WalkDir::new(&import_dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if !path.extension().map(|e| e == "import").unwrap_or(false) {
                continue;
            }

            if let Ok(content) = fs::read_to_string(path) {
                self.parse_import_file_for_uid(&content);
            }
        }

        Ok(())
    }

    fn build_uid_cache_from_import_files(&mut self) -> Result<()> {
        for entry in WalkDir::new(&self.project_root)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if !path.extension().map(|e| e == "import").unwrap_or(false) {
                continue;
            }

            if let Ok(content) = fs::read_to_string(path) {
                self.parse_import_file_for_uid(&content);
            }
        }

        Ok(())
    }

    fn parse_import_file_for_uid(&mut self, content: &str) {
        let mut current_uid = String::new();
        let mut source_file = String::new();

        for line in content.lines() {
            let line = line.trim();

            if line.starts_with("uid=") {
                if let Some(value) = line.strip_prefix("uid=") {
                    current_uid = value.trim_matches('"').to_string();
                }
            }

            if line.starts_with("source_file=") {
                if let Some(value) = line.strip_prefix("source_file=") {
                    source_file = value.trim_matches('"').to_string();
                }
            }

            if !current_uid.is_empty() && !source_file.is_empty() {
                if current_uid.starts_with("uid://") {
                    self.uid_cache.insert(current_uid.clone(), source_file.clone());
                }
                current_uid.clear();
                source_file.clear();
            }
        }
    }

    pub fn resolve_uid(&self, uid: &str) -> Option<String> {
        self.uid_cache.get(uid).cloned()
    }

    pub fn resolve_icon_path(&self, icon_config: &str) -> Option<String> {
        if icon_config.starts_with("uid://") {
            if let Some(source_file) = self.resolve_uid(icon_config) {
                return self.res_to_abs_path(&source_file);
            }
        } else if icon_config.starts_with("res://") {
            return self.res_to_abs_path(icon_config);
        } else {
            let path = Path::new(icon_config);
            if path.exists() {
                return Some(path.to_string_lossy().to_string());
            }
        }
        None
    }

    pub fn res_to_abs_path(&self, res_path: &str) -> Option<String> {
        if !res_path.starts_with("res://") {
            return None;
        }

        let relative_path = &res_path[6..];
        let absolute_path = self.project_root.join(relative_path);

        if absolute_path.exists() {
            Some(absolute_path.to_string_lossy().to_string())
        } else {
            None
        }
    }
}

pub fn parse_import_file(path: &Path) -> Result<ImportFileInfo> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read import file: {:?}", path))?;

    let mut uid = String::new();
    let mut source_file = String::new();
    let mut remap_path = String::new();

    for line in content.lines() {
        let line = line.trim();

        if line.starts_with("uid=") {
            if let Some(value) = line.strip_prefix("uid=") {
                uid = value.trim_matches('"').to_string();
            }
        }

        if line.starts_with("source_file=") {
            if let Some(value) = line.strip_prefix("source_file=") {
                source_file = value.trim_matches('"').to_string();
            }
        }

        if line.starts_with("path=") {
            if let Some(value) = line.strip_prefix("path=") {
                remap_path = value.trim_matches('"').to_string();
            }
        }
    }

    Ok(ImportFileInfo {
        uid,
        source_file,
        remap_path,
    })
}

#[derive(Debug, Clone)]
pub struct ImportFileInfo {
    pub uid: String,
    pub source_file: String,
    pub remap_path: String,
}

pub fn extract_icon_path_advanced(project_godot_path: &Path, project_dir: &Path) -> Option<String> {
    let content = fs::read_to_string(project_godot_path).ok()?;

    for line in content.lines() {
        let line = line.trim();
        if !line.starts_with("config/icon") {
            continue;
        }

        if let Some(value) = line.split('=').nth(1) {
            let value = value.trim().trim_matches('"');

            if value.starts_with("uid://") {
                let mut resolver = GodotResourceResolver::new(project_dir.to_path_buf());
                if resolver.build_uid_cache().is_ok() {
                    if let Some(path) = resolver.resolve_icon_path(value) {
                        return Some(path);
                    }
                }
            } else if value.starts_with("res://") {
                let relative_path = &value[6..];
                let icon_path = project_dir.join(relative_path);
                if icon_path.exists() {
                    return Some(icon_path.to_string_lossy().to_string());
                }
            } else if !value.is_empty() {
                let icon_path = Path::new(value);
                if icon_path.exists() {
                    return Some(icon_path.to_string_lossy().to_string());
                }
            }
        }
    }

    None
}