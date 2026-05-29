use crate::mcp;

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
