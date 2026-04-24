use std::fs;
use std::path::Path;
use anyhow::{Result, Context};
use crate::models::{ProjectBinding, MountStrategy, ApplyResult, ConflictInfo};

pub struct Linker {
    mount_strategy: MountStrategy,
}

impl Linker {
    pub fn new(mount_strategy: MountStrategy) -> Self {
        Self { mount_strategy }
    }

    pub fn apply_bindings(
        &self,
        project_path: &str,
        bindings: &[ProjectBinding],
        plugin_base_path: &str,
    ) -> Result<ApplyResult> {
        let mut result = ApplyResult {
            success: true,
            created: Vec::new(),
            removed: Vec::new(),
            errors: Vec::new(),
        };

        let project = Path::new(project_path);
        let addons_dir = project.join("addons");
        
        if !addons_dir.exists() {
            fs::create_dir_all(&addons_dir)
                .context("Failed to create addons directory")?;
        }

        self.cleanup_old_links(&addons_dir, &mut result)?;

        for binding in bindings {
            match self.apply_binding(binding, &addons_dir, plugin_base_path) {
                Ok(mount_path) => {
                    result.created.push(mount_path);
                }
                Err(e) => {
                    result.errors.push(format!("Failed to apply binding: {}", e));
                    result.success = false;
                }
            }
        }

        Ok(result)
    }

    fn cleanup_old_links(&self, addons_dir: &Path, result: &mut ApplyResult) -> Result<()> {
        if !addons_dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(addons_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if self.is_managed_link(&path)? {
                self.remove_link(&path)?;
                result.removed.push(path.to_string_lossy().to_string());
            }
        }

        Ok(())
    }

    fn apply_binding(
        &self,
        binding: &ProjectBinding,
        addons_dir: &Path,
        plugin_base_path: &str,
    ) -> Result<String> {
        let plugin_path = Path::new(plugin_base_path)
            .join(&binding.plugin_id)
            .join(&binding.version_id)
            .join("payload");
        
        if !plugin_path.exists() {
            anyhow::bail!("Plugin payload directory does not exist: {}", plugin_path.to_string_lossy());
        }
        
        // For now, use the plugin_path directly as the source path
        // This is a temporary fix until we can properly map unit_id to subdirectory
        let source_path = plugin_path;

        let target_path = addons_dir.join(&binding.mount_path);

        if target_path.exists() {
            self.remove_link(&target_path)
                .with_context(|| format!("Failed to remove existing target path: {}", target_path.to_string_lossy()))?;
        }

        match self.mount_strategy {
            MountStrategy::Symlink => {
                #[cfg(unix)]
                {
                    std::os::unix::fs::symlink(&source_path, &target_path)
                        .context("Failed to create symlink")?;
                }
                #[cfg(windows)]
                {
                    std::os::windows::fs::symlink_dir(&source_path, &target_path)
                        .context("Failed to create symlink")?;
                }
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

    fn is_managed_link(&self, path: &Path) -> Result<bool> {
        if !path.exists() {
            return Ok(false);
        }

        let metadata = fs::symlink_metadata(path)?;
        
        Ok(metadata.file_type().is_symlink() || self.is_junction(path))
    }

    fn is_junction(&self, path: &Path) -> bool {
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            if let Ok(metadata) = fs::metadata(path) {
                const IO_REPARSE_TAG_MOUNT_POINT: u32 = 0xA0000003;
                return (metadata.file_attributes() & 0x400) != 0;
            }
        }
        false
    }

    fn remove_link(&self, path: &Path) -> Result<()> {
        if path.is_dir() {
            fs::remove_dir_all(path)?;
        } else {
            fs::remove_file(path)?;
        }
        Ok(())
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

    pub fn check_conflicts(
        &self,
        project_path: &str,
        bindings: &[ProjectBinding],
    ) -> Result<Vec<ConflictInfo>> {
        let mut conflicts = Vec::new();
        
        let project = Path::new(project_path);
        let addons_dir = project.join("addons");
        
        for binding in bindings {
            let target_path = addons_dir.join(&binding.mount_path);
            
            if target_path.exists() && !self.is_managed_link(&target_path)? {
                conflicts.push(ConflictInfo {
                    conflict_type: "path_exists".to_string(),
                    path: target_path.to_string_lossy().to_string(),
                    message: format!("Target path already exists: {}", binding.mount_path),
                });
            }
        }
        
        Ok(conflicts)
    }
}
