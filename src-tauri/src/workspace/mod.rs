use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::models::{MountStrategy, Project};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub workspace_id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub color: String,
    #[serde(default)]
    pub scan_directories: Vec<String>,
    #[serde(default)]
    pub project_ids: Vec<String>,
    #[serde(default)]
    pub plugin_favorites: Vec<String>,
    #[serde(default)]
    pub default_engine_id: Option<String>,
    #[serde(default)]
    pub mount_strategy: Option<MountStrategy>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Workspace {
    pub fn new(name: String, icon: String, color: String) -> Self {
        let now = Utc::now();
        Self {
            workspace_id: Uuid::new_v4().to_string(),
            name,
            description: String::new(),
            icon,
            color,
            scan_directories: Vec::new(),
            project_ids: Vec::new(),
            plugin_favorites: Vec::new(),
            default_engine_id: None,
            mount_strategy: None,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceSummary {
    pub workspace_id: String,
    pub name: String,
    pub icon: String,
    pub color: String,
    pub project_count: usize,
    pub is_active: bool,
}

pub fn create_workspace(
    name: &str,
    icon: &str,
    color: &str,
) -> Workspace {
    Workspace::new(name.to_string(), icon.to_string(), color.to_string())
}

pub fn update_workspace(workspace: &mut Workspace) {
    workspace.updated_at = Utc::now();
}

pub fn get_filtered_projects(
    active_workspace: Option<&Workspace>,
    all_projects: &[Project],
) -> Vec<Project> {
    match active_workspace {
        Some(ws) => {
            let ws_project_ids: std::collections::HashSet<&str> =
                ws.project_ids.iter().map(|s| s.as_str()).collect();
            all_projects
                .iter()
                .filter(|p| ws_project_ids.contains(p.project_id.as_str()))
                .cloned()
                .collect()
        }
        None => all_projects.to_vec(),
    }
}
