use tauri::{AppHandle, Emitter};
use crate::batch_ops::*;
use crate::models::*;
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

    let settings = crate::commands::utils::load_settings(&app);
    let linker = crate::linker::Linker::new(settings.mount_strategy);

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
    ).map_err(|e| format!("应用变更失败: {}", e))?;

    if apply_result.success {
        if let Err(e) = std::fs::create_dir_all(&applied_dir) {
            eprintln!("Failed to create applied_bindings dir: {}", e);
        }
        let applied_storage = crate::storage::Storage::new(applied_dir);
        if let Err(e) = applied_storage.save(&format!("{}.json", project_id), &desired_bindings) {
            eprintln!("Failed to save applied bindings: {}", e);
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
pub fn compare_projects(app: AppHandle, project_id_a: String, project_id_b: String) -> Result<EnvironmentDiff, String> {
    let storage = get_storage(&app);

    let projects: Vec<Project> = storage.load_or_default("projects.json");
    let project_a = projects.iter().find(|p| p.project_id == project_id_a)
        .ok_or("未找到项目 A".to_string())?;
    let project_b = projects.iter().find(|p| p.project_id == project_id_b)
        .ok_or("未找到项目 B".to_string())?;

    let bindings: Vec<ProjectBinding> = storage.load_or_default("bindings.json");
    let plugins: Vec<Plugin> = storage.load_or_default("plugins.json");

    let diff = compare_environments(project_a, &bindings, project_b, &bindings, &plugins);

    log_operation(&app, "compare_projects", &format!("{} vs {}", project_id_a, project_id_b),
        &format!("项目环境比较: 仅A有{} 仅B有{} 版本不同{} 相同{}",
            diff.only_in_a.len(), diff.only_in_b.len(), diff.different_version.len(), diff.same.len()));

    Ok(diff)
}

#[tauri::command]
pub fn global_upgrade_plugin(app: AppHandle, plugin_id: String) -> Result<Vec<GlobalUpgradeResult>, String> {
    let storage = get_storage(&app);

    let plugins: Vec<Plugin> = storage.load_or_default("plugins.json");
    let bindings: Vec<ProjectBinding> = storage.load_or_default("bindings.json");

    let results = global_upgrade_plugin_inner(&plugin_id, &plugins, &bindings, &storage);

    let success_count = results.iter().filter(|r| r.success).count();
    let fail_count = results.len() - success_count;

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
pub fn batch_init_from_template(
    app: AppHandle,
    template_id: String,
    project_names: Vec<String>,
    base_dir: String,
) -> Result<BatchProjectInitResult, String> {
    let storage = get_storage(&app);

    let result = crate::batch_ops::batch_init_from_template(
        &template_id,
        &project_names,
        &base_dir,
        &storage,
    );

    let success_count = result.results.iter().filter(|r| r.success).count();
    let fail_count = result.results.len() - success_count;

    let _ = app.emit("projects-changed", ());

    log_operation(&app, "batch_init_from_template", &template_id,
        &format!("从模板批量创建项目: 成功 {} 失败 {}", success_count, fail_count));

    Ok(result)
}
