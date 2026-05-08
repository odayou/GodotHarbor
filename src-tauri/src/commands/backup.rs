use tauri::AppHandle;
use crate::models::*;
use crate::utils::copy_dir_all;
use super::utils::*;

#[tauri::command]
pub fn list_addon_backups(app: AppHandle, project_id: String) -> Result<Vec<AddonBackupInfo>, String> {
    let storage = get_storage(&app);
    let projects: Vec<Project> = storage.load_or_default("projects.json");
    let project = projects.iter()
        .find(|p| p.project_id == project_id)
        .ok_or_else(|| "未找到指定项目".to_string())?;

    let data_dir = get_data_dir(&app);
    let backup_dir = data_dir.join("backups").join(&project.name);

    if !backup_dir.exists() {
        return Ok(Vec::new());
    }

    let mut backups = Vec::new();
    let entries = std::fs::read_dir(&backup_dir)
        .map_err(|e| format!("读取备份目录失败: {}", e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().map(|e| e == "zip").unwrap_or(false) {
            let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
            if !file_name.starts_with("addons_backup_") {
                continue;
            }
            let file_size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            let created_at = file_name
                .strip_prefix("addons_backup_")
                .and_then(|s| s.strip_suffix(".zip"))
                .unwrap_or("unknown")
                .replace("_", " ");
            backups.push(AddonBackupInfo {
                file_name,
                file_path: path.to_string_lossy().to_string(),
                file_size,
                created_at,
            });
        }
    }

    backups.sort_by(|a, b| b.file_name.cmp(&a.file_name));
    Ok(backups)
}

#[tauri::command]
pub fn restore_addon_backup(app: AppHandle, project_id: String, backup_file: String) -> Result<(), String> {
    let storage = get_storage(&app);
    let projects: Vec<Project> = storage.load_or_default("projects.json");
    let project = projects.iter()
        .find(|p| p.project_id == project_id)
        .ok_or_else(|| "未找到指定项目".to_string())?;

    let addons_dir = std::path::Path::new(&project.path).join("addons");
    let backup_path = std::path::Path::new(&backup_file);

    if !backup_path.exists() {
        return Err("备份文件不存在".to_string());
    }

    if addons_dir.exists() {
        std::fs::remove_dir_all(&addons_dir)
            .map_err(|e| format!("删除当前 addons 目录失败: {}", e))?;
    }

    let file = std::fs::File::open(backup_path)
        .map_err(|e| format!("打开备份文件失败: {}", e))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| format!("读取备份文件失败: {}", e))?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)
            .map_err(|e| format!("读取压缩条目失败: {}", e))?;
        let out_path = addons_dir.join(entry.name());
        if entry.is_dir() {
            std::fs::create_dir_all(&out_path)
                .map_err(|e| format!("创建目录失败: {}", e))?;
        } else {
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("创建父目录失败: {}", e))?;
            }
            let mut out_file = std::fs::File::create(&out_path)
                .map_err(|e| format!("创建文件失败: {}", e))?;
            std::io::copy(&mut entry, &mut out_file)
                .map_err(|e| format!("写入文件失败: {}", e))?;
        }
    }

    log_operation(&app, "restore_addon_backup", &project_id,
        &format!("从备份恢复 addons: {}", backup_path.to_string_lossy()));

    Ok(())
}

#[tauri::command]
pub fn save_as_template(app: AppHandle, project_id: String, template_name: String) -> Result<ProjectTemplate, String> {
    let storage = get_storage(&app);
    let projects: Vec<Project> = storage.load_or_default("projects.json");
    let _project = projects.iter()
        .find(|p| p.project_id == project_id)
        .ok_or_else(|| "未找到指定项目".to_string())?;

    let bindings: Vec<ProjectBinding> = storage.load_or_default("bindings.json");
    let project_bindings: Vec<&ProjectBinding> = bindings.iter()
        .filter(|b| b.project_id == project_id)
        .collect();

    let plugins: Vec<Plugin> = storage.load_or_default("plugins.json");

    let template_bindings: Vec<TemplateBinding> = project_bindings.iter().map(|b| {
        let plugin = plugins.iter().find(|p| p.plugin_id == b.plugin_id);
        let version = plugin.and_then(|p| p.versions.iter().find(|v| v.version_id == b.version_id));
        let unit = version.and_then(|v| v.units.iter().find(|u| u.unit_id == b.unit_id));
        TemplateBinding {
            plugin_id: b.plugin_id.clone(),
            plugin_name: plugin.map(|p| p.name.clone()).unwrap_or_default(),
            version_id: b.version_id.clone(),
            unit_id: b.unit_id.clone(),
            unit_name: unit.map(|u| u.name.clone()).unwrap_or_default(),
            mount_path: b.mount_path.clone(),
            subdirectory: b.subdirectory.clone(),
        }
    }).collect();

    let template = ProjectTemplate {
        template_id: uuid::Uuid::new_v4().to_string(),
        name: template_name.clone(),
        bindings: template_bindings,
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    let data_dir = get_data_dir(&app);
    let templates_file = data_dir.join("templates.json");
    let mut templates: Vec<ProjectTemplate> = if templates_file.exists() {
        let file = std::fs::read_to_string(&templates_file)
            .map_err(|e| format!("读取模板文件失败: {}", e))?;
        serde_json::from_str(&file).unwrap_or_default()
    } else {
        Vec::new()
    };
    templates.push(template.clone());
    let json = serde_json::to_string_pretty(&templates)
        .map_err(|e| format!("序列化模板失败: {}", e))?;
    std::fs::write(&templates_file, json)
        .map_err(|e| format!("保存模板文件失败: {}", e))?;

    log_operation(&app, "save_as_template", &project_id,
        &format!("保存为模板: {} ({} 个绑定)", template_name, template.bindings.len()));

    Ok(template)
}

#[tauri::command]
pub fn list_templates(app: AppHandle) -> Result<Vec<ProjectTemplate>, String> {
    let data_dir = get_data_dir(&app);
    let templates_file = data_dir.join("templates.json");
    if !templates_file.exists() {
        return Ok(Vec::new());
    }
    let file = std::fs::read_to_string(&templates_file)
        .map_err(|e| format!("读取模板文件失败: {}", e))?;
    let templates: Vec<ProjectTemplate> = serde_json::from_str(&file).unwrap_or_default();
    Ok(templates)
}

#[tauri::command]
pub fn delete_template(app: AppHandle, template_id: String) -> Result<(), String> {
    let data_dir = get_data_dir(&app);
    let templates_file = data_dir.join("templates.json");
    if !templates_file.exists() {
        return Ok(());
    }
    let file = std::fs::read_to_string(&templates_file)
        .map_err(|e| format!("读取模板文件失败: {}", e))?;
    let mut templates: Vec<ProjectTemplate> = serde_json::from_str(&file).unwrap_or_default();
    templates.retain(|t| t.template_id != template_id);
    let json = serde_json::to_string_pretty(&templates)
        .map_err(|e| format!("序列化模板失败: {}", e))?;
    std::fs::write(&templates_file, json)
        .map_err(|e| format!("保存模板文件失败: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn apply_template_to_project(app: AppHandle, project_id: String, template_id: String) -> Result<ApplyResult, String> {
    let data_dir = get_data_dir(&app);
    let templates_file = data_dir.join("templates.json");
    let file = std::fs::read_to_string(&templates_file)
        .map_err(|e| format!("读取模板文件失败: {}", e))?;
    let templates: Vec<ProjectTemplate> = serde_json::from_str(&file).unwrap_or_default();
    let template = templates.iter()
        .find(|t| t.template_id == template_id)
        .ok_or_else(|| "未找到指定模板".to_string())?;

    let storage = get_storage(&app);
    let mut bindings: Vec<ProjectBinding> = storage.load_or_default("bindings.json");

    for tb in &template.bindings {
        if let Some(existing) = bindings.iter_mut().find(|b| b.project_id == project_id && b.plugin_id == tb.plugin_id) {
            existing.version_id = tb.version_id.clone();
            existing.unit_id = tb.unit_id.clone();
            existing.mount_path = tb.mount_path.clone();
            existing.subdirectory = tb.subdirectory.clone();
        } else {
            bindings.push(ProjectBinding {
                project_id: project_id.clone(),
                plugin_id: tb.plugin_id.clone(),
                version_id: tb.version_id.clone(),
                unit_id: tb.unit_id.clone(),
                mount_path: tb.mount_path.clone(),
                is_healthy: Some(true),
                subdirectory: tb.subdirectory.clone(),
                created_at: chrono::Utc::now(),
            });
        }
    }

    let json = serde_json::to_string_pretty(&bindings)
        .map_err(|e| format!("序列化绑定失败: {}", e))?;
    let data_dir = get_data_dir(&app);
    std::fs::write(data_dir.join("bindings.json"), json)
        .map_err(|e| format!("保存绑定文件失败: {}", e))?;

    super::plugin::apply_changes(app, project_id)
}


#[tauri::command]
pub fn backup_data(app: AppHandle, backup_path: String) -> Result<String, String> {
    let data_dir = get_data_dir(&app);
    let storage = get_storage(&app);
    
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let base_backup_dir = std::path::Path::new(&backup_path).join(format!("backup_{}", timestamp));
    
    let mut backup_dir = base_backup_dir.clone();
    let mut counter = 1;
    while backup_dir.exists() {
        backup_dir = std::path::Path::new(&backup_path).join(format!("backup_{}_{}", timestamp, counter));
        counter += 1;
    }
    
    std::fs::create_dir_all(&backup_dir)
        .map_err(|e| format!("创建备份目录失败: {}", e))?;

    let files = DATA_FILES;
    let mut backup_files = Vec::new();

    for filename in files {
        let src = data_dir.join(filename);
        if src.exists() {
            let dst = backup_dir.join(filename);
            std::fs::copy(&src, &dst)
                .map_err(|e| format!("备份文件 {} 失败: {}", filename, e))?;
            backup_files.push(filename.to_string());
        }
    }

    let plugins_src_dir = data_dir.join("plugins");
    let plugins_dst_dir = backup_dir.join("plugins");
    
    if plugins_src_dir.exists() {
        copy_dir_all(&plugins_src_dir, &plugins_dst_dir)
            .map_err(|e| format!("备份插件目录失败: {}", e))?;
    }

    let projects: Vec<Project> = storage.load_or_default("projects.json");
    let plugins: Vec<Plugin> = storage.load_or_default("plugins.json");
    let bindings: Vec<ProjectBinding> = storage.load_or_default("bindings.json");

    let backup_info = serde_json::json!({
        "version": "1.0",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "app_version": env!("CARGO_PKG_VERSION"),
        "files": backup_files,
        "project_count": projects.len(),
        "plugin_count": plugins.len(),
        "binding_count": bindings.len()
    });

    std::fs::write(backup_dir.join("backup_info.json"), serde_json::to_string_pretty(&backup_info).unwrap())
        .map_err(|e| format!("创建备份信息文件失败: {}", e))?;

    log_operation(&app, "backup_data", &backup_path, &format!("数据备份成功，共备份 {} 个文件", backup_files.len()));
    Ok(format!("备份成功，备份位置: {}", backup_dir.display()))
}

#[tauri::command]
pub fn restore_data(app: AppHandle, backup_path: String) -> Result<String, String> {
    let data_dir = get_data_dir(&app);
    let backup_dir = std::path::Path::new(&backup_path);

    if !backup_dir.exists() {
        return Err("备份目录不存在".to_string());
    }

    let backup_info_path = backup_dir.join("backup_info.json");
    if !backup_info_path.exists() {
        return Err("无效的备份目录，缺少 backup_info.json".to_string());
    }

    let pre_restore_backup_dir = data_dir.join("restore_backup");
    if let Err(e) = std::fs::create_dir_all(&pre_restore_backup_dir) {
        eprintln!("Failed to create pre-restore backup dir: {}", e);
    } else {
        let files = DATA_FILES;
        for filename in files {
            let src = data_dir.join(filename);
            if src.exists() {
                let dst = pre_restore_backup_dir.join(filename);
                let _ = std::fs::copy(&src, &dst);
            }
        }
        let plugins_src = data_dir.join("plugins");
        if plugins_src.exists() {
            let plugins_dst = pre_restore_backup_dir.join("plugins");
            let _ = copy_dir_all(&plugins_src, &plugins_dst);
        }
    }

    let files = DATA_FILES;
    let mut restore_info = Vec::new();

    for filename in files {
        let src = backup_dir.join(filename);
        if src.exists() {
            let dst = data_dir.join(filename);
            std::fs::copy(&src, &dst)
                .map_err(|e| format!("恢复文件 {} 失败: {}", filename, e))?;
            restore_info.push(filename.to_string());
        }
    }

    let plugins_dst_dir = data_dir.join("plugins");
    let plugins_src_dir = backup_dir.join("plugins");
    
    if plugins_src_dir.exists() {
        if plugins_dst_dir.exists() {
            std::fs::remove_dir_all(&plugins_dst_dir)
                .map_err(|e| format!("删除现有插件目录失败: {}", e))?;
        }
        copy_dir_all(&plugins_src_dir, &plugins_dst_dir)
            .map_err(|e| format!("恢复插件目录失败: {}", e))?;
        restore_info.push("plugins/".to_string());
    }

    log_operation(&app, "restore_data", &backup_path, &format!("数据恢复成功，共恢复 {} 个项目", restore_info.len()));
    Ok(format!("恢复成功，共恢复 {} 个项目", restore_info.len()))
}

#[tauri::command]
pub fn reset_data(app: AppHandle, backup_path: String) -> Result<String, String> {
    let data_dir = get_data_dir(&app);
    let storage = get_storage(&app);
    
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let backup_dir = std::path::Path::new(&backup_path).join(format!("backup_{}", timestamp));
    
    std::fs::create_dir_all(&backup_dir)
        .map_err(|e| format!("创建备份目录失败: {}", e))?;

    let files = DATA_FILES;
    let mut backup_files = Vec::new();

    for filename in files {
        let src = data_dir.join(filename);
        if src.exists() {
            let dst = backup_dir.join(filename);
            std::fs::copy(&src, &dst)
                .map_err(|e| format!("备份文件 {} 失败: {}", filename, e))?;
            backup_files.push(filename.to_string());
        }
    }

    let plugins_src_dir = data_dir.join("plugins");
    let plugins_dst_dir = backup_dir.join("plugins");
    
    if plugins_src_dir.exists() {
        copy_dir_all(&plugins_src_dir, &plugins_dst_dir)
            .map_err(|e| format!("备份插件目录失败: {}", e))?;
    }

    let projects: Vec<Project> = storage.load_or_default("projects.json");
    let plugins: Vec<Plugin> = storage.load_or_default("plugins.json");
    let bindings: Vec<ProjectBinding> = storage.load_or_default("bindings.json");

    let backup_info = serde_json::json!({
        "version": "1.0",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "app_version": env!("CARGO_PKG_VERSION"),
        "files": backup_files,
        "project_count": projects.len(),
        "plugin_count": plugins.len(),
        "binding_count": bindings.len()
    });

    std::fs::write(backup_dir.join("backup_info.json"), serde_json::to_string_pretty(&backup_info).unwrap())
        .map_err(|e| format!("创建备份信息文件失败: {}", e))?;

    log_operation(&app, "backup_data", &backup_path, &format!("数据备份成功，共备份 {} 个文件", backup_files.len()));

    let files_to_delete = DATA_FILES;

    for filename in files_to_delete {
        let file_path = data_dir.join(filename);
        if file_path.exists() {
            std::fs::remove_file(&file_path)
                .map_err(|e| format!("删除文件 {} 失败: {}", filename, e))?;
        }
    }

    if plugins_src_dir.exists() {
        std::fs::remove_dir_all(&plugins_src_dir)
            .map_err(|e| format!("删除插件目录失败: {}", e))?;
    }

    log_operation(&app, "reset_data", backup_dir.to_str().unwrap_or(""), "数据重置成功");
    Ok(format!("数据重置成功！\n\n备份已保存至: {}\n\n请重启应用", backup_dir.display()))
}


