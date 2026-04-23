use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use std::sync::Mutex;
use crate::models::*;
use crate::storage::Storage;
use crate::scanner::ProjectScanner;
use crate::plugin_manager::PluginManager;
use crate::linker::Linker;
use std::collections::HashMap;

struct AppState {
    settings: Mutex<Settings>,
    projects: Mutex<Vec<Project>>,
    plugins: Mutex<Vec<Plugin>>,
    bindings: Mutex<Vec<ProjectBinding>>,
}

impl AppState {
    fn new() -> Self {
        Self {
            settings: Mutex::new(Settings::default()),
            projects: Mutex::new(Vec::new()),
            plugins: Mutex::new(Vec::new()),
            bindings: Mutex::new(Vec::new()),
        }
    }
}

fn get_data_dir(app: &AppHandle) -> PathBuf {
    // 使用 Tauri 标准的跨平台应用数据目录
    // Windows: %APPDATA%/godot-harbor
    // macOS: ~/Library/Application Support/godot-harbor
    // Linux: ~/.config/godot-harbor
    app.path().app_data_dir()
        .expect("Failed to get app data directory")
}

fn get_storage(app: &AppHandle) -> Storage {
    let data_dir = get_data_dir(app);
    Storage::new(data_dir)
}

fn get_plugin_manager(app: &AppHandle) -> PluginManager {
    let data_dir = get_data_dir(app);
    PluginManager::new(data_dir.join("plugins"))
}

#[tauri::command]
pub fn get_settings(app: AppHandle) -> Result<Settings, String> {
    let storage = get_storage(&app);
    let settings = storage.load_or_default("settings.json");
    Ok(settings)
}

#[tauri::command]
pub fn save_settings(app: AppHandle, settings: Settings) -> Result<(), String> {
    let storage = get_storage(&app);
    storage.save("settings.json", &settings)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn scan_projects(app: AppHandle, root_dirs: Vec<String>) -> Result<Vec<Project>, String> {
    let mut all_projects = Vec::new();
    
    for root_dir in root_dirs {
        match ProjectScanner::scan_directory(&root_dir) {
            Ok(projects) => all_projects.extend(projects),
            Err(e) => eprintln!("Failed to scan directory {}: {}", root_dir, e),
        }
    }
    
    let storage = get_storage(&app);
    let mut existing_projects: Vec<Project> = storage.load_or_default("projects.json");
    
    for project in &all_projects {
        if let Some(index) = existing_projects.iter().position(|p| p.path == project.path) {
            // 更新已存在项目的信息（保留 project_id 和 status）
            let mut existing = existing_projects[index].clone();
            existing.name = project.name.clone();
            existing.godot_version = project.godot_version.clone();
            existing_projects[index] = existing;
        } else {
            // 添加新项目
            existing_projects.push(project.clone());
        }
    }
    
    storage.save("projects.json", &existing_projects)
        .map_err(|e| e.to_string())?;
    
    Ok(all_projects)
}

#[tauri::command]
pub fn get_projects(app: AppHandle) -> Result<Vec<Project>, String> {
    let storage = get_storage(&app);
    let projects: Vec<Project> = storage.load_or_default("projects.json");
    Ok(projects)
}

#[tauri::command]
pub fn add_project(app: AppHandle, path: String) -> Result<Project, String> {
    let project_godot = std::path::Path::new(&path).join("project.godot");
    
    if !project_godot.exists() {
        return Err("project.godot not found in the specified path".to_string());
    }
    
    let project = ProjectScanner::parse_project(&project_godot)
        .map_err(|e| e.to_string())?;
    
    let storage = get_storage(&app);
    let mut projects: Vec<Project> = storage.load_or_default("projects.json");
    
    if projects.iter().any(|p| p.path == project.path) {
        return Err("Project already exists".to_string());
    }
    
    projects.push(project.clone());
    storage.save("projects.json", &projects)
        .map_err(|e| e.to_string())?;
    
    Ok(project)
}

#[tauri::command]
pub fn remove_project(app: AppHandle, project_id: String) -> Result<(), String> {
    let storage = get_storage(&app);
    let mut projects: Vec<Project> = storage.load_or_default("projects.json");
    
    projects.retain(|p| p.project_id != project_id);
    
    storage.save("projects.json", &projects)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn import_plugin_from_local(app: AppHandle, path: String) -> Result<Plugin, String> {
    let manager = get_plugin_manager(&app);
    let plugin = manager.import_from_local(&path)
        .map_err(|e| e.to_string())?;
    
    let storage = get_storage(&app);
    let mut plugins: Vec<Plugin> = storage.load_or_default("plugins.json");
    
    plugins.push(plugin.clone());
    storage.save("plugins.json", &plugins)
        .map_err(|e| e.to_string())?;
    
    Ok(plugin)
}

#[tauri::command]
pub fn import_plugin_from_git(app: AppHandle, url: String) -> Result<Plugin, String> {
    let manager = get_plugin_manager(&app);
    let plugin = manager.import_from_git(&url)
        .map_err(|e| e.to_string())?;
    
    let storage = get_storage(&app);
    let mut plugins: Vec<Plugin> = storage.load_or_default("plugins.json");
    
    plugins.push(plugin.clone());
    storage.save("plugins.json", &plugins)
        .map_err(|e| e.to_string())?;
    
    Ok(plugin)
}

#[tauri::command]
pub fn get_plugins(app: AppHandle) -> Result<Vec<Plugin>, String> {
    let storage = get_storage(&app);
    let plugins: Vec<Plugin> = storage.load_or_default("plugins.json");
    Ok(plugins)
}

#[tauri::command]
pub fn remove_plugin(app: AppHandle, plugin_id: String) -> Result<(), String> {
    let storage = get_storage(&app);
    let mut plugins: Vec<Plugin> = storage.load_or_default("plugins.json");
    
    plugins.retain(|p| p.plugin_id != plugin_id);
    
    storage.save("plugins.json", &plugins)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn bind_plugin(
    app: AppHandle,
    project_id: String,
    plugin_id: String,
    version_id: String,
    unit_id: String,
    mount_path: String,
) -> Result<(), String> {
    let binding = ProjectBinding::new(project_id, plugin_id, version_id, unit_id, mount_path);
    
    let storage = get_storage(&app);
    let mut bindings: Vec<ProjectBinding> = storage.load_or_default("bindings.json");
    
    bindings.retain(|b| !(b.project_id == binding.project_id && b.plugin_id == binding.plugin_id));
    bindings.push(binding);
    
    storage.save("bindings.json", &bindings)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn unbind_plugin(app: AppHandle, project_id: String, plugin_id: String) -> Result<(), String> {
    let storage = get_storage(&app);
    let mut bindings: Vec<ProjectBinding> = storage.load_or_default("bindings.json");
    
    bindings.retain(|b| !(b.project_id == project_id && b.plugin_id == plugin_id));
    
    storage.save("bindings.json", &bindings)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn apply_changes(app: AppHandle, project_id: String) -> Result<ApplyResult, String> {
    let storage = get_storage(&app);
    
    let projects: Vec<Project> = storage.load_or_default("projects.json");
    let project = projects.iter()
        .find(|p| p.project_id == project_id)
        .ok_or("Project not found")?;
    
    let bindings: Vec<ProjectBinding> = storage.load_or_default("bindings.json");
    let project_bindings: Vec<ProjectBinding> = bindings.iter()
        .filter(|b| b.project_id == project_id)
        .cloned()
        .collect();
    
    let settings: Settings = storage.load_or_default("settings.json");
    let linker = Linker::new(settings.mount_strategy);
    
    let data_dir = get_data_dir(&app);
    
    linker.apply_bindings(&project.path, &project_bindings, &data_dir.to_string_lossy())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_project_bindings(app: AppHandle, project_id: String) -> Result<Vec<ProjectBinding>, String> {
    let storage = get_storage(&app);
    let bindings: Vec<ProjectBinding> = storage.load_or_default("bindings.json");
    
    Ok(bindings.into_iter()
        .filter(|b| b.project_id == project_id)
        .collect())
}

#[tauri::command]
pub fn scan_project_plugins(app: AppHandle) -> Result<Vec<String>, String> {
    let storage = get_storage(&app);
    let projects: Vec<Project> = storage.load_or_default("projects.json");
    
    let manager = get_plugin_manager(&app);
    let plugin_paths = manager.scan_project_plugins(&projects)
        .map_err(|e| e.to_string())?;
    
    Ok(plugin_paths.iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect())
}

#[tauri::command]
pub fn import_plugins_from_projects(app: AppHandle) -> Result<Vec<Plugin>, String> {
    let storage = get_storage(&app);
    let projects: Vec<Project> = storage.load_or_default("projects.json");
    let mut plugins: Vec<Plugin> = storage.load_or_default("plugins.json");
    
    let manager = get_plugin_manager(&app);
    let plugin_paths = manager.scan_project_plugins(&projects)
        .map_err(|e| e.to_string())?;
    
    let mut imported_plugins = Vec::new();
    
    for plugin_path in plugin_paths {
        let path_str = plugin_path.to_string_lossy().to_string();
        
        // 检查是否已经导入过该插件
        let already_imported = plugins.iter()
            .any(|p| p.source.url == path_str);
        
        if !already_imported {
            match manager.import_from_local(&path_str) {
                Ok(plugin) => {
                    imported_plugins.push(plugin.clone());
                    plugins.push(plugin);
                }
                Err(e) => eprintln!("Failed to import plugin from {}: {}", path_str, e),
            }
        }
    }
    
    storage.save("plugins.json", &plugins)
        .map_err(|e| e.to_string())?;
    
    Ok(imported_plugins)
}
