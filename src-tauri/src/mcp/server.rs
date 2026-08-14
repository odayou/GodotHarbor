use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::net::TcpListener;
use std::thread;
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

/// A1-1: TCP transport 模式，供 harbor-bridge addon 连接。
/// 行分隔 JSON-RPC（与 stdio 协议一致，但走 TCP socket）。
/// 用 Arc 让多客户端共享 McpContext（只读快照，无并发写问题）。
pub fn run_mcp_server_tcp(port: u16) {
    if MCP_RUNNING.swap(true, Ordering::Relaxed) {
        return;
    }
    let listener = TcpListener::bind(("127.0.0.1", port))
        .unwrap_or_else(|e| {
            eprintln!("MCP TCP 监听失败 (port {}): {}", port, e);
            std::process::exit(1);
        });
    eprintln!("MCP server listening on 127.0.0.1:{} (TCP)", port);

    let ctx = Arc::new(McpContext::new());

    while MCP_RUNNING.load(Ordering::Relaxed) {
        match listener.accept() {
            Ok((stream, addr)) => {
                eprintln!("MCP client connected: {}", addr);
                let ctx = ctx.clone();
                thread::spawn(move || {
                    handle_tcp_client(stream, ctx);
                });
            }
            Err(e) => {
                eprintln!("MCP accept 失败: {}", e);
                break;
            }
        }
    }
    MCP_RUNNING.store(false, Ordering::Relaxed);
}

fn handle_tcp_client(stream: std::net::TcpStream, ctx: Arc<McpContext>) {
    let mut writer = match stream.try_clone() {
        Ok(w) => w,
        Err(_) => return,
    };
    let reader = io::BufReader::new(stream);
    for line in reader.lines() {
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
                    let _ = writeln!(writer, "{}", output);
                    let _ = writer.flush();
                }
            }
            Err(_) => break,
        }
    }
}

fn handle_request(ctx: &McpContext, req: &JsonRpcRequest) -> Option<JsonRpcResponse> {
    match req.method.as_str() {
        "initialize" => Some(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: req.id.clone(),
            result: Some(json!({
                "protocolVersion": PROTOCOL_VERSION,
                "capabilities": {
                    "resources": { "subscribe": true, "listChanged": true },
                    "tools": { "listChanged": true },
                    "prompts": { "listChanged": true },
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
