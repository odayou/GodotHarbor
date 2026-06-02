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

#[tauri::command]
pub fn is_mcp_server_running() -> bool {
    mcp::server::is_mcp_running()
}

#[tauri::command]
pub fn get_mcp_server_path(_app: tauri::AppHandle) -> Result<String, String> {
    // MCP 服务器可执行文件与主程序在同一目录
    let exe_dir = std::env::current_exe()
        .map_err(|e| format!("Failed to get exe path: {}", e))?
        .parent()
        .ok_or("Failed to get exe directory")?
        .to_path_buf();

    let exe_name = if cfg!(windows) { "harbor-mcp-server.exe" } else { "harbor-mcp-server" };
    let mcp_path = exe_dir.join(exe_name);

    if mcp_path.exists() {
        Ok(mcp_path.to_string_lossy().to_string())
    } else {
        // 开发模式下可能在上级目录
        if let Some(parent) = exe_dir.parent() {
            let dev_path = parent.join(exe_name);
            if dev_path.exists() {
                return Ok(dev_path.to_string_lossy().to_string());
            }
        }
        // 返回默认路径，让前端显示
        Ok(mcp_path.to_string_lossy().to_string())
    }
}
