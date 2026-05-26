use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use crate::asset_store::AssetStoreClient;
use crate::utils::create_http_client;
use super::plugin::AssetLibrarySearchParams;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssetApiMode {
    Auto,
    Legacy,
    NewStore,
}

impl AssetApiMode {
    pub fn from_str(s: &str) -> Self {
        match s {
            "legacy" => Self::Legacy,
            "new_store" => Self::NewStore,
            _ => Self::Auto,
        }
    }
}

fn get_api_mode(app: &AppHandle) -> AssetApiMode {
    let storage = crate::commands::get_storage(app);
    let settings: crate::models::Settings = storage.load_or_default("settings.json");
    AssetApiMode::from_str(&settings.asset_api_mode)
}

async fn is_new_store_available(app: &AppHandle) -> bool {
    let client = match AssetStoreClient::new(app) {
        Ok(c) => c,
        Err(_) => return false,
    };
    client.check_available().await
}

#[tauri::command]
pub async fn search_assets(
    app: AppHandle,
    params: AssetLibrarySearchParams,
) -> Result<serde_json::Value, String> {
    let mode = get_api_mode(&app);

    let use_new = match mode {
        AssetApiMode::Auto => is_new_store_available(&app).await,
        AssetApiMode::NewStore => true,
        AssetApiMode::Legacy => false,
    };

    if use_new {
        search_new_store(&app, &params).await
    } else {
        search_legacy(&app, &params).await
    }
}

async fn search_new_store(
    app: &AppHandle,
    params: &AssetLibrarySearchParams,
) -> Result<serde_json::Value, String> {
    let client = AssetStoreClient::new(app)?;
    let query = params.filter.as_deref().unwrap_or("");
    let page = params.page.unwrap_or(1);
    let max_results = params.max_results.unwrap_or(20);
    let sort = params.sort.as_deref().unwrap_or("updated");

    client.search(
        query,
        page,
        max_results,
        sort,
        params.category.as_deref(),
        params.godot_version.as_deref(),
        None,
    ).await
}

async fn search_legacy(
    app: &AppHandle,
    params: &AssetLibrarySearchParams,
) -> Result<serde_json::Value, String> {
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

    let asset_lib_base = crate::utils::get_asset_library_base(app);
    let url = format!("{}/asset?{}", asset_lib_base, url_params.join("&"));

    let client = create_http_client(None)?;
    let resp = client.get(&url).send().await
        .map_err(|e| format!("请求 Asset Library 失败: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Asset Library 返回错误状态: {}", resp.status()));
    }

    let text = resp.text().await
        .map_err(|e| format!("读取 Asset Library 响应失败: {}", e))?;

    serde_json::from_str(&text)
        .map_err(|e| format!("解析 Asset Library 响应失败: {} (响应前100字符: {})", e, &text[..text.len().min(100)]))
}

#[tauri::command]
pub async fn get_asset_detail_v2(
    app: AppHandle,
    asset_id: String,
) -> Result<serde_json::Value, String> {
    let mode = get_api_mode(&app);

    let use_new = match mode {
        AssetApiMode::Auto => is_new_store_available(&app).await,
        AssetApiMode::NewStore => true,
        AssetApiMode::Legacy => false,
    };

    if use_new {
        let client = AssetStoreClient::new(&app)?;
        client.get_detail(&asset_id).await
    } else {
        let asset_lib_base = crate::utils::get_asset_library_base(&app);
        let url = format!("{}/asset/{}", asset_lib_base, asset_id);
        let client = create_http_client(None)?;
        let resp = client.get(&url).send().await
            .map_err(|e| format!("请求 Asset Library 失败: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("Asset Library 返回错误状态: {}", resp.status()));
        }
        resp.json().await
            .map_err(|e| format!("解析 Asset Library 响应失败: {}", e))
    }
}

#[tauri::command]
pub async fn get_asset_store_categories(app: AppHandle) -> Result<serde_json::Value, String> {
    let client = AssetStoreClient::new(&app)?;
    client.get_categories().await
}

#[tauri::command]
pub async fn check_asset_api_availability(app: AppHandle) -> Result<serde_json::Value, String> {
    let new_available = is_new_store_available(&app).await;

    let legacy_available = {
        let asset_lib_base = crate::utils::get_asset_library_base(&app);
        let client = create_http_client(Some(std::time::Duration::from_secs(10)));
        match client {
            Ok(c) => {
                let url = format!("{}/asset?max_results=1", asset_lib_base);
                c.get(&url).send().await
                    .map(|r| r.status().is_success())
                    .unwrap_or(false)
            }
            Err(_) => false,
        }
    };

    Ok(serde_json::json!({
        "new_store_available": new_available,
        "legacy_available": legacy_available,
        "recommended_mode": if new_available { "new_store" } else if legacy_available { "legacy" } else { "none" }
    }))
}
