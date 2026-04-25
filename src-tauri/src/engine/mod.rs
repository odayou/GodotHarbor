use std::path::Path;
use crate::models::{Engine, EngineType};
use anyhow::{Result, anyhow};

pub struct EngineManager {
    engines_base_path: std::path::PathBuf,
}

impl EngineManager {
    pub fn new(engines_base_path: std::path::PathBuf) -> Self {
        std::fs::create_dir_all(&engines_base_path).ok();
        Self { engines_base_path }
    }

    pub fn detect_engine(path: &str) -> Result<(EngineType, String)> {
        let engine_path = Path::new(path);

        if !engine_path.exists() {
            return Err(anyhow!("引擎路径不存在"));
        }

        let exe_name = if cfg!(windows) { "godot.exe" } else { "godot" };
        let exe_path = engine_path.join(exe_name);

        if !exe_path.exists() {
            let alt_exe = engine_path.join(format!("bin/{}", exe_name));
            if !alt_exe.exists() {
                return Err(anyhow!("未在指定路径找到 Godot 可执行文件"));
            }
        }

        let version_output = Self::get_version_from_executable(path)?;
        let engine_type = Self::detect_engine_type(&version_output);
        let version = Self::parse_version(&version_output);

        Ok((engine_type, version))
    }

    fn get_version_from_executable(path: &str) -> Result<String> {
        let exe_name = if cfg!(windows) { "godot.exe" } else { "godot" };
        let exe_path = Path::new(path).join(exe_name);

        let actual_exe = if exe_path.exists() {
            exe_path
        } else {
            Path::new(path).join(format!("bin/{}", exe_name))
        };

        let output = std::process::Command::new(&actual_exe)
            .arg("--version")
            .output()?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(anyhow!("获取引擎版本失败"))
        }
    }

    fn detect_engine_type(version_output: &str) -> EngineType {
        if version_output.contains("4.") {
            EngineType::Godot4
        } else if version_output.contains("3.") {
            EngineType::Godot3
        } else {
            EngineType::Unknown
        }
    }

    fn parse_version(version_output: &str) -> String {
        let version_str = version_output.trim();
        if version_str.is_empty() {
            return "Unknown".to_string();
        }
        let parts: Vec<&str> = version_str.split_whitespace().collect();
        if parts.is_empty() {
            return "Unknown".to_string();
        }
        parts.last().unwrap_or(&"Unknown").to_string()
    }

    pub fn validate_engine_path(path: &str) -> bool {
        let engine_path = Path::new(path);
        if !engine_path.exists() {
            return false;
        }

        let exe_name = if cfg!(windows) { "godot.exe" } else { "godot" };
        let exe_path = engine_path.join(exe_name);
        if exe_path.exists() {
            return true;
        }

        let alt_exe = engine_path.join(format!("bin/{}", exe_name));
        alt_exe.exists()
    }

    pub fn get_engine_info(path: &str) -> Result<Engine> {
        let (engine_type, version) = Self::detect_engine(path)?;

        let name = if cfg!(windows) {
            Path::new(path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Godot")
                .to_string()
        } else {
            Path::new(path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Godot")
                .to_string()
        };

        Ok(Engine::new(name, path.to_string(), engine_type, version))
    }

    pub fn discover_engines(existing_paths: &[String]) -> Vec<Engine> {
        let mut discovered = Vec::new();
        let search_dirs = Self::get_search_directories();

        for dir in &search_dirs {
            if !dir.exists() {
                continue;
            }
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if !path.is_dir() {
                        continue;
                    }

                    let path_str = path.to_string_lossy().to_string();
                    if existing_paths.contains(&path_str) {
                        continue;
                    }

                    if Self::validate_engine_path(&path_str) {
                        match Self::get_engine_info(&path_str) {
                            Ok(engine) => {
                                let already_found = discovered
                                    .iter()
                                    .any(|e: &Engine| e.path == engine.path);
                                if !already_found {
                                    discovered.push(engine);
                                }
                            }
                            Err(_) => continue,
                        }
                    }
                }
            }
        }

        Self::discover_from_path(existing_paths)
            .into_iter()
            .for_each(|engine| {
                if !discovered.iter().any(|e| e.path == engine.path) {
                    discovered.push(engine);
                }
            });

        discovered
    }

    fn get_search_directories() -> Vec<std::path::PathBuf> {
        let mut dirs = Vec::new();

        if cfg!(windows) {
            if let Some(program_files) = std::env::var("ProgramFiles").ok() {
                dirs.push(std::path::PathBuf::from(&program_files));
            }
            if let Some(program_files_x86) = std::env::var("ProgramFiles(x86)").ok() {
                dirs.push(std::path::PathBuf::from(&program_files_x86));
            }
            if let Some(local_app_data) = std::env::var("LOCALAPPDATA").ok() {
                dirs.push(std::path::PathBuf::from(&local_app_data).join("Programs"));
                dirs.push(std::path::PathBuf::from(&local_app_data).join("Godot"));
            }
            if let Some(userprofile) = std::env::var("USERPROFILE").ok() {
                dirs.push(std::path::PathBuf::from(&userprofile).join("Downloads"));
                dirs.push(std::path::PathBuf::from(&userprofile).join("Documents").join("Godot"));
                dirs.push(std::path::PathBuf::from(&userprofile).join("Desktop"));
            }
            for drive in ['C', 'D', 'E', 'F'] {
                let godot_dir = std::path::PathBuf::from(format!("{}:\\Godot", drive));
                if godot_dir.exists() {
                    dirs.push(godot_dir);
                }
                let tools_dir = std::path::PathBuf::from(format!("{}:\\Tools\\Godot", drive));
                if tools_dir.exists() {
                    dirs.push(tools_dir);
                }
            }
        } else if cfg!(target_os = "macos") {
            dirs.push(std::path::PathBuf::from("/Applications"));
            if let Some(home) = std::env::var("HOME").ok() {
                dirs.push(std::path::PathBuf::from(&home).join("Applications"));
                dirs.push(std::path::PathBuf::from(&home).join("Downloads"));
            }
            dirs.push(std::path::PathBuf::from("/usr/local/bin"));
        } else {
            if let Some(home) = std::env::var("HOME").ok() {
                dirs.push(std::path::PathBuf::from(&home).join(".local").join("bin"));
                dirs.push(std::path::PathBuf::from(&home).join("Downloads"));
                dirs.push(std::path::PathBuf::from(&home).join("bin"));
                dirs.push(std::path::PathBuf::from(&home).join(".godot"));
            }
            dirs.push(std::path::PathBuf::from("/usr/local/bin"));
            dirs.push(std::path::PathBuf::from("/usr/bin"));
            dirs.push(std::path::PathBuf::from("/opt"));
        }

        dirs
    }

    fn discover_from_path(existing_paths: &[String]) -> Vec<Engine> {
        let mut found = Vec::new();

        let path_var = std::env::var("PATH").unwrap_or_default();
        let separator = if cfg!(windows) { ';' } else { ':' };

        for dir_str in path_var.split(separator) {
            let dir = std::path::Path::new(dir_str);
            if !dir.exists() {
                continue;
            }

            let candidates = if cfg!(windows) {
                vec!["godot.exe", "godot4.exe", "godot3.exe"]
            } else {
                vec!["godot", "godot4", "godot3"]
            };

            for candidate in &candidates {
                let exe_path = dir.join(candidate);
                if !exe_path.exists() {
                    continue;
                }

                let parent = exe_path.parent().unwrap_or(dir);
                let parent_str = parent.to_string_lossy().to_string();

                if existing_paths.contains(&parent_str) {
                    continue;
                }
                if found.iter().any(|e: &Engine| e.path == parent_str) {
                    continue;
                }

                let output = match std::process::Command::new(&exe_path)
                    .arg("--version")
                    .output()
                {
                    Ok(o) => o,
                    Err(_) => continue,
                };

                if !output.status.success() {
                    continue;
                }

                let version_output = String::from_utf8_lossy(&output.stdout).to_string();
                let engine_type = Self::detect_engine_type(&version_output);
                let version = Self::parse_version(&version_output);

                let name = format!(
                    "Godot {}",
                    if matches!(engine_type, EngineType::Godot4) {
                        "4"
                    } else if matches!(engine_type, EngineType::Godot3) {
                        "3"
                    } else {
                        ""
                    }
                );

                found.push(Engine::new(name, parent_str, engine_type, version));
            }
        }

        found
    }
}
