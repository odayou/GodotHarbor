use std::fs;
use std::path::Path;
use walkdir::WalkDir;
use anyhow::{Result, Context};
use crate::models::{Project, ProjectStatus};

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
        
        let mut project = Project::new(project_name, project_path, godot_version);
        
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
                    if value.contains("4.") {
                        return "4.x".to_string();
                    } else if value.contains("3.") {
                        return "3.x".to_string();
                    }
                }
            }
        }
        "Unknown".to_string()
    }

    fn check_project_health(project_dir: &Path) -> bool {
        project_dir.exists() 
            && project_dir.join("project.godot").exists()
            && fs::metadata(project_dir).map(|m| !m.permissions().readonly()).unwrap_or(false)
    }
}
