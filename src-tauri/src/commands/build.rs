use crate::commands::utils::{get_storage, load_settings};
use crate::models::*;
use crate::utils::create_http_client;
use tauri::{AppHandle, Emitter};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const GODOT_EXPORT_TEMPLATES_URL: &str = "https://downloads.tuxfamily.org/godotengine";
const GITHUB_RELEASES_API: &str = "https://api.github.com/repos/godotengine/godot-builds/releases";

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

fn get_godot_template_version_dir(version: &str) -> PathBuf {
    let stable_version = version
        .split('-')
        .next()
        .unwrap_or(version);
    get_godot_templates_dir().join(format!("{}.stable", stable_version))
}

fn extract_tpz_to_godot_dir(tpz_path: &Path, version: &str, _mono: bool) -> Result<(), String> {
    let target_dir = get_godot_template_version_dir(version);
    let is_update = target_dir.exists();
    let backup_dir = if is_update {
        let backup = target_dir.with_extension("stable.bak");
        if backup.exists() {
            let _ = fs::remove_dir_all(&backup);
        }
        fs::rename(&target_dir, &backup)
            .map_err(|e| format!("备份旧模板目录失败: {}", e))?;
        Some(backup)
    } else {
        None
    };

    fs::create_dir_all(&target_dir)
        .map_err(|e| format!("创建 Godot 模板目录 {} 失败: {}", target_dir.display(), e))?;

    let file = fs::File::open(tpz_path)
        .map_err(|e| format!("打开 tpz 文件失败: {}", e))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| {
            if let Some(ref backup) = backup_dir {
                let _ = fs::remove_dir_all(&target_dir);
                let _ = fs::rename(backup, &target_dir);
            }
            format!("解析 tpz 文件失败: {}", e)
        })?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)
            .map_err(|e| format!("读取 zip 条目失败: {}", e))?;

        let entry_path = entry.name().to_string();
        let parts: Vec<&str> = entry_path.splitn(2, '/').collect();

        let relative_path = if parts.len() > 1 {
            parts[1].to_string()
        } else {
            continue;
        };

        if relative_path.is_empty() {
            continue;
        }

        let out_path = target_dir.join(&relative_path);

        if entry.is_dir() {
            fs::create_dir_all(&out_path)
                .map_err(|e| format!("创建目录 {} 失败: {}", out_path.display(), e))?;
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("创建父目录 {} 失败: {}", parent.display(), e))?;
            }
            let mut outfile = fs::File::create(&out_path)
                .map_err(|e| format!("创建文件 {} 失败: {}", out_path.display(), e))?;
            std::io::copy(&mut entry, &mut outfile)
                .map_err(|e| format!("写入文件 {} 失败: {}", out_path.display(), e))?;
        }
    }

    if let Some(ref backup) = backup_dir {
        let _ = fs::remove_dir_all(backup);
    }

    Ok(())
}

fn platform_to_godot_platform(platform: &ExportPlatform) -> &'static str {
    match platform {
        ExportPlatform::Windows => "Windows Desktop",
        ExportPlatform::MacOS => "macOS",
        ExportPlatform::Linux => "Linux/X11",
        ExportPlatform::Web => "Web",
        ExportPlatform::Android => "Android",
        ExportPlatform::IOS => "iOS",
    }
}

fn find_preset_name_for_platform(export_presets_path: &Path, platform: &ExportPlatform) -> Result<String, String> {
    let content = fs::read_to_string(export_presets_path)
        .map_err(|e| format!("读取 export_presets.cfg 失败: {}", e))?;

    let godot_platform = platform_to_godot_platform(platform);

    let mut current_preset_name: Option<String> = None;
    let mut current_platform: Option<String> = None;
    let mut found_presets: Vec<(String, String)> = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("name=") {
            let name = trimmed.strip_prefix("name=").unwrap_or("").trim_matches('"').to_string();
            current_preset_name = Some(name);
        } else if trimmed.starts_with("platform=") {
            let plat = trimmed.strip_prefix("platform=").unwrap_or("").trim_matches('"').to_string();
            current_platform = Some(plat);
        } else if trimmed.starts_with('[') && current_preset_name.is_some() && current_platform.is_some() {
            found_presets.push((current_preset_name.take().unwrap(), current_platform.take().unwrap()));
        }
    }
    if current_preset_name.is_some() && current_platform.is_some() {
        found_presets.push((current_preset_name.take().unwrap(), current_platform.take().unwrap()));
    }

    for (name, plat) in &found_presets {
        if plat == godot_platform {
            return Ok(name.clone());
        }
    }

    let available: Vec<String> = found_presets.iter().map(|(n, p)| format!("{} ({})", n, p)).collect();
    Err(format!(
        "未找到 {} 平台的导出预设，可用预设: {}",
        godot_platform,
        if available.is_empty() { "无".to_string() } else { available.join(", ") }
    ))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportTemplateInfo {
    pub version: String,
    pub mono: bool,
    pub installed: bool,
    pub path: Option<String>,
    pub file_size: Option<u64>,
}

#[tauri::command]
pub async fn list_export_templates(app: AppHandle) -> Result<Vec<ExportTemplateInfo>, String> {
    let app_clone = app.clone();
    tokio::task::spawn_blocking(move || {
        let engines: Vec<Engine> = get_storage(&app_clone).load_or_default("engines.json");
        let godot_templates_dir = get_godot_templates_dir();
        let mut result = Vec::new();

        if godot_templates_dir.exists() {
            if let Ok(entries) = fs::read_dir(&godot_templates_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if !path.is_dir() {
                        continue;
                    }
                    let dir_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                    let version = dir_name.trim_end_matches(".stable").to_string();
                    if version.is_empty() {
                        continue;
                    }

                    let has_templates = path.join("templates").exists()
                        && fs::read_dir(path.join("templates"))
                            .map(|mut e| e.any(|_| true))
                            .unwrap_or(false);

                    let mono = dir_name.contains("mono");
                    let total_size = dir_size_recursive(&path);

                    result.push(ExportTemplateInfo {
                        version: version.clone(),
                        mono,
                        installed: has_templates,
                        path: Some(path.to_string_lossy().to_string()),
                        file_size: if total_size > 0 { Some(total_size) } else { None },
                    });
                }
            }
        }

        for engine in &engines {
            if !result.iter().any(|t| t.version == engine.version && t.mono == engine.is_mono) {
                let version_dir = get_godot_template_version_dir(&engine.version);
                let installed = version_dir.exists();
                result.push(ExportTemplateInfo {
                    version: engine.version.clone(),
                    mono: engine.is_mono,
                    installed,
                    path: if installed { Some(version_dir.to_string_lossy().to_string()) } else { None },
                    file_size: None,
                });
            }
        }

        result.sort_by(|a, b| {
            match (a.version.parse::<semver::Version>(), b.version.parse::<semver::Version>()) {
                (Ok(av), Ok(bv)) => bv.cmp(&av).then(a.mono.cmp(&b.mono)),
                _ => b.version.cmp(&a.version),
            }
        });

        Ok(result)
    }).await.map_err(|e| format!("任务执行失败: {}", e))?
}

fn dir_size_recursive(path: &Path) -> u64 {
    let mut total_size: u64 = 0;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_dir() {
                total_size += dir_size_recursive(&entry_path);
            } else if let Ok(meta) = entry.metadata() {
                if meta.is_file() {
                    total_size += meta.len();
                }
            }
        }
    }
    total_size
}

async fn resolve_template_download_url(app: &AppHandle, version: &str, mono: bool) -> Result<String, String> {
    let settings = load_settings(app);
    let mirror = settings.engine_mirrors.iter().find(|m| m.enabled);

    if let Some(mirror) = mirror {
        if mirror.mirror_type == "direct" && !mirror.is_official {
            let tpz_name = if mono {
                format!("Godot_v{}_mono_export_templates.tpz", version)
            } else {
                format!("Godot_v{}_export_templates.tpz", version)
            };
            let mirror_url = format!("{}/{}/{}",
                mirror.base_url.trim_end_matches('/'),
                version,
                tpz_name
            );
            let client = create_http_client(Some(std::time::Duration::from_secs(10)))?;
            if client.head(&mirror_url).send().await.is_ok() {
                return Ok(mirror_url);
            }
        }
    }

    let stable_version = version.split('-').next().unwrap_or(version);
    let tuxfamily_urls: Vec<String> = if mono {
        vec![
            format!("{}/{}/mono/Godot_v{}_mono_export_templates.tpz", GODOT_EXPORT_TEMPLATES_URL, stable_version, version),
            format!("{}/{}/mono/Godot_v{}_stable_mono_export_templates.tpz", GODOT_EXPORT_TEMPLATES_URL, stable_version, stable_version),
        ]
    } else {
        vec![
            format!("{}/{}/Godot_v{}_export_templates.tpz", GODOT_EXPORT_TEMPLATES_URL, stable_version, version),
            format!("{}/{}/Godot_v{}_stable_export_templates.tpz", GODOT_EXPORT_TEMPLATES_URL, stable_version, stable_version),
        ]
    };

    let client = create_http_client(Some(std::time::Duration::from_secs(15)))?;
    for url in &tuxfamily_urls {
        match client.head(url).send().await {
            Ok(resp) if resp.status().is_success() => return Ok(url.clone()),
            _ => continue,
        }
    }

    let github_url = find_template_on_github(&client, version, mono).await?;
    Ok(github_url)
}

async fn find_template_on_github(client: &reqwest::Client, version: &str, mono: bool) -> Result<String, String> {
    let tag_prefix = format!("v{}-", version.split('-').next().unwrap_or(version));
    let tpz_keyword = if mono { "mono_export_templates.tpz" } else { "export_templates.tpz" };

    for page in 1..=5 {
        let url = format!("{}?per_page=50&page={}", GITHUB_RELEASES_API, page);
        let resp = client.get(&url)
            .header("Accept", "application/vnd.github+json")
            .send().await
            .map_err(|e| format!("访问 GitHub Releases 失败: {}", e))?;

        if !resp.status().is_success() {
            break;
        }

        let releases: Vec<serde_json::Value> = resp.json().await
            .map_err(|e| format!("解析 GitHub Releases 失败: {}", e))?;

        if releases.is_empty() {
            break;
        }

        for release in &releases {
            let tag = release.get("tag_name").and_then(|t| t.as_str()).unwrap_or("");
            if !tag.starts_with(&tag_prefix) && tag != format!("v{}", version) {
                continue;
            }

            if let Some(assets) = release.get("assets").and_then(|a| a.as_array()) {
                for asset in assets {
                    let name = asset.get("name").and_then(|n| n.as_str()).unwrap_or("");
                    let download_url = asset.get("browser_download_url")
                        .and_then(|u| u.as_str()).unwrap_or("");

                    if name.contains(tpz_keyword) {
                        return Ok(download_url.to_string());
                    }
                }
            }
        }
    }

    Err("无法找到导出模板下载地址，请检查网络连接或配置镜像源".to_string())
}

#[tauri::command]
pub async fn download_export_template(app: AppHandle, version: String, mono: bool) -> Result<String, String> {
    let _ = app.emit("export-template-download-progress", serde_json::json!({
        "version": &version,
        "stage": "downloading",
        "progress": 0.0,
        "message": format!("正在下载 Godot {} 导出模板...", version),
    }));

    let temp_dir = std::env::temp_dir().join(format!("godot_harbor_template_{}", version.replace('.', "_")));
    fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("创建临时目录失败: {}", e))?;

    let download_url = resolve_template_download_url(&app, &version, mono).await?;

    let client = create_http_client(Some(std::time::Duration::from_secs(300)))?;
    let resp = client.get(&download_url).send().await
        .map_err(|e| format!("下载导出模板失败: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("下载导出模板失败: HTTP {}", resp.status()));
    }

    let tpz_path = temp_dir.join("templates.tpz");
    let mut file = fs::File::create(&tpz_path)
        .map_err(|e| format!("创建临时文件失败: {}", e))?;

    use std::io::Write;
    use futures::StreamExt;
    let total_size: u64 = resp.headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let mut stream = resp.bytes_stream();
    let mut downloaded: u64 = 0;
    let mut last_emit: std::time::Instant = std::time::Instant::now();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("下载中断: {}", e))?;
        file.write_all(&chunk).map_err(|e| format!("写入文件失败: {}", e))?;
        downloaded += chunk.len() as u64;
        if last_emit.elapsed() >= std::time::Duration::from_millis(500) {
            let progress = if total_size > 0 { downloaded as f64 / total_size as f64 * 0.7 } else { 0.3 };
            let _ = app.emit("export-template-download-progress", serde_json::json!({
                "version": &version,
                "stage": "downloading",
                "progress": progress,
                "message": format!("正在下载 Godot {} 导出模板... {}%", version, if total_size > 0 { (downloaded as f64 / total_size as f64 * 100.0) as u32 } else { 0 }),
            }));
            last_emit = std::time::Instant::now();
        }
    }

    let _ = app.emit("export-template-download-progress", serde_json::json!({
        "version": &version,
        "stage": "extracting",
        "progress": 0.8,
        "message": format!("正在解压 Godot {} 导出模板到标准路径...", version),
    }));

    if let Err(e) = extract_tpz_to_godot_dir(&tpz_path, &version, mono) {
        let _ = fs::remove_dir_all(&temp_dir);
        let _ = app.emit("export-template-download-progress", serde_json::json!({
            "version": &version,
            "stage": "failed",
            "progress": 1.0,
            "message": format!("解压导出模板失败: {}", e),
        }));
        return Err(e);
    }

    let _ = fs::remove_dir_all(&temp_dir);

    let _ = app.emit("export-template-download-progress", serde_json::json!({
        "version": &version,
        "stage": "complete",
        "progress": 1.0,
        "message": format!("Godot {} 导出模板安装完成", version),
    }));

    Ok(format!("导出模板 {} 安装完成", version))
}

#[tauri::command]
pub async fn import_export_template_from_file(app: AppHandle, tpz_path: String, version: String, mono: bool) -> Result<String, String> {
    let path = Path::new(&tpz_path);
    if !path.exists() {
        return Err(format!("文件不存在: {}", tpz_path));
    }
    if !tpz_path.to_lowercase().ends_with(".tpz") && !tpz_path.to_lowercase().ends_with(".zip") {
        return Err("请选择 .tpz 或 .zip 格式的导出模板文件".to_string());
    }

    let _ = app.emit("export-template-download-progress", serde_json::json!({
        "version": &version,
        "stage": "extracting",
        "progress": 0.5,
        "message": format!("正在从本地文件导入 Godot {} 导出模板...", version),
    }));

    let tpz_path_owned = tpz_path.clone();
    let version_owned = version.clone();
    tokio::task::spawn_blocking(move || {
        let path = Path::new(&tpz_path_owned);
        extract_tpz_to_godot_dir(path, &version_owned, mono)
    }).await.map_err(|e| format!("任务执行失败: {}", e))??;

    let _ = app.emit("export-template-download-progress", serde_json::json!({
        "version": &version,
        "stage": "complete",
        "progress": 1.0,
        "message": format!("Godot {} 导出模板导入完成", version),
    }));

    Ok(format!("导出模板 {} 导入完成", version))
}

#[tauri::command]
pub async fn delete_export_template(_app: AppHandle, version: String, _mono: bool) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let version_dir = get_godot_template_version_dir(&version);

        if version_dir.exists() {
            fs::remove_dir_all(&version_dir)
                .map_err(|e| format!("删除导出模板失败: {}", e))?;
        }

        Ok(())
    }).await.map_err(|e| format!("任务执行失败: {}", e))?
}

#[tauri::command]
pub async fn list_export_presets(app: AppHandle, project_id: String) -> Result<Vec<ExportPreset>, String> {
    let app_clone = app.clone();
    tokio::task::spawn_blocking(move || {
        let storage = get_storage(&app_clone);
        let projects: Vec<Project> = storage.load_or_default("projects.json");
        let project = projects.iter().find(|p| p.project_id == project_id)
            .ok_or("项目不存在".to_string())?;

        let config = crate::harbor_config::read_harbor_config_from_project(&project.path)
            .map_err(|e| format!("读取 .harbor.yml 失败: {}", e))?;

        let mut presets = Vec::new();
        if let Some(config) = config {
            let config_upgraded = if config.version < 2 { config.upgrade_to_v2() } else { config };
            for ep in &config_upgraded.export_presets {
                let platform = match ep.platform.as_str() {
                    "windows" => ExportPlatform::Windows,
                    "macos" => ExportPlatform::MacOS,
                    "linux" => ExportPlatform::Linux,
                    "web" => ExportPlatform::Web,
                    "android" => ExportPlatform::Android,
                    "ios" => ExportPlatform::IOS,
                    _ => ExportPlatform::Windows,
                };
                presets.push(ExportPreset {
                    preset_id: format!("{}-{}", ep.platform, ep.name.to_lowercase().replace(' ', "-")),
                    platform,
                    name: ep.name.clone(),
                    config: ep.config.clone(),
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                });
            }
        }

        Ok(presets)
    }).await.map_err(|e| format!("任务执行失败: {}", e))?
}

#[tauri::command]
pub async fn apply_export_preset(app: AppHandle, project_id: String, preset: ExportPreset) -> Result<(), String> {
    let app_clone = app.clone();
    tokio::task::spawn_blocking(move || {
        let storage = get_storage(&app_clone);
        let projects: Vec<Project> = storage.load_or_default("projects.json");
        let project = projects.iter().find(|p| p.project_id == project_id)
            .ok_or("项目不存在".to_string())?;

        let export_cfg_path = Path::new(&project.path).join("export_presets.cfg");
        let platform_str = preset.platform.to_string();

        let mut content = if export_cfg_path.exists() {
            fs::read_to_string(&export_cfg_path)
                .map_err(|e| format!("读取 export_presets.cfg 失败: {}", e))?
        } else {
            "[preset.0]\nname=\"\"\nplatform=\"\"\nrunnable=true\n".to_string()
        };

        let preset_index = count_presets(&content);
        let new_preset = format!(
            "\n[preset.{}]\nname=\"{}\"\nplatform=\"{}\"\nrunnable=true\ncustom_features=\"\"\nexport_filter=\"all_resources\"\ninclude_filter=\"\"\nexclude_filter=\"\"\nexport_path=\"\"\nscript_encryption_key=\"\"\n",
            preset_index, preset.name, platform_str
        );

        if !content.ends_with('\n') {
            content.push('\n');
        }
        content.push_str(&new_preset);

        fs::write(&export_cfg_path, content)
            .map_err(|e| format!("写入 export_presets.cfg 失败: {}", e))?;

        Ok(())
    }).await.map_err(|e| format!("任务执行失败: {}", e))?
}

fn count_presets(content: &str) -> usize {
    let re = regex::Regex::new(r#"\[preset\.(\d+)\]"#).unwrap();
    let max_idx = re.captures_iter(content)
        .filter_map(|c| c[1].parse::<usize>().ok())
        .max()
        .map_or(0, |m| m + 1);
    max_idx
}

#[tauri::command]
pub async fn save_export_preset_to_harbor(app: AppHandle, project_id: String, platform: String, name: String, config: serde_json::Value) -> Result<(), String> {
    let app_clone = app.clone();
    tokio::task::spawn_blocking(move || {
        let storage = get_storage(&app_clone);
        let projects: Vec<Project> = storage.load_or_default("projects.json");
        let project = projects.iter().find(|p| p.project_id == project_id)
            .ok_or("项目不存在".to_string())?;

        let harbor_config_path = crate::harbor_config::get_harbor_config_path(&project.path);
        if !harbor_config_path.exists() {
            return Err("项目缺少 .harbor.yml".to_string());
        }

        let config_existing = crate::harbor_config::read_harbor_config_from_project(&project.path)
            .map_err(|e| format!("读取 .harbor.yml 失败: {}", e))?
            .ok_or("读取 .harbor.yml 失败".to_string())?;

        let mut config_upgraded = if config_existing.version < 2 { config_existing.upgrade_to_v2() } else { config_existing };
        config_upgraded.export_presets.push(crate::harbor_config::HarborExportPreset {
            platform: platform.clone(),
            name: name.clone(),
            config,
        });

        crate::harbor_config::write_harbor_config_to_project(&project.path, &config_upgraded)
            .map_err(|e| format!("写入 .harbor.yml 失败: {}", e))?;

        Ok(())
    }).await.map_err(|e| format!("任务执行失败: {}", e))?
}

#[tauri::command]
pub async fn build_project(app: AppHandle, project_id: String, platform: ExportPlatform, preset_name: Option<String>) -> Result<BuildRecord, String> {
    let storage = get_storage(&app);
    let projects: Vec<Project> = storage.load_or_default("projects.json");
    let project = projects.iter().find(|p| p.project_id == project_id)
        .ok_or("项目不存在".to_string())?;

    let engines: Vec<Engine> = storage.load_or_default("engines.json");
    let engine = engines.iter().find(|e| {
        project.last_used_engine_id.as_ref().map_or(false, |id| &e.engine_id == id)
    }).or_else(|| engines.iter().find(|e| {
        let ev: Vec<&str> = e.version.split('.').collect();
        let pv: Vec<&str> = project.godot_version.split('.').collect();
        ev.len() >= 2 && pv.len() >= 2 && ev[0] == pv[0] && ev[1] == pv[1]
    })).ok_or("未找到匹配的引擎".to_string())?;

    let build_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now();

    let _ = app.emit("build-progress", serde_json::json!({
        "build_id": &build_id,
        "stage": "starting",
        "progress": 0.0,
        "message": format!("正在准备构建 {} ({} {})...", project.name, platform, engine.version),
    }));

    let engine_path = PathBuf::from(&engine.path);

    let godot_bin = if engine_path.is_file() {
        engine_path.clone()
    } else {
        crate::engine::EngineManager::find_executable_in_dir(&engine_path)
            .ok_or_else(|| format!("在 {} 中未找到 Godot 可执行文件", engine_path.display()))?
    };

    if !godot_bin.exists() {
        let _ = app.emit("build-progress", serde_json::json!({
            "build_id": &build_id,
            "stage": "failed",
            "progress": 0.0,
            "message": format!("引擎可执行文件不存在: {}", godot_bin.display()),
        }));
        return Err(format!("引擎可执行文件不存在: {}", godot_bin.display()));
    }

    let template_dir = get_godot_template_version_dir(&engine.version);
    if !template_dir.exists() {
        let _ = app.emit("build-progress", serde_json::json!({
            "build_id": &build_id,
            "stage": "failed",
            "progress": 0.0,
            "message": format!("导出模板未安装: Godot {}，请先在\"导出模板\"页下载", engine.version),
        }));
        return Err(format!("导出模板未安装: Godot {}，请先在\"导出模板\"页下载", engine.version));
    }

    let output_dir = PathBuf::from(&project.path).join("builds").join(platform.to_string());
    fs::create_dir_all(&output_dir)
        .map_err(|e| format!("创建输出目录失败: {}", e))?;

    let export_presets_path = PathBuf::from(&project.path).join("export_presets.cfg");
    if !export_presets_path.exists() {
        let _ = app.emit("build-progress", serde_json::json!({
            "build_id": &build_id,
            "stage": "failed",
            "progress": 0.0,
            "message": "项目缺少 export_presets.cfg，请先在 Godot 编辑器中配置导出预设".to_string(),
        }));
        return Err("项目缺少 export_presets.cfg，请先在 Godot 编辑器中配置导出预设".to_string());
    }

    let preset_name = match preset_name {
        Some(name) => name,
        None => find_preset_name_for_platform(&export_presets_path, &platform)?,
    };

    let output_extension = match platform {
        ExportPlatform::Windows => ".exe",
        ExportPlatform::MacOS => ".app",
        ExportPlatform::Linux => "",
        ExportPlatform::Web => "",
        ExportPlatform::Android => ".apk",
        ExportPlatform::IOS => ".ipa",
    };
    let output_filename = format!("{}{}", project.name, output_extension);
    let output_path = output_dir.join(&output_filename);

    let _ = app.emit("build-progress", serde_json::json!({
        "build_id": &build_id,
        "stage": "building",
        "progress": 0.3,
        "message": format!("正在构建 {} ({} {})...", project.name, platform, engine.version),
    }));

    let build_result = tokio::time::timeout(
        std::time::Duration::from_secs(600),
        tokio::process::Command::new(&godot_bin)
            .arg("--headless")
            .arg("--path").arg(&project.path)
            .arg("--export-release")
            .arg(&preset_name)
            .arg(output_path.to_string_lossy().as_ref())
            .output()
    )
        .await
        .map_err(|_| {
            let _ = app.emit("build-progress", serde_json::json!({
                "build_id": &build_id,
                "stage": "failed",
                "progress": 1.0,
                "message": "构建超时（10分钟）".to_string(),
            }));
            "构建超时（10分钟）".to_string()
        })?
        .map_err(|e| format!("执行构建命令失败: {}", e))?;

    let duration = (chrono::Utc::now() - now).num_seconds() as u64;
    let success = build_result.status.success();
    let stdout = String::from_utf8_lossy(&build_result.stdout).to_string();
    let stderr = String::from_utf8_lossy(&build_result.stderr).to_string();

    let (status, error_message) = if success {
        let _ = app.emit("build-progress", serde_json::json!({
            "build_id": &build_id,
            "stage": "complete",
            "progress": 1.0,
            "message": format!("构建完成: {} ({})", project.name, platform),
        }));
        (BuildStatus::Success, String::new())
    } else {
        let err = format!("{}\n{}", stdout, stderr);
        let _ = app.emit("build-progress", serde_json::json!({
            "build_id": &build_id,
            "stage": "failed",
            "progress": 1.0,
            "message": format!("构建失败: {}", &err[..err.len().min(200)]),
        }));
        (BuildStatus::Failed, err)
    };

    let record = BuildRecord {
        build_id: build_id.clone(),
        project_id: project.project_id.clone(),
        project_name: project.name.clone(),
        platform: platform.clone(),
        engine_version: engine.version.clone(),
        status,
        started_at: now,
        completed_at: Some(chrono::Utc::now()),
        output_path: output_path.to_string_lossy().to_string(),
        error_message,
        duration_secs: duration,
    };

    let mut records: Vec<BuildRecord> = storage.load_or_default("build_records.json");
    records.push(record.clone());
    const MAX_RECORDS: usize = 200;
    if records.len() > MAX_RECORDS {
        records.drain(0..records.len() - MAX_RECORDS);
    }
    storage.save("build_records.json", &records)
        .map_err(|e| format!("保存构建记录失败: {}", e))?;

    Ok(record)
}

#[tauri::command]
pub async fn get_build_records(app: AppHandle, project_id: Option<String>) -> Result<Vec<BuildRecord>, String> {
    let app_clone = app.clone();
    tokio::task::spawn_blocking(move || {
        let storage = get_storage(&app_clone);
        let records: Vec<BuildRecord> = storage.load_or_default("build_records.json");
        match project_id {
            Some(pid) => Ok(records.into_iter().filter(|r| r.project_id == pid).collect()),
            None => Ok(records),
        }
    }).await.map_err(|e| format!("任务执行失败: {}", e))?
}

#[tauri::command]
pub async fn delete_build_record(app: AppHandle, build_id: String) -> Result<(), String> {
    let app_clone = app.clone();
    tokio::task::spawn_blocking(move || {
        let storage = get_storage(&app_clone);
        let mut records: Vec<BuildRecord> = storage.load_or_default("build_records.json");
        records.retain(|r| r.build_id != build_id);
        storage.save("build_records.json", &records)
            .map_err(|e| format!("保存构建记录失败: {}", e))?;
        Ok(())
    }).await.map_err(|e| format!("任务执行失败: {}", e))?
}

#[tauri::command]
pub async fn clear_all_build_records(app: AppHandle) -> Result<(), String> {
    let app_clone = app.clone();
    tokio::task::spawn_blocking(move || {
        let storage = get_storage(&app_clone);
        storage.save("build_records.json", &Vec::<BuildRecord>::new())
            .map_err(|e| format!("清除构建记录失败: {}", e))?;
        Ok(())
    }).await.map_err(|e| format!("任务执行失败: {}", e))?
}

#[tauri::command]
pub async fn generate_github_actions(app: AppHandle, project_id: String, platforms: Vec<String>, godot_version: String) -> Result<String, String> {
    let app_clone = app.clone();
    tokio::task::spawn_blocking(move || {
        let storage = get_storage(&app_clone);
        let projects: Vec<Project> = storage.load_or_default("projects.json");
        let project = projects.iter().find(|p| p.project_id == project_id)
            .ok_or("项目不存在".to_string())?;

        let project_name = project.name.clone();
        let safe_name = project_name.replace(' ', "-").replace(|c: char| !c.is_alphanumeric() && c != '-', "").to_lowercase();

        let mut matrix_entries = Vec::new();
        for p in &platforms {
            let runner = match p.as_str() {
                "macos" | "ios" => "macos-latest",
                _ => "ubuntu-latest",
            };
            matrix_entries.push(format!("          - platform: {}\n            runner: {}\n            artifact: {}-{}", p, runner, safe_name, p));
        }

        let workflow = format!(r#"name: Build Godot Project

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]
  workflow_dispatch:

env:
  GODOT_VERSION: "{}"

jobs:
  build:
    runs-on: ${{{{ matrix.runner }}}}
    strategy:
      matrix:
        include:
{}
    steps:
      - uses: actions/checkout@v4

      - name: Cache Godot
        uses: actions/cache@v4
        with:
          path: |
            ~/godot-bin
            ~/.local/share/godot/export_templates
          key: godot-${{{{ env.GODOT_VERSION }}}}-${{{{ runner.os }}}}

      - name: Download Godot
        run: |
          if [ ! -f ~/godot-bin/godot ]; then
            mkdir -p ~/godot-bin
            if [ "${{{{ runner.os }}}}" = "macOS" ]; then
              wget -q https://downloads.tuxfamily.org/godotengine/${{{{ env.GODOT_VERSION }}}}/Godot_v${{{{ env.GODOT_VERSION }}}}_macos.universal.zip
              unzip Godot_v${{{{ env.GODOT_VERSION }}}}_macos.universal.zip
              mv "Godot_v${{{{ env.GODOT_VERSION }}}}_macos.universal" ~/godot-bin/godot
            else
              wget -q https://downloads.tuxfamily.org/godotengine/${{{{ env.GODOT_VERSION }}}}/Godot_v${{{{ env.GODOT_VERSION }}}}_linux.x86_64.zip
              unzip Godot_v${{{{ env.GODOT_VERSION }}}}_linux.x86_64.zip
              mv Godot_v${{{{ env.GODOT_VERSION }}}}_linux.x86_64 ~/godot-bin/godot
            fi
            chmod +x ~/godot-bin/godot
          fi
          echo "$HOME/godot-bin" >> $GITHUB_PATH

      - name: Download Export Templates
        run: |
          if [ ! -d ~/.local/share/godot/export_templates/${{{{ env.GODOT_VERSION }}}}.stable/templates ]; then
            mkdir -p ~/.local/share/godot/export_templates/${{{{ env.GODOT_VERSION }}}}.stable
            wget -q https://downloads.tuxfamily.org/godotengine/${{{{ env.GODOT_VERSION }}}}/Godot_v${{{{ env.GODOT_VERSION }}}}_export_templates.tpz
            unzip Godot_v${{{{ env.GODOT_VERSION }}}}_export_templates.tpz -d ~/.local/share/godot/export_templates/${{{{ env.GODOT_VERSION }}}}.stable
          fi

      - name: Build Project
        run: |
          godot --headless --export-release ${{{{ matrix.platform }}}} ./build/${{{{ matrix.artifact }}}}

      - name: Upload Artifact
        uses: actions/upload-artifact@v4
        with:
          name: ${{{{ matrix.artifact }}}}
          path: ./build/${{{{ matrix.artifact }}}}
"#, godot_version, matrix_entries.join("\n"));

        Ok(workflow)
    }).await.map_err(|e| format!("任务执行失败: {}", e))?
}

#[tauri::command]
pub async fn generate_gitlab_ci(app: AppHandle, project_id: String, platforms: Vec<String>, godot_version: String) -> Result<String, String> {
    let app_clone = app.clone();
    tokio::task::spawn_blocking(move || {
        let storage = get_storage(&app_clone);
        let projects: Vec<Project> = storage.load_or_default("projects.json");
        let project = projects.iter().find(|p| p.project_id == project_id)
            .ok_or("项目不存在".to_string())?;

        let project_name = project.name.clone();
        let safe_name = project_name.replace(' ', "-").replace(|c: char| !c.is_alphanumeric() && c != '-', "").to_lowercase();

        fn platform_to_preset_name(platform: &str) -> String {
            match platform {
                "windows" => "Windows Desktop".to_string(),
                "macos" => "macOS".to_string(),
                "linux" => "Linux/X11".to_string(),
                "web" => "Web".to_string(),
                "android" => "Android".to_string(),
                "ios" => "iOS".to_string(),
                other => other.to_string(),
            }
        }

        let mut build_jobs = String::new();
        for p in &platforms {
            let job_name = format!("build_{}", p);
            let preset_name = platform_to_preset_name(p);
            build_jobs.push_str(&format!(r#"
{job_name}:
  stage: build
  image: barichello/godot-ci:{godot_version}
  script:
    - mkdir -v -p build/{platform}
    - godot --headless --export-release "{preset_name}" build/{safe_name}-{platform}
  artifacts:
    paths:
      - build/{safe_name}-{platform}
"#, job_name = job_name, godot_version = godot_version, platform = p, preset_name = preset_name, safe_name = safe_name));
        }

        let ci = format!(r#"image: barichello/godot-ci:{godot_version}

stages:
  - build

{build_jobs}
"#, godot_version = godot_version, build_jobs = build_jobs);

        Ok(ci)
    }).await.map_err(|e| format!("任务执行失败: {}", e))?
}

#[tauri::command]
pub async fn write_ci_config(app: AppHandle, project_id: String, provider: String, content: String) -> Result<(), String> {
    let app_clone = app.clone();
    tokio::task::spawn_blocking(move || {
        let storage = get_storage(&app_clone);
        let projects: Vec<Project> = storage.load_or_default("projects.json");
        let project = projects.iter().find(|p| p.project_id == project_id)
            .ok_or("项目不存在".to_string())?;

        let project_path = Path::new(&project.path);

        match provider.as_str() {
            "github-actions" => {
                let workflows_dir = project_path.join(".github").join("workflows");
                fs::create_dir_all(&workflows_dir)
                    .map_err(|e| format!("创建 .github/workflows 目录失败: {}", e))?;
                fs::write(workflows_dir.join("build.yml"), content)
                    .map_err(|e| format!("写入 build.yml 失败: {}", e))?;
            }
            "gitlab-ci" => {
                fs::write(project_path.join(".gitlab-ci.yml"), content)
                    .map_err(|e| format!("写入 .gitlab-ci.yml 失败: {}", e))?;
            }
            _ => return Err(format!("不支持的 CI 提供商: {}", provider)),
        }

        Ok(())
    }).await.map_err(|e| format!("任务执行失败: {}", e))?
}

#[tauri::command]
pub fn get_builtin_export_presets() -> Vec<BuiltinExportPreset> {
    vec![
        BuiltinExportPreset {
            platform: "windows".to_string(),
            name: "Windows Desktop".to_string(),
            description: "Windows 桌面应用（.exe）".to_string(),
            config: serde_json::json!({
                "binary_format": "64",
                "texture_format": "s3tc_bptc",
            }),
        },
        BuiltinExportPreset {
            platform: "web".to_string(),
            name: "HTML5".to_string(),
            description: "Web 浏览器应用".to_string(),
            config: serde_json::json!({
                "texture_format": "s3tc_bptc",
                "html/window_size": "1280x720",
            }),
        },
        BuiltinExportPreset {
            platform: "linux".to_string(),
            name: "Linux/X11".to_string(),
            description: "Linux 桌面应用".to_string(),
            config: serde_json::json!({
                "binary_format": "64",
                "texture_format": "s3tc_bptc",
            }),
        },
        BuiltinExportPreset {
            platform: "macos".to_string(),
            name: "macOS".to_string(),
            description: "macOS 应用（.app）".to_string(),
            config: serde_json::json!({
                "texture_format": "s3tc_bptc",
            }),
        },
        BuiltinExportPreset {
            platform: "android".to_string(),
            name: "Android".to_string(),
            description: "Android 应用（.apk）".to_string(),
            config: serde_json::json!({
                "architectures": "arm64-v8a",
                "keystore/debug": "",
            }),
        },
        BuiltinExportPreset {
            platform: "ios".to_string(),
            name: "iOS".to_string(),
            description: "iOS 应用（.ipa）".to_string(),
            config: serde_json::json!({
                "architectures": "arm64",
            }),
        },
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuiltinExportPreset {
    pub platform: String,
    pub name: String,
    pub description: String,
    pub config: serde_json::Value,
}

#[tauri::command]
pub fn export_preset_to_json(preset: BuiltinExportPreset) -> Result<String, String> {
    serde_json::to_string_pretty(&preset)
        .map_err(|e| format!("序列化预设失败: {}", e))
}

#[tauri::command]
pub fn import_preset_from_json(_project_id: String, json: String) -> Result<ExportPreset, String> {
    serde_json::from_str(&json)
        .map_err(|e| format!("解析预设失败: {}", e))
}
