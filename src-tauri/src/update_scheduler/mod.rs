use crate::models::Settings;
use crate::storage::Storage;
use tauri::{AppHandle, Emitter, Manager};
use std::sync::atomic::{AtomicBool, Ordering};

static LAST_CHECK: AtomicBool = AtomicBool::new(false);

pub fn start_update_scheduler(app: AppHandle) {
    let scheduler_app = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(30)).await;

        check_and_notify(&scheduler_app).await;

        let mut interval_hours = 4u64;
        let data_dir = scheduler_app.path().app_data_dir()
            .expect("Failed to get app data directory");
        let storage = Storage::new(data_dir);
        let settings: Settings = storage.load_or_default("settings.json");
        if settings.update_check_interval_hours > 0 {
            interval_hours = settings.update_check_interval_hours as u64;
        }

        loop {
            tokio::time::sleep(std::time::Duration::from_secs(interval_hours * 3600)).await;

            let data_dir = scheduler_app.path().app_data_dir()
                .expect("Failed to get app data directory");
            let storage = Storage::new(data_dir);
            let settings: Settings = storage.load_or_default("settings.json");

            if !settings.auto_check_app_updates && !settings.auto_check_plugin_updates && !settings.auto_check_engine_updates {
                continue;
            }

            if settings.update_check_interval_hours > 0 {
                interval_hours = settings.update_check_interval_hours as u64;
            }

            check_and_notify(&scheduler_app).await;
        }
    });
}

async fn check_and_notify(app: &AppHandle) {
    let data_dir = app.path().app_data_dir()
        .expect("Failed to get app data directory");
    let storage = Storage::new(data_dir);
    let settings: Settings = storage.load_or_default("settings.json");

    let mut has_updates = false;

    if settings.auto_check_plugin_updates {
        if let Ok(result) = crate::commands::check_all_updates(app.clone()).await {
            if !result.plugin_updates.is_empty() {
                has_updates = true;
                let _ = app.emit("plugin-updates-available", &result.plugin_updates);
            }
            if !result.engine_updates.is_empty() {
                has_updates = true;
                let _ = app.emit("engine-updates-available", &result.engine_updates);
            }
        }
    }

    // if settings.auto_check_app_updates {
    //     if let Ok(Some(update)) = crate::commands::check_app_update(app.clone()).await {
    //         if update.latest_version != settings.skipped_app_version {
    //             has_updates = true;
    //             let _ = app.emit("app-update-available", &update);
    //         }
    //     }
    // }

    if has_updates {
        let _ = app.emit("updates-available", ());
    }

    LAST_CHECK.store(true, Ordering::SeqCst);
}
