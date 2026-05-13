use serde::{Serialize, Deserialize};
use std::fs;
use crate::storage::Storage;
use crate::models::Settings;
use tauri::Manager;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeaturedPlugin {
    pub name: String,
    pub description: String,
    pub author: String,
    pub source_url: String,
    pub compatibility: String,
    pub tags: Vec<String>,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeaturedPluginsList {
    pub version: u32,
    pub updated_at: String,
    pub plugins: Vec<FeaturedPlugin>,
}

impl FeaturedPluginsList {
    pub fn builtin() -> Self {
        Self {
            version: 1,
            updated_at: "2025-05-14".to_string(),
            plugins: vec![
                FeaturedPlugin {
                    name: "GodotSteam".to_string(),
                    description: "Steam API integration for Godot".to_string(),
                    author: "Gramps".to_string(),
                    source_url: "https://github.com/GodotSteam/GodotSteam".to_string(),
                    compatibility: "Godot4".to_string(),
                    tags: vec!["steam".into(), "multiplayer".into(), "sdk".into()],
                    category: "integration".to_string(),
                },
                FeaturedPlugin {
                    name: "Dialogic".to_string(),
                    description: "Create dialog systems and timelines".to_string(),
                    author: "Jowan".to_string(),
                    source_url: "https://github.com/dialogic-godot/dialogic".to_string(),
                    compatibility: "Godot4".to_string(),
                    tags: vec!["dialog".into(), "narrative".into(), "visual-novel".into()],
                    category: "gameplay".to_string(),
                },
                FeaturedPlugin {
                    name: "PhantomCamera".to_string(),
                    description: "Powerful camera system for Godot".to_string(),
                    author: "Marcel".to_string(),
                    source_url: "https://github.com/ramokz/phantom-camera".to_string(),
                    compatibility: "Godot4".to_string(),
                    tags: vec!["camera".into(), "cinematic".into()],
                    category: "gameplay".to_string(),
                },
                FeaturedPlugin {
                    name: "GUT".to_string(),
                    description: "Godot Unit Testing framework".to_string(),
                    author: "Butch".to_string(),
                    source_url: "https://github.com/bitwes/Gut".to_string(),
                    compatibility: "Both".to_string(),
                    tags: vec!["testing".into(), "unit-test".into()],
                    category: "development".to_string(),
                },
                FeaturedPlugin {
                    name: "LimboAI".to_string(),
                    description: "Behavior trees and state machines for Godot".to_string(),
                    author: "LimboAI".to_string(),
                    source_url: "https://github.com/limbonaut/limboai".to_string(),
                    compatibility: "Godot4".to_string(),
                    tags: vec!["ai".into(), "behavior-tree".into(), "state-machine".into()],
                    category: "gameplay".to_string(),
                },
                FeaturedPlugin {
                    name: "Aseprite Wizard".to_string(),
                    description: "Import Aseprite animations into Godot".to_string(),
                    author: "Vinicius".to_string(),
                    source_url: "https://github.com/viniciusgerevini/godot-aseprite-wizard".to_string(),
                    compatibility: "Godot4".to_string(),
                    tags: vec!["animation".into(), "aseprite".into(), "sprite".into()],
                    category: "art".to_string(),
                },
                FeaturedPlugin {
                    name: "Terrain3D".to_string(),
                    description: "High performance terrain system for Godot".to_string(),
                    author: "Cory".to_string(),
                    source_url: "https://github.com/TokisanGames/Terrain3D".to_string(),
                    compatibility: "Godot4".to_string(),
                    tags: vec!["terrain".into(), "3d".into(), "landscape".into()],
                    category: "3d".to_string(),
                },
                FeaturedPlugin {
                    name: "Godot SQLite".to_string(),
                    description: "SQLite database integration for Godot".to_string(),
                    author: "Khairul".to_string(),
                    source_url: "https://github.com/2shady4u/godot-sqlite".to_string(),
                    compatibility: "Both".to_string(),
                    tags: vec!["database".into(), "sqlite".into(), "storage".into()],
                    category: "data".to_string(),
                },
                FeaturedPlugin {
                    name: "Kuhltimate Pixelizer".to_string(),
                    description: "Real-time pixel art post-processing".to_string(),
                    author: "Kuhltimate".to_string(),
                    source_url: "https://github.com/Kuhltimate/Pixelizer".to_string(),
                    compatibility: "Godot4".to_string(),
                    tags: vec!["pixel-art".into(), "shader".into(), "post-processing".into()],
                    category: "art".to_string(),
                },
                FeaturedPlugin {
                    name: "GodotTIE".to_string(),
                    description: "Text Interface Engine for dialog and text effects".to_string(),
                    author: "Jan".to_string(),
                    source_url: "https://github.com/fenix-hub/godot-engine-tei".to_string(),
                    compatibility: "Godot4".to_string(),
                    tags: vec!["text".into(), "dialog".into(), "typewriter".into()],
                    category: "ui".to_string(),
                },
            ],
        }
    }
}

#[tauri::command]
pub fn get_featured_plugins(app: tauri::AppHandle) -> Result<FeaturedPluginsList, String> {
    let data_dir = crate::commands::get_data_dir(&app);
    let cache_path = data_dir.join("cache").join("featured_plugins.json");

    if cache_path.exists() {
        if let Ok(content) = fs::read_to_string(&cache_path) {
            if let Ok(list) = serde_json::from_str::<FeaturedPluginsList>(&content) {
                return Ok(list);
            }
        }
    }

    Ok(FeaturedPluginsList::builtin())
}

#[tauri::command]
pub async fn report_usage_ping(app: tauri::AppHandle) -> Result<(), String> {
    let config_dir = app.path().app_data_dir()
        .map_err(|e| format!("获取配置目录失败: {}", e))?;
    let config_storage = Storage::new(config_dir);
    let mut settings: Settings = config_storage.load_or_default("settings.json");

    if !settings.enable_anonymous_usage_stats {
        return Ok(());
    }

    if settings.anonymous_user_id.is_empty() {
        settings.anonymous_user_id = Uuid::new_v4().to_string();
        config_storage.save("settings.json", &settings)
            .map_err(|e| format!("保存匿名ID失败: {}", e))?;
    }

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let data_dir = crate::commands::get_data_dir(&app);
    let last_ping_file = data_dir.join(".last_usage_ping");
    if last_ping_file.exists() {
        if let Ok(last_ping) = std::fs::read_to_string(&last_ping_file) {
            if last_ping.trim() == today {
                return Ok(());
            }
        }
    }

    let version = crate::commands::get_app_version(app.clone()).unwrap_or_default();
    let os = std::env::consts::OS.to_string();
    let timestamp = chrono::Utc::now().to_rfc3339();

    let uuid = settings.anonymous_user_id.clone();
    let ping_url = format!(
        "https://ping.godot-harbor.com/v1?u={}&v={}&o={}&t={}",
        urlencoding::encode(&uuid),
        urlencoding::encode(&version),
        urlencoding::encode(&os),
        urlencoding::encode(&timestamp)
    );

    let client = crate::utils::create_http_client(Some(std::time::Duration::from_secs(3)))?;

    let _ = client.get(&ping_url)
        .header("User-Agent", "GodotHarbor")
        .timeout(std::time::Duration::from_secs(3))
        .send()
        .await;

    let _ = std::fs::write(&last_ping_file, &today);

    Ok(())
}

#[tauri::command]
pub fn record_plugin_install(app: tauri::AppHandle, plugin_id: String) -> Result<(), String> {
    let storage = crate::commands::get_storage(&app);
    let mut plugins: Vec<crate::models::Plugin> = storage.load_or_default("plugins.json");

    if let Some(plugin) = plugins.iter_mut().find(|p| p.plugin_id == plugin_id) {
        plugin.install_count += 1;
        storage.save("plugins.json", &plugins)
            .map_err(|e| format!("保存插件统计失败: {}", e))?;
    }

    Ok(())
}
