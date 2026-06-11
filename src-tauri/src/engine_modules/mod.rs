use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use crate::models::Engine;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModuleType {
    DotNet,
    Android,
    IOS,
    Web,
    Linux,
    Windows,
    MacOS,
    Editor,
}

impl std::fmt::Display for ModuleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModuleType::DotNet => write!(f, ".NET"),
            ModuleType::Android => write!(f, "Android"),
            ModuleType::IOS => write!(f, "iOS"),
            ModuleType::Web => write!(f, "Web"),
            ModuleType::Linux => write!(f, "Linux"),
            ModuleType::Windows => write!(f, "Windows"),
            ModuleType::MacOS => write!(f, "macOS"),
            ModuleType::Editor => write!(f, "Editor"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineModule {
    pub module_type: ModuleType,
    pub version: String,
    pub is_installed: bool,
    pub install_path: Option<String>,
    pub file_size: Option<u64>,
    pub last_updated: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineModulesInfo {
    pub engine_id: String,
    pub engine_version: String,
    pub modules: Vec<EngineModule>,
    pub missing_for_project: Vec<ModuleType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleInstallProgress {
    pub module_type: ModuleType,
    pub version: String,
    pub stage: String,
    pub progress: f64,
    pub message: String,
}

fn get_godot_templates_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        let appdata = std::env::var("APPDATA").unwrap_or_else(|_| {
            let home = std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\".to_string());
            format!("{}\\AppData\\Roaming", home)
        });
        PathBuf::from(appdata).join("Godot").join("export_templates")
    }
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(home).join("Library").join("Application Support").join("Godot").join("export_templates")
    }
    #[cfg(target_os = "linux")]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let data_dir = std::env::var("XDG_DATA_HOME").unwrap_or_else(|_| format!("{}/.local/share", home));
        PathBuf::from(data_dir).join("godot").join("export_templates")
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(home).join(".local").join("share").join("godot").join("export_templates")
    }
}

fn get_template_version_dir(version: &str) -> PathBuf {
    let stable_version = version.split('-').next().unwrap_or(version);
    get_godot_templates_dir().join(format!("{}.stable", stable_version))
}

fn is_godot4(version: &str) -> bool {
    crate::utils::is_godot4(version)
}

fn file_exists_in_dir(dir: &PathBuf, pattern: &str) -> bool {
    if !dir.exists() {
        return false;
    }
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.contains(pattern) {
                return true;
            }
        }
    }
    false
}

fn dir_size_fast(path: &PathBuf) -> Option<u64> {
    if !path.exists() {
        return None;
    }
    let mut total: u64 = 0;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let _entry_path = entry.path();
            if let Ok(meta) = entry.metadata() {
                if meta.is_file() {
                    total += meta.len();
                }
            }
        }
    }
    if total > 0 { Some(total) } else { None }
}

fn get_file_modified_time(path: &PathBuf) -> Option<String> {
    if !path.exists() {
        return None;
    }
    if let Ok(meta) = fs::metadata(path) {
        if let Ok(modified) = meta.modified() {
            let datetime: chrono::DateTime<chrono::Utc> = modified.into();
            return Some(datetime.to_rfc3339());
        }
    }
    None
}

pub fn detect_installed_modules(engine: &Engine) -> Vec<EngineModule> {
    let version = &engine.version;
    let template_dir = get_template_version_dir(version);
    let is_g4 = is_godot4(version);

    let mut modules = Vec::new();

    // .NET module
    let dotnet_installed = engine.is_mono || {
        let engine_path = PathBuf::from(&engine.path);
        let godotsharp_dir = engine_path.join("GodotSharp");
        godotsharp_dir.exists()
    };
    modules.push(EngineModule {
        module_type: ModuleType::DotNet,
        version: version.clone(),
        is_installed: dotnet_installed,
        install_path: if dotnet_installed { Some(engine.path.clone()) } else { None },
        file_size: None,
        last_updated: None,
    });

    // Windows export template
    let windows_pattern = if is_g4 { "windows_release" } else { "windows_64_release" };
    let windows_installed = file_exists_in_dir(&template_dir, windows_pattern);
    modules.push(EngineModule {
        module_type: ModuleType::Windows,
        version: version.clone(),
        is_installed: windows_installed,
        install_path: if windows_installed { Some(template_dir.to_string_lossy().to_string()) } else { None },
        file_size: if windows_installed { dir_size_fast(&template_dir) } else { None },
        last_updated: if windows_installed { get_file_modified_time(&template_dir) } else { None },
    });

    // Linux export template
    let linux_pattern = if is_g4 { "linux_release" } else { "linux_64_release" };
    let linux_installed = file_exists_in_dir(&template_dir, linux_pattern);
    modules.push(EngineModule {
        module_type: ModuleType::Linux,
        version: version.clone(),
        is_installed: linux_installed,
        install_path: if linux_installed { Some(template_dir.to_string_lossy().to_string()) } else { None },
        file_size: if linux_installed { dir_size_fast(&template_dir) } else { None },
        last_updated: if linux_installed { get_file_modified_time(&template_dir) } else { None },
    });

    // macOS export template
    let macos_pattern = if is_g4 { "macos" } else { "osx" };
    let macos_installed = file_exists_in_dir(&template_dir, macos_pattern);
    modules.push(EngineModule {
        module_type: ModuleType::MacOS,
        version: version.clone(),
        is_installed: macos_installed,
        install_path: if macos_installed { Some(template_dir.to_string_lossy().to_string()) } else { None },
        file_size: if macos_installed { dir_size_fast(&template_dir) } else { None },
        last_updated: if macos_installed { get_file_modified_time(&template_dir) } else { None },
    });

    // Web export template
    let web_pattern = "web_release";
    let web_installed = file_exists_in_dir(&template_dir, web_pattern);
    modules.push(EngineModule {
        module_type: ModuleType::Web,
        version: version.clone(),
        is_installed: web_installed,
        install_path: if web_installed { Some(template_dir.to_string_lossy().to_string()) } else { None },
        file_size: if web_installed { dir_size_fast(&template_dir) } else { None },
        last_updated: if web_installed { get_file_modified_time(&template_dir) } else { None },
    });

    // Android export template
    let android_installed = file_exists_in_dir(&template_dir, "android_release")
        || file_exists_in_dir(&template_dir, "android_source");
    modules.push(EngineModule {
        module_type: ModuleType::Android,
        version: version.clone(),
        is_installed: android_installed,
        install_path: if android_installed { Some(template_dir.to_string_lossy().to_string()) } else { None },
        file_size: if android_installed { dir_size_fast(&template_dir) } else { None },
        last_updated: if android_installed { get_file_modified_time(&template_dir) } else { None },
    });

    // iOS export template
    let ios_installed = file_exists_in_dir(&template_dir, "ios_release")
        || file_exists_in_dir(&template_dir, "ios_source");
    modules.push(EngineModule {
        module_type: ModuleType::IOS,
        version: version.clone(),
        is_installed: ios_installed,
        install_path: if ios_installed { Some(template_dir.to_string_lossy().to_string()) } else { None },
        file_size: if ios_installed { dir_size_fast(&template_dir) } else { None },
        last_updated: if ios_installed { get_file_modified_time(&template_dir) } else { None },
    });

    // Editor module (always installed if engine exists)
    modules.push(EngineModule {
        module_type: ModuleType::Editor,
        version: version.clone(),
        is_installed: true,
        install_path: Some(engine.path.clone()),
        file_size: None,
        last_updated: None,
    });

    modules
}

pub fn detect_all_engines_modules(engines: &[Engine]) -> Vec<EngineModulesInfo> {
    engines.iter().map(|engine| {
        let modules = detect_installed_modules(engine);
        EngineModulesInfo {
            engine_id: engine.engine_id.clone(),
            engine_version: engine.version.clone(),
            modules,
            missing_for_project: Vec::new(),
        }
    }).collect()
}

pub fn get_modules_needed_by_project(project_path: &str) -> Vec<ModuleType> {
    let export_presets_path = PathBuf::from(project_path).join("export_presets.cfg");
    if !export_presets_path.exists() {
        return Vec::new();
    }

    let content = match fs::read_to_string(&export_presets_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut needed: Vec<ModuleType> = Vec::new();
    let mut has_dotnet = false;

    // Check for .csproj or .sln files indicating C# project
    if let Ok(entries) = fs::read_dir(project_path) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string().to_lowercase();
            if name.ends_with(".csproj") || name.ends_with(".sln") {
                has_dotnet = true;
                break;
            }
        }
    }

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("platform=") {
            let platform = trimmed.strip_prefix("platform=").unwrap_or("").trim_matches('"').to_string();
            match platform.as_str() {
                "Windows Desktop" => {
                    if !needed.contains(&ModuleType::Windows) {
                        needed.push(ModuleType::Windows);
                    }
                }
                "Linux/X11" => {
                    if !needed.contains(&ModuleType::Linux) {
                        needed.push(ModuleType::Linux);
                    }
                }
                "macOS" => {
                    if !needed.contains(&ModuleType::MacOS) {
                        needed.push(ModuleType::MacOS);
                    }
                }
                "Web" | "HTML5" => {
                    if !needed.contains(&ModuleType::Web) {
                        needed.push(ModuleType::Web);
                    }
                }
                "Android" => {
                    if !needed.contains(&ModuleType::Android) {
                        needed.push(ModuleType::Android);
                    }
                }
                "iOS" => {
                    if !needed.contains(&ModuleType::IOS) {
                        needed.push(ModuleType::IOS);
                    }
                }
                _ => {}
            }
        }
    }

    if has_dotnet && !needed.contains(&ModuleType::DotNet) {
        needed.push(ModuleType::DotNet);
    }

    needed
}

pub fn check_missing_modules(engine: &Engine, project_path: &str) -> Vec<ModuleType> {
    let needed = get_modules_needed_by_project(project_path);
    let installed = detect_installed_modules(engine);

    needed.into_iter().filter(|module_type| {
        !installed.iter().any(|m| m.module_type == *module_type && m.is_installed)
    }).collect()
}

pub fn get_module_download_url(module_type: &ModuleType, version: &str, is_mono: bool) -> Result<String, String> {
    let _stable_version = version.split('-').next().unwrap_or(version);

    match module_type {
        ModuleType::DotNet => {
            // .NET module is essentially the mono variant of the engine
            let variant = if is_mono { "" } else { "_mono" };
            Ok(format!(
                "https://github.com/godotengine/godot/releases/download/{}/Godot_v{}{}_export_templates.tpz",
                version, version, variant
            ))
        }
        ModuleType::Editor => {
            Err("编辑器模块无需单独下载".to_string())
        }
        _ => {
            // All export templates come as a single .tpz file
            let mono_suffix = if is_mono { "_mono" } else { "" };
            Ok(format!(
                "https://github.com/godotengine/godot/releases/download/{}/Godot_v{}{}_export_templates.tpz",
                version, version, mono_suffix
            ))
        }
    }
}
