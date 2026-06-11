use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use crate::asset_store::AssetStoreClient;
use crate::commands::utils::get_storage;
use crate::models::{Plugin, SourceType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorePlugin {
    pub asset_id: i64,
    pub name: String,
    pub author: String,
    pub description: String,
    pub category: String,
    pub godot_version: String,
    pub support_level: String,
    pub download_count: u64,
    pub rating: f64,
    pub rating_count: u32,
    pub icon_url: String,
    pub preview_images: Vec<String>,
    pub source_url: String,
    pub tags: Vec<String>,
    pub is_installed: bool,
    pub installed_version: Option<String>,
    pub compatible: bool,
    pub last_updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreSearchResult {
    pub plugins: Vec<StorePlugin>,
    pub total: u32,
    pub page: u32,
    pub page_size: u32,
    pub has_more: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreRecommendation {
    pub plugin: StorePlugin,
    pub reason: String,
    pub relevance_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreCategory {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OneClickInstallResult {
    pub success: bool,
    pub plugin_id: Option<String>,
    pub binding_created: bool,
    pub changes_applied: bool,
    pub errors: Vec<String>,
}

fn get_installed_asset_ids(app: &AppHandle) -> std::collections::HashSet<String> {
    let storage = get_storage(app);
    let plugins: Vec<Plugin> = storage.load_or_default("plugins.json");
    let mut ids = std::collections::HashSet::new();
    for p in &plugins {
        if p.source.source_type == SourceType::AssetLibrary {
            if let Some(id) = p.source.url.strip_prefix("asset-library://") {
                ids.insert(id.to_string());
            }
        }
    }
    ids
}

fn get_installed_version_for_asset(app: &AppHandle, asset_id: &str) -> Option<String> {
    let storage = get_storage(app);
    let plugins: Vec<Plugin> = storage.load_or_default("plugins.json");
    for p in &plugins {
        if p.source.source_type == SourceType::AssetLibrary {
            if let Some(id) = p.source.url.strip_prefix("asset-library://") {
                if id == asset_id {
                    return p.versions.first().map(|v| v.version.clone());
                }
            }
        }
    }
    None
}

fn check_godot_compatibility(godot_version: &str, plugin_godot_version: &str) -> bool {
    if godot_version.is_empty() || plugin_godot_version.is_empty() || plugin_godot_version == "any" {
        return true;
    }
    let plugin_lower = plugin_godot_version.to_lowercase();
    let is_godot4 = godot_version.starts_with('4') || godot_version.starts_with("4.");
    let is_godot3 = godot_version.starts_with('3') || godot_version.starts_with("3.");

    if plugin_lower.contains("4.0") || plugin_lower.contains("4.1") || plugin_lower.contains("4.2") || plugin_lower.contains("4.3") || plugin_lower.contains("4.4") || plugin_lower.contains("4.x") {
        is_godot4
    } else if plugin_lower.contains("3.0") || plugin_lower.contains("3.1") || plugin_lower.contains("3.2") || plugin_lower.contains("3.3") || plugin_lower.contains("3.4") || plugin_lower.contains("3.5") || plugin_lower.contains("3.x") {
        is_godot3
    } else {
        true
    }
}

fn json_value_to_store_plugins(
    value: &serde_json::Value,
    app: &AppHandle,
    godot_version_filter: Option<&str>,
) -> Result<Vec<StorePlugin>, String> {
    let installed_ids = get_installed_asset_ids(app);
    let mut result = Vec::new();

    let assets = value.get("results")
        .or_else(|| value.get("result"))
        .and_then(|v| v.as_array());

    let assets = match assets {
        Some(a) => a,
        None => {
            if let Some(arr) = value.as_array() {
                arr
            } else {
                return Ok(result);
            }
        }
    };

    for asset in assets {
        let asset_id_str = asset.get("asset_id")
            .and_then(|v| v.as_str())
            .unwrap_or("0");
        let asset_id: i64 = asset_id_str.parse().unwrap_or(0);

        let name = asset.get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let author = asset.get("author")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let description = asset.get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let category = asset.get("category")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let godot_version = asset.get("godot_version")
            .or_else(|| asset.get("godot_version_min"))
            .and_then(|v| v.as_str())
            .unwrap_or("any")
            .to_string();

        let support_level = asset.get("support_level")
            .and_then(|v| v.as_str())
            .unwrap_or("community")
            .to_string();

        let download_count = asset.get("download_count")
            .or_else(|| asset.get("downloads"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let rating = asset.get("rating")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let rating_count = asset.get("rating_count")
            .or_else(|| asset.get("review_count"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        let icon_url = asset.get("icon_url")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let preview_images = asset.get("preview_images")
            .or_else(|| asset.get("previews"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter().filter_map(|p| {
                    if let Some(s) = p.as_str() {
                        Some(s.to_string())
                    } else {
                        p.get("link").and_then(|l| l.as_str()).map(|s| s.to_string())
                    }
                }).collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let source_url = asset.get("source_url")
            .or_else(|| asset.get("browse_url"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let tags = asset.get("tags")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter().filter_map(|t| t.as_str().map(|s| s.to_string())).collect()
            })
            .unwrap_or_default();

        let last_updated = asset.get("modify_date")
            .or_else(|| asset.get("updated_at"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let is_installed = installed_ids.contains(asset_id_str);
        let installed_version = if is_installed {
            get_installed_version_for_asset(app, asset_id_str)
        } else {
            None
        };

        let compatible = match godot_version_filter {
            Some(gv) => check_godot_compatibility(gv, &godot_version),
            None => true,
        };

        result.push(StorePlugin {
            asset_id,
            name,
            author,
            description,
            category,
            godot_version,
            support_level,
            download_count,
            rating,
            rating_count,
            icon_url,
            preview_images,
            source_url,
            tags,
            is_installed,
            installed_version,
            compatible,
            last_updated,
        });
    }

    Ok(result)
}

pub async fn search_store(
    app: &AppHandle,
    query: &str,
    category: Option<&str>,
    godot_version: Option<&str>,
    sort_by: Option<&str>,
    page: Option<u32>,
    page_size: Option<u32>,
) -> Result<StoreSearchResult, String> {
    let client = AssetStoreClient::new(app)?;
    let p = page.unwrap_or(1);
    let ps = page_size.unwrap_or(20);
    let sort = sort_by.unwrap_or("updated");

    let raw = client.search(query, p, ps, sort, category, godot_version, None).await?;

    let total = raw.get("total")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;

    let pages = raw.get("pages")
        .and_then(|v| v.as_u64())
        .unwrap_or(1) as u32;

    let godot_version_filter = godot_version.and_then(|gv| {
        if gv.is_empty() || gv == "any" { None } else { Some(gv.to_string()) }
    });

    let project_gv = if godot_version_filter.is_none() {
        let storage = get_storage(app);
        let projects: Vec<crate::models::Project> = storage.load_or_default("projects.json");
        projects.first().map(|p| p.godot_version.clone()).filter(|gv| !gv.is_empty())
    } else {
        godot_version_filter
    };

    let plugins = json_value_to_store_plugins(&raw, app, project_gv.as_deref())?;

    Ok(StoreSearchResult {
        plugins,
        total,
        page: p,
        page_size: ps,
        has_more: p < pages,
    })
}

pub async fn get_store_recommendations(
    app: &AppHandle,
    project_id: Option<&str>,
) -> Result<Vec<StoreRecommendation>, String> {
    let client = AssetStoreClient::new(app)?;
    let mut recommendations = Vec::new();

    let project_godot_version: Option<String> = if let Some(pid) = project_id {
        let storage = get_storage(app);
        let projects: Vec<crate::models::Project> = storage.load_or_default("projects.json");
        projects.iter().find(|p| p.project_id == pid).map(|p| p.godot_version.clone())
    } else {
        None
    };

    let gv_filter = project_godot_version.as_deref().and_then(|gv| {
        if gv.is_empty() { None } else { Some(gv.to_string()) }
    });

    // Featured / popular
    let featured_raw = client.search("", 1, 10, "rating", None, gv_filter.as_deref(), None).await?;
    let featured_plugins = json_value_to_store_plugins(&featured_raw, app, gv_filter.as_deref())?;

    for (i, plugin) in featured_plugins.into_iter().take(5).enumerate() {
        let reason = if plugin.support_level == "official" {
            "官方推荐插件".to_string()
        } else if plugin.rating >= 4.5 {
            "高评分热门插件".to_string()
        } else {
            "热门插件".to_string()
        };
        recommendations.push(StoreRecommendation {
            plugin,
            reason,
            relevance_score: 1.0 - (i as f64 * 0.1),
        });
    }

    // Recently updated
    let recent_raw = client.search("", 1, 10, "updated", None, gv_filter.as_deref(), None).await?;
    let recent_plugins = json_value_to_store_plugins(&recent_raw, app, gv_filter.as_deref())?;

    let existing_ids: std::collections::HashSet<i64> = recommendations.iter().map(|r| r.plugin.asset_id).collect();
    for (i, plugin) in recent_plugins.into_iter().take(5).enumerate() {
        if existing_ids.contains(&plugin.asset_id) {
            continue;
        }
        recommendations.push(StoreRecommendation {
            plugin,
            reason: "最近更新".to_string(),
            relevance_score: 0.8 - (i as f64 * 0.05),
        });
    }

    // If project_id provided, also get plugins in the same category as installed ones
    if let Some(pid) = project_id {
        let storage = get_storage(app);
        let bindings: Vec<crate::models::ProjectBinding> = storage.load_or_default("bindings.json");
        let bound_plugin_ids: Vec<String> = bindings.iter()
            .filter(|b| b.project_id == pid)
            .map(|b| b.plugin_id.clone())
            .collect();

        if !bound_plugin_ids.is_empty() {
            let plugins: Vec<Plugin> = storage.load_or_default("plugins.json");
            let bound_tags: Vec<String> = plugins.iter()
                .filter(|p| bound_plugin_ids.contains(&p.plugin_id))
                .flat_map(|p| p.tags.clone())
                .take(3)
                .collect();

            if !bound_tags.is_empty() {
                let tag_query = bound_tags.first().unwrap().clone();
                let related_raw = client.search(&tag_query, 1, 5, "rating", None, gv_filter.as_deref(), None).await?;
                let related_plugins = json_value_to_store_plugins(&related_raw, app, gv_filter.as_deref())?;
                let existing_ids2: std::collections::HashSet<i64> = recommendations.iter().map(|r| r.plugin.asset_id).collect();

                for (i, plugin) in related_plugins.into_iter().take(3).enumerate() {
                    if existing_ids2.contains(&plugin.asset_id) || plugin.is_installed {
                        continue;
                    }
                    recommendations.push(StoreRecommendation {
                        plugin,
                        reason: format!("与已安装插件相关 ({})", tag_query),
                        relevance_score: 0.7 - (i as f64 * 0.05),
                    });
                }
            }
        }
    }

    recommendations.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap_or(std::cmp::Ordering::Equal));
    recommendations.truncate(10);

    Ok(recommendations)
}

pub async fn get_store_categories_with_counts(app: &AppHandle) -> Result<Vec<StoreCategory>, String> {
    let client = AssetStoreClient::new(app)?;
    let raw = client.get_categories().await?;

    let mut categories = Vec::new();

    if let Some(arr) = raw.as_array() {
        for cat in arr {
            let id = cat.get("id")
                .or_else(|| cat.get("slug"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let name = cat.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or(&id)
                .to_string();

            let icon = cat.get("icon")
                .and_then(|v| v.as_str())
                .unwrap_or("📦")
                .to_string();

            let count = cat.get("count")
                .or_else(|| cat.get("asset_count"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32;

            categories.push(StoreCategory { id, name, icon, count });
        }
    } else if let Some(obj) = raw.as_object() {
        for (key, value) in obj {
            let name = value.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or(key)
                .to_string();

            let icon = value.get("icon")
                .and_then(|v| v.as_str())
                .unwrap_or("📦")
                .to_string();

            let count = value.get("count")
                .or_else(|| value.get("asset_count"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32;

            categories.push(StoreCategory {
                id: key.clone(),
                name,
                icon,
                count,
            });
        }
    }

    if categories.is_empty() {
        categories = vec![
            StoreCategory { id: "2d".into(), name: "2D".into(), icon: "🎮".into(), count: 0 },
            StoreCategory { id: "3d".into(), name: "3D".into(), icon: "🧊".into(), count: 0 },
            StoreCategory { id: "ai".into(), name: "AI".into(), icon: "🤖".into(), count: 0 },
            StoreCategory { id: "animation".into(), name: "动画".into(), icon: "🎬".into(), count: 0 },
            StoreCategory { id: "audio".into(), name: "音频".into(), icon: "🔊".into(), count: 0 },
            StoreCategory { id: "gameplay".into(), name: "游戏玩法".into(), icon: "🎯".into(), count: 0 },
            StoreCategory { id: "integration".into(), name: "集成".into(), icon: "🔗".into(), count: 0 },
            StoreCategory { id: "ui".into(), name: "UI".into(), icon: "🖥️".into(), count: 0 },
            StoreCategory { id: "development".into(), name: "开发工具".into(), icon: "🛠️".into(), count: 0 },
            StoreCategory { id: "shaders".into(), name: "着色器".into(), icon: "✨".into(), count: 0 },
        ];
    }

    Ok(categories)
}

pub async fn one_click_install(
    app: &AppHandle,
    asset_id: i64,
    project_id: &str,
    auto_apply: bool,
) -> Result<OneClickInstallResult, String> {
    let mut errors = Vec::new();
    let mut plugin_id: Option<String> = None;
    let mut binding_created = false;
    let mut changes_applied = false;

    // Step 1: Import from asset library (download + import to vault)
    let import_result = crate::commands::import_from_asset_library_with_progress(
        app.clone(),
        asset_id.to_string(),
    ).await;

    match import_result {
        Ok(plugin) => {
            plugin_id = Some(plugin.plugin_id.clone());

            // Record install
            let _ = crate::featured::record_plugin_install(app.clone(), plugin.plugin_id.clone());

            // Step 2: Create binding if project_id provided
            if !project_id.is_empty() {
                let version = plugin.versions.first();
                let unit = version.and_then(|v| v.units.first());

                if let (Some(ver), Some(u)) = (version, unit) {
                    let mount_path = if u.subdirectory.is_empty() {
                        format!("addons/{}", u.dir_name)
                    } else {
                        format!("addons/{}/{}", u.dir_name, u.subdirectory)
                    };

                    let bind_result = crate::commands::bind_plugin(
                        app.clone(),
                        project_id.to_string(),
                        plugin.plugin_id.clone(),
                        ver.version_id.clone(),
                        u.unit_id.clone(),
                        mount_path,
                        u.subdirectory.clone(),
                    );

                    match bind_result {
                        Ok(_) => {
                            binding_created = true;

                            // Step 3: Apply changes if auto_apply
                            if auto_apply {
                                let apply_result = crate::commands::apply_changes(
                                    app.clone(),
                                    project_id.to_string(),
                                ).await;

                                match apply_result {
                                    Ok(result) => {
                                        changes_applied = result.success;
                                        if !result.errors.is_empty() {
                                            errors.extend(result.errors);
                                        }
                                    }
                                    Err(e) => {
                                        errors.push(format!("应用变更失败: {}", e));
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            errors.push(format!("绑定项目失败: {}", e));
                        }
                    }
                } else {
                    errors.push("插件没有可用的版本或单元".to_string());
                }
            }
        }
        Err(e) => {
            errors.push(format!("导入插件失败: {}", e));
        }
    }

    let success = plugin_id.is_some() && (project_id.is_empty() || binding_created);

    Ok(OneClickInstallResult {
        success,
        plugin_id,
        binding_created,
        changes_applied,
        errors,
    })
}
