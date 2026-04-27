use std::fs;
use std::path::Path;
use anyhow::{Result, Context};
use crate::models::{ProjectBinding, MountStrategy, ApplyResult, ConflictInfo};

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
        if !conflicts.is_empty() {
            for conflict in &conflicts {
                result.errors.push(format!("冲突: {}", conflict.message));
            }
            result.success = false;
            return Ok(result);
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
                conflicts.push(ConflictInfo {
                    conflict_type: "path_exists".to_string(),
                    path: target_path.to_string_lossy().to_string(),
                    message: format!(
                        "目标路径已存在且非 Harbor 管理: {}，继续操作将覆盖该目录",
                        binding.mount_path
                    ),
                });
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

        if target_path.exists() {
            self.safe_remove_link(&target_path)
                .with_context(|| format!("Failed to remove existing target path: {}", target_path.to_string_lossy()))?;
        }

        if let Some(parent) = target_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .context("Failed to create parent directory for mount target")?;
            }
        }

        match self.mount_strategy {
            MountStrategy::Symlink => {
                let symlink_result = self.create_symlink_with_fallback(&source_path, &target_path);
                symlink_result.context("Failed to create symlink (and junction fallback)")?;
            }
            MountStrategy::Junction => {
                #[cfg(windows)]
                {
                    self.create_junction(&source_path, &target_path)
                        .context("Failed to create junction")?;
                }
                #[cfg(not(windows))]
                {
                    std::os::unix::fs::symlink(&source_path, &target_path)
                        .context("Failed to create symlink")?;
                }
            }
            MountStrategy::Copy => {
                self.copy_dir_recursive(&source_path, &target_path)
                    .with_context(|| format!("Failed to copy plugin from {} to {}", source_path.to_string_lossy(), target_path.to_string_lossy()))?;
            }
        }

        Ok(target_path.to_string_lossy().to_string())
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
                    self.create_junction(source, target)
                        .context("Symlink failed (likely due to permissions), and junction fallback also failed. Try running as administrator or change mount strategy to Junction/Copy in settings.")?;
                }
            }
        }

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
            fs::remove_dir_all(path)
                .with_context(|| format!("Failed to remove directory: {}", path.to_string_lossy()))?;
            return Ok(AppliedOp::Remove {
                path: path.to_path_buf(),
                was_symlink: false,
            });
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

        let output = Command::new("cmd")
            .args(&["/C", "mklink", "/J"])
            .arg(target)
            .arg(source)
            .output()
            .context("Failed to execute mklink command")?;

        if !output.status.success() {
            anyhow::bail!("Failed to create junction: {}", String::from_utf8_lossy(&output.stderr));
        }

        Ok(())
    }

    fn copy_dir_recursive(&self, src: &Path, dst: &Path) -> Result<()> {
        if !src.exists() {
            anyhow::bail!("Source directory does not exist: {}", src.to_string_lossy());
        }

        if !dst.exists() {
            fs::create_dir_all(dst)
                .with_context(|| format!("Failed to create destination directory: {}", dst.to_string_lossy()))?;
        }

        for entry in fs::read_dir(src)
            .with_context(|| format!("Failed to read source directory: {}", src.to_string_lossy()))? {
            let entry = entry
                .with_context(|| format!("Failed to read directory entry in: {}", src.to_string_lossy()))?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());

            if src_path.is_dir() {
                self.copy_dir_recursive(&src_path, &dst_path)
                    .with_context(|| format!("Failed to copy directory: {} to {}", src_path.to_string_lossy(), dst_path.to_string_lossy()))?;
            } else {
                fs::copy(&src_path, &dst_path)
                    .with_context(|| format!("Failed to copy file: {} to {}", src_path.to_string_lossy(), dst_path.to_string_lossy()))?;
            }
        }

        Ok(())
    }
}

enum AppliedOp {
    Create { path: String },
    Remove { path: std::path::PathBuf, was_symlink: bool },
    None,
}
