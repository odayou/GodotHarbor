use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};

const GODOT4_API_URL: &str = "https://api.github.com/repos/godotengine/godot/releases?per_page=20";
const GODOT3_API_URL: &str = "https://api.github.com/repos/godotengine/godot/releases?per_page=50";
const CACHE_DURATION_SECS: u64 = 3600;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GodotReleaseInfo {
    pub version: String,
    pub tag_name: String,
    pub release_url: String,
    pub release_notes: String,
    pub published_at: String,
    pub is_stable: bool,
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GodotVersionCheckResult {
    pub latest_godot4: Option<GodotReleaseInfo>,
    pub latest_godot3: Option<GodotReleaseInfo>,
    pub local_engines: Vec<LocalEngineVersion>,
    pub updates_available: Vec<VersionUpdateInfo>,
    pub checked_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalEngineVersion {
    pub engine_id: String,
    pub name: String,
    pub version: String,
    pub engine_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionUpdateInfo {
    pub engine_id: String,
    pub engine_name: String,
    pub current_version: String,
    pub latest_version: String,
    pub download_url: String,
    pub release_notes: String,
    pub is_major_update: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedVersionInfo {
    releases: Vec<GodotReleaseInfo>,
    cached_at: u64,
}

pub struct VersionChecker {
    cache_dir: std::path::PathBuf,
}

impl VersionChecker {
    pub fn new(cache_dir: std::path::PathBuf) -> Self {
        std::fs::create_dir_all(&cache_dir).ok();
        Self { cache_dir }
    }

    pub async fn check_for_updates(
        &self,
        local_engines: Vec<LocalEngineVersion>,
    ) -> Result<GodotVersionCheckResult, String> {
        let cached = self.load_cache();
        let releases = if let Some(cached) = cached {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            if now - cached.cached_at < CACHE_DURATION_SECS {
                cached.releases
            } else {
                match self.fetch_releases().await {
                    Ok(releases) => {
                        self.save_cache(&releases);
                        releases
                    }
                    Err(_) => cached.releases,
                }
            }
        } else {
            match self.fetch_releases().await {
                Ok(releases) => {
                    self.save_cache(&releases);
                    releases
                }
                Err(e) => return Err(e),
            }
        };

        let latest_godot4 = releases
            .iter()
            .filter(|r| r.major == 4 && r.is_stable)
            .max_by(|a, b| {
                (a.minor, a.patch).cmp(&(b.minor, b.patch))
            })
            .cloned();

        let latest_godot3 = releases
            .iter()
            .filter(|r| r.major == 3 && r.is_stable)
            .max_by(|a, b| {
                (a.minor, a.patch).cmp(&(b.minor, b.patch))
            })
            .cloned();

        let mut updates_available = Vec::new();

        for engine in &local_engines {
            let (major, minor, patch) = Self::parse_version(&engine.version);
            let latest = if major >= 4 {
                latest_godot4.as_ref()
            } else {
                latest_godot3.as_ref()
            };

            if let Some(latest_release) = latest {
                if Self::is_newer(major, minor, patch, latest_release.major, latest_release.minor, latest_release.patch) {
                    let is_major_update = latest_release.major > major;
                    updates_available.push(VersionUpdateInfo {
                        engine_id: engine.engine_id.clone(),
                        engine_name: engine.name.clone(),
                        current_version: engine.version.clone(),
                        latest_version: latest_release.version.clone(),
                        download_url: latest_release.release_url.clone(),
                        release_notes: latest_release.release_notes.chars().take(300).collect(),
                        is_major_update,
                    });
                }
            }
        }

        Ok(GodotVersionCheckResult {
            latest_godot4,
            latest_godot3,
            local_engines,
            updates_available,
            checked_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    async fn fetch_releases(&self) -> Result<Vec<GodotReleaseInfo>, String> {
        let client = reqwest::Client::builder()
            .user_agent("GodotHarbor")
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

        let mut all_releases = Vec::new();

        for url in &[GODOT4_API_URL, GODOT3_API_URL] {
            match client.get(*url).send().await {
                Ok(resp) => {
                    if resp.status().is_success() {
                        if let Ok(json) = resp.json::<serde_json::Value>().await {
                            if let Some(arr) = json.as_array() {
                                for release in arr {
                                    if let Some(info) = Self::parse_release(release) {
                                        all_releases.push(info);
                                    }
                                }
                            }
                        }
                    }
                }
                Err(_) => continue,
            }
        }

        if all_releases.is_empty() {
            return Err("无法获取 Godot 版本信息，请检查网络连接".to_string());
        }

        Ok(all_releases)
    }

    fn parse_release(release: &serde_json::Value) -> Option<GodotReleaseInfo> {
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
        let (major, minor, patch) = Self::parse_version(version_str);

        let is_stable = !prerelease
            && !version_str.contains("dev")
            && !version_str.contains("beta")
            && !version_str.contains("rc")
            && !version_str.contains("alpha");

        Some(GodotReleaseInfo {
            version: version_str.to_string(),
            tag_name: tag_name.clone(),
            release_url: html_url,
            release_notes: body,
            published_at,
            is_stable,
            major,
            minor,
            patch,
        })
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

    fn is_newer(cur_major: u32, cur_minor: u32, cur_patch: u32, new_major: u32, new_minor: u32, new_patch: u32) -> bool {
        if new_major > cur_major { return true; }
        if new_major == cur_major && new_minor > cur_minor { return true; }
        if new_major == cur_major && new_minor == cur_minor && new_patch > cur_patch { return true; }
        false
    }

    fn cache_path(&self) -> std::path::PathBuf {
        self.cache_dir.join("godot_version_cache.json")
    }

    fn load_cache(&self) -> Option<CachedVersionInfo> {
        let path = self.cache_path();
        if !path.exists() { return None; }
        let content = std::fs::read_to_string(&path).ok()?;
        serde_json::from_str(&content).ok()
    }

    fn save_cache(&self, releases: &[GodotReleaseInfo]) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let cached = CachedVersionInfo {
            releases: releases.to_vec(),
            cached_at: now,
        };

        if let Ok(json) = serde_json::to_string(&cached) {
            let _ = std::fs::write(self.cache_path(), json);
        }
    }
}
