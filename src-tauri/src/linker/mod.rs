use std::fs;
use std::path::Path;
use anyhow::{Result, Context};
use crate::models::{ProjectBinding, MountStrategy, ApplyResult, ConflictInfo};
use crate::utils::copy_dir_all;

#[derive(Clone)]
pub struct Linker {
    mount_strategy: MountStrategy,
}

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
    pub fn new(mount_strategy: MountStrategy) -> Self {
        Self { mount_strategy }
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

        let conflicts = self.check_conflicts(project_path, &diff.to_add, &diff.to_keep)?;
        let blocking_conflicts: Vec<&ConflictInfo> = conflicts.iter()
            .filter(|c| c.conflict_type != "existing_plugin" && c.conflict_type != "path_exists")
            .collect();
        if !blocking_conflicts.is_empty() {
            for conflict in &blocking_conflicts {
                result.errors.push(format!("冲突: {}", conflict.message));
            }
            result.success = false;
            return Ok(result);
        }
        for conflict in &conflicts {
            if conflict.conflict_type == "existing_plugin" || conflict.conflict_type == "path_exists" {
                eprintln!("Info: {}", conflict.message);
            }
        }

        let mut applied_ops: Vec<AppliedOp> = Vec::new();

        for binding in &diff.to_remove {
            let target_path = project.join(&binding.mount_path);
            match self.safe_remove_link(&target_path) {
                Ok(op) => {
                    result.removed.push(target_path.to_string_lossy().to_string());
                    applied_ops.push(op);
                }
                Err(e) => {
                    result.errors.push(format!("移除失败 {}: {}", binding.mount_path, e));
                    result.success = false;
                    self.rollback_ops(&applied_ops);
                    return Ok(result);
                }
            }
        }

        for binding in &diff.to_add {
            match self.apply_binding(binding, project, plugin_base_path) {
                Ok(mount_path) => {
                    result.created.push(mount_path.clone());
                    applied_ops.push(AppliedOp::Create {
                        path: mount_path,
                    });
                }
                Err(e) => {
                    result.errors.push(format!("应用绑定失败: {}", e));
                    result.success = false;
                    self.rollback_ops(&applied_ops);
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

            if target_path.exists() && !self.is_managed_link(&target_path)? {
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
                    conflict_type: if has_plugin_cfg { "existing_plugin" } else { "path_exists" }.to_string(),
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
                        let is_managed = self.is_managed_link(&path).unwrap_or(false);
                        let is_in_bindings = to_add.iter().any(|b| b.mount_path == mount_path)
                            || to_keep.iter().any(|b| b.mount_path == mount_path);
                        if !is_managed && !is_in_bindings {
                            conflicts.push(ConflictInfo {
                                conflict_type: "unmanaged_addon".to_string(),
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

        let backup_path = if target_path.exists() {
            let bak = target_path.with_extension("harbor-bak");
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

        let mount_result = match self.mount_strategy {
            MountStrategy::Symlink => {
                self.create_symlink_with_fallback(&source_path, &target_path)
                    .context("Failed to create symlink (and junction fallback)")
            }
            MountStrategy::Junction => {
                #[cfg(windows)]
                {
                    self.create_junction(&source_path, &target_path)
                        .context("Failed to create junction")
                }
                #[cfg(not(windows))]
                {
                    std::os::unix::fs::symlink(&source_path, &target_path)
                        .context("Failed to create symlink")
                }
            }
            MountStrategy::Copy => {
                copy_dir_all(&source_path, &target_path)
                    .map_err(|e| anyhow::anyhow!(e))
                    .with_context(|| format!("Failed to copy plugin from {} to {}", source_path.to_string_lossy(), target_path.to_string_lossy()))?;

                let harbor_marker = target_path.join(".harbor-managed");
                std::fs::write(&harbor_marker, "managed_by_godot_harbor")
                    .with_context(|| format!("Failed to write harbor marker at {}", harbor_marker.to_string_lossy()))?;

                Ok(())
            }
        };

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

    fn create_symlink_with_fallback(&self, source: &Path, target: &Path) -> Result<()> {
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(source, target)
                .context("Failed to create symlink")?;
        }

        #[cfg(windows)]
        {
            match std::os::windows::fs::symlink_dir(source, target) {
                Ok(_) => {}
                Err(_) => {
                    match self.create_junction_silent(source, target) {
                        Ok(_) => {}
                        Err(_) => {
                            self.copy_dir_fallback(source, target)
                                .context("Symlink and junction both failed, copy fallback also failed. Try changing mount strategy to Copy in settings.")?;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn create_junction_silent(&self, source: &Path, target: &Path) -> Result<()> {
        use std::process::Command;
        #[cfg(windows)]
        use std::os::windows::process::CommandExt;

        #[cfg(windows)]
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        #[cfg(windows)]
        {
            let source_str = source.to_string_lossy();
            let target_str = target.to_string_lossy();
            let mklink_cmd = format!("mklink /J \"{}\" \"{}\"", target_str, source_str);
            let mut cmd = Command::new("cmd");
            cmd.creation_flags(CREATE_NO_WINDOW);
            let output = cmd
                .args(&["/C", &mklink_cmd])
                .output()
                .context("Failed to execute mklink command")?;

            if !output.status.success() {
                anyhow::bail!("junction failed");
            }
        }

        Ok(())
    }

    fn copy_dir_fallback(&self, source: &Path, target: &Path) -> Result<()> {
        copy_dir_all(source, target)
            .map_err(|e| anyhow::anyhow!(e))
            .with_context(|| format!("Failed to copy plugin from {} to {}", source.to_string_lossy(), target.to_string_lossy()))?;

        let harbor_marker = target.join(".harbor-managed");
        std::fs::write(&harbor_marker, "managed_by_godot_harbor")
            .with_context(|| format!("Failed to write harbor marker at {}", harbor_marker.to_string_lossy()))?;

        Ok(())
    }

    fn is_managed_link(&self, path: &Path) -> Result<bool> {
        if !path.exists() {
            return Ok(false);
        }

        let metadata = fs::symlink_metadata(path)?;

        if metadata.file_type().is_symlink() {
            return Ok(true);
        }

        if self.is_junction(path) {
            return Ok(true);
        }

        if path.is_dir() && path.join(".harbor-managed").exists() {
            return Ok(true);
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

    fn safe_remove_link(&self, path: &Path) -> Result<AppliedOp> {
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
            let harbor_marker = path.join(".harbor-managed");
            if harbor_marker.exists() {
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

    fn rollback_ops(&self, ops: &[AppliedOp]) {
        for op in ops.iter().rev() {
            match op {
                AppliedOp::Create { path } => {
                    let p = Path::new(path);
                    if p.exists() {
                        if let Err(e) = self.safe_remove_link(p) {
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

    #[cfg(windows)]
    fn create_junction(&self, source: &Path, target: &Path) -> Result<()> {
        use std::process::Command;
        #[cfg(windows)]
        use std::os::windows::process::CommandExt;

        #[cfg(windows)]
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        #[cfg(windows)]
        {
            let source_str = source.to_string_lossy();
            let target_str = target.to_string_lossy();
            let mklink_cmd = format!("mklink /J \"{}\" \"{}\"", target_str, source_str);
            let mut cmd = Command::new("cmd");
            cmd.creation_flags(CREATE_NO_WINDOW);
            let output = cmd
                .args(&["/C", &mklink_cmd])
                .output()
                .context("Failed to execute mklink command")?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let stdout = String::from_utf8_lossy(&output.stdout);
                anyhow::bail!("Failed to create junction: {} (stdout: {})", stderr, stdout);
            }
        }

        #[cfg(not(windows))]
        {
            std::os::unix::fs::symlink(source, target)
                .context("Failed to create symlink")?;
        }

        Ok(())
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
        let linker = Linker::new(MountStrategy::Symlink);
        let current = vec![];
        let desired = vec![make_binding("p1", "pl1", "v1", "addons/foo")];

        let diff = linker.compute_diff(&current, &desired);
        assert_eq!(diff.to_add.len(), 1);
        assert_eq!(diff.to_remove.len(), 0);
        assert_eq!(diff.to_keep.len(), 0);
    }

    #[test]
    fn test_compute_diff_remove_binding() {
        let linker = Linker::new(MountStrategy::Symlink);
        let current = vec![make_binding("p1", "pl1", "v1", "addons/foo")];
        let desired = vec![];

        let diff = linker.compute_diff(&current, &desired);
        assert_eq!(diff.to_add.len(), 0);
        assert_eq!(diff.to_remove.len(), 1);
        assert_eq!(diff.to_keep.len(), 0);
    }

    #[test]
    fn test_compute_diff_keep_unchanged() {
        let linker = Linker::new(MountStrategy::Symlink);
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
        let linker = Linker::new(MountStrategy::Symlink);
        let current = vec![make_binding("p1", "pl1", "v1", "addons/foo")];
        let desired = vec![make_binding("p1", "pl1", "v2", "addons/foo")];

        let diff = linker.compute_diff(&current, &desired);
        assert_eq!(diff.to_add.len(), 1);
        assert_eq!(diff.to_remove.len(), 1);
        assert_eq!(diff.to_keep.len(), 0);
    }

    #[test]
    fn test_compute_diff_mount_path_change() {
        let linker = Linker::new(MountStrategy::Symlink);
        let current = vec![make_binding("p1", "pl1", "v1", "addons/foo")];
        let desired = vec![make_binding("p1", "pl1", "v1", "addons/bar")];

        let diff = linker.compute_diff(&current, &desired);
        assert_eq!(diff.to_add.len(), 1);
        assert_eq!(diff.to_remove.len(), 1);
    }

    #[test]
    fn test_compute_diff_mixed_operations() {
        let linker = Linker::new(MountStrategy::Symlink);
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
        let linker = Linker::new(MountStrategy::Symlink);
        let result = linker.pre_check("/nonexistent/path", &[], "/plugins");
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.contains("项目路径不存在")));
    }

    #[test]
    fn test_pre_check_missing_project_godot() {
        let dir = tempfile::TempDir::new().unwrap();
        let linker = Linker::new(MountStrategy::Symlink);
        let result = linker.pre_check(dir.path().to_str().unwrap(), &[], "/plugins");
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.contains("project.godot")));
    }

    #[test]
    fn test_pre_check_valid_project() {
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::write(dir.path().join("project.godot"), "[application]\n").unwrap();
        let linker = Linker::new(MountStrategy::Symlink);
        let result = linker.pre_check(dir.path().to_str().unwrap(), &[], "/plugins");
        assert!(result.is_valid);
    }

    #[test]
    fn test_pre_check_missing_plugin_source() {
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::write(dir.path().join("project.godot"), "[application]\n").unwrap();
        let linker = Linker::new(MountStrategy::Symlink);
        let bindings = vec![make_binding("p1", "pl1", "v1", "addons/foo")];
        let result = linker.pre_check(dir.path().to_str().unwrap(), &bindings, "/nonexistent/plugins");
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.contains("插件源不存在")));
    }

    #[test]
    fn test_is_newer_logic() {
        assert!(Linker::new(MountStrategy::Symlink).compute_diff(&[], &[]).to_add.is_empty());
    }

    #[test]
    fn test_safe_remove_link_harbor_managed_dir() {
        let dir = tempfile::TempDir::new().unwrap();
        let managed_dir = dir.path().join("addons/my_plugin");
        fs::create_dir_all(&managed_dir).unwrap();
        fs::write(managed_dir.join(".harbor-managed"), "managed_by_godot_harbor").unwrap();
        fs::write(managed_dir.join("plugin.cfg"), "name=\"test\"").unwrap();

        let linker = Linker::new(MountStrategy::Copy);
        let result = linker.safe_remove_link(&managed_dir);
        assert!(result.is_ok());
        assert!(!managed_dir.exists());
    }

    #[test]
    fn test_safe_remove_link_non_managed_dir_preserved() {
        let dir = tempfile::TempDir::new().unwrap();
        let original_dir = dir.path().join("addons/original_plugin");
        fs::create_dir_all(&original_dir).unwrap();
        fs::write(original_dir.join("plugin.cfg"), "name=\"original\"").unwrap();

        let linker = Linker::new(MountStrategy::Copy);
        let result = linker.safe_remove_link(&original_dir);
        assert!(result.is_ok());
        assert!(original_dir.exists());
        assert!(original_dir.join("plugin.cfg").exists());
    }

    #[test]
    fn test_safe_remove_link_nonexistent_path() {
        let linker = Linker::new(MountStrategy::Copy);
        let result = linker.safe_remove_link(Path::new("/nonexistent/path/xyz"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_safe_remove_link_regular_file() {
        let dir = tempfile::TempDir::new().unwrap();
        let file_path = dir.path().join("some_file.txt");
        fs::write(&file_path, "content").unwrap();

        let linker = Linker::new(MountStrategy::Copy);
        let result = linker.safe_remove_link(&file_path);
        assert!(result.is_ok());
        assert!(!file_path.exists());
    }

    #[test]
    fn test_is_managed_link_with_marker() {
        let dir = tempfile::TempDir::new().unwrap();
        let managed_dir = dir.path().join("plugin_dir");
        fs::create_dir_all(&managed_dir).unwrap();
        fs::write(managed_dir.join(".harbor-managed"), "managed").unwrap();

        let linker = Linker::new(MountStrategy::Copy);
        assert!(linker.is_managed_link(&managed_dir).unwrap());
    }

    #[test]
    fn test_is_managed_link_without_marker() {
        let dir = tempfile::TempDir::new().unwrap();
        let plain_dir = dir.path().join("plugin_dir");
        fs::create_dir_all(&plain_dir).unwrap();
        fs::write(plain_dir.join("plugin.cfg"), "name=\"test\"").unwrap();

        let linker = Linker::new(MountStrategy::Copy);
        assert!(!linker.is_managed_link(&plain_dir).unwrap());
    }

    #[test]
    fn test_is_managed_link_nonexistent() {
        let linker = Linker::new(MountStrategy::Copy);
        assert!(!linker.is_managed_link(Path::new("/nonexistent")).unwrap());
    }

    #[test]
    fn test_apply_bindings_copy_mode_creates_marker() {
        let project_dir = tempfile::TempDir::new().unwrap();
        fs::write(project_dir.path().join("project.godot"), "[application]\n").unwrap();

        let plugin_base = tempfile::TempDir::new().unwrap();
        let plugin_payload = plugin_base.path().join("pl1").join("v1").join("payload");
        fs::create_dir_all(&plugin_payload).unwrap();
        fs::write(plugin_payload.join("plugin.cfg"), "name=\"test_plugin\"").unwrap();
        fs::write(plugin_payload.join("script.gd"), "extends Node").unwrap();

        let linker = Linker::new(MountStrategy::Copy);
        let binding = make_binding("p1", "pl1", "v1", "addons/test_plugin");
        let result = linker.apply_bindings(
            project_dir.path().to_str().unwrap(),
            &[],
            &[binding],
            plugin_base.path().to_str().unwrap(),
        ).unwrap();

        assert!(result.success);
        let installed = project_dir.path().join("addons/test_plugin");
        assert!(installed.exists());
        assert!(installed.join(".harbor-managed").exists());
        assert!(installed.join("plugin.cfg").exists());
    }

    #[test]
    fn test_apply_bindings_copy_mode_then_remove_preserves_original() {
        let project_dir = tempfile::TempDir::new().unwrap();
        fs::write(project_dir.path().join("project.godot"), "[application]\n").unwrap();

        let plugin_base = tempfile::TempDir::new().unwrap();
        let plugin_payload = plugin_base.path().join("pl1").join("v1").join("payload");
        fs::create_dir_all(&plugin_payload).unwrap();
        fs::write(plugin_payload.join("plugin.cfg"), "name=\"managed\"").unwrap();

        let linker = Linker::new(MountStrategy::Copy);

        let binding = make_binding("p1", "pl1", "v1", "addons/managed_plugin");
        let result = linker.apply_bindings(
            project_dir.path().to_str().unwrap(),
            &[],
            &[binding.clone()],
            plugin_base.path().to_str().unwrap(),
        ).unwrap();
        assert!(result.success);
        assert!(project_dir.path().join("addons/managed_plugin").join(".harbor-managed").exists());

        let managed_path = project_dir.path().join("addons/managed_plugin");
        let result = linker.safe_remove_link(&managed_path);
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
        fs::write(managed.join(".harbor-managed"), "managed").unwrap();
        fs::write(managed.join("plugin.cfg"), "name=\"managed\"").unwrap();

        let original = addons.join("original_plugin");
        fs::create_dir_all(&original).unwrap();
        fs::write(original.join("plugin.cfg"), "name=\"original\"").unwrap();

        let linker = Linker::new(MountStrategy::Copy);
        let result = linker.safe_remove_link(&managed);
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

        let linker = Linker::new(MountStrategy::Copy);
        let _binding = make_binding("p1", "pl1", "v1", "addons/non_managed");

        let result = linker.safe_remove_link(&project_dir.path().join("addons/non_managed"));
        assert!(result.is_ok());
        assert!(non_managed.exists());
        assert!(non_managed.join("plugin.cfg").exists());
    }

    #[test]
    fn test_apply_bindings_upgrade_version() {
        let project_dir = tempfile::TempDir::new().unwrap();
        fs::write(project_dir.path().join("project.godot"), "[application]\n").unwrap();

        let plugin_base = tempfile::TempDir::new().unwrap();

        let v1_payload = plugin_base.path().join("pl1").join("v1").join("payload");
        fs::create_dir_all(&v1_payload).unwrap();
        fs::write(v1_payload.join("plugin.cfg"), "name=\"test\"\nversion=\"1.0.0\"").unwrap();

        let v2_payload = plugin_base.path().join("pl1").join("v2").join("payload");
        fs::create_dir_all(&v2_payload).unwrap();
        fs::write(v2_payload.join("plugin.cfg"), "name=\"test\"\nversion=\"2.0.0\"").unwrap();

        let linker = Linker::new(MountStrategy::Copy);

        let binding_v1 = make_binding("p1", "pl1", "v1", "addons/test");
        let result = linker.apply_bindings(
            project_dir.path().to_str().unwrap(),
            &[],
            &[binding_v1.clone()],
            plugin_base.path().to_str().unwrap(),
        ).unwrap();
        assert!(result.success);
        assert!(result.created.len() == 1);

        let binding_v2 = make_binding("p1", "pl1", "v2", "addons/test");
        let result = linker.apply_bindings(
            project_dir.path().to_str().unwrap(),
            &[binding_v1],
            &[binding_v2],
            plugin_base.path().to_str().unwrap(),
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

        let linker = Linker::new(MountStrategy::Copy);
        let binding = make_binding("p1", "pl1", "v1", "addons/existing_plugin");

        let conflicts = linker.check_conflicts(
            project_dir.path().to_str().unwrap(),
            &[binding],
            &[],
        ).unwrap();

        assert!(!conflicts.is_empty());
        assert!(conflicts.iter().any(|c| c.conflict_type == "existing_plugin"));
    }

    #[test]
    fn test_check_conflicts_managed_existing_no_conflict() {
        let project_dir = tempfile::TempDir::new().unwrap();
        fs::write(project_dir.path().join("project.godot"), "[application]\n").unwrap();

        let managed = project_dir.path().join("addons/managed_plugin");
        fs::create_dir_all(&managed).unwrap();
        fs::write(managed.join(".harbor-managed"), "managed").unwrap();
        fs::write(managed.join("plugin.cfg"), "name=\"managed\"").unwrap();

        let linker = Linker::new(MountStrategy::Copy);
        let binding = make_binding("p1", "pl1", "v1", "addons/managed_plugin");

        let conflicts = linker.check_conflicts(
            project_dir.path().to_str().unwrap(),
            &[binding],
            &[],
        ).unwrap();

        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_apply_bindings_multiple_plugins() {
        let project_dir = tempfile::TempDir::new().unwrap();
        fs::write(project_dir.path().join("project.godot"), "[application]\n").unwrap();

        let plugin_base = tempfile::TempDir::new().unwrap();

        for (pl_id, ver_id) in [("pl1", "v1"), ("pl2", "v1")] {
            let payload = plugin_base.path().join(pl_id).join(ver_id).join("payload");
            fs::create_dir_all(&payload).unwrap();
            fs::write(payload.join("plugin.cfg"), format!("name=\"{}\"", pl_id)).unwrap();
        }

        let linker = Linker::new(MountStrategy::Copy);
        let bindings = vec![
            make_binding("p1", "pl1", "v1", "addons/pl1"),
            make_binding("p1", "pl2", "v1", "addons/pl2"),
        ];

        let result = linker.apply_bindings(
            project_dir.path().to_str().unwrap(),
            &[],
            &bindings,
            plugin_base.path().to_str().unwrap(),
        ).unwrap();

        assert!(result.success);
        assert_eq!(result.created.len(), 2);
        assert!(project_dir.path().join("addons/pl1").join(".harbor-managed").exists());
        assert!(project_dir.path().join("addons/pl2").join(".harbor-managed").exists());
    }

    #[test]
    fn test_apply_bindings_with_subdirectory() {
        let project_dir = tempfile::TempDir::new().unwrap();
        fs::write(project_dir.path().join("project.godot"), "[application]\n").unwrap();

        let plugin_base = tempfile::TempDir::new().unwrap();
        let payload = plugin_base.path().join("pl1").join("v1").join("payload");
        let subdir = payload.join("addons").join("my_plugin");
        fs::create_dir_all(&subdir).unwrap();
        fs::write(subdir.join("plugin.cfg"), "name=\"sub_plugin\"").unwrap();

        let linker = Linker::new(MountStrategy::Copy);
        let mut binding = make_binding("p1", "pl1", "v1", "addons/my_plugin");
        binding.subdirectory = "addons/my_plugin".to_string();

        let result = linker.apply_bindings(
            project_dir.path().to_str().unwrap(),
            &[],
            &[binding],
            plugin_base.path().to_str().unwrap(),
        ).unwrap();

        assert!(result.success);
        let installed = project_dir.path().join("addons/my_plugin");
        assert!(installed.exists());
        assert!(installed.join("plugin.cfg").exists());
        assert!(installed.join(".harbor-managed").exists());
    }

    #[test]
    fn test_apply_bindings_existing_non_managed_merge_install() {
        let project_dir = tempfile::TempDir::new().unwrap();
        fs::write(project_dir.path().join("project.godot"), "[application]\n").unwrap();

        let existing = project_dir.path().join("addons/godot_mcp");
        fs::create_dir_all(&existing).unwrap();
        fs::write(existing.join("plugin.cfg"), "name=\"Godot MCP\"").unwrap();
        fs::write(existing.join("user_custom.gd"), "extends Node2D").unwrap();
        assert!(!existing.join(".harbor-managed").exists());

        let plugin_base = tempfile::TempDir::new().unwrap();
        let plugin_payload = plugin_base.path().join("pl1").join("v1").join("payload");
        fs::create_dir_all(&plugin_payload).unwrap();
        fs::write(plugin_payload.join("plugin.cfg"), "name=\"Godot MCP\"").unwrap();
        fs::write(plugin_payload.join("new_script.gd"), "extends Node").unwrap();

        let linker = Linker::new(MountStrategy::Copy);
        let binding = make_binding("p1", "pl1", "v1", "addons/godot_mcp");

        let result = linker.apply_bindings(
            project_dir.path().to_str().unwrap(),
            &[],
            &[binding],
            plugin_base.path().to_str().unwrap(),
        ).unwrap();

        assert!(result.success);
        assert!(existing.join(".harbor-managed").exists());
        assert!(existing.join("plugin.cfg").exists());
        assert!(existing.join("new_script.gd").exists());
        let backup = project_dir.path().join("addons/godot_mcp.harbor-bak");
        assert!(!backup.exists());
    }

    #[test]
    fn test_apply_bindings_existing_managed_replaced() {
        let project_dir = tempfile::TempDir::new().unwrap();
        fs::write(project_dir.path().join("project.godot"), "[application]\n").unwrap();

        let managed = project_dir.path().join("addons/old_plugin");
        fs::create_dir_all(&managed).unwrap();
        fs::write(managed.join(".harbor-managed"), "managed").unwrap();
        fs::write(managed.join("plugin.cfg"), "name=\"Old\"").unwrap();

        let plugin_base = tempfile::TempDir::new().unwrap();
        let plugin_payload = plugin_base.path().join("pl1").join("v1").join("payload");
        fs::create_dir_all(&plugin_payload).unwrap();
        fs::write(plugin_payload.join("plugin.cfg"), "name=\"New\"").unwrap();

        let linker = Linker::new(MountStrategy::Copy);
        let binding = make_binding("p1", "pl1", "v1", "addons/old_plugin");

        let result = linker.apply_bindings(
            project_dir.path().to_str().unwrap(),
            &[],
            &[binding],
            plugin_base.path().to_str().unwrap(),
        ).unwrap();

        assert!(result.success);
        assert!(managed.exists());
        let cfg = fs::read_to_string(managed.join("plugin.cfg")).unwrap();
        assert_eq!(cfg, "name=\"New\"");
    }

    #[test]
    fn test_apply_bindings_non_managed_merge_preserves_custom_files() {
        let project_dir = tempfile::TempDir::new().unwrap();
        fs::write(project_dir.path().join("project.godot"), "[application]\n").unwrap();

        let existing = project_dir.path().join("addons/existing");
        fs::create_dir_all(&existing).unwrap();
        fs::write(existing.join("plugin.cfg"), "name=\"Original\"").unwrap();
        fs::write(existing.join("custom_script.gd"), "extends Node2D").unwrap();

        let plugin_base = tempfile::TempDir::new().unwrap();
        let plugin_payload = plugin_base.path().join("pl1").join("v1").join("payload");
        fs::create_dir_all(&plugin_payload).unwrap();
        fs::write(plugin_payload.join("plugin.cfg"), "name=\"Different\"").unwrap();
        fs::write(plugin_payload.join("new_feature.gd"), "extends Node3D").unwrap();

        let linker = Linker::new(MountStrategy::Copy);
        let binding = make_binding("p1", "pl1", "v1", "addons/existing");

        let result = linker.apply_bindings(
            project_dir.path().to_str().unwrap(),
            &[],
            &[binding],
            plugin_base.path().to_str().unwrap(),
        ).unwrap();

        assert!(result.success);
        assert!(existing.join("new_feature.gd").exists());
        assert!(existing.join(".harbor-managed").exists());
        let backup = project_dir.path().join("addons/existing.harbor-bak");
        assert!(!backup.exists());
    }

    #[test]
    fn test_apply_bindings_symlink_strategy_non_managed_gets_marker() {
        let project_dir = tempfile::TempDir::new().unwrap();
        fs::write(project_dir.path().join("project.godot"), "[application]\n").unwrap();

        let target = project_dir.path().join("addons/existing");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("plugin.cfg"), "name=\"Original\"").unwrap();

        let plugin_base = tempfile::TempDir::new().unwrap();
        let plugin_payload = plugin_base.path().join("pl1").join("v1").join("payload");
        fs::create_dir_all(&plugin_payload).unwrap();
        fs::write(plugin_payload.join("plugin.cfg"), "name=\"Different\"").unwrap();

        let linker = Linker::new(MountStrategy::Symlink);
        let binding = make_binding("p1", "pl1", "v1", "addons/existing");

        let result = linker.apply_bindings(
            project_dir.path().to_str().unwrap(),
            &[],
            &[binding],
            plugin_base.path().to_str().unwrap(),
        ).unwrap();

        assert!(result.success);
        assert!(target.exists());
        let backup = project_dir.path().join("addons/existing.harbor-bak");
        assert!(!backup.exists());
    }
}
