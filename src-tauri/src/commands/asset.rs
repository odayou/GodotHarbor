use std::fs;
use serde::{Serialize, Deserialize};
use tauri::{AppHandle, Emitter};
use crate::models::*;
use uuid::Uuid;
use crate::utils::create_http_client;
use super::utils::*;
use super::plugin::AssetLibrarySearchParams;

#[tauri::command]
pub async fn search_asset_library(app: AppHandle, params: AssetLibrarySearchParams) -> Result<serde_json::Value, String> {
    let mut url_params = vec![];

    if let Some(f) = &params.filter {
        url_params.push(format!("filter={}", urlencoding::encode(f)));
    } else {
        url_params.push("filter=".to_string());
    }

    url_params.push(format!("type={}", params.asset_type.as_deref().unwrap_or("any")));

    if let Some(c) = &params.category {
        url_params.push(format!("category={}", c));
    }
    if let Some(s) = &params.support {
        url_params.push(format!("support={}", s));
    }
    if let Some(c) = &params.cost {
        url_params.push(format!("cost={}", c));
    }

    url_params.push(format!("godot_version={}", params.godot_version.as_deref().unwrap_or("any")));
    url_params.push(format!("max_results={}", params.max_results.unwrap_or(20)));

    if let Some(p) = params.page {
        url_params.push(format!("page={}", p));
    }

    url_params.push(format!("sort={}", params.sort.as_deref().unwrap_or("updated")));

    if params.reverse.unwrap_or(false) {
        url_params.push("reverse".to_string());
    }

    let asset_lib_base = crate::utils::get_asset_library_base(&app);
    let url = format!("{}/asset?{}", asset_lib_base, url_params.join("&"));

    let client = create_http_client(None)?;

    let resp = client.get(&url).send().await
        .map_err(|e| format!("请求 Asset Library 失败: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Asset Library 返回错误状态: {}", resp.status()));
    }

    let text = resp.text().await
        .map_err(|e| format!("读取 Asset Library 响应失败: {}", e))?;

    let json: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| format!("解析 Asset Library 响应失败: {} (响应前100字符: {})", e, &text[..text.len().min(100)]))?;

    let filter_str = params.filter.as_deref().unwrap_or("");
    log_operation(&app, "search_asset_library", "", &format!("搜索 Asset Library: {}", filter_str));
    Ok(json)
}

#[tauri::command]
pub async fn get_asset_library_configure(app: AppHandle) -> Result<serde_json::Value, String> {
    let asset_lib_base = crate::utils::get_asset_library_base(&app);
    let url = format!("{}/configure?type=any", asset_lib_base);

    let client = create_http_client(None)?;

    let resp = client.get(url).send().await
        .map_err(|e| format!("请求 Asset Library 配置失败: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Asset Library 返回错误状态: {}", resp.status()));
    }

    let json: serde_json::Value = resp.json().await
        .map_err(|e| format!("解析 Asset Library 配置失败: {}", e))?;

    log_operation(&app, "get_asset_library_configure", "", "获取 Asset Library 配置");
    Ok(json)
}

#[tauri::command]
pub async fn get_asset_detail(app: AppHandle, asset_id: String) -> Result<serde_json::Value, String> {
    let asset_lib_base = crate::utils::get_asset_library_base(&app);
    let url = format!(
        "{}/asset/{}",
        asset_lib_base, asset_id
    );

    let client = create_http_client(None)?;

    let resp = client.get(&url).send().await
        .map_err(|e| format!("请求 Asset Library 失败: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Asset Library 返回错误状态: {}", resp.status()));
    }

    let json: serde_json::Value = resp.json().await
        .map_err(|e| format!("解析 Asset Library 响应失败: {}", e))?;

    log_operation(&app, "get_asset_detail", &asset_id, &format!("获取资产详情: {}", asset_id));
    Ok(json)
}

#[tauri::command]
pub async fn import_from_asset_library(app: AppHandle, asset_id: String) -> Result<Plugin, String> {
    let asset_lib_base = crate::utils::get_asset_library_base(&app);
    let url = format!(
        "{}/asset/{}",
        asset_lib_base, asset_id
    );

    let client = create_http_client(None)?;

    let resp = client.get(&url).send().await
        .map_err(|e| format!("请求 Asset Library 失败: {}", e))?;

    let asset: serde_json::Value = resp.json().await
        .map_err(|e| format!("解析 Asset Library 响应失败: {}", e))?;

    let download_url = asset.get("download_url")
        .and_then(|v| v.as_str())
        .ok_or("未找到下载链接")?;

    let asset_name = asset.get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let author_name = asset.get("author")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let desc = asset.get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let plugin_source = PluginSource {
        source_type: SourceType::AssetLibrary,
        url: format!("asset-library://{}", asset_id),
        imported_at: chrono::Utc::now(),
    };

    let mut plugin = Plugin::new(asset_name.clone(), plugin_source);
    plugin.description = desc;
    plugin.author = author_name;

    let version_id = Uuid::new_v4().to_string();
    let version_dir = get_data_dir(&app).join("plugins").join(&plugin.plugin_id).join(&version_id);
    let payload_dir = version_dir.join("payload");

    fs::create_dir_all(&payload_dir)
        .map_err(|e| format!("创建目录失败: {}", e))?;

    let temp_zip = version_dir.join("download.zip");
    let resp = client.get(download_url).send().await
        .map_err(|e| format!("下载资源失败: {}", e))?;

    let bytes = resp.bytes().await
        .map_err(|e| format!("读取下载数据失败: {}", e))?;

    fs::write(&temp_zip, &bytes)
        .map_err(|e| format!("写入文件失败: {}", e))?;

    let file = std::fs::File::open(&temp_zip)
        .map_err(|e| format!("打开压缩文件失败: {}", e))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| format!("解压失败: {}", e))?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| format!("读取压缩条目失败: {}", e))?;
        let outpath = match entry.enclosed_name() {
            Some(path) => payload_dir.join(path),
            None => continue,
        };
        if entry.is_dir() {
            std::fs::create_dir_all(&outpath).ok();
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(p).ok();
                }
            }
            let mut outfile = std::fs::File::create(&outpath)
                .map_err(|e| format!("创建文件失败: {}", e))?;
            std::io::copy(&mut entry, &mut outfile)
                .map_err(|e| format!("写入文件失败: {}", e))?;
        }
    }

    let _ = std::fs::remove_file(&temp_zip);

    let manager = get_plugin_manager(&app);
    let (units, asset_type) = manager.analyze_asset_type(&payload_dir, &asset_name);

    let compatibility = manager.detect_compatibility(&payload_dir);

    let content_hash = crate::models::compute_dir_hash(&payload_dir).unwrap_or_default();

    let (unit_version, unit_name) = if let Some(first_unit) = units.first() {
        (
            if first_unit.version.is_empty() { "1.0.0".to_string() } else { first_unit.version.clone() },
            if first_unit.name.is_empty() { asset_name.clone() } else { first_unit.name.clone() },
        )
    } else {
        ("1.0.0".to_string(), asset_name.clone())
    };

    let plugin_version = PluginVersion {
        version_id: version_id.clone(),
        version: unit_version,
        path: payload_dir.to_string_lossy().to_string(),
        created_at: chrono::Utc::now(),
        units,
    };

    plugin.versions.push(plugin_version);
    plugin.compatibility = compatibility;
    plugin.name = unit_name;
    plugin.content_hash = content_hash;
    plugin.asset_type = asset_type;

    upsert_plugin(&app, &plugin, "import_asset_library", &asset_id.to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetImportProgressPayload {
    pub asset_id: String,
    pub stage: String,
    pub progress: f64,
    pub message: String,
}

#[tauri::command]
pub async fn import_from_asset_library_with_progress(app: AppHandle, asset_id: String) -> Result<Plugin, String> {
    let _ = app.emit("asset-import-progress", AssetImportProgressPayload {
        asset_id: asset_id.clone(),
        stage: "downloading".to_string(),
        progress: 0.0,
        message: "正在获取资产信息...".to_string(),
    });

    let asset_lib_base = crate::utils::get_asset_library_base(&app);
    let url = format!(
        "{}/asset/{}",
        asset_lib_base, asset_id
    );

    let client = create_http_client(None)?;

    let resp = client.get(&url).send().await
        .map_err(|e| format!("请求 Asset Library 失败: {}", e))?;

    let asset: serde_json::Value = resp.json().await
        .map_err(|e| format!("解析 Asset Library 响应失败: {}", e))?;

    let download_url = asset.get("download_url")
        .and_then(|v| v.as_str())
        .ok_or("未找到下载链接")?;

    let asset_name = asset.get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let author_name = asset.get("author")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let desc = asset.get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let plugin_source = PluginSource {
        source_type: SourceType::AssetLibrary,
        url: format!("asset-library://{}", asset_id),
        imported_at: chrono::Utc::now(),
    };

    let mut plugin = Plugin::new(asset_name.clone(), plugin_source);
    plugin.description = desc;
    plugin.author = author_name;

    let version_id = Uuid::new_v4().to_string();
    let version_dir = get_data_dir(&app).join("plugins").join(&plugin.plugin_id).join(&version_id);
    let payload_dir = version_dir.join("payload");

    fs::create_dir_all(&payload_dir)
        .map_err(|e| format!("创建目录失败: {}", e))?;

    let _ = app.emit("asset-import-progress", AssetImportProgressPayload {
        asset_id: asset_id.clone(),
        stage: "downloading".to_string(),
        progress: 0.1,
        message: format!("正在下载 {}...", asset_name),
    });

    let temp_zip = version_dir.join("download.zip");
    let resp = client.get(download_url).send().await
        .map_err(|e| format!("下载资源失败: {}", e))?;

    let total_size = resp.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;
    let mut file = std::fs::File::create(&temp_zip)
        .map_err(|e| format!("创建临时文件失败: {}", e))?;
    let mut stream = resp.bytes_stream();
    use futures::StreamExt;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("读取下载数据失败: {}", e))?;
        std::io::Write::write_all(&mut file, &chunk)
            .map_err(|e| format!("写入文件失败: {}", e))?;
        downloaded += chunk.len() as u64;
        if total_size > 0 {
            let progress = 0.1 + 0.6 * (downloaded as f64 / total_size as f64);
            let _ = app.emit("asset-import-progress", AssetImportProgressPayload {
                asset_id: asset_id.clone(),
                stage: "downloading".to_string(),
                progress,
                message: format!("正在下载 {} ({:.0}/{:.0} KB)...", asset_name, downloaded as f64 / 1024.0, total_size as f64 / 1024.0),
            });
        }
    }
    drop(file);

    let _ = app.emit("asset-import-progress", AssetImportProgressPayload {
        asset_id: asset_id.clone(),
        stage: "extracting".to_string(),
        progress: 0.7,
        message: format!("正在解压 {}...", asset_name),
    });

    let file = std::fs::File::open(&temp_zip)
        .map_err(|e| format!("打开压缩文件失败: {}", e))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| format!("解压失败: {}", e))?;

    let total_entries = archive.len();
    for i in 0..total_entries {
        let mut entry = archive.by_index(i).map_err(|e| format!("读取压缩条目失败: {}", e))?;
        let outpath = match entry.enclosed_name() {
            Some(path) => payload_dir.join(path),
            None => continue,
        };
        if entry.is_dir() {
            std::fs::create_dir_all(&outpath).ok();
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(p).ok();
                }
            }
            let mut outfile = std::fs::File::create(&outpath)
                .map_err(|e| format!("创建文件失败: {}", e))?;
            std::io::copy(&mut entry, &mut outfile)
                .map_err(|e| format!("写入文件失败: {}", e))?;
        }
        let progress = 0.7 + 0.2 * ((i + 1) as f64 / total_entries as f64);
        let _ = app.emit("asset-import-progress", AssetImportProgressPayload {
            asset_id: asset_id.clone(),
            stage: "extracting".to_string(),
            progress,
            message: format!("正在解压 {} ({}/{})...", asset_name, i + 1, total_entries),
        });
    }

    let _ = std::fs::remove_file(&temp_zip);

    let _ = app.emit("asset-import-progress", AssetImportProgressPayload {
        asset_id: asset_id.clone(),
        stage: "parsing".to_string(),
        progress: 0.9,
        message: format!("正在解析插件 {}...", asset_name),
    });

    let manager = get_plugin_manager(&app);
    let (units, asset_type) = manager.analyze_asset_type(&payload_dir, &asset_name);

    let compatibility = manager.detect_compatibility(&payload_dir);

    let content_hash = crate::models::compute_dir_hash(&payload_dir).unwrap_or_default();

    let (unit_version, unit_name) = if let Some(first_unit) = units.first() {
        (
            if first_unit.version.is_empty() { "1.0.0".to_string() } else { first_unit.version.clone() },
            if first_unit.name.is_empty() { asset_name.clone() } else { first_unit.name.clone() },
        )
    } else {
        ("1.0.0".to_string(), asset_name.clone())
    };

    let plugin_version = PluginVersion {
        version_id: version_id.clone(),
        version: unit_version,
        path: payload_dir.to_string_lossy().to_string(),
        created_at: chrono::Utc::now(),
        units,
    };

    plugin.versions.push(plugin_version);
    plugin.compatibility = compatibility;
    plugin.name = unit_name;
    plugin.content_hash = content_hash;
    plugin.asset_type = asset_type;

    let result = upsert_plugin(&app, &plugin, "import_asset_library", &asset_id.to_string())?;

    let _ = app.emit("asset-import-progress", AssetImportProgressPayload {
        asset_id: asset_id.clone(),
        stage: "complete".to_string(),
        progress: 1.0,
        message: format!("{} 导入完成", result.name),
    });

    Ok(result)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectImportResult {
    pub project_id: String,
    pub name: String,
    pub path: String,
    pub godot_version: String,
}

#[tauri::command]
pub async fn import_project_from_asset_library(app: AppHandle, asset_id: u64, target_dir: String) -> Result<ProjectImportResult, String> {
    let base_url = crate::utils::get_asset_library_base(&app);
    let client = create_http_client(None).map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let detail_url = format!("{}/asset/{}", base_url, asset_id);
    let detail_resp = client.get(&detail_url)
        .send().await
        .map_err(|e| format!("获取资产详情失败: {}", e))?;
    let detail: serde_json::Value = detail_resp.json().await
        .map_err(|e| format!("解析资产详情失败: {}", e))?;

    let download_url = detail["download_url"].as_str().unwrap_or("");
    let asset_title = detail["title"].as_str().unwrap_or("Unknown").to_string();

    if download_url.is_empty() {
        return Err("资产没有可用的下载链接".to_string());
    }

    let temp_dir = std::env::temp_dir().join(format!("godot_harbor_project_{}", asset_id));
    let _ = fs::create_dir_all(&temp_dir);
    let temp_zip = temp_dir.join("download.zip");

    let mut resp = client.get(download_url).send().await
        .map_err(|e| format!("下载失败: {}", e))?;

    {
        let mut file = fs::File::create(&temp_zip)
            .map_err(|e| format!("创建临时文件失败: {}", e))?;
        use std::io::Write;
        while let Some(chunk) = resp.chunk().await.map_err(|e| format!("下载中断: {}", e))? {
            file.write_all(&chunk).map_err(|e| format!("写入失败: {}", e))?;
        }
    }

    let project_dir = Path::new(&target_dir).join(&asset_title);
    let _ = fs::create_dir_all(&project_dir);

    {
        let file = fs::File::open(&temp_zip).map_err(|e| format!("打开 ZIP 失败: {}", e))?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("解析 ZIP 失败: {}", e))?;

        let mut all_prefix: Option<String> = None;
        let mut file_count = 0u32;
        for i in 0..archive.len() {
            let entry = archive.by_index(i).map_err(|e| format!("读取 ZIP 条目失败: {}", e))?;
            let name = entry.name().to_string();
            if name.ends_with('/') { continue; }
            file_count += 1;
            let slash_pos = name.find('/').unwrap_or(name.len());
            let prefix = &name[..slash_pos];
            if file_count == 1 {
                all_prefix = Some(prefix.to_string());
            } else if all_prefix.as_deref() != Some(prefix) {
                all_prefix = None;
                break;
            }
        }

        let strip_depth = if all_prefix.is_some() && file_count > 0 { 1usize } else { 0usize };

        for i in 0..archive.len() {
            let mut entry = archive.by_index(i).map_err(|e| format!("读取 ZIP 条目失败: {}", e))?;
            let name = entry.name().to_string();
            let mut path_parts: Vec<&str> = name.split('/').collect();
            if path_parts.last().map(|s| s.is_empty()).unwrap_or(false) {
                path_parts.pop();
            }
            if path_parts.is_empty() { continue; }
            if strip_depth > 0 && path_parts.len() <= strip_depth { continue; }
            let stripped: Vec<&str> = path_parts[strip_depth..].to_vec();
            if stripped.is_empty() { continue; }

            let target_path = project_dir.join(stripped.join("/"));

            if entry.is_dir() {
                let _ = fs::create_dir_all(&target_path);
            } else {
                if let Some(parent) = target_path.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                let mut file = fs::File::create(&target_path)
                    .map_err(|e| format!("创建文件失败: {}", e))?;
                std::io::copy(&mut entry, &mut file)
                    .map_err(|e| format!("写入文件失败: {}", e))?;
            }
        }
    }

    let _ = fs::remove_dir_all(&temp_dir);

    let project_godot_path = find_project_godot(&project_dir);
    let (name, godot_version) = if let Some(pg_path) = &project_godot_path {
        parse_project_godot(pg_path)
    } else {
        (asset_title.clone(), String::new())
    };

    let project = Project {
        project_id: Uuid::new_v4().to_string(),
        name,
        path: project_dir.to_string_lossy().to_string(),
        godot_version,
        icon_path: String::new(),
        group: String::new(),
        status: ProjectStatus::Ready,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        last_synced_at: None,
    };

    let project_id = project.project_id.clone();
    let project_name = project.name.clone();
    let project_path = project.path.clone();
    let project_godot_version = project.godot_version.clone();

    let storage = get_storage(&app);
    let mut projects: Vec<Project> = storage.load_or_default("projects.json");
    projects.push(project);
    storage.save("projects.json", &projects)
        .map_err(|e| format!("保存项目列表失败: {}", e))?;

    log_operation(&app, "import_project_from_asset_library", "", &format!("从 Asset Library 导入项目: {}", project_name));

    Ok(ProjectImportResult {
        project_id,
        name: project_name,
        path: project_path,
        godot_version: project_godot_version,
    })
}

use std::path::Path;

fn find_project_godot(dir: &Path) -> Option<std::path::PathBuf> {
    for entry in walkdir::WalkDir::new(dir)
        .max_depth(3)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_name() == "project.godot" {
            return Some(entry.path().to_path_buf());
        }
    }
    None
}

fn parse_project_godot(path: &Path) -> (String, String) {
    let content = fs::read_to_string(path).unwrap_or_default();
    let mut name = String::new();
    let mut version = String::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("config/name=") {
            name = trimmed[12..].trim_matches('"').to_string();
        }
        if trimmed.starts_with("config/features=") {
            if trimmed.contains("4.") {
                version = "4".to_string();
            } else if trimmed.contains("3.") {
                version = "3".to_string();
            }
        }
    }
    if name.is_empty() {
        name = path.parent()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
    }
    (name, version)
}


