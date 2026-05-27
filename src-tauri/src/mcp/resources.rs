use serde_json::{json, Value};
use crate::models::*;
use crate::harbor_config;
use super::server::McpContext;

pub fn list_resources() -> Value {
    json!({
        "resources": [
            {
                "uri": "harbor://projects",
                "name": "Projects",
                "description": "List all managed Godot projects",
                "mimeType": "application/json"
            },
            {
                "uri": "harbor://project/{id}/environment",
                "name": "Project Environment",
                "description": "Current environment state of a project (installed plugins, engine version)",
                "mimeType": "application/json"
            },
            {
                "uri": "harbor://project/{id}/expected",
                "name": "Project Expected Environment",
                "description": "Expected environment from .harbor.yml",
                "mimeType": "application/json"
            },
            {
                "uri": "harbor://project/{id}/drift",
                "name": "Project Drift Report",
                "description": "Environment drift report for a project",
                "mimeType": "application/json"
            },
            {
                "uri": "harbor://plugins/available",
                "name": "Available Plugins",
                "description": "List all available plugins in the library",
                "mimeType": "application/json"
            },
            {
                "uri": "harbor://engines/installed",
                "name": "Installed Engines",
                "description": "List all installed Godot engine versions",
                "mimeType": "application/json"
            },
            {
                "uri": "harbor://templates",
                "name": "Templates",
                "description": "List all available project templates",
                "mimeType": "application/json"
            }
        ]
    })
}

pub fn list_resource_templates() -> Value {
    json!({
        "resourceTemplates": [
            {
                "uriTemplate": "harbor://project/{id}/environment",
                "name": "Project Environment",
                "description": "Get current environment state for a project",
                "mimeType": "application/json"
            },
            {
                "uriTemplate": "harbor://project/{id}/expected",
                "name": "Project Expected Environment",
                "description": "Get expected environment from .harbor.yml",
                "mimeType": "application/json"
            },
            {
                "uriTemplate": "harbor://project/{id}/drift",
                "name": "Project Drift Report",
                "description": "Get drift report for a project",
                "mimeType": "application/json"
            }
        ]
    })
}

pub fn read_resource(ctx: &McpContext, params: &Value) -> Value {
    let uri = params["uri"].as_str().unwrap_or("");

    match uri {
        "harbor://projects" => read_projects(ctx),
        "harbor://plugins/available" => read_plugins(ctx),
        "harbor://engines/installed" => read_engines(ctx),
        "harbor://templates" => read_templates(ctx),
        u if u.starts_with("harbor://project/") && u.ends_with("/environment") => {
            let id = extract_project_id(u, "/environment");
            read_project_environment(ctx, &id)
        }
        u if u.starts_with("harbor://project/") && u.ends_with("/expected") => {
            let id = extract_project_id(u, "/expected");
            read_project_expected(ctx, &id)
        }
        u if u.starts_with("harbor://project/") && u.ends_with("/drift") => {
            let id = extract_project_id(u, "/drift");
            read_project_drift(ctx, &id)
        }
        _ => json!({
            "contents": [{
                "uri": uri,
                "mimeType": "application/json",
                "text": serde_json::to_string_pretty(&json!({"error": "Unknown resource", "uri": uri})).unwrap()
            }]
        }),
    }
}

fn extract_project_id<'a>(uri: &'a str, suffix: &str) -> String {
    let prefix = "harbor://project/";
    uri.strip_prefix(prefix)
        .unwrap_or("")
        .strip_suffix(suffix)
        .unwrap_or("")
        .to_string()
}

fn read_projects(ctx: &McpContext) -> Value {
    let projects: Vec<Project> = ctx.storage.load_or_default("projects.json");
    let summary: Vec<Value> = projects.iter().map(|p| {
        json!({
            "project_id": p.project_id,
            "name": p.name,
            "path": p.path,
            "godot_version": p.godot_version,
        })
    }).collect();
    json!({
        "contents": [{
            "uri": "harbor://projects",
            "mimeType": "application/json",
            "text": serde_json::to_string_pretty(&summary).unwrap()
        }]
    })
}

fn read_plugins(ctx: &McpContext) -> Value {
    let plugins: Vec<Plugin> = ctx.storage.load_or_default("plugins.json");
    let summary: Vec<Value> = plugins.iter().map(|p| {
        json!({
            "plugin_id": p.plugin_id,
            "name": p.name,
            "versions": p.versions.iter().map(|v| &v.version).collect::<Vec<_>>(),
            "source_type": match p.source.source_type {
                SourceType::Git => "git",
                SourceType::AssetLibrary => "asset-store",
                SourceType::Url => "url",
                SourceType::Local => "local",
            },
        })
    }).collect();
    json!({
        "contents": [{
            "uri": "harbor://plugins/available",
            "mimeType": "application/json",
            "text": serde_json::to_string_pretty(&summary).unwrap()
        }]
    })
}

fn read_engines(ctx: &McpContext) -> Value {
    let engines: Vec<Engine> = ctx.storage.load_or_default("engines.json");
    let summary: Vec<Value> = engines.iter().map(|e| {
        json!({
            "engine_id": e.engine_id,
            "version": e.version,
            "is_mono": e.is_mono,
            "path": e.path,
        })
    }).collect();
    json!({
        "contents": [{
            "uri": "harbor://engines/installed",
            "mimeType": "application/json",
            "text": serde_json::to_string_pretty(&summary).unwrap()
        }]
    })
}

fn read_templates(ctx: &McpContext) -> Value {
    let templates_dir = ctx.data_dir.join("templates");
    let mut templates = Vec::new();
    if templates_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&templates_dir) {
            for entry in entries.flatten() {
                let template_yml = entry.path().join("template.yml");
                if template_yml.exists() {
                    if let Ok(content) = std::fs::read_to_string(&template_yml) {
                        if let Ok(tpl) = serde_yaml::from_str::<Template>(&content) {
                            templates.push(json!({
                                "template_id": tpl.template_id,
                                "name": tpl.name,
                                "category": tpl.category,
                                "godot_version": tpl.godot.version,
                            }));
                        }
                    }
                }
            }
        }
    }
    json!({
        "contents": [{
            "uri": "harbor://templates",
            "mimeType": "application/json",
            "text": serde_json::to_string_pretty(&templates).unwrap()
        }]
    })
}

fn read_project_environment(ctx: &McpContext, project_id: &str) -> Value {
    let projects: Vec<Project> = ctx.storage.load_or_default("projects.json");
    let project = match projects.iter().find(|p| p.project_id == project_id) {
        Some(p) => p,
        None => {
            return json!({
                "contents": [{
                    "uri": format!("harbor://project/{}/environment", project_id),
                    "mimeType": "application/json",
                    "text": serde_json::to_string_pretty(&json!({"error": "Project not found"})).unwrap()
                }]
            });
        }
    };

    let bindings: Vec<ProjectBinding> = ctx.storage.load_or_default("bindings.json");
    let plugins: Vec<Plugin> = ctx.storage.load_or_default("plugins.json");
    let engines: Vec<Engine> = ctx.storage.load_or_default("engines.json");

    let project_bindings: Vec<&ProjectBinding> = bindings.iter()
        .filter(|b| b.project_id == project_id)
        .collect();

    let bound_plugins: Vec<Value> = project_bindings.iter().filter_map(|b| {
        plugins.iter().find(|p| p.plugin_id == b.plugin_id).map(|p| {
            json!({
                "name": p.name,
                "version": p.versions.first().map(|v| v.version.clone()).unwrap_or_default(),
                "source": match p.source.source_type {
                    SourceType::Git => "git",
                    SourceType::AssetLibrary => "asset-store",
                    SourceType::Url => "url",
                    SourceType::Local => "local",
                },
            })
        })
    }).collect();

    let engine_info = project.last_used_engine_id.as_ref().and_then(|eid| {
        engines.iter().find(|e| &e.engine_id == eid).map(|e| {
            json!({
                "version": e.version,
                "is_mono": e.is_mono,
                "path": e.path,
            })
        })
    });

    let env = json!({
        "project_id": project_id,
        "project_name": project.name,
        "engine": engine_info,
        "plugins": bound_plugins,
    });

    json!({
        "contents": [{
            "uri": format!("harbor://project/{}/environment", project_id),
            "mimeType": "application/json",
            "text": serde_json::to_string_pretty(&env).unwrap()
        }]
    })
}

fn read_project_expected(ctx: &McpContext, project_id: &str) -> Value {
    let projects: Vec<Project> = ctx.storage.load_or_default("projects.json");
    let project = match projects.iter().find(|p| p.project_id == project_id) {
        Some(p) => p,
        None => {
            return json!({
                "contents": [{
                    "uri": format!("harbor://project/{}/expected", project_id),
                    "mimeType": "application/json",
                    "text": serde_json::to_string_pretty(&json!({"error": "Project not found"})).unwrap()
                }]
            });
        }
    };

    let config = harbor_config::read_harbor_config_from_project(&project.path);
    let expected = match config {
        Ok(Some(c)) => {
            let upgraded = if c.version < 2 { c.upgrade_to_v2() } else { c };
            json!({
                "godot": upgraded.godot,
                "plugins": upgraded.plugins,
                "export_presets": upgraded.export_presets,
                "ci": upgraded.ci,
            })
        }
        _ => json!({"error": "No .harbor.yml found", "project_id": project_id}),
    };

    json!({
        "contents": [{
            "uri": format!("harbor://project/{}/expected", project_id),
            "mimeType": "application/json",
            "text": serde_json::to_string_pretty(&expected).unwrap()
        }]
    })
}

fn read_project_drift(ctx: &McpContext, project_id: &str) -> Value {
    let projects: Vec<Project> = ctx.storage.load_or_default("projects.json");
    let project = match projects.iter().find(|p| p.project_id == project_id) {
        Some(p) => p,
        None => {
            return json!({
                "contents": [{
                    "uri": format!("harbor://project/{}/drift", project_id),
                    "mimeType": "application/json",
                    "text": serde_json::to_string_pretty(&json!({"error": "Project not found"})).unwrap()
                }]
            });
        }
    };

    let config = harbor_config::read_harbor_config_from_project(&project.path);
    let report = match config {
        Ok(Some(c)) => {
            let upgraded = if c.version < 2 { c.upgrade_to_v2() } else { c };
            compute_drift_report(ctx, project_id, &project.path, &upgraded)
        }
        _ => json!({"has_drift": true, "items": [{"item_type": "config", "name": ".harbor.yml", "status": "Missing", "message": "No .harbor.yml found"}]}),
    };

    json!({
        "contents": [{
            "uri": format!("harbor://project/{}/drift", project_id),
            "mimeType": "application/json",
            "text": serde_json::to_string_pretty(&report).unwrap()
        }]
    })
}

fn compute_drift_report(ctx: &McpContext, project_id: &str, _project_path: &str, config: &harbor_config::HarborConfig) -> Value {
    let engines: Vec<Engine> = ctx.storage.load_or_default("engines.json");
    let bindings: Vec<ProjectBinding> = ctx.storage.load_or_default("bindings.json");
    let plugins: Vec<Plugin> = ctx.storage.load_or_default("plugins.json");

    let mut items = Vec::new();

    if let Some(ref godot_cfg) = config.godot {
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
            items.push(json!({
                "item_type": "engine",
                "name": "godot",
                "status": "Missing",
                "expected": godot_cfg.version,
                "actual": null,
                "message": format!("Engine version {} (mono={}) not installed", godot_cfg.version, godot_cfg.mono)
            }));
        }
    }

    let project_bindings: Vec<&ProjectBinding> = bindings.iter()
        .filter(|b| b.project_id == project_id)
        .collect();

    for pc in &config.plugins {
        let binding_exists = project_bindings.iter().any(|b| {
            plugins.iter().find(|p| p.plugin_id == b.plugin_id)
                .map_or(false, |p| p.name.to_lowercase() == pc.name.to_lowercase())
        });
        if !binding_exists {
            items.push(json!({
                "item_type": "plugin",
                "name": pc.name,
                "status": "Missing",
                "expected": pc.version,
                "actual": null,
                "message": format!("Plugin {} v{} not installed", pc.name, pc.version)
            }));
        }
    }

    json!({
        "has_drift": !items.is_empty(),
        "items": items,
    })
}
