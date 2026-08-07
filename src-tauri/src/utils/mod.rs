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

/// 在同步/阻塞上下文里驱动一个 future 到完成。
/// 复用进程级共享的 tokio Runtime，避免每次调用都新建/销毁一个
/// multi-thread runtime（网络下载等场景的常见浪费）。
///
/// 注意：必须从无 runtime 上下文的阻塞线程（如 `spawn_blocking` 的线程池线程）
/// 调用；若从 async 任务线程直接调用会触发嵌套 runtime 的 block_on panic。
pub fn block_on<F: std::future::Future>(fut: F) -> F::Output {
    use std::sync::OnceLock;
    static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    let rt = RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("failed to build shared tokio runtime")
    });
    rt.block_on(fut)
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
    let timeout_duration = timeout.unwrap_or(std::time::Duration::from_secs(30));
    reqwest::Client::builder()
        .user_agent("GodotHarbor")
        .timeout(timeout_duration)
        .connect_timeout(std::time::Duration::from_secs(10))
        .read_timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))
}

pub fn get_github_api_base(app: &tauri::AppHandle) -> String {
    let settings = crate::commands::load_settings(app);
    if !settings.github_api_proxy.is_empty() {
        settings.github_api_proxy.trim_end_matches('/').to_string()
    } else {
        "https://api.github.com".to_string()
    }
}

pub fn get_asset_library_base(app: &tauri::AppHandle) -> String {
    let settings = crate::commands::load_settings(app);
    if !settings.asset_library_mirror.is_empty() {
        settings.asset_library_mirror.trim_end_matches('/').to_string()
    } else {
        "https://godotengine.org/asset-library/api".to_string()
    }
}

pub fn apply_github_api_proxy(url: &str, proxy_base: &str) -> String {
    if proxy_base.is_empty() {
        return url.to_string();
    }
    url.replace("https://api.github.com", proxy_base.trim_end_matches('/'))
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

/// 判断 Godot 版本是否恰为 Godot 4.x（主版本号 == 4）。
/// 语义为主版本精确匹配，避免 Godot 5+ 被错误归类为 Godot 4。
pub fn is_godot4(version: &str) -> bool {
    parse_version(version).0 == 4
}

/// 判断 Godot 版本是否恰为 Godot 3.x（主版本号 == 3）。
pub fn is_godot3(version: &str) -> bool {
    parse_version(version).0 == 3
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_should_skip_dir_git() {
        assert!(should_skip_dir(".git"));
    }

    #[test]
    fn test_should_skip_dir_node_modules() {
        assert!(should_skip_dir("node_modules"));
    }

    #[test]
    fn test_should_skip_dir_godot() {
        assert!(should_skip_dir(".godot"));
    }

    #[test]
    fn test_should_skip_dir_normal() {
        assert!(!should_skip_dir("addons"));
        assert!(!should_skip_dir("src"));
    }

    #[test]
    fn test_should_skip_dir_case_insensitive() {
        assert!(should_skip_dir(".Git"));
        assert!(should_skip_dir("Node_Modules"));
    }

    #[test]
    fn test_copy_dir_all_basic() {
        let src = TempDir::new().unwrap();
        let dst = TempDir::new().unwrap();

        std::fs::write(src.path().join("file.txt"), b"hello").unwrap();
        std::fs::create_dir_all(src.path().join("subdir")).unwrap();
        std::fs::write(src.path().join("subdir").join("nested.txt"), b"world").unwrap();

        copy_dir_all(src.path(), dst.path()).unwrap();

        assert!(dst.path().join("file.txt").exists());
        assert!(dst.path().join("subdir").join("nested.txt").exists());
        assert_eq!(std::fs::read_to_string(dst.path().join("file.txt")).unwrap(), "hello");
    }

    #[test]
    fn test_copy_dir_all_empty() {
        let src = TempDir::new().unwrap();
        let dst = TempDir::new().unwrap();

        copy_dir_all(src.path(), dst.path()).unwrap();
        assert!(dst.path().exists());
    }

    #[test]
    fn test_parse_version_standard() {
        assert_eq!(parse_version("4.2.1"), (4, 2, 1));
    }

    #[test]
    fn test_parse_version_two_parts() {
        assert_eq!(parse_version("4.2"), (4, 2, 0));
    }

    #[test]
    fn test_parse_version_with_suffix() {
        assert_eq!(parse_version("4.3-rc1"), (4, 3, 0));
    }

    #[test]
    fn test_parse_version_single() {
        assert_eq!(parse_version("4"), (4, 0, 0));
    }

    #[test]
    fn test_parse_version_invalid() {
        assert_eq!(parse_version("abc"), (0, 0, 0));
    }

    #[test]
    fn test_apply_github_api_proxy_empty() {
        assert_eq!(apply_github_api_proxy("https://api.github.com/repos/test", ""), "https://api.github.com/repos/test");
    }

    #[test]
    fn test_apply_github_api_proxy_with_proxy() {
        assert_eq!(
            apply_github_api_proxy("https://api.github.com/repos/test", "https://mirror.example.com"),
            "https://mirror.example.com/repos/test"
        );
    }

    #[test]
    fn test_apply_github_api_proxy_trailing_slash() {
        assert_eq!(
            apply_github_api_proxy("https://api.github.com/repos/test", "https://mirror.example.com/"),
            "https://mirror.example.com/repos/test"
        );
    }

    #[test]
    fn test_is_godot4_standard() {
        assert!(is_godot4("4.2.1"));
        assert!(is_godot4("4.0"));
        assert!(is_godot4("4"));
    }

    #[test]
    fn test_is_godot4_not_3x() {
        assert!(!is_godot4("3.4.1"));
        assert!(!is_godot4("3.5"));
        assert!(!is_godot4("3"));
    }

    #[test]
    fn test_is_godot4_not_5x() {
        assert!(!is_godot4("5.0"));
        assert!(!is_godot4("5.1.2"));
        assert!(!is_godot4("5"));
        assert!(!is_godot4("6.0"));
    }

    #[test]
    fn test_is_godot3_standard() {
        assert!(is_godot3("3.4.1"));
        assert!(is_godot3("3.5"));
        assert!(is_godot3("3"));
    }

    #[test]
    fn test_is_godot3_not_4x() {
        assert!(!is_godot3("4.2.1"));
        assert!(!is_godot3("4.0"));
    }

    #[test]
    fn test_is_godot3_not_5x() {
        assert!(!is_godot3("5.0"));
        assert!(!is_godot3("5.4.1"));
        assert!(!is_godot3("6.0"));
    }
}
