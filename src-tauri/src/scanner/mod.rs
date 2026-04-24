use std::fs;
use std::path::Path;
use walkdir::WalkDir;
use anyhow::{Result, Context};
use crate::models::{Project, ProjectStatus};
use crate::godot_resolver::extract_icon_path_advanced;

pub struct ProjectScanner;

impl ProjectScanner {
    pub fn scan_directory(root_path: &str) -> Result<Vec<Project>> {
        let mut projects = Vec::new();
        let root = Path::new(root_path);

        if !root.exists() {
            return Ok(projects);
        }

        for entry in WalkDir::new(root)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            if path.file_name().map(|f| f == "project.godot").unwrap_or(false) {
                if let Ok(project) = Self::parse_project(path) {
                    projects.push(project);
                }
            }
        }

        Ok(projects)
    }

    pub fn parse_project(project_godot_path: &Path) -> Result<Project> {
        let content = fs::read_to_string(project_godot_path)
            .context("Failed to read project.godot")?;

        let project_dir = project_godot_path.parent()
            .context("Failed to get project directory")?;

        let project_name = project_dir.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown Project")
            .to_string();

        let project_path = project_dir.to_string_lossy().to_string();

        let godot_version = Self::extract_godot_version(&content);

        let icon_path = Self::extract_icon_path_with_uid_fallback(&content, project_dir);

        let mut project = Project::new(project_name, project_path, godot_version, icon_path);

        if Self::check_project_health(project_dir) {
            project.status = ProjectStatus::Ready;
        } else {
            project.status = ProjectStatus::Warning;
        }

        Ok(project)
    }

    fn extract_godot_version(content: &str) -> String {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("config/features") {
                if let Some(value) = line.split('=').nth(1) {
                    let value = value.trim();
                    let version = Self::find_version_in_string(value);
                    if !version.is_empty() {
                        return version;
                    }
                }
            }
        }

        "Unknown".to_string()
    }

    fn extract_icon_path_with_uid_fallback(content: &str, project_dir: &Path) -> String {
        if let Some(icon_path) = extract_icon_path_advanced(&project_dir.join("project.godot"), project_dir) {
            return icon_path;
        }

        Self::extract_icon_path_legacy(content, project_dir)
    }

    fn extract_icon_path_legacy(content: &str, project_dir: &Path) -> String {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("config/icon") {
                if let Some(value) = line.split('=').nth(1) {
                    let value = value.trim().trim_matches('"');
                    if value.starts_with("res://") {
                        let relative_path = &value[6..];
                        let icon_path = project_dir.join(relative_path);
                        if icon_path.exists() {
                            return icon_path.to_string_lossy().to_string();
                        }
                    } else if !value.is_empty() {
                        let icon_path = Path::new(value);
                        if icon_path.exists() {
                            return icon_path.to_string_lossy().to_string();
                        }
                    }
                }
            }
        }

        String::new()
    }

    fn find_version_in_string(s: &str) -> String {
        let mut best_match = String::new();
        let mut in_quotes = false;
        let mut current = String::new();

        for ch in s.chars() {
            if ch == '"' {
                if in_quotes {
                    if Self::is_version_string(&current) {
                        if current.len() > best_match.len() {
                            best_match = current.clone();
                        }
                    }
                    current.clear();
                }
                in_quotes = !in_quotes;
            } else if in_quotes {
                current.push(ch);
            }
        }

        best_match
    }

    fn is_version_string(s: &str) -> bool {
        if s.is_empty() {
            return false;
        }
        let parts: Vec<&str> = s.split('.').collect();
        if parts.is_empty() {
            return false;
        }
        if parts[0].parse::<u32>().is_err() {
            return false;
        }
        parts.iter().all(|p| p.parse::<u32>().is_ok())
    }

    fn check_project_health(project_dir: &Path) -> bool {
        project_dir.exists()
            && project_dir.join("project.godot").exists()
            && fs::metadata(project_dir).map(|m| !m.permissions().readonly()).unwrap_or(false)
    }
}
