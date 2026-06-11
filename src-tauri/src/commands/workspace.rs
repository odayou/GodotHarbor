use tauri::AppHandle;
use crate::workspace::{self, Workspace, WorkspaceSummary};
use crate::models::Project;
use super::utils::*;

const WORKSPACES_FILE: &str = "workspaces.json";

fn load_workspaces(app: &AppHandle) -> Vec<Workspace> {
    let storage = get_storage(app);
    storage.load_or_default(WORKSPACES_FILE)
}

fn save_workspaces(app: &AppHandle, workspaces: &Vec<Workspace>) -> Result<(), String> {
    let storage = get_storage(app);
    storage.save(WORKSPACES_FILE, workspaces)
        .map_err(|e| format!("保存工作区失败: {}", e))
}

#[tauri::command]
pub fn create_workspace(
    app: AppHandle,
    name: String,
    icon: Option<String>,
    color: Option<String>,
) -> Result<Workspace, String> {
    if name.trim().is_empty() {
        return Err("工作区名称不能为空".to_string());
    }

    let ws = workspace::create_workspace(
        &name,
        &icon.unwrap_or_else(|| "📁".to_string()),
        &color.unwrap_or_else(|| "#3B82F6".to_string()),
    );

    let mut workspaces = load_workspaces(&app);
    workspaces.push(ws.clone());
    save_workspaces(&app, &workspaces)?;

    log_operation(&app, "create_workspace", &ws.workspace_id,
        &format!("创建工作区: {}", name));

    Ok(ws)
}

#[tauri::command]
pub fn update_workspace(app: AppHandle, workspace: Workspace) -> Result<(), String> {
    if workspace.name.trim().is_empty() {
        return Err("工作区名称不能为空".to_string());
    }

    let mut workspaces = load_workspaces(&app);
    let idx = workspaces.iter().position(|w| w.workspace_id == workspace.workspace_id)
        .ok_or("未找到指定工作区".to_string())?;

    let mut updated = workspace;
    workspace::update_workspace(&mut updated);
    workspaces[idx] = updated.clone();
    save_workspaces(&app, &workspaces)?;

    log_operation(&app, "update_workspace", &updated.workspace_id,
        &format!("更新工作区: {}", updated.name));

    Ok(())
}

#[tauri::command]
pub fn delete_workspace(app: AppHandle, workspace_id: String) -> Result<(), String> {
    let mut workspaces = load_workspaces(&app);
    let ws = workspaces.iter().find(|w| w.workspace_id == workspace_id)
        .ok_or("未找到指定工作区".to_string())?;
    let name = ws.name.clone();

    workspaces.retain(|w| w.workspace_id != workspace_id);
    save_workspaces(&app, &workspaces)?;

    // If the deleted workspace was active, clear the active workspace
    let mut settings = load_settings(&app);
    if settings.active_workspace_id.as_ref() == Some(&workspace_id) {
        settings.active_workspace_id = None;
        save_settings_to_config(&app, &settings)?;
    }

    log_operation(&app, "delete_workspace", &workspace_id,
        &format!("删除工作区: {}", name));

    Ok(())
}

#[tauri::command]
pub fn list_workspaces(app: AppHandle) -> Result<Vec<WorkspaceSummary>, String> {
    let workspaces = load_workspaces(&app);
    let settings = load_settings(&app);
    let active_id = settings.active_workspace_id.as_deref();

    let summaries: Vec<WorkspaceSummary> = workspaces.iter().map(|ws| {
        WorkspaceSummary {
            workspace_id: ws.workspace_id.clone(),
            name: ws.name.clone(),
            icon: ws.icon.clone(),
            color: ws.color.clone(),
            project_count: ws.project_ids.len(),
            is_active: active_id == Some(ws.workspace_id.as_str()),
        }
    }).collect();

    Ok(summaries)
}

#[tauri::command]
pub fn get_workspace(app: AppHandle, workspace_id: String) -> Result<Workspace, String> {
    let workspaces = load_workspaces(&app);
    workspaces.into_iter()
        .find(|w| w.workspace_id == workspace_id)
        .ok_or("未找到指定工作区".to_string())
}

#[tauri::command]
pub fn add_project_to_workspace(
    app: AppHandle,
    workspace_id: String,
    project_id: String,
) -> Result<(), String> {
    let mut workspaces = load_workspaces(&app);
    let ws = workspaces.iter_mut()
        .find(|w| w.workspace_id == workspace_id)
        .ok_or("未找到指定工作区".to_string())?;

    if ws.project_ids.contains(&project_id) {
        return Ok(()); // Already in workspace
    }

    // Verify project exists
    let storage = get_storage(&app);
    let projects: Vec<Project> = storage.load_or_default("projects.json");
    if !projects.iter().any(|p| p.project_id == project_id) {
        return Err("未找到指定项目".to_string());
    }

    ws.project_ids.push(project_id.clone());
    ws.updated_at = chrono::Utc::now();
    save_workspaces(&app, &workspaces)?;

    log_operation(&app, "add_project_to_workspace", &workspace_id,
        &format!("添加项目到工作区: {}", project_id));

    Ok(())
}

#[tauri::command]
pub fn remove_project_from_workspace(
    app: AppHandle,
    workspace_id: String,
    project_id: String,
) -> Result<(), String> {
    let mut workspaces = load_workspaces(&app);
    let ws = workspaces.iter_mut()
        .find(|w| w.workspace_id == workspace_id)
        .ok_or("未找到指定工作区".to_string())?;

    ws.project_ids.retain(|id| id != &project_id);
    ws.updated_at = chrono::Utc::now();
    save_workspaces(&app, &workspaces)?;

    log_operation(&app, "remove_project_from_workspace", &workspace_id,
        &format!("从工作区移除项目: {}", project_id));

    Ok(())
}

#[tauri::command]
pub fn get_active_workspace(app: AppHandle) -> Result<Option<String>, String> {
    let settings = load_settings(&app);
    Ok(settings.active_workspace_id)
}

#[tauri::command]
pub fn set_active_workspace(
    app: AppHandle,
    workspace_id: Option<String>,
) -> Result<(), String> {
    // Validate workspace exists if provided
    if let Some(ref id) = workspace_id {
        let workspaces = load_workspaces(&app);
        if !workspaces.iter().any(|w| w.workspace_id == *id) {
            return Err("未找到指定工作区".to_string());
        }
    }

    let mut settings = load_settings(&app);
    settings.active_workspace_id = workspace_id.clone();
    save_settings_to_config(&app, &settings)?;

    let desc = workspace_id.as_ref()
        .map(|id| format!("切换到工作区: {}", id))
        .unwrap_or_else(|| "切换到全部项目视图".to_string());
    log_operation(&app, "set_active_workspace", &workspace_id.unwrap_or_default(), &desc);

    Ok(())
}

#[tauri::command]
pub fn move_project_to_workspace(
    app: AppHandle,
    project_id: String,
    from_workspace_id: Option<String>,
    to_workspace_id: Option<String>,
) -> Result<(), String> {
    // Remove from source workspace
    if let Some(ref from_id) = from_workspace_id {
        let mut workspaces = load_workspaces(&app);
        if let Some(ws) = workspaces.iter_mut().find(|w| w.workspace_id == *from_id) {
            ws.project_ids.retain(|id| id != &project_id);
            ws.updated_at = chrono::Utc::now();
        }
        save_workspaces(&app, &workspaces)?;
    }

    // Add to target workspace
    if let Some(ref to_id) = to_workspace_id {
        let mut workspaces = load_workspaces(&app);
        if let Some(ws) = workspaces.iter_mut().find(|w| w.workspace_id == *to_id) {
            if !ws.project_ids.contains(&project_id) {
                ws.project_ids.push(project_id.clone());
                ws.updated_at = chrono::Utc::now();
            }
        }
        save_workspaces(&app, &workspaces)?;
    }

    log_operation(&app, "move_project_to_workspace", &project_id,
        &format!("移动项目: {} -> {}",
            from_workspace_id.as_deref().unwrap_or("无"),
            to_workspace_id.as_deref().unwrap_or("无")));

    Ok(())
}
