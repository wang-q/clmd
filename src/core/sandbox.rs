//! Sandbox mode for clmd.
//!
//! This module provides sandboxing capabilities for safe document conversion,
//! inspired by Pandoc's sandbox mode. When enabled, all file system and network
//! operations are restricted.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::core::error::ClmdError;

/// Sandbox mode configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SandboxMode {
    /// No sandboxing - all operations allowed.
    Disabled,
    /// Relaxed sandboxing - allows access to resource paths and data directory.
    #[default]
    Relaxed,
    /// Strict sandboxing - only allows access to explicitly allowed paths.
    Strict,
}

/// A sandbox policy that controls file system and network access.
#[derive(Debug, Clone)]
pub struct SandboxPolicy {
    /// The sandbox mode.
    pub mode: SandboxMode,
    /// Explicitly allowed paths (for strict mode).
    pub allowed_paths: HashSet<PathBuf>,
    /// Blocked paths (applies to all modes).
    pub blocked_paths: HashSet<PathBuf>,
    /// Whether network access is allowed.
    pub allow_network: bool,
    /// Whether file writes are allowed.
    pub allow_writes: bool,
    /// Maximum file size that can be read (in bytes).
    pub max_file_size: Option<usize>,
    /// Maximum total bytes that can be read.
    pub max_total_read: Option<usize>,
}

impl Default for SandboxPolicy {
    fn default() -> Self {
        Self {
            mode: SandboxMode::default(),
            allowed_paths: HashSet::new(),
            blocked_paths: HashSet::new(),
            allow_network: true,
            allow_writes: true,
            max_file_size: None,
            max_total_read: None,
        }
    }
}

impl SandboxPolicy {
    /// Create a new sandbox policy with the given mode.
    pub fn new(mode: SandboxMode) -> Self {
        Self {
            mode,
            ..Default::default()
        }
    }

    /// Create a strict sandbox policy.
    pub fn strict() -> Self {
        Self::new(SandboxMode::Strict)
    }

    /// Create a relaxed sandbox policy.
    pub fn relaxed() -> Self {
        Self::new(SandboxMode::Relaxed)
    }

    /// Disable sandboxing.
    pub fn disabled() -> Self {
        Self::new(SandboxMode::Disabled)
    }

    /// Add an allowed path.
    pub fn allow_path(mut self, path: impl AsRef<Path>) -> Self {
        self.allowed_paths.insert(path.as_ref().to_path_buf());
        self
    }

    /// Add a blocked path.
    pub fn block_path(mut self, path: impl AsRef<Path>) -> Self {
        self.blocked_paths.insert(path.as_ref().to_path_buf());
        self
    }

    /// Disable network access.
    pub fn without_network(mut self) -> Self {
        self.allow_network = false;
        self
    }

    /// Disable file writes.
    pub fn without_writes(mut self) -> Self {
        self.allow_writes = false;
        self
    }

    /// Set maximum file size.
    pub fn with_max_file_size(mut self, size: usize) -> Self {
        self.max_file_size = Some(size);
        self
    }

    /// Set maximum total read bytes.
    pub fn with_max_total_read(mut self, size: usize) -> Self {
        self.max_total_read = Some(size);
        self
    }

    /// Check if a path is allowed.
    pub fn is_path_allowed(&self, path: &Path, resource_paths: &[PathBuf]) -> bool {
        // Check blocked paths first
        for blocked in &self.blocked_paths {
            if path.starts_with(blocked) {
                return false;
            }
        }

        match self.mode {
            SandboxMode::Disabled => true,
            SandboxMode::Relaxed => {
                // Allow access to resource paths and current directory
                for allowed in resource_paths {
                    if path.starts_with(allowed) {
                        return true;
                    }
                }
                // Allow absolute paths that exist
                path.is_absolute() && path.exists()
            }
            SandboxMode::Strict => {
                // Only allow explicitly allowed paths
                for allowed in &self.allowed_paths {
                    if path.starts_with(allowed) {
                        return true;
                    }
                }
                false
            }
        }
    }

    /// Check if network access is allowed.
    pub fn is_network_allowed(&self) -> bool {
        self.allow_network
    }

    /// Check if file writes are allowed.
    pub fn are_writes_allowed(&self) -> bool {
        self.allow_writes
    }

    /// Validate file size.
    pub fn validate_file_size(&self, size: usize) -> Result<(), ClmdError> {
        if let Some(max) = self.max_file_size {
            if size > max {
                return Err(ClmdError::sandbox_error(format!(
                    "File size {} exceeds maximum allowed {}",
                    size, max
                )));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_policy_disabled() {
        let policy = SandboxPolicy::disabled();
        assert!(policy.is_path_allowed(Path::new("/any/path"), &[]));
        assert!(policy.is_network_allowed());
        assert!(policy.are_writes_allowed());
    }

    #[test]
    fn test_sandbox_policy_strict() {
        let policy = SandboxPolicy::strict()
            .allow_path("/allowed")
            .block_path("/allowed/blocked");

        assert!(policy.is_path_allowed(Path::new("/allowed/file.txt"), &[]));
        assert!(!policy.is_path_allowed(Path::new("/other/file.txt"), &[]));
        assert!(!policy.is_path_allowed(Path::new("/allowed/blocked/file.txt"), &[]));
    }

    #[test]
    fn test_sandbox_policy_relaxed() {
        let policy = SandboxPolicy::relaxed();
        let resources = vec![PathBuf::from("/resources")];

        assert!(policy.is_path_allowed(Path::new("/resources/file.txt"), &resources));
    }

    #[test]
    fn test_sandbox_policy_network() {
        let policy = SandboxPolicy::disabled().without_network();
        assert!(!policy.is_network_allowed());
    }

    #[test]
    fn test_sandbox_policy_writes() {
        let policy = SandboxPolicy::disabled().without_writes();
        assert!(!policy.are_writes_allowed());
    }

    #[test]
    fn test_sandbox_policy_file_size() {
        let policy = SandboxPolicy::disabled().with_max_file_size(100);
        assert!(policy.validate_file_size(50).is_ok());
        assert!(policy.validate_file_size(150).is_err());
    }
}
