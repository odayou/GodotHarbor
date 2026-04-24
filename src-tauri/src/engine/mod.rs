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
}