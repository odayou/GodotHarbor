use std::fs;
use std::path::Path;
use anyhow::{Result, Context};
use crate::models::{ProjectBinding, ApplyResult, ConflictInfo, ConflictType};
use crate::utils::copy_dir_all;

#[derive(Clone)]
pub struct Linker {}

pub struct DiffResult {
    pub to_add: Vec<ProjectBinding>,
    pub to_remove: Vec<ProjectBinding>,
    pub to_keep: Vec<ProjectBinding>,
}

pub struct PreCheckResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl Linker {
    pub fn new() -> Self {
        Self {}
    }

    pub fn pre_check(
        &self,
        project_path: &str,
        bindings: &[ProjectBinding],
        plugin_base_path: &str,
    ) -> PreCheckResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        let project = Path::new(project_path);
        if !project.exists() {
            errors.push("项目路径不存在".to_string());
            return PreCheckResult { is_valid: false, errors, warnings };
        }

        let project_godot = project.join("project.godot");
        if !project_godot.exists() {
            errors.push("项目路径下未找到 project.godot 文件".to_string());
        }

        let addons_dir = project.join("addons");
        if addons_dir.exists() {
            match fs::metadata(&addons_dir) {
                Ok(meta) => {
                    if meta.permissions().readonly() {
                        errors.push("addons 目录没有写权限".to_string());
                    }
                }
                Err(e) => {
                    warnings.push(format!("无法检查 addons 目录权限: {}", e));
                }
            }
        } else {
            match fs::create_dir_all(&addons_dir) {
                Ok(_) => {}
                Err(e) => {
                    errors.push(format!("无法创建 addons 目录: {}", e));
                }
            }
        }

        for binding in bindings {
            let plugin_path = Path::new(plugin_base_path)
                .join(&binding.plugin_id)
                .join(&binding.version_id)
                .join("payload");

            if !plugin_path.exists() {
                errors.push(format!(
                    "插件源不存在: {} (版本: {})",
                    binding.plugin_id, binding.version_id
                ));
            }
        }

        PreCheckResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
        }
    }

    pub fn compute_diff(
        &self,
        current_bindings: &[ProjectBinding],
        desired_bindings: &[ProjectBinding],
    ) -> DiffResult {
        let mut to_add = Vec::new();
        let mut to_remove = Vec::new();
        let mut to_keep = Vec::new();

        for desired in desired_bindings {
            let current = current_bindings.iter().find(|c| {
                c.project_id == desired.project_id && c.plugin_id == desired.plugin_id
            });

            match current {
                Some(curr) if curr.version_id == desired.version_id
                    && curr.mount_path == desired.mount_path =>
                {
                    to_keep.push(desired.clone());
                }
                Some(curr) => {
                    to_remove.push(curr.clone());
                    to_add.push(desired.clone());
                }
                None => {
                    to_add.push(desired.clone());
                }
            }
        }

        for current in current_bindings {
            let still_desired = desired_bindings.iter().any(|d| {
                d.project_id == current.project_id && d.plugin_id == current.plugin_id
            });
            if !still_desired {
                to_remove.push(current.clone());
            }
        }

        DiffResult {
            to_add,
            to_remove,
            to_keep,
        }
    }

    pub fn apply_bindings(
        &self,
        project_path: &str,
        current_bindings: &[ProjectBinding],
        desired_bindings: &[ProjectBinding],
        plugin_base_path: &str,
        app_data_dir: &str,
    ) -> Result<ApplyResult> {
        let mut result = ApplyResult {
            success: true,
            created: Vec::new(),
            removed: Vec::new(),
            errors: Vec::new(),
        };

        let pre_check = self.pre_check(project_path, desired_bindings, plugin_base_path);
        if !pre_check.is_valid {
            result.success = false;
            result.errors = pre_check.errors;
            return Ok(result);
        }

        for warning in &pre_check.warnings {
            eprintln!("Warning: {}", warning);
        }

        let project = Path::new(project_path);

        let diff = self.compute_diff(current_bindings, desired_bindings);

        let conflicts = self.check_conflicts(project_path, &diff.to_add, &diff.to_keep, current_bindings)?;
        let blocking_conflicts: Vec<&ConflictInfo> = conflicts.iter()
            .filter(|c| !matches!(c.conflict_type, ConflictType::ExistingPlugin | ConflictType::PathExists))
            .collect();
        if !blocking_conflicts.is_empty() {
            for conflict in &blocking_conflicts {
                result.errors.push(format!("冲突: {}", conflict.message));
            }
            result.success = false;
            return Ok(result);
        }
        for conflict in &conflicts {
            if matches!(conflict.conflict_type, ConflictType::ExistingPlugin | ConflictType::PathExists) {
                eprintln!("Info: {}", conflict.message);
            }
        }

        let mut applied_ops: Vec<AppliedOp> = Vec::new();

        for binding in &diff.to_remove {
            let target_path = project.join(&binding.mount_path);
            // to_remove 来自 current_bindings，必定为 Harbor 管理
            match self.safe_remove_link(&target_path, true) {
                Ok(op) => {
                    result.removed.push(target_path.to_string_lossy().to_string());
                    applied_ops.push(op);
                }
                Err(e) => {
                    result.errors.push(format!("移除失败 {}: {}", binding.mount_path, e));
                    result.success = false;
                    self.rollback_ops(&applied_ops, project_path);
                    return Ok(result);
                }
            }
        }

        for binding in &diff.to_add {
            match self.apply_binding(binding, project, plugin_base_path, app_data_dir) {
                Ok(mount_path) => {
                    result.created.push(mount_path.clone());
                    applied_ops.push(AppliedOp::Create {
                        path: mount_path,
                    });
                }
                Err(e) => {
                    result.errors.push(format!("应用绑定失败: {}", e));
                    result.success = false;
                    self.rollback_ops(&applied_ops, project_path);
                    return Ok(result);
                }
            }
        }

        Ok(result)
    }

    pub fn check_conflicts(
        &self,
        project_path: &str,
        to_add: &[ProjectBinding],
        to_keep: &[ProjectBinding],
        current_bindings: &[ProjectBinding],
    ) -> Result<Vec<ConflictInfo>> {
        let mut conflicts = Vec::new();

        let project = Path::new(project_path);

        let kept_paths: std::collections::HashSet<String> = to_keep.iter()
            .map(|b| b.mount_path.clone())
            .collect();

        for binding in to_add {
            if kept_paths.contains(&binding.mount_path) {
                continue;
            }

            let target_path = project.join(&binding.mount_path);

            if target_path.exists() && !self.is_managed_link(&target_path, project, current_bindings)? {
                let has_plugin_cfg = target_path.join("plugin.cfg").exists();
                let message = if has_plugin_cfg {
                    format!(
                        "目标路径已存在且包含插件 (非 Harbor 管理): {}，将合并安装",
                        binding.mount_path
                    )
                } else {
                    format!(
                        "目标路径已存在且非 Harbor 管理: {}，将合并安装",
                        binding.mount_path
                    )
                };
                conflicts.push(ConflictInfo {
                    conflict_type: if has_plugin_cfg { ConflictType::ExistingPlugin } else { ConflictType::PathExists },
                    path: target_path.to_string_lossy().to_string(),
                    message,
                });
            }
        }

        let addons_dir = project.join("addons");
        if addons_dir.exists() {
            if let Ok(entries) = fs::read_dir(&addons_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() && path.join("plugin.cfg").exists() {
                        let mount_path = format!("addons/{}", path.file_name().unwrap_or_default().to_string_lossy());
                        let is_managed = self.is_managed_link(&path, project, current_bindings).unwrap_or(false);
                        let is_in_bindings = to_add.iter().any(|b| b.mount_path == mount_path)
                            || to_keep.iter().any(|b| b.mount_path == mount_path);
                        if !is_managed && !is_in_bindings {
                            conflicts.push(ConflictInfo {
                                conflict_type: ConflictType::UnmanagedAddon,
                                path: path.to_string_lossy().to_string(),
                                message: format!(
                                    "发现非 Harbor 管理的插件: {}，建议先导入到 Harbor 管理",
                                    mount_path
                                ),
                            });
                        }
                    }
                }
            }
        }

        Ok(conflicts)
    }

    fn apply_binding(
        &self,
        binding: &ProjectBinding,
        project: &Path,
        plugin_base_path: &str,
        app_data_dir: &str,
    ) -> Result<String> {
        let plugin_path = Path::new(plugin_base_path)
            .join(&binding.plugin_id)
            .join(&binding.version_id)
            .join("payload");

        if !plugin_path.exists() {
            anyhow::bail!("Plugin payload directory does not exist: {}", plugin_path.to_string_lossy());
        }

        let source_path = if binding.subdirectory.is_empty() {
            plugin_path
        } else {
            plugin_path.join(&binding.subdirectory)
        };

        if !source_path.exists() {
            anyhow::bail!("Plugin subdirectory does not exist: {}", source_path.to_string_lossy());
        }

        let target_path = project.join(&binding.mount_path);

        // 备份到 <app_data>/backups/<project_id>/<sanitized_mount_path>，避免污染项目目录
        let backup_path = if target_path.exists() {
            let sanitized = binding.mount_path.replace(['/', '\\'], "_");
            let bak_dir = Path::new(app_data_dir).join("backups").join(&binding.project_id);
            fs::create_dir_all(&bak_dir)
                .with_context(|| format!("Failed to create backup dir: {}", bak_dir.to_string_lossy()))?;
            let bak = bak_dir.join(&sanitized);
            if bak.exists() {
                let _ = fs::remove_dir_all(&bak);
            }
            fs::rename(&target_path, &bak)
                .with_context(|| format!("Failed to backup existing directory: {}", target_path.to_string_lossy()))?;
            Some(bak)
        } else {
            None
        };

        if let Some(parent) = target_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .context("Failed to create parent directory for mount target")?;
            }
        }

        let mount_result: Result<()> = (|| {
            copy_dir_all(&source_path, &target_path)
                .map_err(|e| anyhow::anyhow!(e))
                .with_context(|| format!("Failed to copy plugin from {} to {}", source_path.to_string_lossy(), target_path.to_string_lossy()))?;
            Ok(())
        })();

        match mount_result {
            Ok(()) => {
                if let Some(bak) = &backup_path {
                    let _ = fs::remove_dir_all(bak);
                }
                Ok(target_path.to_string_lossy().to_string())
            }
            Err(e) => {
                if let Some(bak) = &backup_path {
                    let _ = fs::rename(bak, &target_path);
                }
                Err(e)
            }
        }
    }

    /// 判断路径是否由 Harbor 管理：
    /// - symlink/junction（旧链接遗留）→ 是
    /// - 路径在 current_bindings 的 mount_path 中 → 是
    /// - 否则 → 否
    fn is_managed_link(
        &self,
        path: &Path,
        project_path: &Path,
        current_bindings: &[ProjectBinding],
    ) -> Result<bool> {
        if !path.exists() && path.symlink_metadata().is_err() {
            return Ok(false);
        }

        let metadata = fs::symlink_metadata(path)?;

        if metadata.file_type().is_symlink() {
            return Ok(true);
        }

        if self.is_junction(path) {
            return Ok(true);
        }

        // 比对 bindings：路径相对项目根的 mount_path 是否在当前绑定中
        if let Ok(rel) = path.strip_prefix(project_path) {
            let mount_path = rel.to_string_lossy().replace('\\', "/");
            if current_bindings.iter().any(|b| b.mount_path == mount_path) {
                return Ok(true);
            }
        }

        Ok(false)
    }

    #[allow(unused_variables)]
    fn is_junction(&self, path: &Path) -> bool {
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            if let Ok(metadata) = fs::symlink_metadata(path) {
                const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
                if metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
                    return true;
                }
            }
        }
        false
    }

    /// 安全移除挂载路径。
    /// - `managed=true`: 路径确认由 Harbor 管理（在 bindings 中或刚由 apply_binding 创建），目录会被 remove_dir_all
    /// - `managed=false`: 目录保留（防止误删用户数据）；普通文件仍可移除
    fn safe_remove_link(&self, path: &Path, managed: bool) -> Result<AppliedOp> {
        if !path.exists() && !path.symlink_metadata().is_ok() {
            return Ok(AppliedOp::None);
        }

        let metadata = fs::symlink_metadata(path);

        if let Ok(meta) = &metadata {
            if meta.file_type().is_symlink() {
                if path.is_dir() {
                    fs::remove_dir(path)
                        .with_context(|| format!("Failed to remove symlink directory: {}", path.to_string_lossy()))?;
                } else {
                    fs::remove_file(path)
                        .with_context(|| format!("Failed to remove symlink file: {}", path.to_string_lossy()))?;
                }
                return Ok(AppliedOp::Remove {
                    path: path.to_path_buf(),
                    was_symlink: true,
                });
            }
        }

        if self.is_junction(path) {
            fs::remove_dir(path)
                .with_context(|| format!("Failed to remove junction: {}", path.to_string_lossy()))?;
            return Ok(AppliedOp::Remove {
                path: path.to_path_buf(),
                was_symlink: true,
            });
        }

        if path.is_dir() {
            if managed {
                fs::remove_dir_all(path)
                    .with_context(|| format!("Failed to remove directory: {}", path.to_string_lossy()))?;
                return Ok(AppliedOp::Remove {
                    path: path.to_path_buf(),
                    was_symlink: false,
                });
            }
            return Ok(AppliedOp::None);
        }

        fs::remove_file(path)
            .with_context(|| format!("Failed to remove file: {}", path.to_string_lossy()))?;
        Ok(AppliedOp::Remove {
            path: path.to_path_buf(),
            was_symlink: false,
        })
    }

    fn rollback_ops(&self, ops: &[AppliedOp], _project_path: &str) {
        for op in ops.iter().rev() {
            match op {
                AppliedOp::Create { path } => {
                    let p = Path::new(path);
                    if p.exists() {
                        // 回滚刚创建的路径，必定是 Harbor 管理的
                        if let Err(e) = self.safe_remove_link(p, true) {
                            eprintln!("Rollback: failed to remove created path {}: {}", path, e);
                        }
                    }
                }
                AppliedOp::Remove { path, was_symlink } => {
                    eprintln!(
                        "Rollback: cannot restore removed {} at {} (manual recovery may be needed)",
                        if *was_symlink { "symlink/junction" } else { "directory" },
                        path.to_string_lossy()
                    );
                }
                AppliedOp::None => {}
            }
        }
    }
}

enum AppliedOp {
    Create { path: String },
    Remove { path: std::path::PathBuf, was_symlink: bool },
    None,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_binding(project_id: &str, plugin_id: &str, version_id: &str, mount_path: &str) -> ProjectBinding {
        ProjectBinding {
            project_id: project_id.to_string(),
            plugin_id: plugin_id.to_string(),
            version_id: version_id.to_string(),
            unit_id: "u1".to_string(),
            mount_path: mount_path.to_string(),
            created_at: Utc::now(),
            is_healthy: None,
            subdirectory: String::new(),
        }
    }

    #[test]
    fn test_compute_diff_add_new() {
        let linker = Linker::new();
        let current = vec![];
        let desired = vec![make_binding("p1", "pl1", "v1", "addons/foo")];

        let diff = linker.compute_diff(&current, &desired);
        assert_eq!(diff.to_add.len(), 1);
        assert_eq!(diff.to_remove.len(), 0);
        assert_eq!(diff.to_keep.len(), 0);
    }

    #[test]
    fn test_compute_diff_remove_binding() {
        let linker = Linker::new();
        let current = vec![make_binding("p1", "pl1", "v1", "addons/foo")];
        let desired = vec![];

        let diff = linker.compute_diff(&current, &desired);
        assert_eq!(diff.to_add.len(), 0);
        assert_eq!(diff.to_remove.len(), 1);
        assert_eq!(diff.to_keep.len(), 0);
    }

    #[test]
    fn test_compute_diff_keep_unchanged() {
        let linker = Linker::new();
        let binding = make_binding("p1", "pl1", "v1", "addons/foo");
        let current = vec![binding.clone()];
        let desired = vec![binding];

        let diff = linker.compute_diff(&current, &desired);
        assert_eq!(diff.to_add.len(), 0);
        assert_eq!(diff.to_remove.len(), 0);
        assert_eq!(diff.to_keep.len(), 1);
    }

    #[test]
    fn test_compute_diff_version_change() {
        let linker = Linker::new();
        let current = vec![make_binding("p1", "pl1", "v1", "addons/foo")];
        let desired = vec![make_binding("p1", "pl1", "v2", "addons/foo")];

        let diff = linker.compute_diff(&current, &desired);
        assert_eq!(diff.to_add.len(), 1);
        assert_eq!(diff.to_remove.len(), 1);
        assert_eq!(diff.to_keep.len(), 0);
    }

    #[test]
    fn test_compute_diff_mount_path_change() {
        let linker = Linker::new();
        let current = vec![make_binding("p1", "pl1", "v1", "addons/foo")];
        let desired = vec![make_binding("p1", "pl1", "v1", "addons/bar")];

        let diff = linker.compute_diff(&current, &desired);
        assert_eq!(diff.to_add.len(), 1);
        assert_eq!(diff.to_remove.len(), 1);
    }

    #[test]
    fn test_compute_diff_mixed_operations() {
        let linker = Linker::new();
        let current = vec![
            make_binding("p1", "pl1", "v1", "addons/foo"),
            make_binding("p1", "pl2", "v1", "addons/bar"),
        ];
        let desired = vec![
            make_binding("p1", "pl1", "v1", "addons/foo"),
            make_binding("p1", "pl3", "v1", "addons/baz"),
        ];

        let diff = linker.compute_diff(&current, &desired);
        assert_eq!(diff.to_keep.len(), 1);
        assert_eq!(diff.to_remove.len(), 1);
        assert_eq!(diff.to_add.len(), 1);
    }

    #[test]
    fn test_pre_check_nonexistent_project() {
        let linker = Linker::new();
        let result = linker.pre_check("/nonexistent/path", &[], "/plugins");
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.contains("项目路径不存在")));
    }

    #[test]
    fn test_pre_check_missing_project_godot() {
        let dir = tempfile::TempDir::new().unwrap();
        let linker = Linker::new();
        let result = linker.pre_check(dir.path().to_str().unwrap(), &[], "/plugins");
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.contains("project.godot")));
    }

    #[test]
    fn test_pre_check_valid_project() {
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::write(dir.path().join("project.godot"), "[application]\n").unwrap();
        let linker = Linker::new();
        let result = linker.pre_check(dir.path().to_str().unwrap(), &[], "/plugins");
        assert!(result.is_valid);
    }

    #[test]
    fn test_pre_check_missing_plugin_source() {
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::write(dir.path().join("project.godot"), "[application]\n").unwrap();
        let linker = Linker::new();
        let bindings = vec![make_binding("p1", "pl1", "v1", "addons/foo")];
        let result = linker.pre_check(dir.path().to_str().unwrap(), &bindings, "/nonexistent/plugins");
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.contains("插件源不存在")));
    }

    #[test]
    fn test_is_newer_logic() {
        assert!(Linker::new().compute_diff(&[], &[]).to_add.is_empty());
    }

    #[test]
    fn test_safe_remove_link_managed_dir() {
        let dir = tempfile::TempDir::new().unwrap();
        let managed_dir = dir.path().join("addons/my_plugin");
        fs::create_dir_all(&managed_dir).unwrap();
        fs::write(managed_dir.join("plugin.cfg"), "name=\"test\"").unwrap();

        let linker = Linker::new();
        let result = linker.safe_remove_link(&managed_dir, true);
        assert!(result.is_ok());
        assert!(!managed_dir.exists());
    }

    #[test]
    fn test_safe_remove_link_non_managed_dir_preserved() {
        let dir = tempfile::TempDir::new().unwrap();
        let original_dir = dir.path().join("addons/original_plugin");
        fs::create_dir_all(&original_dir).unwrap();
        fs::write(original_dir.join("plugin.cfg"), "name=\"original\"").unwrap();

        let linker = Linker::new();
        let result = linker.safe_remove_link(&original_dir, false);
        assert!(result.is_ok());
        assert!(original_dir.exists());
        assert!(original_dir.join("plugin.cfg").exists());
    }

    #[test]
    fn test_safe_remove_link_nonexistent_path() {
        let linker = Linker::new();
        let result = linker.safe_remove_link(Path::new("/nonexistent/path/xyz"), true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_safe_remove_link_regular_file() {
        let dir = tempfile::TempDir::new().unwrap();
        let file_path = dir.path().join("some_file.txt");
        fs::write(&file_path, "content").unwrap();

        let linker = Linker::new();
        // 普通文件无论 managed 与否都可移除
        let result = linker.safe_remove_link(&file_path, false);
        assert!(result.is_ok());
        assert!(!file_path.exists());
    }

    #[test]
    fn test_is_managed_link_in_bindings() {
        let project_dir = tempfile::TempDir::new().unwrap();
        let managed_dir = project_dir.path().join("addons/my_plugin");
        fs::create_dir_all(&managed_dir).unwrap();
        fs::write(managed_dir.join("plugin.cfg"), "name=\"test\"").unwrap();

        let linker = Linker::new();
        let bindings = vec![make_binding("p1", "pl1", "v1", "addons/my_plugin")];
        assert!(linker.is_managed_link(&managed_dir, project_dir.path(), &bindings).unwrap());
    }

    #[test]
    fn test_is_managed_link_not_in_bindings() {
        let project_dir = tempfile::TempDir::new().unwrap();
        let plain_dir = project_dir.path().join("addons/other_plugin");
        fs::create_dir_all(&plain_dir).unwrap();
        fs::write(plain_dir.join("plugin.cfg"), "name=\"test\"").unwrap();

        let linker = Linker::new();
        let bindings = vec![make_binding("p1", "pl1", "v1", "addons/my_plugin")];
        assert!(!linker.is_managed_link(&plain_dir, project_dir.path(), &bindings).unwrap());
    }

    #[test]
    fn test_is_managed_link_nonexistent() {
        let project_dir = tempfile::TempDir::new().unwrap();
        let linker = Linker::new();
        assert!(!linker.is_managed_link(Path::new("/nonexistent"), project_dir.path(), &[]).unwrap());
    }

    #[test]
    fn test_apply_bindings_no_marker_written() {
        let project_dir = tempfile::TempDir::new().unwrap();
        fs::write(project_dir.path().join("project.godot"), "[application]\n").unwrap();

        let app_data_dir = tempfile::TempDir::new().unwrap();
        let plugin_base = tempfile::TempDir::new().unwrap();
        let plugin_payload = plugin_base.path().join("pl1").join("v1").join("payload");
        fs::create_dir_all(&plugin_payload).unwrap();
        fs::write(plugin_payload.join("plugin.cfg"), "name=\"test_plugin\"").unwrap();
        fs::write(plugin_payload.join("script.gd"), "extends Node").unwrap();

        let linker = Linker::new();
        let binding = make_binding("p1", "pl1", "v1", "addons/test_plugin");
        let result = linker.apply_bindings(
            project_dir.path().to_str().unwrap(),
            &[],
            &[binding],
            plugin_base.path().to_str().unwrap(),
            app_data_dir.path().to_str().unwrap(),
        ).unwrap();

        assert!(result.success);
        let installed = project_dir.path().join("addons/test_plugin");
        assert!(installed.exists());
        // 不再写 .harbor-managed 标记
        assert!(!installed.join(".harbor-managed").exists());
        assert!(installed.join("plugin.cfg").exists());
    }

    #[test]
    fn test_apply_bindings_then_safe_remove() {
        let project_dir = tempfile::TempDir::new().unwrap();
        fs::write(project_dir.path().join("project.godot"), "[application]\n").unwrap();

        let app_data_dir = tempfile::TempDir::new().unwrap();
        let plugin_base = tempfile::TempDir::new().unwrap();
        let plugin_payload = plugin_base.path().join("pl1").join("v1").join("payload");
        fs::create_dir_all(&plugin_payload).unwrap();
        fs::write(plugin_payload.join("plugin.cfg"), "name=\"managed\"").unwrap();

        let linker = Linker::new();

        let binding = make_binding("p1", "pl1", "v1", "addons/managed_plugin");
        let result = linker.apply_bindings(
            project_dir.path().to_str().unwrap(),
            &[],
            &[binding.clone()],
            plugin_base.path().to_str().unwrap(),
            app_data_dir.path().to_str().unwrap(),
        ).unwrap();
        assert!(result.success);

        let managed_path = project_dir.path().join("addons/managed_plugin");
        // 已知由 Harbor 管理 → managed=true
        let result = linker.safe_remove_link(&managed_path, true);
        assert!(result.is_ok());

        assert!(!managed_path.exists());
    }

    #[test]
    fn test_safe_remove_link_preserves_non_managed_neighbor() {
        let project_dir = tempfile::TempDir::new().unwrap();
        let addons = project_dir.path().join("addons");
        fs::create_dir_all(&addons).unwrap();

        let managed = addons.join("managed_plugin");
        fs::create_dir_all(&managed).unwrap();
        fs::write(managed.join("plugin.cfg"), "name=\"managed\"").unwrap();

        let original = addons.join("original_plugin");
        fs::create_dir_all(&original).unwrap();
        fs::write(original.join("plugin.cfg"), "name=\"original\"").unwrap();

        let linker = Linker::new();
        let result = linker.safe_remove_link(&managed, true);
        assert!(result.is_ok());

        assert!(!managed.exists());
        assert!(original.exists());
        assert!(original.join("plugin.cfg").exists());
    }

    #[test]
    fn test_apply_bindings_remove_non_managed_preserves() {
        let project_dir = tempfile::TempDir::new().unwrap();
        fs::write(project_dir.path().join("project.godot"), "[application]\n").unwrap();

        let non_managed = project_dir.path().join("addons/non_managed");
        fs::create_dir_all(&non_managed).unwrap();
        fs::write(non_managed.join("plugin.cfg"), "name=\"non_managed\"").unwrap();

        let linker = Linker::new();
        // 未在 bindings 中 → managed=false → 保留
        let result = linker.safe_remove_link(&project_dir.path().join("addons/non_managed"), false);
        assert!(result.is_ok());
        assert!(non_managed.exists());
        assert!(non_managed.join("plugin.cfg").exists());
    }

    #[test]
    fn test_apply_bindings_upgrade_version() {
        let project_dir = tempfile::TempDir::new().unwrap();
        fs::write(project_dir.path().join("project.godot"), "[application]\n").unwrap();

        let app_data_dir = tempfile::TempDir::new().unwrap();
        let plugin_base = tempfile::TempDir::new().unwrap();

        let v1_payload = plugin_base.path().join("pl1").join("v1").join("payload");
        fs::create_dir_all(&v1_payload).unwrap();
        fs::write(v1_payload.join("plugin.cfg"), "name=\"test\"\nversion=\"1.0.0\"").unwrap();

        let v2_payload = plugin_base.path().join("pl1").join("v2").join("payload");
        fs::create_dir_all(&v2_payload).unwrap();
        fs::write(v2_payload.join("plugin.cfg"), "name=\"test\"\nversion=\"2.0.0\"").unwrap();

        let linker = Linker::new();

        let binding_v1 = make_binding("p1", "pl1", "v1", "addons/test");
        let result = linker.apply_bindings(
            project_dir.path().to_str().unwrap(),
            &[],
            &[binding_v1.clone()],
            plugin_base.path().to_str().unwrap(),
            app_data_dir.path().to_str().unwrap(),
        ).unwrap();
        assert!(result.success);
        assert!(result.created.len() == 1);

        let binding_v2 = make_binding("p1", "pl1", "v2", "addons/test");
        let result = linker.apply_bindings(
            project_dir.path().to_str().unwrap(),
            &[binding_v1],
            &[binding_v2],
            plugin_base.path().to_str().unwrap(),
            app_data_dir.path().to_str().unwrap(),
        ).unwrap();
        assert!(result.success);
        assert!(result.removed.len() == 1);
        assert!(result.created.len() == 1);

        let installed = project_dir.path().join("addons/test");
        assert!(installed.exists());
        let cfg_content = fs::read_to_string(installed.join("plugin.cfg")).unwrap();
        assert!(cfg_content.contains("2.0.0"));
    }

    #[test]
    fn test_check_conflicts_non_managed_existing() {
        let project_dir = tempfile::TempDir::new().unwrap();
        fs::write(project_dir.path().join("project.godot"), "[application]\n").unwrap();

        let existing = project_dir.path().join("addons/existing_plugin");
        fs::create_dir_all(&existing).unwrap();
        fs::write(existing.join("plugin.cfg"), "name=\"existing\"").unwrap();

        let linker = Linker::new();
        let binding = make_binding("p1", "pl1", "v1", "addons/existing_plugin");

        let conflicts = linker.check_conflicts(
            project_dir.path().to_str().unwrap(),
            &[binding],
            &[],
            &[],
        ).unwrap();

        assert!(!conflicts.is_empty());
        assert!(conflicts.iter().any(|c| matches!(c.conflict_type, ConflictType::ExistingPlugin)));
    }

    #[test]
    fn test_check_conflicts_managed_existing_no_conflict() {
        let project_dir = tempfile::TempDir::new().unwrap();
        fs::write(project_dir.path().join("project.godot"), "[application]\n").unwrap();

        let managed = project_dir.path().join("addons/managed_plugin");
        fs::create_dir_all(&managed).unwrap();
        fs::write(managed.join("plugin.cfg"), "name=\"managed\"").unwrap();

        let linker = Linker::new();
        let binding = make_binding("p1", "pl1", "v1", "addons/managed_plugin");
        // current_bindings 包含该 mount_path → Harbor 管理 → 无冲突
        let current = vec![binding.clone()];

        let conflicts = linker.check_conflicts(
            project_dir.path().to_str().unwrap(),
            &[binding],
            &[],
            &current,
        ).unwrap();

        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_apply_bindings_multiple_plugins() {
        let project_dir = tempfile::TempDir::new().unwrap();
        fs::write(project_dir.path().join("project.godot"), "[application]\n").unwrap();

        let app_data_dir = tempfile::TempDir::new().unwrap();
        let plugin_base = tempfile::TempDir::new().unwrap();

        for (pl_id, ver_id) in [("pl1", "v1"), ("pl2", "v1")] {
            let payload = plugin_base.path().join(pl_id).join(ver_id).join("payload");
            fs::create_dir_all(&payload).unwrap();
            fs::write(payload.join("plugin.cfg"), format!("name=\"{}\"", pl_id)).unwrap();
        }

        let linker = Linker::new();
        let bindings = vec![
            make_binding("p1", "pl1", "v1", "addons/pl1"),
            make_binding("p1", "pl2", "v1", "addons/pl2"),
        ];

        let result = linker.apply_bindings(
            project_dir.path().to_str().unwrap(),
            &[],
            &bindings,
            plugin_base.path().to_str().unwrap(),
            app_data_dir.path().to_str().unwrap(),
        ).unwrap();

        assert!(result.success);
        assert_eq!(result.created.len(), 2);
        assert!(project_dir.path().join("addons/pl1").exists());
        assert!(project_dir.path().join("addons/pl2").exists());
        // 不应有标记文件
        assert!(!project_dir.path().join("addons/pl1").join(".harbor-managed").exists());
    }

    #[test]
    fn test_apply_bindings_with_subdirectory() {
        let project_dir = tempfile::TempDir::new().unwrap();
        fs::write(project_dir.path().join("project.godot"), "[application]\n").unwrap();

        let app_data_dir = tempfile::TempDir::new().unwrap();
        let plugin_base = tempfile::TempDir::new().unwrap();
        let payload = plugin_base.path().join("pl1").join("v1").join("payload");
        let subdir = payload.join("addons").join("my_plugin");
        fs::create_dir_all(&subdir).unwrap();
        fs::write(subdir.join("plugin.cfg"), "name=\"sub_plugin\"").unwrap();

        let linker = Linker::new();
        let mut binding = make_binding("p1", "pl1", "v1", "addons/my_plugin");
        binding.subdirectory = "addons/my_plugin".to_string();

        let result = linker.apply_bindings(
            project_dir.path().to_str().unwrap(),
            &[],
            &[binding],
            plugin_base.path().to_str().unwrap(),
            app_data_dir.path().to_str().unwrap(),
        ).unwrap();

        assert!(result.success);
        let installed = project_dir.path().join("addons/my_plugin");
        assert!(installed.exists());
        assert!(installed.join("plugin.cfg").exists());
        assert!(!installed.join(".harbor-managed").exists());
    }

    #[test]
    fn test_apply_bindings_existing_non_managed_replaced() {
        let project_dir = tempfile::TempDir::new().unwrap();
        fs::write(project_dir.path().join("project.godot"), "[application]\n").unwrap();

        let app_data_dir = tempfile::TempDir::new().unwrap();
        let existing = project_dir.path().join("addons/godot_mcp");
        fs::create_dir_all(&existing).unwrap();
        fs::write(existing.join("plugin.cfg"), "name=\"Godot MCP\"").unwrap();
        fs::write(existing.join("user_custom.gd"), "extends Node2D").unwrap();

        let plugin_base = tempfile::TempDir::new().unwrap();
        let plugin_payload = plugin_base.path().join("pl1").join("v1").join("payload");
        fs::create_dir_all(&plugin_payload).unwrap();
        fs::write(plugin_payload.join("plugin.cfg"), "name=\"Godot MCP\"").unwrap();
        fs::write(plugin_payload.join("new_script.gd"), "extends Node").unwrap();

        let linker = Linker::new();
        let binding = make_binding("p1", "pl1", "v1", "addons/godot_mcp");

        let result = linker.apply_bindings(
            project_dir.path().to_str().unwrap(),
            &[],
            &[binding],
            plugin_base.path().to_str().unwrap(),
            app_data_dir.path().to_str().unwrap(),
        ).unwrap();

        assert!(result.success);
        assert!(existing.join("plugin.cfg").exists());
        assert!(existing.join("new_script.gd").exists());
        // 旧 user_custom.gd 被覆盖（copy 模式为替换语义）
        assert!(!existing.join("user_custom.gd").exists());
        // 不再写标记
        assert!(!existing.join(".harbor-managed").exists());
        // 备份不在项目目录
        let backup_in_project = project_dir.path().join("addons/godot_mcp.harbor-bak");
        assert!(!backup_in_project.exists());
    }

    #[test]
    fn test_apply_bindings_existing_managed_replaced() {
        let project_dir = tempfile::TempDir::new().unwrap();
        fs::write(project_dir.path().join("project.godot"), "[application]\n").unwrap();

        let app_data_dir = tempfile::TempDir::new().unwrap();
        let managed = project_dir.path().join("addons/old_plugin");
        fs::create_dir_all(&managed).unwrap();
        fs::write(managed.join("plugin.cfg"), "name=\"Old\"").unwrap();

        let plugin_base = tempfile::TempDir::new().unwrap();
        // 旧版本 v1 payload（current）
        let payload_v1 = plugin_base.path().join("pl1").join("v1").join("payload");
        fs::create_dir_all(&payload_v1).unwrap();
        fs::write(payload_v1.join("plugin.cfg"), "name=\"Old\"").unwrap();
        // 新版本 v2 payload（desired）
        let payload_v2 = plugin_base.path().join("pl1").join("v2").join("payload");
        fs::create_dir_all(&payload_v2).unwrap();
        fs::write(payload_v2.join("plugin.cfg"), "name=\"New\"").unwrap();

        let linker = Linker::new();
        // 版本升级 v1→v2：to_remove v1（managed，删除旧目录）+ to_add v2（拷贝新内容）
        let current = vec![make_binding("p1", "pl1", "v1", "addons/old_plugin")];
        let desired = vec![make_binding("p1", "pl1", "v2", "addons/old_plugin")];

        let result = linker.apply_bindings(
            project_dir.path().to_str().unwrap(),
            &current,
            &desired,
            plugin_base.path().to_str().unwrap(),
            app_data_dir.path().to_str().unwrap(),
        ).unwrap();

        assert!(result.success);
        assert!(managed.exists());
        let cfg = fs::read_to_string(managed.join("plugin.cfg")).unwrap();
        assert_eq!(cfg, "name=\"New\"");
    }

    #[test]
    fn test_apply_bindings_existing_replaced_no_project_backup() {
        let project_dir = tempfile::TempDir::new().unwrap();
        fs::write(project_dir.path().join("project.godot"), "[application]\n").unwrap();

        let app_data_dir = tempfile::TempDir::new().unwrap();
        let target = project_dir.path().join("addons/existing");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("plugin.cfg"), "name=\"Original\"").unwrap();

        let plugin_base = tempfile::TempDir::new().unwrap();
        let plugin_payload = plugin_base.path().join("pl1").join("v1").join("payload");
        fs::create_dir_all(&plugin_payload).unwrap();
        fs::write(plugin_payload.join("plugin.cfg"), "name=\"Different\"").unwrap();

        let linker = Linker::new();
        let binding = make_binding("p1", "pl1", "v1", "addons/existing");

        let result = linker.apply_bindings(
            project_dir.path().to_str().unwrap(),
            &[],
            &[binding],
            plugin_base.path().to_str().unwrap(),
            app_data_dir.path().to_str().unwrap(),
        ).unwrap();

        assert!(result.success);
        assert!(target.exists());
        // 项目目录内不应残留 .harbor-bak
        let backup = project_dir.path().join("addons/existing.harbor-bak");
        assert!(!backup.exists());
    }
}
