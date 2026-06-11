use tauri::AppHandle;
use crate::plugin_store::*;

#[tauri::command]
pub async fn search_plugin_store(
    app: AppHandle,
    query: String,
    category: Option<String>,
    godot_version: Option<String>,
    sort_by: Option<String>,
    page: Option<u32>,
    page_size: Option<u32>,
) -> Result<StoreSearchResult, String> {
    crate::plugin_store::search_store(
        &app,
        &query,
        category.as_deref(),
        godot_version.as_deref(),
        sort_by.as_deref(),
        page,
        page_size,
    ).await
}

#[tauri::command]
pub async fn get_plugin_store_recommendations(
    app: AppHandle,
    project_id: Option<String>,
) -> Result<Vec<StoreRecommendation>, String> {
    crate::plugin_store::get_store_recommendations(
        &app,
        project_id.as_deref(),
    ).await
}

#[tauri::command]
pub async fn get_plugin_store_categories_with_counts(
    app: AppHandle,
) -> Result<Vec<StoreCategory>, String> {
    crate::plugin_store::get_store_categories_with_counts(&app).await
}

#[tauri::command]
pub async fn one_click_install_plugin(
    app: AppHandle,
    asset_id: i64,
    project_id: String,
    auto_apply: Option<bool>,
) -> Result<OneClickInstallResult, String> {
    crate::plugin_store::one_click_install(
        &app,
        asset_id,
        &project_id,
        auto_apply.unwrap_or(false),
    ).await
}
