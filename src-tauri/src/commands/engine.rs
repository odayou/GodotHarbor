use std::fs;
use tauri::{AppHandle, Emitter};
use crate::models::*;
use uuid::Uuid;
use super::utils::*;

#[tauri::command]
pub fn launch_engine(app: AppHandle, engine_id: String, project_path: Option<String>, project_id: Option<String>) -> Result<(), String> {
    let storage = get_storage(&app);
    let engines: Vec<Engine> = storage.load_or_default("engines.json");

    let engine = engines.iter()
        .find(|e| e.engine_id == engine_id)
        .ok_or("未找到指定引擎".to_string())?;

    let exe_path = crate::engine::EngineManager::find_executable_in_dir(std::path::Path::new(&engine.path))
        .ok_or("未找到引擎可执行文件".to_string())?;

    let mut cmd = detached_cmd(&exe_path);
    if let Some(ref path) = project_path {
        cmd.arg("--editor").arg("--path").arg(path);
    }

    cmd.spawn()
        .map_err(|e| format!("启动引擎失败: {}", e))?;

    if project_id.is_some() || project_path.is_some() {
        let mut projects: Vec<Project> = storage.load_or_default("projects.json");
        let mut found = false;
        for proj in &mut projects {
            let id_match = project_id.as_ref().map_or(false, |id| proj.project_id == *id);
            let path_match = project_path.as_ref().map_or(false, |p| {
                proj.path.replace('\\', "/").trim_end_matches('/').to_lowercase()
                    == p.replace('\\', "/").trim_end_matches('/').to_lowercase()
            });
            if id_match || path_match {
                proj.last_opened_at = Some(chrono::Utc::now());
                proj.last_used_engine_id = Some(engine_id.clone());
                found = true;
                break;
            }
        }
        if found {
            storage.save("projects.json", &projects)
                .map_err(|e| format!("保存项目信息失败: {}", e))?;
        }
        let _ = app.emit("project-opened", ());
    }

    let log_msg = match &project_path {
        Some(p) => format!("启动引擎打开项目: {} ({})", engine.name, p),
        None => format!("启动引擎: {}", engine.name),
    };
    log_operation(&app, "launch_engine", &engine_id, &log_msg);
    Ok(())
}

#[tauri::command]
pub fn set_project_default_engine(app: AppHandle, project_id: String, engine_id: String) -> Result<(), String> {
    let storage = get_storage(&app);
    let mut projects: Vec<Project> = storage.load_or_default("projects.json");
    let proj = projects.iter_mut().find(|p| p.project_id == project_id)
        .ok_or("未找到指定项目".to_string())?;
    proj.last_used_engine_id = Some(engine_id);
    storage.save("projects.json", &projects)
        .map_err(|e| format!("保存失败: {}", e))?;
    let _ = app.emit("project-opened", ());
    Ok(())
}

#[derive(serde::Serialize)]
pub struct MatchedEngine {
    pub engine: Engine,
    pub match_level: String,
}

#[tauri::command]
pub fn find_matching_engines(app: AppHandle, godot_version: String) -> Result<Vec<MatchedEngine>, String> {
    let storage = get_storage(&app);
    let engines: Vec<Engine> = storage.load_or_default("engines.json");

    let project_parts: Vec<&str> = godot_version.split('.').collect();
    let project_major = project_parts.first().and_then(|s| s.parse::<u32>().ok());
    let project_minor = project_parts.get(1).and_then(|s| s.parse::<u32>().ok());
    let project_patch = project_parts.get(2).and_then(|s| s.parse::<u32>().ok());

    let mut matched: Vec<MatchedEngine> = Vec::new();

    for engine in engines {
        let engine_parts: Vec<&str> = engine.version.split('.').collect();
        let engine_major = engine_parts.first().and_then(|s| s.parse::<u32>().ok());
        let engine_minor = engine_parts.get(1).and_then(|s| s.parse::<u32>().ok());
        let engine_patch = engine_parts.get(2).and_then(|s| s.parse::<u32>().ok());

        let match_level = match (project_major, project_minor, project_patch, engine_major, engine_minor, engine_patch) {
            (Some(pm), Some(pn), Some(pp), Some(em), Some(en), Some(ep))
                if pm == em && pn == en && pp == ep => "exact",
            (Some(pm), Some(pn), _, Some(em), Some(en), _)
                if pm == em && pn == en => "minor",
            (Some(pm), _, _, Some(em), _, _)
                if pm == em => "major",
            _ => "none",
        };

        if match_level != "none" {
            matched.push(MatchedEngine {
                engine,
                match_level: match_level.to_string(),
            });
        }
    }

    matched.sort_by(|a, b| {
        let level_order = |l: &str| match l {
            "exact" => 0,
            "minor" => 1,
            "major" => 2,
            _ => 3,
        };
        level_order(&a.match_level).cmp(&level_order(&b.match_level))
    });

    Ok(matched)
}

#[tauri::command]
pub async fn fetch_remote_engine_versions(
    app: AppHandle,
    mirror_id: String,
    force_refresh: Option<bool>,
) -> Result<Vec<crate::models::RemoteEngineVersion>, String> {
    let settings = load_settings(&app);

    let mirror = settings.engine_mirrors.iter()
        .find(|m| m.id == mirror_id)
        .ok_or("未找到指定的镜像配置".to_string())?;

    if !mirror.enabled {
        return Err("该镜像已被禁用".to_string());
    }

    let force = force_refresh.unwrap_or(false);
    let current_cache_version: u32 = 2;

    let cache_dir = get_data_dir(&app).join("cache");
    let cache_file = cache_dir.join(format!("remote_versions_{}.json", mirror_id));

    let mut expired_cache: Option<crate::models::CachedRemoteVersions> = None;

    if force {
        let _ = fs::remove_file(&cache_file);
    } else if cache_file.exists() {
        if let Ok(content) = fs::read_to_string(&cache_file) {
            if let Ok(cached) = serde_json::from_str::<crate::models::CachedRemoteVersions>(&content) {
                if cached.cache_version != current_cache_version {
                    let _ = fs::remove_file(&cache_file);
                    log_operation(&app, "fetch_remote_engine_versions", &mirror_id,
                        "缓存版本不匹配，已清除旧缓存");
                } else if let Ok(cached_time) = chrono::DateTime::parse_from_rfc3339(&cached.cached_at) {
                    let elapsed = chrono::Utc::now().signed_duration_since(cached_time.with_timezone(&chrono::Utc));
                    if elapsed.num_minutes() < 30 {
                        log_operation(&app, "fetch_remote_engine_versions", &mirror_id,
                            &format!("使用缓存，共 {} 个版本", cached.versions.len()));
                        return Ok(cached.versions);
                    }
                    if !cached.versions.is_empty() {
                        expired_cache = Some(cached);
                    }
                }
            }
        }
    }

    let storage = get_storage(&app);
    let engines: Vec<Engine> = storage.load_or_default("engines.json");
    let local_versions: Vec<String> = engines.iter().map(|e| e.version.clone()).collect();

    match crate::engine_downloader::EngineDownloader::fetch_remote_versions(mirror, &local_versions).await {
        Ok(versions) => {
            if versions.is_empty() {
                if let Some(ref cached) = expired_cache {
                    log_operation(&app, "fetch_remote_engine_versions", &mirror_id,
                        "API 返回空结果，使用过期缓存");
                    return Ok(cached.versions.clone());
                }
                log_operation(&app, "fetch_remote_engine_versions", &mirror_id,
                    "API 返回空结果且无缓存");
                return Ok(versions);
            }

            let cache_dir = get_data_dir(&app).join("cache");
            let _ = fs::create_dir_all(&cache_dir);
            let cache_file = cache_dir.join(format!("remote_versions_{}.json", mirror_id));
            let cached = crate::models::CachedRemoteVersions {
                cache_version: current_cache_version,
                cached_at: chrono::Utc::now().to_rfc3339(),
                mirror_id: mirror_id.clone(),
                versions: versions.clone(),
            };
            if let Ok(json) = serde_json::to_string_pretty(&cached) {
                let _ = fs::write(&cache_file, json);
            }

            log_operation(&app, "fetch_remote_engine_versions", &mirror_id,
                &format!("获取远程引擎版本列表，共 {} 个版本", versions.len()));

            Ok(versions)
        }
        Err(e) => {
            if let Some(ref cached) = expired_cache {
                log_operation(&app, "fetch_remote_engine_versions", &mirror_id,
                    &format!("API 请求失败({}), 使用过期缓存，共 {} 个版本", e, cached.versions.len()));
                return Ok(cached.versions.clone());
            }
            Err(e)
        }
    }
}

#[tauri::command]
pub async fn download_engine(
    app: AppHandle,
    remote_version: crate::models::RemoteEngineVersion,
) -> Result<crate::models::DownloadEngineResult, String> {
    let data_dir = get_data_dir(&app);
    let engines_dir = data_dir.join("engines");
    std::fs::create_dir_all(&engines_dir)
        .map_err(|e| format!("创建引擎目录失败: {}", e))?;

    if remote_version.file_size > 0 {
        if let Ok(available) = fs2::available_space(&engines_dir) {
            let required = remote_version.file_size as u64 * 3;
            if available < required {
                let avail_mb = available as f64 / 1024.0 / 1024.0;
                let req_mb = required as f64 / 1024.0 / 1024.0;
                return Ok(crate::models::DownloadEngineResult {
                    success: false,
                    cancelled: false,
                    engine: None,
                    error: Some(format!("磁盘空间不足，可用 {:.0}MB，需要约 {:.0}MB（包含下载文件 + 解压空间 + 余量）", avail_mb, req_mb)),
                });
            }
        }
    }

    let installed_path = match crate::engine_downloader::EngineDownloader::download_and_install(
        &app, &remote_version, engines_dir,
    ).await {
        Ok(path) => path,
        Err(e) => {
            let is_cancelled = e == "下载已取消";
            if is_cancelled {
                return Ok(crate::models::DownloadEngineResult {
                    success: false,
                    cancelled: true,
                    engine: None,
                    error: Some(e),
                });
            }
            return Err(e);
        }
    };

    let path_str = installed_path.to_string_lossy().to_string();

    let engine = match crate::engine::EngineManager::get_engine_info(&path_str) {
        Ok(e) => e,
        Err(detail) => {
            let _ = std::fs::remove_dir_all(&installed_path);
            return Ok(crate::models::DownloadEngineResult {
                success: false,
                cancelled: false,
                engine: None,
                error: Some(format!("下载的引擎文件无效: {}", detail)),
            });
        }
    };

    let mut registered_engine = engine;
    registered_engine.name = if remote_version.variant == "mono" {
        format!("Godot {} (.NET)", remote_version.version)
    } else {
        format!("Godot {}", remote_version.version)
    };

    let storage = get_storage(&app);
    let mut engines: Vec<Engine> = storage.load_or_default("engines.json");

    engines.retain(|e| e.path != registered_engine.path);

    engines.push(registered_engine.clone());
    storage.save("engines.json", &engines)
        .map_err(|e| format!("保存引擎信息失败: {}", e))?;

    log_operation(&app, "download_engine", &remote_version.version,
        &format!("已下载并注册引擎: {}", registered_engine.name));

    let _ = app.emit("engines-discovered", ());

    Ok(crate::models::DownloadEngineResult {
        success: true,
        cancelled: false,
        engine: Some(registered_engine),
        error: None,
    })
}

#[tauri::command]
pub async fn download_engine_from_url(
    app: AppHandle,
    url: String,
    engine_name: Option<String>,
) -> Result<crate::models::DownloadEngineResult, String> {
    if url.is_empty() {
        return Err("请输入下载地址".to_string());
    }
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err("请输入有效的 HTTP/HTTPS 地址".to_string());
    }

    let data_dir = get_data_dir(&app);
    let engines_dir = data_dir.join("engines");
    std::fs::create_dir_all(&engines_dir)
        .map_err(|e| format!("创建引擎目录失败: {}", e))?;

    let url_path = url.split('?').next().unwrap_or(&url);
    let file_name = url_path.split('/').last().unwrap_or("engine").to_string();

    let version_key = format!("url_{}", Uuid::new_v4());
    let variant = "standard";
    crate::engine_downloader::reset_cancel(&version_key, variant);

    let download_dir = get_data_dir(&app).join("downloads");
    std::fs::create_dir_all(&download_dir)
        .map_err(|e| format!("创建下载目录失败: {}", e))?;

    let archive_path = download_dir.join(&file_name);

    let download_result = crate::engine_downloader::EngineDownloader::download_file(
        &app, &url, &archive_path, &version_key, variant, 0,
    ).await;

    if let Err(e) = download_result {
        crate::engine_downloader::cleanup_on_error(&archive_path, false, &version_key, variant);
        return Ok(crate::models::DownloadEngineResult {
            success: false,
            cancelled: e == "下载已取消",
            engine: None,
            error: Some(e),
        });
    }

    let target_dir_name = format!("custom_{}", version_key.replace('-', "_"));
    let target_dir = engines_dir.join(&target_dir_name);

    if target_dir.exists() {
        let _ = std::fs::remove_dir_all(&target_dir);
    }
    std::fs::create_dir_all(&target_dir)
        .map_err(|e| format!("创建引擎目录失败: {}", e))?;

    let extract_result = crate::engine_downloader::EngineDownloader::extract_archive(
        &app, &version_key, variant, &archive_path, &target_dir,
    );
    let _ = std::fs::remove_file(&archive_path);

    if let Err(e) = extract_result {
        crate::engine_downloader::cleanup_on_error(&target_dir, true, &version_key, variant);
        return Ok(crate::models::DownloadEngineResult {
            success: false,
            cancelled: false,
            engine: None,
            error: Some(format!("解压引擎文件失败: {}", e)),
        });
    }

    let path_str = target_dir.to_string_lossy().to_string();
    let engine = match crate::engine::EngineManager::get_engine_info(&path_str) {
        Ok(e) => e,
        Err(detail) => {
            let _ = std::fs::remove_dir_all(&target_dir);
            return Ok(crate::models::DownloadEngineResult {
                success: false,
                cancelled: false,
                engine: None,
                error: Some(format!("下载的引擎文件无效: {}", detail)),
            });
        }
    };

    let mut registered_engine = engine;
    if let Some(name) = engine_name {
        if !name.is_empty() {
            registered_engine.name = name;
        }
    }

    let storage = get_storage(&app);
    let mut engines: Vec<Engine> = storage.load_or_default("engines.json");
    engines.retain(|e| e.path != registered_engine.path);
    engines.push(registered_engine.clone());
    storage.save("engines.json", &engines)
        .map_err(|e| format!("保存引擎信息失败: {}", e))?;

    let mut settings = load_settings(&app);
    let parent = std::path::Path::new(&path_str)
        .parent()
        .map(|p| p.to_string_lossy().to_string());
    if let Some(parent_path) = parent {
        if !settings.known_engine_paths.iter().any(|p| p.to_lowercase() == parent_path.to_lowercase()) {
            settings.known_engine_paths.push(parent_path);
            let config_storage = get_config_storage(&app);
            let _ = config_storage.save("settings.json", &settings);
        }
    }

    log_operation(&app, "download_engine_from_url", &url,
        &format!("从 URL 下载并注册引擎: {}", registered_engine.name));

    let _ = app.emit("engines-discovered", ());

    Ok(crate::models::DownloadEngineResult {
        success: true,
        cancelled: false,
        engine: Some(registered_engine),
        error: None,
    })
}

#[tauri::command]
pub fn cancel_engine_download(version: String, variant: String) -> Result<(), String> {
    crate::engine_downloader::request_cancel_download(&version, &variant);
    Ok(())
}

#[tauri::command]
pub fn get_active_downloads() -> Vec<crate::models::EngineDownloadProgress> {
    crate::engine_downloader::get_active_downloads()
}

#[tauri::command]
pub fn cleanup_download_temp(app: AppHandle) -> Result<u64, String> {
    let download_dir = get_data_dir(&app).join("downloads");
    if !download_dir.exists() {
        return Ok(0);
    }
    let mut cleaned = 0u64;
    if let Ok(entries) = fs::read_dir(&download_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Ok(metadata) = fs::metadata(&path) {
                    let modified = metadata.modified().ok();
                    let age = modified.and_then(|m| m.elapsed().ok());
                    if age.map_or(false, |d| d.as_secs() > 3600) {
                        if fs::remove_file(&path).is_ok() {
                            cleaned += 1;
                        }
                    }
                }
            }
        }
    }
    Ok(cleaned)
}


#[tauri::command]
pub fn register_engine(app: AppHandle, path: String, name: String) -> Result<Engine, String> {
    if path.is_empty() {
        return Err("引擎路径不能为空".to_string());
    }

    let engine_dir = {
        let p = std::path::Path::new(&path);
        if p.is_file() {
            p.parent().map(|dir| dir.to_string_lossy().to_string()).unwrap_or(path.clone())
        } else {
            path.clone()
        }
    };

    if let Err(detail) = crate::engine::EngineManager::validate_engine_path_detail(&engine_dir) {
        log_error(&app, "register_engine", &path, &detail);
        return Err(format!("引擎路径无效: {}", detail));
    }

    let engine = crate::engine::EngineManager::get_engine_info(&engine_dir)
        .map_err(|e| format!("获取引擎信息失败: {}", e))?;

    let mut registered_engine = engine;
    registered_engine.name = if name.is_empty() { registered_engine.name.clone() } else { name };

    let storage = get_storage(&app);
    let mut engines: Vec<Engine> = storage.load_or_default("engines.json");

    engines.retain(|e| e.path != registered_engine.path);

    engines.push(registered_engine.clone());
    storage.save("engines.json", &engines)
        .map_err(|e| format!("保存引擎信息失败: {}", e))?;

    let mut settings = load_settings(&app);
    let parent = std::path::Path::new(&engine_dir)
        .parent()
        .map(|p| p.to_string_lossy().to_string());
    if let Some(parent_path) = parent {
        if !settings.known_engine_paths.iter().any(|p| p.to_lowercase() == parent_path.to_lowercase()) {
            settings.known_engine_paths.push(parent_path);
            let config_storage = get_config_storage(&app);
            let _ = config_storage.save("settings.json", &settings);
        }
    }

    log_operation(&app, "register_engine", &path, &format!("已注册引擎: {}", registered_engine.name));
    Ok(registered_engine)
}

#[tauri::command]
pub fn get_engines(app: AppHandle) -> Result<Vec<Engine>, String> {
    let storage = get_storage(&app);
    let mut engines: Vec<Engine> = storage.load_or_default("engines.json");

    let migrated: bool = storage.load_or_default("engine_version_migrated_v2.json");
    if !migrated {
        for engine in &mut engines {
            if let Ok((_, version)) = crate::engine::EngineManager::detect_engine(&engine.path) {
                if version != engine.version {
                    let is_mono = version.to_lowercase().contains("mono");
                    engine.name = if is_mono {
                        format!("Godot {} (.NET)", version)
                    } else {
                        format!("Godot {}", version)
                    };
                    engine.version = version;
                }
            }
        }
        let _ = storage.save("engines.json", &engines);
        let _ = storage.save("engine_version_migrated_v2.json", &true);
    }

    Ok(engines)
}

#[tauri::command]
pub fn remove_engine(app: AppHandle, engine_id: String, delete_files: bool) -> Result<(), String> {
    let storage = get_storage(&app);
    let mut engines: Vec<Engine> = storage.load_or_default("engines.json");

    let engine = engines.iter().find(|e| e.engine_id == engine_id)
        .ok_or("未找到指定引擎".to_string())?;
    let engine_name = engine.name.clone();
    let engine_path = engine.path.clone();

    engines.retain(|e| e.engine_id != engine_id);

    storage.save("engines.json", &engines)
        .map_err(|e| format!("保存引擎列表失败: {}", e))?;

    if delete_files && !engine_path.is_empty() {
        let path = std::path::Path::new(&engine_path);
        if path.exists() && path.is_dir() {
            let data_dir = get_data_dir(&app);
            let engines_dir = data_dir.join("engines");
            if path.starts_with(&engines_dir) {
                let _ = std::fs::remove_dir_all(path);
            }
        }
    }

    log_operation(&app, "remove_engine", &engine_id, &format!("已删除引擎: {}{}", engine_name, if delete_files { "（含文件）" } else { "" }));
    Ok(())
}


#[tauri::command]
pub async fn auto_discover_engines(app: AppHandle) -> Result<Vec<Engine>, String> {
    let settings = load_settings(&app);

    if !settings.auto_discover_engines {
        log_operation(&app, "auto_discover_engines", "", "自动发现已关闭");
        return Ok(Vec::new());
    }

    let storage = get_storage(&app);
    let mut engines: Vec<Engine> = storage.load_or_default("engines.json");

    let mut removed_count = 0;
    engines.retain(|e| {
        let valid = std::path::Path::new(&e.path).exists();
        if !valid {
            removed_count += 1;
        }
        valid
    });
    if removed_count > 0 {
        let _ = storage.save("engines.json", &engines);
        log_operation(&app, "auto_discover_engines", "",
            &format!("清理 {} 个失效引擎", removed_count));
    }

    let existing_paths: Vec<String> = engines.iter().map(|e| e.path.clone()).collect();
    let scan_dirs = settings.scan_directories.clone();
    let known_engine_paths = settings.known_engine_paths.clone();

    log_operation(&app, "auto_discover_engines", "", "开始自动发现引擎");

    let discovered = tokio::task::spawn_blocking(move || {
        crate::engine::EngineManager::discover_engines_with_known_paths(
            &existing_paths,
            &scan_dirs,
            &known_engine_paths,
        )
    })
    .await
    .map_err(|e| format!("发现引擎任务失败: {}", e))?;

    if discovered.is_empty() {
        return Ok(Vec::new());
    }

    let discovered_count = discovered.len();
    for engine in &discovered {
        engines.push(engine.clone());
    }

    let storage = get_storage(&app);
    storage.save("engines.json", &engines)
        .map_err(|e| format!("保存引擎列表失败: {}", e))?;

    let mut settings = load_settings(&app);
    for engine in &discovered {
        let parent = std::path::Path::new(&engine.path)
            .parent()
            .map(|p| p.to_string_lossy().to_string());
        if let Some(parent_path) = parent {
            if !settings.known_engine_paths.iter().any(|p| p.to_lowercase() == parent_path.to_lowercase()) {
                settings.known_engine_paths.push(parent_path);
            }
        }
    }
    let config_storage = get_config_storage(&app);
    let _ = config_storage.save("settings.json", &settings);

    let _ = app.emit("engines-discovered", &discovered);

    log_operation(&app, "auto_discover_engines", "",
        &format!("自动发现 {} 个 Godot 引擎", discovered_count));

    Ok(discovered)
}

#[tauri::command]
pub fn check_engine_health(app: AppHandle, engine_id: String) -> Result<bool, String> {
    let storage = get_storage(&app);
    let engines: Vec<Engine> = storage.load_or_default("engines.json");

    let engine = engines.iter()
        .find(|e| e.engine_id == engine_id)
        .ok_or("未找到指定引擎".to_string())?;

    let exe_path = crate::engine::EngineManager::find_executable_in_dir(std::path::Path::new(&engine.path));
    Ok(exe_path.is_some())
}

#[tauri::command]
pub fn rename_engine(app: AppHandle, engine_id: String, new_name: String) -> Result<(), String> {
    if new_name.trim().is_empty() {
        return Err("引擎名称不能为空".to_string());
    }

    let storage = get_storage(&app);
    let mut engines: Vec<Engine> = storage.load_or_default("engines.json");

    let old_name;
    let new_engine_name;
    {
        let engine = engines.iter_mut()
            .find(|e| e.engine_id == engine_id)
            .ok_or("未找到指定引擎".to_string())?;

        old_name = engine.name.clone();
        new_engine_name = new_name.trim().to_string();
        engine.name = new_engine_name.clone();
    }

    storage.save("engines.json", &engines)
        .map_err(|e| format!("保存引擎列表失败: {}", e))?;

    log_operation(&app, "rename_engine", &engine_id, &format!("引擎重命名: {} -> {}", old_name, new_engine_name));
    Ok(())
}

#[tauri::command]
pub async fn check_godot_updates(app: AppHandle) -> Result<crate::version_checker::GodotVersionCheckResult, String> {
    let storage = get_storage(&app);
    let engines: Vec<Engine> = storage.load_or_default("engines.json");
    let settings: crate::models::Settings = storage.load_or_default("settings.json");

    let allowed_channels: Vec<String> = settings.engine_update_channels;

    let local_engines: Vec<crate::version_checker::LocalEngineVersion> = engines.iter().map(|e| {
        crate::version_checker::LocalEngineVersion {
            engine_id: e.engine_id.clone(),
            name: e.name.clone(),
            version: e.version.clone(),
            engine_type: format!("{:?}", e.engine_type),
        }
    }).collect();

    let data_dir = get_data_dir(&app);
    let cache_dir = data_dir.join("cache");
    let github_base = crate::utils::get_github_api_base(&app);
    let checker = crate::version_checker::VersionChecker::new(cache_dir)
        .with_github_api_base(github_base);

    let mut result = checker.check_for_updates(local_engines).await
        .map_err(|e| format!("检查Godot更新失败: {}", e))?;

    if !allowed_channels.is_empty() {
        result.updates_available.retain(|u| allowed_channels.contains(&u.channel));
    }

    if !result.updates_available.is_empty() {
        let _ = app.emit("godot-update-available", &result.updates_available);
    }

    log_operation(&app, "check_godot_updates", "",
        &format!("Godot版本检查完成，发现 {} 个可用更新", result.updates_available.len()));

    Ok(result)
}
