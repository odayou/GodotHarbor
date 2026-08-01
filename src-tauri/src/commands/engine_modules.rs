use tauri::{AppHandle, Emitter};
use crate::commands::utils::get_storage;
use crate::engine_modules::*;
use crate::models::Engine;

#[tauri::command]
pub fn get_engine_modules(app: AppHandle, engine_id: String) -> Result<EngineModulesInfo, String> {
    let storage = get_storage(&app);
    let engines: Vec<Engine> = storage.load_or_default("engines.json");

    let engine = engines.iter()
        .find(|e| e.engine_id == engine_id)
        .ok_or("未找到指定引擎".to_string())?;

    let modules = detect_installed_modules(engine);

    Ok(EngineModulesInfo {
        engine_id: engine.engine_id.clone(),
        engine_version: engine.version.clone(),
        modules,
        missing_for_project: Vec::new(),
    })
}

#[tauri::command]
pub fn get_all_engines_modules(app: AppHandle) -> Result<Vec<EngineModulesInfo>, String> {
    let storage = get_storage(&app);
    let engines: Vec<Engine> = storage.load_or_default("engines.json");

    Ok(detect_all_engines_modules(&engines))
}

#[tauri::command]
pub async fn check_project_missing_modules(app: AppHandle, project_id: String) -> Result<Vec<ModuleType>, String> {
    let app_clone = app.clone();
    tokio::task::spawn_blocking(move || {
        let storage = get_storage(&app_clone);
        let projects: Vec<crate::models::Project> = storage.load_or_default("projects.json");
        let project = projects.iter()
            .find(|p| p.project_id == project_id)
            .ok_or("未找到指定项目".to_string())?;

        let engines: Vec<Engine> = storage.load_or_default("engines.json");

        // Find the engine used by the project
        let engine = if let Some(ref engine_id) = project.last_used_engine_id {
            engines.iter().find(|e| e.engine_id == *engine_id)
        } else {
            None
        }.or_else(|| {
            // Try to find matching engine by version
            engines.iter().find(|e| {
                let ev: Vec<&str> = e.version.split('.').collect();
                let pv: Vec<&str> = project.godot_version.split('.').collect();
                ev.len() >= 2 && pv.len() >= 2 && ev[0] == pv[0] && ev[1] == pv[1]
            })
        });

        match engine {
            Some(engine) => Ok(check_missing_modules(engine, &project.path)),
            None => {
                // No matching engine, report all needed modules as missing
                let needed = get_modules_needed_by_project(&project.path);
                Ok(needed)
            }
        }
    })
    .await
    .map_err(|e| format!("任务执行失败: {}", e))?
}

/// 批量检查所有项目的缺失模块。一次 invoke 完成所有项目扫描，
/// 避免前端 N 次并发 invoke 占满 blocking pool。
#[tauri::command]
pub async fn batch_check_missing_modules(app: AppHandle) -> Result<Vec<(String, Vec<ModuleType>)>, String> {
    let app_clone = app.clone();
    tokio::task::spawn_blocking(move || {
        let storage = get_storage(&app_clone);
        let projects: Vec<crate::models::Project> = storage.load_or_default("projects.json");
        let engines: Vec<Engine> = storage.load_or_default("engines.json");

        let mut results = Vec::new();
        for project in &projects {
            let engine = if let Some(ref engine_id) = project.last_used_engine_id {
                engines.iter().find(|e| e.engine_id == *engine_id)
            } else {
                None
            }.or_else(|| {
                engines.iter().find(|e| {
                    let ev: Vec<&str> = e.version.split('.').collect();
                    let pv: Vec<&str> = project.godot_version.split('.').collect();
                    ev.len() >= 2 && pv.len() >= 2 && ev[0] == pv[0] && ev[1] == pv[1]
                })
            });

            let missing = match engine {
                Some(engine) => check_missing_modules(engine, &project.path),
                None => get_modules_needed_by_project(&project.path),
            };
            results.push((project.project_id.clone(), missing));
        }
        Ok(results)
    })
    .await
    .map_err(|e| format!("任务执行失败: {}", e))?
}

#[tauri::command]
pub async fn install_engine_module(app: AppHandle, engine_id: String, module_type: ModuleType) -> Result<(), String> {
    let storage = get_storage(&app);
    let engines: Vec<Engine> = storage.load_or_default("engines.json");

    let engine = engines.iter()
        .find(|e| e.engine_id == engine_id)
        .ok_or("未找到指定引擎".to_string())?;

    let version = engine.version.clone();
    let is_mono = engine.is_mono;

    match module_type {
        ModuleType::Editor => {
            return Err("编辑器模块无需单独安装".to_string());
        }
        ModuleType::DotNet => {
            // .NET support requires downloading the mono variant of the engine
            return Err(".NET 支持需要下载 .NET 版本的 Godot 引擎，请在引擎下载页面选择 .NET 变体".to_string());
        }
        _ => {
            // All other modules are export templates - use the existing download flow
            let _ = app.emit("module-install-progress", ModuleInstallProgress {
                module_type: module_type.clone(),
                version: version.clone(),
                stage: "downloading".to_string(),
                progress: 0.0,
                message: format!("正在下载 {} 导出模板...", module_type),
            });

            // Reuse the existing download_export_template logic
            crate::commands::build::download_export_template_inner(&app, version.clone(), is_mono).await?;

            let _ = app.emit("module-install-progress", ModuleInstallProgress {
                module_type: module_type.clone(),
                version: version.clone(),
                stage: "complete".to_string(),
                progress: 1.0,
                message: format!("{} 模块安装完成", module_type),
            });
        }
    }

    Ok(())
}

#[tauri::command]
pub fn get_module_download_info(module_type: ModuleType, version: String, is_mono: bool) -> Result<serde_json::Value, String> {
    let url = get_module_download_url(&module_type, &version, is_mono)?;

    Ok(serde_json::json!({
        "module_type": module_type.to_string(),
        "version": version,
        "is_mono": is_mono,
        "download_url": url,
        "description": match module_type {
            ModuleType::DotNet => ".NET/C# 支持，需要下载 .NET 版本的 Godot 引擎",
            ModuleType::Android => "Android 平台导出模板",
            ModuleType::IOS => "iOS 平台导出模板",
            ModuleType::Web => "Web/HTML5 平台导出模板",
            ModuleType::Linux => "Linux 平台导出模板",
            ModuleType::Windows => "Windows 平台导出模板",
            ModuleType::MacOS => "macOS 平台导出模板",
            ModuleType::Editor => "编辑器模块（已随引擎安装）",
        }
    }))
}
