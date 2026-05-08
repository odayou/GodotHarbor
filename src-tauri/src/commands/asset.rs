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
    let units = match manager.parse_plugin_units(&payload_dir) {
        Ok(u) => u,
        Err(e) => {
            let _ = std::fs::remove_dir_all(&version_dir);
            return Err(format!("解析插件失败: {}，已清理下载文件", e));
        }
    };

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
    let units = match manager.parse_plugin_units(&payload_dir) {
        Ok(u) => u,
        Err(e) => {
            let _ = std::fs::remove_dir_all(&version_dir);
            let _ = app.emit("asset-import-progress", AssetImportProgressPayload {
                asset_id: asset_id.clone(),
                stage: "error".to_string(),
                progress: 0.0,
                message: format!("解析插件失败: {}", e),
            });
            return Err(format!("解析插件失败: {}，已清理下载文件", e));
        }
    };

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

    let result = upsert_plugin(&app, &plugin, "import_asset_library", &asset_id.to_string())?;

    let _ = app.emit("asset-import-progress", AssetImportProgressPayload {
        asset_id: asset_id.clone(),
        stage: "complete".to_string(),
        progress: 1.0,
        message: format!("{} 导入完成", result.name),
    });

    Ok(result)
}


