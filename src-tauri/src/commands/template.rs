use std::fs;
use std::path::Path;
use tauri::{AppHandle, Emitter};
use crate::models::*;
use crate::utils::create_http_client;
use crate::linker::Linker;
use super::utils::*;

fn get_templates_dir(app: &AppHandle) -> std::path::PathBuf {
    get_data_dir(app).join("templates")
}

#[tauri::command]
pub fn list_hub_templates(app: AppHandle) -> Result<Vec<Template>, String> {
    let templates_dir = get_templates_dir(&app);
    if !templates_dir.exists() {
        return Ok(Vec::new());
    }

    let mut templates = Vec::new();
    let entries = fs::read_dir(&templates_dir)
        .map_err(|e| format!("读取模板目录失败: {}", e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let template_file = path.join("template.yml");
            if template_file.exists() {
                let content = fs::read_to_string(&template_file)
                    .map_err(|e| format!("读取模板文件失败: {}", e))?;
                match Template::from_yaml(&content) {
                    Ok(t) => templates.push(t),
                    Err(e) => {
                        eprintln!("跳过无效模板 {:?}: {}", path, e);
                    }
                }
            }
        }
    }

    templates.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(templates)
}

#[tauri::command]
pub fn get_hub_template(app: AppHandle, template_id: String) -> Result<Template, String> {
    let templates_dir = get_templates_dir(&app);
    let template_dir = templates_dir.join(&template_id);
    let template_file = template_dir.join("template.yml");

    if !template_file.exists() {
        return Err(format!("模板不存在: {}", template_id));
    }

    let content = fs::read_to_string(&template_file)
        .map_err(|e| format!("读取模板文件失败: {}", e))?;
    Template::from_yaml(&content)
}

#[tauri::command]
pub fn save_hub_template(app: AppHandle, template: Template) -> Result<Template, String> {
    let templates_dir = get_templates_dir(&app);
    let template_dir = templates_dir.join(&template.template_id);
    fs::create_dir_all(&template_dir)
        .map_err(|e| format!("创建模板目录失败: {}", e))?;

    let template_file = template_dir.join("template.yml");
    let yaml = template.to_yaml()?;
    fs::write(&template_file, yaml)
        .map_err(|e| format!("写入模板文件失败: {}", e))?;

    log_operation(&app, "save_hub_template", &template.template_id, &format!("保存模板: {}", template.name));
    Ok(template)
}

#[tauri::command]
pub fn delete_hub_template(app: AppHandle, template_id: String) -> Result<(), String> {
    let templates_dir = get_templates_dir(&app);
    let template_dir = templates_dir.join(&template_id);

    if !template_dir.exists() {
        return Err(format!("模板不存在: {}", template_id));
    }

    fs::remove_dir_all(&template_dir)
        .map_err(|e| format!("删除模板失败: {}", e))?;

    log_operation(&app, "delete_hub_template", &template_id, &format!("删除模板: {}", template_id));
    Ok(())
}

#[tauri::command]
pub async fn import_template_from_url(app: AppHandle, url: String) -> Result<Template, String> {
    let client = create_http_client(Some(std::time::Duration::from_secs(30)))?;
    let resp = client.get(&url).send().await
        .map_err(|e| format!("下载模板失败: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("下载模板返回错误: {}", resp.status()));
    }

    let content = resp.text().await
        .map_err(|e| format!("读取模板内容失败: {}", e))?;

    let mut template = Template::from_yaml(&content)?;
    template.template_id = uuid::Uuid::new_v4().to_string();
    template.source_url = url;
    template.is_builtin = false;

    save_hub_template(app, template)
}

fn generate_project_godot(template: &Template) -> String {
    let mut content = String::new();
    content.push_str("; Engine configuration file.\n; It's best edited using the editor UI and not directly,\n; since the parameters that go here are not all obvious.\n;\n; Format:\n;   [section] ; section goes between []\n;   param=value ; assign values to parameters\n\n");

    content.push_str("[application]\n");
    content.push_str(&format!("config/name=\"{}\"\n", template.name));

    if !template.godot.mono {
        content.push_str("run/main_scene=\"res://scenes/main.tscn\"\n");
    }

    content.push_str("config/features=PackedStringArray(\"4.2\", \"Forward+\")\n\n");

    content.push_str("[display]\n");
    content.push_str("window/size/viewport_width=1280\n");
    content.push_str("window/size/viewport_height=720\n");
    content.push_str("window/stretch/mode=\"canvas_items\"\n\n");

    content.push_str("[rendering]\n");
    if !template.godot.rendering.is_empty() {
        content.push_str(&format!("renderer/rendering_method=\"{}\"\n", template.godot.rendering));
    } else {
        content.push_str("renderer/rendering_method=\"forward_plus\"\n");
    }

    content
}

fn create_directory_structure(project_dir: &Path, template: &Template) -> Result<Vec<String>, String> {
    let default_dirs = vec![
        "scenes",
        "scripts",
        "assets",
        "assets/sprites",
        "assets/audio",
        "assets/fonts",
    ];

    let mut created = Vec::new();

    for dir in &default_dirs {
        let path = project_dir.join(dir);
        fs::create_dir_all(&path)
            .map_err(|e| format!("创建目录失败: {}", e))?;
        created.push(dir.to_string());
    }

    for dir_spec in &template.directories {
        let path = project_dir.join(&dir_spec.path);
        fs::create_dir_all(&path)
            .map_err(|e| format!("创建目录 {} 失败: {}", dir_spec.path, e))?;
        if !created.contains(&dir_spec.path) {
            created.push(dir_spec.path.clone());
        }
    }

    Ok(created)
}

fn create_main_scene(project_dir: &Path) -> Result<(), String> {
    let scenes_dir = project_dir.join("scenes");
    fs::create_dir_all(&scenes_dir)
        .map_err(|e| format!("创建场景目录失败: {}", e))?;

    let main_tscn = scenes_dir.join("main.tscn");
    let content = "[gd_scene load_steps=2 format=3]\n\n[ext_resource type=\"Script\" path=\"res://scripts/main.gd\" id=\"1\"]\n\n[node name=\"Main\" type=\"Node2D\"]\nscript = ExtResource(\"1\")\n";
    fs::write(&main_tscn, content)
        .map_err(|e| format!("创建主场景失败: {}", e))?;

    let scripts_dir = project_dir.join("scripts");
    fs::create_dir_all(&scripts_dir)
        .map_err(|e| format!("创建脚本目录失败: {}", e))?;

    let main_gd = scripts_dir.join("main.gd");
    let script_content = "extends Node2D\n\nfunc _ready() -> void:\n\tpass\n";
    fs::write(&main_gd, script_content)
        .map_err(|e| format!("创建主脚本失败: {}", e))?;

    Ok(())
}

fn generate_export_presets_cfg(template: &Template) -> String {
    let mut content = String::new();
    content.push_str("[preset.0]\n\n");

    for (i, preset) in template.export_presets.iter().enumerate() {
        if i > 0 {
            content.push_str(&format!("[preset.{}]\n\n", i));
        }
        content.push_str(&format!("name=\"{}\"\n", preset.name));
        content.push_str(&format!("platform=\"{}\"\n", preset.platform));
        content.push_str("runnable=true\n");
        content.push_str("dedicated_server=false\n");
        content.push_str("custom_features=\"\"\n");
        content.push_str("export_filter=\"all_resources\"\n");
        content.push_str("export_filter.exclude_filter=\"\"\n");
        content.push_str("export_filter.export_plugins=PoolStringArray()\n");
        content.push_str("script_encryption_key=\"\"\n");
        content.push_str("script_encryption_key.editable=false\n");
        if !preset.config.is_null() {
            content.push_str(&format!("custom_template={}\n", preset.config));
        }
        content.push('\n');
    }

    content.push_str("[preset.0.options]\n\n");
    for preset in &template.export_presets {
        match preset.platform.as_str() {
            "windows" => {
                content.push_str("custom_template/debug=\"\"\n");
                content.push_str("custom_template/release=\"\"\n");
                content.push_str("debug/export_console_wrapper=1\n");
                content.push_str("binary_format/embed_pck=false\n");
                content.push_str("texture_format/bptc=true\n");
                content.push_str("texture_format/s3tc=true\n");
                content.push_str("texture_format/etc=false\n");
                content.push_str("texture_format/etc2=false\n");
                content.push_str("binary_format/architecture=\"x86_64\"\n");
            }
            "web" => {
                content.push_str("custom_template/debug=\"\"\n");
                content.push_str("custom_template/release=\"\"\n");
                content.push_str("variant/extensions_capability=false\n");
                content.push_str("vram_texture_compression/for_desktop=true\n");
                content.push_str("vram_texture_compression/for_mobile=false\n");
                content.push_str("html/export_icon=\"\"\n");
                content.push_str("html/custom_html_shell=\"\"\n");
                content.push_str("html/head_include=\"\"\n");
            }
            "linux" => {
                content.push_str("custom_template/debug=\"\"\n");
                content.push_str("custom_template/release=\"\"\n");
                content.push_str("debug/export_console_wrapper=1\n");
                content.push_str("binary_format/embed_pck=false\n");
                content.push_str("texture_format/bptc=true\n");
                content.push_str("texture_format/s3tc=true\n");
                content.push_str("texture_format/etc=false\n");
                content.push_str("texture_format/etc2=false\n");
                content.push_str("binary_format/architecture=\"x86_64\"\n");
            }
            "macos" => {
                content.push_str("custom_template/debug=\"\"\n");
                content.push_str("custom_template/release=\"\"\n");
                content.push_str("debug/export_console_wrapper=1\n");
                content.push_str("application/icon=\"\"\n");
                content.push_str("application/icon_interpolation=4\n");
                content.push_str("application/bundle_identifier=\"\"\n");
                content.push_str("application/signature=\"\"\n");
                content.push_str("application/app_category=\"Games\"\n");
                content.push_str("application/short_version=\"1.0\"\n");
                content.push_str("application/version=\"1.0\"\n");
                content.push_str("application/copyright=\"\"\n");
                content.push_str("display/high_res=true\n");
                content.push_str("privacy/camera_usage_description=\"\"\n");
                content.push_str("privacy/microphone_usage_description=\"\"\n");
            }
            _ => {}
        }
        break;
    }

    content
}

fn generate_harbor_yml(template: &Template) -> String {
    let mut yaml = String::new();
    yaml.push_str("version: 2\n\n");
    yaml.push_str(&format!("godot:\n  version: \"{}\"\n  mono: {}\n\n", template.godot.version, template.godot.mono));

    if !template.plugins.is_empty() {
        yaml.push_str("plugins:\n");
        for plugin in &template.plugins {
            yaml.push_str(&format!("  - name: {}\n", plugin.name));
            yaml.push_str(&format!("    version: \"{}\"\n", plugin.version));
            yaml.push_str(&format!("    source: {}\n", match plugin.source {
                TemplatePluginSource::AssetStore => "asset-store",
                TemplatePluginSource::Git => "git",
                TemplatePluginSource::Local => "local",
            }));
            if !plugin.url.is_empty() {
                yaml.push_str(&format!("    url: \"{}\"\n", plugin.url));
            }
            if !plugin.git_ref.is_empty() {
                yaml.push_str(&format!("    ref: \"{}\"\n", plugin.git_ref));
            }
            if !plugin.mount.is_empty() {
                yaml.push_str(&format!("    mount: {}\n", plugin.mount));
            }
        }
        yaml.push('\n');
    }

    if !template.export_presets.is_empty() {
        yaml.push_str("export_presets:\n");
        for preset in &template.export_presets {
            yaml.push_str(&format!("  - platform: \"{}\"\n", preset.platform));
            yaml.push_str(&format!("    name: \"{}\"\n", preset.name));
        }
        yaml.push('\n');
    }

    yaml.push_str("settings:\n  mount_strategy: copy\n  auto_sync: true\n");
    yaml
}

#[tauri::command]
pub async fn instantiate_template(
    app: AppHandle,
    template_id: String,
    project_name: String,
    target_dir: String,
) -> Result<TemplateInstantiationResult, String> {
    let start = std::time::Instant::now();

    let template = get_hub_template(app.clone(), template_id.clone())?;

    let project_dir = Path::new(&target_dir).join(&project_name);
    if project_dir.exists() {
        return Err(format!("目标目录已存在: {}", project_dir.display()));
    }

    let _ = app.emit("template-instantiation-progress", TemplateInstantiationProgress {
        template_id: template.template_id.clone(),
        stage: "creating".to_string(),
        progress: 0.05,
        message: format!("正在创建项目目录 {}...", project_name),
        detail: String::new(),
    });

    fs::create_dir_all(&project_dir)
        .map_err(|e| format!("创建项目目录失败: {}", e))?;

    let _ = app.emit("template-instantiation-progress", TemplateInstantiationProgress {
        template_id: template.template_id.clone(),
        stage: "generating".to_string(),
        progress: 0.1,
        message: "正在生成项目文件...".to_string(),
        detail: String::new(),
    });

    let project_godot_content = generate_project_godot(&template);
    fs::write(project_dir.join("project.godot"), project_godot_content)
        .map_err(|e| format!("写入 project.godot 失败: {}", e))?;

    let created_dirs = create_directory_structure(&project_dir, &template)?;

    create_main_scene(&project_dir)?;

    let _ = app.emit("template-instantiation-progress", TemplateInstantiationProgress {
        template_id: template.template_id.clone(),
        stage: "generating_harbor_yml".to_string(),
        progress: 0.15,
        message: "正在生成 .harbor.yml...".to_string(),
        detail: String::new(),
    });

    let harbor_yml = generate_harbor_yml(&template);
    fs::write(project_dir.join(".harbor.yml"), harbor_yml)
        .map_err(|e| format!("写入 .harbor.yml 失败: {}", e))?;

    let mut installed_plugins = Vec::new();
    let mut failed_plugins = Vec::new();

    if !template.plugins.is_empty() {
        let total = template.plugins.len();
        for (i, plugin_spec) in template.plugins.iter().enumerate() {
            let progress = 0.15 + 0.55 * ((i + 1) as f64 / total as f64);
            let _ = app.emit("template-instantiation-progress", TemplateInstantiationProgress {
                template_id: template.template_id.clone(),
                stage: "installing_plugin".to_string(),
                progress,
                message: format!("正在安装插件 {}/{}: {}...", i + 1, total, plugin_spec.name),
                detail: plugin_spec.name.clone(),
            });

            match install_template_plugin(&app, &project_dir, plugin_spec) {
                Ok(_) => {
                    installed_plugins.push(plugin_spec.name.clone());
                }
                Err(e) => {
                    failed_plugins.push(format!("{}: {}", plugin_spec.name, e));
                }
            }
        }
    }

    let mut engine_installed = false;

    let _ = app.emit("template-instantiation-progress", TemplateInstantiationProgress {
        template_id: template.template_id.clone(),
        stage: "checking_engine".to_string(),
        progress: 0.72,
        message: format!("正在检查引擎 Godot {}...", template.godot.version),
        detail: String::new(),
    });

    let storage = get_storage(&app);
    let engines: Vec<Engine> = storage.load_or_default("engines.json");
    let engine_exists = engines.iter().any(|e| {
        let ev: Vec<&str> = e.version.split('.').collect();
        let tv: Vec<&str> = template.godot.version.split('.').collect();
        if ev.len() >= 2 && tv.len() >= 2 {
            ev[0] == tv[0] && ev[1] == tv[1] && e.is_mono == template.godot.mono
        } else {
            e.version == template.godot.version && e.is_mono == template.godot.mono
        }
    });

    if !engine_exists {
        let _ = app.emit("template-instantiation-progress", TemplateInstantiationProgress {
            template_id: template.template_id.clone(),
            stage: "downloading_engine".to_string(),
            progress: 0.75,
            message: format!("正在下载引擎 Godot {}...", template.godot.version),
            detail: String::new(),
        });

        let settings = load_settings(&app);
        let mirror = settings.engine_mirrors.iter().find(|m| m.enabled)
            .cloned()
            .unwrap_or_else(EngineMirrorConfig::official);

        match crate::engine_downloader::EngineDownloader::fetch_remote_versions(&mirror, &[]).await {
            Ok(versions) => {
                let variant = if template.godot.mono { "mono" } else { "standard" };
                let matching = versions.iter().find(|v| {
                    v.version == template.godot.version && v.variant == variant
                });
                if let Some(remote_version) = matching {
                    let engines_dir = get_data_dir(&app).join("engines");
                    match crate::engine_downloader::EngineDownloader::download_and_install(&app, remote_version, engines_dir).await {
                        Ok(installed_path) => {
                            if let Ok(engine) = crate::engine::EngineManager::get_engine_info(&installed_path.to_string_lossy()) {
                                let mut engines: Vec<Engine> = storage.load_or_default("engines.json");
                                engines.retain(|e| e.path != engine.path);
                                engines.push(engine);
                                storage.save("engines.json", &engines)
                                    .map_err(|e| format!("保存引擎信息失败: {}", e))?;
                                engine_installed = true;
                            }
                        }
                        Err(e) => {
                            failed_plugins.push(format!("引擎下载失败: {}", e));
                        }
                    }
                } else {
                    failed_plugins.push(format!("未找到引擎 Godot {} {} 版本可供下载", template.godot.version, variant));
                }
            }
            Err(e) => {
                failed_plugins.push(format!("获取远程引擎版本失败: {}", e));
            }
        }
    } else {
        engine_installed = true;
    }

    let mut applied_presets = Vec::new();

    if !template.export_presets.is_empty() {
        let _ = app.emit("template-instantiation-progress", TemplateInstantiationProgress {
            template_id: template.template_id.clone(),
            stage: "applying_presets".to_string(),
            progress: 0.85,
            message: "正在应用导出预设...".to_string(),
            detail: String::new(),
        });

        let presets_content = generate_export_presets_cfg(&template);
        if let Err(e) = fs::write(project_dir.join("export_presets.cfg"), presets_content) {
            failed_plugins.push(format!("写入导出预设失败: {}", e));
        }

        for preset in &template.export_presets {
            applied_presets.push(preset.name.clone());
        }
    }

    let _ = app.emit("template-instantiation-progress", TemplateInstantiationProgress {
        template_id: template.template_id.clone(),
        stage: "registering".to_string(),
        progress: 0.9,
        message: "正在注册项目...".to_string(),
        detail: String::new(),
    });

    let project = Project {
        project_id: uuid::Uuid::new_v4().to_string(),
        name: project_name.clone(),
        path: project_dir.to_string_lossy().to_string(),
        godot_version: template.godot.version.clone(),
        icon_path: String::new(),
        group: String::new(),
        status: ProjectStatus::Ready,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        last_synced_at: None,
        last_opened_at: None,
        last_used_engine_id: None,
    };

    let project_id = project.project_id.clone();
    let mut projects: Vec<Project> = storage.load_or_default("projects.json");
    projects.push(project.clone());
    storage.save("projects.json", &projects)
        .map_err(|e| format!("保存项目列表失败: {}", e))?;

    let mut bindings: Vec<ProjectBinding> = storage.load_or_default("bindings.json");
    for binding in bindings.iter_mut() {
        if binding.project_id.is_empty() {
            binding.project_id = project_id.clone();
        }
    }
    storage.save("bindings.json", &bindings)
        .map_err(|e| format!("保存绑定关系失败: {}", e))?;

    let settings = load_settings(&app);
    let linker = Linker::new(settings.mount_strategy);
    let data_dir = get_data_dir(&app);
    let plugin_base_path = data_dir.join("plugins");
    let project_bindings: Vec<ProjectBinding> = storage.load_or_default("bindings.json");
    let desired_bindings: Vec<ProjectBinding> = project_bindings.iter()
        .filter(|b| b.project_id == project_id)
        .cloned()
        .collect();

    if !desired_bindings.is_empty() {
        let _ = linker.apply_bindings(
            &project_dir.to_string_lossy(),
            &[],
            &desired_bindings,
            &plugin_base_path.to_string_lossy(),
        );
    }

    let duration = start.elapsed().as_secs();

    let _ = app.emit("template-instantiation-progress", TemplateInstantiationProgress {
        template_id: template.template_id.clone(),
        stage: "complete".to_string(),
        progress: 1.0,
        message: format!("项目 {} 创建完成！", project_name),
        detail: String::new(),
    });

    Ok(TemplateInstantiationResult {
        project_id: project.project_id,
        project_name: project_name.clone(),
        project_path: project_dir.to_string_lossy().to_string(),
        template_id: template.template_id.clone(),
        godot_version: template.godot.version.clone(),
        installed_plugins,
        failed_plugins,
        created_directories: created_dirs,
        applied_presets,
        engine_installed,
        duration_secs: duration,
    })
}

fn install_template_plugin(
    app: &AppHandle,
    _project_dir: &Path,
    plugin_spec: &TemplatePlugin,
) -> Result<(), String> {
    let storage = get_storage(app);
    let mut plugins: Vec<Plugin> = storage.load_or_default("plugins.json");
    let mut bindings: Vec<ProjectBinding> = storage.load_or_default("bindings.json");

    let existing_plugin = plugins.iter().find(|p| {
        p.name.to_lowercase() == plugin_spec.name.to_lowercase()
    });

    let plugin_id = if let Some(ep) = existing_plugin {
        ep.plugin_id.clone()
    } else {
        let manager = get_plugin_manager(app);
        let new_plugin = match plugin_spec.source {
            TemplatePluginSource::Git => {
                if plugin_spec.url.is_empty() {
                    return Err(format!("插件 {} 为 Git 来源但未提供 URL", plugin_spec.name));
                }
                let git_ref = if plugin_spec.git_ref.is_empty() { None } else { Some(plugin_spec.git_ref.as_str()) };
                manager.import_from_git(&plugin_spec.url, git_ref, app)
                    .map_err(|e| format!("从 Git 导入插件 {} 失败: {}", plugin_spec.name, e))?
            }
            TemplatePluginSource::AssetStore => {
                if plugin_spec.url.is_empty() {
                    return Err(format!("插件 {} 为 Asset Store 来源但未提供 URL", plugin_spec.name));
                }
                manager.import_from_url(&plugin_spec.url, app)
                    .map_err(|e| format!("从 Asset Store 导入插件 {} 失败: {}", plugin_spec.name, e))?
            }
            TemplatePluginSource::Local => {
                return Err(format!("插件 {} 为本地来源，模板实例化无法自动安装，需手动导入", plugin_spec.name));
            }
        };
        let pid = new_plugin.plugin_id.clone();
        plugins.push(new_plugin);
        pid
    };

    storage.save("plugins.json", &plugins)
        .map_err(|e| format!("保存插件列表失败: {}", e))?;

    let mount_path = format!("addons/{}", plugin_spec.name);
    let already_bound = bindings.iter().any(|b| b.project_id.is_empty() && b.plugin_id == plugin_id && b.mount_path == mount_path);

    if !already_bound {
        let plugins: Vec<Plugin> = storage.load_or_default("plugins.json");
        let plugin = plugins.iter().find(|p| p.plugin_id == plugin_id)
            .ok_or("刚导入的插件未找到".to_string())?;
        let version = plugin.versions.first()
            .ok_or(format!("插件 {} 无可用版本", plugin_spec.name))?;
        let unit = version.units.first()
            .ok_or(format!("插件 {} 无可用单元", plugin_spec.name))?;

        bindings.push(ProjectBinding::new(
            String::new(),
            plugin_id.clone(),
            version.version_id.clone(),
            unit.unit_id.clone(),
            mount_path,
            plugin_spec.subdirectory.clone(),
        ));

        storage.save("bindings.json", &bindings)
            .map_err(|e| format!("保存绑定关系失败: {}", e))?;
    }

    Ok(())
}

#[tauri::command]
pub fn generate_template_from_project(
    app: AppHandle,
    project_id: String,
    template_name: String,
    category: String,
) -> Result<Template, String> {
    let storage = get_storage(&app);
    let projects: Vec<Project> = storage.load_or_default("projects.json");
    let project = projects.iter().find(|p| p.project_id == project_id)
        .ok_or("项目不存在".to_string())?;

    let project_path = Path::new(&project.path);
    if !project_path.exists() {
        return Err("项目路径不存在".to_string());
    }

    let cat = match category.as_str() {
        "starter_2d" => TemplateCategory::Starter2D,
        "starter_3d" => TemplateCategory::Starter3D,
        "rpg" => TemplateCategory::RPG,
        "platformer" => TemplateCategory::Platformer,
        "multiplayer" => TemplateCategory::Multiplayer,
        "mobile" => TemplateCategory::Mobile,
        "blank" => TemplateCategory::Blank,
        _ => TemplateCategory::Custom,
    };

    let mut template = Template::new(template_name, cat, project.godot_version.clone());

    let bindings: Vec<ProjectBinding> = storage.load_or_default(
        &format!("bindings_{}.json", project_id)
    );

    for binding in &bindings {
        let plugins: Vec<Plugin> = storage.load_or_default("plugins.json");
        if let Some(plugin) = plugins.iter().find(|p| p.plugin_id == binding.plugin_id) {
            let source = match plugin.source.source_type {
                SourceType::AssetLibrary => TemplatePluginSource::AssetStore,
                SourceType::Git => TemplatePluginSource::Git,
                _ => TemplatePluginSource::Local,
            };
            template.plugins.push(TemplatePlugin {
                name: plugin.name.clone(),
                version: plugin.versions.first().map(|v| v.version.clone()).unwrap_or_default(),
                source,
                url: if plugin.source.source_type == SourceType::Git { plugin.source.url.clone() } else { String::new() },
                git_ref: plugin.source.git_ref.clone(),
                mount: String::new(),
                subdirectory: binding.subdirectory.clone(),
            });
        }
    }

    let addons_dir = project_path.join("addons");
    if addons_dir.exists() {
        if let Ok(entries) = fs::read_dir(&addons_dir) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    let dir_name = entry.file_name().to_string_lossy().to_string();
                    let already_listed = template.plugins.iter().any(|p| p.name == dir_name);
                    if !already_listed {
                        template.plugins.push(TemplatePlugin {
                            name: dir_name,
                            version: "1.0.0".to_string(),
                            source: TemplatePluginSource::Local,
                            url: String::new(),
                            git_ref: String::new(),
                            mount: String::new(),
                            subdirectory: String::new(),
                        });
                    }
                }
            }
        }
    }

    template.is_builtin = false;
    template.description = format!("从项目 {} 生成的模板", project.name);

    save_hub_template(app, template)
}

#[tauri::command]
pub fn ensure_builtin_templates(app: AppHandle) -> Result<Vec<Template>, String> {
    let templates_dir = get_templates_dir(&app);
    fs::create_dir_all(&templates_dir)
        .map_err(|e| format!("创建模板目录失败: {}", e))?;

    let existing = list_hub_templates(app.clone())?;
    let builtin_ids: Vec<&str> = existing.iter()
        .filter(|t| t.is_builtin)
        .map(|t| t.template_id.as_str())
        .collect();

    let mut created = Vec::new();

    let blank_template = Template {
        template_id: "builtin-blank-recommended".to_string(),
        name: "空白项目（推荐插件）".to_string(),
        description: "空白 Godot 4 项目，预装推荐插件，适合大多数2D/3D项目".to_string(),
        author: "Godot Harbor".to_string(),
        category: TemplateCategory::Blank,
        tags: vec!["blank".to_string(), "recommended".to_string(), "2d".to_string(), "3d".to_string()],
        icon_url: String::new(),
        preview_images: Vec::new(),
        godot: TemplateGodotConfig {
            version: "4.4.1".to_string(),
            mono: false,
            rendering: String::new(),
        },
        plugins: vec![
            TemplatePlugin { name: "phantom-camera".to_string(), version: "0.11".to_string(), source: TemplatePluginSource::AssetStore, url: String::new(), git_ref: String::new(), mount: "copy".to_string(), subdirectory: String::new() },
        ],
        directories: vec![
            TemplateDirectory { path: "scenes".to_string(), description: "场景文件".to_string() },
            TemplateDirectory { path: "scripts".to_string(), description: "脚本文件".to_string() },
            TemplateDirectory { path: "assets/sprites".to_string(), description: "精灵图".to_string() },
            TemplateDirectory { path: "assets/audio".to_string(), description: "音频文件".to_string() },
            TemplateDirectory { path: "assets/fonts".to_string(), description: "字体文件".to_string() },
        ],
        export_presets: vec![
            TemplateExportPreset { platform: "windows".to_string(), name: "Windows Desktop".to_string(), config: serde_json::Value::Null },
        ],
        project_config: TemplateProjectConfig::default(),
        is_builtin: true,
        source_url: String::new(),
        version: "1.0.0".to_string(),
        created_at: chrono::Utc::now(),
        updated_at: None,
    };

    let platformer_template = Template {
        template_id: "builtin-2d-platformer".to_string(),
        name: "2D 平台起步包".to_string(),
        description: "2D 平台游戏起步模板，包含 Phantom Camera、输入映射和基础场景组织".to_string(),
        author: "Godot Harbor".to_string(),
        category: TemplateCategory::Starter2D,
        tags: vec!["2d".to_string(), "platformer".to_string(), "starter".to_string()],
        icon_url: String::new(),
        preview_images: Vec::new(),
        godot: TemplateGodotConfig {
            version: "4.4.1".to_string(),
            mono: false,
            rendering: "compatible".to_string(),
        },
        plugins: vec![
            TemplatePlugin { name: "phantom-camera".to_string(), version: "0.11".to_string(), source: TemplatePluginSource::AssetStore, url: String::new(), git_ref: String::new(), mount: "copy".to_string(), subdirectory: String::new() },
        ],
        directories: vec![
            TemplateDirectory { path: "scenes/levels".to_string(), description: "关卡场景".to_string() },
            TemplateDirectory { path: "scenes/player".to_string(), description: "玩家场景".to_string() },
            TemplateDirectory { path: "scenes/enemies".to_string(), description: "敌人场景".to_string() },
            TemplateDirectory { path: "scenes/ui".to_string(), description: "UI 场景".to_string() },
            TemplateDirectory { path: "scripts/player".to_string(), description: "玩家脚本".to_string() },
            TemplateDirectory { path: "scripts/enemies".to_string(), description: "敌人脚本".to_string() },
            TemplateDirectory { path: "assets/sprites/player".to_string(), description: "玩家精灵".to_string() },
            TemplateDirectory { path: "assets/sprites/enemies".to_string(), description: "敌人精灵".to_string() },
            TemplateDirectory { path: "assets/sprites/tilesets".to_string(), description: "瓦片集".to_string() },
            TemplateDirectory { path: "assets/audio/sfx".to_string(), description: "音效".to_string() },
            TemplateDirectory { path: "assets/audio/music".to_string(), description: "背景音乐".to_string() },
        ],
        export_presets: vec![
            TemplateExportPreset { platform: "windows".to_string(), name: "Windows Desktop".to_string(), config: serde_json::Value::Null },
            TemplateExportPreset { platform: "web".to_string(), name: "HTML5".to_string(), config: serde_json::Value::Null },
        ],
        project_config: TemplateProjectConfig::default(),
        is_builtin: true,
        source_url: String::new(),
        version: "1.0.0".to_string(),
        created_at: chrono::Utc::now(),
        updated_at: None,
    };

    for template in [blank_template, platformer_template] {
        if !builtin_ids.contains(&template.template_id.as_str()) {
            let saved = save_hub_template(app.clone(), template)?;
            created.push(saved);
        }
    }

    Ok(created)
}
