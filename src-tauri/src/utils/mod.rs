use std::path::Path;
use std::fs;

#[cfg(windows)]
pub fn no_window_cmd(program: impl AsRef<std::ffi::OsStr>) -> std::process::Command {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    let mut cmd = std::process::Command::new(program);
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd
}

#[cfg(not(windows))]
pub fn no_window_cmd(program: impl AsRef<std::ffi::OsStr>) -> std::process::Command {
    std::process::Command::new(program)
}

pub const SKIP_DIRS: &[&str] = &[
    ".git", ".svn", ".hg",
    "node_modules", "__pycache__",
    ".godot", ".import",
    "build", "dist", ".cache",
    "Library", "Temp",
];

pub fn should_skip_dir(name: &str) -> bool {
    let lower = name.to_lowercase();
    SKIP_DIRS.iter().any(|skip| lower == *skip)
}

pub fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<(), String> {
    let src = src.as_ref();
    let dst = dst.as_ref();

    fs::create_dir_all(dst)
        .map_err(|e| format!("创建目录失败: {}", e))?;

    for entry in fs::read_dir(src)
        .map_err(|e| format!("读取目录失败: {}", e))?
    {
        let entry = entry.map_err(|e| format!("读取目录条目失败: {}", e))?;
        let ty = entry.file_type().map_err(|e| format!("获取文件类型失败: {}", e))?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.join(entry.file_name()))
                .map_err(|e| format!("复制文件失败: {}", e))?;
        }
    }
    Ok(())
}

pub fn create_http_client(timeout: Option<std::time::Duration>) -> Result<reqwest::Client, String> {
    let mut builder = reqwest::Client::builder().user_agent("GodotHarbor");
    if let Some(d) = timeout {
        builder = builder.timeout(d);
    }
    builder.build().map_err(|e| format!("创建 HTTP 客户端失败: {}", e))
}

pub fn parse_version(version: &str) -> (u32, u32, u32) {
    let clean = version
        .split('-')
        .next()
        .unwrap_or(version)
        .trim();
    let parts: Vec<&str> = clean.split('.').collect();
    let major = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
    let minor = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    let patch = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
    (major, minor, patch)
}
