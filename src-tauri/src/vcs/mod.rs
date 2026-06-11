use serde::{Serialize, Deserialize};
use anyhow::{Result, Context, bail};
use std::path::Path;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

// ─── Data Models ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VcsType {
    Git,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VcsStatus {
    Clean,
    Modified,
    Untracked,
    Ahead,
    Behind,
    Diverged,
    NoRemote,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VcsInfo {
    pub vcs_type: VcsType,
    pub branch: String,
    pub remote: String,
    pub status: VcsStatus,
    pub ahead: u32,
    pub behind: u32,
    pub staged_files: u32,
    pub modified_files: u32,
    pub untracked_files: u32,
    pub last_commit_hash: String,
    pub last_commit_message: String,
    pub last_commit_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VcsCommit {
    pub hash: String,
    pub short_hash: String,
    pub message: String,
    pub author: String,
    pub date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VcsDiffSummary {
    pub added: u32,
    pub modified: u32,
    pub deleted: u32,
    pub files: Vec<VcsDiffFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VcsDiffFile {
    pub path: String,
    pub status: String,
    pub old_path: Option<String>,
}

impl Default for VcsInfo {
    fn default() -> Self {
        Self {
            vcs_type: VcsType::None,
            branch: String::new(),
            remote: String::new(),
            status: VcsStatus::Clean,
            ahead: 0,
            behind: 0,
            staged_files: 0,
            modified_files: 0,
            untracked_files: 0,
            last_commit_hash: String::new(),
            last_commit_message: String::new(),
            last_commit_date: None,
        }
    }
}

// ─── VCS Cache ───

struct CacheEntry {
    info: VcsInfo,
    fetched_at: Instant,
}

static VCS_CACHE: Mutex<Option<HashMap<String, CacheEntry>>> = Mutex::new(None);
const CACHE_TTL_SECS: u64 = 30;

fn get_cached(project_path: &str) -> Option<VcsInfo> {
    let cache = VCS_CACHE.lock().ok()?;
    let map = cache.as_ref()?;
    let entry = map.get(project_path)?;
    if entry.fetched_at.elapsed().as_secs() < CACHE_TTL_SECS {
        Some(entry.info.clone())
    } else {
        None
    }
}

fn set_cached(project_path: &str, info: &VcsInfo) {
    if let Ok(mut cache) = VCS_CACHE.lock() {
        let map = cache.get_or_insert_with(HashMap::new);
        map.insert(project_path.to_string(), CacheEntry {
            info: info.clone(),
            fetched_at: Instant::now(),
        });
    }
}

// ─── Core Functions ───

pub fn detect_vcs(project_path: &str) -> VcsType {
    let path = Path::new(project_path);
    if path.join(".git").exists() {
        VcsType::Git
    } else {
        VcsType::None
    }
}

pub fn get_vcs_info(project_path: &str) -> Result<VcsInfo> {
    if let Some(cached) = get_cached(project_path) {
        return Ok(cached);
    }

    let vcs_type = detect_vcs(project_path);
    if matches!(vcs_type, VcsType::None) {
        let info = VcsInfo::default();
        return Ok(info);
    }

    let repo = git2::Repository::open(project_path)
        .with_context(|| format!("无法打开 Git 仓库: {}", project_path))?;

    let mut info = VcsInfo {
        vcs_type: VcsType::Git,
        ..Default::default()
    };

    // Branch
    if let Ok(head) = repo.head() {
        if let Some(name) = head.shorthand() {
            info.branch = name.to_string();
        }
        if let Some(target) = head.target() {
            info.last_commit_hash = target.to_string();
            if let Ok(commit) = repo.find_commit(target) {
                info.last_commit_message = commit.message().unwrap_or("").to_string();
                let time = commit.time();
                let secs = time.seconds();
                let dt = chrono::DateTime::from_timestamp(secs, 0);
                info.last_commit_date = dt.map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string());
            }
        }
    }

    // Remote
    if let Ok(remote) = repo.find_remote("origin") {
        if let Some(url) = remote.url() {
            info.remote = url.to_string();
        }
    }

    // Status counts
    let statuses = match repo.statuses(Some(
        git2::StatusOptions::new()
            .include_untracked(true)
            .recurse_untracked_dirs(false)
    )) {
        Ok(s) => s,
        Err(_) => match repo.statuses(None) {
            Ok(s) => s,
            Err(_) => {
                // Cannot get statuses, return info with zero counts
                set_cached(project_path, &info);
                return Ok(info);
            }
        }
    };

    let mut staged = 0u32;
    let mut modified = 0u32;
    let mut untracked = 0u32;

    for entry in statuses.iter() {
        let s = entry.status();
        if s.is_index_new() || s.is_index_modified() || s.is_index_deleted() || s.is_index_renamed() {
            staged += 1;
        }
        if s.is_wt_modified() || s.is_wt_deleted() || s.is_wt_renamed() {
            modified += 1;
        }
        if s.is_wt_new() {
            untracked += 1;
        }
    }

    info.staged_files = staged;
    info.modified_files = modified;
    info.untracked_files = untracked;

    // Ahead/Behind
    let (ahead, behind) = compute_ahead_behind(&repo);
    info.ahead = ahead;
    info.behind = behind;

    // Determine overall status
    info.status = determine_status(&info);

    set_cached(project_path, &info);
    Ok(info)
}

fn compute_ahead_behind(repo: &git2::Repository) -> (u32, u32) {
    let head = match repo.head() {
        Ok(h) => h,
        Err(_) => return (0, 0),
    };
    let head_oid = match head.target() {
        Some(oid) => oid,
        None => return (0, 0),
    };

    let branch_name = match head.shorthand() {
        Some(name) => name,
        None => return (0, 0),
    };

    let remote_branch = format!("origin/{}", branch_name);
    let remote_ref = match repo.find_reference(&format!("refs/remotes/{}", remote_branch))
        .or_else(|_| repo.find_reference(&format!("refs/remotes/origin/HEAD")))
    {
        Ok(r) => r,
        Err(_) => return (0, 0),
    };

    let remote_oid = match remote_ref.target() {
        Some(oid) => oid,
        None => return (0, 0),
    };

    match repo.graph_ahead_behind(head_oid, remote_oid) {
        Ok((a, b)) => (a as u32, b as u32),
        Err(_) => (0, 0),
    }
}

fn determine_status(info: &VcsInfo) -> VcsStatus {
    if info.remote.is_empty() {
        return VcsStatus::NoRemote;
    }
    if info.ahead > 0 && info.behind > 0 {
        return VcsStatus::Diverged;
    }
    if info.ahead > 0 {
        return VcsStatus::Ahead;
    }
    if info.behind > 0 {
        return VcsStatus::Behind;
    }
    if info.staged_files > 0 || info.modified_files > 0 {
        return VcsStatus::Modified;
    }
    if info.untracked_files > 0 {
        return VcsStatus::Untracked;
    }
    VcsStatus::Clean
}

pub fn get_vcs_status(project_path: &str) -> Result<VcsStatus> {
    let info = get_vcs_info(project_path)?;
    Ok(info.status)
}

pub fn get_commit_history(project_path: &str, limit: u32) -> Result<Vec<VcsCommit>> {
    let repo = git2::Repository::open(project_path)
        .with_context(|| format!("无法打开 Git 仓库: {}", project_path))?;

    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;
    revwalk.set_sorting(git2::Sort::TIME)?;

    let mut commits = Vec::new();
    for oid in revwalk.take(limit as usize) {
        let oid = oid?;
        let commit = repo.find_commit(oid)?;
        let hash = commit.id().to_string();
        let short_hash = if hash.len() >= 7 {
            hash[..7].to_string()
        } else {
            hash.clone()
        };
        let message = commit.message().unwrap_or("").to_string();
        let author = commit.author().name().unwrap_or("未知").to_string();
        let time = commit.time();
        let secs = time.seconds();
        let dt = chrono::DateTime::from_timestamp(secs, 0);
        let date = dt
            .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_default();

        commits.push(VcsCommit {
            hash,
            short_hash,
            message,
            author,
            date,
        });
    }

    Ok(commits)
}

pub fn pull(project_path: &str) -> Result<String> {
    let repo = git2::Repository::open(project_path)
        .with_context(|| format!("无法打开 Git 仓库: {}", project_path))?;

    let branch_name = repo.head()
        .ok()
        .and_then(|h| h.shorthand().map(|s| s.to_string()))
        .unwrap_or_default();

    let mut remote = repo.find_remote("origin")
        .map_err(|_| anyhow::anyhow!("未找到 origin 远程仓库，请先配置远程仓库"))?;

    let refspec = if branch_name.is_empty() {
        "refs/heads/main:refs/heads/main".to_string()
    } else {
        format!("refs/heads/{0}:refs/heads/{0}", branch_name)
    };

    let mut callbacks = git2::RemoteCallbacks::new();
    callbacks.credentials(|_url, username_from_url, _allowed_types| {
        git2::Cred::default()
            .or_else(|_| {
                if let Some(username) = username_from_url {
                    git2::Cred::ssh_key_from_agent(username)
                } else {
                    git2::Cred::default()
                }
            })
    });

    let mut fetch_options = git2::FetchOptions::new();
    fetch_options.remote_callbacks(callbacks);

    remote.fetch(&[&refspec], Some(&mut fetch_options), None)
        .map_err(|e| anyhow::anyhow!("拉取失败: {}。请检查网络连接和凭据配置", e))?;

    let fetch_head = repo.find_reference("FETCH_HEAD")?;
    let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)?;

    let analysis = repo.merge_analysis(&[&fetch_commit])?;

    if analysis.0.is_up_to_date() {
        invalidate_cache(project_path);
        return Ok("已是最新".to_string());
    }

    if analysis.0.is_fast_forward() {
        let refname = format!("refs/heads/{}", branch_name);
        let mut reference = repo.find_reference(&refname)?;
        reference.set_target(fetch_commit.id(), "Fast-forward")?;
        repo.set_head(&refname)?;
        repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
        invalidate_cache(project_path);
        return Ok("快进合并成功".to_string());
    }

    // Attempt merge
    let head_commit = repo.reference_to_annotated_commit(&repo.head()?)?;
    let head_oid = head_commit.id();
    let fetch_oid = fetch_commit.id();

    let head_merge = repo.find_commit(head_oid)?;
    let fetch_merge = repo.find_commit(fetch_oid)?;

    let ancestor = repo.merge_base(head_oid, fetch_oid)
        .and_then(|oid| repo.find_commit(oid))
        .ok();

    let mut idx = match ancestor {
        Some(_anc) => repo.merge_commits(&head_merge, &fetch_merge, Some(&git2::MergeOptions::new()))?,
        None => repo.merge_commits(&head_merge, &fetch_merge, None)?,
    };

    if idx.has_conflicts() {
        repo.cleanup_state()?;
        bail!("合并冲突，请手动解决冲突后再拉取");
    }

    let result_tree = repo.find_tree(idx.write_tree_to(&repo)?)?;
    let sig = repo.signature()?;
    let _merge_commit = repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        &format!("Merge branch '{}' of origin", branch_name),
        &result_tree,
        &[&head_merge, &fetch_merge],
    )?;

    repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
    invalidate_cache(project_path);
    Ok("合并拉取成功".to_string())
}

pub fn push(project_path: &str) -> Result<String> {
    let repo = git2::Repository::open(project_path)
        .with_context(|| format!("无法打开 Git 仓库: {}", project_path))?;

    let branch_name = repo.head()
        .ok()
        .and_then(|h| h.shorthand().map(|s| s.to_string()))
        .unwrap_or_default();

    if branch_name.is_empty() {
        bail!("无法确定当前分支");
    }

    let mut remote = repo.find_remote("origin")
        .map_err(|_| anyhow::anyhow!("未找到 origin 远程仓库，请先配置远程仓库"))?;

    let refspec = format!("refs/heads/{0}:refs/heads/{0}", branch_name);

    let mut callbacks = git2::RemoteCallbacks::new();
    callbacks.credentials(|_url, username_from_url, _allowed_types| {
        git2::Cred::default()
            .or_else(|_| {
                if let Some(username) = username_from_url {
                    git2::Cred::ssh_key_from_agent(username)
                } else {
                    git2::Cred::default()
                }
            })
    });
    callbacks.push_update_reference(|refname, status| {
        if let Some(s) = status {
            eprintln!("推送引用 {} 失败: {}", refname, s);
        }
        Ok(())
    });

    let mut push_options = git2::PushOptions::new();
    push_options.remote_callbacks(callbacks);

    remote.push(&[&refspec], Some(&mut push_options))
        .map_err(|e| anyhow::anyhow!("推送失败: {}。请检查网络连接和凭据配置", e))?;

    invalidate_cache(project_path);
    Ok("推送成功".to_string())
}

pub fn commit(project_path: &str, message: &str, add_all: bool) -> Result<String> {
    let repo = git2::Repository::open(project_path)
        .with_context(|| format!("无法打开 Git 仓库: {}", project_path))?;

    let mut index = repo.index()?;

    if add_all {
        // Stage all changes when explicitly requested
        index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
        index.write()?;
    } else {
        // Only commit what's already staged; check if there's anything staged
        let statuses = repo.statuses(Some(
            git2::StatusOptions::new()
                .include_untracked(true)
                .recurse_untracked_dirs(false),
        ))?;

        let has_staged = statuses.iter().any(|e| {
            let s = e.status();
            s.is_index_new() || s.is_index_modified() || s.is_index_deleted() || s.is_index_renamed()
        });

        if !has_staged {
            let has_unstaged = statuses.iter().any(|e| {
                let s = e.status();
                s.is_wt_modified() || s.is_wt_deleted() || s.is_wt_renamed() || s.is_wt_new()
            });
            if has_unstaged {
                bail!("没有暂存的更改。请先暂存要提交的文件，或使用 --all 选项。");
            } else {
                bail!("没有可提交的更改。");
            }
        }
    }

    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;

    let head = repo.head()?;
    let parent_commit = repo.find_commit(head.target().unwrap())?;

    let sig = repo.signature()?;
    let commit_id = repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        message,
        &tree,
        &[&parent_commit],
    )?;

    invalidate_cache(project_path);
    Ok(format!("提交成功: {}", commit_id))
}

pub fn get_diff_summary(project_path: &str) -> Result<VcsDiffSummary> {
    let repo = git2::Repository::open(project_path)
        .with_context(|| format!("无法打开 Git 仓库: {}", project_path))?;

    let mut summary = VcsDiffSummary {
        added: 0,
        modified: 0,
        deleted: 0,
        files: Vec::new(),
    };

    let statuses = repo.statuses(Some(
        git2::StatusOptions::new()
            .include_untracked(true)
            .recurse_untracked_dirs(false),
    ))?;

    for entry in statuses.iter() {
        let s = entry.status();
        let path = entry.path().unwrap_or("").to_string();
        if path.is_empty() {
            continue;
        }

        let (status, old_path) = if s.is_index_new() || s.is_wt_new() {
            summary.added += 1;
            ("added".to_string(), None)
        } else if s.is_index_modified() || s.is_wt_modified() {
            summary.modified += 1;
            ("modified".to_string(), None)
        } else if s.is_index_deleted() || s.is_wt_deleted() {
            summary.deleted += 1;
            ("deleted".to_string(), None)
        } else if s.is_index_renamed() || s.is_wt_renamed() {
            summary.modified += 1;
            let old = entry.head_to_index()
                .or_else(|| entry.index_to_workdir())
                .and_then(|d| d.old_file().path().map(|p| p.to_string_lossy().to_string()));
            ("renamed".to_string(), old)
        } else {
            continue;
        };

        summary.files.push(VcsDiffFile {
            path,
            status,
            old_path,
        });
    }

    Ok(summary)
}

pub fn ensure_gitignore(project_path: &str, harbor_managed_paths: &[String]) -> Result<()> {
    if harbor_managed_paths.is_empty() {
        return Ok(());
    }

    let gitignore_path = Path::new(project_path).join(".gitignore");
    let mut existing_content = String::new();
    let mut existing_lines: Vec<String> = Vec::new();

    if gitignore_path.exists() {
        existing_content = std::fs::read_to_string(&gitignore_path)
            .with_context(|| "无法读取 .gitignore 文件")?;
        existing_lines = existing_content.lines().map(|l| l.to_string()).collect();
    }

    // Check if already managed
    let marker = "# Godot Harbor managed";
    if existing_content.contains(marker) {
        // Already has our section, update it
        return update_gitignore_section(&gitignore_path, &existing_lines, harbor_managed_paths, marker);
    }

    // Add new section
    let mut new_content = if existing_content.is_empty() {
        String::new()
    } else if !existing_content.ends_with('\n') {
        existing_content + "\n\n"
    } else {
        existing_content + "\n"
    };

    new_content.push_str(marker);
    new_content.push('\n');
    for path in harbor_managed_paths {
        let normalized = path.replace('\\', "/");
        if !existing_lines.iter().any(|l| l.trim() == normalized) {
            new_content.push_str(&normalized);
            new_content.push('\n');
        }
    }
    new_content.push_str("# End of Godot Harbor managed\n");

    std::fs::write(&gitignore_path, new_content)
        .with_context(|| "无法写入 .gitignore 文件")?;

    Ok(())
}

fn update_gitignore_section(
    gitignore_path: &Path,
    existing_lines: &[String],
    harbor_managed_paths: &[String],
    marker: &str,
) -> Result<()> {
    let end_marker = "# End of Godot Harbor managed";
    let mut new_lines: Vec<String> = Vec::new();
    let mut in_section = false;
    let mut section_inserted = false;

    for line in existing_lines {
        if line.trim() == marker {
            in_section = true;
            new_lines.push(line.clone());
            // Add all managed paths
            for path in harbor_managed_paths {
                let normalized = path.replace('\\', "/");
                if !existing_lines.iter().any(|l| l.trim() == normalized) && !new_lines.iter().any(|l| l.trim() == normalized) {
                    new_lines.push(normalized);
                }
            }
            section_inserted = true;
            continue;
        }
        if in_section && line.trim() == end_marker {
            in_section = false;
            new_lines.push(line.clone());
            continue;
        }
        if in_section {
            // Skip old managed paths - they'll be replaced
            continue;
        }
        new_lines.push(line.clone());
    }

    // If marker was found but end marker wasn't, add end marker
    if section_inserted && !new_lines.iter().any(|l| l.trim() == end_marker) {
        new_lines.push(end_marker.to_string());
    }

    // If no section was found (shouldn't happen), add one
    if !section_inserted {
        new_lines.push(String::new());
        new_lines.push(marker.to_string());
        for path in harbor_managed_paths {
            let normalized = path.replace('\\', "/");
            if !new_lines.iter().any(|l| l.trim() == normalized) {
                new_lines.push(normalized);
            }
        }
        new_lines.push(end_marker.to_string());
    }

    let content = new_lines.join("\n");
    std::fs::write(gitignore_path, content)
        .with_context(|| "无法写入 .gitignore 文件")?;

    Ok(())
}

fn invalidate_cache(project_path: &str) {
    if let Ok(mut cache) = VCS_CACHE.lock() {
        if let Some(map) = cache.as_mut() {
            map.remove(project_path);
        }
    }
}
