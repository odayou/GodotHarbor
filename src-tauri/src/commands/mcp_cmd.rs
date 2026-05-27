use crate::mcp;

#[tauri::command]
pub fn start_mcp_server() -> Result<String, String> {
    std::thread::spawn(|| {
        mcp::server::run_mcp_server();
    });
    Ok("MCP Server started".to_string())
}
