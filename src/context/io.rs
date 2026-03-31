//! IO Context implementation for real file operations.
//!
//! This module provides the [`IoContext`] struct, which implements
//! the [`ClmdContext`] trait for real file system operations.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::context::{common, ClmdContext, CommonState, LogLevel, LogMessage, Verbosity};
use crate::error::ClmdError;
use crate::mediabag::MediaItem;

/// IO Context for real file operations.
///
/// This context performs actual file system operations and is suitable
/// for production use.
///
/// # Example
///
/// ```
/// use clmd::context::{ClmdContext, IoContext, LogLevel};
///
/// let mut ctx = IoContext::new();
/// ctx.info("Processing started");
/// ```
#[derive(Debug, Clone)]
pub struct IoContext {
    /// The common state for this context.
    state: CommonState,
}

impl IoContext {
    /// Create a new IO context with default settings.
    pub fn new() -> Self {
        Self {
            state: CommonState {
                user_data_dir: common::default_user_data_dir(),
                ..Default::default()
            },
        }
    }

    /// Create a new IO context with a specific user data directory.
    pub fn with_user_data_dir(user_data_dir: PathBuf) -> Self {
        Self {
            state: CommonState {
                user_data_dir: Some(user_data_dir),
                ..Default::default()
            },
        }
    }

    /// Create a new IO context with the given common state.
    pub fn with_state(state: CommonState) -> Self {
        Self { state }
    }
}

impl Default for IoContext {
    fn default() -> Self {
        Self::new()
    }
}

impl ClmdContext for IoContext {
    type Error = ClmdError;

    fn read_file(&self, path: &Path) -> Result<Vec<u8>, Self::Error> {
        fs::read(path).map_err(|e| {
            ClmdError::io_error(format!("Failed to read {}: {}", path.display(), e))
        })
    }

    fn write_file(&self, path: &Path, content: &[u8]) -> Result<(), Self::Error> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                ClmdError::io_error(format!(
                    "Failed to create directory {}: {}",
                    parent.display(),
                    e
                ))
            })?;
        }

        fs::write(path, content).map_err(|e| {
            ClmdError::io_error(format!("Failed to write {}: {}", path.display(), e))
        })
    }

    fn file_exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn get_modification_time(&self, path: &Path) -> Result<SystemTime, Self::Error> {
        fs::metadata(path)
            .map_err(|e| {
                ClmdError::io_error(format!(
                    "Failed to get metadata for {}: {}",
                    path.display(),
                    e
                ))
            })?
            .modified()
            .map_err(|e| {
                ClmdError::io_error(format!(
                    "Failed to get modification time for {}: {}",
                    path.display(),
                    e
                ))
            })
    }

    fn find_file(&self, filename: &str) -> Option<PathBuf> {
        self.state.find_file(filename)
    }

    fn report(&self, level: LogLevel, message: String) {
        self.state.log(level, message.clone());

        // Also print to stderr/stdout based on level
        if level.should_log(self.state.verbosity.as_u8()) {
            match level {
                LogLevel::Error => eprintln!("[ERROR] {}", message),
                LogLevel::Warning => eprintln!("[WARNING] {}", message),
                LogLevel::Info => println!("[INFO] {}", message),
                LogLevel::Debug => println!("[DEBUG] {}", message),
            }
        }
    }

    fn get_logs(&self) -> Vec<LogMessage> {
        self.state.get_logs()
    }

    fn get_verbosity(&self) -> Verbosity {
        self.state.verbosity
    }

    fn set_verbosity(&mut self, verbosity: Verbosity) {
        self.state.verbosity = verbosity;
    }

    fn get_state(&self) -> &CommonState {
        &self.state
    }

    fn get_state_mut(&mut self) -> &mut CommonState {
        &mut self.state
    }

    fn insert_media(
        &self,
        path: &Path,
        mime_type: Option<&str>,
        data: Vec<u8>,
    ) -> Result<String, Self::Error> {
        let mut bag = self.state.media_bag.lock().unwrap();

        // Handle data URIs specially
        let path_str = path.to_string_lossy();
        if common::is_data_uri(&path_str) {
            // For data URIs, use a hash-based path
            let hash = format!("{:x}", md5::compute(&data));
            let ext = mime_type
                .and_then(|m| m.split('/').nth(1))
                .map(|s| s.split(';').next().unwrap_or(s))
                .map(|s| format!(".{}", s))
                .unwrap_or_default();
            let new_path = format!("{}{}", hash, ext);
            bag.insert_opt(PathBuf::from(&new_path), mime_type, data);
            return Ok(new_path);
        }

        let canonical = common::canonicalize_path(path);
        bag.insert_opt(path, mime_type, data);
        Ok(canonical)
    }

    fn lookup_media(&self, path: &Path) -> Option<MediaItem> {
        let bag = self.state.media_bag.lock().unwrap();
        bag.lookup(path).cloned()
    }

    fn get_current_time(&self) -> SystemTime {
        SystemTime::now()
    }

    fn get_random_bytes(&self, len: usize) -> Vec<u8> {
        use rand::RngCore;
        let mut bytes = vec![0u8; len];
        rand::thread_rng().fill_bytes(&mut bytes);
        bytes
    }

    fn invalid_utf8_error(path: &Path) -> Self::Error {
        ClmdError::io_error(format!("Invalid UTF-8 in file {}", path.display()))
    }

    fn read_file_to_string_dyn(&self, path: &Path) -> Result<String, Self::Error> {
        self.read_file_to_string(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_io_context_new() {
        let ctx = IoContext::new();
        assert_eq!(ctx.get_verbosity(), Verbosity::Normal);
    }

    #[test]
    fn test_io_context_verbosity() {
        let mut ctx = IoContext::new();
        ctx.set_verbosity(Verbosity::Verbose);
        assert_eq!(ctx.get_verbosity(), Verbosity::Verbose);
    }

    #[test]
    fn test_read_write_file() {
        let ctx = IoContext::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"Hello, World!").unwrap();

        let path = temp_file.path();
        assert!(ctx.file_exists(path));

        let content = ctx.read_file(path).unwrap();
        assert_eq!(content, b"Hello, World!");
    }

    #[test]
    fn test_read_nonexistent_file() {
        let ctx = IoContext::new();
        let path = PathBuf::from("/nonexistent/file.txt");
        assert!(!ctx.file_exists(&path));

        let result = ctx.read_file(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_log_messages() {
        let ctx = IoContext::new();
        ctx.info("Test message");

        let logs = ctx.get_logs();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].level, LogLevel::Info);
        assert_eq!(logs[0].message, "Test message");
    }

    #[test]
    fn test_log_verbosity_filtering() {
        let mut ctx = IoContext::new();
        ctx.set_verbosity(Verbosity::Quiet);

        ctx.debug("Debug message");
        ctx.info("Info message");
        ctx.warn("Warning message");
        ctx.error("Error message");

        let logs = ctx.get_logs();
        // Only errors should be logged in quiet mode
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].level, LogLevel::Error);
    }

    #[test]
    fn test_media_bag_operations() {
        let ctx = IoContext::new();
        let path = PathBuf::from("test.png");
        let data = vec![0x89, 0x50, 0x4E, 0x47]; // PNG magic bytes

        let canonical = ctx
            .insert_media(&path, Some("image/png"), data.clone())
            .unwrap();
        assert!(!canonical.is_empty());

        let item = ctx.lookup_media(&path);
        assert!(item.is_some());
        let item = item.unwrap();
        assert_eq!(item.contents(), data.as_slice());
    }

    #[test]
    fn test_clone_context() {
        let ctx = IoContext::new();
        let ctx2 = ctx.clone();
        
        // Both should have independent state
        assert_eq!(ctx.get_verbosity(), ctx2.get_verbosity());
    }

    #[test]
    fn test_get_modification_time() {
        let ctx = IoContext::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"test").unwrap();

        let mtime = ctx.get_modification_time(temp_file.path());
        assert!(mtime.is_ok());
    }

    #[test]
    fn test_find_file() {
        let mut ctx = IoContext::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"test").unwrap();
        
        let path = temp_file.path();
        let parent = path.parent().unwrap();
        ctx.add_resource_path(parent.to_path_buf());
        
        let filename = path.file_name().unwrap().to_str().unwrap();
        let found = ctx.find_file(filename);
        assert!(found.is_some());
    }
}
