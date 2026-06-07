use serde_json::{json, Value};
use crate::models::*;
use super::server::McpContext;

pub fn list_prompts() -> Value {
    json!({
        "prompts": [
            {
                "name": "diagnose_environment",
                "description": "Diagnose environment issues for a Godot project. Checks drift, missing plugins, engine compatibility.",
                "arguments": [
                    {
                        "name": "project_id",
                        "description": "The project ID to diagnose",
                        "required": true
                    }
                ]
            },
            {
                "name": "suggest_plugins",
                "description": "Suggest plugins based on project type and existing setup.",
                "arguments": [
                    {
                        "name": "project_id",
                        "description": "The project ID to analyze",
                        "required": true
                    }
                ]
            },
            {
                "name": "setup_ci_cd",
                "description": "Generate CI/CD configuration for a Godot project.",
                "arguments": [
                    {
                        "name": "project_id",
                        "description": "The project ID",
                        "required": true
                    },
                    {
                        "name": "provider",
                        "description": "CI provider (github-actions or gitlab-ci)",
                        "required": false
                    },
                    {
                        "name": "platforms",
                        "description": "Target platforms (comma-separated, e.g. windows,web)",
                        "required": false
                    }
                ]
            }
        ]
    })
}

pub fn get_prompt(ctx: &McpContext, params: &Value) -> Value {
    let prompt_name = params["name"].as_str().unwrap_or("");
    let arguments = &params["arguments"];

    match prompt_name {
        "diagnose_environment" => prompt_diagnose_environment(ctx, arguments),
        "suggest_plugins" => prompt_suggest_plugins(ctx, arguments),
        "setup_ci_cd" => prompt_setup_ci_cd(ctx, arguments),
        _ => json!({
            "description": "Unknown prompt",
            "messages": []
        }),
    }
}

fn prompt_diagnose_environment(ctx: &McpContext, arguments: &Value) -> Value {
    let project_id = arguments["project_id"].as_str().unwrap_or("");

    let projects: Vec<Project> = ctx.storage.load_or_default("projects.json");
    let project = match projects.iter().find(|p| p.project_id == project_id) {
        Some(p) => p,
        None => {
            return json!({
                "description": "Diagnose environment issues",
                "messages": [{
                    "role": "user",
                    "content": {"type": "text", "text": format!("Project {} not found. Please provide a valid project ID.", project_id)}
                }]
            });
        }
    };

    let engines: Vec<Engine> = ctx.storage.load_or_default("engines.json");
    let bindings: Vec<ProjectBinding> = ctx.storage.load_or_default("bindings.json");
    let plugins: Vec<Plugin> = ctx.storage.load_or_default("plugins.json");

    let project_bindings: Vec<&ProjectBinding> = bindings.iter()
        .filter(|b| b.project_id == project_id)
        .collect();

    let bound_plugins: Vec<String> = project_bindings.iter().filter_map(|b| {
        plugins.iter().find(|p| p.plugin_id == b.plugin_id).map(|p| {
            let version = p.versions.iter()
                .find(|v| v.version_id == b.version_id)
                .map(|v| v.version.clone())
                .unwrap_or_default();
            let health = match b.is_healthy {
                Some(true) => "✅",
                Some(false) => "❌ broken",
                None => "❓ unknown",
            };
            format!("{} v{} {}", p.name, version, health)
        })
    }).collect();

    let engine_info = project.last_used_engine_id.as_ref().and_then(|eid| {
        engines.iter().find(|e| &e.engine_id == eid).map(|e| format!("Godot {} (mono={})", e.version, e.is_mono))
    }).unwrap_or_else(|| "No engine bound".to_string());

    let config = crate::harbor_config::read_harbor_config_from_project(&project.path);
    let has_harbor_yml = config.is_ok() && config.as_ref().unwrap().is_some();

    let mut diagnosis = vec![
        format!("Project: {} ({})", project.name, project.path),
        format!("Godot version: {}", project.godot_version),
        format!("Engine: {}", engine_info),
        format!("Plugins: {}", if bound_plugins.is_empty() { "None".to_string() } else { bound_plugins.join(", ") }),
        format!(".harbor.yml: {}", if has_harbor_yml { "Present" } else { "Missing ⚠️" }),
    ];

    if let Ok(Some(cfg)) = config {
        let cfg_upgraded = if cfg.version < 2 { cfg.upgrade_to_v2() } else { cfg };
        if let Some(ref godot_cfg) = cfg_upgraded.godot {
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
                diagnosis.push(format!("⚠️ Declared engine {} not installed", godot_cfg.version));
            }
            for pc in &cfg_upgraded.plugins {
                let installed = project_bindings.iter().any(|b| {
                    plugins.iter().find(|p| p.plugin_id == b.plugin_id)
                        .map_or(false, |p| p.name.to_lowercase() == pc.name.to_lowercase())
                });
                if !installed {
                    diagnosis.push(format!("⚠️ Declared plugin {} v{} not installed", pc.name, pc.version));
                }
            }
        }
    }

    let prompt_text = format!(
        "Please diagnose the following Godot project environment and suggest fixes:\n\n{}\n\nAnalyze the environment state, identify any issues (missing plugins, engine version mismatches, missing .harbor.yml), and provide actionable recommendations.",
        diagnosis.join("\n")
    );

    json!({
        "description": format!("Diagnose environment for project {}", project.name),
        "messages": [{
            "role": "user",
            "content": {"type": "text", "text": prompt_text}
        }]
    })
}

fn prompt_suggest_plugins(ctx: &McpContext, arguments: &Value) -> Value {
    let project_id = arguments["project_id"].as_str().unwrap_or("");

    let projects: Vec<Project> = ctx.storage.load_or_default("projects.json");
    let project = match projects.iter().find(|p| p.project_id == project_id) {
        Some(p) => p,
        None => {
            return json!({
                "description": "Suggest plugins",
                "messages": [{
                    "role": "user",
                    "content": {"type": "text", "text": format!("Project {} not found.", project_id)}
                }]
            });
        }
    };

    let bindings: Vec<ProjectBinding> = ctx.storage.load_or_default("bindings.json");
    let plugins: Vec<Plugin> = ctx.storage.load_or_default("plugins.json");

    let project_bindings: Vec<&ProjectBinding> = bindings.iter()
        .filter(|b| b.project_id == project_id)
        .collect();

    let bound_plugins: Vec<String> = project_bindings.iter().filter_map(|b| {
        plugins.iter().find(|p| p.plugin_id == b.plugin_id).map(|p| {
            let version = p.versions.first().map(|v| v.version.clone()).unwrap_or_default();
            format!("{} v{} ({:?})", p.name, version, p.source.source_type)
        })
    }).collect();

    let available_plugins: Vec<String> = plugins.iter()
        .filter(|p| !project_bindings.iter().any(|b| b.plugin_id == p.plugin_id))
        .map(|p| {
            let version = p.versions.first().map(|v| v.version.clone()).unwrap_or_default();
            format!("{} v{} ({:?})", p.name, version, p.source.source_type)
        })
        .collect();

    let prompt_text = format!(
        "Please suggest plugins for this Godot project:\n\nProject: {} ({})\nGodot version: {}\nCurrently using: {}\nAvailable plugins not yet installed: {}\n\nBased on the project type and existing plugins, recommend plugins that would be useful. Consider common Godot development patterns (2D, 3D, UI, networking, etc.). You can also use the search_asset_library tool to find more plugins from the Godot Asset Library.",
        project.name,
        project.path,
        project.godot_version,
        if bound_plugins.is_empty() { "None".to_string() } else { bound_plugins.join(", ") },
        if available_plugins.is_empty() { "None".to_string() } else { available_plugins.join(", ") },
    );

    json!({
        "description": format!("Suggest plugins for project {}", project.name),
        "messages": [{
            "role": "user",
            "content": {"type": "text", "text": prompt_text}
        }]
    })
}

fn prompt_setup_ci_cd(ctx: &McpContext, arguments: &Value) -> Value {
    let project_id = arguments["project_id"].as_str().unwrap_or("");
    let provider = arguments["provider"].as_str().unwrap_or("github-actions");
    let platforms_str = arguments["platforms"].as_str().unwrap_or("windows,web");

    let projects: Vec<Project> = ctx.storage.load_or_default("projects.json");
    let project = match projects.iter().find(|p| p.project_id == project_id) {
        Some(p) => p,
        None => {
            return json!({
                "description": "Setup CI/CD",
                "messages": [{
                    "role": "user",
                    "content": {"type": "text", "text": format!("Project {} not found.", project_id)}
                }]
            });
        }
    };

    let engines: Vec<Engine> = ctx.storage.load_or_default("engines.json");
    let engine_version = project.last_used_engine_id.as_ref().and_then(|eid| {
        engines.iter().find(|e| &e.engine_id == eid).map(|e| e.version.clone())
    }).unwrap_or_else(|| project.godot_version.clone());

    let platforms: Vec<&str> = platforms_str.split(',').map(|s| s.trim()).collect();

    let prompt_text = format!(
        "Please generate a CI/CD configuration for this Godot project:\n\nProject: {} ({})\nEngine version: {}\nCI provider: {}\nTarget platforms: {}\n\nGenerate a complete CI/CD configuration file that:\n1. Downloads the correct Godot engine version\n2. Installs export templates\n3. Builds the project for each target platform\n4. Uploads build artifacts\n\nProvide the configuration in the appropriate format for the chosen CI provider.",
        project.name,
        project.path,
        engine_version,
        provider,
        platforms.join(", "),
    );

    json!({
        "description": format!("Setup CI/CD for project {}", project.name),
        "messages": [{
            "role": "user",
            "content": {"type": "text", "text": prompt_text}
        }]
    })
}
