use crate::commands::utils::{get_storage, get_data_dir};
use crate::models::*;
use crate::utils::create_http_client;
use tauri::{AppHandle, Emitter};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const GODOT_EXPORT_TEMPLATES_URL: &str = "https://downloads.tuxfamily.org/godotengine";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportTemplateInfo {
    pub version: String,
    pub mono: bool,
    pub installed: bool,
    pub path: Option<String>,
    pub file_size: Option<u64>,
}

#[tauri::command]
pub fn list_export_templates(app: AppHandle) -> Result<Vec<ExportTemplateInfo>, String> {
    let data_dir = get_data_dir(&app);
    let templates_dir = data_dir.join("export_templates");
    if !templates_dir.exists() {
        return Ok(Vec::new());
    }

    let engines: Vec<Engine> = get_storage(&app).load_or_default("engines.json");
    let mut result = Vec::new();

    let entries = fs::read_dir(&templates_dir)
        .map_err(|e| format!("读取导出模板目录失败: {}", e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let dir_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
        let parts: Vec<&str> = dir_name.splitn(2, '_').collect();
        let version = parts[0].to_string();
        let mono = parts.len() > 1 && parts[1] == "mono";

        let tpk_file = path.join("templates.tpz");
        let installed = tpk_file.exists();
        let file_size = if installed {
            fs::metadata(&tpk_file).ok().map(|m| m.len())
        } else {
            None
        };

        result.push(ExportTemplateInfo {
            version: version.clone(),
            mono,
            installed,
            path: if installed { Some(tpk_file.to_string_lossy().to_string()) } else { None },
            file_size,
        });
    }

    for engine in &engines {
        let _key = if engine.is_mono {
            format!("{}_mono", engine.version)
        } else {
            engine.version.clone()
        };
        if !result.iter().any(|t| t.version == engine.version && t.mono == engine.is_mono) {
            result.push(ExportTemplateInfo {
                version: engine.version.clone(),
                mono: engine.is_mono,
                installed: false,
                path: None,
                file_size: None,
            });
        }
    }

    result.sort_by(|a, b| {
        match (a.version.parse::<semver::Version>(), b.version.parse::<semver::Version>()) {
            (Ok(av), Ok(bv)) => bv.cmp(&av).then(a.mono.cmp(&b.mono)),
            _ => b.version.cmp(&a.version),
        }
    });

    Ok(result)
}

#[tauri::command]
pub async fn download_export_template(app: AppHandle, version: String, mono: bool) -> Result<String, String> {
    let _ = app.emit("export-template-download-progress", serde_json::json!({
        "version": &version,
        "stage": "downloading",
        "progress": 0.0,
        "message": format!("正在下载 Godot {} 导出模板...", version),
    }));

    let data_dir = get_data_dir(&app);
    let templates_dir = data_dir.join("export_templates");
    let version_dir = if mono {
        templates_dir.join(format!("{}_mono", version))
    } else {
        templates_dir.join(&version)
    };
    fs::create_dir_all(&version_dir)
        .map_err(|e| format!("创建导出模板目录失败: {}", e))?;

    let stable_version = version.trim_end_matches("-dev").trim_end_matches("-beta").trim_end_matches("-rc");
    let url = if mono {
        format!("{}/{}/mono/Godot_v{}_mono_export_templates.tpz", GODOT_EXPORT_TEMPLATES_URL, stable_version, version)
    } else {
        format!("{}/{}/Godot_v{}_export_templates.tpz", GODOT_EXPORT_TEMPLATES_URL, stable_version, version)
    };

    let client = create_http_client(Some(std::time::Duration::from_secs(300)))?;
    let mut resp = client.get(&url).send().await
        .map_err(|e| format!("下载导出模板失败: {}", e))?;

    if !resp.status().is_success() {
        let alt_url = if mono {
            format!("{}/{}/mono/Godot_v{}_stable_mono_export_templates.tpz", GODOT_EXPORT_TEMPLATES_URL, stable_version, stable_version)
        } else {
            format!("{}/{}/Godot_v{}_stable_export_templates.tpz", GODOT_EXPORT_TEMPLATES_URL, stable_version, stable_version)
        };
        resp = client.get(&alt_url).send().await
            .map_err(|e| format!("下载导出模板失败(备用URL): {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("下载导出模板失败: HTTP {}", resp.status()));
        }
    }

    let tpz_path = version_dir.join("templates.tpz");
    let mut file = fs::File::create(&tpz_path)
        .map_err(|e| format!("创建临时文件失败: {}", e))?;

    use std::io::Write;
    use futures::StreamExt;
    let mut stream = resp.bytes_stream();
    let mut _downloaded: u64 = 0;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("下载中断: {}", e))?;
        file.write_all(&chunk).map_err(|e| format!("写入文件失败: {}", e))?;
        _downloaded += chunk.len() as u64;
    }

    let _ = app.emit("export-template-download-progress", serde_json::json!({
        "version": &version,
        "stage": "complete",
        "progress": 1.0,
        "message": format!("Godot {} 导出模板下载完成", version),
    }));

    Ok(format!("导出模板 {} 下载完成", version))
}

#[tauri::command]
pub fn delete_export_template(app: AppHandle, version: String, mono: bool) -> Result<(), String> {
    let data_dir = get_data_dir(&app);
    let templates_dir = data_dir.join("export_templates");
    let version_dir = if mono {
        templates_dir.join(format!("{}_mono", version))
    } else {
        templates_dir.join(&version)
    };

    if version_dir.exists() {
        fs::remove_dir_all(&version_dir)
            .map_err(|e| format!("删除导出模板失败: {}", e))?;
    }

    Ok(())
}

#[tauri::command]
pub fn list_export_presets(app: AppHandle, project_id: String) -> Result<Vec<ExportPreset>, String> {
    let storage = get_storage(&app);
    let projects: Vec<Project> = storage.load_or_default("projects.json");
    let project = projects.iter().find(|p| p.project_id == project_id)
        .ok_or("项目不存在".to_string())?;

    let config = crate::harbor_config::read_harbor_config_from_project(&project.path)
        .map_err(|e| format!("读取 .harbor.yml 失败: {}", e))?;

    let mut presets = Vec::new();
    if let Some(config) = config {
        let config_upgraded = if config.version < 2 { config.upgrade_to_v2() } else { config };
        for ep in &config_upgraded.export_presets {
            let platform = match ep.platform.as_str() {
                "windows" => ExportPlatform::Windows,
                "macos" => ExportPlatform::MacOS,
                "linux" => ExportPlatform::Linux,
                "web" => ExportPlatform::Web,
                "android" => ExportPlatform::Android,
                "ios" => ExportPlatform::IOS,
                _ => ExportPlatform::Windows,
            };
            presets.push(ExportPreset {
                preset_id: uuid::Uuid::new_v4().to_string(),
                platform,
                name: ep.name.clone(),
                config: ep.config.clone(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            });
        }
    }

    Ok(presets)
}

#[tauri::command]
pub fn apply_export_preset(app: AppHandle, project_id: String, preset: ExportPreset) -> Result<(), String> {
    let storage = get_storage(&app);
    let projects: Vec<Project> = storage.load_or_default("projects.json");
    let project = projects.iter().find(|p| p.project_id == project_id)
        .ok_or("项目不存在".to_string())?;

    let export_cfg_path = Path::new(&project.path).join("export_presets.cfg");
    let platform_str = preset.platform.to_string();

    let mut content = if export_cfg_path.exists() {
        fs::read_to_string(&export_cfg_path)
            .map_err(|e| format!("读取 export_presets.cfg 失败: {}", e))?
    } else {
        "[preset.0]\nname=\"\"\nplatform=\"\"\nrunnable=true\n".to_string()
    };

    let preset_index = count_presets(&content);
    let new_preset = format!(
        "\n[preset.{}]\nname=\"{}\"\nplatform=\"{}\"\nrunnable=true\n",
        preset_index, preset.name, platform_str
    );

    if !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str(&new_preset);

    fs::write(&export_cfg_path, content)
        .map_err(|e| format!("写入 export_presets.cfg 失败: {}", e))?;

    Ok(())
}

fn count_presets(content: &str) -> usize {
    let re = regex::Regex::new(r#"\[preset\.(\d+)\]"#).unwrap();
    let max_idx = re.captures_iter(content)
        .filter_map(|c| c[1].parse::<usize>().ok())
        .max()
        .map_or(0, |m| m + 1);
    max_idx
}

#[tauri::command]
pub fn save_export_preset_to_harbor(app: AppHandle, project_id: String, platform: String, name: String, config: serde_json::Value) -> Result<(), String> {
    let storage = get_storage(&app);
    let projects: Vec<Project> = storage.load_or_default("projects.json");
    let project = projects.iter().find(|p| p.project_id == project_id)
        .ok_or("项目不存在".to_string())?;

    let harbor_config_path = crate::harbor_config::get_harbor_config_path(&project.path);
    if !harbor_config_path.exists() {
        return Err("项目缺少 .harbor.yml".to_string());
    }

    let config_existing = crate::harbor_config::read_harbor_config_from_project(&project.path)
        .map_err(|e| format!("读取 .harbor.yml 失败: {}", e))?
        .ok_or("读取 .harbor.yml 失败".to_string())?;

    let mut config_upgraded = if config_existing.version < 2 { config_existing.upgrade_to_v2() } else { config_existing };
    config_upgraded.export_presets.push(crate::harbor_config::HarborExportPreset {
        platform: platform.clone(),
        name: name.clone(),
        config,
    });

    crate::harbor_config::write_harbor_config_to_project(&project.path, &config_upgraded)
        .map_err(|e| format!("写入 .harbor.yml 失败: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn build_project(app: AppHandle, project_id: String, platform: ExportPlatform, preset_name: Option<String>) -> Result<BuildRecord, String> {
    let storage = get_storage(&app);
    let projects: Vec<Project> = storage.load_or_default("projects.json");
    let project = projects.iter().find(|p| p.project_id == project_id)
        .ok_or("项目不存在".to_string())?;

    let engines: Vec<Engine> = storage.load_or_default("engines.json");
    let engine = engines.iter().find(|e| {
        project.last_used_engine_id.as_ref().map_or(false, |id| &e.engine_id == id)
    }).or_else(|| engines.iter().find(|e| {
        let ev: Vec<&str> = e.version.split('.').collect();
        let pv: Vec<&str> = project.godot_version.split('.').collect();
        ev.len() >= 2 && pv.len() >= 2 && ev[0] == pv[0] && ev[1] == pv[1]
    })).ok_or("未找到匹配的引擎".to_string())?;

    let build_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now();

    let _ = app.emit("build-progress", serde_json::json!({
        "build_id": &build_id,
        "stage": "starting",
        "progress": 0.0,
        "message": format!("正在准备构建 {} ({} {})...", project.name, platform, engine.version),
    }));

    let engine_path = PathBuf::from(&engine.path);
    let godot_executable = if engine.is_mono {
        engine_path.join("GodotSharp")
    } else {
        engine_path.join("Godot")
    };

    let godot_bin = if cfg!(windows) {
        godot_executable.with_extension("exe")
    } else {
        godot_executable
    };

    if !godot_bin.exists() {
        let _ = app.emit("build-progress", serde_json::json!({
            "build_id": &build_id,
            "stage": "failed",
            "progress": 0.0,
            "message": format!("引擎可执行文件不存在: {}", godot_bin.display()),
        }));
        return Err(format!("引擎可执行文件不存在: {}", godot_bin.display()));
    }

    let data_dir = get_data_dir(&app);
    let templates_dir = data_dir.join("export_templates");
    let template_key = if engine.is_mono {
        format!("{}_mono", engine.version)
    } else {
        engine.version.clone()
    };
    let template_dir = templates_dir.join(&template_key);
    let tpk_file = template_dir.join("templates.tpz");
    if !tpk_file.exists() {
        let _ = app.emit("build-progress", serde_json::json!({
            "build_id": &build_id,
            "stage": "failed",
            "progress": 0.0,
            "message": format!("导出模板未安装: Godot {}，请先在\"导出模板\"页下载", engine.version),
        }));
        return Err(format!("导出模板未安装: Godot {}，请先在\"导出模板\"页下载", engine.version));
    }

    let output_dir = PathBuf::from(&project.path).join("builds").join(platform.to_string());
    fs::create_dir_all(&output_dir)
        .map_err(|e| format!("创建输出目录失败: {}", e))?;

    let _preset_flag = preset_name.as_ref().map_or(String::new(), |n| format!("--export-release=\"{}\"", n));
    let platform_flag = format!("--export-release {}", platform);

    let _ = app.emit("build-progress", serde_json::json!({
        "build_id": &build_id,
        "stage": "building",
        "progress": 0.3,
        "message": format!("正在构建 {} ({} {})...", project.name, platform, engine.version),
    }));

    let _project_godot = PathBuf::from(&project.path).join("project.godot");

    let output = tokio::process::Command::new(&godot_bin)
        .arg("--headless")
        .arg("--path").arg(&project.path)
        .arg(&platform_flag)
        .output()
        .await
        .map_err(|e| format!("执行构建命令失败: {}", e))?;

    let duration = (chrono::Utc::now() - now).num_seconds() as u64;
    let success = output.status.success();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    let (status, error_message) = if success {
        let _ = app.emit("build-progress", serde_json::json!({
            "build_id": &build_id,
            "stage": "complete",
            "progress": 1.0,
            "message": format!("构建完成: {} ({})", project.name, platform),
        }));
        (BuildStatus::Success, String::new())
    } else {
        let err = format!("{}\n{}", stdout, stderr);
        let _ = app.emit("build-progress", serde_json::json!({
            "build_id": &build_id,
            "stage": "failed",
            "progress": 1.0,
            "message": format!("构建失败: {}", &err[..err.len().min(200)]),
        }));
        (BuildStatus::Failed, err)
    };

    let record = BuildRecord {
        build_id: build_id.clone(),
        project_id: project.project_id.clone(),
        project_name: project.name.clone(),
        platform: platform.clone(),
        engine_version: engine.version.clone(),
        status,
        started_at: now,
        completed_at: Some(chrono::Utc::now()),
        output_path: output_dir.to_string_lossy().to_string(),
        error_message,
        duration_secs: duration,
    };

    let mut records: Vec<BuildRecord> = storage.load_or_default("build_records.json");
    records.push(record.clone());
    storage.save("build_records.json", &records)
        .map_err(|e| format!("保存构建记录失败: {}", e))?;

    Ok(record)
}

#[tauri::command]
pub fn get_build_records(app: AppHandle, project_id: Option<String>) -> Result<Vec<BuildRecord>, String> {
    let storage = get_storage(&app);
    let records: Vec<BuildRecord> = storage.load_or_default("build_records.json");
    match project_id {
        Some(pid) => Ok(records.into_iter().filter(|r| r.project_id == pid).collect()),
        None => Ok(records),
    }
}

#[tauri::command]
pub fn delete_build_record(app: AppHandle, build_id: String) -> Result<(), String> {
    let storage = get_storage(&app);
    let mut records: Vec<BuildRecord> = storage.load_or_default("build_records.json");
    records.retain(|r| r.build_id != build_id);
    storage.save("build_records.json", &records)
        .map_err(|e| format!("保存构建记录失败: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn generate_github_actions(app: AppHandle, project_id: String, platforms: Vec<String>, godot_version: String) -> Result<String, String> {
    let storage = get_storage(&app);
    let projects: Vec<Project> = storage.load_or_default("projects.json");
    let project = projects.iter().find(|p| p.project_id == project_id)
        .ok_or("项目不存在".to_string())?;

    let project_name = project.name.clone();
    let safe_name = project_name.replace(' ', "-").to_lowercase();

    let mut matrix_entries = Vec::new();
    for p in &platforms {
        matrix_entries.push(format!("          - platform: {}\n            artifact: {}-{}", p, safe_name, p));
    }

    let workflow = format!(r#"name: Build Godot Project

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]
  workflow_dispatch:

env:
  GODOT_VERSION: "{}"

jobs:
  build:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        include:
{}
    steps:
      - uses: actions/checkout@v4

      - name: Download Godot
        run: |
          wget -q https://downloads.tuxfamily.org/godotengine/${{{{ env.GODOT_VERSION }}}}/Godot_v${{{{ env.GODOT_VERSION }}}}_linux.x86_64.zip
          unzip Godot_v${{{{ env.GODOT_VERSION }}}}_linux.x86_64.zip
          chmod +x Godot_v${{{{ env.GODOT_VERSION }}}}_linux.x86_64

      - name: Download Export Templates
        run: |
          mkdir -p ~/.local/share/godot/export_templates/${{{{ env.GODOT_VERSION }}}}.stable
          wget -q https://downloads.tuxfamily.org/godotengine/${{{{ env.GODOT_VERSION }}}}/Godot_v${{{{ env.GODOT_VERSION }}}}_export_templates.tpz
          unzip Godot_v${{{{ env.GODOT_VERSION }}}}_export_templates.tpz -d ~/.local/share/godot/export_templates/${{{{ env.GODOT_VERSION }}}}.stable

      - name: Build Project
        run: |
          ./Godot_v${{{{ env.GODOT_VERSION }}}}_linux.x86_64 --headless --export-release ${{{{ matrix.platform }}}} ./build/${{{{ matrix.artifact }}}}

      - name: Upload Artifact
        uses: actions/upload-artifact@v4
        with:
          name: ${{{{ matrix.artifact }}}}
          path: ./build/${{{{ matrix.artifact }}}}
"#, godot_version, matrix_entries.join("\n"));

    Ok(workflow)
}

#[tauri::command]
pub fn generate_gitlab_ci(app: AppHandle, project_id: String, platforms: Vec<String>, godot_version: String) -> Result<String, String> {
    let storage = get_storage(&app);
    let projects: Vec<Project> = storage.load_or_default("projects.json");
    let project = projects.iter().find(|p| p.project_id == project_id)
        .ok_or("项目不存在".to_string())?;

    let project_name = project.name.clone();
    let safe_name = project_name.replace(' ', "-").to_lowercase();

    let mut build_jobs = String::new();
    for p in &platforms {
        let job_name = format!("build_{}", p);
        build_jobs.push_str(&format!(r#"
{job_name}:
  stage: build
  image: barichello/godot-ci:{godot_version}
  script:
    - mkdir -v -p build/{platform}
    - godot --headless --export-release "{platform_upper}" build/{safe_name}-{platform}
  artifacts:
    paths:
      - build/{safe_name}-{platform}
"#, job_name = job_name, godot_version = godot_version, platform = p, platform_upper = p.to_uppercase(), safe_name = safe_name));
    }

    let ci = format!(r#"image: barichello/godot-ci:{godot_version}

stages:
  - build

{build_jobs}
"#, godot_version = godot_version, build_jobs = build_jobs);

    Ok(ci)
}

#[tauri::command]
pub fn write_ci_config(app: AppHandle, project_id: String, provider: String, content: String) -> Result<(), String> {
    let storage = get_storage(&app);
    let projects: Vec<Project> = storage.load_or_default("projects.json");
    let project = projects.iter().find(|p| p.project_id == project_id)
        .ok_or("项目不存在".to_string())?;

    let project_path = Path::new(&project.path);

    match provider.as_str() {
        "github-actions" => {
            let workflows_dir = project_path.join(".github").join("workflows");
            fs::create_dir_all(&workflows_dir)
                .map_err(|e| format!("创建 .github/workflows 目录失败: {}", e))?;
            fs::write(workflows_dir.join("build.yml"), content)
                .map_err(|e| format!("写入 build.yml 失败: {}", e))?;
        }
        "gitlab-ci" => {
            fs::write(project_path.join(".gitlab-ci.yml"), content)
                .map_err(|e| format!("写入 .gitlab-ci.yml 失败: {}", e))?;
        }
        _ => return Err(format!("不支持的 CI 提供商: {}", provider)),
    }

    Ok(())
}

#[tauri::command]
pub fn get_builtin_export_presets() -> Vec<BuiltinExportPreset> {
    vec![
        BuiltinExportPreset {
            platform: "windows".to_string(),
            name: "Windows Desktop".to_string(),
            description: "Windows 桌面应用（.exe）".to_string(),
            config: serde_json::json!({
                "binary_format": "64",
                "texture_format": "s3tc_bptc",
            }),
        },
        BuiltinExportPreset {
            platform: "web".to_string(),
            name: "HTML5".to_string(),
            description: "Web 浏览器应用".to_string(),
            config: serde_json::json!({
                "texture_format": "s3tc_bptc",
                "html/window_size": "1280x720",
            }),
        },
        BuiltinExportPreset {
            platform: "linux".to_string(),
            name: "Linux/X11".to_string(),
            description: "Linux 桌面应用".to_string(),
            config: serde_json::json!({
                "binary_format": "64",
                "texture_format": "s3tc_bptc",
            }),
        },
        BuiltinExportPreset {
            platform: "macos".to_string(),
            name: "macOS".to_string(),
            description: "macOS 应用（.app）".to_string(),
            config: serde_json::json!({
                "texture_format": "s3tc_bptc",
            }),
        },
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuiltinExportPreset {
    pub platform: String,
    pub name: String,
    pub description: String,
    pub config: serde_json::Value,
}

#[tauri::command]
pub fn export_preset_to_json(preset: ExportPreset) -> Result<String, String> {
    serde_json::to_string_pretty(&preset)
        .map_err(|e| format!("序列化预设失败: {}", e))
}

#[tauri::command]
pub fn import_preset_from_json(json: String) -> Result<ExportPreset, String> {
    serde_json::from_str(&json)
        .map_err(|e| format!("解析预设失败: {}", e))
}
