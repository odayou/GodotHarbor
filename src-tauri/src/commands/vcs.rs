use tauri::AppHandle;
use crate::vcs;
use crate::commands::utils::{get_storage, get_logger};
use crate::models::Project;

fn find_project_path(app: &AppHandle, project_id: &str) -> Result<String, String> {
    let storage = get_storage(app);
    let projects: Vec<Project> = storage.load_or_default("projects.json");
    let project = projects.iter()
        .find(|p| p.project_id == project_id)
        .ok_or_else(|| format!("未找到项目: {}", project_id))?;
    Ok(project.path.clone())
}

#[tauri::command]
pub fn get_project_vcs_info(app: AppHandle, project_id: String) -> Result<vcs::VcsInfo, String> {
    let project_path = find_project_path(&app, &project_id)?;
    vcs::get_vcs_info(&project_path)
        .map_err(|e| {
            let _ = get_logger(&app).log_error("vcs_info", &project_id, &e.to_string());
            format!("获取 VCS 信息失败: {}", e)
        })
}

#[tauri::command]
pub fn get_project_vcs_history(app: AppHandle, project_id: String, limit: Option<u32>) -> Result<Vec<vcs::VcsCommit>, String> {
    let project_path = find_project_path(&app, &project_id)?;
    let limit = limit.unwrap_or(10);
    vcs::get_commit_history(&project_path, limit)
        .map_err(|e| {
            let _ = get_logger(&app).log_error("vcs_history", &project_id, &e.to_string());
            format!("获取提交历史失败: {}", e)
        })
}

#[tauri::command]
pub async fn vcs_pull(app: AppHandle, project_id: String) -> Result<String, String> {
    let project_path = find_project_path(&app, &project_id)?;
    tauri::async_runtime::spawn_blocking(move || {
        vcs::pull(&project_path)
            .map_err(|e| format!("拉取失败: {}", e))
    }).await
        .map_err(|e| format!("拉取操作异常: {}", e))?
}

#[tauri::command]
pub async fn vcs_push(app: AppHandle, project_id: String) -> Result<String, String> {
    let project_path = find_project_path(&app, &project_id)?;
    tauri::async_runtime::spawn_blocking(move || {
        vcs::push(&project_path)
            .map_err(|e| format!("推送失败: {}", e))
    }).await
        .map_err(|e| format!("推送操作异常: {}", e))?
}

#[tauri::command]
pub async fn vcs_commit(app: AppHandle, project_id: String, message: String, add_all: Option<bool>) -> Result<String, String> {
    let project_path = find_project_path(&app, &project_id)?;
    let add_all = add_all.unwrap_or(false);
    tauri::async_runtime::spawn_blocking(move || {
        vcs::commit(&project_path, &message, add_all)
            .map_err(|e| format!("提交失败: {}", e))
    }).await
        .map_err(|e| format!("提交操作异常: {}", e))?
}

#[tauri::command]
pub fn vcs_get_diff(app: AppHandle, project_id: String) -> Result<vcs::VcsDiffSummary, String> {
    let project_path = find_project_path(&app, &project_id)?;
    vcs::get_diff_summary(&project_path)
        .map_err(|e| {
            let _ = get_logger(&app).log_error("vcs_diff", &project_id, &e.to_string());
            format!("获取差异摘要失败: {}", e)
        })
}

#[tauri::command]
pub fn vcs_update_gitignore(app: AppHandle, project_id: String) -> Result<(), String> {
    let project_path = find_project_path(&app, &project_id)?;

    // Get bindings for this project to find managed paths
    let storage = get_storage(&app);
    let bindings: Vec<crate::models::ProjectBinding> = storage.load_or_default("bindings.json");
    let project_bindings: Vec<&crate::models::ProjectBinding> = bindings.iter()
        .filter(|b| b.project_id == project_id)
        .collect();

    let managed_paths: Vec<String> = project_bindings.iter()
        .map(|b| b.mount_path.clone())
        .collect();

    vcs::ensure_gitignore(&project_path, &managed_paths)
        .map_err(|e| format!("更新 .gitignore 失败: {}", e))
}

#[tauri::command]
pub async fn batch_get_vcs_info(app: AppHandle, project_ids: Vec<String>) -> Result<Vec<(String, vcs::VcsInfo)>, String> {
    let app_clone = app.clone();
    tokio::task::spawn_blocking(move || {
        let storage = get_storage(&app_clone);
        let projects: Vec<Project> = storage.load_or_default("projects.json");

        let mut results = Vec::new();
        for project_id in &project_ids {
            if let Some(project) = projects.iter().find(|p| p.project_id == *project_id) {
                match vcs::get_vcs_info(&project.path) {
                    Ok(info) => results.push((project_id.clone(), info)),
                    Err(_) => results.push((project_id.clone(), vcs::VcsInfo::default())),
                }
            }
        }

        Ok(results)
    })
    .await
    .map_err(|e| format!("任务执行失败: {}", e))?
}

#[tauri::command]
pub fn vcs_list_branches(app: AppHandle, project_id: String) -> Result<Vec<vcs::VcsBranch>, String> {
    let project_path = find_project_path(&app, &project_id)?;
    vcs::list_branches(&project_path)
        .map_err(|e| format!("获取分支列表失败: {}", e))
}

#[tauri::command]
pub async fn vcs_checkout(app: AppHandle, project_id: String, branch: String) -> Result<(), String> {
    let project_path = find_project_path(&app, &project_id)?;
    tauri::async_runtime::spawn_blocking(move || {
        vcs::checkout_branch(&project_path, &branch)
            .map_err(|e| format!("切换分支失败: {}", e))
    }).await
        .map_err(|e| format!("切换分支操作异常: {}", e))?
}

#[tauri::command]
pub async fn vcs_create_branch(app: AppHandle, project_id: String, branch: String) -> Result<(), String> {
    let project_path = find_project_path(&app, &project_id)?;
    tauri::async_runtime::spawn_blocking(move || {
        vcs::create_branch(&project_path, &branch)
            .map_err(|e| format!("创建分支失败: {}", e))
    }).await
        .map_err(|e| format!("创建分支操作异常: {}", e))?
}
