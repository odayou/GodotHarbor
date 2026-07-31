use serde::{Deserialize, Serialize};
use std::path::Path;
use anyhow::{Context, Result};
use sha2::{Sha256, Digest};

use crate::models::{compute_dir_hash, Engine, Plugin, Project, ProjectBinding, SourceType};

// ─── Data Models ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarborLock {
    pub version: String,
    pub locked_at: String,
    pub project_name: String,
    pub godot_version: String,
    pub engine: Option<LockedEngine>,
    pub plugins: Vec<LockedPlugin>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedEngine {
    pub engine_id: String,
    pub version: String,
    pub engine_type: String,
    pub path_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedPlugin {
    pub plugin_id: String,
    pub plugin_name: String,
    pub version_id: String,
    pub version: String,
    pub unit_id: String,
    pub unit_name: String,
    pub mount_path: String,
    pub subdirectory: String,
    pub content_hash: String,
    pub source_type: String,
    pub source_url: String,
    /// P4-4: Git 源插件的 commit SHA pin。空表示旧版 lockfile 或非 Git 源。
    /// restore 时若非空，强制 checkout 到此 commit；本地 HEAD 不一致则计入 mismatch。
    #[serde(default)]
    pub commit_sha: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockDiff {
    pub added: Vec<LockedPlugin>,
    pub removed: Vec<LockedPlugin>,
    pub changed: Vec<LockChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockChange {
    pub plugin_name: String,
    pub field: String,
    pub old_value: String,
    pub new_value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockVerifyResult {
    pub is_valid: bool,
    pub mismatches: Vec<LockMismatch>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockMismatch {
    pub plugin_name: String,
    pub expected_hash: String,
    pub actual_hash: String,
    pub expected_version: String,
    pub actual_version: String,
    #[serde(default)]
    pub mount_path_issue: Option<String>,
    /// P4-4: commit SHA pin 不匹配的提示（restore 后 HEAD 与 lockfile 声明不一致）。
    #[serde(default)]
    pub commit_sha_issue: Option<String>,
}

/// 「还原项目环境」命令的返回结果。
/// ready: 本地已存在直接复用的插件名；
/// imported: 通过 Git/Url 自动导入成功的插件名；
/// failed: 导入或应用失败的条目（含原因）；
/// missing: source_type 为 AssetLibrary/Local 无 URL，无法跨机器还原的插件名。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreEnvResult {
    pub ready: Vec<String>,
    pub imported: Vec<String>,
    pub failed: Vec<String>,
    pub missing: Vec<String>,
}

// ─── Core Functions ───

fn compute_path_hash(path: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(path.as_bytes());
    format!("sha256:{:x}", hasher.finalize())
}

pub fn get_lock_path(project_path: &str) -> std::path::PathBuf {
    Path::new(project_path).join("harbor.lock")
}

pub fn generate_lock(
    project: &Project,
    bindings: &[ProjectBinding],
    plugins: &[Plugin],
    engines: &[Engine],
    engine_bindings: &[(String, String)],
) -> HarborLock {
    let project_bindings: Vec<&ProjectBinding> = bindings
        .iter()
        .filter(|b| b.project_id == project.project_id)
        .collect();

    let locked_engine = engine_bindings
        .iter()
        .find(|(pid, _)| pid == &project.project_id)
        .and_then(|(_, engine_id)| engines.iter().find(|e| e.engine_id == *engine_id))
        .map(|e| LockedEngine {
            engine_id: e.engine_id.clone(),
            version: e.version.clone(),
            engine_type: e.engine_type.to_string(),
            path_hash: compute_path_hash(&e.path),
        });

    let mut locked_plugins = Vec::new();
    for binding in &project_bindings {
        if let Some(plugin) = plugins.iter().find(|p| p.plugin_id == binding.plugin_id) {
            let version = plugin.versions.iter()
                .find(|v| v.version_id == binding.version_id);
            let unit = version
                .and_then(|v| v.units.iter().find(|u| u.unit_id == binding.unit_id));

            let version_str = version.map(|v| v.version.clone()).unwrap_or_default();
            let unit_name = unit.map(|u| u.name.clone()).unwrap_or_default();

            let content_hash = if let Some(ver) = version {
                let plugin_dir = Path::new(&ver.path);
                if plugin_dir.exists() {
                    compute_dir_hash(plugin_dir).unwrap_or_default()
                } else {
                    plugin.content_hash.clone()
                }
            } else {
                plugin.content_hash.clone()
            };

            // P4-4: 提取 Git 源插件的 commit SHA pin。
            // 优先读 version path 同级的 git_store_dir/HEAD（import_from_git 已备份）；
            // 失败则 fallback 到 plugin.source.git_ref（若已是 40 位 SHA）。
            let commit_sha = if matches!(plugin.source.source_type, SourceType::Git) {
                if let Some(ver) = version {
                    let payload_dir = Path::new(&ver.path);
                    let git_store_dir = payload_dir.parent().map(|p| p.join("git"));
                    let from_store = git_store_dir
                        .and_then(|gsd| if gsd.exists() { read_git_commit_sha(&gsd) } else { None });
                    from_store.unwrap_or_else(|| {
                        if is_commit_sha(&plugin.source.git_ref) {
                            plugin.source.git_ref.clone()
                        } else {
                            String::new()
                        }
                    })
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            let source_type = match plugin.source.source_type {
                SourceType::Git => "Git".to_string(),
                SourceType::Local => "Local".to_string(),
                SourceType::AssetLibrary => "AssetLibrary".to_string(),
                SourceType::Url => "Url".to_string(),
            };

            locked_plugins.push(LockedPlugin {
                plugin_id: plugin.plugin_id.clone(),
                plugin_name: plugin.name.clone(),
                version_id: binding.version_id.clone(),
                version: version_str,
                unit_id: binding.unit_id.clone(),
                unit_name,
                mount_path: binding.mount_path.clone(),
                subdirectory: binding.subdirectory.clone(),
                content_hash,
                source_type,
                source_url: plugin.source.url.clone(),
                commit_sha,
            });
        }
    }

    HarborLock {
        version: "1".to_string(),
        locked_at: chrono::Utc::now().to_rfc3339(),
        project_name: project.name.clone(),
        godot_version: project.godot_version.clone(),
        engine: locked_engine,
        plugins: locked_plugins,
    }
}

pub fn write_lock(project_path: &str, lock: &HarborLock) -> Result<()> {
    let lock_path = get_lock_path(project_path);
    let content = serde_yaml::to_string(lock)
        .context("序列化 harbor.lock 失败")?;
    std::fs::write(&lock_path, content)
        .with_context(|| format!("写入 harbor.lock 失败: {}", lock_path.to_string_lossy()))?;
    Ok(())
}

/// 根据 project 与全部 bindings（含 plugins/engines）生成并写入 harbor.lock。
/// 绑定变更应用成功后应调用此函数刷新 lockfile，使项目目录与持久化锁文件保持一致。
/// `all_bindings` 应传 storage 中完整的 bindings 列表（内部按 project_id 过滤）。
pub fn write_lock_for_project(
    project: &Project,
    all_bindings: &[ProjectBinding],
    plugins: &[Plugin],
    engines: &[Engine],
) -> Result<()> {
    let engine_bindings = if let Some(ref engine_id) = project.last_used_engine_id {
        vec![(project.project_id.clone(), engine_id.clone())]
    } else {
        vec![]
    };
    let lock = generate_lock(project, all_bindings, plugins, engines, &engine_bindings);
    write_lock(&project.path, &lock)
}

pub fn read_lock(project_path: &str) -> Result<Option<HarborLock>> {
    let lock_path = get_lock_path(project_path);
    if !lock_path.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(&lock_path)
        .with_context(|| format!("读取 harbor.lock 失败: {}", lock_path.to_string_lossy()))?;
    let lock: HarborLock = serde_yaml::from_str(&content)
        .context("解析 harbor.lock 失败")?;
    Ok(Some(lock))
}

pub fn verify_lock(
    project_path: &str,
    lock: &HarborLock,
    plugins: &[Plugin],
) -> LockVerifyResult {
    let mut mismatches = Vec::new();

    for locked in &lock.plugins {
        let mut mount_path_issue: Option<String> = None;

        // Check project directory state for this plugin's mount path
        let mount_full_path = Path::new(project_path).join(&locked.mount_path);
        if !mount_full_path.exists() {
            mount_path_issue = Some(format!(
                "挂载路径 '{}' 在项目目录中不存在", locked.mount_path
            ));
        } else {
            let metadata = std::fs::symlink_metadata(&mount_full_path);
            match &metadata {
                Ok(meta) => {
                    if meta.file_type().is_symlink() {
                        // Symlink: check if it points to a valid target
                        if let Ok(target) = std::fs::read_link(&mount_full_path) {
                            if !target.exists() {
                                mount_path_issue = Some(format!(
                                    "符号链接 '{}' 指向不存在的目标: {}",
                                    locked.mount_path,
                                    target.to_string_lossy()
                                ));
                            }
                        }
                    } else if is_junction_path(&mount_full_path) {
                        // Junction: check if target exists
                        let plugin_payload = Path::new("plugins")
                            .join(&locked.plugin_id)
                            .join(&locked.version_id)
                            .join("payload");
                        let source_path = if locked.subdirectory.is_empty() {
                            plugin_payload
                        } else {
                            plugin_payload.join(&locked.subdirectory)
                        };
                        if !source_path.exists() {
                            mount_path_issue = Some(format!(
                                "Junction '{}' 的源路径不存在", locked.mount_path
                            ));
                        }
                    } else if mount_full_path.is_dir() {
                        // Copy mode: verify hash (managed by bindings)
                        // 仅 SHA256 格式（64 位）强制校验；旧版 SipHash（16 位）跳过并提示升级
                        if is_sha256_hash(&locked.content_hash) {
                            let actual_hash = compute_dir_hash(&mount_full_path).unwrap_or_default();
                            if !actual_hash.is_empty() && actual_hash != locked.content_hash {
                                mount_path_issue = Some(format!(
                                    "复制模式目录 '{}' 内容哈希不匹配", locked.mount_path
                                ));
                            }
                        } else if !locked.content_hash.is_empty() {
                            mount_path_issue = Some(format!(
                                "插件 {} 使用旧版哈希格式，建议重新生成锁文件以启用完整性校验",
                                locked.plugin_name
                            ));
                        }
                    }
                }
                Err(e) => {
                    mount_path_issue = Some(format!(
                        "无法读取挂载路径 '{}' 的元数据: {}", locked.mount_path, e
                    ));
                }
            }
        }

        if let Some(plugin) = plugins.iter().find(|p| p.plugin_id == locked.plugin_id) {
            let version = plugin.versions.iter()
                .find(|v| v.version_id == locked.version_id);

            if let Some(ver) = version {
                // 仅 SHA256 格式强制校验源路径哈希；旧版 SipHash 跳过 hash 比对
                let (hash_mismatch, actual_hash) = if is_sha256_hash(&locked.content_hash) {
                    let actual = compute_dir_hash(Path::new(&ver.path))
                        .unwrap_or_default();
                    (!actual.is_empty() && actual != locked.content_hash, actual)
                } else {
                    (false, String::new())
                };

                // P4-4: commit SHA pin 校验。
                // 若 lockfile 声明了 commit_sha，校验本地 git_store_dir 的 HEAD OID 是否一致。
                // 不一致意味着本地 clone 与 lockfile 声明的供应链状态偏离，计入 mismatch。
                let commit_sha_issue = if !locked.commit_sha.is_empty() {
                    let payload_dir = Path::new(&ver.path);
                    let git_store_dir = payload_dir.parent().map(|p| p.join("git"));
                    match git_store_dir {
                        Some(gsd) if gsd.exists() => {
                            match read_git_commit_sha(&gsd) {
                                Some(actual_oid) if actual_oid == locked.commit_sha => None,
                                Some(actual_oid) => Some(format!(
                                    "commit SHA 不匹配: 锁文件声明 {}，本地实际 {}",
                                    locked.commit_sha, actual_oid
                                )),
                                None => Some(format!(
                                    "无法读取本地 git HEAD（git_store_dir 损坏）"
                                )),
                            }
                        }
                        _ => Some("本地缺少 git_store_dir，无法校验 commit SHA".to_string()),
                    }
                } else {
                    None
                };

                if hash_mismatch || mount_path_issue.is_some() || commit_sha_issue.is_some() {
                    let actual_version = ver.version.clone();
                    mismatches.push(LockMismatch {
                        plugin_name: locked.plugin_name.clone(),
                        expected_hash: locked.content_hash.clone(),
                        actual_hash,
                        expected_version: locked.version.clone(),
                        actual_version,
                        mount_path_issue,
                        commit_sha_issue,
                    });
                }
            } else {
                mismatches.push(LockMismatch {
                    plugin_name: locked.plugin_name.clone(),
                    expected_hash: locked.content_hash.clone(),
                    actual_hash: String::new(),
                    expected_version: locked.version.clone(),
                    actual_version: "未找到版本".to_string(),
                    mount_path_issue,
                    commit_sha_issue: None,
                });
            }
        } else {
            mismatches.push(LockMismatch {
                plugin_name: locked.plugin_name.clone(),
                expected_hash: locked.content_hash.clone(),
                actual_hash: String::new(),
                expected_version: locked.version.clone(),
                actual_version: "未安装".to_string(),
                mount_path_issue,
                commit_sha_issue: None,
            });
        }
    }

    LockVerifyResult {
        is_valid: mismatches.is_empty(),
        mismatches,
    }
}

fn is_junction_path(path: &Path) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if let Ok(metadata) = std::fs::symlink_metadata(path) {
            const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
            return metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0;
        }
    }
    let _ = path;
    false
}

/// 判断 content_hash 是否为 SHA256 格式（64 位十六进制）。
/// 旧版 SipHash64 输出 16 位十六进制，无法与 SHA256 比对，
/// verify 时应跳过强校验并提示用户重新生成锁文件。
fn is_sha256_hash(h: &str) -> bool {
    h.len() == 64 && h.chars().all(|c| c.is_ascii_hexdigit())
}

/// 判断字符串是否为 Git commit SHA（40 位十六进制，git2 OID 标准长度）。
/// 用于区分 plugin.source.git_ref 存的是 commit SHA 还是 branch/tag 名。
pub fn is_commit_sha(s: &str) -> bool {
    s.len() == 40 && s.chars().all(|c| c.is_ascii_hexdigit())
}

/// 从 git_store_dir 读取 HEAD 的 commit SHA。
/// import_from_git 在 clone 后会把 .git 备份到 git_store_dir，
/// 此函数复用该备份以避免 payload_dir 中 .git 被删除后无法读取 OID。
pub fn read_git_commit_sha(git_store_dir: &Path) -> Option<String> {
    let repo = git2::Repository::open(git_store_dir).ok()?;
    let head = repo.head().ok()?;
    let oid = head.target()?;
    Some(oid.to_string())
}

/// P4-4: 校验 URL 是否在 allowlist 内。
/// allowlist 条目为 host glob（如 `github.com`、`*.github.com`、`gitlab.com`）。
/// allowlist 为空 → 允许所有（向后兼容）。
/// URL 解析失败 → 不允许（保守策略，阻止畸形 URL 绕过）。
/// 支持 SSH（git@github.com:org/repo）与 HTTPS 两种形式。
pub fn is_url_allowed(url: &str, allowlist: &[String]) -> bool {
    if allowlist.is_empty() {
        return true;
    }
    let host = extract_host(url);
    match host {
        Some(h) => allowlist.iter().any(|pattern| host_matches(&h, pattern)),
        None => false,
    }
}

/// 从 git URL 提取 host（不依赖 url crate，避免新增依赖）。
/// 支持形式：
///   - `https://github.com/org/repo.git` → `github.com`
///   - `http://github.com/org/repo.git` → `github.com`
///   - `git@github.com:org/repo.git` → `github.com`
///   - `ssh://git@github.com:22/org/repo.git` → `github.com`
///   - `ssh://git@github.com/org/repo.git` → `github.com`
///   - `github.com/org/repo`（无 scheme）→ `github.com`
fn extract_host(url: &str) -> Option<String> {
    let url = url.trim();
    if url.is_empty() {
        return None;
    }
    // 1) SCP 风格 SSH：git@host:path（无 scheme，含 @ 和 :）
    if let Some(at_idx) = url.find('@') {
        let after_at = &url[at_idx + 1..];
        if let Some(colon_idx) = after_at.find(':') {
            let host = &after_at[..colon_idx];
            if !host.is_empty() && !host.contains('/') {
                return Some(host.to_lowercase());
            }
        }
    }
    // 2) 带 scheme：scheme://[user@]host[:port]/path
    let after_scheme = if let Some(scheme_end) = url.find("://") {
        &url[scheme_end + 3..]
    } else {
        // 3) 无 scheme：当作 host/path 处理
        url
    };
    // 去掉 user@ 前缀
    let after_user = if let Some(at_idx) = after_scheme.find('@') {
        &after_scheme[at_idx + 1..]
    } else {
        after_scheme
    };
    // 取第一个 / 或结尾之前的部分作为 host[:port]
    let authority = after_user
        .split('/')
        .next()
        .unwrap_or("");
    // 去掉 :port
    let host = authority.split(':').next().unwrap_or("");
    if host.is_empty() {
        None
    } else {
        Some(host.to_lowercase())
    }
}

/// 判断 host 是否匹配 glob 模式。
/// `*.github.com` 匹配 `api.github.com`、`a.b.github.com`（任意子域），不匹配裸域 `github.com`。
/// `github.com` 精确匹配 `github.com`。
/// 严格 glob 语义（与 CSP/CORS 一致）：`*` 不匹配空字符串，裸域需单独列出。
fn host_matches(host: &str, pattern: &str) -> bool {
    let pattern = pattern.trim().to_lowercase();
    if pattern.is_empty() {
        return false;
    }
    if let Some(suffix) = pattern.strip_prefix("*.") {
        // `*.github.com` 仅匹配以 `.github.com` 结尾的 host（含多级子域）
        return host.ends_with(&format!(".{}", suffix));
    }
    host == pattern
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_commit_sha_valid() {
        assert!(is_commit_sha("0123456789abcdef0123456789abcdef01234567"));
        assert!(is_commit_sha("abcdef0123456789abcdef0123456789abcdef01"));
    }

    #[test]
    fn test_is_commit_sha_invalid() {
        assert!(!is_commit_sha(""));
        assert!(!is_commit_sha("main"));
        assert!(!is_commit_sha("v1.0.0"));
        assert!(!is_commit_sha("0123456789abcdef")); // 16 位，太短
        assert!(!is_commit_sha("0123456789abcdef0123456789abcdef0123456789")); // 40 位但非十六进制
        assert!(!is_commit_sha("z123456789abcdef0123456789abcdef0123456")); // 含非十六进制字符
    }

    #[test]
    fn test_is_url_allowed_empty_allowlist_allows_all() {
        assert!(is_url_allowed("https://github.com/any/repo", &[]));
        assert!(is_url_allowed("https://example.com/x", &[]));
        assert!(is_url_allowed("git@github.com:org/repo", &[]));
    }

    #[test]
    fn test_is_url_allowed_exact_match() {
        let allowlist = vec!["github.com".to_string(), "gitlab.com".to_string()];
        assert!(is_url_allowed("https://github.com/org/repo.git", &allowlist));
        assert!(is_url_allowed("https://gitlab.com/org/repo.git", &allowlist));
        assert!(!is_url_allowed("https://evil.com/org/repo.git", &allowlist));
    }

    #[test]
    fn test_is_url_allowed_glob_match() {
        let allowlist = vec!["*.github.com".to_string()];
        assert!(is_url_allowed("https://api.github.com/org/repo", &allowlist));
        assert!(is_url_allowed("https://raw.github.com/x", &allowlist)); // 一级子域
        assert!(!is_url_allowed("https://raw.githubusercontent.com/x", &allowlist)); // 后缀是 usercontent.com，不匹配
        assert!(!is_url_allowed("https://github.com/org/repo", &allowlist)); // glob 不匹配裸域
        assert!(!is_url_allowed("https://evil.com/x", &allowlist));
    }

    #[test]
    fn test_is_url_allowed_ssh_form() {
        let allowlist = vec!["github.com".to_string()];
        assert!(is_url_allowed("git@github.com:org/repo.git", &allowlist));
        assert!(is_url_allowed("ssh://git@github.com:22/org/repo.git", &allowlist));
        assert!(!is_url_allowed("git@evil.com:org/repo.git", &allowlist));
    }

    #[test]
    fn test_is_url_allowed_malformed_url_blocked() {
        let allowlist = vec!["github.com".to_string()];
        // 畸形 URL 在 allowlist 非空时应被保守阻断
        assert!(!is_url_allowed("not a url at all", &allowlist));
        assert!(!is_url_allowed("", &allowlist));
    }

    #[test]
    fn test_is_url_allowed_case_insensitive() {
        let allowlist = vec!["GitHub.com".to_string()];
        assert!(is_url_allowed("https://GITHUB.COM/org/repo", &allowlist));
        assert!(is_url_allowed("git@Github.Com:org/repo.git", &allowlist));
    }

    #[test]
    fn test_extract_host_various_forms() {
        assert_eq!(extract_host("https://github.com/org/repo.git").as_deref(), Some("github.com"));
        assert_eq!(extract_host("git@github.com:org/repo.git").as_deref(), Some("github.com"));
        assert_eq!(extract_host("ssh://git@github.com:22/org/repo.git").as_deref(), Some("github.com"));
        assert_eq!(extract_host("https://api.github.com/x").as_deref(), Some("api.github.com"));
    }

    #[test]
    fn test_host_matches_glob() {
        assert!(host_matches("api.github.com", "*.github.com"));
        assert!(host_matches("raw.github.com", "*.github.com"));
        assert!(host_matches("a.b.github.com", "*.github.com")); // 多级子域也匹配（宽松 ends_with 语义）
        assert!(!host_matches("raw.githubusercontent.com", "*.github.com")); // 后缀不同
        assert!(!host_matches("github.com", "*.github.com")); // glob 不匹配裸域
        assert!(host_matches("github.com", "github.com"));
        assert!(!host_matches("evil.com", "github.com"));
        assert!(!host_matches("github.com", "")); // 空 pattern 拒绝
    }
}

pub fn diff_locks(old: &HarborLock, new: &HarborLock) -> LockDiff {
    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut changed = Vec::new();

    for new_plugin in &new.plugins {
        if let Some(old_plugin) = old.plugins.iter().find(|p| p.plugin_id == new_plugin.plugin_id) {
            if old_plugin.version != new_plugin.version {
                changed.push(LockChange {
                    plugin_name: new_plugin.plugin_name.clone(),
                    field: "version".to_string(),
                    old_value: old_plugin.version.clone(),
                    new_value: new_plugin.version.clone(),
                });
            }
            if old_plugin.content_hash != new_plugin.content_hash {
                changed.push(LockChange {
                    plugin_name: new_plugin.plugin_name.clone(),
                    field: "content_hash".to_string(),
                    old_value: old_plugin.content_hash.clone(),
                    new_value: new_plugin.content_hash.clone(),
                });
            }
            if old_plugin.mount_path != new_plugin.mount_path {
                changed.push(LockChange {
                    plugin_name: new_plugin.plugin_name.clone(),
                    field: "mount_path".to_string(),
                    old_value: old_plugin.mount_path.clone(),
                    new_value: new_plugin.mount_path.clone(),
                });
            }
        } else {
            added.push(new_plugin.clone());
        }
    }

    for old_plugin in &old.plugins {
        if !new.plugins.iter().any(|p| p.plugin_id == old_plugin.plugin_id) {
            removed.push(old_plugin.clone());
        }
    }

    LockDiff { added, removed, changed }
}

pub fn sync_from_lock(
    _project_path: &str,
    lock: &HarborLock,
    plugins: &[Plugin],
    bindings: &mut Vec<ProjectBinding>,
    project_id: &str,
    strict: bool,
) -> Result<Vec<String>> {
    let mut messages = Vec::new();

    for locked in &lock.plugins {
        let existing_binding = bindings.iter()
            .find(|b| b.project_id == project_id && b.plugin_id == locked.plugin_id);

        if let Some(plugin) = plugins.iter().find(|p| p.plugin_id == locked.plugin_id) {
            let version = plugin.versions.iter()
                .find(|v| v.version_id == locked.version_id);

            if let Some(ver) = version {
                if strict && is_sha256_hash(&locked.content_hash) {
                    let actual_hash = compute_dir_hash(Path::new(&ver.path))
                        .unwrap_or_default();
                    if actual_hash != locked.content_hash {
                        messages.push(format!(
                            "插件 {} 内容哈希不匹配（严格模式），跳过同步",
                            locked.plugin_name
                        ));
                        continue;
                    }
                }

                if let Some(existing) = existing_binding {
                    if existing.version_id != locked.version_id
                        || existing.mount_path != locked.mount_path
                    {
                        if let Some(idx) = bindings.iter().position(|b| {
                            b.project_id == project_id && b.plugin_id == locked.plugin_id
                        }) {
                            bindings[idx] = ProjectBinding::new(
                                project_id.to_string(),
                                locked.plugin_id.clone(),
                                locked.version_id.clone(),
                                locked.unit_id.clone(),
                                locked.mount_path.clone(),
                                locked.subdirectory.clone(),
                            );
                            messages.push(format!("已更新插件 {} 的绑定", locked.plugin_name));
                        }
                    }
                } else {
                    bindings.push(ProjectBinding::new(
                        project_id.to_string(),
                        locked.plugin_id.clone(),
                        locked.version_id.clone(),
                        locked.unit_id.clone(),
                        locked.mount_path.clone(),
                        locked.subdirectory.clone(),
                    ));
                    messages.push(format!("已添加插件 {} 的绑定", locked.plugin_name));
                }
            } else {
                messages.push(format!(
                    "插件 {} 的版本 {} 未找到，跳过同步",
                    locked.plugin_name, locked.version
                ));
            }
        } else {
            messages.push(format!(
                "插件 {} 未安装，无法从锁文件同步",
                locked.plugin_name
            ));
        }
    }

    Ok(messages)
}
