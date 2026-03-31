//! IO Context implementation for real file operations.
//!
//! This module provides the [`IoContext`] struct, which implements
//! the [`Context`] trait for real file system operations.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};

use crate::context::{common, LogLevel, LogMessage};
use crate::error::{ClmdError, ClmdResult};
use crate::mediabag::{MediaBag, MediaItem};

use super::Context;

/// IO Context for real file operations.
///
/// This context performs actual file system operations and is suitable
/// for production use.
///
/// # Example
///
/// ```
/// use clmd::context::{Context, IoContext};
///
/// let ctx = IoContext::new();
/// // Use ctx for file operations
/// ```
#[derive(Debug)]
pub struct IoContext {
    /// The media bag for storing binary resources.
    media_bag: Arc<Mutex<MediaBag>>,
    /// Log messages.
    logs: Arc<Mutex<Vec<LogMessage>>>,
    /// User data directory.
    user_data_dir: Option<PathBuf>,
    /// Verbosity level (0 = quiet, 1 = normal, 2 = verbose).
    verbosity: RwLock<u8>,
}

impl IoContext {
    /// Create a new IO context with default settings.
    pub fn new() -> Self {
        Self {
            media_bag: Arc::new(Mutex::new(MediaBag::new())),
            logs: Arc::new(Mutex::new(Vec::new())),
            user_data_dir: Self::default_user_data_dir(),
            verbosity: RwLock::new(1),
        }
    }

    /// Create a new IO context with a specific user data directory.
    pub fn with_user_data_dir(user_data_dir: PathBuf) -> Self {
        Self {
            media_bag: Arc::new(Mutex::new(MediaBag::new())),
            logs: Arc::new(Mutex::new(Vec::new())),
            user_data_dir: Some(user_data_dir),
            verbosity: RwLock::new(1),
        }
    }

    /// Get the default user data directory.
    fn default_user_data_dir() -> Option<PathBuf> {
        // Try XDG_DATA_HOME first
        if let Ok(xdg_data_home) = std::env::var("XDG_DATA_HOME") {
            let dir = PathBuf::from(xdg_data_home).join("clmd");
            if dir.exists() {
                return Some(dir);
            }
        }

        // Try platform-specific directories
        #[cfg(target_os = "macos")]
        {
            if let Ok(home) = std::env::var("HOME") {
                let dir = PathBuf::from(home).join("Library/Application Support/clmd");
                if dir.exists() {
                    return Some(dir);
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            if let Ok(app_data) = std::env::var("APPDATA") {
                let dir = PathBuf::from(app_data).join("clmd");
                if dir.exists() {
                    return Some(dir);
                }
            }
        }

        // Try ~/.local/share/clmd (XDG default)
        if let Ok(home) = std::env::var("HOME") {
            let dir = PathBuf::from(&home).join(".local/share/clmd");
            if dir.exists() {
                return Some(dir);
            }
            // Legacy: ~/.clmd
            let legacy_dir = PathBuf::from(&home).join(".clmd");
            if legacy_dir.exists() {
                return Some(legacy_dir);
            }
        }

        None
    }
}

impl Default for IoContext {
    fn default() -> Self {
        Self::new()
    }
}

impl Context for IoContext {
    fn read_file(&self, path: &Path) -> ClmdResult<Vec<u8>> {
        fs::read(path).map_err(|e| {
            ClmdError::io_error(format!("Failed to read {}: {}", path.display(), e))
        })
    }

    fn write_file(&self, path: &Path, content: &[u8]) -> ClmdResult<()> {
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

    fn log(&self, level: LogLevel, message: &str) {
        // Only log if verbosity level permits
        let verbosity = *self.verbosity.read().unwrap();
        let should_log = match level {
            LogLevel::Error => true,
            LogLevel::Warning => verbosity >= 1,
            LogLevel::Info => verbosity >= 1,
            LogLevel::Debug => verbosity >= 2,
        };

        if should_log {
            let mut logs = self.logs.lock().unwrap();
            logs.push(LogMessage::new(level, message.to_string()));

            // Also print to stderr for errors and warnings
            match level {
                LogLevel::Error => eprintln!("[ERROR] {}", message),
                LogLevel::Warning => eprintln!("[WARNING] {}", message),
                LogLevel::Info => println!("[INFO] {}", message),
                LogLevel::Debug => println!("[DEBUG] {}", message),
            }
        }
    }

    fn get_logs(&self) -> Vec<LogMessage> {
        self.logs.lock().unwrap().clone()
    }

    fn get_media_bag(&self) -> Arc<Mutex<MediaBag>> {
        Arc::clone(&self.media_bag)
    }

    fn insert_media(
        &self,
        path: &Path,
        mime_type: Option<&str>,
        data: Vec<u8>,
    ) -> ClmdResult<String> {
        let mut bag = self.media_bag.lock().unwrap();

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
        let bag = self.media_bag.lock().unwrap();
        bag.lookup(path).cloned()
    }

    fn get_user_data_dir(&self) -> Option<PathBuf> {
        self.user_data_dir.clone()
    }

    fn get_verbosity(&self) -> u8 {
        *self.verbosity.read().unwrap()
    }

    fn set_verbosity(&self, level: u8) {
        *self.verbosity.write().unwrap() = level;
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
        assert_eq!(ctx.get_verbosity(), 1);
    }

    #[test]
    fn test_io_context_verbosity() {
        let ctx = IoContext::new();
        ctx.set_verbosity(2);
        assert_eq!(ctx.get_verbosity(), 2);
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
        ctx.log(LogLevel::Info, "Test message");

        let logs = ctx.get_logs();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].level, LogLevel::Info);
        assert_eq!(logs[0].message, "Test message");
    }

    #[test]
    fn test_log_verbosity_filtering() {
        let ctx = IoContext::new();
        ctx.set_verbosity(0); // Quiet mode

        ctx.log(LogLevel::Debug, "Debug message");
        ctx.log(LogLevel::Info, "Info message");
        ctx.log(LogLevel::Warning, "Warning message");
        ctx.log(LogLevel::Error, "Error message");

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
}
