use crate::models::{
    EngineMirrorConfig, EngineReleaseChannel, RemoteEngineVersion, EngineDownloadProgress,
};
use crate::utils::{create_http_client, parse_version};
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter, Manager};
use std::sync::atomic::{AtomicBool, Ordering};
use std::collections::HashMap;
use std::sync::Mutex;

static CANCEL_FLAGS: once_cell::sync::Lazy<Mutex<HashMap<String, AtomicBool>>> =
    once_cell::sync::Lazy::new(|| Mutex::new(HashMap::new()));

static ACTIVE_DOWNLOADS: once_cell::sync::Lazy<Mutex<HashMap<String, EngineDownloadProgress>>> =
    once_cell::sync::Lazy::new(|| Mutex::new(HashMap::new()));

pub fn get_active_downloads() -> Vec<EngineDownloadProgress> {
    if let Ok(map) = ACTIVE_DOWNLOADS.lock() {
        map.values().cloned().collect()
    } else {
        vec![]
    }
}

fn download_key(version: &str, variant: &str) -> String {
    format!("{}_{}", version, variant)
}

pub fn request_cancel_download(version: &str, variant: &str) {
    let key = download_key(version, variant);
    if let Ok(map) = CANCEL_FLAGS.lock() {
        if let Some(flag) = map.get(&key) {
            flag.store(true, Ordering::SeqCst);
        }
    }
}

fn is_cancelled(version: &str, variant: &str) -> bool {
    let key = download_key(version, variant);
    if let Ok(map) = CANCEL_FLAGS.lock() {
        if let Some(flag) = map.get(&key) {
            return flag.load(Ordering::SeqCst);
        }
    }
    false
}

fn reset_cancel(version: &str, variant: &str) {
    let key = download_key(version, variant);
    if let Ok(mut map) = CANCEL_FLAGS.lock() {
        map.insert(key, AtomicBool::new(false));
    }
}

fn remove_cancel_flag(version: &str, variant: &str) {
    let key = download_key(version, variant);
    if let Ok(mut map) = CANCEL_FLAGS.lock() {
        map.remove(&key);
    }
}

fn clear_active_download(version: &str, variant: &str) {
    let key = download_key(version, variant);
    if let Ok(mut map) = ACTIVE_DOWNLOADS.lock() {
        map.remove(&key);
    }
}

fn cleanup_on_error(path_to_remove: &Path, is_dir: bool, version: &str, variant: &str) {
    if is_dir {
        let _ = std::fs::remove_dir_all(path_to_remove);
    } else {
        let _ = std::fs::remove_file(path_to_remove);
    }
    remove_cancel_flag(version, variant);
    clear_active_download(version, variant);
}

struct TagInfo {
    channel: EngineReleaseChannel,
    channel_number: u32,
    is_stable: bool,
    is_lts: bool,
}

fn parse_tag(tag: &str) -> TagInfo {
    let version_str = tag.trim_start_matches('v');
    let parts: Vec<&str> = version_str.splitn(2, '-').collect();

    let (channel, channel_number, is_stable) = if parts.len() > 1 {
        let suffix = parts[1];
        let lower = suffix.to_lowercase();

        let (prefix, num) = if let Some(rest) = lower.strip_prefix("stable") {
            ("stable", rest.parse().unwrap_or(0))
        } else if let Some(rest) = lower.strip_prefix("rc") {
            ("rc", rest.parse().unwrap_or(0))
        } else if let Some(rest) = lower.strip_prefix("beta") {
            ("beta", rest.parse().unwrap_or(0))
        } else if let Some(rest) = lower.strip_prefix("alpha") {
            ("alpha", rest.parse().unwrap_or(0))
        } else if let Some(rest) = lower.strip_prefix("dev") {
            ("dev", rest.parse().unwrap_or(0))
        } else {
            ("unknown", 0)
        };

        let channel = match prefix {
            "stable" => EngineReleaseChannel::Stable,
            "rc" => EngineReleaseChannel::Rc,
            "beta" => EngineReleaseChannel::Beta,
            "alpha" => EngineReleaseChannel::Alpha,
            "dev" => EngineReleaseChannel::Dev,
            _ => EngineReleaseChannel::Dev,
        };

        (channel, num, prefix == "stable")
    } else {
        (EngineReleaseChannel::Stable, 0, true)
    };

    let major: u32 = parts[0].split('.').next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let is_lts = major == 3;

    TagInfo {
        channel,
        channel_number,
        is_stable,
        is_lts,
    }
}

fn channel_priority(channel: &EngineReleaseChannel) -> u32 {
    match channel {
        EngineReleaseChannel::Stable => 5,
        EngineReleaseChannel::Rc => 4,
        EngineReleaseChannel::Beta => 3,
        EngineReleaseChannel::Alpha => 2,
        EngineReleaseChannel::Dev => 1,
    }
}

fn is_platform_asset(name: &str) -> bool {
    let lower = name.to_lowercase();

    if !lower.ends_with(".zip") {
        return false;
    }

    if lower.contains("template") || lower.contains("server") || lower.contains("headless") {
        return false;
    }

    if lower.contains("console") {
        return false;
    }

    if lower.contains("debug_symbols") || lower.contains("native_debug") {
        return false;
    }

    if lower.contains("export_templates") || lower.contains(".tpz") {
        return false;
    }

    if lower.contains("source") || lower.contains("web_editor") {
        return false;
    }

    if cfg!(target_os = "windows") {
        if lower.contains("_x86_32.") || lower.contains("_x86_32_") || lower.contains("win32") {
            return false;
        }
        if lower.contains("arm64") || lower.contains("arm32") {
            return false;
        }
        lower.contains("win64")
    } else if cfg!(target_os = "macos") {
        if lower.contains("universal") {
            return true;
        }
        if lower.contains("_arm64") && !lower.contains("_x86_64") {
            return true;
        }
        if lower.contains("_x86_64") && !lower.contains("_arm64") {
            return true;
        }
        lower.contains("macos")
    } else {
        lower.contains("linux") && (lower.contains("x86_64") || lower.contains("x86_32") || lower.contains("arm64") || lower.contains("arm32"))
    }
}

pub struct EngineDownloader;

impl EngineDownloader {
    pub async fn fetch_remote_versions(
        mirror: &EngineMirrorConfig,
        local_versions: &[String],
    ) -> Result<Vec<RemoteEngineVersion>, String> {
        let client = create_http_client(Some(std::time::Duration::from_secs(30)))?;

        let mut all_versions = Vec::new();
        let max_pages = 10;
        let per_page = 100;

        let api_base = if mirror.mirror_type == "direct" {
            "https://api.github.com"
        } else {
            mirror.base_url.trim_end_matches('/')
        };

        let repo = "godotengine/godot-builds";
        for page in 1..=max_pages {
            let url = format!(
                "{}/repos/{}/releases?per_page={}&page={}",
                api_base, repo, per_page, page
            );

                let resp = client.get(&url).send().await;
                match resp {
                    Ok(resp) if resp.status().is_success() => {
                        if let Ok(json) = resp.json::<serde_json::Value>().await {
                            if let Some(arr) = json.as_array() {
                                if arr.is_empty() {
                                    break;
                                }
                                for release in arr {
                                    let versions = Self::parse_remote_release(release, mirror, local_versions);
                                    all_versions.extend(versions);
                                }
                                if arr.len() < per_page {
                                    break;
                                }
                            } else {
                                break;
                            }
                        }
                    }
                    Ok(resp) => {
                        let status = resp.status().as_u16();
                        if status == 403 {
                            return Err("RATE_LIMITED".to_string());
                        }
                        eprintln!("获取 {} 返回状态码: {}", url, resp.status());
                        break;
                    }
                    Err(e) => {
                        if e.is_connect() || e.is_timeout() || e.is_request() {
                            return Err("NETWORK_ERROR".to_string());
                        }
                        eprintln!("获取 {} 失败: {}", url, e);
                        break;
                    }
                }
        }

        all_versions.sort_by(|a, b| {
            match b.major.cmp(&a.major) {
                std::cmp::Ordering::Equal => match b.minor.cmp(&a.minor) {
                    std::cmp::Ordering::Equal => match b.patch.cmp(&a.patch) {
                        std::cmp::Ordering::Equal => {
                            let pa = channel_priority(&a.channel);
                            let pb = channel_priority(&b.channel);
                            match pb.cmp(&pa) {
                                std::cmp::Ordering::Equal => b.channel_number.cmp(&a.channel_number),
                                other => other,
                            }
                        }
                        other => other,
                    },
                    other => other,
                },
                other => other,
            }
        });

        let mut seen = std::collections::HashSet::new();
        all_versions.retain(|v| seen.insert(format!("{}_{}", v.version, v.variant)));

        Ok(all_versions)
    }

    fn parse_remote_release(
        release: &serde_json::Value,
        mirror: &EngineMirrorConfig,
        local_versions: &[String],
    ) -> Vec<RemoteEngineVersion> {
        let tag_name = match release.get("tag_name").and_then(|t| t.as_str()) {
            Some(t) => t.to_string(),
            None => return vec![],
        };
        let html_url = release.get("html_url")
            .and_then(|u| u.as_str())
            .unwrap_or("")
            .to_string();
        let body = release.get("body")
            .and_then(|b| b.as_str())
            .unwrap_or("")
            .to_string();
        let published_at = release.get("published_at")
            .and_then(|p| p.as_str())
            .unwrap_or("")
            .to_string();

        let draft = release.get("draft")
            .and_then(|d| d.as_bool())
            .unwrap_or(false);

        if draft {
            return vec![];
        }

        let version_str = tag_name.trim_start_matches('v');
        let tag_info = parse_tag(&tag_name);
        let (major, minor, patch) = parse_version(version_str);

        let assets = release.get("assets")
            .and_then(|a| a.as_array())
            .cloned()
            .unwrap_or_default();

        let mut standard_asset: Option<(String, String, u64)> = None;
        let mut mono_asset: Option<(String, String, u64)> = None;

        for asset in &assets {
            let name = asset.get("name")
                .and_then(|n| n.as_str())
                .unwrap_or("");
            let download_url = asset.get("browser_download_url")
                .and_then(|u| u.as_str())
                .unwrap_or("");
            let size = asset.get("size")
                .and_then(|s| s.as_u64())
                .unwrap_or(0);

            if !is_platform_asset(name) {
                continue;
            }

            let name_lower = name.to_lowercase();
            let is_mono = name_lower.contains("mono");

            let is_preferred = if cfg!(target_os = "macos") {
                name_lower.contains("universal")
            } else {
                true
            };

            if is_mono {
                match &mono_asset {
                    None => {
                        mono_asset = Some((name.to_string(), download_url.to_string(), size));
                    }
                    Some((_, _, _)) if is_preferred => {
                        mono_asset = Some((name.to_string(), download_url.to_string(), size));
                    }
                    _ => {}
                }
            } else {
                match &standard_asset {
                    None => {
                        standard_asset = Some((name.to_string(), download_url.to_string(), size));
                    }
                    Some((_, _, _)) if is_preferred => {
                        standard_asset = Some((name.to_string(), download_url.to_string(), size));
                    }
                    _ => {}
                }
            }
        }

        let version_lower = version_str.trim().to_lowercase();
        let remote_base = version_str.split('-').next().unwrap_or(version_str).trim().to_lowercase();

        let is_installed_standard = local_versions.iter().any(|lv| {
            let local_clean = lv.trim().to_lowercase();
            if local_clean.contains("mono") {
                return false;
            }
            local_clean == version_lower || local_clean == remote_base
        });

        let is_installed_mono = local_versions.iter().any(|lv| {
            let local_clean = lv.trim().to_lowercase();
            if !local_clean.contains("mono") {
                return false;
            }
            let expected_mono = format!("{}-mono", version_lower);
            let expected_mono_base = format!("{}-mono", remote_base);
            local_clean == expected_mono || local_clean == expected_mono_base
        });

        let mut results = Vec::new();

        for (variant, asset_opt, is_installed) in &[
            ("standard", &standard_asset, is_installed_standard),
            ("mono", &mono_asset, is_installed_mono),
        ] {
            if let Some((file_name, download_url, file_size)) = asset_opt {
                let final_download_url = if mirror.mirror_type == "direct" {
                    format!("{}/{}/{}",
                        mirror.base_url.trim_end_matches('/'),
                        version_str,
                        file_name
                    )
                } else {
                    download_url.clone()
                };

                results.push(RemoteEngineVersion {
                    version: version_str.to_string(),
                    tag_name: tag_name.clone(),
                    channel: tag_info.channel.clone(),
                    channel_number: tag_info.channel_number,
                    major,
                    minor,
                    patch,
                    is_stable: tag_info.is_stable,
                    is_lts: tag_info.is_lts,
                    published_at: published_at.clone(),
                    release_url: html_url.clone(),
                    release_notes: body.chars().take(500).collect(),
                    download_url: final_download_url,
                    file_name: file_name.clone(),
                    file_size: *file_size,
                    is_installed: *is_installed,
                    variant: variant.to_string(),
                });
            }
        }

        results
    }

    pub async fn download_and_install(
        app: &AppHandle,
        remote_version: &RemoteEngineVersion,
        engines_dir: PathBuf,
    ) -> Result<PathBuf, String> {
        let variant = &remote_version.variant;
        let version = &remote_version.version;
        reset_cancel(version, variant);

        let version_dir_name = if variant == "mono" {
            format!("godot_{}_dotnet", version.replace('.', "_").replace('-', "_"))
        } else {
            format!("godot_{}", version.replace('.', "_").replace('-', "_"))
        };
        let target_dir = engines_dir.join(&version_dir_name);

        if target_dir.exists() {
            std::fs::remove_dir_all(&target_dir)
                .map_err(|e| format!("删除旧引擎目录失败: {}", e))?;
        }

        let download_dir = app.path().app_data_dir()
            .map_err(|e| format!("获取应用数据目录失败: {}", e))?
            .join("downloads");
        std::fs::create_dir_all(&download_dir)
            .map_err(|e| format!("创建下载目录失败: {}", e))?;

        let archive_name = if variant == "mono" {
            format!("{}_dotnet_{}", version.replace('.', "_").replace('-', "_"), remote_version.file_name)
        } else {
            format!("{}_{}", version.replace('.', "_").replace('-', "_"), remote_version.file_name)
        };
        let archive_path = download_dir.join(&archive_name);

        let download_result = Self::download_file(app, &remote_version.download_url, &archive_path, version, variant, remote_version.file_size).await;

        if let Err(e) = download_result {
            cleanup_on_error(&archive_path, false, version, variant);
            return Err(e);
        }

        if is_cancelled(version, variant) {
            cleanup_on_error(&archive_path, false, version, variant);
            return Err("下载已取消".to_string());
        }

        Self::emit_progress(app, version, variant, "extracting", 0.0, "正在解压引擎文件...", 0, 0);

        std::fs::create_dir_all(&target_dir)
            .map_err(|e| format!("创建引擎目录失败: {}", e))?;

        let extract_result = Self::extract_archive(app, version, variant, &archive_path, &target_dir);

        let _ = std::fs::remove_file(&archive_path);

        if let Err(e) = extract_result {
            cleanup_on_error(&target_dir, true, version, variant);
            return Err(e);
        }

        Self::emit_progress(app, version, variant, "complete", 100.0, "引擎下载安装完成", 0, 0);

        remove_cancel_flag(version, variant);

        Ok(target_dir)
    }

    async fn download_file(
        app: &AppHandle,
        url: &str,
        path: &Path,
        version: &str,
        variant: &str,
        total_size: u64,
    ) -> Result<(), String> {
        Self::emit_progress(app, version, variant, "downloading", 0.0, "正在下载引擎...", 0, total_size);

        let client = create_http_client(Some(std::time::Duration::from_secs(300)))?;

        let max_retries = 3;
        let mut attempt = 0;

        loop {
            attempt += 1;
            let mut response = match client.get(url).send().await {
                Ok(resp) => resp,
                Err(e) => {
                    if attempt < max_retries && (e.is_connect() || e.is_timeout() || e.is_request()) {
                        let msg = format!("下载请求失败，第 {} 次重试...", attempt);
                        Self::emit_progress(app, version, variant, "downloading", 0.0, &msg, 0, total_size);
                        tokio::time::sleep(std::time::Duration::from_secs(2u64.pow(attempt as u32 - 1))).await;
                        continue;
                    }
                    return Err(format!("下载请求失败: {}", e));
                }
            };

            if !response.status().is_success() {
                let status = response.status().as_u16();
                if status >= 500 && attempt < max_retries {
                    let msg = format!("服务器错误 ({}), 第 {} 次重试...", status, attempt);
                    Self::emit_progress(app, version, variant, "downloading", 0.0, &msg, 0, total_size);
                    tokio::time::sleep(std::time::Duration::from_secs(2u64.pow(attempt as u32 - 1))).await;
                    continue;
                }
                return Err(format!("下载失败，HTTP 状态码: {}", response.status()));
            }

            let total = if total_size > 0 {
                total_size
            } else {
                response.content_length().unwrap_or(0)
            };

            let mut file = std::fs::File::create(path)
                .map_err(|e| format!("创建文件失败: {}", e))?;

            let mut downloaded: u64 = 0;

            use std::io::Write;

            loop {
                if is_cancelled(version, variant) {
                    return Err("下载已取消".to_string());
                }

                let chunk = response.chunk().await
                    .map_err(|e| format!("读取下载数据失败: {}", e))?;

                match chunk {
                    Some(data) => {
                        file.write_all(&data)
                            .map_err(|e| format!("写入文件失败: {}", e))?;
                        downloaded += data.len() as u64;

                        let progress = if total > 0 {
                            (downloaded as f64 / total as f64) * 100.0
                        } else {
                            0.0
                        };

                        let size_mb = downloaded as f64 / 1024.0 / 1024.0;
                        let total_mb = total as f64 / 1024.0 / 1024.0;
                        let msg = if total > 0 {
                            format!("正在下载: {:.1}MB / {:.1}MB", size_mb, total_mb)
                        } else {
                            format!("正在下载: {:.1}MB", size_mb)
                        };

                        Self::emit_progress(app, version, variant, "downloading", progress, &msg, downloaded, total);
                    }
                    None => break,
                }
            }

            file.flush()
                .map_err(|e| format!("刷新文件失败: {}", e))?;

            return Ok(());
        }
    }

    fn extract_archive(app: &AppHandle, version: &str, variant: &str, archive_path: &Path, target_dir: &Path) -> Result<(), String> {
        let file = std::fs::File::open(archive_path)
            .map_err(|e| format!("打开压缩包失败: {}", e))?;

        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| format!("解析压缩包失败: {}", e))?;

        let total_entries = archive.len();
        let mut extracted = 0usize;

        for i in 0..total_entries {
            let mut entry = archive.by_index(i)
                .map_err(|e| format!("读取压缩包条目失败: {}", e))?;

            let entry_path = entry.mangled_name();
            let entry_name = entry_path.to_string_lossy().to_string();

            let file_name = entry_name
                .split('/')
                .last()
                .unwrap_or(&entry_name)
                .to_string();

            if file_name.is_empty() || file_name.ends_with('/') {
                extracted += 1;
                continue;
            }

            let dest_path = target_dir.join(&file_name);

            if entry.is_file() {
                if let Some(parent) = dest_path.parent() {
                    std::fs::create_dir_all(parent)
                        .map_err(|e| format!("创建目录失败: {}", e))?;
                }

                let mut outfile = std::fs::File::create(&dest_path)
                    .map_err(|e| format!("创建文件失败: {}", e))?;

                std::io::copy(&mut entry, &mut outfile)
                    .map_err(|e| format!("解压文件失败: {}", e))?;

                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let file_name_lower = file_name.to_lowercase();
                    if file_name_lower.contains("godot") && !file_name_lower.contains(".") {
                        let mut perms = std::fs::metadata(&dest_path)
                            .map_err(|e| format!("获取文件元数据失败: {}", e))?
                            .permissions();
                        perms.set_mode(0o755);
                        std::fs::set_permissions(&dest_path, perms)
                            .map_err(|e| format!("设置文件权限失败: {}", e))?;
                    }
                }
            }

            extracted += 1;
            if extracted % 5 == 0 || extracted == total_entries {
                let progress = (extracted as f64 / total_entries as f64) * 100.0;
                let msg = format!("正在解压: {}/{}", extracted, total_entries);
                Self::emit_progress(app, version, variant, "extracting", progress, &msg, 0, 0);
            }
        }

        Ok(())
    }

    fn emit_progress(app: &AppHandle, version: &str, variant: &str, stage: &str, progress: f64, message: &str, downloaded_bytes: u64, total_bytes: u64) {
        let progress_info = EngineDownloadProgress {
            version: version.to_string(),
            variant: variant.to_string(),
            stage: stage.to_string(),
            downloaded_bytes,
            total_bytes,
            progress,
            message: message.to_string(),
        };
        {
            let key = download_key(version, variant);
            if let Ok(mut map) = ACTIVE_DOWNLOADS.lock() {
                if stage == "complete" {
                    map.remove(&key);
                } else {
                    map.insert(key, progress_info.clone());
                }
            }
        }
        let _ = app.emit("engine-download-progress", &progress_info);
    }
}
