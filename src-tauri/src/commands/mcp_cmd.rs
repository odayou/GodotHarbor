use crate::mcp;
use serde_json::{json, Value};
use tauri::Manager;

#[tauri::command]
pub fn start_mcp_server() -> Result<String, String> {
    std::thread::spawn(|| {
        mcp::server::run_mcp_server();
    });
    Ok("MCP Server started".to_string())
}

#[tauri::command]
pub fn stop_mcp_server() -> Result<String, String> {
    if mcp::server::stop_mcp() {
        Ok("MCP Server stopped".to_string())
    } else {
        Err("MCP Server is not running".to_string())
    }
}

#[tauri::command]
pub fn is_mcp_server_running() -> bool {
    mcp::server::is_mcp_running()
}

#[tauri::command]
pub fn get_mcp_server_path(app: tauri::AppHandle) -> Result<String, String> {
    let exe_name = if cfg!(windows) { "harbor-mcp-server.exe" } else { "harbor-mcp-server" };

    let candidates: Vec<std::path::PathBuf> = {
        let mut paths = Vec::new();

        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                paths.push(dir.join(exe_name));
                if let Some(parent) = dir.parent() {
                    paths.push(parent.join(exe_name));
                    paths.push(parent.join("release").join(exe_name));
                    paths.push(parent.join("debug").join(exe_name));
                    if let Some(grandparent) = parent.parent() {
                        paths.push(grandparent.join("release").join(exe_name));
                        paths.push(grandparent.join("debug").join(exe_name));
                    }
                }
            }
        }

        if let Ok(resolved) = app.path().app_local_data_dir() {
            paths.push(resolved.join(exe_name));
        }

        if let Ok(resolved) = app.path().resource_dir() {
            paths.push(resolved.join(exe_name));
            paths.push(resolved.join("binaries").join(exe_name));
        }

        paths
    };

    for path in &candidates {
        if path.exists() {
            return Ok(path.to_string_lossy().to_string());
        }
    }

    candidates.first()
        .map(|p| Ok(p.to_string_lossy().to_string()))
        .unwrap_or_else(|| Err("Cannot determine MCP server path".to_string()))
}

#[tauri::command]
pub fn get_mcp_capabilities() -> Value {
    let tools_val = mcp::tools::list_tools();
    let resources_val = mcp::resources::list_resources();
    let prompts_val = mcp::prompts::list_prompts();

    let tools_list: Vec<Value> = tools_val["tools"].as_array()
        .map(|arr| arr.iter().map(|t| json!({
            "name": t["name"],
            "description": t["description"]
        })).collect())
        .unwrap_or_default();

    let resources_list: Vec<Value> = resources_val["resources"].as_array()
        .map(|arr| arr.iter().map(|r| json!({
            "uri": r["uri"],
            "name": r["name"],
            "description": r["description"]
        })).collect())
        .unwrap_or_default();

    let prompts_list: Vec<Value> = prompts_val["prompts"].as_array()
        .map(|arr| arr.iter().map(|p| json!({
            "name": p["name"],
            "description": p["description"]
        })).collect())
        .unwrap_or_default();

    json!({
        "tools": tools_list,
        "tools_count": tools_list.len(),
        "resources": resources_list,
        "resources_count": resources_list.len(),
        "prompts": prompts_list,
        "prompts_count": prompts_list.len()
    })
}
