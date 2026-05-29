use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use crate::storage::Storage;
use crate::models::*;
use super::resources;
use super::tools;
use super::prompts;

static MCP_RUNNING: AtomicBool = AtomicBool::new(false);

pub fn is_mcp_running() -> bool {
    MCP_RUNNING.load(Ordering::Relaxed)
}

pub fn stop_mcp() -> bool {
    if MCP_RUNNING.load(Ordering::Relaxed) {
        MCP_RUNNING.store(false, Ordering::Relaxed);
        true
    } else {
        false
    }
}

const SERVER_NAME: &str = "godot-harbor-mcp";
const SERVER_VERSION: &str = "1.0.0";
const PROTOCOL_VERSION: &str = "2024-11-05";

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    #[allow(dead_code)]
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Value,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

pub struct McpContext {
    pub storage: Storage,
    pub data_dir: PathBuf,
}

impl McpContext {
    fn new() -> Self {
        let data_dir = get_harbor_data_dir();
        let storage = Storage::new(data_dir.clone());
        Self { storage, data_dir }
    }
}

fn get_harbor_data_dir() -> PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("godot-harbor");
    if config_dir.exists() {
        let config_storage = Storage::new(config_dir.clone());
        let settings: Settings = config_storage.load_or_default("settings.json");
        if !settings.custom_data_dir.is_empty() {
            return PathBuf::from(settings.custom_data_dir);
        }
    }
    config_dir
}

pub fn run_mcp_server() {
    if MCP_RUNNING.swap(true, Ordering::Relaxed) {
        return;
    }
    let ctx = McpContext::new();
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        if !MCP_RUNNING.load(Ordering::Relaxed) {
            break;
        }
        match line {
            Ok(line) => {
                if line.trim().is_empty() {
                    continue;
                }
                let request: JsonRpcRequest = match serde_json::from_str(&line) {
                    Ok(r) => r,
                    Err(_) => continue,
                };
                let response = handle_request(&ctx, &request);
                if let Some(resp) = response {
                    let output = serde_json::to_string(&resp).unwrap_or_default();
                    let _ = writeln!(stdout, "{}", output);
                    let _ = stdout.flush();
                }
            }
            Err(_) => break,
        }
    }
    MCP_RUNNING.store(false, Ordering::Relaxed);
}

fn handle_request(ctx: &McpContext, req: &JsonRpcRequest) -> Option<JsonRpcResponse> {
    match req.method.as_str() {
        "initialize" => Some(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: req.id.clone(),
            result: Some(json!({
                "protocolVersion": PROTOCOL_VERSION,
                "capabilities": {
                    "resources": { "subscribe": false, "listChanged": false },
                    "tools": { "listChanged": false },
                    "prompts": { "listChanged": false },
                },
                "serverInfo": {
                    "name": SERVER_NAME,
                    "version": SERVER_VERSION,
                }
            })),
            error: None,
        }),
        "notifications/initialized" => None,
        "ping" => Some(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: req.id.clone(),
            result: Some(json!({})),
            error: None,
        }),
        "resources/list" => Some(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: req.id.clone(),
            result: Some(resources::list_resources()),
            error: None,
        }),
        "resources/read" => Some(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: req.id.clone(),
            result: Some(resources::read_resource(ctx, &req.params)),
            error: None,
        }),
        "resources/templates/list" => Some(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: req.id.clone(),
            result: Some(resources::list_resource_templates()),
            error: None,
        }),
        "tools/list" => Some(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: req.id.clone(),
            result: Some(tools::list_tools()),
            error: None,
        }),
        "tools/call" => {
            let result = tools::call_tool(ctx, &req.params);
            Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: req.id.clone(),
                result: Some(result),
                error: None,
            })
        },
        "prompts/list" => Some(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: req.id.clone(),
            result: Some(prompts::list_prompts()),
            error: None,
        }),
        "prompts/get" => Some(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: req.id.clone(),
            result: Some(prompts::get_prompt(ctx, &req.params)),
            error: None,
        }),
        _ => Some(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: req.id.clone(),
            result: None,
            error: Some(JsonRpcError {
                code: -32601,
                message: format!("Method not found: {}", req.method),
            }),
        }),
    }
}
