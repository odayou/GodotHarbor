use std::path::PathBuf;
use std::fs;

use anyhow::{Context, Result, anyhow, bail};
use clap::{Parser, Subcommand};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use serde::Serialize;

use crate::models::*;
use crate::storage::Storage;
use crate::scanner::ProjectScanner;
use crate::plugin_manager::PluginManager;
use crate::linker::Linker;
use crate::engine::EngineManager;
use crate::harbor_config;

// ============================================================================
// CLI Definition
// ============================================================================

#[derive(Parser)]
#[command(name = "harbor")]
#[command(version)]
#[command(about = "Godot Harbor - Godot 插件与项目管理工具的命令行界面")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// 输出格式 (human 或 json)
    #[arg(long, global = true, default_value = "human")]
    pub format: String,

    /// 自定义数据目录路径
    #[arg(long, global = true)]
    pub data_dir: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 显示全局状态概览
    Status,

    /// 项目管理
    Projects {
        #[command(subcommand)]
        command: ProjectCommands,
    },

    /// 插件管理
    Plugins {
        #[command(subcommand)]
        command: PluginCommands,
    },

    /// 绑定插件到项目
    Bind {
        /// 项目名称（支持模糊匹配）
        project: String,
        /// 插件名称（支持模糊匹配）
        plugin: String,
        /// 指定版本
        #[arg(long)]
        version: Option<String>,
        /// 指定单元名称
        #[arg(long)]
        unit: Option<String>,
    },

    /// 解绑项目中的插件
    Unbind {
        /// 项目名称（支持模糊匹配）
        project: String,
        /// 插件名称（支持模糊匹配）
        plugin: String,
    },

    /// 应用绑定变更
    Apply {
        /// 指定项目名称，不指定则应用所有项目的变更
        #[arg(long)]
        project: Option<String>,
    },

    /// 引擎管理
    Engines {
        #[command(subcommand)]
        command: EngineCommands,
    },

    /// 同步 .harbor.yml 配置
    Sync {
        /// 指定项目名称，不指定则同步所有项目
        #[arg(long)]
        project: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum ProjectCommands {
    /// 列出所有项目
    List,

    /// 扫描项目
    Scan {
        /// 指定扫描目录
        #[arg(long)]
        dir: Option<String>,
    },

    /// 使用引擎打开项目
    Open {
        /// 项目名称（支持模糊匹配）
        name: String,
    },
}

#[derive(Subcommand)]
pub enum PluginCommands {
    /// 列出所有插件
    List,

    /// 导入插件
    Import {
        /// 插件来源（本地路径、Git URL 或 HTTP URL）
        source: String,
    },

    /// 更新插件
    Update {
        /// 插件名称（不指定则需配合 --all）
        name: Option<String>,
        /// 更新所有插件
        #[arg(long)]
        all: bool,
    },
}

#[derive(Subcommand)]
pub enum EngineCommands {
    /// 列出引擎
    List,

    /// 下载并安装引擎
    Install {
        /// 引擎版本号
        version: String,
        /// 镜像 ID
        #[arg(long)]
        mirror: Option<String>,
        /// 引擎变体 (standard 或 mono)
        #[arg(long, default_value = "standard")]
        variant: String,
    },
}

// ============================================================================
// CliContext - CLI execution context
// ============================================================================

pub struct CliContext {
    storage: Storage,
    data_dir: PathBuf,
    output_format: String,
}

impl CliContext {
    pub fn new(data_dir: PathBuf, output_format: String) -> Self {
        let storage = Storage::new(data_dir.clone());
        Self { storage, data_dir, output_format }
    }

    pub fn resolve_data_dir(cli_data_dir: Option<&str>) -> Result<PathBuf> {
        if let Some(dir) = cli_data_dir {
            let path = PathBuf::from(dir);
            fs::create_dir_all(&path)
                .with_context(|| format!("无法创建数据目录: {}", dir))?;
            return Ok(path);
        }

        // Try app data dir first (same logic as lib.rs setup)
        let app_data_dir = dirs::data_dir()
            .context("无法确定应用数据目录")?;
        let config_dir = app_data_dir.join("com.godot-harbor.app");
        if config_dir.exists() {
            let config_storage = Storage::new(config_dir.clone());
            let settings: Settings = config_storage.load_or_default("settings.json");
            if !settings.custom_data_dir.is_empty() {
                return Ok(PathBuf::from(&settings.custom_data_dir));
            }
            if !settings.data_dir_initialized {
                let root = get_app_root_dir();
                let data = root.join("GodotHarborData");
                return Ok(data);
            }
            return Ok(config_dir);
        }

        // Fallback: try default data dir
        let default_data = app_data_dir.join("com.godot-harbor.app");
        fs::create_dir_all(&default_data)
            .with_context(|| "无法创建默认数据目录")?;
        Ok(default_data)
    }

    fn load_settings(&self) -> Settings {
        self.storage.load_or_default("settings.json")
    }

    fn load_projects(&self) -> Vec<Project> {
        self.storage.load_or_default("projects.json")
    }

    fn save_projects(&self, projects: &[Project]) -> Result<()> {
        self.storage.save("projects.json", &projects)
            .map_err(|e| anyhow!("保存项目列表失败: {}", e))
    }

    fn load_plugins(&self) -> Vec<Plugin> {
        self.storage.load_or_default("plugins.json")
    }

    fn save_plugins(&self, plugins: &[Plugin]) -> Result<()> {
        self.storage.save("plugins.json", &plugins)
            .map_err(|e| anyhow!("保存插件列表失败: {}", e))
    }

    fn load_bindings(&self) -> Vec<ProjectBinding> {
        self.storage.load_or_default("bindings.json")
    }

    fn save_bindings(&self, bindings: &[ProjectBinding]) -> Result<()> {
        self.storage.save("bindings.json", &bindings)
            .map_err(|e| anyhow!("保存绑定列表失败: {}", e))
    }

    fn load_engines(&self) -> Vec<Engine> {
        self.storage.load_or_default("engines.json")
    }

    fn save_engines(&self, engines: &[Engine]) -> Result<()> {
        self.storage.save("engines.json", &engines)
            .map_err(|e| anyhow!("保存引擎列表失败: {}", e))
    }

    fn get_plugin_manager(&self) -> PluginManager {
        PluginManager::new(self.data_dir.join("plugins"))
    }

    fn get_linker(&self) -> Linker {
        let settings = self.load_settings();
        Linker::new(settings.mount_strategy.clone())
    }

    fn is_json_output(&self) -> bool {
        self.output_format == "json"
    }

    fn acquire_lock(&self) -> Result<std::fs::File> {
        let lock_path = self.data_dir.join(".harbor.lock");
        let lock_file = fs::File::create(&lock_path)
            .with_context(|| "无法创建锁文件")?;
        fs2::FileExt::lock_exclusive(&lock_file)
            .with_context(|| "无法获取数据目录锁，可能 GUI 应用正在运行")?;
        Ok(lock_file)
    }
}

// ============================================================================
// Output helpers
// ============================================================================

fn print_json<T: Serialize>(data: &T) -> Result<()> {
    let json = serde_json::to_string_pretty(data)
        .context("序列化 JSON 失败")?;
    println!("{}", json);
    Ok(())
}

fn fuzzy_match<'a, T>(items: &'a [T], name: &str, get_name: impl Fn(&T) -> &str) -> Vec<&'a T> {
    let lower = name.to_lowercase();
    items.iter().filter(|item| {
        let item_name = get_name(item).to_lowercase();
        item_name.contains(&lower) || lower.contains(&item_name)
    }).collect()
}

fn find_by_name<'a, T>(
    items: &'a [T],
    name: &str,
    get_name: impl Fn(&T) -> &str,
    item_type: &str,
) -> Result<&'a T> {
    // Exact match first
    if let Some(item) = items.iter().find(|item| get_name(item) == name) {
        return Ok(item);
    }
    // Case-insensitive match
    if let Some(item) = items.iter().find(|item| get_name(item).to_lowercase() == name.to_lowercase()) {
        return Ok(item);
    }
    // Fuzzy match
    let matches = fuzzy_match(items, name, &get_name);
    match matches.len() {
        0 => bail!("未找到{}: {}", item_type, name),
        1 => Ok(matches[0]),
        _ => {
            let names: Vec<String> = matches.iter().map(|m| get_name(m).to_string()).collect();
            bail!(
                "{}名称 '{}' 匹配到多个结果，请更精确地指定:\n  {}",
                item_type,
                name,
                names.join("\n  ")
            )
        }
    }
}

// ============================================================================
// Command implementations
// ============================================================================

pub fn run(cli: Cli) -> Result<()> {
    let data_dir = CliContext::resolve_data_dir(cli.data_dir.as_deref())?;
    let ctx = CliContext::new(data_dir, cli.format);

    match cli.command {
        Commands::Status => cmd_status(&ctx),
        Commands::Projects { command } => match command {
            ProjectCommands::List => cmd_projects_list(&ctx),
            ProjectCommands::Scan { dir } => cmd_projects_scan(&ctx, dir),
            ProjectCommands::Open { name } => cmd_projects_open(&ctx, &name),
        },
        Commands::Plugins { command } => match command {
            PluginCommands::List => cmd_plugins_list(&ctx),
            PluginCommands::Import { source } => cmd_plugins_import(&ctx, &source),
            PluginCommands::Update { name, all } => cmd_plugins_update(&ctx, name.as_deref(), all),
        },
        Commands::Bind { project, plugin, version, unit } => cmd_bind(&ctx, &project, &plugin, version, unit),
        Commands::Unbind { project, plugin } => cmd_unbind(&ctx, &project, &plugin),
        Commands::Apply { project } => cmd_apply(&ctx, project.as_deref()),
        Commands::Engines { command } => match command {
            EngineCommands::List => cmd_engines_list(&ctx),
            EngineCommands::Install { version, mirror, variant } => cmd_engines_install(&ctx, &version, mirror, &variant),
        },
        Commands::Sync { project } => cmd_sync(&ctx, project.as_deref()),
    }
}

// ----------------------------------------------------------------------------
// harbor status
// ----------------------------------------------------------------------------

#[derive(Serialize)]
struct StatusOutput {
    project_count: usize,
    plugin_count: usize,
    binding_count: usize,
    engine_count: usize,
    drift_warnings: usize,
}

fn cmd_status(ctx: &CliContext) -> Result<()> {
    let projects = ctx.load_projects();
    let plugins = ctx.load_plugins();
    let bindings = ctx.load_bindings();
    let engines = ctx.load_engines();

    // Check drifts
    let mut drift_count = 0;
    for project in &projects {
        let config_path = harbor_config::get_harbor_config_path(&project.path);
        if config_path.exists() {
            drift_count += 1;
        }
    }

    let output = StatusOutput {
        project_count: projects.len(),
        plugin_count: plugins.len(),
        binding_count: bindings.len(),
        engine_count: engines.len(),
        drift_warnings: drift_count,
    };

    if ctx.is_json_output() {
        return print_json(&output);
    }

    println!("{}", style("Godot Harbor 状态概览").bold());
    println!();
    println!("  {}  项目: {}", style("●").cyan(), output.project_count);
    println!("  {}  插件: {}", style("●").green(), output.plugin_count);
    println!("  {}  绑定: {}", style("●").yellow(), output.binding_count);
    println!("  {}  引擎: {}", style("●").magenta(), output.engine_count);
    if output.drift_warnings > 0 {
        println!("  {}  配置文件: {} 个项目含 .harbor.yml", style("⚠").yellow(), output.drift_warnings);
    }
    println!();
    println!("  数据目录: {}", ctx.data_dir.to_string_lossy());

    Ok(())
}

// ----------------------------------------------------------------------------
// harbor projects list
// ----------------------------------------------------------------------------

#[derive(Serialize)]
struct ProjectListOutput {
    projects: Vec<ProjectItem>,
}

#[derive(Serialize)]
struct ProjectItem {
    name: String,
    path: String,
    godot_version: String,
    status: String,
    group: String,
}

fn cmd_projects_list(ctx: &CliContext) -> Result<()> {
    let projects = ctx.load_projects();

    let items: Vec<ProjectItem> = projects.iter().map(|p| ProjectItem {
        name: p.name.clone(),
        path: p.path.clone(),
        godot_version: p.godot_version.clone(),
        status: format!("{:?}", p.status),
        group: p.group.clone(),
    }).collect();

    if ctx.is_json_output() {
        return print_json(&ProjectListOutput { projects: items });
    }

    if items.is_empty() {
        println!("暂无项目，使用 {} 扫描项目", style("harbor projects scan").cyan());
        return Ok(());
    }

    println!("{}", style("项目列表").bold());
    println!();
    for item in &items {
        let status_icon = match item.status.as_str() {
            "Ready" => style("✓").green(),
            "Warning" => style("⚠").yellow(),
            "MissingSource" => style("✗").red(),
            _ => style("?").dim(),
        };
        println!("  {} {} ({})", status_icon, style(&item.name).bold(), item.godot_version);
        println!("    路径: {}", item.path);
        if !item.group.is_empty() {
            println!("    分组: {}", item.group);
        }
    }
    println!();
    println!("共 {} 个项目", items.len());

    Ok(())
}

// ----------------------------------------------------------------------------
// harbor projects scan
// ----------------------------------------------------------------------------

fn cmd_projects_scan(ctx: &CliContext, dir: Option<String>) -> Result<()> {
    let _lock = ctx.acquire_lock()?;

    let dirs = match dir {
        Some(d) => vec![d],
        None => {
            let settings = ctx.load_settings();
            if settings.scan_directories.is_empty() {
                get_default_scan_dirs()
            } else {
                settings.scan_directories
            }
        }
    };

    if dirs.is_empty() {
        bail!("未指定扫描目录，请使用 --dir 参数或在设置中配置扫描目录");
    }

    if !ctx.is_json_output() {
        println!("正在扫描项目...");
        for d in &dirs {
            println!("  扫描目录: {}", d);
        }
    }

    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner()
        .template("{spinner} {msg}")
        .unwrap());
    pb.set_message("扫描中...");
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    let all_projects = ProjectScanner::scan_directories_parallel(&dirs)
        .map_err(|e| anyhow!("扫描失败: {}", e))?;

    pb.finish_with_message("扫描完成");

    let mut existing = ctx.load_projects();
    let mut added = 0;
    let mut updated = 0;

    for project in &all_projects {
        if let Some(idx) = existing.iter().position(|p| {
            p.path.replace('\\', "/").trim_end_matches('/').to_lowercase()
                == project.path.replace('\\', "/").trim_end_matches('/').to_lowercase()
        }) {
            existing[idx].name = project.name.clone();
            existing[idx].godot_version = project.godot_version.clone();
            updated += 1;
        } else {
            existing.push(project.clone());
            added += 1;
        }
    }

    ctx.save_projects(&existing)?;

    if ctx.is_json_output() {
        print_json(&serde_json::json!({
            "total": all_projects.len(),
            "added": added,
            "updated": updated,
            "projects": all_projects,
        }))?;
    } else {
        println!();
        println!("扫描完成: 发现 {} 个项目 (新增 {}, 更新 {})", all_projects.len(), added, updated);
    }

    Ok(())
}

// ----------------------------------------------------------------------------
// harbor projects open
// ----------------------------------------------------------------------------

fn cmd_projects_open(ctx: &CliContext, name: &str) -> Result<()> {
    let projects = ctx.load_projects();
    let project = find_by_name(&projects, name, |p| &p.name, "项目")?;

    let engines = ctx.load_engines();

    // Find matching engine
    let matched = find_matching_engine(&engines, &project.godot_version);

    let engine = match matched {
        Some(e) => e,
        None => bail!(
            "未找到匹配 Godot {} 的引擎，请先注册或下载引擎",
            project.godot_version
        ),
    };

    let exe_path = EngineManager::find_executable_in_dir(std::path::Path::new(&engine.path))
        .ok_or_else(|| anyhow!("未找到引擎可执行文件: {}", engine.path))?;

    let mut cmd = detached_cmd(&exe_path);
    cmd.arg("--path").arg(&project.path).arg("-e");

    cmd.spawn()
        .with_context(|| format!("启动引擎失败: {}", exe_path.to_string_lossy()))?;

    if !ctx.is_json_output() {
        println!("已使用 {} 打开项目: {}", style(&engine.name).bold(), style(&project.name).cyan());
    }

    Ok(())
}

fn find_matching_engine(engines: &[Engine], godot_version: &str) -> Option<Engine> {
    let project_parts: Vec<&str> = godot_version.split('.').collect();
    let project_major = project_parts.first().and_then(|s| s.parse::<u32>().ok());
    let project_minor = project_parts.get(1).and_then(|s| s.parse::<u32>().ok());

    let mut best: Option<(Engine, u8)> = None;

    for engine in engines {
        let engine_parts: Vec<&str> = engine.version.split('.').collect();
        let engine_major = engine_parts.first().and_then(|s| s.parse::<u32>().ok());
        let engine_minor = engine_parts.get(1).and_then(|s| s.parse::<u32>().ok());

        let level = match (project_major, project_minor, engine_major, engine_minor) {
            (Some(pm), Some(pn), Some(em), Some(en)) if pm == em && pn == en => 3,
            (Some(pm), _, Some(em), _) if pm == em => 2,
            _ => 0,
        };

        if level > 0 {
            if best.is_none() || level > best.as_ref().unwrap().1 {
                best = Some((engine.clone(), level));
            }
        }
    }

    best.map(|(e, _)| e)
}

// ----------------------------------------------------------------------------
// harbor plugins list
// ----------------------------------------------------------------------------

#[derive(Serialize)]
struct PluginListOutput {
    plugins: Vec<PluginItem>,
}

#[derive(Serialize)]
struct PluginItem {
    name: String,
    source_type: String,
    compatibility: String,
    version_count: usize,
    is_favorite: bool,
}

fn cmd_plugins_list(ctx: &CliContext) -> Result<()> {
    let plugins = ctx.load_plugins();

    let items: Vec<PluginItem> = plugins.iter().map(|p| PluginItem {
        name: p.name.clone(),
        source_type: format!("{:?}", p.source.source_type),
        compatibility: format!("{:?}", p.compatibility),
        version_count: p.versions.len(),
        is_favorite: p.is_favorite,
    }).collect();

    if ctx.is_json_output() {
        return print_json(&PluginListOutput { plugins: items });
    }

    if items.is_empty() {
        println!("暂无插件，使用 {} 导入插件", style("harbor plugins import <source>").cyan());
        return Ok(());
    }

    println!("{}", style("插件列表").bold());
    println!();
    for item in &items {
        let fav = if item.is_favorite { style("★").yellow().to_string() } else { " ".to_string() };
        println!(
            "  {} {} [{}] ({} 个版本, 兼容: {})",
            fav,
            style(&item.name).bold(),
            style(&item.source_type).dim(),
            item.version_count,
            item.compatibility,
        );
    }
    println!();
    println!("共 {} 个插件", items.len());

    Ok(())
}

// ----------------------------------------------------------------------------
// harbor plugins import
// ----------------------------------------------------------------------------

fn cmd_plugins_import(ctx: &CliContext, source: &str) -> Result<()> {
    let _lock = ctx.acquire_lock()?;

    let source_type = detect_source_type(source);

    if !ctx.is_json_output() {
        println!("正在导入插件...");
        println!("  来源: {}", source);
        println!("  类型: {}", source_type);
    }

    let manager = ctx.get_plugin_manager();

    let new_plugin = match source_type {
        "local" => {
            let pb = ProgressBar::new_spinner();
            pb.set_style(ProgressStyle::default_spinner().template("{spinner} {msg}").unwrap());
            pb.set_message("复制插件文件...");
            pb.enable_steady_tick(std::time::Duration::from_millis(100));

            let result = manager.import_from_local(source)
                .map_err(|e| anyhow!("导入本地插件失败: {}", e))?;

            pb.finish_with_message("导入完成");
            result
        }
        "git" => {
            // CLI cannot use AppHandle for git import, so we do a simplified version
            // that clones without progress events
            let pb = ProgressBar::new_spinner();
            pb.set_style(ProgressStyle::default_spinner().template("{spinner} {msg}").unwrap());
            pb.set_message("克隆 Git 仓库...");
            pb.enable_steady_tick(std::time::Duration::from_millis(100));

            let result = import_plugin_from_git_cli(&manager, source, None)
                .map_err(|e| anyhow!("导入 Git 插件失败: {}", e))?;

            pb.finish_with_message("导入完成");
            result
        }
        "url" => {
            let pb = ProgressBar::new_spinner();
            pb.set_style(ProgressStyle::default_spinner().template("{spinner} {msg}").unwrap());
            pb.set_message("下载插件...");
            pb.enable_steady_tick(std::time::Duration::from_millis(100));

            let result = import_plugin_from_url_cli(&manager, source)
                .map_err(|e| anyhow!("导入 URL 插件失败: {}", e))?;

            pb.finish_with_message("导入完成");
            result
        }
        _ => bail!("无法识别的来源类型"),
    };

    // Upsert plugin
    let mut plugins = ctx.load_plugins();
    let existing_idx = plugins.iter().position(|p| p.source.url == new_plugin.source.url);
    if let Some(idx) = existing_idx {
        plugins[idx].versions.extend(new_plugin.versions.clone());
        if !new_plugin.content_hash.is_empty() {
            plugins[idx].content_hash = new_plugin.content_hash.clone();
        }
        let updated = plugins[idx].clone();
        ctx.save_plugins(&plugins)?;
        if ctx.is_json_output() {
            print_json(&updated)?;
        } else {
            println!("已为插件 {} 添加新版本", style(&updated.name).bold());
        }
    } else {
        plugins.push(new_plugin.clone());
        ctx.save_plugins(&plugins)?;
        if ctx.is_json_output() {
            print_json(&new_plugin)?;
        } else {
            println!("已导入插件: {}", style(&new_plugin.name).bold());
        }
    }

    Ok(())
}

fn detect_source_type(source: &str) -> &'static str {
    let lower = source.to_lowercase();
    if lower.starts_with("http://") || lower.starts_with("https://") {
        if lower.contains("github.com") || lower.contains("gitlab.com") || lower.ends_with(".git") {
            "git"
        } else {
            "url"
        }
    } else if lower.ends_with(".git") {
        "git"
    } else {
        let path = std::path::Path::new(source);
        if path.exists() {
            "local"
        } else if lower.contains("github.com") || lower.contains("gitlab.com") {
            "git"
        } else {
            "local"
        }
    }
}

/// Import plugin from git without AppHandle (CLI version)
fn import_plugin_from_git_cli(manager: &PluginManager, git_url: &str, git_ref: Option<&str>) -> Result<Plugin> {
    let plugin_name = git_url
        .split('/')
        .last()
        .unwrap_or("unknown")
        .trim_end_matches(".git")
        .to_string();

    let plugin_source = PluginSource {
        source_type: SourceType::Git,
        url: git_url.to_string(),
        git_ref: git_ref.map(|r| r.to_string()).unwrap_or_default(),
        imported_at: chrono::Utc::now(),
    };

    let mut plugin = Plugin::new(plugin_name.clone(), plugin_source);

    let version_id = uuid::Uuid::new_v4().to_string();
    let plugins_dir = manager.plugins_dir();
    let version_dir = plugins_dir.join(&plugin.plugin_id).join(&version_id);
    let payload_dir = version_dir.join("payload");
    let git_store_dir = version_dir.join("git");

    fs::create_dir_all(&payload_dir)
        .context("无法创建版本目录")?;

    let mut builder = git2::build::RepoBuilder::new();
    if let Some(git_ref) = git_ref {
        builder.branch(git_ref);
    }

    if let Err(e) = builder.clone(git_url, &payload_dir) {
        let _ = fs::remove_dir_all(&version_dir);
        let err_msg = format!("{}", e);
        let user_msg = if err_msg.contains("401") || err_msg.contains("status code: 401") {
            "仓库需要认证或为私有仓库，无法访问"
        } else if err_msg.contains("404") || err_msg.contains("status code: 404") {
            "仓库不存在或地址错误"
        } else if err_msg.contains("timed out") || err_msg.contains("connection") {
            "网络连接失败，请检查网络"
        } else {
            return Err(anyhow!("克隆仓库失败: {}", e));
        };
        bail!("{}", user_msg);
    }

    // Backup .git and clean up
    let git_dir = payload_dir.join(".git");
    if git_dir.exists() {
        if !git_store_dir.exists() {
            fs::create_dir_all(&git_store_dir).ok();
        }
        let actual_ref = git2::Repository::open(&payload_dir)
            .ok()
            .and_then(|repo| {
                let head = repo.head().ok()?;
                head.target().map(|oid| oid.to_string())
            })
            .unwrap_or_default();

        if let Err(e) = crate::utils::copy_dir_all(&git_dir, &git_store_dir) {
            eprintln!("Warning: failed to backup .git directory: {}", e);
        }
        fs::remove_dir_all(&git_dir).ok();

        if !actual_ref.is_empty() {
            plugin.source.git_ref = actual_ref;
        }
    }

    manager.finalize_import_cli(&mut plugin, &payload_dir, &version_id, &plugin_name)?;

    Ok(plugin)
}

/// Import plugin from URL without AppHandle (CLI version)
fn import_plugin_from_url_cli(manager: &PluginManager, url: &str) -> Result<Plugin> {
    let url_path = url.split('?').next().unwrap_or(url);
    let file_name = url_path.split('/').last().unwrap_or("plugin").to_string();

    let plugin_name = if file_name.contains('.') {
        file_name.rsplitn(2, '.').last().unwrap_or("plugin").to_string()
    } else {
        file_name.clone()
    };

    let plugin_source = PluginSource {
        source_type: SourceType::Url,
        url: url.to_string(),
        git_ref: String::new(),
        imported_at: chrono::Utc::now(),
    };

    let mut plugin = Plugin::new(plugin_name.clone(), plugin_source);

    let version_id = uuid::Uuid::new_v4().to_string();
    let plugins_dir = manager.plugins_dir();
    let version_dir = plugins_dir.join(&plugin.plugin_id).join(&version_id);
    let payload_dir = version_dir.join("payload");
    let download_dir = version_dir.join("download");
    let archive_path = download_dir.join(&file_name);

    fs::create_dir_all(&version_dir)
        .context("无法创建版本目录")?;

    // Download
    let rt = tokio::runtime::Runtime::new()
        .context("无法创建 tokio 运行时")?;
    rt.block_on(async {
        let client = crate::utils::create_http_client(None)
            .map_err(|e| anyhow::anyhow!(e))?;
        let resp = client.get(url).send().await
            .map_err(|e| anyhow::anyhow!("下载失败: {}", e))?;
        if !resp.status().is_success() {
            return Err(anyhow::anyhow!("下载失败，HTTP 状态码: {}", resp.status()));
        }
        fs::create_dir_all(&download_dir)
            .context("无法创建下载目录")?;
        let bytes = resp.bytes().await
            .map_err(|e| anyhow::anyhow!("读取响应内容失败: {}", e))?;
        fs::write(&archive_path, &bytes)
            .context("无法写入下载文件")?;
        Ok(())
    }).map_err(|e: anyhow::Error| {
        let _ = fs::remove_dir_all(&version_dir);
        e
    })?;

    let is_archive = file_name.ends_with(".zip")
        || file_name.ends_with(".tar.gz")
        || file_name.ends_with(".tgz")
        || file_name.ends_with(".tar.bz2")
        || file_name.ends_with(".gz");

    if is_archive {
        fs::create_dir_all(&payload_dir)
            .context("无法创建目标目录")?;

        let extract_result = if file_name.ends_with(".zip") {
            PluginManager::extract_zip_cli(&archive_path, &payload_dir)
        } else {
            PluginManager::extract_tar_cli(&archive_path, &payload_dir)
        };

        let _ = fs::remove_dir_all(&download_dir);

        if let Err(e) = extract_result {
            let _ = fs::remove_dir_all(&version_dir);
            bail!("解压插件文件失败: {}", e);
        }

        let actual_payload = if let Some(single_dir) = PluginManager::find_single_subdir_cli(&payload_dir) {
            single_dir
        } else {
            payload_dir.clone()
        };

        manager.finalize_import_cli(&mut plugin, &actual_payload, &version_id, &plugin_name)?;
    } else {
        let _ = fs::remove_dir_all(&download_dir);
        let _ = fs::remove_dir_all(&version_dir);
        bail!("不支持的文件格式，请提供 .zip 或 .tar.gz 压缩包");
    }

    Ok(plugin)
}

// ----------------------------------------------------------------------------
// harbor plugins update
// ----------------------------------------------------------------------------

fn cmd_plugins_update(ctx: &CliContext, name: Option<&str>, all: bool) -> Result<()> {
    let _lock = ctx.acquire_lock()?;

    let plugins = ctx.load_plugins();

    let targets: Vec<&Plugin> = if all {
        plugins.iter().filter(|p| p.source.source_type == SourceType::Git).collect()
    } else if let Some(name) = name {
        let plugin = find_by_name(&plugins, name, |p| &p.name, "插件")?;
        vec![plugin]
    } else {
        bail!("请指定插件名称或使用 --all 更新所有插件");
    };

    if targets.is_empty() {
        println!("没有可更新的插件");
        return Ok(());
    }

    let mut updated_count = 0;
    let mut failed_count = 0;
    let mut errors = Vec::new();

    for plugin in &targets {
        if !ctx.is_json_output() {
            println!("正在更新插件: {}...", style(&plugin.name).bold());
        }

        match update_git_plugin(ctx, plugin) {
            Ok(_) => {
                updated_count += 1;
                if !ctx.is_json_output() {
                    println!("  {} 已更新", style("✓").green());
                }
            }
            Err(e) => {
                failed_count += 1;
                errors.push(format!("{}: {}", plugin.name, e));
                if !ctx.is_json_output() {
                    println!("  {} 更新失败: {}", style("✗").red(), e);
                }
            }
        }
    }

    if ctx.is_json_output() {
        print_json(&serde_json::json!({
            "updated": updated_count,
            "failed": failed_count,
            "errors": errors,
        }))?;
    } else {
        println!();
        println!("更新完成: {} 成功, {} 失败", updated_count, failed_count);
    }

    Ok(())
}

fn update_git_plugin(ctx: &CliContext, plugin: &Plugin) -> Result<()> {
    if plugin.source.source_type != SourceType::Git {
        bail!("仅支持更新 Git 来源的插件");
    }

    let manager = ctx.get_plugin_manager();
    let new_plugin = import_plugin_from_git_cli(&manager, &plugin.source.url, None)?;

    let mut plugins = ctx.load_plugins();
    if let Some(idx) = plugins.iter().position(|p| p.plugin_id == plugin.plugin_id) {
        plugins[idx].versions.extend(new_plugin.versions.clone());
        if !new_plugin.content_hash.is_empty() {
            plugins[idx].content_hash = new_plugin.content_hash.clone();
        }
        plugins[idx].updated_at = chrono::Utc::now();
        ctx.save_plugins(&plugins)?;
    }

    Ok(())
}

// ----------------------------------------------------------------------------
// harbor bind
// ----------------------------------------------------------------------------

fn cmd_bind(
    ctx: &CliContext,
    project_name: &str,
    plugin_name: &str,
    version: Option<String>,
    unit: Option<String>,
) -> Result<()> {
    let _lock = ctx.acquire_lock()?;

    let projects = ctx.load_projects();
    let plugins = ctx.load_plugins();
    let mut bindings = ctx.load_bindings();

    let project = find_by_name(&projects, project_name, |p| &p.name, "项目")?;
    let plugin = find_by_name(&plugins, plugin_name, |p| &p.name, "插件")?;

    // Select version
    let plugin_version = if let Some(ref ver) = version {
        plugin.versions.iter().find(|v| v.version == *ver)
            .ok_or_else(|| anyhow!("未找到版本 '{}'，可用版本: {}", ver,
                plugin.versions.iter().map(|v| v.version.as_str()).collect::<Vec<_>>().join(", ")))?
    } else {
        plugin.versions.first()
            .ok_or_else(|| anyhow!("插件没有可用版本"))?
    };

    // Select unit
    let unit = if let Some(ref unit_name) = unit {
        plugin_version.units.iter().find(|u| u.name == *unit_name || u.dir_name == *unit_name)
            .ok_or_else(|| anyhow!("未找到单元 '{}'，可用单元: {}", unit_name,
                plugin_version.units.iter().map(|u| u.name.as_str()).collect::<Vec<_>>().join(", ")))?
    } else {
        plugin_version.units.first()
            .ok_or_else(|| anyhow!("插件版本没有可用单元"))?
    };

    let mount_path = if unit.subdirectory.is_empty() {
        format!("addons/{}", unit.dir_name)
    } else {
        format!("addons/{}", unit.dir_name)
    };

    // Check if already bound
    let existing = bindings.iter().find(|b| {
        b.project_id == project.project_id && b.plugin_id == plugin.plugin_id
    });

    if let Some(existing) = existing {
        if existing.version_id == plugin_version.version_id && existing.unit_id == unit.unit_id {
            bail!("插件已绑定到该项目（相同版本和单元）");
        }
        // Remove old binding
        bindings.retain(|b| !(b.project_id == project.project_id && b.plugin_id == plugin.plugin_id));
    }

    let binding = ProjectBinding::new(
        project.project_id.clone(),
        plugin.plugin_id.clone(),
        plugin_version.version_id.clone(),
        unit.unit_id.clone(),
        mount_path.clone(),
        unit.subdirectory.clone(),
    );

    bindings.push(binding.clone());
    ctx.save_bindings(&bindings)?;

    if ctx.is_json_output() {
        print_json(&binding)?;
    } else {
        println!("已绑定: {} -> {} (版本: {}, 挂载: {})",
            style(&project.name).cyan(),
            style(&plugin.name).green(),
            plugin_version.version,
            mount_path,
        );
        println!("使用 {} 应用变更", style("harbor apply").cyan());
    }

    Ok(())
}

// ----------------------------------------------------------------------------
// harbor unbind
// ----------------------------------------------------------------------------

fn cmd_unbind(ctx: &CliContext, project_name: &str, plugin_name: &str) -> Result<()> {
    let _lock = ctx.acquire_lock()?;

    let projects = ctx.load_projects();
    let plugins = ctx.load_plugins();
    let mut bindings = ctx.load_bindings();

    let project = find_by_name(&projects, project_name, |p| &p.name, "项目")?;
    let plugin = find_by_name(&plugins, plugin_name, |p| &p.name, "插件")?;

    let before = bindings.len();
    bindings.retain(|b| !(b.project_id == project.project_id && b.plugin_id == plugin.plugin_id));

    if bindings.len() == before {
        bail!("项目 '{}' 未绑定插件 '{}'", project.name, plugin.name);
    }

    ctx.save_bindings(&bindings)?;

    if ctx.is_json_output() {
        print_json(&serde_json::json!({
            "project": project.name,
            "plugin": plugin.name,
            "status": "unbound",
        }))?;
    } else {
        println!("已解绑: {} - {}", style(&project.name).cyan(), style(&plugin.name).green());
        println!("使用 {} 应用变更", style("harbor apply").cyan());
    }

    Ok(())
}

// ----------------------------------------------------------------------------
// harbor apply
// ----------------------------------------------------------------------------

fn cmd_apply(ctx: &CliContext, project_name: Option<&str>) -> Result<()> {
    let _lock = ctx.acquire_lock()?;

    let projects = ctx.load_projects();
    let bindings = ctx.load_bindings();
    let linker = ctx.get_linker();
    let plugins_base = ctx.data_dir.join("plugins");

    let target_projects: Vec<&Project> = if let Some(name) = project_name {
        let project = find_by_name(&projects, name, |p| &p.name, "项目")?;
        vec![project]
    } else {
        projects.iter().collect()
    };

    let mut total_created = 0;
    let mut total_removed = 0;
    let mut total_errors = 0;

    for project in &target_projects {
        let project_bindings: Vec<ProjectBinding> = bindings.iter()
            .filter(|b| b.project_id == project.project_id)
            .cloned()
            .collect();

        if project_bindings.is_empty() && project_name.is_none() {
            continue;
        }

        if !ctx.is_json_output() {
            println!("正在应用项目: {}...", style(&project.name).bold());
        }

        let result = linker.apply_bindings(
            &project.path,
            &[],
            &project_bindings,
            plugins_base.to_string_lossy().as_ref(),
        ).map_err(|e| anyhow!("应用绑定失败: {}", e))?;

        total_created += result.created.len();
        total_removed += result.removed.len();
        total_errors += result.errors.len();

        if !ctx.is_json_output() {
            if !result.created.is_empty() {
                for path in &result.created {
                    println!("  {} 创建: {}", style("✓").green(), path);
                }
            }
            if !result.removed.is_empty() {
                for path in &result.removed {
                    println!("  {} 移除: {}", style("−").yellow(), path);
                }
            }
            for err in &result.errors {
                println!("  {} 错误: {}", style("✗").red(), err);
            }
        }
    }

    if ctx.is_json_output() {
        print_json(&serde_json::json!({
            "created": total_created,
            "removed": total_removed,
            "errors": total_errors,
        }))?;
    } else {
        println!();
        println!("应用完成: 创建 {}, 移除 {}, 错误 {}",
            total_created, total_removed, total_errors);
    }

    Ok(())
}

// ----------------------------------------------------------------------------
// harbor engines list
// ----------------------------------------------------------------------------

#[derive(Serialize)]
struct EngineListOutput {
    engines: Vec<EngineItem>,
}

#[derive(Serialize)]
struct EngineItem {
    name: String,
    version: String,
    engine_type: String,
    path: String,
    is_mono: bool,
}

fn cmd_engines_list(ctx: &CliContext) -> Result<()> {
    let engines = ctx.load_engines();

    let items: Vec<EngineItem> = engines.iter().map(|e| EngineItem {
        name: e.name.clone(),
        version: e.version.clone(),
        engine_type: format!("{:?}", e.engine_type),
        path: e.path.clone(),
        is_mono: e.is_mono,
    }).collect();

    if ctx.is_json_output() {
        return print_json(&EngineListOutput { engines: items });
    }

    if items.is_empty() {
        println!("暂无引擎，使用 {} 安装引擎", style("harbor engines install <version>").cyan());
        return Ok(());
    }

    println!("{}", style("引擎列表").bold());
    println!();
    for item in &items {
        let mono_tag = if item.is_mono { style(" [.NET]").magenta().to_string() } else { String::new() };
        println!("  {} {}{} ({})", style(&item.name).bold(), style(&item.version).cyan(), mono_tag, item.engine_type);
        println!("    路径: {}", item.path);
    }
    println!();
    println!("共 {} 个引擎", items.len());

    Ok(())
}

// ----------------------------------------------------------------------------
// harbor engines install
// ----------------------------------------------------------------------------

fn cmd_engines_install(ctx: &CliContext, version: &str, mirror_id: Option<String>, variant: &str) -> Result<()> {
    let _lock = ctx.acquire_lock()?;

    let settings = ctx.load_settings();

    // Find mirror
    let mirror = match &mirror_id {
        Some(id) => settings.engine_mirrors.iter().find(|m| m.id == *id),
        None => settings.engine_mirrors.iter().find(|m| m.is_official),
    };

    let mirror = match mirror {
        Some(m) => m.clone(),
        None => bail!("未找到镜像配置，请使用 --mirror 指定镜像 ID"),
    };

    if !mirror.enabled {
        bail!("镜像 '{}' 已被禁用", mirror.name);
    }

    if !ctx.is_json_output() {
        println!("正在获取远程引擎版本列表...");
    }

    // Fetch remote versions
    let local_versions: Vec<String> = ctx.load_engines().iter().map(|e| e.version.clone()).collect();

    let rt = tokio::runtime::Runtime::new()
        .context("无法创建 tokio 运行时")?;

    let remote_versions = rt.block_on(async {
        crate::engine_downloader::EngineDownloader::fetch_remote_versions(&mirror, &local_versions).await
            .map_err(|e| anyhow!("获取远程版本失败: {}", e))
    })?;

    // Find matching version
    let target = remote_versions.iter().find(|v| {
        v.version == version && v.variant == variant
    }).or_else(|| remote_versions.iter().find(|v| {
        v.version.starts_with(version) && v.variant == variant
    }));

    let target = match target {
        Some(t) => t.clone(),
        None => {
            let available: Vec<String> = remote_versions.iter()
                .filter(|v| v.variant == variant)
                .map(|v| v.version.clone())
                .collect();
            bail!(
                "未找到版本 '{}' (variant: {})，可用版本:\n  {}",
                version,
                variant,
                available.join("\n  ")
            );
        }
    };

    if !ctx.is_json_output() {
        println!("正在下载引擎: {} ({})...", target.version, target.variant);
        println!("  文件大小: {}", format_size(target.file_size));
    }

    let engines_dir = ctx.data_dir.join("engines");
    fs::create_dir_all(&engines_dir)
        .context("无法创建引擎目录")?;

    // Check disk space
    if target.file_size > 0 {
        if let Ok(available) = fs2::available_space(&engines_dir) {
            let required = target.file_size * 3;
            if available < required {
                bail!(
                    "磁盘空间不足，可用 {:.0}MB，需要约 {:.0}MB",
                    available as f64 / 1024.0 / 1024.0,
                    required as f64 / 1024.0 / 1024.0
                );
            }
        }
    }

    // Download and install (without Tauri AppHandle)
    let version_dir_name = if variant == "mono" {
        format!("godot_{}_dotnet", version.replace('.', "_").replace('-', "_"))
    } else {
        format!("godot_{}", version.replace('.', "_").replace('-', "_"))
    };
    let target_dir = engines_dir.join(&version_dir_name);

    if target_dir.exists() {
        fs::remove_dir_all(&target_dir)
            .with_context(|| "删除旧引擎目录失败")?;
    }

    let download_dir = ctx.data_dir.join("downloads");
    fs::create_dir_all(&download_dir)
        .context("无法创建下载目录")?;

    let archive_name = if variant == "mono" {
        format!("{}_dotnet_{}", version.replace('.', "_").replace('-', "_"), target.file_name)
    } else {
        format!("{}_{}", version.replace('.', "_").replace('-', "_"), target.file_name)
    };
    let archive_path = download_dir.join(&archive_name);

    // Download with progress bar
    let pb = ProgressBar::new(target.file_size);
    pb.set_style(ProgressStyle::with_template(
        "{msg}\n{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})"
    ).unwrap()
        .progress_chars("#>-"));
    pb.set_message(format!("下载 Godot {} ({})", target.version, target.variant));

    let download_result: Result<()> = rt.block_on(async {
        let client = crate::utils::create_http_client(Some(std::time::Duration::from_secs(300)))
            .map_err(|e| anyhow!(e))?;
        let mut response = client.get(&target.download_url).send().await
            .map_err(|e| anyhow!("下载请求失败: {}", e))?;
        if !response.status().is_success() {
            bail!("下载失败，HTTP 状态码: {}", response.status());
        }

        let mut file = fs::File::create(&archive_path)
            .with_context(|| "创建下载文件失败")?;
        let mut downloaded: u64 = 0;

        use std::io::Write;
        loop {
            let chunk = response.chunk().await
                .map_err(|e| anyhow!("读取下载数据失败: {}", e))?;
            match chunk {
                Some(data) => {
                    file.write_all(&data)
                        .map_err(|e| anyhow!("写入文件失败: {}", e))?;
                    downloaded += data.len() as u64;
                    pb.set_position(downloaded);
                }
                None => break,
            }
        }
        file.flush().with_context(|| "刷新文件失败")?;
        Ok(())
    });

    if let Err(e) = download_result {
        let _ = fs::remove_file(&archive_path);
        return Err(e);
    }
    pb.finish_with_message("下载完成");

    // Extract
    if !ctx.is_json_output() {
        println!("正在解压引擎文件...");
    }

    fs::create_dir_all(&target_dir)
        .context("无法创建引擎目录")?;

    let extract_result = extract_engine_archive(&archive_path, &target_dir);
    let _ = fs::remove_file(&archive_path);

    if let Err(e) = extract_result {
        let _ = fs::remove_dir_all(&target_dir);
        bail!("解压引擎文件失败: {}", e);
    }

    let path_str = target_dir.to_string_lossy().to_string();

    let engine = EngineManager::get_engine_info(&path_str)
        .map_err(|e| {
            let _ = fs::remove_dir_all(&target_dir);
            anyhow!("下载的引擎文件无效: {}", e)
        })?;

    let mut registered_engine = engine;
    registered_engine.name = if variant == "mono" {
        format!("Godot {} (.NET)", version)
    } else {
        format!("Godot {}", version)
    };

    let mut engines = ctx.load_engines();
    engines.retain(|e| e.path != registered_engine.path);
    engines.push(registered_engine.clone());
    ctx.save_engines(&engines)?;

    if ctx.is_json_output() {
        print_json(&registered_engine)?;
    } else {
        println!("{} 已安装引擎: {}", style("✓").green(), style(&registered_engine.name).bold());
    }

    Ok(())
}

/// Extract engine archive without Tauri dependency
fn extract_engine_archive(archive_path: &std::path::Path, target_dir: &std::path::Path) -> Result<()> {
    let file = fs::File::open(archive_path)
        .context("打开压缩包失败")?;

    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| anyhow!("解析压缩包失败: {}", e))?;

    let total_entries = archive.len();

    for i in 0..total_entries {
        let mut entry = archive.by_index(i)
            .map_err(|e| anyhow!("读取压缩包条目失败: {}", e))?;

        let entry_path = entry.mangled_name();
        let entry_name = entry_path.to_string_lossy().to_string();

        let file_name = entry_name
            .split('/')
            .last()
            .unwrap_or(&entry_name)
            .to_string();

        if file_name.is_empty() || file_name.ends_with('/') {
            continue;
        }

        let dest_path = target_dir.join(&file_name);

        if entry.is_file() {
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| "创建目录失败")?;
            }

            let mut outfile = fs::File::create(&dest_path)
                .with_context(|| "创建文件失败")?;

            std::io::copy(&mut entry, &mut outfile)
                .map_err(|e| anyhow!("解压文件失败: {}", e))?;

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let file_name_lower = file_name.to_lowercase();
                if file_name_lower.contains("godot") && !file_name_lower.contains(".") {
                    let mut perms = fs::metadata(&dest_path)
                        .map_err(|e| anyhow!("获取文件元数据失败: {}", e))?
                        .permissions();
                    perms.set_mode(0o755);
                    fs::set_permissions(&dest_path, perms)
                        .map_err(|e| anyhow!("设置文件权限失败: {}", e))?;
                }
            }
        }
    }

    Ok(())
}

// ----------------------------------------------------------------------------
// harbor sync
// ----------------------------------------------------------------------------

fn cmd_sync(ctx: &CliContext, project_name: Option<&str>) -> Result<()> {
    let _lock = ctx.acquire_lock()?;

    let projects = ctx.load_projects();
    let plugins = ctx.load_plugins();
    let bindings = ctx.load_bindings();

    let target_projects: Vec<&Project> = if let Some(name) = project_name {
        let project = find_by_name(&projects, name, |p| &p.name, "项目")?;
        vec![project]
    } else {
        projects.iter().collect()
    };

    let mut synced = 0;
    let mut skipped = 0;

    for project in &target_projects {
        let config_path = harbor_config::get_harbor_config_path(&project.path);

        if !config_path.exists() {
            skipped += 1;
            continue;
        }

        let (config, _) = harbor_config::generate_config_from_bindings(project, &plugins, &bindings);
        harbor_config::write_harbor_config_to_project(&project.path, &config)
            .with_context(|| format!("同步项目 {} 的 .harbor.yml 失败", project.name))?;

        if !ctx.is_json_output() {
            println!("  {} {}", style("✓").green(), project.name);
        }
        synced += 1;
    }

    if ctx.is_json_output() {
        print_json(&serde_json::json!({
            "synced": synced,
            "skipped": skipped,
        }))?;
    } else {
        println!();
        println!("同步完成: {} 个项目已更新, {} 个项目跳过（无 .harbor.yml）", synced, skipped);
    }

    Ok(())
}

// ============================================================================
// Utility functions
// ============================================================================

fn get_app_root_dir() -> PathBuf {
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let mut dir = exe_dir.to_path_buf();
            if dir.ends_with("target\\debug") || dir.ends_with("target/release") {
                if let Some(parent) = dir.parent() {
                    if let Some(grandparent) = parent.parent() {
                        dir = grandparent.to_path_buf();
                    }
                }
            }
            return dir;
        }
    }
    PathBuf::from(".")
}

fn get_default_scan_dirs() -> Vec<String> {
    let mut dirs = Vec::new();

    if let Some(home) = dirs::home_dir() {
        dirs.push(home.join("Documents").to_string_lossy().to_string());
        dirs.push(home.join("Projects").to_string_lossy().to_string());
        dirs.push(home.join("Desktop").to_string_lossy().to_string());
        dirs.push(home.to_string_lossy().to_string());
    }

    if cfg!(windows) {
        if let Ok(d) = std::env::var("USERPROFILE") {
            dirs.push(d);
        }
    }

    dirs.sort();
    dirs.dedup();
    dirs
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;
    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

#[cfg(windows)]
fn detached_cmd(program: impl AsRef<std::ffi::OsStr>) -> std::process::Command {
    use std::os::windows::process::CommandExt;
    const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
    const DETACHED_PROCESS: u32 = 0x00000008;
    let mut cmd = std::process::Command::new(program);
    cmd.creation_flags(CREATE_NEW_PROCESS_GROUP | DETACHED_PROCESS);
    cmd.stdout(std::process::Stdio::null());
    cmd.stderr(std::process::Stdio::null());
    cmd.stdin(std::process::Stdio::null());
    cmd
}

#[cfg(not(windows))]
fn detached_cmd(program: impl AsRef<std::ffi::OsStr>) -> std::process::Command {
    let mut cmd = std::process::Command::new(program);
    cmd.stdout(std::process::Stdio::null());
    cmd.stderr(std::process::Stdio::null());
    cmd.stdin(std::process::Stdio::null());
    cmd
}
