use serde_json::{json, Value};
use crate::models::*;
use crate::harbor_config;
use super::server::McpContext;

pub fn list_tools() -> Value {
    json!({
        "tools": [
            {
                "name": "check_drift",
                "description": "Check environment drift for a Godot project. Compares .harbor.yml declarations against actual installed state.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "project_id": { "type": "string", "description": "The project ID to check" }
                    },
                    "required": ["project_id"]
                }
            },
            {
                "name": "sync_environment",
                "description": "Synchronize project environment to match .harbor.yml declarations. Installs missing plugins, updates versions.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "project_id": { "type": "string", "description": "The project ID to sync" }
                    },
                    "required": ["project_id"]
                }
            },
            {
                "name": "install_plugin",
                "description": "Install a plugin to a project. Supports asset-store, git, and local sources.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "project_id": { "type": "string", "description": "The project ID" },
                        "plugin_name": { "type": "string", "description": "Plugin name" },
                        "source": { "type": "string", "enum": ["asset-store", "git", "local"], "description": "Plugin source type" },
                        "url": { "type": "string", "description": "URL for git/url sources" },
                        "version": { "type": "string", "description": "Version to install" }
                    },
                    "required": ["project_id", "plugin_name", "source"]
                }
            },
            {
                "name": "switch_engine",
                "description": "Switch the engine version for a project. Downloads the engine if not installed.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "project_id": { "type": "string", "description": "The project ID" },
                        "version": { "type": "string", "description": "Godot engine version (e.g. 4.4.1)" },
                        "mono": { "type": "boolean", "description": "Whether to use .NET/Mono version", "default": false }
                    },
                    "required": ["project_id", "version"]
                }
            },
            {
                "name": "create_from_template",
                "description": "Create a new project from a template.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "template_id": { "type": "string", "description": "Template ID" },
                        "project_name": { "type": "string", "description": "New project name" },
                        "target_dir": { "type": "string", "description": "Target directory path" }
                    },
                    "required": ["template_id", "project_name", "target_dir"]
                }
            },
            {
                "name": "build_project",
                "description": "Build a Godot project for a target platform using headless export.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "project_id": { "type": "string", "description": "The project ID" },
                        "platform": { "type": "string", "enum": ["Windows", "Web", "Linux", "MacOS", "Android", "IOS"], "description": "Target platform" }
                    },
                    "required": ["project_id", "platform"]
                }
            },
            {
                "name": "check_updates",
                "description": "Check for available updates for plugins and engines.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "project_id": { "type": "string", "description": "Optional project ID to scope check" }
                    }
                }
            }
        ]
    })
}

pub fn call_tool(ctx: &McpContext, params: &Value) -> Value {
    let tool_name = params["name"].as_str().unwrap_or("");
    let arguments = &params["arguments"];

    let result = match tool_name {
        "check_drift" => tool_check_drift(ctx, arguments),
        "sync_environment" => tool_sync_environment(ctx, arguments),
        "install_plugin" => tool_install_plugin(ctx, arguments),
        "switch_engine" => tool_switch_engine(ctx, arguments),
        "create_from_template" => tool_create_from_template(ctx, arguments),
        "build_project" => tool_build_project(ctx, arguments),
        "check_updates" => tool_check_updates(ctx, arguments),
        _ => Err(format!("Unknown tool: {}", tool_name)),
    };

    match result {
        Ok(content) => json!({
            "content": [{"type": "text", "text": content}],
            "isError": false
        }),
        Err(e) => json!({
            "content": [{"type": "text", "text": e}],
            "isError": true
        }),
    }
}

fn tool_check_drift(ctx: &McpContext, args: &Value) -> Result<String, String> {
    let project_id = args["project_id"].as_str().ok_or("project_id is required")?;
    let projects: Vec<Project> = ctx.storage.load_or_default("projects.json");
    let project = projects.iter().find(|p| p.project_id == project_id)
        .ok_or("Project not found".to_string())?;

    let config = harbor_config::read_harbor_config_from_project(&project.path)
        .map_err(|e| format!("Failed to read .harbor.yml: {}", e))?
        .ok_or("No .harbor.yml found".to_string())?;

    let config_upgraded = if config.version < 2 { config.upgrade_to_v2() } else { config };

    let engines: Vec<Engine> = ctx.storage.load_or_default("engines.json");
    let bindings: Vec<ProjectBinding> = ctx.storage.load_or_default("bindings.json");
    let plugins: Vec<Plugin> = ctx.storage.load_or_default("plugins.json");

    let mut drift_items = Vec::new();

    if let Some(ref godot_cfg) = config_upgraded.godot {
        let engine_match = engines.iter().find(|e| {
            let ev: Vec<&str> = e.version.split('.').collect();
            let tv: Vec<&str> = godot_cfg.version.split('.').collect();
            if ev.len() >= 2 && tv.len() >= 2 {
                ev[0] == tv[0] && ev[1] == tv[1] && e.is_mono == godot_cfg.mono
            } else {
                e.version == godot_cfg.version && e.is_mono == godot_cfg.mono
            }
        });
        if engine_match.is_none() {
            drift_items.push(format!("⚠️ Engine: Godot {} (mono={}) not installed", godot_cfg.version, godot_cfg.mono));
        } else {
            drift_items.push(format!("✅ Engine: Godot {} installed", godot_cfg.version));
        }
    }

    let project_bindings: Vec<&ProjectBinding> = bindings.iter()
        .filter(|b| b.project_id == project_id)
        .collect();

    for pc in &config_upgraded.plugins {
        let binding = project_bindings.iter().find(|b| {
            plugins.iter().find(|p| p.plugin_id == b.plugin_id)
                .map_or(false, |p| p.name.to_lowercase() == pc.name.to_lowercase())
        });
        if let Some(_b) = binding {
            drift_items.push(format!("✅ Plugin: {} v{} installed", pc.name, pc.version));
        } else {
            drift_items.push(format!("⚠️ Plugin: {} v{} not installed", pc.name, pc.version));
        }
    }

    let has_drift = drift_items.iter().any(|i| i.contains("⚠️"));
    let header = if has_drift { "🚨 Environment drift detected" } else { "✅ No drift detected" };

    Ok(format!("{}\n\n{}", header, drift_items.join("\n")))
}

fn tool_sync_environment(ctx: &McpContext, args: &Value) -> Result<String, String> {
    let project_id = args["project_id"].as_str().ok_or("project_id is required")?;
    let projects: Vec<Project> = ctx.storage.load_or_default("projects.json");
    let project = projects.iter().find(|p| p.project_id == project_id)
        .ok_or("Project not found".to_string())?;

    let config = harbor_config::read_harbor_config_from_project(&project.path)
        .map_err(|e| format!("Failed to read .harbor.yml: {}", e))?
        .ok_or("No .harbor.yml found".to_string())?;

    let config_upgraded = if config.version < 2 { config.upgrade_to_v2() } else { config };

    let engines: Vec<Engine> = ctx.storage.load_or_default("engines.json");
    let bindings: Vec<ProjectBinding> = ctx.storage.load_or_default("bindings.json");
    let plugins: Vec<Plugin> = ctx.storage.load_or_default("plugins.json");

    let mut synced = 0;
    let mut skipped = 0;
    let mut details = Vec::new();

    if let Some(ref godot_cfg) = config_upgraded.godot {
        let engine_match = engines.iter().find(|e| {
            let ev: Vec<&str> = e.version.split('.').collect();
            let tv: Vec<&str> = godot_cfg.version.split('.').collect();
            if ev.len() >= 2 && tv.len() >= 2 {
                ev[0] == tv[0] && ev[1] == tv[1] && e.is_mono == godot_cfg.mono
            } else {
                e.version == godot_cfg.version && e.is_mono == godot_cfg.mono
            }
        });
        if let Some(_engine) = engine_match {
            details.push(format!("✅ Engine {} already installed", godot_cfg.version));
        } else {
            details.push(format!("⚠️ Engine {} not installed - please install via Harbor GUI", godot_cfg.version));
            skipped += 1;
        }
    }

    let project_bindings: Vec<&ProjectBinding> = bindings.iter()
        .filter(|b| b.project_id == project_id)
        .collect();

    for pc in &config_upgraded.plugins {
        let binding_exists = project_bindings.iter().any(|b| {
            plugins.iter().find(|p| p.plugin_id == b.plugin_id)
                .map_or(false, |p| p.name.to_lowercase() == pc.name.to_lowercase())
        });
        if binding_exists {
            details.push(format!("✅ Plugin {} already bound", pc.name));
        } else {
            let plugin_match = plugins.iter().find(|p| p.name.to_lowercase() == pc.name.to_lowercase());
            if let Some(plugin) = plugin_match {
                let version_id = plugin.versions.first().map(|v| v.version_id.clone()).unwrap_or_default();
                let unit_id = plugin.versions.first()
                    .and_then(|v| v.units.first())
                    .map(|u| u.unit_id.clone())
                    .unwrap_or_default();
                let mount_path = plugin.versions.first()
                    .and_then(|v| v.units.first())
                    .map(|u| {
                        if u.subdirectory.is_empty() {
                            format!("res://addons/{}", u.dir_name)
                        } else {
                            format!("res://{}", u.subdirectory)
                        }
                    })
                    .unwrap_or_else(|| format!("res://addons/{}", plugin.name.to_lowercase()));

                // 拷贝插件文件到项目 addons 目录
                let project_dir = std::path::Path::new(&project.path);
                let plugin_dir = ctx.data_dir.join("plugins").join(&plugin.plugin_id);
                let mut copied = false;
                if let Some(first_version) = plugin.versions.first() {
                    for unit in &first_version.units {
                        let src = plugin_dir.join(&first_version.version_id).join("payload").join(&unit.dir_name);
                        let dst = project_dir.join("addons").join(&unit.dir_name);
                        if src.exists() {
                            if let Ok(()) = copy_dir_recursive(&src, &dst) {
                                copied = true;
                            }
                        }
                    }
                }

                let new_binding = ProjectBinding {
                    project_id: project_id.to_string(),
                    plugin_id: plugin.plugin_id.clone(),
                    version_id,
                    unit_id,
                    mount_path,
                    created_at: chrono::Utc::now(),
                    is_healthy: Some(true),
                    subdirectory: String::new(),
                };
                let mut all_bindings: Vec<ProjectBinding> = ctx.storage.load_or_default("bindings.json");
                all_bindings.push(new_binding);
                ctx.storage.save("bindings.json", &all_bindings)
                    .map_err(|e| format!("Failed to save binding: {}", e))?;
                if copied {
                    details.push(format!("✅ Plugin {} installed and bound to project", pc.name));
                } else {
                    details.push(format!("✅ Plugin {} bound to project (no files copied)", pc.name));
                }
                synced += 1;
            } else {
                details.push(format!("⚠️ Plugin {} not found in library - please install via Harbor GUI", pc.name));
                skipped += 1;
            }
        }
    }

    Ok(format!("Sync completed: {} synced, {} skipped\n\n{}", synced, skipped, details.join("\n")))
}

fn tool_install_plugin(ctx: &McpContext, args: &Value) -> Result<String, String> {
    let project_id = args["project_id"].as_str().ok_or("project_id is required")?;
    let plugin_name = args["plugin_name"].as_str().ok_or("plugin_name is required")?;
    let _source = args["source"].as_str().ok_or("source is required")?;

    // 查找项目
    let projects: Vec<Project> = ctx.storage.load_or_default("projects.json");
    let project = projects.iter().find(|p| p.project_id == project_id)
        .ok_or("Project not found".to_string())?;
    let project_dir = std::path::Path::new(&project.path);

    let plugins: Vec<Plugin> = ctx.storage.load_or_default("plugins.json");
    let plugin = plugins.iter().find(|p| p.name.to_lowercase() == plugin_name.to_lowercase());

    if let Some(plugin) = plugin {
        let version_id = plugin.versions.first().map(|v| v.version_id.clone()).unwrap_or_default();
        let unit_id = plugin.versions.first()
            .and_then(|v| v.units.first())
            .map(|u| u.unit_id.clone())
            .unwrap_or_default();
        let mount_path = plugin.versions.first()
            .and_then(|v| v.units.first())
            .map(|u| {
                if u.subdirectory.is_empty() {
                    format!("res://addons/{}", u.dir_name)
                } else {
                    format!("res://{}", u.subdirectory)
                }
            })
            .unwrap_or_else(|| format!("res://addons/{}", plugin.name.to_lowercase()));

        // 拷贝插件文件到项目 addons 目录
        let plugin_dir = ctx.data_dir.join("plugins").join(&plugin.plugin_id);
        let mut copied_units = Vec::new();
        if let Some(first_version) = plugin.versions.first() {
            for unit in &first_version.units {
                let src = plugin_dir.join(&first_version.version_id).join("payload").join(&unit.dir_name);
                let dst = project_dir.join("addons").join(&unit.dir_name);
                if src.exists() {
                    match copy_dir_recursive(&src, &dst) {
                        Ok(()) => copied_units.push(unit.dir_name.clone()),
                        Err(e) => return Err(format!("Failed to copy plugin files: {}", e)),
                    }
                }
            }
        }

        let new_binding = ProjectBinding {
            project_id: project_id.to_string(),
            plugin_id: plugin.plugin_id.clone(),
            version_id,
            unit_id,
            mount_path,
            created_at: chrono::Utc::now(),
            is_healthy: Some(true),
            subdirectory: String::new(),
        };
        let mut all_bindings: Vec<ProjectBinding> = ctx.storage.load_or_default("bindings.json");
        let already_bound = all_bindings.iter().any(|b| b.project_id == project_id && b.plugin_id == plugin.plugin_id);
        if already_bound {
            return Ok(format!("Plugin {} is already bound to this project", plugin_name));
        }
        all_bindings.push(new_binding);
        ctx.storage.save("bindings.json", &all_bindings)
            .map_err(|e| format!("Failed to save binding: {}", e))?;
        if copied_units.is_empty() {
            Ok(format!("✅ Plugin {} bound to project (no files to copy - plugin may need re-import)", plugin_name))
        } else {
            Ok(format!("✅ Plugin {} installed and bound to project (copied: {})", plugin_name, copied_units.join(", ")))
        }
    } else {
        Ok(format!("⚠️ Plugin '{}' not found in library. Please install it via Harbor GUI first, or provide the source URL.", plugin_name))
    }
}

fn tool_switch_engine(ctx: &McpContext, args: &Value) -> Result<String, String> {
    let project_id = args["project_id"].as_str().ok_or("project_id is required")?;
    let version = args["version"].as_str().ok_or("version is required")?;
    let mono = args["mono"].as_bool().unwrap_or(false);

    let engines: Vec<Engine> = ctx.storage.load_or_default("engines.json");
    let engine = engines.iter().find(|e| {
        let ev: Vec<&str> = e.version.split('.').collect();
        let tv: Vec<&str> = version.split('.').collect();
        if ev.len() >= 2 && tv.len() >= 2 {
            ev[0] == tv[0] && ev[1] == tv[1] && e.is_mono == mono
        } else {
            e.version == version && e.is_mono == mono
        }
    });

    if let Some(engine) = engine {
        let mut projects: Vec<Project> = ctx.storage.load_or_default("projects.json");
        if let Some(project) = projects.iter_mut().find(|p| p.project_id == project_id) {
            project.last_used_engine_id = Some(engine.engine_id.clone());
            project.godot_version = engine.version.clone();
            let project_path = project.path.clone();
            ctx.storage.save("projects.json", &projects)
                .map_err(|e| format!("Failed to update project: {}", e))?;

            let config = harbor_config::read_harbor_config_from_project(&project_path);
            if let Ok(Some(cfg)) = config {
                let mut cfg_upgraded = if cfg.version < 2 { cfg.upgrade_to_v2() } else { cfg };
                cfg_upgraded.godot = Some(harbor_config::HarborGodot {
                    version: engine.version.clone(),
                    mono: engine.is_mono,
                });
                let _ = harbor_config::write_harbor_config_to_project(&project_path, &cfg_upgraded);
            }

            Ok(format!("✅ Switched project engine to Godot {} (mono={})", engine.version, engine.is_mono))
        } else {
            Err("Project not found".to_string())
        }
    } else {
        Ok(format!("⚠️ Godot {} (mono={}) not installed. Please install it via Harbor GUI first.", version, mono))
    }
}

fn tool_create_from_template(ctx: &McpContext, args: &Value) -> Result<String, String> {
    let template_id = args["template_id"].as_str().ok_or("template_id is required")?;
    let project_name = args["project_name"].as_str().ok_or("project_name is required")?;
    let target_dir = args["target_dir"].as_str().ok_or("target_dir is required")?;

    // 读取模板
    let templates_dir = ctx.data_dir.join("templates").join(template_id);
    if !templates_dir.exists() {
        return Err(format!("Template {} not found", template_id));
    }
    let template_yml = templates_dir.join("template.yml");
    let content = std::fs::read_to_string(&template_yml)
        .map_err(|e| format!("Failed to read template: {}", e))?;
    let template: Template = serde_yaml::from_str(&content)
        .map_err(|e| format!("Failed to parse template: {}", e))?;

    let project_dir = std::path::Path::new(target_dir).join(project_name);
    if project_dir.exists() {
        return Err(format!("Target directory already exists: {}", project_dir.display()));
    }

    // 创建项目目录
    std::fs::create_dir_all(&project_dir)
        .map_err(|e| format!("Failed to create project directory: {}", e))?;

    // 生成 project.godot
    let project_godot = generate_project_godot_content(&template);
    std::fs::write(project_dir.join("project.godot"), project_godot)
        .map_err(|e| format!("Failed to create project.godot: {}", e))?;

    // 创建目录结构
    let mut created_dirs = Vec::new();
    for dir in &template.directories {
        let dir_path = project_dir.join(&dir.path);
        if !dir_path.exists() {
            std::fs::create_dir_all(&dir_path)
                .map_err(|e| format!("Failed to create directory {}: {}", dir.path, e))?;
            created_dirs.push(dir.path.clone());
        }
    }

    // 拷贝模板 framework
    let framework_dir = templates_dir.join("framework");
    if framework_dir.exists() {
        copy_dir_recursive(&framework_dir, &project_dir)?;
    }

    // 生成 .harbor.yml
    let harbor_yml = generate_harbor_yml_content(&template);
    std::fs::write(project_dir.join(".harbor.yml"), harbor_yml)
        .map_err(|e| format!("Failed to create .harbor.yml: {}", e))?;

    // 生成 export_presets.cfg
    if !template.export_presets.is_empty() {
        let presets_content = generate_export_presets_content(&template);
        std::fs::write(project_dir.join("export_presets.cfg"), presets_content)
            .map_err(|e| format!("Failed to create export_presets.cfg: {}", e))?;
    }

    // 安装插件（从已安装的插件库中绑定）
    let mut installed_plugins = Vec::new();
    let mut skipped_plugins = Vec::new();
    let plugins: Vec<Plugin> = ctx.storage.load_or_default("plugins.json");
    let mut all_bindings: Vec<ProjectBinding> = ctx.storage.load_or_default("bindings.json");

    let project_id = uuid::Uuid::new_v4().to_string();

    for plugin_spec in &template.plugins {
        let plugin_match = plugins.iter().find(|p| p.name.to_lowercase() == plugin_spec.name.to_lowercase());
        if let Some(plugin) = plugin_match {
            let version_id = plugin.versions.first().map(|v| v.version_id.clone()).unwrap_or_default();
            let unit_id = plugin.versions.first()
                .and_then(|v| v.units.first())
                .map(|u| u.unit_id.clone())
                .unwrap_or_default();
            let mount_path = plugin.versions.first()
                .and_then(|v| v.units.first())
                .map(|u| {
                    if u.subdirectory.is_empty() {
                        format!("res://addons/{}", u.dir_name)
                    } else {
                        format!("res://{}", u.subdirectory)
                    }
                })
                .unwrap_or_else(|| format!("res://addons/{}", plugin.name.to_lowercase()));

            // 拷贝插件文件到项目
            let plugin_dir = ctx.data_dir.join("plugins").join(&plugin.plugin_id);
            if let Some(first_version) = plugin.versions.first() {
                for unit in &first_version.units {
                    let src = plugin_dir.join(&first_version.version_id).join("payload").join(&unit.dir_name);
                    let dst = project_dir.join("addons").join(&unit.dir_name);
                    if src.exists() {
                        if let Ok(()) = copy_dir_recursive(&src, &dst) {
                            // success
                        }
                    }
                }
            }

            let new_binding = ProjectBinding {
                project_id: project_id.clone(),
                plugin_id: plugin.plugin_id.clone(),
                version_id,
                unit_id,
                mount_path,
                created_at: chrono::Utc::now(),
                is_healthy: Some(true),
                subdirectory: String::new(),
            };
            all_bindings.push(new_binding);
            installed_plugins.push(plugin_spec.name.clone());
        } else {
            skipped_plugins.push(plugin_spec.name.clone());
        }
    }
    ctx.storage.save("bindings.json", &all_bindings)
        .map_err(|e| format!("Failed to save bindings: {}", e))?;

    // 注册项目
    let project = Project {
        project_id: project_id.clone(),
        name: project_name.to_string(),
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

    let mut projects: Vec<Project> = ctx.storage.load_or_default("projects.json");
    projects.push(project);
    ctx.storage.save("projects.json", &projects)
        .map_err(|e| format!("Failed to save project: {}", e))?;

    let mut result = vec![
        format!("✅ Project '{}' created from template '{}' at {}", project_name, template_id, project_dir.display()),
        format!("   Godot version: {}", template.godot.version),
        format!("   Directories created: {}", created_dirs.len()),
    ];
    if !installed_plugins.is_empty() {
        result.push(format!("   Plugins installed: {}", installed_plugins.join(", ")));
    }
    if !skipped_plugins.is_empty() {
        result.push(format!("   ⚠️ Plugins skipped (not in library): {}", skipped_plugins.join(", ")));
    }

    Ok(result.join("\n"))
}

fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> Result<(), String> {
    if !src.exists() {
        return Ok(());
    }
    std::fs::create_dir_all(dst)
        .map_err(|e| format!("Failed to create dir {}: {}", dst.display(), e))?;
    for entry in std::fs::read_dir(src).map_err(|e| format!("Failed to read dir {}: {}", src.display(), e))? {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)
                .map_err(|e| format!("Failed to copy {}: {}", src_path.display(), e))?;
        }
    }
    Ok(())
}

fn generate_project_godot_content(template: &Template) -> String {
    let mut content = String::new();
    content.push_str("; Engine configuration file.\n; Created by Godot Harbor MCP Server\n\n");

    content.push_str("[application]\n");
    content.push_str(&format!("config/name=\"{}\"\n", template.name));
    content.push_str("run/main_scene=\"\"\n\n");

    content.push_str("[display]\n");
    content.push_str("window/size/viewport_width=1280\n");
    content.push_str("window/size/viewport_height=720\n");
    content.push_str("window/stretch/mode=\"canvas_items\"\n\n");

    content.push_str("[rendering]\n");
    content.push_str("renderer/rendering_method=\"forward_plus\"\n");

    if !template.plugins.is_empty() {
        content.push('\n');
        content.push_str("[editor_plugins]\n\n");
        let plugin_paths: Vec<String> = template.plugins.iter().map(|p| {
            let dir = if p.subdirectory.is_empty() {
                p.name.to_lowercase().replace(' ', "_")
            } else {
                p.subdirectory.trim_start_matches("addons/").to_string()
            };
            format!("res://addons/{}/plugin.cfg", dir)
        }).collect();
        content.push_str(&format!("enabled=PackedStringArray({})", plugin_paths.iter().map(|p| format!("\"{}\"", p)).collect::<Vec<_>>().join(", ")));
    }

    content
}

fn generate_harbor_yml_content(template: &Template) -> String {
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

fn generate_export_presets_content(template: &Template) -> String {
    let mut content = String::new();
    for (i, preset) in template.export_presets.iter().enumerate() {
        content.push_str(&format!("[preset.{}]\n\n", i));
        content.push_str(&format!("name=\"{}\"\n", preset.name));
        let godot_platform = match preset.platform.as_str() {
            "windows" => "Windows Desktop",
            "macos" => "macOS",
            "linux" => "Linux/X11",
            "web" => "Web",
            "android" => "Android",
            "ios" => "iOS",
            other => other,
        };
        content.push_str(&format!("platform=\"{}\"\n", godot_platform));
        content.push_str("runnable=true\n\n");
    }
    content
}

fn tool_build_project(ctx: &McpContext, args: &Value) -> Result<String, String> {
    let project_id = args["project_id"].as_str().ok_or("project_id is required")?;
    let platform_str = args["platform"].as_str().ok_or("platform is required")?;

    let platform = match platform_str {
        "Windows" => ExportPlatform::Windows,
        "Web" => ExportPlatform::Web,
        "Linux" => ExportPlatform::Linux,
        "MacOS" => ExportPlatform::MacOS,
        "Android" => ExportPlatform::Android,
        "IOS" => ExportPlatform::IOS,
        _ => return Err(format!("Unknown platform: {}", platform_str)),
    };

    let projects: Vec<Project> = ctx.storage.load_or_default("projects.json");
    let project = projects.iter().find(|p| p.project_id == project_id)
        .ok_or("Project not found".to_string())?;

    let engines: Vec<Engine> = ctx.storage.load_or_default("engines.json");
    let engine = engines.iter().find(|e| {
        project.last_used_engine_id.as_ref().map_or(false, |id| &e.engine_id == id)
    }).ok_or("No engine bound to this project".to_string())?;

    let engine_path = std::path::PathBuf::from(&engine.path);
    let godot_bin = if engine_path.is_file() {
        engine_path
    } else {
        crate::engine::EngineManager::find_executable_in_dir(&engine_path)
            .ok_or_else(|| format!("Godot executable not found in {}", engine_path.display()))?
    };

    if !godot_bin.exists() {
        return Err(format!("Engine executable not found: {}", godot_bin.display()));
    }

    let output_dir = std::path::PathBuf::from(&project.path).join("builds").join(platform.to_string());
    std::fs::create_dir_all(&output_dir)
        .map_err(|e| format!("Failed to create output directory: {}", e))?;

    // 查找匹配的 export preset 名称
    let preset_name = find_export_preset_name(&project.path, &platform)
        .unwrap_or_else(|| platform_str.to_string());

    // 根据平台确定输出文件名
    let output_filename = match platform {
        ExportPlatform::Windows => format!("{}.exe", project.name),
        ExportPlatform::MacOS => format!("{}.zip", project.name),
        ExportPlatform::Linux => project.name.clone(),
        ExportPlatform::Web => "index.html".to_string(),
        ExportPlatform::Android => format!("{}.apk", project.name.to_lowercase().replace(' ', "_")),
        ExportPlatform::IOS => format!("{}.ipa", project.name.to_lowercase().replace(' ', "_")),
    };
    let output_path = output_dir.join(&output_filename);

    let output = std::process::Command::new(&godot_bin)
        .arg("--headless")
        .arg("--path").arg(&project.path)
        .arg("--export-release").arg(&preset_name).arg(output_path.to_string_lossy().as_ref())
        .output()
        .map_err(|e| format!("Failed to execute build: {}", e))?;

    let build_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now();
    let success = output.status.success();
    let duration = 0u64;

    let record = BuildRecord {
        build_id,
        project_id: project_id.to_string(),
        project_name: project.name.clone(),
        platform,
        engine_version: engine.version.clone(),
        status: if success { BuildStatus::Success } else { BuildStatus::Failed },
        started_at: now,
        completed_at: Some(chrono::Utc::now()),
        output_path: output_dir.to_string_lossy().to_string(),
        error_message: if success { String::new() } else { String::from_utf8_lossy(&output.stderr).to_string() },
        duration_secs: duration,
    };

    let mut records: Vec<BuildRecord> = ctx.storage.load_or_default("build_records.json");
    records.push(record);
    ctx.storage.save("build_records.json", &records)
        .map_err(|e| format!("Failed to save build record: {}", e))?;

    if success {
        Ok(format!("✅ Build succeeded for {} ({})", project.name, platform_str))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Ok(format!("❌ Build failed for {} ({}):\n{}", project.name, platform_str, &stderr[..stderr.len().min(500)]))
    }
}

fn tool_check_updates(ctx: &McpContext, _args: &Value) -> Result<String, String> {
    let plugins: Vec<Plugin> = ctx.storage.load_or_default("plugins.json");
    let engines: Vec<Engine> = ctx.storage.load_or_default("engines.json");

    let mut results = Vec::new();
    results.push(format!("📋 {} plugins in library", plugins.len()));
    results.push(format!("📋 {} engines installed", engines.len()));
    results.push(String::new());
    results.push("💡 Use Harbor GUI to check for detailed update information.".to_string());

    Ok(results.join("\n"))
}

fn find_export_preset_name(project_path: &str, platform: &ExportPlatform) -> Option<String> {
    let presets_file = std::path::Path::new(project_path).join("export_presets.cfg");
    if !presets_file.exists() {
        return None;
    }
    let content = std::fs::read_to_string(&presets_file).ok()?;
    let target_platform = match platform {
        ExportPlatform::Windows => "Windows Desktop",
        ExportPlatform::MacOS => "macOS",
        ExportPlatform::Linux => "Linux/X11",
        ExportPlatform::Web => "Web",
        ExportPlatform::Android => "Android",
        ExportPlatform::IOS => "iOS",
    };
    // 简单解析 export_presets.cfg 查找匹配平台的 preset 名称
    let mut current_name: Option<String> = None;
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(name_val) = trimmed.strip_prefix("name=") {
            current_name = Some(name_val.trim_matches('"').to_string());
        }
        if let Some(platform_val) = trimmed.strip_prefix("platform=") {
            let p = platform_val.trim_matches('"');
            if p == target_platform {
                return current_name.clone();
            }
        }
    }
    None
}
