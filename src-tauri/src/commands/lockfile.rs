use crate::lockfile::{self, HarborLock, LockDiff, LockVerifyResult, RestoreEnvResult};
use crate::commands::utils::{get_storage, log_operation, get_data_dir, get_plugin_manager, upsert_plugin, load_settings};
use crate::storage::Storage;
use crate::linker::Linker;
use tauri::AppHandle;

#[tauri::command]
pub fn generate_project_lock(app: AppHandle, project_id: String) -> Result<HarborLock, String> {
    let storage = get_storage(&app);
    let projects: Vec<crate::models::Project> = storage.load_or_default("projects.json");
    let project = projects.iter().find(|p| p.project_id == project_id)
        .ok_or("项目不存在".to_string())?;

    let plugins: Vec<crate::models::Plugin> = storage.load_or_default("plugins.json");
    let bindings: Vec<crate::models::ProjectBinding> = storage.load_or_default("bindings.json");
    let engines: Vec<crate::models::Engine> = storage.load_or_default("engines.json");

    let engine_bindings = if let Some(ref engine_id) = project.last_used_engine_id {
        vec![(project.project_id.clone(), engine_id.clone())]
    } else {
        vec![]
    };

    let lock = lockfile::generate_lock(project, &bindings, &plugins, &engines, &engine_bindings);
    Ok(lock)
}

#[tauri::command]
pub fn write_project_lock(app: AppHandle, project_id: String) -> Result<(), String> {
    let storage = get_storage(&app);
    let projects: Vec<crate::models::Project> = storage.load_or_default("projects.json");
    let project = projects.iter().find(|p| p.project_id == project_id)
        .ok_or("项目不存在".to_string())?;

    let plugins: Vec<crate::models::Plugin> = storage.load_or_default("plugins.json");
    let bindings: Vec<crate::models::ProjectBinding> = storage.load_or_default("bindings.json");
    let engines: Vec<crate::models::Engine> = storage.load_or_default("engines.json");

    let engine_bindings = if let Some(ref engine_id) = project.last_used_engine_id {
        vec![(project.project_id.clone(), engine_id.clone())]
    } else {
        vec![]
    };

    let lock = lockfile::generate_lock(project, &bindings, &plugins, &engines, &engine_bindings);
    lockfile::write_lock(&project.path, &lock)
        .map_err(|e| format!("写入 harbor.lock 失败: {}", e))?;

    log_operation(&app, "write_project_lock", &project_id,
        &format!("已生成 harbor.lock（{} 个插件）", lock.plugins.len()));

    Ok(())
}

/// 刷新指定项目的 harbor.lock：从 storage 读取最新数据并写入项目目录。
/// 供 commands 层在 apply_bindings 成功后调用，使 lockfile 与实际绑定保持同步。
pub fn refresh_project_lock(app: &AppHandle, project_id: &str) -> Result<(), String> {
    let storage = get_storage(app);
    let projects: Vec<crate::models::Project> = storage.load_or_default("projects.json");
    let project = projects.iter().find(|p| p.project_id == project_id)
        .ok_or("项目不存在".to_string())?;
    let plugins: Vec<crate::models::Plugin> = storage.load_or_default("plugins.json");
    let bindings: Vec<crate::models::ProjectBinding> = storage.load_or_default("bindings.json");
    let engines: Vec<crate::models::Engine> = storage.load_or_default("engines.json");
    lockfile::write_lock_for_project(project, &bindings, &plugins, &engines)
        .map_err(|e| format!("写入 harbor.lock 失败: {}", e))
}

#[tauri::command]
pub fn read_project_lock(app: AppHandle, project_id: String) -> Result<Option<HarborLock>, String> {
    let storage = get_storage(&app);
    let projects: Vec<crate::models::Project> = storage.load_or_default("projects.json");
    let project = projects.iter().find(|p| p.project_id == project_id)
        .ok_or("项目不存在".to_string())?;

    lockfile::read_lock(&project.path)
        .map_err(|e| format!("读取 harbor.lock 失败: {}", e))
}

#[tauri::command]
pub fn verify_project_lock(app: AppHandle, project_id: String) -> Result<LockVerifyResult, String> {
    let storage = get_storage(&app);
    let projects: Vec<crate::models::Project> = storage.load_or_default("projects.json");
    let project = projects.iter().find(|p| p.project_id == project_id)
        .ok_or("项目不存在".to_string())?;

    let lock = lockfile::read_lock(&project.path)
        .map_err(|e| format!("读取 harbor.lock 失败: {}", e))?
        .ok_or("项目未找到 harbor.lock 文件".to_string())?;

    let plugins: Vec<crate::models::Plugin> = storage.load_or_default("plugins.json");
    Ok(lockfile::verify_lock(&project.path, &lock, &plugins))
}

#[tauri::command]
pub fn diff_project_lock(app: AppHandle, project_id: String) -> Result<Option<LockDiff>, String> {
    let storage = get_storage(&app);
    let projects: Vec<crate::models::Project> = storage.load_or_default("projects.json");
    let project = projects.iter().find(|p| p.project_id == project_id)
        .ok_or("项目不存在".to_string())?;

    let existing_lock = lockfile::read_lock(&project.path)
        .map_err(|e| format!("读取 harbor.lock 失败: {}", e))?;

    let Some(existing_lock) = existing_lock else {
        return Ok(None);
    };

    let plugins: Vec<crate::models::Plugin> = storage.load_or_default("plugins.json");
    let bindings: Vec<crate::models::ProjectBinding> = storage.load_or_default("bindings.json");
    let engines: Vec<crate::models::Engine> = storage.load_or_default("engines.json");

    let engine_bindings = if let Some(ref engine_id) = project.last_used_engine_id {
        vec![(project.project_id.clone(), engine_id.clone())]
    } else {
        vec![]
    };

    let current_lock = lockfile::generate_lock(project, &bindings, &plugins, &engines, &engine_bindings);
    let diff = lockfile::diff_locks(&existing_lock, &current_lock);

    Ok(Some(diff))
}

#[tauri::command]
pub fn sync_from_lock(app: AppHandle, project_id: String, strict: Option<bool>) -> Result<Vec<String>, String> {
    let storage = get_storage(&app);
    let projects: Vec<crate::models::Project> = storage.load_or_default("projects.json");
    let project = projects.iter().find(|p| p.project_id == project_id)
        .ok_or("项目不存在".to_string())?;

    let lock = lockfile::read_lock(&project.path)
        .map_err(|e| format!("读取 harbor.lock 失败: {}", e))?
        .ok_or("项目未找到 harbor.lock 文件".to_string())?;

    let plugins: Vec<crate::models::Plugin> = storage.load_or_default("plugins.json");
    let mut bindings: Vec<crate::models::ProjectBinding> = storage.load_or_default("bindings.json");

    let strict_mode = strict.unwrap_or(false);
    let mut messages = lockfile::sync_from_lock(
        &project.path,
        &lock,
        &plugins,
        &mut bindings,
        &project_id,
        strict_mode,
    ).map_err(|e| format!("从锁文件同步失败: {}", e))?;

    storage.save("bindings.json", &bindings)
        .map_err(|e| format!("保存绑定列表失败: {}", e))?;

    // Apply changes to project directory via linker
    let desired_bindings: Vec<crate::models::ProjectBinding> = bindings.iter()
        .filter(|b| b.project_id == project_id)
        .cloned()
        .collect();

    if !desired_bindings.is_empty() {
        let linker = Linker::new();
        let data_dir = get_data_dir(&app);
        let plugin_base_path = data_dir.join("plugins");

        let applied_dir = data_dir.join("applied_bindings");
        let applied_file = applied_dir.join(format!("{}.json", project_id));
        let current_bindings: Vec<crate::models::ProjectBinding> = if applied_file.exists() {
            let applied_storage = Storage::new(applied_dir.clone());
            applied_storage.load_or_default::<Vec<crate::models::ProjectBinding>>(&format!("{}.json", project_id))
        } else {
            Vec::new()
        };

        match linker.apply_bindings(
            &project.path,
            &current_bindings,
            &desired_bindings,
            &plugin_base_path.to_string_lossy(),
            &data_dir.to_string_lossy()
        ) {
            Ok(result) => {
                if result.success {
                    if let Err(e) = std::fs::create_dir_all(&applied_dir) {
                        eprintln!("Failed to create applied_bindings dir: {}", e);
                    }
                    let applied_storage = Storage::new(applied_dir);
                    if let Err(e) = applied_storage.save(&format!("{}.json", project_id), &desired_bindings) {
                        eprintln!("Failed to save applied bindings: {}", e);
                    }
                    if let Err(e) = refresh_project_lock(&app, &project_id) {
                        eprintln!("Failed to write harbor.lock: {}", e);
                    }
                    messages.push(format!(
                        "已应用变更到项目目录: 创建 {} 项, 移除 {} 项",
                        result.created.len(), result.removed.len()
                    ));
                } else {
                    for err in &result.errors {
                        messages.push(format!("应用变更错误: {}", err));
                    }
                }
            }
            Err(e) => {
                messages.push(format!("应用变更到项目目录失败: {}", e));
            }
        }
    }

    log_operation(&app, "sync_from_lock", &project_id,
        &format!("从 harbor.lock 同步完成（{}）", if strict_mode { "严格模式" } else { "宽松模式" }));

    Ok(messages)
}

#[tauri::command]
pub fn batch_check_locks(app: AppHandle, project_ids: Vec<String>) -> Result<Vec<(String, Option<HarborLock>, LockVerifyResult)>, String> {
    let storage = get_storage(&app);
    let projects: Vec<crate::models::Project> = storage.load_or_default("projects.json");
    let plugins: Vec<crate::models::Plugin> = storage.load_or_default("plugins.json");

    let mut results = Vec::new();
    for pid in &project_ids {
        if let Some(project) = projects.iter().find(|p| &p.project_id == pid) {
            let lock = lockfile::read_lock(&project.path).ok().flatten();
            let verify_result = if let Some(ref lock) = lock {
                lockfile::verify_lock(&project.path, lock, &plugins)
            } else {
                LockVerifyResult {
                    is_valid: false,
                    mismatches: vec![],
                }
            };
            results.push((pid.clone(), lock, verify_result));
        }
    }

    Ok(results)
}

/// 「还原项目环境」：打开带 harbor.lock 的项目时一键还原。
/// 读 lockfile → 对每个 locked plugin 在本地 plugins.json 找匹配（按 source_url 优先）→
/// 未找到但 source_type 为 Git/Url 且 source_url 非空则自动导入 → AssetLibrary/Local 标记 missing →
/// 用本地实际 plugin_id/version_id/unit_id 重建绑定（mount_path/subdirectory 取自 lockfile）→
/// apply_bindings → 返回 {ready, imported, failed, missing}。
#[tauri::command]
pub async fn restore_project_environment(app: AppHandle, project_id: String) -> Result<RestoreEnvResult, String> {
    let app_clone = app.clone();
    tokio::task::spawn_blocking(move || -> Result<RestoreEnvResult, String> {
        let storage = get_storage(&app_clone);
        let projects: Vec<crate::models::Project> = storage.load_or_default("projects.json");
        let project = projects.iter().find(|p| p.project_id == project_id)
            .ok_or("项目不存在".to_string())?;

        let lock = lockfile::read_lock(&project.path)
            .map_err(|e| format!("读取 harbor.lock 失败: {}", e))?
            .ok_or("项目未找到 harbor.lock 文件".to_string())?;

        let mut plugins: Vec<crate::models::Plugin> = storage.load_or_default("plugins.json");
        let mut bindings: Vec<crate::models::ProjectBinding> = storage.load_or_default("bindings.json");
        let manager = get_plugin_manager(&app_clone);

        // P4-4: 加载 allowlist。空列表表示不限制（向后兼容）；非空时对 lockfile 带入的外部 URL 强制校验。
        let settings = load_settings(&app_clone);
        let allowlist = settings.plugin_source_allowlist.clone();

        let mut ready = Vec::new();
        let mut imported = Vec::new();
        let mut failed = Vec::new();
        let mut missing = Vec::new();
        let mut desired_new_bindings: Vec<crate::models::ProjectBinding> = Vec::new();

        for locked in &lock.plugins {
            // P4-4: allowlist 门控。对 lockfile 带入的外部 URL（Git/Url 源）强制校验 host。
            // AssetLibrary/Local 源无 URL，allowlist 不适用（这些源在本地已就绪）。
            let source_type = locked.source_type.as_str();
            let needs_url_check = (source_type == "Git" || source_type == "Url") && !locked.source_url.is_empty();
            if needs_url_check && !lockfile::is_url_allowed(&locked.source_url, &allowlist) {
                failed.push(format!(
                    "{}: 源 URL {} 不在 allowlist 内，已阻断（在设置中配置 plugin_source_allowlist）",
                    locked.plugin_name, locked.source_url
                ));
                continue;
            }

            // 优先按 source_url 匹配（跨机器可移植，lockfile 的 plugin_id/version_id 是本地 UUID 不可用）
            let matched_plugin = if !locked.source_url.is_empty() {
                plugins.iter().find(|p| p.source.url == locked.source_url).cloned()
            } else {
                None
            };

            let mut was_imported = false;
            let matched_plugin = match matched_plugin {
                Some(p) => Some(p),
                None => {
                    if (source_type == "Git" || source_type == "Url") && !locked.source_url.is_empty() {
                        let import_result = if source_type == "Git" {
                            // P4-4: 若 lockfile 声明了 commit_sha，使用 pinned 导入锁定供应链状态；
                            // 否则保持原行为（clone 默认分支），但提示用户重新生成 lockfile 以启用 pin。
                            if !locked.commit_sha.is_empty() {
                                manager.import_from_git_pinned(&locked.source_url, &locked.commit_sha, &app_clone)
                            } else {
                                manager.import_from_git(&locked.source_url, None, &app_clone)
                            }
                        } else {
                            manager.import_from_url(&locked.source_url, &app_clone)
                        };
                        match import_result {
                            Ok(new_plugin) => {
                                let upserted = upsert_plugin(
                                    &app_clone, &new_plugin,
                                    "restore_env_import", &locked.source_url,
                                )?;
                                // upsert 已写盘，重新加载以保证后续循环匹配到最新列表
                                plugins = storage.load_or_default("plugins.json");
                                was_imported = true;
                                Some(upserted)
                            }
                            Err(e) => {
                                failed.push(format!("{}: {}", locked.plugin_name, e));
                                None
                            }
                        }
                    } else {
                        // AssetLibrary / Local 无 URL，跨机器无法还原
                        missing.push(locked.plugin_name.clone());
                        None
                    }
                }
            };

            let Some(plugin) = matched_plugin else { continue; };

            // 选版本：优先 content_hash 相同，其次 version 字符串相同，最后取最新
            let version = plugin.versions.iter()
                .find(|v| {
                    let h = crate::models::compute_dir_hash(std::path::Path::new(&v.path)).unwrap_or_default();
                    !h.is_empty() && h == locked.content_hash
                })
                .or_else(|| plugin.versions.iter().find(|v| v.version == locked.version))
                .or_else(|| plugin.versions.last())
                .cloned();

            let Some(version) = version else {
                failed.push(format!("{}: 无可用版本", locked.plugin_name));
                continue;
            };

            // 选 unit：优先 unit_name 相同，否则取第一个（lockfile 的 unit_id 是本地 UUID 不可用）
            let unit_id = version.units.iter()
                .find(|u| u.name == locked.unit_name)
                .or_else(|| version.units.first())
                .map(|u| u.unit_id.clone())
                .unwrap_or_default();

            desired_new_bindings.push(crate::models::ProjectBinding::new(
                project_id.clone(),
                plugin.plugin_id.clone(),
                version.version_id.clone(),
                unit_id,
                locked.mount_path.clone(),
                locked.subdirectory.clone(),
            ));

            if was_imported {
                imported.push(locked.plugin_name.clone());
            } else {
                ready.push(locked.plugin_name.clone());
            }
        }

        // 用 lockfile 中的绑定覆盖项目下同 plugin_id 的旧绑定（不动用户其他绑定）
        let plugin_ids_in_lock: std::collections::HashSet<String> = desired_new_bindings.iter()
            .map(|b| b.plugin_id.clone()).collect();
        bindings.retain(|b| !(b.project_id == project_id && plugin_ids_in_lock.contains(&b.plugin_id)));
        bindings.extend(desired_new_bindings);
        storage.save("bindings.json", &bindings)
            .map_err(|e| format!("保存绑定列表失败: {}", e))?;

        // apply 到项目目录
        let desired_bindings: Vec<crate::models::ProjectBinding> = bindings.iter()
            .filter(|b| b.project_id == project_id)
            .cloned()
            .collect();

        if !desired_bindings.is_empty() {
            let linker = Linker::new();
            let data_dir = get_data_dir(&app_clone);
            let plugin_base_path = data_dir.join("plugins");
            let applied_dir = data_dir.join("applied_bindings");
            let applied_file = applied_dir.join(format!("{}.json", project_id));
            let current_bindings: Vec<crate::models::ProjectBinding> = if applied_file.exists() {
                let applied_storage = Storage::new(applied_dir.clone());
                applied_storage.load_or_default::<Vec<crate::models::ProjectBinding>>(&format!("{}.json", project_id))
            } else {
                Vec::new()
            };

            match linker.apply_bindings(
                &project.path,
                &current_bindings,
                &desired_bindings,
                &plugin_base_path.to_string_lossy(),
                &data_dir.to_string_lossy()
            ) {
                Ok(result) => {
                    if result.success {
                        if let Err(e) = std::fs::create_dir_all(&applied_dir) {
                            eprintln!("Failed to create applied_bindings dir: {}", e);
                        }
                        let applied_storage = Storage::new(applied_dir);
                        if let Err(e) = applied_storage.save(&format!("{}.json", project_id), &desired_bindings) {
                            eprintln!("Failed to save applied bindings: {}", e);
                        }
                        match refresh_project_lock(&app_clone, &project_id) {
                            Ok(()) => {
                                // P4-1: restore 后强制校验，检测 apply 结果与锁文件声明的一致性
                                // P4-4: 同时校验 commit_sha pin 是否与本地 HEAD 一致
                                if let Ok(Some(refreshed_lock)) = lockfile::read_lock(&project.path) {
                                    let verify_result = lockfile::verify_lock(&project.path, &refreshed_lock, &plugins);
                                    for m in &verify_result.mismatches {
                                        if let Some(issue) = &m.mount_path_issue {
                                            // 旧版哈希格式提示是信息性的，不计入失败
                                            if issue.contains("旧版哈希格式") {
                                                continue;
                                            }
                                            failed.push(format!("{}: 还原后校验失败 - {}", m.plugin_name, issue));
                                        }
                                        if let Some(issue) = &m.commit_sha_issue {
                                            failed.push(format!("{}: 供应链校验失败 - {}", m.plugin_name, issue));
                                        }
                                    }
                                }
                            }
                            Err(e) => eprintln!("Failed to write harbor.lock: {}", e),
                        }
                    } else {
                        for err in &result.errors {
                            failed.push(format!("应用错误: {}", err));
                        }
                    }
                }
                Err(e) => {
                    return Err(format!("应用绑定失败: {}", e));
                }
            }
        }

        log_operation(&app_clone, "restore_project_environment", &project_id,
            &format!("还原环境: 就绪 {}, 导入 {}, 失败 {}, 缺失 {}",
                ready.len(), imported.len(), failed.len(), missing.len()));

        Ok(RestoreEnvResult { ready, imported, failed, missing })
    }).await.map_err(|e| format!("任务执行失败: {}", e))?
}
