//! Sandbox mode for clmd.
//!
//! This module provides sandboxing capabilities for safe document conversion,
//! inspired by Pandoc's sandbox mode. When enabled, all file system and network
//! operations are restricted.
//!
//! # Example
//!
//! ```ignore
//! use clmd::core::{ClmdIO, ClmdMonad, SandboxMode};
//! use std::path::Path;
//!
//! let monad = ClmdIO::new().with_sandbox_mode(SandboxMode::Strict);
//! // File operations outside allowed paths will fail
//! ```

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
            mode: SandboxMode::Disabled,
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

/// A sandboxed wrapper for monad operations.
#[derive(Debug, Clone)]
pub struct SandboxedMonad<M> {
    inner: M,
    policy: SandboxPolicy,
    resource_paths: Vec<PathBuf>,
    bytes_read: std::sync::Arc<std::sync::atomic::AtomicUsize>,
}

impl<M> SandboxedMonad<M> {
    /// Create a new sandboxed monad.
    pub fn new(inner: M, policy: SandboxPolicy) -> Self {
        Self {
            inner,
            policy,
            resource_paths: vec![PathBuf::from(".")],
            bytes_read: std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        }
    }

    /// Set resource paths.
    pub fn with_resource_paths(mut self, paths: Vec<PathBuf>) -> Self {
        self.resource_paths = paths;
        self
    }

    /// Get the inner monad.
    pub fn inner(&self) -> &M {
        &self.inner
    }

    /// Get the sandbox policy.
    pub fn policy(&self) -> &SandboxPolicy {
        &self.policy
    }

    /// Check if a path is allowed.
    pub fn check_path(&self, path: &Path) -> Result<(), ClmdError> {
        if !self.policy.is_path_allowed(path, &self.resource_paths) {
            return Err(ClmdError::sandbox_error(format!(
                "Access to path '{}' is not allowed in sandbox mode",
                path.display()
            )));
        }
        Ok(())
    }

    /// Check if network access is allowed.
    pub fn check_network(&self) -> Result<(), ClmdError> {
        if !self.policy.is_network_allowed() {
            return Err(ClmdError::sandbox_error(
                "Network access is not allowed in sandbox mode",
            ));
        }
        Ok(())
    }

    /// Check if writes are allowed.
    pub fn check_write(&self) -> Result<(), ClmdError> {
        if !self.policy.are_writes_allowed() {
            return Err(ClmdError::sandbox_error(
                "File writes are not allowed in sandbox mode",
            ));
        }
        Ok(())
    }

    /// Track bytes read.
    pub fn track_read(&self, bytes: usize) -> Result<(), ClmdError> {
        if let Some(max) = self.policy.max_total_read {
            let current = self
                .bytes_read
                .fetch_add(bytes, std::sync::atomic::Ordering::SeqCst);
            if current + bytes > max {
                return Err(ClmdError::sandbox_error(format!(
                    "Total read bytes {} exceeds maximum allowed {}",
                    current + bytes,
                    max
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

    #[test]
    fn test_sandboxed_monad_path_check() {
        let policy = SandboxPolicy::strict().allow_path("/allowed");
        let monad = SandboxedMonad::new((), policy);

        assert!(monad.check_path(Path::new("/allowed/file.txt")).is_ok());
        assert!(monad.check_path(Path::new("/other/file.txt")).is_err());
    }

    #[test]
    fn test_sandboxed_monad_network_check() {
        let policy = SandboxPolicy::disabled().without_network();
        let monad = SandboxedMonad::new((), policy);

        assert!(monad.check_network().is_err());
    }

    #[test]
    fn test_sandboxed_monad_write_check() {
        let policy = SandboxPolicy::disabled().without_writes();
        let monad = SandboxedMonad::new((), policy);

        assert!(monad.check_write().is_err());
    }
}
