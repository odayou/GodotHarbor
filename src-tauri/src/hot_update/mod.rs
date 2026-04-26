use crate::models::HotUpdateInfo;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotUpdateManifest {
    pub version: String,
    pub min_compatible_app_version: String,
    pub max_compatible_app_version: String,
    pub release_notes: String,
    pub pub_date: String,
    pub download_url: String,
    pub download_size: u64,
    pub checksum: String,
    pub files: Vec<HotUpdateFileEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotUpdateFileEntry {
    pub path: String,
    pub checksum: String,
    pub size: u64,
}

pub struct HotUpdateManager {
    data_dir: PathBuf,
}

impl HotUpdateManager {
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            data_dir,
        }
    }

    fn hot_update_dir(&self) -> PathBuf {
        self.data_dir.join("hot_updates")
    }

    fn current_version_file(&self) -> PathBuf {
        self.hot_update_dir().join("current_version.json")
    }

    fn staging_dir(&self) -> PathBuf {
        self.hot_update_dir().join("staging")
    }

    fn backup_dir(&self) -> PathBuf {
        self.hot_update_dir().join("backup")
    }

    pub fn overlay_dir(&self) -> PathBuf {
        self.data_dir.join("hotupdate_overlay")
    }

    pub fn check_for_hot_update(&self, manifest_url: &str, current_app_version: &str) -> Result<Option<HotUpdateInfo>, String> {
        let client = reqwest::blocking::ClientBuilder::new()
            .user_agent("GodotHarbor")
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .map_err(|e| format!("创建HTTP客户端失败: {}", e))?;

        let resp = client.get(manifest_url).send()
            .map_err(|e| format!("请求热更新清单失败: {}", e))?;

        if !resp.status().is_success() {
            return Ok(None);
        }

        let manifest: HotUpdateManifest = resp.json()
            .map_err(|e| format!("解析热更新清单失败: {}", e))?;

        let is_compatible = self.is_version_compatible(
            current_app_version,
            &manifest.min_compatible_app_version,
            &manifest.max_compatible_app_version,
        );

        if !is_compatible {
            return Ok(None);
        }

        let current = self.get_current_hot_update_version()?;
        if current.as_deref() == Some(manifest.version.as_str()) {
            return Ok(None);
        }

        Ok(Some(HotUpdateInfo {
            version: manifest.version.clone(),
            min_compatible_app_version: manifest.min_compatible_app_version.clone(),
            max_compatible_app_version: manifest.max_compatible_app_version.clone(),
            release_notes: manifest.release_notes.clone(),
            pub_date: manifest.pub_date.clone(),
            download_size: manifest.download_size,
            checksum: manifest.checksum.clone(),
            download_url: manifest.download_url.clone(),
        }))
    }

    pub fn download_and_apply(&self, app: &AppHandle, manifest_url: &str) -> Result<(), String> {
        let client = reqwest::blocking::ClientBuilder::new()
            .user_agent("GodotHarbor")
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .map_err(|e| format!("创建HTTP客户端失败: {}", e))?;

        let _ = app.emit("hot-update-progress", serde_json::json!({
            "stage": "downloading",
            "progress": 0,
            "message": "正在下载热更新清单..."
        }));

        let resp = client.get(manifest_url).send()
            .map_err(|e| format!("请求热更新清单失败: {}", e))?;

        let manifest: HotUpdateManifest = resp.json()
            .map_err(|e| format!("解析热更新清单失败: {}", e))?;

        let staging = self.staging_dir();
        if staging.exists() {
            fs::remove_dir_all(&staging)
                .map_err(|e| format!("清理暂存目录失败: {}", e))?;
        }
        fs::create_dir_all(&staging)
            .map_err(|e| format!("创建暂存目录失败: {}", e))?;

        let _ = app.emit("hot-update-progress", serde_json::json!({
            "stage": "downloading",
            "progress": 20,
            "message": "正在下载更新包..."
        }));

        let archive_resp = client.get(&manifest.download_url).send()
            .map_err(|e| format!("下载更新包失败: {}", e))?;

        let archive_data = archive_resp.bytes()
            .map_err(|e| format!("读取更新包数据失败: {}", e))?;

        let _ = app.emit("hot-update-progress", serde_json::json!({
            "stage": "extracting",
            "progress": 60,
            "message": "正在解压更新包..."
        }));

        let archive_path = staging.join("update.zip");
        fs::write(&archive_path, &archive_data)
            .map_err(|e| format!("写入更新包失败: {}", e))?;

        let extract_dir = staging.join("extracted");
        fs::create_dir_all(&extract_dir)
            .map_err(|e| format!("创建解压目录失败: {}", e))?;

        self.extract_zip(&archive_path, &extract_dir)?;

        let _ = app.emit("hot-update-progress", serde_json::json!({
            "stage": "applying",
            "progress": 80,
            "message": "正在应用更新..."
        }));

        let backup = self.backup_dir();
        if backup.exists() {
            fs::remove_dir_all(&backup)
                .map_err(|e| format!("清理旧备份失败: {}", e))?;
        }

        self.backup_current_resources(app)?;

        self.apply_hot_update(app, &extract_dir)?;

        self.save_current_version(&manifest.version)?;

        let _ = app.emit("hot-update-progress", serde_json::json!({
            "stage": "complete",
            "progress": 100,
            "message": "热更新完成，部分更改将在重启后生效"
        }));

        Ok(())
    }

    pub fn rollback(&self, app: &AppHandle) -> Result<(), String> {
        let backup = self.backup_dir();
        if !backup.exists() {
            return Err("没有可回滚的备份".to_string());
        }

        self.apply_hot_update(app, &backup)?;

        let version_file = self.current_version_file();
        if version_file.exists() {
            fs::remove_file(&version_file)
                .map_err(|e| format!("删除版本文件失败: {}", e))?;
        }

        Ok(())
    }

    pub fn get_current_hot_update_version(&self) -> Result<Option<String>, String> {
        let version_file = self.current_version_file();
        if !version_file.exists() {
            return Ok(None);
        }
        let content = fs::read_to_string(&version_file)
            .map_err(|e| format!("读取版本文件失败: {}", e))?;
        let v: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| format!("解析版本文件失败: {}", e))?;
        Ok(v.get("version").and_then(|v| v.as_str()).map(|s| s.to_string()))
    }

    fn save_current_version(&self, version: &str) -> Result<(), String> {
        let dir = self.hot_update_dir();
        fs::create_dir_all(&dir)
            .map_err(|e| format!("创建目录失败: {}", e))?;
        let content = serde_json::json!({
            "version": version,
            "applied_at": chrono::Utc::now().to_rfc3339()
        });
        fs::write(self.current_version_file(), content.to_string())
            .map_err(|e| format!("保存版本文件失败: {}", e))?;
        Ok(())
    }

    fn is_version_compatible(&self, current: &str, min: &str, max: &str) -> bool {
        let current_semver = semver::Version::parse(current.trim_start_matches('v')).ok();
        let min_semver = semver::Version::parse(min.trim_start_matches('v')).ok();
        let max_semver = semver::Version::parse(max.trim_start_matches('v')).ok();

        match (current_semver, min_semver, max_semver) {
            (Some(c), Some(mn), Some(mx)) => c >= mn && c <= mx,
            _ => true,
        }
    }

    fn backup_current_resources(&self, app: &AppHandle) -> Result<(), String> {
        let backup = self.backup_dir();
        fs::create_dir_all(&backup)
            .map_err(|e| format!("创建备份目录失败: {}", e))?;

        let resource_dir = app.path().resource_dir()
            .map_err(|e| format!("获取资源目录失败: {}", e))?;

        let web_assets = resource_dir.join("web");
        if web_assets.exists() {
            let dest = backup.join("web");
            self.copy_dir_recursive(&web_assets, &dest)?;
        }

        Ok(())
    }

    fn apply_hot_update(&self, app: &AppHandle, source_dir: &Path) -> Result<(), String> {
        let resource_dir = app.path().resource_dir()
            .map_err(|e| format!("获取资源目录失败: {}", e))?;

        let overlay = self.overlay_dir();

        for entry in walkdir::WalkDir::new(source_dir)
            .max_depth(2)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                let relative = entry.path().strip_prefix(source_dir)
                    .map_err(|e| format!("路径处理失败: {}", e))?;

                let target = resource_dir.join(relative);
                if let Some(parent) = target.parent() {
                    fs::create_dir_all(parent).ok();
                }
                fs::copy(entry.path(), &target)
                    .map_err(|e| format!("复制文件失败: {}", e))?;

                let overlay_target = overlay.join(relative);
                if let Some(parent) = overlay_target.parent() {
                    fs::create_dir_all(parent).ok();
                }
                fs::copy(entry.path(), &overlay_target)
                    .map_err(|e| format!("复制到overlay失败: {}", e))?;
            }
        }

        Ok(())
    }

    fn copy_dir_recursive(&self, src: &Path, dst: &Path) -> Result<(), String> {
        if !dst.exists() {
            fs::create_dir_all(dst)
                .map_err(|e| format!("创建目录失败: {}", e))?;
        }

        for entry in fs::read_dir(src)
            .map_err(|e| format!("读取目录失败: {}", e))?
        {
            let entry = entry.map_err(|e| format!("读取目录条目失败: {}", e))?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());

            if src_path.is_dir() {
                self.copy_dir_recursive(&src_path, &dst_path)?;
            } else {
                fs::copy(&src_path, &dst_path)
                    .map_err(|e| format!("复制文件失败: {}", e))?;
            }
        }

        Ok(())
    }

    fn extract_zip(&self, zip_path: &Path, dest: &Path) -> Result<(), String> {
        let file = fs::File::open(zip_path)
            .map_err(|e| format!("打开zip文件失败: {}", e))?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| format!("解析zip文件失败: {}", e))?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)
                .map_err(|e| format!("读取zip条目失败: {}", e))?;
            let outpath = match file.enclosed_name() {
                Some(path) => dest.join(path),
                None => continue,
            };

            if file.is_dir() {
                fs::create_dir_all(&outpath)
                    .map_err(|e| format!("创建目录失败: {}", e))?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(p)
                            .map_err(|e| format!("创建目录失败: {}", e))?;
                    }
                }
                let mut outfile = fs::File::create(&outpath)
                    .map_err(|e| format!("创建文件失败: {}", e))?;
                std::io::copy(&mut file, &mut outfile)
                    .map_err(|e| format!("写入文件失败: {}", e))?;
            }
        }

        Ok(())
    }
}
