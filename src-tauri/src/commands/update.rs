use std::path::{PathBuf, Path};
use std::fs;
use std::io::Write;
use tauri::{AppHandle, Emitter};
use crate::models::*;
use crate::utils::{create_http_client, no_window_cmd};
use super::utils::*;
use super::plugin::{APP_GITHUB_OWNER, APP_GITHUB_REPO, check_plugin_updates};

use serde_json;

#[tauri::command]
pub async fn check_app_update(app: AppHandle, force_refresh: Option<bool>) -> Result<Option<AppUpdateInfo>, String> {
    let force = force_refresh.unwrap_or(false);
    let cache_version: u32 = 1;

    let cache_dir = get_data_dir(&app).join("cache");
    let cache_file = cache_dir.join("app_update.json");

    if !force && cache_file.exists() {
        if let Ok(content) = fs::read_to_string(&cache_file) {
            if let Ok(cached) = serde_json::from_str::<crate::models::CachedAppUpdate>(&content) {
                if cached.cache_version == cache_version {
                    if let Ok(cached_time) = chrono::DateTime::parse_from_rfc3339(&cached.cached_at) {
                        let elapsed = chrono::Utc::now().signed_duration_since(cached_time.with_timezone(&chrono::Utc));
                        if elapsed.num_minutes() < 30 {
                            return Ok(cached.update_info);
                        }
                    }
                } else {
                    let _ = fs::remove_file(&cache_file);
                }
            }
        }
    }

    let current_version = app.config().version.clone().unwrap_or_default();

    let mut settings = load_settings(&app);
    if !settings.skipped_app_version.is_empty() {
        let skipped = semver::Version::parse(settings.skipped_app_version.trim_start_matches('v')).ok();
        let current = semver::Version::parse(current_version.trim_start_matches('v')).ok();
        if let (Some(s), Some(c)) = (skipped, current) {
            if s <= c {
                settings.skipped_app_version = String::new();
                let _ = save_settings_to_config(&app, &settings);
            }
        }
    }

    let client = create_http_client(Some(std::time::Duration::from_secs(15)))?;

    let github_base = crate::utils::get_github_api_base(&app);
    let api_url = format!(
        "{}/repos/{}/{}/releases/latest",
        github_base, APP_GITHUB_OWNER, APP_GITHUB_REPO
    );

    let resp = client.get(&api_url).send().await
        .map_err(|e| format!("检查应用更新失败: {}", e))?;

    if !resp.status().is_success() {
        return Ok(None);
    }

    let json: serde_json::Value = resp.json().await
        .map_err(|e| format!("解析更新信息失败: {}", e))?;

    let tag = json.get("tag_name").and_then(|t| t.as_str()).unwrap_or("");
    let latest_version = tag.trim_start_matches('v').to_string();

    let current_semver = semver::Version::parse(current_version.trim_start_matches('v')).ok();
    let latest_semver = semver::Version::parse(&latest_version).ok();

    if let (Some(cur), Some(lat)) = (&current_semver, &latest_semver) {
        if lat <= cur {
            return Ok(None);
        }
    } else if latest_version == current_version {
        return Ok(None);
    }

    if !settings.skipped_app_version.is_empty() {
        if let Some(skipped) = semver::Version::parse(settings.skipped_app_version.trim_start_matches('v')).ok() {
            if let Some(lat) = &latest_semver {
                if &skipped >= lat {
                    return Ok(None);
                }
            }
        }
    }

    let release_notes = json.get("body").and_then(|b| b.as_str()).unwrap_or("").to_string();
    let pub_date = json.get("published_at").and_then(|d| d.as_str()).unwrap_or("").to_string();

    let target_ext = if cfg!(target_os = "windows") { ".nsis.zip" } else if cfg!(target_os = "macos") { ".app.tar.gz" } else { ".AppImage.tar.gz" };
    let mut download_url = None;
    let mut download_size = None;

    if let Some(assets) = json.get("assets").and_then(|a| a.as_array()) {
        for asset in assets {
            let name = asset.get("name").and_then(|n| n.as_str()).unwrap_or("");
            if name.ends_with(target_ext) {
                download_url = asset.get("browser_download_url").and_then(|u| u.as_str()).map(|s| s.to_string());
                download_size = asset.get("size").and_then(|s| s.as_u64());
                break;
            }
        }
        if download_url.is_none() {
            for asset in assets {
                let name = asset.get("name").and_then(|n| n.as_str()).unwrap_or("");
                if name.ends_with(".exe") || name.ends_with(".msi") {
                    download_url = asset.get("browser_download_url").and_then(|u| u.as_str()).map(|s| s.to_string());
                    download_size = asset.get("size").and_then(|s| s.as_u64());
                    break;
                }
            }
        }
    }

    let result = Ok(Some(AppUpdateInfo {
        current_version,
        latest_version,
        release_notes,
        pub_date,
        download_size,
        is_hot_update: false,
        download_url,
    }));

    if let Ok(ref info) = result {
        let _ = fs::create_dir_all(&cache_dir);
        let cached = crate::models::CachedAppUpdate {
            cache_version,
            cached_at: chrono::Utc::now().to_rfc3339(),
            update_info: info.clone(),
        };
        if let Ok(json) = serde_json::to_string_pretty(&cached) {
            let _ = fs::write(&cache_file, json);
        }
    }

    result
}

#[tauri::command]
pub async fn install_app_update(app: AppHandle) -> Result<(), String> {
    let update_info = check_app_update(app.clone(), Some(true)).await
        .map_err(|e| format!("检查更新失败: {}", e))?
        .ok_or("没有可用的更新".to_string())?;

    let download_url = update_info.download_url.clone()
        .ok_or("未找到下载链接".to_string())?;

    let download_url = {
        let storage = get_storage(&app);
        let settings: Settings = storage.load_or_default("settings.json");
        if !settings.github_api_proxy.is_empty() {
            download_url.replace("https://github.com/odayou/GodotHarbor/releases/download", 
                &format!("https://gitee.com/odayou/godot-harbor/releases/download"))
        } else {
            download_url
        }
    };

    let temp_dir = std::env::temp_dir().join("godot-harbor-update");
    fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("创建临时目录失败: {}", e))?;

    let file_name = download_url.split('/').last().unwrap_or("update.exe").to_string();
    let file_path = temp_dir.join(&file_name);

    let already_downloaded = if file_path.exists() {
        if let Ok(metadata) = fs::metadata(&file_path) {
            if let Some(expected_size) = update_info.download_size {
                metadata.len() == expected_size
            } else {
                metadata.len() > 0
            }
        } else {
            false
        }
    } else {
        false
    };

    if !already_downloaded {
        let client = create_http_client(None)?;

        let _ = app.emit("app-update-progress", serde_json::json!({
            "stage": "downloading",
            "progress": 0,
            "message": "正在下载更新..."
        }));

        let resp = client.get(&download_url).send().await
            .map_err(|e| format!("下载更新失败: {}", e))?;

        let total_size = resp.content_length();

        let mut file = fs::File::create(&file_path)
            .map_err(|e| format!("创建文件失败: {}", e))?;

        let mut downloaded: u64 = 0;
        let mut stream = resp.bytes_stream();
        use futures::StreamExt;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| format!("下载数据失败: {}", e))?;
            file.write_all(&chunk)
                .map_err(|e| format!("写入文件失败: {}", e))?;
            downloaded += chunk.len() as u64;

            let progress = if let Some(total) = total_size {
                ((downloaded as f64 / total as f64) * 100.0) as u32
            } else {
                0
            };

            let _ = app.emit("app-update-progress", serde_json::json!({
                "stage": "downloading",
                "progress": progress.min(100),
                "message": format!("下载中... {}%", progress.min(100))
            }));
        }
    } else {
        let _ = app.emit("app-update-progress", serde_json::json!({
            "stage": "installing",
            "progress": 100,
            "message": "安装包已就绪，正在启动安装程序..."
        }));
    }

    let _ = app.emit("app-update-progress", serde_json::json!({
        "stage": "installing",
        "progress": 100,
        "message": "下载完成，正在启动安装程序..."
    }));

    let data_dir = get_data_dir(&app);
    let hot_update_dir = data_dir.join("hot_updates");
    if hot_update_dir.exists() {
        let _ = fs::remove_dir_all(&hot_update_dir);
    }
    let overlay_dir = data_dir.join("hotupdate_overlay");
    if overlay_dir.exists() {
        let _ = fs::remove_dir_all(&overlay_dir);
    }

    if cfg!(target_os = "windows") {
        if file_name.ends_with(".nsis.zip") {
            let extract_dir = temp_dir.join("nsis_extract");
            let _ = fs::create_dir_all(&extract_dir);
            let extract_result = no_window_cmd("powershell")
                .args(["-NoProfile", "-Command", &format!(
                    "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
                    file_path.display(), extract_dir.display()
                )])
                .output();

            if let Err(e) = extract_result {
                return Err(format!("解压更新包失败: {}", e));
            }

            let installer = walk_dir_for_exe(&extract_dir);
            if let Some(installer) = installer {
                no_window_cmd(&installer)
                    .args(&["/S", "--force-run"])
                    .spawn()
                    .map_err(|e| format!("启动安装程序失败: {}", e))?;
            } else {
                open_file_in_os(&file_path)?;
            }
        } else {
            no_window_cmd(&file_path)
                .spawn()
                .map_err(|e| format!("启动安装程序失败: {}", e))?;
        }
    } else if cfg!(target_os = "macos") {
        let app_dir = std::env::current_exe()
            .ok()
            .and_then(|e| e.parent().and_then(|p| p.parent().map(|pp| pp.to_path_buf())))
            .unwrap_or_else(|| PathBuf::from("/Applications"));

        let _ = app.emit("app-update-progress", serde_json::json!({
            "stage": "installing",
            "progress": 100,
            "message": "正在解压并安装更新..."
        }));

        let extract_result = std::process::Command::new("tar")
            .args(["-xzf", &file_path.to_string_lossy(), "-C", &app_dir.to_string_lossy()])
            .output();

        if let Err(e) = extract_result {
            return Err(format!("解压更新包失败: {}", e));
        }

        let app_name = "Godot Harbor.app";
        std::process::Command::new("open")
            .arg(app_dir.join(app_name))
            .spawn()
            .map_err(|e| format!("启动应用失败: {}", e))?;
    } else {
        let _ = app.emit("app-update-progress", serde_json::json!({
            "stage": "installing",
            "progress": 100,
            "message": "正在解压并安装更新..."
        }));

        let extract_dir = temp_dir.join("appimage_extract");
        let _ = fs::create_dir_all(&extract_dir);

        let extract_result = std::process::Command::new("tar")
            .args(["-xzf", &file_path.to_string_lossy(), "-C", &extract_dir.to_string_lossy()])
            .output();

        if let Err(e) = extract_result {
            return Err(format!("解压更新包失败: {}", e));
        }

        let appimage = walk_dir_for_appimage(&extract_dir);
        if let Some(appimage_path) = appimage {
            let current_exe = std::env::current_exe().unwrap_or_default();
            let install_dir = current_exe.parent().unwrap_or(Path::new("/usr/bin"));
            let dest = install_dir.join(appimage_path.file_name().unwrap_or_default());
            let _ = fs::copy(&appimage_path, &dest);

            std::process::Command::new("chmod")
                .args(["+x", &dest.to_string_lossy()])
                .output().ok();

            std::process::Command::new(&dest)
                .spawn()
                .map_err(|e| format!("启动应用失败: {}", e))?;
        } else {
            open_file_in_os(&file_path)?;
        }
    }

    let _ = app.emit("app-update-progress", serde_json::json!({
        "stage": "complete",
        "progress": 100,
        "message": "安装程序已启动，即将重启..."
    }));

    record_update_history(&app, "app", "Godot Harbor", &update_info.current_version, &update_info.latest_version, "success", "安装程序已启动");

    app.exit(0);
    Ok(())
}

fn walk_dir_for_exe(dir: &Path) -> Option<PathBuf> {
    for entry in fs::read_dir(dir).ok()? {
        let entry = entry.ok()?;
        let path = entry.path();
        if path.is_dir() {
            if let Some(exe) = walk_dir_for_exe(&path) {
                return Some(exe);
            }
        } else if path.extension().map(|e| e == "exe").unwrap_or(false) {
            return Some(path);
        }
    }
    None
}

#[cfg(target_os = "linux")]
fn walk_dir_for_appimage(dir: &Path) -> Option<PathBuf> {
    for entry in fs::read_dir(dir).ok()? {
        let entry = entry.ok()?;
        let path = entry.path();
        if path.is_dir() {
            if let Some(f) = walk_dir_for_appimage(&path) {
                return Some(f);
            }
        } else if path.to_string_lossy().ends_with(".AppImage") {
            return Some(path);
        }
    }
    None
}

#[cfg(not(target_os = "linux"))]
fn walk_dir_for_appimage(_dir: &Path) -> Option<PathBuf> {
    None
}

fn open_file_in_os(path: &Path) -> Result<(), String> {
    if cfg!(target_os = "windows") {
        no_window_cmd("explorer")
            .arg(path)
            .spawn()
            .map_err(|e| format!("打开文件失败: {}", e))?;
    } else if cfg!(target_os = "macos") {
        std::process::Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("打开文件失败: {}", e))?;
    } else {
        std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("打开文件失败: {}", e))?;
    }
    Ok(())
}


#[tauri::command]
pub fn skip_app_version(app: AppHandle, version: String) -> Result<(), String> {
    let mut settings = load_settings(&app);
    settings.skipped_app_version = version;
    save_settings_to_config(&app, &settings)
        .map_err(|e| format!("保存设置失败: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn check_all_updates(app: AppHandle, force_refresh: Option<bool>) -> Result<UpdateCheckResult, String> {
    let force = force_refresh.unwrap_or(false);

    let cache_dir = get_data_dir(&app).join("cache");
    let rate_limit_file = cache_dir.join("last_update_check.txt");

    if !force && rate_limit_file.exists() {
        if let Ok(content) = fs::read_to_string(&rate_limit_file) {
            if let Ok(last_time) = chrono::DateTime::parse_from_rfc3339(content.trim()) {
                let elapsed = chrono::Utc::now().signed_duration_since(last_time.with_timezone(&chrono::Utc));
                if elapsed.num_minutes() < 5 {
                    let cache_file = cache_dir.join("last_update_check_result.json");
                    if let Ok(cached) = fs::read_to_string(&cache_file) {
                        if let Ok(result) = serde_json::from_str::<UpdateCheckResult>(&cached) {
                            return Ok(result);
                        }
                    }
                    return Ok(UpdateCheckResult {
                        app_update: None,
                        hot_update: None,
                        plugin_updates: vec![],
                        engine_updates: vec![],
                        checked_at: last_time.to_rfc3339(),
                    });
                }
            }
        }
    }

    let _ = fs::create_dir_all(&cache_dir);
    let _ = fs::write(&rate_limit_file, chrono::Utc::now().to_rfc3339());

    let plugin_updates = check_plugin_updates(app.clone(), Some(force)).await.unwrap_or_default();

    let storage = get_storage(&app);
    let engines: Vec<Engine> = storage.load_or_default("engines.json");
    let local_engines: Vec<crate::version_checker::LocalEngineVersion> = engines.iter().map(|e| {
        crate::version_checker::LocalEngineVersion {
            engine_id: e.engine_id.clone(),
            name: e.name.clone(),
            version: e.version.clone(),
            engine_type: e.engine_type.to_string(),
        }
    }).collect();

    let data_dir = get_data_dir(&app);
    let github_base = crate::utils::get_github_api_base(&app);
    let checker = crate::version_checker::VersionChecker::new(data_dir)
        .with_github_api_base(github_base);
    let engine_result = checker.check_for_updates(local_engines).await.ok();
    let mut engine_updates = engine_result.map(|r| r.updates_available).unwrap_or_default();

    let settings: crate::models::Settings = storage.load_or_default("settings.json");
    let allowed_channels: &[String] = &settings.engine_update_channels;
    if !allowed_channels.is_empty() {
        engine_updates.retain(|u| allowed_channels.contains(&u.channel));
    }

    let app_update = check_app_update(app.clone(), Some(force)).await.ok().flatten();

    let hot_update = check_hot_update(app.clone(), None).await.ok().flatten();

    let result = UpdateCheckResult {
        app_update,
        hot_update,
        plugin_updates,
        engine_updates,
        checked_at: chrono::Utc::now().to_rfc3339(),
    };

    let cache_file = cache_dir.join("last_update_check_result.json");
    let _ = fs::write(&cache_file, serde_json::to_string(&result).unwrap_or_default());

    Ok(result)
}

#[tauri::command]
pub fn get_app_version(app: AppHandle) -> Result<String, String> {
    Ok(app.config().version.clone().unwrap_or_default())
}

#[tauri::command]
pub async fn check_hot_update(app: AppHandle, manifest_url: Option<String>) -> Result<Option<HotUpdateInfo>, String> {
    let data_dir = get_data_dir(&app);
    let settings = load_settings(&app);
    let current_version = app.config().version.clone().unwrap_or_default();

    if !settings.skipped_app_version.is_empty() {
        let skipped_semver = semver::Version::parse(settings.skipped_app_version.trim_start_matches('v')).ok();
        let current_semver = semver::Version::parse(current_version.trim_start_matches('v')).ok();
        if let (Some(skipped), Some(current)) = (skipped_semver, current_semver) {
            if skipped > current {
                return Ok(None);
            }
        }
    }

    let url = manifest_url.unwrap_or_else(|| "https://godotharbor.odayou.workers.dev/hot-update/manifest.json".to_string());
    let manager = crate::hot_update::HotUpdateManager::new(data_dir);
    manager.check_for_hot_update(&url, &current_version).await
}

#[tauri::command]
pub async fn install_hot_update(app: AppHandle, manifest_url: Option<String>) -> Result<(), String> {
    let url = manifest_url.unwrap_or_else(|| "https://godotharbor.odayou.workers.dev/hot-update/manifest.json".to_string());
    let data_dir = get_data_dir(&app);
    let manager = crate::hot_update::HotUpdateManager::new(data_dir);
    manager.download_and_apply(&app, &url).await
}

#[tauri::command]
pub fn rollback_hot_update(app: AppHandle) -> Result<(), String> {
    let data_dir = get_data_dir(&app);
    let manager = crate::hot_update::HotUpdateManager::new(data_dir);
    manager.rollback(&app)
}

#[tauri::command]
pub fn get_current_hot_update_version(app: AppHandle) -> Result<Option<String>, String> {
    let data_dir = get_data_dir(&app);
    let manager = crate::hot_update::HotUpdateManager::new(data_dir);
    manager.get_current_hot_update_version()
}

#[tauri::command]
pub fn get_update_history(app: AppHandle) -> Result<Vec<crate::models::UpdateHistoryEntry>, String> {
    let storage = get_storage(&app);
    let history: Vec<crate::models::UpdateHistoryEntry> = storage.load_or_default("update_history.json");
    Ok(history)
}

#[tauri::command]
pub fn clear_update_history(app: AppHandle) -> Result<(), String> {
    let storage = get_storage(&app);
    let empty: Vec<crate::models::UpdateHistoryEntry> = Vec::new();
    storage.save("update_history.json", &empty)
        .map_err(|e| format!("保存更新历史失败: {}", e))?;
    Ok(())
}

pub fn record_update_history(app: &AppHandle, update_type: &str, target_name: &str, from_version: &str, to_version: &str, status: &str, notes: &str) {
    let storage = get_storage(app);
    let mut history: Vec<crate::models::UpdateHistoryEntry> = storage.load_or_default("update_history.json");
    
    history.insert(0, crate::models::UpdateHistoryEntry {
        id: uuid::Uuid::new_v4().to_string(),
        update_type: update_type.to_string(),
        target_name: target_name.to_string(),
        from_version: from_version.to_string(),
        to_version: to_version.to_string(),
        status: status.to_string(),
        applied_at: chrono::Utc::now().to_rfc3339(),
        notes: notes.to_string(),
    });

    if history.len() > 100 {
        history.truncate(100);
    }

    let _ = storage.save("update_history.json", &history);
}


