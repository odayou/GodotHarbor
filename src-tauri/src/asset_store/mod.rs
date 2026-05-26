use crate::utils::create_http_client;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

const ASSET_STORE_BASE: &str = "https://store.godotengine.org";
const ASSET_STORE_API_BASE: &str = "https://store.godotengine.org/api";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetStoreSearchResult {
    pub results: Vec<AssetStoreAsset>,
    pub page: u32,
    pub pages: u32,
    pub total: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetStoreAsset {
    pub asset_id: String,
    pub title: String,
    pub author: String,
    pub author_id: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub godot_version_min: String,
    #[serde(default)]
    pub godot_version_max: String,
    #[serde(default)]
    pub license: String,
    #[serde(default)]
    pub cost: String,
    #[serde(default)]
    pub icon_url: String,
    #[serde(default)]
    pub rating: f64,
    #[serde(default)]
    pub review_count: u32,
    #[serde(default)]
    pub download_url: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub version_string: String,
    #[serde(default)]
    pub modify_date: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub source_url: String,
    #[serde(default)]
    pub store_url: String,
    #[serde(default)]
    pub asset_type: String,
    #[serde(default)]
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetStoreRelease {
    pub version_id: String,
    pub version: String,
    #[serde(default)]
    pub changelog: String,
    #[serde(default)]
    pub download_url: String,
    #[serde(default)]
    pub sha256: String,
    #[serde(default)]
    pub godot_version_min: String,
    #[serde(default)]
    pub godot_version_max: String,
    #[serde(default)]
    pub file_size: u64,
    #[serde(default)]
    pub published_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetStoreDetail {
    pub asset_id: String,
    pub title: String,
    pub author: String,
    pub author_id: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub godot_version_min: String,
    #[serde(default)]
    pub godot_version_max: String,
    #[serde(default)]
    pub license: String,
    #[serde(default)]
    pub cost: String,
    #[serde(default)]
    pub icon_url: String,
    #[serde(default)]
    pub rating: f64,
    #[serde(default)]
    pub review_count: u32,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub source_url: String,
    #[serde(default)]
    pub store_url: String,
    #[serde(default)]
    pub asset_type: String,
    #[serde(default)]
    pub verified: bool,
    #[serde(default)]
    pub releases: Vec<AssetStoreRelease>,
    #[serde(default)]
    pub preview_images: Vec<String>,
}

pub struct AssetStoreClient {
    client: reqwest::Client,
    base_url: String,
}

impl AssetStoreClient {
    pub fn new(app: &AppHandle) -> Result<Self, String> {
        let client = create_http_client(Some(std::time::Duration::from_secs(60)))?;
        let base_url = {
            let storage = crate::commands::get_storage(app);
            let settings: crate::models::Settings = storage.load_or_default("settings.json");
            if !settings.asset_library_mirror.is_empty() {
                settings.asset_library_mirror.trim_end_matches('/').to_string()
            } else {
                ASSET_STORE_API_BASE.to_string()
            }
        };
        Ok(Self { client, base_url })
    }

    pub fn store_page_url(&self, publisher: &str, asset_slug: &str) -> String {
        format!("{}/asset/{}/{}", ASSET_STORE_BASE, publisher, asset_slug)
    }

    pub fn download_url(&self, publisher: &str, asset_slug: &str, version_id: &str) -> String {
        format!("{}/asset/{}/{}/download/{}", ASSET_STORE_BASE, publisher, asset_slug, version_id)
    }

    pub async fn search(
        &self,
        query: &str,
        page: u32,
        max_results: u32,
        sort: &str,
        category: Option<&str>,
        godot_version: Option<&str>,
        tags: Option<&[String]>,
    ) -> Result<serde_json::Value, String> {
        let mut params = vec![
            ("query", query.to_string()),
            ("page", page.to_string()),
            ("max_results", max_results.to_string()),
            ("sort", sort.to_string()),
        ];

        if let Some(cat) = category {
            params.push(("category", cat.to_string()));
        }
        if let Some(gv) = godot_version {
            params.push(("godot_version", gv.to_string()));
        }
        if let Some(tag_list) = tags {
            for tag in tag_list {
                params.push(("tag", tag.clone()));
            }
        }

        let url = format!("{}/asset", self.base_url);
        let resp = self.client.get(&url)
            .query(&params)
            .send()
            .await
            .map_err(|e| format!("请求 Asset Store 搜索失败: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("Asset Store 搜索返回错误: {}", resp.status()));
        }

        resp.json().await
            .map_err(|e| format!("解析 Asset Store 搜索结果失败: {}", e))
    }

    pub async fn get_detail(&self, asset_id: &str) -> Result<serde_json::Value, String> {
        let url = format!("{}/asset/{}", self.base_url, asset_id);
        let resp = self.client.get(&url)
            .send()
            .await
            .map_err(|e| format!("请求 Asset Store 详情失败: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("Asset Store 详情返回错误: {}", resp.status()));
        }

        resp.json().await
            .map_err(|e| format!("解析 Asset Store 详情失败: {}", e))
    }

    pub async fn get_releases(&self, asset_id: &str) -> Result<serde_json::Value, String> {
        let url = format!("{}/asset/{}/releases", self.base_url, asset_id);
        let resp = self.client.get(&url)
            .send()
            .await
            .map_err(|e| format!("请求 Asset Store 版本列表失败: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("Asset Store 版本列表返回错误: {}", resp.status()));
        }

        resp.json().await
            .map_err(|e| format!("解析 Asset Store 版本列表失败: {}", e))
    }

    pub async fn get_categories(&self) -> Result<serde_json::Value, String> {
        let url = format!("{}/category", self.base_url);
        let resp = self.client.get(&url)
            .send()
            .await
            .map_err(|e| format!("请求 Asset Store 分类失败: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("Asset Store 分类返回错误: {}", resp.status()));
        }

        resp.json().await
            .map_err(|e| format!("解析 Asset Store 分类失败: {}", e))
    }

    pub async fn check_available(&self) -> bool {
        let url = format!("{}/asset", self.base_url);
        self.client.get(&url)
            .query(&[("max_results", "1")])
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }
}

pub fn get_asset_store_base(app: &tauri::AppHandle) -> String {
    let storage = crate::commands::get_storage(app);
    let settings: crate::models::Settings = storage.load_or_default("settings.json");
    if !settings.asset_library_mirror.is_empty() {
        settings.asset_library_mirror.trim_end_matches('/').to_string()
    } else {
        ASSET_STORE_API_BASE.to_string()
    }
}
