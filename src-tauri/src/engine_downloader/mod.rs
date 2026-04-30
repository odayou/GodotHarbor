use crate::models::{
    EngineMirrorConfig, EngineReleaseChannel, RemoteEngineVersion, EngineDownloadProgress,
};
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter, Manager};
use std::sync::atomic::{AtomicBool, Ordering};
use std::collections::HashMap;
use std::sync::Mutex;

static CANCEL_FLAGS: once_cell::sync::Lazy<Mutex<HashMap<String, AtomicBool>>> =
    once_cell::sync::Lazy::new(|| Mutex::new(HashMap::new()));

pub fn request_cancel_download(version: &str) {
    if let Ok(map) = CANCEL_FLAGS.lock() {
        if let Some(flag) = map.get(version) {
            flag.store(true, Ordering::SeqCst);
        }
    }
}

fn is_cancelled(version: &str) -> bool {
    if let Ok(map) = CANCEL_FLAGS.lock() {
        if let Some(flag) = map.get(version) {
            return flag.load(Ordering::SeqCst);
        }
    }
    false
}

fn reset_cancel(version: &str) {
    if let Ok(mut map) = CANCEL_FLAGS.lock() {
        map.insert(version.to_string(), AtomicBool::new(false));
    }
}

fn remove_cancel_flag(version: &str) {
    if let Ok(mut map) = CANCEL_FLAGS.lock() {
        map.remove(version);
    }
}

fn classify_channel(version: &str) -> EngineReleaseChannel {
    let lower = version.to_lowercase();
    if lower.contains("dev") {
        EngineReleaseChannel::Dev
    } else if lower.contains("alpha") {
        EngineReleaseChannel::Alpha
    } else if lower.contains("beta") {
        EngineReleaseChannel::Beta
    } else if lower.contains("rc") {
        EngineReleaseChannel::Rc
    } else {
        EngineReleaseChannel::Stable
    }
}

fn parse_version(version: &str) -> (u32, u32, u32) {
    let clean = version
        .split('-')
        .next()
        .unwrap_or(version)
        .trim();
    let parts: Vec<&str> = clean.split('.').collect();
    let major = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
    let minor = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    let patch = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
    (major, minor, patch)
}

fn platform_keywords() -> Vec<&'static str> {
    if cfg!(target_os = "windows") {
        vec!["win64", "windows", "win"]
    } else if cfg!(target_os = "macos") {
        vec!["macos", "mac", "darwin"]
    } else {
        vec!["linux", "x86_64"]
    }
}

fn platform_extension() -> &'static str {
    if cfg!(target_os = "windows") {
        ".zip"
    } else if cfg!(target_os = "macos") {
        ".zip"
    } else {
        ".zip"
    }
}

fn is_platform_asset(name: &str) -> bool {
    let lower = name.to_lowercase();
    let keywords = platform_keywords();
    let ext = platform_extension();

    if !lower.ends_with(ext) {
        return false;
    }

    if lower.contains("template") || lower.contains("server") || lower.contains("headless") {
        return false;
    }

    if lower.contains("console") {
        return false;
    }

    if cfg!(target_os = "windows") {
        if lower.contains("_win32.") || lower.contains("_x86_32.") || lower.contains("_32.") {
            return false;
        }
    }

    if cfg!(target_os = "macos") {
        if lower.contains("_universal") {
            return true;
        }
        if lower.contains("_arm64") && !lower.contains("_x86_64") {
            return true;
        }
        if lower.contains("_x86_64") && !lower.contains("_arm64") {
            return true;
        }
    }

    keywords.iter().any(|kw| lower.contains(kw))
}

pub struct EngineDownloader;

impl EngineDownloader {
    pub async fn fetch_remote_versions(
        mirror: &EngineMirrorConfig,
        local_versions: &[String],
    ) -> Result<Vec<RemoteEngineVersion>, String> {
        let client = reqwest::Client::builder()
            .user_agent("GodotHarbor")
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

        let mut all_versions = Vec::new();
        let max_pages = 3;
        let per_page = 50;

        let api_base = if mirror.mirror_type == "direct" {
            "https://api.github.com"
        } else {
            mirror.base_url.trim_end_matches('/')
        };

        for repo in &["godotengine/godot", "godotengine/godot-builds"] {
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
                                    if let Some(version) = Self::parse_remote_release(release, mirror, local_versions) {
                                        all_versions.push(version);
                                    }
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
        }

        all_versions.sort_by(|a, b| {
            match b.major.cmp(&a.major) {
                std::cmp::Ordering::Equal => match b.minor.cmp(&a.minor) {
                    std::cmp::Ordering::Equal => b.patch.cmp(&a.patch),
                    other => other,
                },
                other => other,
            }
        });

        let mut seen = std::collections::HashSet::new();
        all_versions.retain(|v| seen.insert(v.version.clone()));

        Ok(all_versions)
    }

    fn parse_remote_release(
        release: &serde_json::Value,
        mirror: &EngineMirrorConfig,
        local_versions: &[String],
    ) -> Option<RemoteEngineVersion> {
        let tag_name = release.get("tag_name")?.as_str()?.to_string();
        let html_url = release.get("html_url")?.as_str()?.to_string();
        let body = release.get("body")
            .and_then(|b| b.as_str())
            .unwrap_or("")
            .to_string();
        let published_at = release.get("published_at")
            .and_then(|p| p.as_str())
            .unwrap_or("")
            .to_string();

        let prerelease = release.get("prerelease")
            .and_then(|p| p.as_bool())
            .unwrap_or(true);

        let draft = release.get("draft")
            .and_then(|d| d.as_bool())
            .unwrap_or(false);

        if draft {
            return None;
        }

        let version_str = tag_name.trim_start_matches('v');
        let (major, minor, patch) = parse_version(version_str);
        let channel = classify_channel(version_str);
        let is_stable = !prerelease
            && !version_str.contains("dev")
            && !version_str.contains("beta")
            && !version_str.contains("rc")
            && !version_str.contains("alpha");

        let assets = release.get("assets")
            .and_then(|a| a.as_array())
            .cloned()
            .unwrap_or_default();

        let mut best_asset: Option<(String, String, u64)> = None;

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

            if is_platform_asset(name) {
                let name_lower = name.to_lowercase();
                let is_preferred = if cfg!(target_os = "macos") {
                    name_lower.contains("universal")
                } else {
                    true
                };

                match &best_asset {
                    None => {
                        best_asset = Some((name.to_string(), download_url.to_string(), size));
                    }
                    Some((_, _, _)) if is_preferred => {
                        best_asset = Some((name.to_string(), download_url.to_string(), size));
                    }
                    _ => {}
                }
            }
        }

        let (file_name, download_url, file_size) = best_asset?;

        let is_installed = local_versions.iter().any(|lv| {
            let local_clean = lv.trim().to_lowercase();
            let remote_clean = version_str.trim().to_lowercase();
            local_clean == remote_clean
                || local_clean.starts_with(&remote_clean)
                || remote_clean.starts_with(&local_clean)
        });

        let final_download_url = if mirror.mirror_type == "direct" {
            let version_dir = version_str.replace('.', "_");
            format!("{}/{}/{}",
                mirror.base_url.trim_end_matches('/'),
                version_dir,
                file_name
            )
        } else {
            download_url
        };

        Some(RemoteEngineVersion {
            version: version_str.to_string(),
            tag_name,
            channel,
            major,
            minor,
            patch,
            is_stable,
            published_at,
            release_url: html_url,
            release_notes: body.chars().take(500).collect(),
            download_url: final_download_url,
            file_name,
            file_size,
            is_installed,
        })
    }

    pub async fn download_and_install(
        app: &AppHandle,
        remote_version: &RemoteEngineVersion,
        engines_dir: PathBuf,
    ) -> Result<PathBuf, String> {
        reset_cancel(&remote_version.version);

        let version_dir_name = format!("godot_{}", remote_version.version.replace('.', "_"));
        let target_dir = engines_dir.join(&version_dir_name);

        if target_dir.exists() {
            return Ok(target_dir);
        }

        let download_dir = app.path().app_data_dir()
            .map_err(|e| format!("获取应用数据目录失败: {}", e))?
            .join("downloads");
        std::fs::create_dir_all(&download_dir)
            .map_err(|e| format!("创建下载目录失败: {}", e))?;

        let archive_path = download_dir.join(&remote_version.file_name);

        let download_result = Self::download_file(app, &remote_version.download_url, &archive_path, &remote_version.version, remote_version.file_size).await;

        if let Err(e) = download_result {
            let _ = std::fs::remove_file(&archive_path);
            remove_cancel_flag(&remote_version.version);
            return Err(e);
        }

        if is_cancelled(&remote_version.version) {
            let _ = std::fs::remove_file(&archive_path);
            remove_cancel_flag(&remote_version.version);
            return Err("下载已取消".to_string());
        }

        Self::emit_progress(app, &remote_version.version, "extracting", 0.0, "正在解压引擎文件...", 0, 0);

        std::fs::create_dir_all(&target_dir)
            .map_err(|e| format!("创建引擎目录失败: {}", e))?;

        let extract_result = Self::extract_archive(app, &remote_version.version, &archive_path, &target_dir);

        let _ = std::fs::remove_file(&archive_path);

        if let Err(e) = extract_result {
            let _ = std::fs::remove_dir_all(&target_dir);
            remove_cancel_flag(&remote_version.version);
            return Err(e);
        }

        Self::emit_progress(app, &remote_version.version, "complete", 100.0, "引擎下载安装完成", 0, 0);

        remove_cancel_flag(&remote_version.version);

        Ok(target_dir)
    }

    async fn download_file(
        app: &AppHandle,
        url: &str,
        path: &Path,
        version: &str,
        total_size: u64,
    ) -> Result<(), String> {
        Self::emit_progress(app, version, "downloading", 0.0, "正在下载引擎...", 0, total_size);

        let client = reqwest::Client::builder()
            .user_agent("GodotHarbor")
            .timeout(std::time::Duration::from_secs(300))
            .build()
            .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

        let max_retries = 3;
        let mut attempt = 0;

        loop {
            attempt += 1;
            let mut response = match client.get(url).send().await {
                Ok(resp) => resp,
                Err(e) => {
                    if attempt < max_retries && (e.is_connect() || e.is_timeout() || e.is_request()) {
                        let msg = format!("下载请求失败，第 {} 次重试...", attempt);
                        Self::emit_progress(app, version, "downloading", 0.0, &msg, 0, total_size);
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
                    Self::emit_progress(app, version, "downloading", 0.0, &msg, 0, total_size);
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
                if is_cancelled(version) {
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

                        Self::emit_progress(app, version, "downloading", progress, &msg, downloaded, total);
                    }
                    None => break,
                }
            }

            file.flush()
                .map_err(|e| format!("刷新文件失败: {}", e))?;

            return Ok(());
        }
    }

    fn extract_archive(app: &AppHandle, version: &str, archive_path: &Path, target_dir: &Path) -> Result<(), String> {
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
                Self::emit_progress(app, version, "extracting", progress, &msg, 0, 0);
            }
        }

        Ok(())
    }

    fn emit_progress(app: &AppHandle, version: &str, stage: &str, progress: f64, message: &str, downloaded_bytes: u64, total_bytes: u64) {
        let progress_info = EngineDownloadProgress {
            version: version.to_string(),
            stage: stage.to_string(),
            downloaded_bytes,
            total_bytes,
            progress,
            message: message.to_string(),
        };
        let _ = app.emit("engine-download-progress", &progress_info);
    }
}
