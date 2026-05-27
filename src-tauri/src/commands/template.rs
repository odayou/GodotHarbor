use std::fs;
use std::path::Path;
use tauri::{AppHandle, Emitter, Manager};
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

fn get_builtin_framework_dir(app: &AppHandle) -> std::path::PathBuf {
    app.path().resource_dir()
        .unwrap_or_else(|_| {
            std::env::current_exe()
                .unwrap_or_default()
                .parent()
                .unwrap_or(std::path::Path::new("."))
                .to_path_buf()
        })
        .join("templates")
}

fn get_builtin_modules_dir(app: &AppHandle) -> std::path::PathBuf {
    app.path().resource_dir()
        .unwrap_or_else(|_| {
            std::env::current_exe()
                .unwrap_or_default()
                .parent()
                .unwrap_or(std::path::Path::new("."))
                .to_path_buf()
        })
        .join("modules")
}

fn copy_dir_recursive_skip(src: &Path, dst: &Path, skip_names: &[&str]) -> Result<(), String> {
    if !src.exists() {
        return Err(format!("源目录不存在: {}", src.display()));
    }
    fs::create_dir_all(dst)
        .map_err(|e| format!("创建目录 {} 失败: {}", dst.display(), e))?;
    let entries = fs::read_dir(src)
        .map_err(|e| format!("读取目录 {} 失败: {}", src.display(), e))?;
    for entry in entries.flatten() {
        let src_path = entry.path();
        let file_name = entry.file_name();
        let name_str = file_name.to_string_lossy();
        if skip_names.iter().any(|s| *s == name_str) {
            continue;
        }
        let dst_path = dst.join(&file_name);
        if src_path.is_dir() {
            copy_dir_recursive_skip(&src_path, &dst_path, skip_names)?;
        } else {
            fs::copy(&src_path, &dst_path)
                .map_err(|e| format!("拷贝 {} 失败: {}", src_path.display(), e))?;
        }
    }
    Ok(())
}

fn scaffold_template_framework(app: &AppHandle, project_dir: &Path, template: &Template) -> Result<(), String> {
    let framework_dir = get_builtin_framework_dir(app).join(&template.template_id).join("framework");
    if framework_dir.exists() {
        copy_dir_recursive_skip(&framework_dir, project_dir, &["project.godot"])?;
    } else {
        let cached_dir = get_templates_dir_from_template(template).join("framework");
        if cached_dir.exists() {
            copy_dir_recursive_skip(&cached_dir, project_dir, &["project.godot"])?;
        }
    }
    Ok(())
}

fn get_templates_dir_from_template(_template: &Template) -> std::path::PathBuf {
    std::path::PathBuf::new()
}

fn apply_mobile_support(app: &AppHandle, project_dir: &Path) -> Result<(), String> {
    let module_dir = get_builtin_modules_dir(app).join("mobile-support").join("framework");
    if module_dir.exists() {
        copy_dir_recursive_skip(&module_dir, project_dir, &["project.godot"])?;
    }
    Ok(())
}

fn generate_project_godot(template: &Template, enable_mobile: bool) -> String {
    let mut content = String::new();
    content.push_str("; Engine configuration file.\n; It's best edited using the editor UI and not directly,\n; since the parameters that go here are not all obvious.\n;\n; Format:\n;   [section] ; section goes between []\n;   param=value ; assign values to parameters\n\n");

    content.push_str("[application]\n");
    content.push_str(&format!("config/name=\"{}\"\n", template.name));

    content.push_str("run/main_scene=\"res://scenes/main.tscn\"\n");

    let major_minor = template.godot.version.split('.').take(2).collect::<Vec<_>>().join(".");
    let rendering_method = if template.godot.rendering.is_empty() {
        "Forward+"
    } else if template.godot.rendering == "compatible" {
        "Compatibility"
    } else {
        "Forward+"
    };
    content.push_str(&format!("config/features=PackedStringArray(\"{}\", \"{}\")\n\n", major_minor, rendering_method));

    let mut autoloads = collect_autoloads_from_config(&template.project_config);
    if enable_mobile {
        autoloads.push(("TouchManager".to_string(), "res://scripts/autoload/touch_manager.gd".to_string()));
    }
    if !autoloads.is_empty() {
        content.push_str("[autoload]\n\n");
        for (name, path) in &autoloads {
            content.push_str(&format!("{}=*\"{}\"\n", name, path));
        }
        content.push('\n');
    }

    let mut input_mappings = collect_input_mappings_from_config(&template.project_config);
    if enable_mobile {
        input_mappings.push(("touch_left".to_string(), r#""deadzone": 0.5,
"events": [Object(InputEventScreenTouch,"resource_local_to_scene":false,"resource_name":"","device":-1,"index":0,"pressed":false,"canceled":false,"position":Vector2(0, 0),"double_click":false,"script":null)]"#.to_string()));
        input_mappings.push(("touch_right".to_string(), r#""deadzone": 0.5,
"events": [Object(InputEventScreenTouch,"resource_local_to_scene":false,"resource_name":"","device":-1,"index":1,"pressed":false,"canceled":false,"position":Vector2(0, 0),"double_click":false,"script":null)]"#.to_string()));
        input_mappings.push(("joystick_left".to_string(), r#""deadzone": 0.5,
"events": [Object(InputEventScreenDrag,"resource_local_to_scene":false,"resource_name":"","device":-1,"index":0,"position":Vector2(0, 0),"relative":Vector2(0, 0),"velocity":Vector2(0, 0),"script":null)]"#.to_string()));
    }
    if !input_mappings.is_empty() {
        content.push_str("[input]\n\n");
        for (action, mapping) in &input_mappings {
            content.push_str(&format!("{}={{\n{}\n}}\n", action, mapping));
        }
        content.push('\n');
    }

    let layer_names = collect_layer_names_from_config(&template.project_config);
    if !layer_names.is_empty() {
        content.push_str("[layer_names]\n");
        content.push_str(&layer_names);
        content.push('\n');
    }

    content.push_str("[display]\n");
    if enable_mobile {
        content.push_str("window/size/viewport_width=720\n");
        content.push_str("window/size/viewport_height=1280\n");
        content.push_str("window/stretch/mode=\"canvas_items\"\n");
        content.push_str("window/handheld/orientation=1\n");
        content.push_str("window/stretch/aspect=\"keep\"\n\n");
    } else {
        content.push_str("window/size/viewport_width=1280\n");
        content.push_str("window/size/viewport_height=720\n");
        content.push_str("window/stretch/mode=\"canvas_items\"\n\n");
    }

    content.push_str("[rendering]\n");
    if !template.godot.rendering.is_empty() {
        content.push_str(&format!("renderer/rendering_method=\"{}\"\n", template.godot.rendering));
    } else {
        content.push_str("renderer/rendering_method=\"forward_plus\"\n");
    }

    content
}

fn collect_autoloads_from_config(config: &TemplateProjectConfig) -> Vec<(String, String)> {
    let mut result = Vec::new();
    if let serde_json::Value::Object(map) = &config.autoloads {
        for (key, value) in map {
            if let serde_json::Value::String(path) = value {
                result.push((key.clone(), path.clone()));
            }
        }
    }
    result
}

fn collect_input_mappings_from_config(config: &TemplateProjectConfig) -> Vec<(String, String)> {
    let mut result = Vec::new();
    if let serde_json::Value::Object(map) = &config.input_mappings {
        for (key, value) in map {
            let mapping = json_input_to_godot_format(key, value);
            result.push((key.clone(), mapping));
        }
    }
    result
}

fn json_input_to_godot_format(_action: &str, value: &serde_json::Value) -> String {
    let deadzone = value.get("deadzone").and_then(|v| v.as_f64()).unwrap_or(0.5);
    let events = value.get("events").and_then(|v| v.as_array()).cloned().unwrap_or_default();

    let mut event_strs = Vec::new();
    for ev in &events {
        let ev_type = ev.get("type").and_then(|v| v.as_str()).unwrap_or("InputEventKey");
        let keycode = ev.get("keycode").and_then(|v| v.as_u64()).unwrap_or(0);
        match ev_type {
            "InputEventKey" => {
                event_strs.push(format!(
                    "Object(InputEventKey,\"resource_local_to_scene\":false,\"resource_name\":\"\",\"device\":-1,\"window_id\":0,\"alt_pressed\":false,\"shift_pressed\":false,\"ctrl_pressed\":false,\"meta_pressed\":false,\"pressed\":false,\"keycode\":{},\"physical_keycode\":0,\"key_label\":0,\"unicode\":0,\"location\":0,\"echo\":false,\"script\":null)",
                    keycode
                ));
            }
            "InputEventScreenTouch" => {
                let index = ev.get("index").and_then(|v| v.as_u64()).unwrap_or(0);
                event_strs.push(format!(
                    "Object(InputEventScreenTouch,\"resource_local_to_scene\":false,\"resource_name\":\"\",\"device\":-1,\"index\":{},\"pressed\":false,\"canceled\":false,\"position\":Vector2(0, 0),\"double_click\":false,\"script\":null)",
                    index
                ));
            }
            "InputEventScreenDrag" => {
                let index = ev.get("index").and_then(|v| v.as_u64()).unwrap_or(0);
                event_strs.push(format!(
                    "Object(InputEventScreenDrag,\"resource_local_to_scene\":false,\"resource_name\":\"\",\"device\":-1,\"index\":{},\"position\":Vector2(0, 0),\"relative\":Vector2(0, 0),\"velocity\":Vector2(0, 0),\"script\":null)",
                    index
                ));
            }
            _ => {}
        }
    }

    format!("\"deadzone\": {},\n\"events\": [{}]", deadzone, event_strs.join(", "))
}

fn collect_layer_names_from_config(config: &TemplateProjectConfig) -> String {
    let mut result = String::new();
    if let serde_json::Value::Object(map) = &config.layer_names {
        for (section, value) in map {
            if let serde_json::Value::Array(layers) = value {
                for (i, layer) in layers.iter().enumerate() {
                    if let serde_json::Value::String(name) = layer {
                        result.push_str(&format!("{}_physics/layer_{}=\"{}\"\n", section, i + 1, name));
                    }
                }
            }
        }
    }
    result
}

fn create_directory_structure(project_dir: &Path, template: &Template) -> Result<Vec<String>, String> {
    let mut created = Vec::new();
    for dir in &template.directories {
        let dir_path = project_dir.join(&dir.path);
        if !dir_path.exists() {
            fs::create_dir_all(&dir_path)
                .map_err(|e| format!("创建目录 {} 失败: {}", dir.path, e))?;
            created.push(dir.path.clone());
        }
    }
    Ok(created)
}

fn generate_export_presets_cfg(template: &Template) -> String {
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

    for (i, preset) in template.export_presets.iter().enumerate() {
        content.push_str(&format!("[preset.{}.options]\n\n", i));
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
            "android" => {
                content.push_str("custom_template/debug=\"\"\n");
                content.push_str("custom_template/release=\"\"\n");
                content.push_str("gradle_build/gradle_build_dir=\"\"\n");
                content.push_str("gradle_build/gradle_build_dir.editable=false\n");
                content.push_str("architectures/armeabi-v7a=false\n");
                content.push_str("architectures/arm64-v8a=true\n");
                content.push_str("architectures/x86=false\n");
                content.push_str("architectures/x86_64=false\n");
                content.push_str("version/code=1\n");
                content.push_str("version/name=\"1.0\"\n");
                content.push_str("package/unique_name=\"com.example.game\"\n");
                content.push_str("package/name=\"\"\n");
                content.push_str("package/signing/debug_keystore=\"\"\n");
            }
            "ios" => {
                content.push_str("custom_template/debug=\"\"\n");
                content.push_str("custom_template/release=\"\"\n");
                content.push_str("application/bundle_identifier=\"\"\n");
                content.push_str("application/icon=\"\"\n");
                content.push_str("application/signature=\"\"\n");
                content.push_str("application/short_version=\"1.0\"\n");
                content.push_str("application/version=\"1.0\"\n");
                content.push_str("application/copyright=\"\"\n");
            }
            _ => {}
        }
        content.push('\n');
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
    enable_mobile_support: Option<bool>,
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

    let enable_mobile = enable_mobile_support.unwrap_or(false);

    let project_godot_content = generate_project_godot(&template, enable_mobile);
    fs::write(project_dir.join("project.godot"), project_godot_content)
        .map_err(|e| format!("写入 project.godot 失败: {}", e))?;

    let created_dirs = create_directory_structure(&project_dir, &template)?;

    scaffold_template_framework(&app, &project_dir, &template)?;

    if enable_mobile {
        let _ = app.emit("template-instantiation-progress", TemplateInstantiationProgress {
            template_id: template.template_id.clone(),
            stage: "applying_module".to_string(),
            progress: 0.13,
            message: "正在应用移动端支持模块...".to_string(),
            detail: String::new(),
        });
        apply_mobile_support(&app, &project_dir)?;
    }

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
        description: "空白 Godot 4 项目，预装 Phantom Camera 和 Gut 测试框架，适合大多数2D/3D项目快速起步".to_string(),
        author: "Godot Harbor".to_string(),
        category: TemplateCategory::Blank,
        tags: vec!["blank".to_string(), "recommended".to_string(), "2d".to_string(), "3d".to_string()],
        icon_url: String::new(),
        preview_images: Vec::new(),
        godot: TemplateGodotConfig {
            version: "4.6".to_string(),
            mono: false,
            rendering: String::new(),
        },
        plugins: vec![
            TemplatePlugin { name: "phantom-camera".to_string(), version: "0.11".to_string(), source: TemplatePluginSource::Git, url: "https://github.com/ramokz/phantom-camera.git".to_string(), git_ref: String::new(), mount: "copy".to_string(), subdirectory: String::new() },
            TemplatePlugin { name: "gut".to_string(), version: "9.2.0".to_string(), source: TemplatePluginSource::Git, url: "https://github.com/bitwes/Gut.git".to_string(), git_ref: String::new(), mount: "copy".to_string(), subdirectory: String::new() },
        ],
        directories: vec![
            TemplateDirectory { path: "scenes".to_string(), description: "场景文件".to_string() },
            TemplateDirectory { path: "scripts".to_string(), description: "脚本文件".to_string() },
            TemplateDirectory { path: "scripts/autoload".to_string(), description: "全局自动加载脚本".to_string() },
            TemplateDirectory { path: "test".to_string(), description: "单元测试".to_string() },
            TemplateDirectory { path: "assets/sprites".to_string(), description: "精灵图".to_string() },
            TemplateDirectory { path: "assets/audio".to_string(), description: "音频文件".to_string() },
            TemplateDirectory { path: "assets/fonts".to_string(), description: "字体文件".to_string() },
            TemplateDirectory { path: "assets/shaders".to_string(), description: "着色器".to_string() },
        ],
        export_presets: vec![
            TemplateExportPreset { platform: "windows".to_string(), name: "Windows Desktop".to_string(), config: serde_json::Value::Null },
        ],
        project_config: TemplateProjectConfig {
            input_mappings: serde_json::json!({
                "move_left": { "deadzone": 0.5, "events": [{"type": "InputEventKey", "keycode": 65}] },
                "move_right": { "deadzone": 0.5, "events": [{"type": "InputEventKey", "keycode": 68}] },
                "move_up": { "deadzone": 0.5, "events": [{"type": "InputEventKey", "keycode": 87}] },
                "move_down": { "deadzone": 0.5, "events": [{"type": "InputEventKey", "keycode": 83}] },
                "ui_accept": { "deadzone": 0.5, "events": [{"type": "InputEventKey", "keycode": 4194309}] },
                "ui_cancel": { "deadzone": 0.5, "events": [{"type": "InputEventKey", "keycode": 4194305}] }
            }),
            layer_names: serde_json::json!({
                "2d_physics": ["player", "enemy", "environment", "pickup"],
                "2d_render": ["background", "foreground", "ui"]
            }),
            autoloads: serde_json::json!({}),
            project_settings: serde_json::json!({}),
        },
        is_builtin: true,
        source_url: String::new(),
        version: "1.1.0".to_string(),
        created_at: chrono::Utc::now(),
        updated_at: None,
    };

    let platformer_template = Template {
        template_id: "builtin-2d-platformer".to_string(),
        name: "2D 平台起步包".to_string(),
        description: "2D 平台游戏起步模板，包含 Phantom Camera、输入映射、粒子系统和基础场景组织，适合横版跳跃类游戏".to_string(),
        author: "Godot Harbor".to_string(),
        category: TemplateCategory::Starter2D,
        tags: vec!["2d".to_string(), "platformer".to_string(), "starter".to_string()],
        icon_url: String::new(),
        preview_images: Vec::new(),
        godot: TemplateGodotConfig {
            version: "4.6".to_string(),
            mono: false,
            rendering: "compatible".to_string(),
        },
        plugins: vec![
            TemplatePlugin { name: "phantom-camera".to_string(), version: "0.11".to_string(), source: TemplatePluginSource::Git, url: "https://github.com/ramokz/phantom-camera.git".to_string(), git_ref: String::new(), mount: "copy".to_string(), subdirectory: String::new() },
            TemplatePlugin { name: "godot-states".to_string(), version: "2.0.2".to_string(), source: TemplatePluginSource::Git, url: "https://github.com/quitbug/godot-state-machines.git".to_string(), git_ref: String::new(), mount: "copy".to_string(), subdirectory: String::new() },
        ],
        directories: vec![
            TemplateDirectory { path: "scenes/levels".to_string(), description: "关卡场景".to_string() },
            TemplateDirectory { path: "scenes/player".to_string(), description: "玩家场景".to_string() },
            TemplateDirectory { path: "scenes/enemies".to_string(), description: "敌人场景".to_string() },
            TemplateDirectory { path: "scenes/ui".to_string(), description: "UI 场景".to_string() },
            TemplateDirectory { path: "scenes/particles".to_string(), description: "粒子效果场景".to_string() },
            TemplateDirectory { path: "scripts/player".to_string(), description: "玩家脚本".to_string() },
            TemplateDirectory { path: "scripts/player/states".to_string(), description: "玩家状态机".to_string() },
            TemplateDirectory { path: "scripts/enemies".to_string(), description: "敌人脚本".to_string() },
            TemplateDirectory { path: "scripts/enemies/states".to_string(), description: "敌人状态机".to_string() },
            TemplateDirectory { path: "scripts/autoload".to_string(), description: "全局管理器".to_string() },
            TemplateDirectory { path: "assets/sprites/player".to_string(), description: "玩家精灵".to_string() },
            TemplateDirectory { path: "assets/sprites/enemies".to_string(), description: "敌人精灵".to_string() },
            TemplateDirectory { path: "assets/sprites/tilesets".to_string(), description: "瓦片集".to_string() },
            TemplateDirectory { path: "assets/sprites/vfx".to_string(), description: "特效精灵".to_string() },
            TemplateDirectory { path: "assets/audio/sfx".to_string(), description: "音效".to_string() },
            TemplateDirectory { path: "assets/audio/music".to_string(), description: "背景音乐".to_string() },
        ],
        export_presets: vec![
            TemplateExportPreset { platform: "windows".to_string(), name: "Windows Desktop".to_string(), config: serde_json::Value::Null },
            TemplateExportPreset { platform: "web".to_string(), name: "HTML5".to_string(), config: serde_json::Value::Null },
        ],
        project_config: TemplateProjectConfig {
            input_mappings: serde_json::json!({
                "move_left": { "deadzone": 0.5, "events": [{"type": "InputEventKey", "keycode": 65}, {"type": "InputEventJoypadButton", "button": 14}] },
                "move_right": { "deadzone": 0.5, "events": [{"type": "InputEventKey", "keycode": 68}, {"type": "InputEventJoypadButton", "button": 15}] },
                "jump": { "deadzone": 0.5, "events": [{"type": "InputEventKey", "keycode": 87}, {"type": "InputEventKey", "keycode": 4194320}, {"type": "InputEventJoypadButton", "button": 0}] },
                "attack": { "deadzone": 0.5, "events": [{"type": "InputEventKey", "keycode": 74}, {"type": "InputEventJoypadButton", "button": 2}] },
                "dash": { "deadzone": 0.5, "events": [{"type": "InputEventKey", "keycode": 4194324}, {"type": "InputEventJoypadButton", "button": 1}] },
                "ui_accept": { "deadzone": 0.5, "events": [{"type": "InputEventKey", "keycode": 4194309}] },
                "ui_cancel": { "deadzone": 0.5, "events": [{"type": "InputEventKey", "keycode": 4194305}] }
            }),
            layer_names: serde_json::json!({
                "2d_physics": ["player", "enemy", "environment", "pickup", "hazard", "platform"],
                "2d_render": ["background", "midground", "foreground", "ui"]
            }),
            autoloads: serde_json::json!({
                "GameManager": "res://scripts/autoload/game_manager.gd",
                "AudioManager": "res://scripts/autoload/audio_manager.gd"
            }),
            project_settings: serde_json::json!({
                "physics/common/physics_fps": 60
            }),
        },
        is_builtin: true,
        source_url: String::new(),
        version: "1.1.0".to_string(),
        created_at: chrono::Utc::now(),
        updated_at: None,
    };

    let rpg_template = Template {
        template_id: "builtin-2d-rpg".to_string(),
        name: "2D RPG 起步包".to_string(),
        description: "2D RPG 游戏起步模板，包含 Dialogic 对话系统、Phantom Camera 和状态机，适合叙事驱动的RPG项目".to_string(),
        author: "Godot Harbor".to_string(),
        category: TemplateCategory::RPG,
        tags: vec!["2d".to_string(), "rpg".to_string(), "dialogue".to_string(), "starter".to_string()],
        icon_url: String::new(),
        preview_images: Vec::new(),
        godot: TemplateGodotConfig {
            version: "4.6".to_string(),
            mono: false,
            rendering: "compatible".to_string(),
        },
        plugins: vec![
            TemplatePlugin { name: "phantom-camera".to_string(), version: "0.11".to_string(), source: TemplatePluginSource::Git, url: "https://github.com/ramokz/phantom-camera.git".to_string(), git_ref: String::new(), mount: "copy".to_string(), subdirectory: String::new() },
            TemplatePlugin { name: "dialogic".to_string(), version: "2.0".to_string(), source: TemplatePluginSource::Git, url: "https://github.com/dialogic-godot/dialogic.git".to_string(), git_ref: "main".to_string(), mount: "copy".to_string(), subdirectory: String::new() },
            TemplatePlugin { name: "godot-states".to_string(), version: "2.0.2".to_string(), source: TemplatePluginSource::Git, url: "https://github.com/quitbug/godot-state-machines.git".to_string(), git_ref: String::new(), mount: "copy".to_string(), subdirectory: String::new() },
        ],
        directories: vec![
            TemplateDirectory { path: "scenes/maps".to_string(), description: "地图场景".to_string() },
            TemplateDirectory { path: "scenes/characters".to_string(), description: "角色场景".to_string() },
            TemplateDirectory { path: "scenes/ui".to_string(), description: "UI 场景".to_string() },
            TemplateDirectory { path: "scenes/ui/menus".to_string(), description: "菜单场景".to_string() },
            TemplateDirectory { path: "scenes/ui/hud".to_string(), description: "HUD 场景".to_string() },
            TemplateDirectory { path: "scenes/dialogue".to_string(), description: "对话场景".to_string() },
            TemplateDirectory { path: "scenes/cutscenes".to_string(), description: "过场动画".to_string() },
            TemplateDirectory { path: "scripts/characters".to_string(), description: "角色脚本".to_string() },
            TemplateDirectory { path: "scripts/characters/states".to_string(), description: "角色状态机".to_string() },
            TemplateDirectory { path: "scripts/items".to_string(), description: "物品系统".to_string() },
            TemplateDirectory { path: "scripts/quests".to_string(), description: "任务系统".to_string() },
            TemplateDirectory { path: "scripts/autoload".to_string(), description: "全局管理器".to_string() },
            TemplateDirectory { path: "assets/sprites/characters".to_string(), description: "角色精灵".to_string() },
            TemplateDirectory { path: "assets/sprites/tilesets".to_string(), description: "瓦片集".to_string() },
            TemplateDirectory { path: "assets/portraits".to_string(), description: "角色立绘".to_string() },
            TemplateDirectory { path: "assets/audio/bgm".to_string(), description: "背景音乐".to_string() },
            TemplateDirectory { path: "assets/audio/sfx".to_string(), description: "音效".to_string() },
            TemplateDirectory { path: "assets/dialogue".to_string(), description: "对话资源".to_string() },
        ],
        export_presets: vec![
            TemplateExportPreset { platform: "windows".to_string(), name: "Windows Desktop".to_string(), config: serde_json::Value::Null },
            TemplateExportPreset { platform: "web".to_string(), name: "HTML5".to_string(), config: serde_json::Value::Null },
        ],
        project_config: TemplateProjectConfig {
            input_mappings: serde_json::json!({
                "move_left": { "deadzone": 0.5, "events": [{"type": "InputEventKey", "keycode": 65}] },
                "move_right": { "deadzone": 0.5, "events": [{"type": "InputEventKey", "keycode": 68}] },
                "move_up": { "deadzone": 0.5, "events": [{"type": "InputEventKey", "keycode": 87}] },
                "move_down": { "deadzone": 0.5, "events": [{"type": "InputEventKey", "keycode": 83}] },
                "interact": { "deadzone": 0.5, "events": [{"type": "InputEventKey", "keycode": 69}] },
                "menu": { "deadzone": 0.5, "events": [{"type": "InputEventKey", "keycode": 4194305}] },
                "ui_accept": { "deadzone": 0.5, "events": [{"type": "InputEventKey", "keycode": 4194309}] },
                "ui_cancel": { "deadzone": 0.5, "events": [{"type": "InputEventKey", "keycode": 4194305}] }
            }),
            layer_names: serde_json::json!({
                "2d_physics": ["player", "npc", "environment", "pickup", "trigger"],
                "2d_render": ["background", "midground", "foreground", "ui"]
            }),
            autoloads: serde_json::json!({
                "GameManager": "res://scripts/autoload/game_manager.gd",
                "QuestManager": "res://scripts/autoload/quest_manager.gd",
                "AudioManager": "res://scripts/autoload/audio_manager.gd"
            }),
            project_settings: serde_json::json!({}),
        },
        is_builtin: true,
        source_url: String::new(),
        version: "1.1.0".to_string(),
        created_at: chrono::Utc::now(),
        updated_at: None,
    };

    let starter_3d_template = Template {
        template_id: "builtin-3d-starter".to_string(),
        name: "3D 起步包".to_string(),
        description: "3D 游戏起步模板，包含 Phantom Camera 3D、基础3D场景组织、灯光和相机设置，适合3D游戏快速起步".to_string(),
        author: "Godot Harbor".to_string(),
        category: TemplateCategory::Starter3D,
        tags: vec!["3d".to_string(), "starter".to_string()],
        icon_url: String::new(),
        preview_images: Vec::new(),
        godot: TemplateGodotConfig {
            version: "4.6".to_string(),
            mono: false,
            rendering: "forward_plus".to_string(),
        },
        plugins: vec![
            TemplatePlugin { name: "phantom-camera".to_string(), version: "0.11".to_string(), source: TemplatePluginSource::Git, url: "https://github.com/ramokz/phantom-camera.git".to_string(), git_ref: String::new(), mount: "copy".to_string(), subdirectory: String::new() },
            TemplatePlugin { name: "godot-states".to_string(), version: "2.0.2".to_string(), source: TemplatePluginSource::Git, url: "https://github.com/quitbug/godot-state-machines.git".to_string(), git_ref: String::new(), mount: "copy".to_string(), subdirectory: String::new() },
        ],
        directories: vec![
            TemplateDirectory { path: "scenes/levels".to_string(), description: "关卡场景".to_string() },
            TemplateDirectory { path: "scenes/player".to_string(), description: "玩家场景".to_string() },
            TemplateDirectory { path: "scenes/enemies".to_string(), description: "敌人场景".to_string() },
            TemplateDirectory { path: "scenes/ui".to_string(), description: "UI 场景".to_string() },
            TemplateDirectory { path: "scenes/environment".to_string(), description: "环境场景".to_string() },
            TemplateDirectory { path: "scripts/player".to_string(), description: "玩家脚本".to_string() },
            TemplateDirectory { path: "scripts/player/states".to_string(), description: "玩家状态机".to_string() },
            TemplateDirectory { path: "scripts/enemies".to_string(), description: "敌人脚本".to_string() },
            TemplateDirectory { path: "scripts/autoload".to_string(), description: "全局管理器".to_string() },
            TemplateDirectory { path: "scripts/camera".to_string(), description: "相机控制".to_string() },
            TemplateDirectory { path: "assets/models".to_string(), description: "3D 模型".to_string() },
            TemplateDirectory { path: "assets/models/characters".to_string(), description: "角色模型".to_string() },
            TemplateDirectory { path: "assets/models/environment".to_string(), description: "环境模型".to_string() },
            TemplateDirectory { path: "assets/textures".to_string(), description: "纹理贴图".to_string() },
            TemplateDirectory { path: "assets/materials".to_string(), description: "材质".to_string() },
            TemplateDirectory { path: "assets/audio/sfx".to_string(), description: "音效".to_string() },
            TemplateDirectory { path: "assets/audio/music".to_string(), description: "背景音乐".to_string() },
            TemplateDirectory { path: "assets/shaders".to_string(), description: "着色器".to_string() },
        ],
        export_presets: vec![
            TemplateExportPreset { platform: "windows".to_string(), name: "Windows Desktop".to_string(), config: serde_json::Value::Null },
        ],
        project_config: TemplateProjectConfig {
            input_mappings: serde_json::json!({
                "move_left": { "deadzone": 0.5, "events": [{"type": "InputEventKey", "keycode": 65}, {"type": "InputEventJoypadButton", "button": 14}] },
                "move_right": { "deadzone": 0.5, "events": [{"type": "InputEventKey", "keycode": 68}, {"type": "InputEventJoypadButton", "button": 15}] },
                "move_forward": { "deadzone": 0.5, "events": [{"type": "InputEventKey", "keycode": 87}, {"type": "InputEventJoypadButton", "button": 12}] },
                "move_backward": { "deadzone": 0.5, "events": [{"type": "InputEventKey", "keycode": 83}, {"type": "InputEventJoypadButton", "button": 13}] },
                "jump": { "deadzone": 0.5, "events": [{"type": "InputEventKey", "keycode": 4194320}, {"type": "InputEventJoypadButton", "button": 0}] },
                "interact": { "deadzone": 0.5, "events": [{"type": "InputEventKey", "keycode": 69}, {"type": "InputEventJoypadButton", "button": 2}] },
                "camera_up": { "deadzone": 0.5, "events": [{"type": "InputEventKey", "keycode": 4194320}] },
                "camera_down": { "deadzone": 0.5, "events": [{"type": "InputEventKey", "keycode": 4194322}] },
                "ui_accept": { "deadzone": 0.5, "events": [{"type": "InputEventKey", "keycode": 4194309}] },
                "ui_cancel": { "deadzone": 0.5, "events": [{"type": "InputEventKey", "keycode": 4194305}] }
            }),
            layer_names: serde_json::json!({
                "3d_physics": ["player", "enemy", "environment", "pickup", "hazard"],
                "3d_render": ["background", "environment", "characters", "foreground", "ui"]
            }),
            autoloads: serde_json::json!({
                "GameManager": "res://scripts/autoload/game_manager.gd",
                "AudioManager": "res://scripts/autoload/audio_manager.gd"
            }),
            project_settings: serde_json::json!({
                "rendering/renderer/rendering_method": "forward_plus"
            }),
        },
        is_builtin: true,
        source_url: String::new(),
        version: "1.0.0".to_string(),
        created_at: chrono::Utc::now(),
        updated_at: None,
    };

    let multiplayer_template = Template {
        template_id: "builtin-multiplayer".to_string(),
        name: "多人游戏起步包".to_string(),
        description: "多人联机游戏起步模板，包含 SceneMultiplayer 配置、网络同步架构和聊天系统，适合局域网/在线多人游戏".to_string(),
        author: "Godot Harbor".to_string(),
        category: TemplateCategory::Multiplayer,
        tags: vec!["multiplayer".to_string(), "networking".to_string(), "online".to_string(), "starter".to_string()],
        icon_url: String::new(),
        preview_images: Vec::new(),
        godot: TemplateGodotConfig {
            version: "4.6".to_string(),
            mono: false,
            rendering: "compatible".to_string(),
        },
        plugins: vec![
            TemplatePlugin { name: "phantom-camera".to_string(), version: "0.11".to_string(), source: TemplatePluginSource::Git, url: "https://github.com/ramokz/phantom-camera.git".to_string(), git_ref: String::new(), mount: "copy".to_string(), subdirectory: String::new() },
        ],
        directories: vec![
            TemplateDirectory { path: "scenes/lobby".to_string(), description: "大厅场景".to_string() },
            TemplateDirectory { path: "scenes/game".to_string(), description: "游戏场景".to_string() },
            TemplateDirectory { path: "scenes/ui".to_string(), description: "UI 场景".to_string() },
            TemplateDirectory { path: "scenes/ui/chat".to_string(), description: "聊天UI".to_string() },
            TemplateDirectory { path: "scripts/network".to_string(), description: "网络管理".to_string() },
            TemplateDirectory { path: "scripts/network/rpc".to_string(), description: "RPC 调用".to_string() },
            TemplateDirectory { path: "scripts/network/sync".to_string(), description: "状态同步".to_string() },
            TemplateDirectory { path: "scripts/player".to_string(), description: "玩家脚本".to_string() },
            TemplateDirectory { path: "scripts/autoload".to_string(), description: "全局管理器".to_string() },
            TemplateDirectory { path: "assets/sprites".to_string(), description: "精灵图".to_string() },
            TemplateDirectory { path: "assets/audio/sfx".to_string(), description: "音效".to_string() },
            TemplateDirectory { path: "assets/audio/music".to_string(), description: "背景音乐".to_string() },
        ],
        export_presets: vec![
            TemplateExportPreset { platform: "windows".to_string(), name: "Windows Desktop".to_string(), config: serde_json::Value::Null },
            TemplateExportPreset { platform: "web".to_string(), name: "HTML5".to_string(), config: serde_json::Value::Null },
        ],
        project_config: TemplateProjectConfig {
            input_mappings: serde_json::json!({
                "move_left": { "deadzone": 0.5, "events": [{"type": "InputEventKey", "keycode": 65}, {"type": "InputEventJoypadButton", "button": 14}] },
                "move_right": { "deadzone": 0.5, "events": [{"type": "InputEventKey", "keycode": 68}, {"type": "InputEventJoypadButton", "button": 15}] },
                "move_up": { "deadzone": 0.5, "events": [{"type": "InputEventKey", "keycode": 87}] },
                "move_down": { "deadzone": 0.5, "events": [{"type": "InputEventKey", "keycode": 83}] },
                "chat": { "deadzone": 0.5, "events": [{"type": "InputEventKey", "keycode": 4194306}] },
                "ui_accept": { "deadzone": 0.5, "events": [{"type": "InputEventKey", "keycode": 4194309}] },
                "ui_cancel": { "deadzone": 0.5, "events": [{"type": "InputEventKey", "keycode": 4194305}] }
            }),
            layer_names: serde_json::json!({
                "2d_physics": ["player", "other_player", "environment", "pickup", "hazard"],
                "2d_render": ["background", "foreground", "ui"]
            }),
            autoloads: serde_json::json!({
                "NetworkManager": "res://scripts/autoload/network_manager.gd",
                "GameManager": "res://scripts/autoload/game_manager.gd",
                "AudioManager": "res://scripts/autoload/audio_manager.gd"
            }),
            project_settings: serde_json::json!({
                "network/limits/max_packet_size": 65536
            }),
        },
        is_builtin: true,
        source_url: String::new(),
        version: "1.0.0".to_string(),
        created_at: chrono::Utc::now(),
        updated_at: None,
    };

    for template in [blank_template, platformer_template, rpg_template, starter_3d_template, multiplayer_template] {
        if !builtin_ids.contains(&template.template_id.as_str()) {
            let saved = save_hub_template(app.clone(), template)?;
            created.push(saved);
        }
    }

    Ok(created)
}
