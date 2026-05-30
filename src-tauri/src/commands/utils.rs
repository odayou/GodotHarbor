use std::path::PathBuf;
use std::io::Write;
use serde::{Serialize, Deserialize};
use tauri::{AppHandle, Manager};
use crate::models::*;
use crate::storage::Storage;
use crate::plugin_manager::PluginManager;
use crate::operation_log::OperationLogger;

pub fn get_config_dir(app: &AppHandle) -> PathBuf {
    app.path().app_data_dir()
        .expect("Failed to get app data directory")
}

pub fn get_data_dir(app: &AppHandle) -> PathBuf {
    let config_dir = get_config_dir(app);
    let config_storage = Storage::new(config_dir.clone());
    let settings: Settings = config_storage.load_or_default("settings.json");
    if !settings.custom_data_dir.is_empty() {
        PathBuf::from(&settings.custom_data_dir)
    } else if !settings.data_dir_initialized {
        get_app_root_dir().join("GodotHarborData")
    } else {
        config_dir
    }
}

pub fn get_app_root_dir() -> PathBuf {
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let mut dir = exe_dir.to_path_buf();
            if dir.ends_with("target\\debug") || dir.ends_with("target/release") {
                if let Some(parent) = dir.parent() {
                    if let Some(grandparent) = parent.parent() {
                        dir = grandparent.to_path_buf();
                    }
                }
            }
            return dir;
        }
    }
    PathBuf::from(".")
}

pub fn get_config_storage(app: &AppHandle) -> Storage {
    Storage::new(get_config_dir(app))
}

pub fn get_storage(app: &AppHandle) -> Storage {
    Storage::new(get_data_dir(app))
}

pub fn load_settings(app: &AppHandle) -> Settings {
    get_config_storage(app).load_or_default("settings.json")
}

pub fn save_settings_to_config(app: &AppHandle, settings: &Settings) -> Result<(), String> {
    get_config_storage(app).save("settings.json", settings)
        .map_err(|e| format!("保存设置失败: {}", e))
}

pub fn get_plugin_manager(app: &AppHandle) -> PluginManager {
    let plugins_dir = get_data_dir(app).join("plugins");
    PluginManager::new(plugins_dir)
}

pub fn validate_project_path(app: &AppHandle, project_path: &std::path::Path) -> Result<(), String> {
    let storage = get_storage(app);
    let engines: Vec<Engine> = storage.load_or_default("engines.json");

    let canonical_project = project_path.canonicalize()
        .or_else(|_| std::path::absolute(project_path))
        .unwrap_or_else(|_| project_path.to_path_buf());

    for engine in &engines {
        let engine_dir = std::path::Path::new(&engine.path);
        let exe_path = crate::engine::EngineManager::find_executable_in_dir(engine_dir);
        let check_path = match &exe_path {
            Some(exe) => exe.parent().unwrap_or(engine_dir).to_path_buf(),
            None => engine_dir.to_path_buf(),
        };

        let canonical_engine = check_path.canonicalize()
            .or_else(|_| std::path::absolute(&check_path))
            .unwrap_or_else(|_| check_path.clone());

        if canonical_project == canonical_engine {
            return Err(format!(
                "项目路径与引擎目录冲突（{}），Godot 不允许在引擎目录创建项目，请选择其他路径",
                engine.name
            ));
        }

        let mut parent = canonical_project.as_path();
        while let Some(p) = parent.parent() {
            if p == canonical_engine {
                return Err(format!(
                    "项目路径位于引擎目录内（{}），Godot 不允许在引擎目录创建项目，请选择其他路径",
                    engine.name
                ));
            }
            parent = p;
        }

        let mut engine_parent = canonical_engine.as_path();
        while let Some(p) = engine_parent.parent() {
            if p == canonical_project {
                return Err(format!(
                    "引擎目录（{}）位于项目路径内，Godot 不允许此配置，请选择其他项目路径",
                    engine.name
                ));
            }
            engine_parent = p;
        }
    }

    Ok(())
}

pub fn get_logger(app: &AppHandle) -> OperationLogger {
    let data_dir = get_data_dir(app);
    OperationLogger::new(data_dir)
}

pub fn upsert_plugin(app: &AppHandle, new_plugin: &crate::models::Plugin, operation: &str, source_desc: &str) -> Result<crate::models::Plugin, String> {
    let storage = get_storage(app);
    let mut plugins: Vec<crate::models::Plugin> = storage.load_or_default("plugins.json");
    let plugin_name = new_plugin.name.clone();
    let existing_idx = plugins.iter().position(|p| p.source.url == new_plugin.source.url);
    if let Some(idx) = existing_idx {
        plugins[idx].versions.extend(new_plugin.versions.clone());
        if !new_plugin.content_hash.is_empty() {
            plugins[idx].content_hash = new_plugin.content_hash.clone();
        }
        let result = plugins[idx].clone();
        storage.save("plugins.json", &plugins)
            .map_err(|e| format!("保存插件列表失败: {}", e))?;
        log_operation(app, operation, source_desc, &format!("已为插件 {} 添加新版本", plugin_name));
        Ok(result)
    } else {
        if !new_plugin.content_hash.is_empty() {
            if let Some(dup) = plugins.iter().find(|p| !p.content_hash.is_empty() && p.content_hash == new_plugin.content_hash) {
                log_operation(app, operation, source_desc,
                    &format!("插件 {} 与已有插件 {} 内容相同(hash匹配)", plugin_name, dup.name));
            }
        }
        plugins.push(new_plugin.clone());
        storage.save("plugins.json", &plugins)
            .map_err(|e| format!("保存插件列表失败: {}", e))?;
        log_operation(app, operation, source_desc, &format!("已导入插件: {}", plugin_name));
        Ok(new_plugin.clone())
    }
}

pub const DATA_FILES: &[&str] = &[
    "settings.json",
    "projects.json",
    "plugins.json",
    "bindings.json",
    "engines.json",
    "operation_logs.json",
    "update_logs.json"
];

pub fn log_operation(app: &AppHandle, action: &str, target: &str, detail: &str) {
    let logger = get_logger(app);
    if let Err(e) = logger.log(action, target, detail) {
        eprintln!("Failed to write operation log: {}", e);
    }
}

pub fn log_error(app: &AppHandle, action: &str, target: &str, error: &str) {
    let logger = get_logger(app);
    if let Err(e) = logger.log_error(action, target, error) {
        eprintln!("Failed to write error log: {}", e);
    }
}

pub fn backup_addons_dir(addons_dir: &std::path::Path, backup_file: &std::path::Path) -> Result<(), String> {
    let file = std::fs::File::create(backup_file)
        .map_err(|e| format!("创建备份文件失败: {}", e))?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    let mut stack = vec![(addons_dir.to_path_buf(), String::new())];
    while let Some((dir, prefix)) = stack.pop() {
        let entries = std::fs::read_dir(&dir)
            .map_err(|e| format!("读取目录失败: {}", e))?;
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            let entry_path = if prefix.is_empty() {
                name.clone()
            } else {
                format!("{}/{}", prefix, name)
            };
            if path.is_dir() {
                zip.add_directory(&entry_path, options)
                    .map_err(|e| format!("写入目录条目失败: {}", e))?;
                stack.push((path, entry_path));
            } else {
                let data = std::fs::read(&path)
                    .map_err(|e| format!("读取文件失败: {}", e))?;
                zip.start_file(&entry_path, options)
                    .map_err(|e| format!("写入文件条目失败: {}", e))?;
                zip.write_all(&data)
                    .map_err(|e| format!("写入文件数据失败: {}", e))?;
            }
        }
    }
    zip.finish().map_err(|e| format!("完成压缩失败: {}", e))?;
    Ok(())
}

pub fn cleanup_old_backups(backup_dir: &std::path::Path, max_keep: usize) {
    if let Ok(entries) = std::fs::read_dir(backup_dir) {
        let mut backups: Vec<std::path::PathBuf> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| {
                p.extension().map(|e| e == "zip").unwrap_or(false)
                && p.file_name()
                    .map(|n| n.to_string_lossy().starts_with("addons_backup_"))
                    .unwrap_or(false)
            })
            .collect();
        if backups.len() > max_keep {
            backups.sort();
            let to_remove = backups.len() - max_keep;
            for old_file in backups.iter().take(to_remove) {
                let _ = std::fs::remove_file(old_file);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddonBackupInfo {
    pub file_name: String,
    pub file_path: String,
    pub file_size: u64,
    pub created_at: String,
}

#[cfg(windows)]
pub fn detached_cmd(program: impl AsRef<std::ffi::OsStr>) -> std::process::Command {
    use std::os::windows::process::CommandExt;
    const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
    const DETACHED_PROCESS: u32 = 0x00000008;
    let mut cmd = std::process::Command::new(program);
    cmd.creation_flags(CREATE_NEW_PROCESS_GROUP | DETACHED_PROCESS);
    cmd
}

#[cfg(not(windows))]
pub fn detached_cmd(program: impl AsRef<std::ffi::OsStr>) -> std::process::Command {
    let mut cmd = std::process::Command::new(program);
    cmd
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct AutoSetupState {
    pub completed_at: i64,
    pub settings_hash: String,
}

pub fn compute_settings_hash(settings: &Settings) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    settings.scan_directories.join(",").hash(&mut hasher);
    settings.auto_scan_on_startup.hash(&mut hasher);
    settings.auto_discover_engines.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;
    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

pub fn dir_size(path: &std::path::Path) -> u64 {
    walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter_map(|e| e.metadata().ok())
        .filter(|m| m.is_file())
        .map(|m| m.len())
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size_bytes() {
        assert_eq!(format_size(500), "500 B");
    }

    #[test]
    fn test_format_size_kb() {
        assert_eq!(format_size(1536), "1.50 KB");
    }

    #[test]
    fn test_format_size_mb() {
        assert_eq!(format_size(1048576), "1.00 MB");
    }

    #[test]
    fn test_format_size_gb() {
        assert_eq!(format_size(1073741824), "1.00 GB");
    }

    #[test]
    fn test_format_size_zero() {
        assert_eq!(format_size(0), "0 B");
    }

    #[test]
    fn test_format_size_exact_kb() {
        assert_eq!(format_size(1024), "1.00 KB");
    }

    #[test]
    fn test_compute_settings_hash_deterministic() {
        let settings = Settings::default();
        let hash1 = compute_settings_hash(&settings);
        let hash2 = compute_settings_hash(&settings);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_compute_settings_hash_changes_with_settings() {
        let settings1 = Settings::default();
        let mut settings2 = Settings::default();
        settings2.scan_directories = vec!["/different/path".to_string()];
        assert_ne!(compute_settings_hash(&settings1), compute_settings_hash(&settings2));
    }

    #[test]
    fn test_dir_size_empty() {
        let dir = tempfile::TempDir::new().unwrap();
        assert_eq!(dir_size(dir.path()), 0);
    }

    #[test]
    fn test_dir_size_with_files() {
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::write(dir.path().join("test.txt"), b"hello").unwrap();
        std::fs::write(dir.path().join("test2.txt"), b"world!").unwrap();
        assert_eq!(dir_size(dir.path()), 11);
    }
}
