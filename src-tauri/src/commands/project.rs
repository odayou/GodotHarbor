use serde::{Serialize, Deserialize};
use tauri::{AppHandle, Emitter};
use crate::models::*;
use crate::scanner::ProjectScanner;
use super::utils::*;
use super::system::get_default_scan_dirs;

#[tauri::command]
pub async fn scan_projects(app: AppHandle, root_dirs: Vec<String>) -> Result<Vec<Project>, String> {
    if root_dirs.is_empty() {
        log_error(&app, "scan_projects", "", "未提供扫描目录");
        return Err("请至少指定一个扫描目录".to_string());
    }

    let valid_dirs: Vec<String> = root_dirs.iter()
        .filter(|d| std::path::Path::new(d).exists())
        .cloned()
        .collect();

    if valid_dirs.is_empty() {
        return Err("所有指定的目录均不存在".to_string());
    }

    let app_clone = app.clone();
    let all_projects = tokio::task::spawn_blocking(move || {
        ProjectScanner::scan_directories_parallel(&valid_dirs)
    }).await.map_err(|e| format!("扫描任务失败: {}", e))?
    .map_err(|e| format!("扫描失败: {}", e))?;

    let storage = get_storage(&app_clone);
    let mut existing_projects: Vec<Project> = storage.load_or_default("projects.json");

    for project in &all_projects {
        if let Some(index) = existing_projects.iter().position(|p| p.path == project.path) {
            let mut existing = existing_projects[index].clone();
            existing.name = project.name.clone();
            existing.godot_version = project.godot_version.clone();
            existing_projects[index] = existing;
        } else {
            existing_projects.push(project.clone());
        }
    }

    storage.save("projects.json", &existing_projects)
        .map_err(|e| format!("保存项目列表失败: {}", e))?;

    log_operation(&app, "scan_projects", &root_dirs.join(", "),
        &format!("扫描完成，发现 {} 个项目", all_projects.len()));

    Ok(all_projects)
}

#[tauri::command]
pub fn get_projects(app: AppHandle) -> Result<Vec<Project>, String> {
    let storage = get_storage(&app);
    let mut projects: Vec<Project> = storage.load_or_default("projects.json");

    for project in projects.iter_mut() {
        let project_path = std::path::Path::new(&project.path);
        if !project_path.exists() || !project_path.join("project.godot").exists() {
            project.status = ProjectStatus::MissingSource;
        }
    }

    Ok(projects)
}

#[tauri::command]
pub fn add_project(app: AppHandle, path: String) -> Result<Project, String> {
    let project_path = std::path::Path::new(&path);

    if !project_path.exists() {
        return Err("指定的路径不存在".to_string());
    }

    let project_godot = project_path.join("project.godot");

    if !project_godot.exists() {
        return Err("指定路径下未找到 project.godot 文件，请确认是否为 Godot 项目目录".to_string());
    }

    let project = ProjectScanner::parse_project(&project_godot)
        .map_err(|e| format!("解析项目失败: {}", e))?;

    let storage = get_storage(&app);
    let mut projects: Vec<Project> = storage.load_or_default("projects.json");

    if projects.iter().any(|p| p.path == project.path) {
        return Err("该项目已存在，请勿重复添加".to_string());
    }

    let project_name = project.name.clone();
    projects.push(project.clone());
    storage.save("projects.json", &projects)
        .map_err(|e| format!("保存项目失败: {}", e))?;

    log_operation(&app, "add_project", &path, &format!("已添加项目: {}", project_name));
    Ok(project)
}

#[tauri::command]
pub async fn import_project_from_git(
    app: AppHandle,
    git_url: String,
    target_dir: Option<String>,
) -> Result<Project, String> {
    if git_url.is_empty() {
        return Err("请输入 Git 仓库地址".to_string());
    }

    let repo_name = git_url
        .split('/')
        .last()
        .unwrap_or("unknown")
        .trim_end_matches(".git")
        .to_string();

    let data_dir = get_data_dir(&app);
    let clone_base = if let Some(dir) = &target_dir {
        if !dir.is_empty() {
            std::path::PathBuf::from(dir)
        } else {
            data_dir.join("projects")
        }
    } else {
        data_dir.join("projects")
    };

    std::fs::create_dir_all(&clone_base)
        .map_err(|e| format!("创建项目目录失败: {}", e))?;

    let clone_target = clone_base.join(&repo_name);

    if clone_target.exists() {
        let project_godot = clone_target.join("project.godot");
        if project_godot.exists() {
            let storage = get_storage(&app);
            let projects: Vec<Project> = storage.load_or_default("projects.json");
            let existing_path = clone_target.to_string_lossy().to_string();
            if projects.iter().any(|p| p.path == existing_path) {
                return Err("该项目已存在，请勿重复添加".to_string());
            }
            let project = ProjectScanner::parse_project(&project_godot)
                .map_err(|e| format!("解析项目失败: {}", e))?;
            let project_name = project.name.clone();
            let mut all_projects: Vec<Project> = storage.load_or_default("projects.json");
            all_projects.push(project.clone());
            storage.save("projects.json", &all_projects)
                .map_err(|e| format!("保存项目失败: {}", e))?;
            log_operation(&app, "import_project_from_git", &git_url,
                &format!("从 Git 导入项目（目录已存在）: {}", project_name));
            let _ = app.emit("projects-changed", ());
            return Ok(project);
        }
        return Err(format!("目标目录已存在但不是有效的 Godot 项目: {}", clone_target.display()));
    }

    let mut callbacks = git2::RemoteCallbacks::new();
    let app_handle_clone = app.clone();
    let git_url_for_callback = git_url.clone();
    callbacks.transfer_progress(move |progress| {
        let received = progress.received_objects();
        let total = progress.total_objects();
        let percentage = if total > 0 {
            (received as f64 / total as f64 * 100.0) as u32
        } else {
            0
        };
        let _ = app_handle_clone.emit("git-clone-progress", serde_json::json!({
            "url": git_url_for_callback,
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

    if let Err(e) = builder.clone(&git_url, &clone_target) {
        let _ = std::fs::remove_dir_all(&clone_target);
        return Err(format!("克隆 Git 仓库失败: {}", e));
    }

    let project_godot = clone_target.join("project.godot");
    if !project_godot.exists() {
        let _ = std::fs::remove_dir_all(&clone_target);
        return Err("克隆成功但未找到 project.godot 文件，不是有效的 Godot 项目".to_string());
    }

    let project = ProjectScanner::parse_project(&project_godot)
        .map_err(|e| {
            let _ = std::fs::remove_dir_all(&clone_target);
            format!("解析项目失败: {}", e)
        })?;

    let project_name = project.name.clone();
    let storage = get_storage(&app);
    let mut all_projects: Vec<Project> = storage.load_or_default("projects.json");

    if all_projects.iter().any(|p| p.path == project.path) {
        let _ = std::fs::remove_dir_all(&clone_target);
        return Err("该项目已存在，请勿重复添加".to_string());
    }

    all_projects.push(project.clone());
    storage.save("projects.json", &all_projects)
        .map_err(|e| format!("保存项目失败: {}", e))?;

    log_operation(&app, "import_project_from_git", &git_url,
        &format!("从 Git 导入项目: {}", project_name));

    let _ = app.emit("projects-changed", ());

    Ok(project)
}

#[tauri::command]
pub fn remove_project(app: AppHandle, project_id: String, delete_files: Option<bool>) -> Result<(), String> {
    let storage = get_storage(&app);
    let mut projects: Vec<Project> = storage.load_or_default("projects.json");

    let project = projects.iter().find(|p| p.project_id == project_id)
        .ok_or("未找到指定项目".to_string())?;
    let project_name = project.name.clone();
    let project_path = project.path.clone();

    if delete_files.unwrap_or(false) {
        let path = std::path::Path::new(&project_path);
        if path.exists() {
            std::fs::remove_dir_all(path)
                .map_err(|e| format!("删除项目文件失败: {}", e))?;
        }
    }

    projects.retain(|p| p.project_id != project_id);

    let mut bindings: Vec<ProjectBinding> = storage.load_or_default("bindings.json");
    let had_bindings = bindings.iter().any(|b| b.project_id == project_id);
    bindings.retain(|b| b.project_id != project_id);
    if had_bindings {
        storage.save("bindings.json", &bindings)
            .map_err(|e| format!("保存绑定列表失败: {}", e))?;
    }

    storage.save("projects.json", &projects)
        .map_err(|e| format!("保存项目列表失败: {}", e))?;

    log_operation(&app, "remove_project", &project_id, &format!("已删除项目: {}{}", project_name, if delete_files.unwrap_or(false) { "（含文件）" } else { "" }));
    Ok(())
}


#[tauri::command]
pub fn update_project_group(app: AppHandle, project_id: String, group: String) -> Result<(), String> {
    let storage = get_storage(&app);
    let mut projects: Vec<Project> = storage.load_or_default("projects.json");

    let project = projects.iter_mut()
        .find(|p| p.project_id == project_id)
        .ok_or("未找到指定项目".to_string())?;

    project.group = group.clone();

    storage.save("projects.json", &projects)
        .map_err(|e| format!("保存项目分组失败: {}", e))?;

    log_operation(&app, "update_project_group", &project_id, &format!("项目分组已更新: {}", group));
    Ok(())
}

#[tauri::command]
pub fn get_project_groups(app: AppHandle) -> Result<Vec<String>, String> {
    let storage = get_storage(&app);
    let projects: Vec<Project> = storage.load_or_default("projects.json");

    let mut groups: Vec<String> = projects.iter()
        .filter(|p| !p.group.is_empty())
        .map(|p| p.group.clone())
        .collect();
    groups.sort();
    groups.dedup();

    Ok(groups)
}


#[tauri::command]
pub async fn auto_scan_projects(app: AppHandle) -> Result<Vec<Project>, String> {
    let settings = load_settings(&app);

    if !settings.auto_scan_on_startup {
        return Ok(Vec::new());
    }

    let scan_dirs = if settings.scan_directories.is_empty() {
        let mut dirs = get_default_scan_dirs();
        if !cfg!(windows) {
            if let Some(home) = std::env::var("HOME").ok() {
                dirs.push(format!("{}/Documents/godot", home));
            }
        }
        dirs
    } else {
        settings.scan_directories
    };

    let mut all_new_projects = Vec::new();
    let storage = get_storage(&app);
    let mut existing_projects: Vec<Project> = storage.load_or_default("projects.json");
    let existing_paths: Vec<String> = existing_projects.iter().map(|p| p.path.clone()).collect();

    for dir in &scan_dirs {
        if !std::path::Path::new(dir).exists() {
            continue;
        }

        match ProjectScanner::scan_directory(dir) {
            Ok(scanned) => {
                for project in scanned {
                    if !existing_paths.contains(&project.path) {
                        existing_projects.push(project.clone());
                        all_new_projects.push(project);
                    }
                }
            }
            Err(_) => continue,
        }
    }

    if !all_new_projects.is_empty() {
        storage.save("projects.json", &existing_projects)
            .map_err(|e| format!("保存项目失败: {}", e))?;

        log_operation(&app, "auto_scan", "", &format!("自动扫描发现 {} 个新项目", all_new_projects.len()));
    }

    let _ = app.emit("scan-complete", &all_new_projects);

    Ok(all_new_projects)
}

#[tauri::command]
pub fn relocate_project(app: AppHandle, project_id: String, new_path: String) -> Result<Project, String> {
    let new_project_path = std::path::Path::new(&new_path);
    if !new_project_path.exists() {
        return Err("指定的新路径不存在".to_string());
    }
    if !new_project_path.join("project.godot").exists() {
        return Err("指定路径不是有效的 Godot 项目".to_string());
    }

    let storage = get_storage(&app);
    let mut projects: Vec<Project> = storage.load_or_default("projects.json");

    let project = projects.iter_mut()
        .find(|p| p.project_id == project_id)
        .ok_or("未找到指定项目".to_string())?;

    let old_path = project.path.clone();
    project.path = new_path.clone();
    project.status = ProjectStatus::Ready;

    let updated_project = project.clone();

    storage.save("projects.json", &projects)
        .map_err(|e| format!("保存项目失败: {}", e))?;

    log_operation(&app, "relocate_project", &project_id,
        &format!("项目路径已从 {} 更新为 {}", old_path, new_path));

    Ok(updated_project)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovedProjectCandidate {
    pub project_id: String,
    pub old_path: String,
    pub old_name: String,
    pub new_path: String,
    pub new_name: String,
}

#[tauri::command]
pub fn detect_moved_projects(app: AppHandle) -> Result<Vec<MovedProjectCandidate>, String> {
    let storage = get_storage(&app);
    let projects: Vec<Project> = storage.load_or_default("projects.json");

    let missing_projects: Vec<&Project> = projects.iter()
        .filter(|p| !std::path::Path::new(&p.path).exists())
        .collect();

    if missing_projects.is_empty() {
        return Ok(Vec::new());
    }

    let settings = load_settings(&app);
    let scan_dirs = if settings.scan_directories.is_empty() {
        get_default_scan_dirs()
    } else {
        settings.scan_directories
    };

    let mut all_scanned = Vec::new();
    for dir in &scan_dirs {
        if std::path::Path::new(dir).exists() {
            if let Ok(scanned) = ProjectScanner::scan_directory(dir) {
                all_scanned.extend(scanned);
            }
        }
    }

    let existing_paths: Vec<String> = projects.iter().map(|p| p.path.clone()).collect();
    let new_projects: Vec<&Project> = all_scanned.iter()
        .filter(|p| !existing_paths.contains(&p.path))
        .collect();

    let mut candidates = Vec::new();

    for missing in &missing_projects {
        for new_proj in &new_projects {
            if missing.name == new_proj.name {
                candidates.push(MovedProjectCandidate {
                    project_id: missing.project_id.clone(),
                    old_path: missing.path.clone(),
                    old_name: missing.name.clone(),
                    new_path: new_proj.path.clone(),
                    new_name: new_proj.name.clone(),
                });
            }
        }
    }

    Ok(candidates)
}

#[tauri::command]
pub fn confirm_project_relocation(app: AppHandle, project_id: String, new_path: String) -> Result<Project, String> {
    relocate_project(app, project_id, new_path)
}

#[tauri::command]
pub fn sync_projects(app: AppHandle) -> Result<Vec<Project>, String> {
    let storage = get_storage(&app);
    let mut projects: Vec<Project> = storage.load_or_default("projects.json");
    let now = chrono::Utc::now();
    let mut synced_count = 0;

    for project in projects.iter_mut() {
        let project_path = std::path::Path::new(&project.path);
        let project_godot = project_path.join("project.godot");

        if !project_path.exists() || !project_godot.exists() {
            project.status = ProjectStatus::MissingSource;
            project.last_synced_at = Some(now);
            synced_count += 1;
            continue;
        }

        match ProjectScanner::parse_project(&project_godot) {
            Ok(scanned) => {
                project.name = scanned.name;
                project.godot_version = scanned.godot_version;
                project.icon_path = scanned.icon_path;
                project.status = scanned.status;
                project.updated_at = now;
                project.last_synced_at = Some(now);
                synced_count += 1;
            }
            Err(_) => {
                project.status = ProjectStatus::Warning;
                project.last_synced_at = Some(now);
                synced_count += 1;
            }
        }
    }

    storage.save("projects.json", &projects)
        .map_err(|e| format!("保存项目列表失败: {}", e))?;

    log_operation(&app, "sync_projects", "",
        &format!("增量同步完成，共同步 {} 个项目", synced_count));

    Ok(projects)
}


#[tauri::command]
pub fn batch_remove_projects(app: AppHandle, project_ids: Vec<String>, delete_files: Option<bool>) -> Result<BatchResult, String> {
    let storage = get_storage(&app);
    let mut projects: Vec<Project> = storage.load_or_default("projects.json");
    let mut success_count = 0;
    let mut failed_count = 0;
    let mut errors = Vec::new();

    let ids_set: std::collections::HashSet<_> = project_ids.iter().cloned().collect();

    let should_delete_files = delete_files.unwrap_or(false);
    let paths_to_delete: Vec<String> = if should_delete_files {
        projects.iter()
            .filter(|p| ids_set.contains(&p.project_id))
            .map(|p| p.path.clone())
            .collect()
    } else {
        Vec::new()
    };

    for project_id in &project_ids {
        if projects.iter().any(|p| p.project_id == *project_id) {
            projects.retain(|p| p.project_id != *project_id);
            success_count += 1;
        } else {
            failed_count += 1;
            errors.push(format!("未找到项目: {}", project_id));
        }
    }

    for project_path in &paths_to_delete {
        let path = std::path::Path::new(project_path);
        if path.exists() {
            if let Err(e) = std::fs::remove_dir_all(path) {
                errors.push(format!("删除项目文件失败 {}: {}", project_path, e));
            }
        }
    }

    let mut bindings: Vec<ProjectBinding> = storage.load_or_default("bindings.json");
    let had_bindings = bindings.iter().any(|b| ids_set.contains(&b.project_id));
    bindings.retain(|b| !ids_set.contains(&b.project_id));
    if had_bindings {
        storage.save("bindings.json", &bindings)
            .map_err(|e| format!("保存绑定列表失败: {}", e))?;
    }

    storage.save("projects.json", &projects)
        .map_err(|e| format!("保存项目列表失败: {}", e))?;

    log_operation(&app, "batch_remove_projects", "",
        &format!("批量删除项目: 成功 {}, 失败 {}{}", success_count, failed_count, if should_delete_files { "（含文件）" } else { "" }));

    Ok(BatchResult { success_count, failed_count, errors })
}


