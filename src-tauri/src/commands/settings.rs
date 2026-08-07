use std::path::{PathBuf, Path};
use std::fs;
use tauri::AppHandle;
use crate::models::*;
use crate::storage::Storage;
use crate::utils::copy_dir_all;
use super::utils::*;

#[tauri::command]
pub fn get_settings(app: AppHandle) -> Result<Settings, String> {
    let settings = load_settings(&app);
    Ok(settings)
}

#[tauri::command]
pub fn save_settings(app: AppHandle, settings: Settings) -> Result<(), String> {
    save_settings_to_config(&app, &settings)?;
    log_operation(&app, "save_settings", "settings.json", "设置已保存");
    Ok(())
}






#[tauri::command]
pub fn check_auto_setup_needed(app: AppHandle) -> Result<bool, String> {
    let storage = get_storage(&app);
    let settings = load_settings(&app);
    let current_hash = compute_settings_hash(&settings);

    let state: Option<AutoSetupState> = storage.load("auto_setup_state.json").ok();

    match state {
        Some(s) => {
            let now = chrono::Utc::now().timestamp();
            let elapsed_hours = (now - s.completed_at) / 3600;
            if s.settings_hash == current_hash && elapsed_hours < 24 {
                Ok(false)
            } else {
                Ok(true)
            }
        }
        None => Ok(true),
    }
}

#[tauri::command]
pub fn mark_auto_setup_done(app: AppHandle) -> Result<(), String> {
    let storage = get_storage(&app);
    let settings = load_settings(&app);
    let state = AutoSetupState {
        completed_at: chrono::Utc::now().timestamp(),
        settings_hash: compute_settings_hash(&settings),
    };
    storage.save("auto_setup_state.json", &state)
        .map_err(|e| format!("保存自动设置状态失败: {}", e))
}


#[tauri::command]
pub fn check_data_dir_setup_needed(app: AppHandle) -> Result<bool, String> {
    let settings = load_settings(&app);
    Ok(!settings.data_dir_initialized)
}

#[tauri::command]
pub fn confirm_data_dir(app: AppHandle, custom_dir: Option<String>) -> Result<String, String> {
    let config_dir = get_config_dir(&app);
    let config_storage = Storage::new(config_dir.clone());
    let mut settings: Settings = config_storage.load_or_default("settings.json");

    let data_dir = if let Some(dir) = custom_dir {
        if !dir.is_empty() {
            let path = PathBuf::from(&dir);
            std::fs::create_dir_all(&path)
                .map_err(|e| format!("创建数据目录失败: {}", e))?;
            settings.custom_data_dir = dir.clone();
            dir
        } else {
            settings.custom_data_dir = String::new();
            config_dir.to_string_lossy().to_string()
        }
    } else {
        let root = super::utils::get_app_root_dir();
        let fallback = root.join("GodotHarborData");
        std::fs::create_dir_all(&fallback)
            .map_err(|e| format!("创建数据目录失败: {}", e))?;
        settings.custom_data_dir = fallback.to_string_lossy().to_string();
        fallback.to_string_lossy().to_string()
    };

    settings.data_dir_initialized = true;
    save_settings_to_config(&app, &settings)?;

    Ok(data_dir)
}

#[tauri::command]
pub fn migrate_data_dir(app: AppHandle, new_data_dir: String) -> Result<(), String> {
    let new_path = Path::new(&new_data_dir);
    if new_path.exists() && !new_path.is_dir() {
        return Err("目标路径已存在但不是目录".to_string());
    }

    let old_data_dir = get_data_dir(&app);
    let old_str = old_data_dir.to_string_lossy().to_string();
    if old_str == new_data_dir {
        return Err("新目录与当前目录相同".to_string());
    }

    std::fs::create_dir_all(new_path)
        .map_err(|e| format!("创建目录失败: {}", e))?;

    let exclude_names: &[&str] = &["settings.json"];
    for entry in fs::read_dir(&old_data_dir)
        .map_err(|e| format!("读取源目录失败: {}", e))?
    {
        let entry = entry.map_err(|e| format!("读取目录条目失败: {}", e))?;
        let file_name = entry.file_name();
        let name_str = file_name.to_string_lossy();
        if exclude_names.iter().any(|ex| *ex == name_str) {
            continue;
        }
        let ty = entry.file_type().map_err(|e| format!("获取文件类型失败: {}", e))?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &new_path.join(&file_name))?;
        } else {
            fs::copy(&entry.path(), &new_path.join(&file_name))
                .map_err(|e| format!("复制 {} 失败: {}", name_str, e))?;
        }
    }

    let mut settings = load_settings(&app);
    settings.custom_data_dir = new_data_dir.clone();
    save_settings_to_config(&app, &settings)?;

    let new_data_dir_path = PathBuf::from(&new_data_dir);
    let new_engines_json = new_data_dir_path.join("engines.json");
    if new_engines_json.exists() {
        let new_storage = Storage::new(new_data_dir_path.clone());
        let mut engines: Vec<Engine> = new_storage.load_or_default("engines.json");
        let mut changed = false;
        for engine in &mut engines {
            if engine.path.starts_with(&old_str) {
                engine.path = engine.path.replacen(&old_str, &new_data_dir, 1);
                changed = true;
            }
        }
        if changed {
            let _ = new_storage.save("engines.json", &engines);
        }
    }

    let new_projects_json = new_data_dir_path.join("projects.json");
    if new_projects_json.exists() {
        let new_storage = Storage::new(new_data_dir_path.clone());
        let mut projects: Vec<Project> = new_storage.load_or_default("projects.json");
        let mut changed = false;
        for project in &mut projects {
            if project.path.starts_with(&old_str) {
                project.path = project.path.replacen(&old_str, &new_data_dir, 1);
                changed = true;
            }
        }
        if changed {
            let _ = new_storage.save("projects.json", &projects);
        }
    }

    let new_plugins_json = new_data_dir_path.join("plugins.json");
    if new_plugins_json.exists() {
        let new_storage = Storage::new(new_data_dir_path.clone());
        let mut plugins: Vec<Plugin> = new_storage.load_or_default("plugins.json");
        let mut changed = false;
        for plugin in &mut plugins {
            for version in &mut plugin.versions {
                if version.path.starts_with(&old_str) {
                    version.path = version.path.replacen(&old_str, &new_data_dir, 1);
                    changed = true;
                }
                for unit in &mut version.units {
                    if unit.plugin_cfg_path.starts_with(&old_str) {
                        unit.plugin_cfg_path = unit.plugin_cfg_path.replacen(&old_str, &new_data_dir, 1);
                        changed = true;
                    }
                }
            }
        }
        if changed {
            let _ = new_storage.save("plugins.json", &plugins);
        }
    }

    for entry in fs::read_dir(&old_data_dir)
        .map_err(|e| format!("读取源目录失败: {}", e))?
    {
        let entry = entry.map_err(|e| format!("读取目录条目失败: {}", e))?;
        let file_name = entry.file_name();
        let name_str = file_name.to_string_lossy();
        if exclude_names.iter().any(|ex| *ex == name_str) {
            continue;
        }
        let ty = entry.file_type().map_err(|e| format!("获取文件类型失败: {}", e))?;
        if ty.is_dir() {
            std::fs::remove_dir_all(entry.path())
                .map_err(|e| format!("删除目录 {} 失败: {}", name_str, e))?;
        } else {
            std::fs::remove_file(entry.path())
                .map_err(|e| format!("删除文件 {} 失败: {}", name_str, e))?;
        }
    }

    log_operation(&app, "migrate_data_dir", &new_data_dir,
        &format!("数据目录已迁移: {} -> {}", old_str, new_data_dir));
    Ok(())
}
#[tauri::command]
pub fn get_storage_paths(app: AppHandle) -> Result<StoragePaths, String> {
    let config_dir = get_config_dir(&app);
    let data_dir = get_data_dir(&app);
    let plugins_dir = data_dir.join("plugins");
    Ok(StoragePaths {
        app_data_dir: data_dir.to_string_lossy().to_string(),
        plugins_dir: plugins_dir.to_string_lossy().to_string(),
        engines_dir: data_dir.join("engines").to_string_lossy().to_string(),
        cache_dir: data_dir.join("cache").to_string_lossy().to_string(),
        logs_dir: data_dir.join("logs").to_string_lossy().to_string(),
        hot_updates_dir: data_dir.join("hot_updates").to_string_lossy().to_string(),
        settings_file: config_dir.join("settings.json").to_string_lossy().to_string(),
        projects_file: data_dir.join("projects.json").to_string_lossy().to_string(),
        engines_file: data_dir.join("engines.json").to_string_lossy().to_string(),
    })
}

