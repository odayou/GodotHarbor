use crate::lockfile::{self, HarborLock, LockDiff, LockVerifyResult};
use crate::commands::utils::{get_storage, log_operation};
use tauri::AppHandle;

#[tauri::command]
pub fn generate_project_lock(app: AppHandle, project_id: String) -> Result<HarborLock, String> {
    let storage = get_storage(&app);
    let projects: Vec<crate::models::Project> = storage.load_or_default("projects.json");
    let project = projects.iter().find(|p| p.project_id == project_id)
        .ok_or("项目不存在".to_string())?;

    let plugins: Vec<crate::models::Plugin> = storage.load_or_default("plugins.json");
    let bindings: Vec<crate::models::ProjectBinding> = storage.load_or_default("bindings.json");
    let engines: Vec<crate::models::Engine> = storage.load_or_default("engines.json");

    let engine_bindings = if let Some(ref engine_id) = project.last_used_engine_id {
        vec![(project.project_id.clone(), engine_id.clone())]
    } else {
        vec![]
    };

    let lock = lockfile::generate_lock(project, &bindings, &plugins, &engines, &engine_bindings);
    Ok(lock)
}

#[tauri::command]
pub fn write_project_lock(app: AppHandle, project_id: String) -> Result<(), String> {
    let storage = get_storage(&app);
    let projects: Vec<crate::models::Project> = storage.load_or_default("projects.json");
    let project = projects.iter().find(|p| p.project_id == project_id)
        .ok_or("项目不存在".to_string())?;

    let plugins: Vec<crate::models::Plugin> = storage.load_or_default("plugins.json");
    let bindings: Vec<crate::models::ProjectBinding> = storage.load_or_default("bindings.json");
    let engines: Vec<crate::models::Engine> = storage.load_or_default("engines.json");

    let engine_bindings = if let Some(ref engine_id) = project.last_used_engine_id {
        vec![(project.project_id.clone(), engine_id.clone())]
    } else {
        vec![]
    };

    let lock = lockfile::generate_lock(project, &bindings, &plugins, &engines, &engine_bindings);
    lockfile::write_lock(&project.path, &lock)
        .map_err(|e| format!("写入 harbor.lock 失败: {}", e))?;

    log_operation(&app, "write_project_lock", &project_id,
        &format!("已生成 harbor.lock（{} 个插件）", lock.plugins.len()));

    Ok(())
}

#[tauri::command]
pub fn read_project_lock(app: AppHandle, project_id: String) -> Result<Option<HarborLock>, String> {
    let storage = get_storage(&app);
    let projects: Vec<crate::models::Project> = storage.load_or_default("projects.json");
    let project = projects.iter().find(|p| p.project_id == project_id)
        .ok_or("项目不存在".to_string())?;

    lockfile::read_lock(&project.path)
        .map_err(|e| format!("读取 harbor.lock 失败: {}", e))
}

#[tauri::command]
pub fn verify_project_lock(app: AppHandle, project_id: String) -> Result<LockVerifyResult, String> {
    let storage = get_storage(&app);
    let projects: Vec<crate::models::Project> = storage.load_or_default("projects.json");
    let project = projects.iter().find(|p| p.project_id == project_id)
        .ok_or("项目不存在".to_string())?;

    let lock = lockfile::read_lock(&project.path)
        .map_err(|e| format!("读取 harbor.lock 失败: {}", e))?
        .ok_or("项目未找到 harbor.lock 文件".to_string())?;

    let plugins: Vec<crate::models::Plugin> = storage.load_or_default("plugins.json");
    Ok(lockfile::verify_lock(&project.path, &lock, &plugins))
}

#[tauri::command]
pub fn diff_project_lock(app: AppHandle, project_id: String) -> Result<Option<LockDiff>, String> {
    let storage = get_storage(&app);
    let projects: Vec<crate::models::Project> = storage.load_or_default("projects.json");
    let project = projects.iter().find(|p| p.project_id == project_id)
        .ok_or("项目不存在".to_string())?;

    let existing_lock = lockfile::read_lock(&project.path)
        .map_err(|e| format!("读取 harbor.lock 失败: {}", e))?;

    let Some(existing_lock) = existing_lock else {
        return Ok(None);
    };

    let plugins: Vec<crate::models::Plugin> = storage.load_or_default("plugins.json");
    let bindings: Vec<crate::models::ProjectBinding> = storage.load_or_default("bindings.json");
    let engines: Vec<crate::models::Engine> = storage.load_or_default("engines.json");

    let engine_bindings = if let Some(ref engine_id) = project.last_used_engine_id {
        vec![(project.project_id.clone(), engine_id.clone())]
    } else {
        vec![]
    };

    let current_lock = lockfile::generate_lock(project, &bindings, &plugins, &engines, &engine_bindings);
    let diff = lockfile::diff_locks(&existing_lock, &current_lock);

    Ok(Some(diff))
}

#[tauri::command]
pub fn sync_from_lock(app: AppHandle, project_id: String, strict: Option<bool>) -> Result<Vec<String>, String> {
    let storage = get_storage(&app);
    let projects: Vec<crate::models::Project> = storage.load_or_default("projects.json");
    let project = projects.iter().find(|p| p.project_id == project_id)
        .ok_or("项目不存在".to_string())?;

    let lock = lockfile::read_lock(&project.path)
        .map_err(|e| format!("读取 harbor.lock 失败: {}", e))?
        .ok_or("项目未找到 harbor.lock 文件".to_string())?;

    let plugins: Vec<crate::models::Plugin> = storage.load_or_default("plugins.json");
    let mut bindings: Vec<crate::models::ProjectBinding> = storage.load_or_default("bindings.json");

    let strict_mode = strict.unwrap_or(false);
    let messages = lockfile::sync_from_lock(
        &project.path,
        &lock,
        &plugins,
        &mut bindings,
        &project_id,
        strict_mode,
    ).map_err(|e| format!("从锁文件同步失败: {}", e))?;

    storage.save("bindings.json", &bindings)
        .map_err(|e| format!("保存绑定列表失败: {}", e))?;

    log_operation(&app, "sync_from_lock", &project_id,
        &format!("从 harbor.lock 同步完成（{}）", if strict_mode { "严格模式" } else { "宽松模式" }));

    Ok(messages)
}

#[tauri::command]
pub fn batch_check_locks(app: AppHandle, project_ids: Vec<String>) -> Result<Vec<(String, Option<HarborLock>, LockVerifyResult)>, String> {
    let storage = get_storage(&app);
    let projects: Vec<crate::models::Project> = storage.load_or_default("projects.json");
    let plugins: Vec<crate::models::Plugin> = storage.load_or_default("plugins.json");

    let mut results = Vec::new();
    for pid in &project_ids {
        if let Some(project) = projects.iter().find(|p| &p.project_id == pid) {
            let lock = lockfile::read_lock(&project.path).ok().flatten();
            let verify_result = if let Some(ref lock) = lock {
                lockfile::verify_lock(&project.path, lock, &plugins)
            } else {
                LockVerifyResult {
                    is_valid: false,
                    mismatches: vec![],
                }
            };
            results.push((pid.clone(), lock, verify_result));
        }
    }

    Ok(results)
}
