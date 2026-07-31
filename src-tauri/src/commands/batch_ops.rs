use tauri::{AppHandle, Emitter};
use crate::batch_ops::*;
use crate::models::*;
use crate::linker::Linker;
use crate::storage::Storage;
use super::utils::*;

#[tauri::command]
pub fn create_snapshot(app: AppHandle, project_id: String) -> Result<EnvironmentSnapshot, String> {
    let storage = get_storage(&app);
    let data_dir = get_data_dir(&app);

    let projects: Vec<Project> = storage.load_or_default("projects.json");
    let project = projects.iter().find(|p| p.project_id == project_id)
        .ok_or("未找到指定项目".to_string())?;

    let bindings: Vec<ProjectBinding> = storage.load_or_default("bindings.json");
    let plugins: Vec<Plugin> = storage.load_or_default("plugins.json");
    let engines: Vec<Engine> = storage.load_or_default("engines.json");

    let snapshot = create_environment_snapshot(project, &bindings, &plugins, &engines);
    save_snapshot(&data_dir, &snapshot)?;

    log_operation(&app, "create_snapshot", &project_id,
        &format!("已创建快照: {} ({} 个插件)", snapshot.snapshot_id, snapshot.plugins.len()));

    Ok(snapshot)
}

#[tauri::command]
pub fn list_snapshots(app: AppHandle, project_id: String) -> Result<Vec<EnvironmentSnapshot>, String> {
    let data_dir = get_data_dir(&app);
    load_snapshots(&data_dir, &project_id)
}

#[tauri::command]
pub fn restore_snapshot(app: AppHandle, project_id: String, snapshot_id: String) -> Result<Vec<String>, String> {
    let data_dir = get_data_dir(&app);
    let storage = get_storage(&app);

    let snapshots = load_snapshots(&data_dir, &project_id)?;
    let snapshot = snapshots.iter().find(|s| s.snapshot_id == snapshot_id)
        .ok_or("未找到指定快照".to_string())?
        .clone();

    let restored = restore_from_snapshot(&snapshot, &storage)?;

    // Apply changes for the project
    let projects: Vec<Project> = storage.load_or_default("projects.json");
    let project = projects.iter().find(|p| p.project_id == project_id)
        .ok_or_else(|| "未找到指定项目".to_string())?;

    let bindings: Vec<ProjectBinding> = storage.load_or_default("bindings.json");
    let desired_bindings: Vec<ProjectBinding> = bindings.iter()
        .filter(|b| b.project_id == project_id)
        .cloned()
        .collect();

    let linker = crate::linker::Linker::new();

    let data_dir = crate::commands::utils::get_data_dir(&app);
    let plugin_base_path = data_dir.join("plugins");

    let applied_dir = data_dir.join("applied_bindings");
    let applied_file = applied_dir.join(format!("{}.json", project_id));
    let current_bindings: Vec<ProjectBinding> = if applied_file.exists() {
        let applied_storage = crate::storage::Storage::new(applied_dir.clone());
        applied_storage.load_or_default::<Vec<ProjectBinding>>(&format!("{}.json", project_id))
    } else {
        Vec::new()
    };

    let apply_result = linker.apply_bindings(
        &project.path,
        &current_bindings,
        &desired_bindings,
        &plugin_base_path.to_string_lossy(),
        &data_dir.to_string_lossy(),
    ).map_err(|e| format!("应用变更失败: {}", e))?;

    if apply_result.success {
        if let Err(e) = std::fs::create_dir_all(&applied_dir) {
            eprintln!("Failed to create applied_bindings dir: {}", e);
        }
        let applied_storage = crate::storage::Storage::new(applied_dir);
        if let Err(e) = applied_storage.save(&format!("{}.json", project_id), &desired_bindings) {
            eprintln!("Failed to save applied bindings: {}", e);
        }
        if let Err(e) = crate::commands::lockfile::refresh_project_lock(&app, &project_id) {
            eprintln!("Failed to write harbor.lock: {}", e);
        }
    }

    let _ = app.emit("bindings-changed", ());

    log_operation(&app, "restore_snapshot", &snapshot_id,
        &format!("已从快照恢复项目: {} 个插件, 应用结果: 创建 {} 移除 {} 错误 {}",
            restored.len(), apply_result.created.len(), apply_result.removed.len(), apply_result.errors.len()));

    Ok(restored)
}

#[tauri::command]
pub fn delete_snapshot(app: AppHandle, snapshot_id: String) -> Result<(), String> {
    let data_dir = get_data_dir(&app);

    // Find which project this snapshot belongs to by scanning all snapshot directories
    let snapshots_dir = data_dir.join("snapshots");
    if !snapshots_dir.exists() {
        return Err("快照目录不存在".to_string());
    }

    let entries = std::fs::read_dir(&snapshots_dir)
        .map_err(|e| format!("读取快照目录失败: {}", e))?;

    for entry in entries.flatten() {
        let project_dir = entry.path();
        if project_dir.is_dir() {
            let snapshot_file = project_dir.join(format!("{}.json", snapshot_id));
            if snapshot_file.exists() {
                let project_id = project_dir.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                delete_snapshot_file(&data_dir, &snapshot_id, &project_id)?;
                log_operation(&app, "delete_snapshot", &snapshot_id, "已删除快照");
                return Ok(());
            }
        }
    }

    Err("未找到指定快照".to_string())
}

#[tauri::command]
pub fn global_upgrade_plugin(app: AppHandle, plugin_id: String) -> Result<Vec<GlobalUpgradeResult>, String> {
    let storage = get_storage(&app);

    let plugins: Vec<Plugin> = storage.load_or_default("plugins.json");
    let bindings: Vec<ProjectBinding> = storage.load_or_default("bindings.json");

    let results = global_upgrade_plugin_inner(&plugin_id, &plugins, &bindings, &storage);

    let success_count = results.iter().filter(|r| r.success).count();
    let fail_count = results.len() - success_count;

    // Apply bindings for each affected project
    // Reload bindings since global_upgrade_plugin_inner updated them
    let updated_bindings: Vec<ProjectBinding> = storage.load_or_default("bindings.json");
    let projects: Vec<Project> = storage.load_or_default("projects.json");
    let linker = crate::linker::Linker::new();
    let data_dir = crate::commands::utils::get_data_dir(&app);
    let plugin_base_path = data_dir.join("plugins");
    let applied_dir = data_dir.join("applied_bindings");

    // Collect unique project IDs from affected bindings
    let affected_project_ids: std::collections::HashSet<String> = updated_bindings
        .iter()
        .filter(|b| b.plugin_id == plugin_id)
        .map(|b| b.project_id.clone())
        .collect();

    for project_id in &affected_project_ids {
        let project = match projects.iter().find(|p| p.project_id == *project_id) {
            Some(p) => p,
            None => continue,
        };

        let desired_bindings: Vec<ProjectBinding> = updated_bindings
            .iter()
            .filter(|b| b.project_id == *project_id)
            .cloned()
            .collect();

        let applied_file = applied_dir.join(format!("{}.json", project_id));
        let current_bindings: Vec<ProjectBinding> = if applied_file.exists() {
            let applied_storage = crate::storage::Storage::new(applied_dir.clone());
            applied_storage.load_or_default::<Vec<ProjectBinding>>(&format!("{}.json", project_id))
        } else {
            Vec::new()
        };

        let apply_result = linker.apply_bindings(
            &project.path,
            &current_bindings,
            &desired_bindings,
            &plugin_base_path.to_string_lossy(),
            &data_dir.to_string_lossy(),
        );

        if let Ok(apply_result) = apply_result {
            if apply_result.success {
                if let Err(e) = std::fs::create_dir_all(&applied_dir) {
                    eprintln!("Failed to create applied_bindings dir: {}", e);
                }
                let applied_storage = crate::storage::Storage::new(applied_dir.clone());
                if let Err(e) = applied_storage.save(&format!("{}.json", project_id), &desired_bindings) {
                    eprintln!("Failed to save applied bindings: {}", e);
                }
                if let Err(e) = crate::commands::lockfile::refresh_project_lock(&app, project_id) {
                    eprintln!("Failed to write harbor.lock: {}", e);
                }
            }
        }
    }

    let _ = app.emit("bindings-changed", ());

    log_operation(&app, "global_upgrade_plugin", &plugin_id,
        &format!("全局升级插件: 成功 {} 失败 {}", success_count, fail_count));

    Ok(results)
}

fn global_upgrade_plugin_inner(
    plugin_id: &str,
    plugins: &[Plugin],
    bindings: &[ProjectBinding],
    storage: &crate::storage::Storage,
) -> Vec<GlobalUpgradeResult> {
    crate::batch_ops::global_upgrade_plugin(plugin_id, plugins, bindings, storage)
}

#[tauri::command]
pub fn sync_all_bindings(app: AppHandle) -> Result<BatchApplyResult, String> {
    let storage = get_storage(&app);
    let projects: Vec<Project> = storage.load_or_default("projects.json");
    let all_bindings: Vec<ProjectBinding> = storage.load_or_default("bindings.json");
    let linker = Linker::new();
    let data_dir = get_data_dir(&app);
    let plugin_base_path = data_dir.join("plugins");
    let applied_dir = data_dir.join("applied_bindings");

    let mut results: Vec<ProjectApplyResult> = Vec::new();
    for project in &projects {
        let project_id = &project.project_id;
        let desired_bindings: Vec<ProjectBinding> = all_bindings.iter()
            .filter(|b| b.project_id == *project_id)
            .cloned()
            .collect();

        if desired_bindings.is_empty() {
            results.push(ProjectApplyResult {
                project_id: project_id.clone(),
                project_name: project.name.clone(),
                success: true,
                created: Vec::new(),
                removed: Vec::new(),
                errors: Vec::new(),
            });
            continue;
        }

        let applied_file = applied_dir.join(format!("{}.json", project_id));
        let current_bindings: Vec<ProjectBinding> = if applied_file.exists() {
            let applied_storage = Storage::new(applied_dir.clone());
            applied_storage.load_or_default::<Vec<ProjectBinding>>(&format!("{}.json", project_id))
        } else {
            Vec::new()
        };

        match linker.apply_bindings(
            &project.path,
            &current_bindings,
            &desired_bindings,
            &plugin_base_path.to_string_lossy(),
            &data_dir.to_string_lossy(),
        ) {
            Ok(apply_result) => {
                if apply_result.success {
                    if let Err(e) = std::fs::create_dir_all(&applied_dir) {
                        eprintln!("Failed to create applied_bindings dir: {}", e);
                    }
                    let applied_storage = Storage::new(applied_dir.clone());
                    if let Err(e) = applied_storage.save(&format!("{}.json", project_id), &desired_bindings) {
                        eprintln!("Failed to save applied bindings: {}", e);
                    }
                    if let Err(e) = crate::commands::lockfile::refresh_project_lock(&app, project_id) {
                        eprintln!("Failed to write harbor.lock: {}", e);
                    }
                }
                results.push(ProjectApplyResult {
                    project_id: project_id.clone(),
                    project_name: project.name.clone(),
                    success: apply_result.success,
                    created: apply_result.created,
                    removed: apply_result.removed,
                    errors: apply_result.errors,
                });
            }
            Err(e) => {
                results.push(ProjectApplyResult {
                    project_id: project_id.clone(),
                    project_name: project.name.clone(),
                    success: false,
                    created: Vec::new(),
                    removed: Vec::new(),
                    errors: vec![format!("应用变更失败: {}", e)],
                });
            }
        }
    }

    let _ = app.emit("bindings-changed", ());

    log_operation(&app, "sync_all_bindings", "",
        &format!("同步全部项目绑定完成，共处理 {} 个项目", results.len()));

    Ok(BatchApplyResult { results })
}
