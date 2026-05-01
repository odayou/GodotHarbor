use std::path::Path;
use std::collections::HashSet;
use crate::models::{Engine, EngineType};
use anyhow::{Result, anyhow};
use rayon::prelude::*;
use regex::Regex;
use walkdir::WalkDir;

const MAX_SCAN_DEPTH: usize = 5;

pub struct EngineManager {
    #[allow(dead_code)]
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

        let exe_path = Self::find_executable_in_dir(engine_path);
        if exe_path.is_none() {
            return Err(anyhow!("未在指定路径找到 Godot 可执行文件"));
        }

        let version_output = Self::get_version_from_executable(path)?;
        let engine_type = Self::detect_engine_type(&version_output);
        let version = Self::parse_version(&version_output);

        Ok((engine_type, version))
    }

    pub fn find_executable_in_dir(dir: &Path) -> Option<std::path::PathBuf> {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && Self::is_godot_executable(&path) {
                    return Some(path);
                }
            }
        }

        let bin_dir = dir.join("bin");
        if bin_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&bin_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() && Self::is_godot_executable(&path) {
                        return Some(path);
                    }
                }
            }
        }

        None
    }

    fn is_godot_executable(path: &Path) -> bool {
        let file_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name,
            None => return false,
        };

        let lower = file_name.to_lowercase();

        if lower.contains("template") || lower.contains("project") || lower.contains("harbor") {
            return false;
        }

        #[cfg(windows)]
        {
            if !lower.ends_with(".exe") {
                return false;
            }
            let stem = lower.trim_end_matches(".exe");
            stem == "godot"
                || stem == "godot3"
                || stem == "godot4"
                || stem == "godot_mono"
                || stem == "godot3_mono"
                || stem == "godot4_mono"
                || stem.starts_with("godot_v")
                || stem.starts_with("godot4.")
                || stem.starts_with("godot3.")
                || stem.starts_with("godot-")
                || (stem.starts_with("godot") && stem.contains("_") && !stem.contains("harbor"))
        }

        #[cfg(not(windows))]
        {
            lower == "godot"
                || lower == "godot3"
                || lower == "godot4"
                || lower == "godot_mono"
                || lower == "godot3_mono"
                || lower == "godot4_mono"
                || lower.starts_with("godot_v")
                || lower.starts_with("godot4.")
                || lower.starts_with("godot3.")
                || lower.starts_with("godot-")
                || (lower.starts_with("godot") && lower.contains("_") && !lower.contains("harbor"))
        }
    }

    fn get_version_from_executable(dir: &str) -> Result<String> {
        let dir_path = Path::new(dir);
        let exe_path = Self::find_executable_in_dir(dir_path)
            .ok_or_else(|| anyhow!("未找到 Godot 可执行文件"))?;

        let output = std::process::Command::new(&exe_path)
            .arg("--version")
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()
            .ok();

        if let Some(output) = output {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout).to_string();
                if !version.trim().is_empty() {
                    return Ok(version);
                }
            }
        }

        let stem = exe_path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("godot");
        let re = Regex::new(r"(\d+\.\d+(?:\.\d+)*)").unwrap();
        if let Some(caps) = re.captures(stem) {
            if let Some(m) = caps.get(1) {
                return Ok(m.as_str().trim_end_matches('.').to_string());
            }
        }

        Err(anyhow!("获取引擎版本失败"))
    }

    fn detect_engine_type(version_output: &str) -> EngineType {
        let version_str = version_output.trim();
        if version_str.starts_with('4') || version_str.contains(".4.") {
            EngineType::Godot4
        } else if version_str.starts_with('3') || version_str.contains(".3.") {
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

        let lower = version_str.to_lowercase();
        let is_mono = lower.contains("mono") || lower.contains(".net");

        let re = Regex::new(r"(\d+\.\d+(?:\.\d+)*)").unwrap();
        let numeric_part = if let Some(caps) = re.captures(version_str) {
            if let Some(m) = caps.get(1) {
                m.as_str().trim_end_matches('.').to_string()
            } else {
                let parts: Vec<&str> = version_str.split('.').collect();
                parts.first().unwrap_or(&"Unknown").to_string()
            }
        } else {
            let parts: Vec<&str> = version_str.split_whitespace().collect();
            parts.first().unwrap_or(&"Unknown").to_string()
        };

        let channel = if lower.contains("stable") {
            "stable".to_string()
        } else if lower.contains("rc") {
            if let Some(caps) = Regex::new(r"rc(\d+)").unwrap().captures(&lower) {
                caps.get(1).map_or("rc".to_string(), |m| format!("rc{}", m.as_str()))
            } else {
                "rc".to_string()
            }
        } else if lower.contains("beta") {
            if let Some(caps) = Regex::new(r"beta(\d+)").unwrap().captures(&lower) {
                caps.get(1).map_or("beta".to_string(), |m| format!("beta{}", m.as_str()))
            } else {
                "beta".to_string()
            }
        } else if lower.contains("alpha") {
            if let Some(caps) = Regex::new(r"alpha(\d+)").unwrap().captures(&lower) {
                caps.get(1).map_or("alpha".to_string(), |m| format!("alpha{}", m.as_str()))
            } else {
                "alpha".to_string()
            }
        } else if lower.contains("dev") {
            if let Some(caps) = Regex::new(r"dev(\d+)").unwrap().captures(&lower) {
                caps.get(1).map_or("dev".to_string(), |m| format!("dev{}", m.as_str()))
            } else {
                "dev".to_string()
            }
        } else {
            String::new()
        };

        let mut result = numeric_part;
        if !channel.is_empty() {
            result.push('-');
            result.push_str(&channel);
        }
        if is_mono {
            result.push_str("-mono");
        }

        result
    }

    pub fn validate_engine_path(path: &str) -> bool {
        let engine_path = Path::new(path);
        if !engine_path.exists() {
            return false;
        }
        Self::find_executable_in_dir(engine_path).is_some()
    }

    pub fn validate_engine_path_detail(path: &str) -> Result<(), String> {
        let engine_path = Path::new(path);
        if !engine_path.exists() {
            return Err(format!("引擎目录不存在: {}", path));
        }
        if !engine_path.is_dir() {
            return Err(format!("路径不是目录: {}", path));
        }
        if let Ok(entries) = std::fs::read_dir(engine_path) {
            let files: Vec<String> = entries
                .filter_map(|e| e.ok())
                .map(|e| e.file_name().to_string_lossy().to_string())
                .collect();
            if files.is_empty() {
                return Err("引擎目录为空，未找到任何文件".to_string());
            }
            let exe_files: Vec<&String> = files.iter().filter(|f| {
                let lower = f.to_lowercase();
                lower.ends_with(".exe") || (!lower.contains('.') && lower.contains("godot"))
            }).collect();
            if exe_files.is_empty() {
                return Err(format!("目录中未找到 Godot 可执行文件，目录内容: {}", files.join(", ")));
            }
        }
        if Self::find_executable_in_dir(engine_path).is_none() {
            return Err("找到文件但无法识别为有效的 Godot 引擎可执行文件（文件名需匹配 godot/godot3/godot4/godot_v* 等模式）".to_string());
        }
        Ok(())
    }

    pub fn get_engine_info(path: &str) -> Result<Engine> {
        let (engine_type, version) = Self::detect_engine(path)?;

        let is_mono = version.to_lowercase().contains("mono");
        let name = if is_mono {
            format!("Godot {} (.NET)", version)
        } else {
            format!("Godot {}", version)
        };

        Ok(Engine::new(name, path.to_string(), engine_type, version))
    }

    pub fn discover_engines(existing_paths: &[String]) -> Vec<Engine> {
        let mut discovered = Vec::new();
        let mut seen_paths: HashSet<String> = existing_paths.iter().cloned().collect();

        let platform_engines = Self::discover_from_platform();
        for engine in platform_engines {
            if !seen_paths.contains(&engine.path) {
                seen_paths.insert(engine.path.clone());
                discovered.push(engine);
            }
        }

        let path_engines = Self::discover_from_path();
        for engine in path_engines {
            if !seen_paths.contains(&engine.path) {
                seen_paths.insert(engine.path.clone());
                discovered.push(engine);
            }
        }

        let search_dirs = Self::get_search_directories();
        let dir_engines = Self::search_directories_parallel(&search_dirs, &seen_paths);
        for engine in dir_engines {
            if !seen_paths.contains(&engine.path) {
                seen_paths.insert(engine.path.clone());
                discovered.push(engine);
            }
        }

        discovered
    }

    pub fn discover_engines_with_custom_paths(
        existing_paths: &[String],
        custom_paths: &[String],
    ) -> Vec<Engine> {
        let mut all_discovered = Self::discover_engines(existing_paths);
        let mut seen_paths: HashSet<String> = existing_paths.iter().cloned().collect();
        for e in &all_discovered {
            seen_paths.insert(e.path.clone());
        }

        let custom_dirs: Vec<std::path::PathBuf> = custom_paths
            .iter()
            .filter(|p| Path::new(p).exists())
            .map(std::path::PathBuf::from)
            .collect();

        let custom_engines = Self::search_directories_parallel(&custom_dirs, &seen_paths);
        for engine in custom_engines {
            if !seen_paths.contains(&engine.path) {
                seen_paths.insert(engine.path.clone());
                all_discovered.push(engine);
            }
        }

        all_discovered
    }

    #[cfg(windows)]
    fn discover_from_platform() -> Vec<Engine> {
        Self::discover_from_windows_registry()
    }

    #[cfg(target_os = "macos")]
    fn discover_from_platform() -> Vec<Engine> {
        let mut results = Vec::new();
        results.extend(Self::discover_from_macos_applications());
        results.extend(Self::discover_from_macos_spotlight());
        results
    }

    #[cfg(target_os = "linux")]
    fn discover_from_platform() -> Vec<Engine> {
        Self::discover_from_linux_desktop_entries()
    }

    #[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
    fn discover_from_platform() -> Vec<Engine> {
        Vec::new()
    }

    #[cfg(windows)]
    fn discover_from_windows_registry() -> Vec<Engine> {
        use winreg::enums::*;
        use winreg::RegKey;

        let mut found = Vec::new();
        let mut seen = HashSet::new();

        let hives = [
            (HKEY_LOCAL_MACHINE, vec![
                r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
                r"SOFTWARE\Wow6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
            ]),
            (HKEY_CURRENT_USER, vec![
                r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
            ]),
        ];

        for (hive, subkeys) in &hives {
            let root = RegKey::predef(*hive);
            for subkey in subkeys {
                if let Ok(uninstall_key) = root.open_subkey(subkey) {
                    for entry in uninstall_key.enum_keys().flatten() {
                        if let Ok(app_key) = uninstall_key.open_subkey(&entry) {
                            let display_name: Result<String, _> = app_key.get_value("DisplayName");
                            if let Ok(name) = display_name {
                                if name.to_lowercase().contains("godot") {
                                    let install_location: Result<String, _> = app_key.get_value("InstallLocation");
                                    let display_icon: Result<String, _> = app_key.get_value("DisplayIcon");
                                    let uninstall_string: Result<String, _> = app_key.get_value("UninstallString");

                                    let dir_path = install_location
                                        .ok()
                                        .or_else(|| display_icon.ok().and_then(|s| {
                                            let p = Path::new(&s);
                                            p.parent().map(|pp| pp.to_string_lossy().to_string())
                                        }))
                                        .or_else(|| uninstall_string.ok().and_then(|s| {
                                            let p = Path::new(&s);
                                            p.parent().map(|pp| pp.to_string_lossy().to_string())
                                        }));

                                    if let Some(dir_str) = dir_path {
                                        let dir_path = Path::new(&dir_str);
                                        let canonical = dir_path.to_string_lossy().to_string();

                                        if seen.contains(&canonical) {
                                            continue;
                                        }

                                        if Self::validate_engine_path(&canonical) {
                                            seen.insert(canonical.clone());
                                            match Self::get_engine_info(&canonical) {
                                                Ok(engine) => found.push(engine),
                                                Err(_) => {}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        found
    }

    #[cfg(target_os = "macos")]
    fn discover_from_macos_applications() -> Vec<Engine> {
        let mut found = Vec::new();
        let mut seen = HashSet::new();

        let app_dirs = [
            "/Applications",
        ];

        let home = std::env::var("HOME").unwrap_or_default();
        let user_app_dir = format!("{}/Applications", home);

        let mut search_paths: Vec<&str> = app_dirs.to_vec();
        let user_app_ref: &str = &user_app_dir;
        search_paths.push(user_app_ref);

        for dir_str in &search_paths {
            let dir = Path::new(dir_str);
            if !dir.exists() {
                continue;
            }
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if !path.is_dir() {
                        continue;
                    }
                    let file_name = path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_lowercase();

                    if !file_name.starts_with("godot") || !file_name.ends_with(".app") {
                        continue;
                    }

                    let macos_dir = path.join("Contents").join("MacOS");
                    if !macos_dir.exists() {
                        continue;
                    }

                    if let Ok(exe_entries) = std::fs::read_dir(&macos_dir) {
                        for exe_entry in exe_entries.flatten() {
                            let exe_path = exe_entry.path();
                            if exe_path.is_file() {
                                let canonical = path.to_string_lossy().to_string();
                                if seen.contains(&canonical) {
                                    continue;
                                }

                                let parent_str = path.parent()
                                    .map(|p| p.to_string_lossy().to_string())
                                    .unwrap_or_default();

                                if Self::validate_engine_path(&parent_str) {
                                    seen.insert(canonical);
                                    match Self::get_engine_info(&parent_str) {
                                        Ok(engine) => found.push(engine),
                                        Err(_) => {}
                                    }
                                } else {
                                    let exe_str = exe_path.to_string_lossy().to_string();
                                    let output = std::process::Command::new(&exe_path)
                                        .arg("--version")
                                        .output();
                                    if let Ok(output) = output {
                                        if output.status.success() {
                                            let version_output = String::from_utf8_lossy(&output.stdout).to_string();
                                            let engine_type = Self::detect_engine_type(&version_output);
                                            let version = Self::parse_version(&version_output);
                                            let name = path.file_stem()
                                                .and_then(|n| n.to_str())
                                                .unwrap_or("Godot")
                                                .to_string();
                                            let engine_dir = exe_path.parent()
                                                .map(|p| p.to_string_lossy().to_string())
                                                .unwrap_or_default();
                                            seen.insert(canonical);
                                            found.push(Engine::new(name, engine_dir, engine_type, version));
                                        }
                                    }
                                }
                                break;
                            }
                        }
                    }
                }
            }
        }

        found
    }

    #[cfg(target_os = "macos")]
    fn discover_from_macos_spotlight() -> Vec<Engine> {
        let mut found = Vec::new();

        let output = std::process::Command::new("mdfind")
            .arg("kMDItemCFBundleIdentifier == 'org.godotengine.godot*'")
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                let result = String::from_utf8_lossy(&output.stdout);
                for line in result.lines() {
                    let app_path = line.trim();
                    if !app_path.is_empty() && Path::new(app_path).exists() {
                        let parent = Path::new(app_path).parent()
                            .map(|p| p.to_string_lossy().to_string())
                            .unwrap_or_default();

                        if Self::validate_engine_path(&parent) {
                            match Self::get_engine_info(&parent) {
                                Ok(engine) => {
                                    if !found.iter().any(|e: &Engine| e.path == engine.path) {
                                        found.push(engine);
                                    }
                                }
                                Err(_) => {}
                            }
                        }
                    }
                }
            }
        }

        found
    }

    #[cfg(target_os = "linux")]
    fn discover_from_linux_desktop_entries() -> Vec<Engine> {
        let mut found = Vec::new();
        let mut seen = HashSet::new();

        let home = std::env::var("HOME").unwrap_or_default();

        let home_apps = format!("{}/.local/share/applications", home);
        let flatpak_apps = format!("{}/.local/share/flatpak/exports/share/applications", home);

        let desktop_dirs: Vec<&str> = vec![
            "/usr/share/applications",
            &home_apps,
            "/var/lib/flatpak/exports/share/applications",
            &flatpak_apps,
        ];

        for dir_str in &desktop_dirs {
            let dir = Path::new(dir_str);
            if !dir.exists() {
                continue;
            }
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    let file_name = path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_lowercase();

                    if !file_name.starts_with("godot") || !file_name.ends_with(".desktop") {
                        continue;
                    }

                    if let Ok(content) = std::fs::read_to_string(&path) {
                        for line in content.lines() {
                            let trimmed = line.trim();
                            if trimmed.starts_with("Exec=") {
                                let exec_value = trimmed.trim_start_matches("Exec=");
                                let exe_path_str = exec_value
                                    .split_whitespace()
                                    .next()
                                    .unwrap_or("")
                                    .trim_start_matches('"')
                                    .trim_end_matches('"');

                                if exe_path_str.is_empty() {
                                    continue;
                                }

                                let exe_path = Path::new(exe_path_str);
                                if !exe_path.exists() {
                                    continue;
                                }

                                let parent_str = exe_path.parent()
                                    .map(|p| p.to_string_lossy().to_string())
                                    .unwrap_or_default();

                                if seen.contains(&parent_str) {
                                    continue;
                                }

                                if Self::validate_engine_path(&parent_str) {
                                    seen.insert(parent_str.clone());
                                    match Self::get_engine_info(&parent_str) {
                                        Ok(engine) => found.push(engine),
                                        Err(_) => {}
                                    }
                                } else {
                                    let output = std::process::Command::new(exe_path)
                                        .arg("--version")
                                        .output();
                                    if let Ok(output) = output {
                                        if output.status.success() {
                                            let version_output = String::from_utf8_lossy(&output.stdout).to_string();
                                            let engine_type = Self::detect_engine_type(&version_output);
                                            let version = Self::parse_version(&version_output);
                                            let name = path.file_stem()
                                                .and_then(|n| n.to_str())
                                                .unwrap_or("Godot")
                                                .to_string();
                                            seen.insert(parent_str.clone());
                                            found.push(Engine::new(name, parent_str, engine_type, version));
                                        }
                                    }
                                }
                                break;
                            }
                        }
                    }
                }
            }
        }

        found
    }

    fn discover_from_path() -> Vec<Engine> {
        let mut found = Vec::new();
        let mut seen = HashSet::new();

        let path_var = std::env::var("PATH").unwrap_or_default();
        let separator = if cfg!(windows) { ';' } else { ':' };

        let candidates: Vec<&str> = if cfg!(windows) {
            vec!["godot.exe", "godot4.exe", "godot3.exe"]
        } else {
            vec!["godot", "godot4", "godot3"]
        };

        for dir_str in path_var.split(separator) {
            let dir = std::path::Path::new(dir_str);
            if !dir.exists() {
                continue;
            }

            for candidate in &candidates {
                let exe_path = dir.join(candidate);
                if !exe_path.exists() {
                    continue;
                }

                let parent = exe_path.parent().unwrap_or(dir);
                let parent_str = parent.to_string_lossy().to_string();

                if seen.contains(&parent_str) {
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
                    match engine_type {
                        EngineType::Godot4 => "4",
                        EngineType::Godot3 => "3",
                        EngineType::Unknown => "",
                    }
                );

                seen.insert(parent_str.clone());
                found.push(Engine::new(name, parent_str, engine_type, version));
            }
        }

        found
    }

    fn search_directories_parallel(
        dirs: &[std::path::PathBuf],
        seen_paths: &HashSet<String>,
    ) -> Vec<Engine> {
        let results: Vec<Vec<Engine>> = dirs
            .par_iter()
            .filter_map(|dir| {
                if !dir.exists() {
                    return None;
                }
                let mut local_found = Vec::new();
                let mut local_seen = seen_paths.clone();

                for entry in WalkDir::new(dir)
                    .max_depth(MAX_SCAN_DEPTH)
                    .follow_links(false)
                    .into_iter()
                    .filter_map(|e| e.ok())
                {
                    let path = entry.path();
                    if !path.is_dir() {
                        continue;
                    }

                    let path_str = path.to_string_lossy().to_string();
                    if local_seen.contains(&path_str) {
                        continue;
                    }

                    let dir_name = path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_lowercase();

                    if dir_name.ends_with(".app") {
                        continue;
                    }

                    let skip_dirs = ["target", "node_modules", ".git", "build", "dist", "cache", "__pycache__", ".cargo", "deps"];
                    if skip_dirs.iter().any(|sd| dir_name == *sd) {
                        continue;
                    }

                    let is_godot_dir = dir_name.contains("godot")
                        || Self::dir_contains_godot_executable(path);

                    if is_godot_dir {
                        if Self::validate_engine_path(&path_str) {
                            match Self::get_engine_info(&path_str) {
                                Ok(engine) => {
                                    if !local_seen.contains(&engine.path) {
                                        local_seen.insert(engine.path.clone());
                                        local_found.push(engine);
                                    }
                                }
                                Err(_) => {
                                    let name = path.file_name()
                                        .and_then(|n| n.to_str())
                                        .unwrap_or("Godot")
                                        .to_string();
                                    let engine = Engine::new(name, path_str.clone(), EngineType::Unknown, "Unknown".to_string());
                                    if !local_seen.contains(&engine.path) {
                                        local_seen.insert(engine.path.clone());
                                        local_found.push(engine);
                                    }
                                }
                            }
                        }
                    }
                }

                if local_found.is_empty() {
                    None
                } else {
                    Some(local_found)
                }
            })
            .collect();

        let mut all_engines = Vec::new();
        let mut global_seen = seen_paths.clone();
        for engines in results {
            for engine in engines {
                if !global_seen.contains(&engine.path) {
                    global_seen.insert(engine.path.clone());
                    all_engines.push(engine);
                }
            }
        }

        all_engines
    }

    fn dir_contains_godot_executable(dir: &Path) -> bool {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && Self::is_godot_executable(&path) {
                    return true;
                }
            }
        }
        false
    }

    #[cfg(windows)]
    fn scan_drive_root_dirs() -> Vec<std::path::PathBuf> {
        let mut result = Vec::new();
        let keywords = ["godot", "engine", "dev", "tool", "game", "program"];

        for letter in b'A'..=b'Z' {
            let drive = format!("{}:\\", letter as char);
            let drive_path = std::path::PathBuf::from(&drive);
            if !drive_path.exists() {
                continue;
            }

            if let Ok(entries) = std::fs::read_dir(&drive_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if !path.is_dir() {
                        continue;
                    }
                    let dir_name = path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_lowercase();

                    let matches_keyword = keywords.iter().any(|kw| dir_name.contains(kw));
                    if matches_keyword || Self::dir_contains_godot_executable(&path) {
                        result.push(path);
                    }
                }
            }
        }

        result
    }

    #[cfg(not(windows))]
    fn scan_drive_root_dirs() -> Vec<std::path::PathBuf> {
        Vec::new()
    }

    fn get_search_directories() -> Vec<std::path::PathBuf> {
        let mut dirs = Vec::new();

        if cfg!(windows) {
            if let Some(program_files) = std::env::var("ProgramFiles").ok() {
                dirs.push(std::path::PathBuf::from(&program_files));
                dirs.push(std::path::PathBuf::from(&program_files).join("Godot"));
                dirs.push(std::path::PathBuf::from(&program_files).join("Tools"));
            }
            if let Some(program_files_x86) = std::env::var("ProgramFiles(x86)").ok() {
                dirs.push(std::path::PathBuf::from(&program_files_x86));
                dirs.push(std::path::PathBuf::from(&program_files_x86).join("Godot"));
                dirs.push(std::path::PathBuf::from(&program_files_x86).join("Tools"));
            }
            if let Some(local_app_data) = std::env::var("LOCALAPPDATA").ok() {
                dirs.push(std::path::PathBuf::from(&local_app_data).join("Programs"));
                dirs.push(std::path::PathBuf::from(&local_app_data).join("Godot"));
            }
            if let Some(userprofile) = std::env::var("USERPROFILE").ok() {
                dirs.push(std::path::PathBuf::from(&userprofile).join("Downloads"));
                dirs.push(std::path::PathBuf::from(&userprofile).join("Desktop"));
                dirs.push(std::path::PathBuf::from(&userprofile).join("Godot"));
                dirs.push(std::path::PathBuf::from(&userprofile).join("Tools"));
                dirs.push(std::path::PathBuf::from(&userprofile).join("Programs"));
            }

            let drive_subdirs = Self::scan_drive_root_dirs();
            dirs.extend(drive_subdirs);
        } else if cfg!(target_os = "macos") {
            dirs.push(std::path::PathBuf::from("/Applications"));
            if let Some(home) = std::env::var("HOME").ok() {
                dirs.push(std::path::PathBuf::from(&home).join("Applications"));
                dirs.push(std::path::PathBuf::from(&home).join("Downloads"));
                dirs.push(std::path::PathBuf::from(&home).join("Godot"));
                dirs.push(std::path::PathBuf::from(&home).join("Tools"));
            }
            dirs.push(std::path::PathBuf::from("/usr/local/bin"));
        } else {
            if let Some(home) = std::env::var("HOME").ok() {
                dirs.push(std::path::PathBuf::from(&home).join(".local").join("bin"));
                dirs.push(std::path::PathBuf::from(&home).join("Downloads"));
                dirs.push(std::path::PathBuf::from(&home).join("bin"));
                dirs.push(std::path::PathBuf::from(&home).join(".godot"));
                dirs.push(std::path::PathBuf::from(&home).join("Godot"));
                dirs.push(std::path::PathBuf::from(&home).join("Tools"));
            }
            dirs.push(std::path::PathBuf::from("/usr/local/bin"));
            dirs.push(std::path::PathBuf::from("/usr/bin"));
            dirs.push(std::path::PathBuf::from("/opt"));
            dirs.push(std::path::PathBuf::from("/opt/godot"));
        }

        dirs
    }
}
