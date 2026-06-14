use serde::{Serialize, Deserialize};
use tauri::{AppHandle, Emitter};
use crate::models::*;
use crate::scanner::ProjectScanner;
use super::utils::*;
use super::system::get_default_scan_dirs;

const GROUPS_FILE: &str = "groups.json";

fn load_groups(app: &AppHandle) -> Vec<ProjectGroup> {
    let storage = get_storage(app);
    storage.load_or_default(GROUPS_FILE)
}

fn save_groups(app: &AppHandle, groups: &Vec<ProjectGroup>) -> Result<(), String> {
    let storage = get_storage(app);
    storage.save(GROUPS_FILE, groups)
        .map_err(|e| format!("保存分组失败: {}", e))
}

pub fn normalize_path(path: &str) -> String {
    let p = std::path::Path::new(path);
    match std::fs::canonicalize(p) {
        Ok(canonical) => {
            let s = canonical.to_string_lossy().to_string();
            if cfg!(windows) {
                s.trim_start_matches(r"\\?\").to_string()
            } else {
                s
            }
        }
        Err(_) => {
            if cfg!(windows) {
                path.to_lowercase().replace('/', "\\")
            } else {
                path.to_string()
            }
        }
    }
}

fn path_matches(a: &str, b: &str) -> bool {
    if cfg!(windows) {
        normalize_path(a) == normalize_path(b)
    } else {
        a == b
    }
}

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
        let normalized_path = normalize_path(&project.path);
        if let Some(index) = existing_projects.iter().position(|p| path_matches(&p.path, &normalized_path)) {
            let mut existing = existing_projects[index].clone();
            existing.name = project.name.clone();
            existing.godot_version = project.godot_version.clone();
            existing_projects[index] = existing;
        } else {
            let mut normalized = project.clone();
            normalized.path = normalized_path;
            existing_projects.push(normalized);
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

    let original_len = projects.len();
    let mut seen = Vec::new();
    let mut deduped = Vec::new();
    for project in projects {
        let norm = normalize_path(&project.path);
        if !seen.contains(&norm) {
            seen.push(norm);
            deduped.push(project);
        }
    }
    if deduped.len() < original_len {
        let _ = storage.save("projects.json", &deduped);
    }

    Ok(deduped)
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

    validate_project_path(&app, project_path)?;

    let mut project = ProjectScanner::parse_project(&project_godot)
        .map_err(|e| format!("解析项目失败: {}", e))?;

    project.path = normalize_path(&project.path);

    let storage = get_storage(&app);
    let mut projects: Vec<Project> = storage.load_or_default("projects.json");

    if projects.iter().any(|p| path_matches(&p.path, &project.path)) {
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
            if projects.iter().any(|p| path_matches(&p.path, &existing_path)) {
                return Err("该项目已存在，请勿重复添加".to_string());
            }
            let project = ProjectScanner::parse_project(&project_godot)
                .map_err(|e| format!("解析项目失败: {}", e))?;
            let project_name = project.name.clone();
            let mut all_projects: Vec<Project> = storage.load_or_default("projects.json");
            let mut normalized_project = project.clone();
            normalized_project.path = normalize_path(&project.path);
            all_projects.push(normalized_project);
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

    let mut normalized_project = project.clone();
    normalized_project.path = normalize_path(&project.path);

    if all_projects.iter().any(|p| path_matches(&p.path, &normalized_project.path)) {
        let _ = std::fs::remove_dir_all(&clone_target);
        return Err("该项目已存在，请勿重复添加".to_string());
    }

    all_projects.push(normalized_project);
    storage.save("projects.json", &all_projects)
        .map_err(|e| format!("保存项目失败: {}", e))?;

    log_operation(&app, "import_project_from_git", &git_url,
        &format!("从 Git 导入项目: {}", project_name));

    let _ = app.emit("projects-changed", ());

    Ok(project)
}

#[tauri::command]
pub async fn remove_project(app: AppHandle, project_id: String, delete_files: Option<bool>) -> Result<(), String> {
    let storage = get_storage(&app);
    let mut projects: Vec<Project> = storage.load_or_default("projects.json");

    let project = projects.iter().find(|p| p.project_id == project_id)
        .ok_or("未找到指定项目".to_string())?;
    let project_name = project.name.clone();
    let project_path = project.path.clone();

    if delete_files.unwrap_or(false) {
        let path = std::path::PathBuf::from(&project_path);
        tokio::task::spawn_blocking(move || {
            if path.exists() {
                std::fs::remove_dir_all(&path)
                    .map_err(|e| format!("删除项目文件失败: {}", e))
            } else {
                Ok(())
            }
        }).await.map_err(|e| format!("删除任务失败: {}", e))??;
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
pub fn update_project_group(app: AppHandle, project_id: String, group_id: String) -> Result<(), String> {
    let storage = get_storage(&app);
    let mut projects: Vec<Project> = storage.load_or_default("projects.json");

    let project = projects.iter_mut()
        .find(|p| p.project_id == project_id)
        .ok_or("未找到指定项目".to_string())?;

    project.group = group_id.clone();

    storage.save("projects.json", &projects)
        .map_err(|e| format!("保存项目分组失败: {}", e))?;

    log_operation(&app, "update_project_group", &project_id, &format!("项目分组已更新: {}", group_id));
    Ok(())
}

#[tauri::command]
pub fn get_project_groups(app: AppHandle) -> Result<Vec<ProjectGroup>, String> {
    let groups = load_groups(&app);
    Ok(groups)
}

#[tauri::command]
pub fn create_project_group(
    app: AppHandle,
    name: String,
    icon: Option<String>,
    color: Option<String>,
    description: Option<String>,
) -> Result<ProjectGroup, String> {
    if name.trim().is_empty() {
        return Err("分组名称不能为空".to_string());
    }

    let group = ProjectGroup::new(
        name,
        icon.unwrap_or_default(),
        color.unwrap_or_default(),
        description.unwrap_or_default(),
    );

    let mut groups = load_groups(&app);
    groups.push(group.clone());
    save_groups(&app, &groups)?;

    log_operation(&app, "create_project_group", &group.group_id,
        &format!("创建分组: {}", group.name));

    Ok(group)
}

#[tauri::command]
pub fn update_project_group_info(app: AppHandle, group: ProjectGroup) -> Result<(), String> {
    if group.name.trim().is_empty() {
        return Err("分组名称不能为空".to_string());
    }

    let mut groups = load_groups(&app);
    let idx = groups.iter().position(|g| g.group_id == group.group_id)
        .ok_or("未找到指定分组".to_string())?;

    let mut updated = group;
    updated.updated_at = chrono::Utc::now();
    groups[idx] = updated.clone();
    save_groups(&app, &groups)?;

    log_operation(&app, "update_project_group_info", &updated.group_id,
        &format!("更新分组: {}", updated.name));

    Ok(())
}

#[tauri::command]
pub fn delete_project_group(app: AppHandle, group_id: String) -> Result<(), String> {
    let mut groups = load_groups(&app);
    let group = groups.iter().find(|g| g.group_id == group_id)
        .ok_or("未找到指定分组".to_string())?;
    let name = group.name.clone();

    groups.retain(|g| g.group_id != group_id);
    save_groups(&app, &groups)?;

    // Ungroup projects in this group
    let storage = get_storage(&app);
    let mut projects: Vec<Project> = storage.load_or_default("projects.json");
    let mut changed = false;
    for project in projects.iter_mut() {
        if project.group == group_id {
            project.group = String::new();
            changed = true;
        }
    }
    if changed {
        storage.save("projects.json", &projects)
            .map_err(|e| format!("保存项目列表失败: {}", e))?;
    }

    log_operation(&app, "delete_project_group", &group_id,
        &format!("删除分组: {}", name));

    Ok(())
}

#[tauri::command]
pub fn batch_set_project_group(app: AppHandle, project_ids: Vec<String>, group_id: String) -> Result<(), String> {
    let storage = get_storage(&app);
    let mut projects: Vec<Project> = storage.load_or_default("projects.json");

    let mut count = 0;
    for project in projects.iter_mut() {
        if project_ids.contains(&project.project_id) && project.group != group_id {
            project.group = group_id.clone();
            count += 1;
        }
    }

    storage.save("projects.json", &projects)
        .map_err(|e| format!("保存项目分组失败: {}", e))?;

    log_operation(&app, "batch_set_project_group", &group_id,
        &format!("批量设置分组: {} 个项目", count));

    Ok(())
}

/// Migrate workspaces.json → groups.json and old-style text groups → ProjectGroup entries
pub fn migrate_groups(app: &AppHandle) {
    let storage = get_storage(app);

    // 1. Migrate workspaces.json → groups.json
    if storage.exists("workspaces.json") {
        #[derive(serde::Deserialize)]
        struct OldWorkspace {
            workspace_id: String,
            name: String,
            #[serde(default)]
            icon: String,
            #[serde(default)]
            color: String,
            #[serde(default)]
            project_ids: Vec<String>,
        }

        if let Ok(workspaces) = storage.load::<Vec<OldWorkspace>>("workspaces.json") {
            let mut groups: Vec<ProjectGroup> = load_groups(app);
            let mut projects: Vec<Project> = storage.load_or_default("projects.json");
            let mut changed = false;

            for ws in &workspaces {
                // Create a ProjectGroup from the workspace
                let group = ProjectGroup::new(
                    ws.name.clone(),
                    ws.icon.clone(),
                    if ws.color.is_empty() { "#3B82F6".to_string() } else { ws.color.clone() },
                    String::new(),
                );

                // Update projects that were in this workspace to reference the new group_id
                for pid in &ws.project_ids {
                    if let Some(project) = projects.iter_mut().find(|p| p.project_id == *pid) {
                        project.group = group.group_id.clone();
                        changed = true;
                    }
                }

                groups.push(group);
            }

            if let Err(e) = save_groups(app, &groups) {
                eprintln!("迁移工作区到分组失败: {}", e);
            }

            if changed {
                if let Err(e) = storage.save("projects.json", &projects) {
                    eprintln!("迁移项目分组引用失败: {}", e);
                }
            }

            // Delete workspaces.json after successful migration
            let ws_path = get_data_dir(app).join("workspaces.json");
            let _ = std::fs::remove_file(ws_path);

            eprintln!("工作区迁移完成: {} 个工作区已转为分组", workspaces.len());
        }
    }

    // 2. Migrate old-style text groups → ProjectGroup entries
    // If groups.json doesn't exist yet but projects have text group values
    let groups: Vec<ProjectGroup> = load_groups(app);
    if groups.is_empty() {
        let projects: Vec<Project> = storage.load_or_default("projects.json");
        let text_groups: Vec<String> = projects.iter()
            .filter(|p| !p.group.is_empty())
            .map(|p| p.group.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        if !text_groups.is_empty() {
            let mut new_groups: Vec<ProjectGroup> = Vec::new();
            let mut projects = projects;
            let mut changed = false;

            for group_name in &text_groups {
                let group = ProjectGroup::new(
                    group_name.clone(),
                    String::new(),
                    String::new(),
                    String::new(),
                );

                // Update projects referencing this text group name to use group_id
                for project in projects.iter_mut() {
                    if project.group == *group_name {
                        project.group = group.group_id.clone();
                        changed = true;
                    }
                }

                new_groups.push(group);
            }

            if let Err(e) = save_groups(app, &new_groups) {
                eprintln!("迁移文本分组失败: {}", e);
            }

            if changed {
                if let Err(e) = storage.save("projects.json", &projects) {
                    eprintln!("更新项目分组引用失败: {}", e);
                }
            }

            eprintln!("文本分组迁移完成: {} 个分组已创建", new_groups.len());
        }
    }
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

    for dir in &scan_dirs {
        if !std::path::Path::new(dir).exists() {
            continue;
        }

        match ProjectScanner::scan_directory(dir) {
            Ok(scanned) => {
                for project in scanned {
                    let mut normalized = project.clone();
                    normalized.path = normalize_path(&project.path);
                    if !existing_projects.iter().any(|p| path_matches(&p.path, &normalized.path)) {
                        existing_projects.push(normalized);
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
pub async fn detect_moved_projects(app: AppHandle) -> Result<Vec<MovedProjectCandidate>, String> {
    let app_clone = app.clone();
    tokio::task::spawn_blocking(move || {
        let storage = get_storage(&app_clone);
        let projects: Vec<Project> = storage.load_or_default("projects.json");

        let missing_projects: Vec<&Project> = projects.iter()
            .filter(|p| !std::path::Path::new(&p.path).exists())
            .collect();

        if missing_projects.is_empty() {
            return Ok(Vec::new());
        }

        let settings = load_settings(&app_clone);
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

        let new_projects: Vec<&Project> = all_scanned.iter()
            .filter(|p| !projects.iter().any(|ep| path_matches(&ep.path, &p.path)))
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
    }).await.map_err(|e| format!("任务执行失败: {}", e))?
}

#[tauri::command]
pub fn confirm_project_relocation(app: AppHandle, project_id: String, new_path: String) -> Result<Project, String> {
    relocate_project(app, project_id, new_path)
}

#[tauri::command]
pub async fn sync_projects(app: AppHandle) -> Result<Vec<Project>, String> {
    let app_clone = app.clone();
    tokio::task::spawn_blocking(move || {
        let storage = get_storage(&app_clone);
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

        log_operation(&app_clone, "sync_projects", "",
            &format!("增量同步完成，共同步 {} 个项目", synced_count));

        Ok(projects)
    }).await.map_err(|e| format!("任务执行失败: {}", e))?
}


#[tauri::command]
pub async fn batch_remove_projects(app: AppHandle, project_ids: Vec<String>, delete_files: Option<bool>) -> Result<BatchResult, String> {
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

    if should_delete_files && !paths_to_delete.is_empty() {
        let paths = paths_to_delete.clone();
        let delete_errors: Vec<String> = tokio::task::spawn_blocking(move || {
            let mut errs = Vec::new();
            for project_path in &paths {
                let path = std::path::Path::new(project_path);
                if path.exists() {
                    if let Err(e) = std::fs::remove_dir_all(path) {
                        errs.push(format!("删除项目文件失败 {}: {}", project_path, e));
                    }
                }
            }
            errs
        }).await.map_err(|e| format!("删除任务失败: {}", e))?;
        errors.extend(delete_errors);
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


