//! Pure Context implementation for testing.
//!
//! This module provides the [`PureContext`] struct, which implements
//! the [`Context`] trait using in-memory storage. This is useful for
//! testing and pure functional code.
//!
//! # Example
//!
//! ```
//! use clmd::context::{Context, PureContext};
//! use std::path::Path;
//!
//! let mut ctx = PureContext::new();
//! ctx.add_file("test.md", b"# Hello World");
//!
//! let content = ctx.read_file(Path::new("test.md")).unwrap();
//! assert_eq!(content, b"# Hello World");
//! ```

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};

use crate::context::{common, LogLevel, LogMessage};
use crate::error::{ClmdError, ClmdResult};
use crate::mediabag::{MediaBag, MediaItem};

use super::Context;

/// Pure Context for testing and pure functional code.
///
/// This context stores files in memory and performs no actual IO operations.
/// It is useful for testing and for pure functional code that needs to
/// abstract over IO operations.
///
/// # Example
///
/// ```
/// use clmd::context::{Context, PureContext};
///
/// let mut ctx = PureContext::new();
/// ctx.add_file("input.md", b"# Test");
///
/// let content = ctx.read_file(std::path::Path::new("input.md")).unwrap();
/// assert_eq!(content, b"# Test");
/// ```
#[derive(Debug, Clone)]
pub struct PureContext {
    /// In-memory file storage.
    files: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    /// The media bag for storing binary resources.
    media_bag: Arc<Mutex<MediaBag>>,
    /// Log messages.
    logs: Arc<Mutex<Vec<LogMessage>>>,
    /// User data directory (simulated).
    user_data_dir: Option<PathBuf>,
    /// Verbosity level (0 = quiet, 1 = normal, 2 = verbose).
    verbosity: Arc<RwLock<u8>>,
}

impl PureContext {
    /// Create a new pure context with default settings.
    pub fn new() -> Self {
        Self {
            files: Arc::new(Mutex::new(HashMap::new())),
            media_bag: Arc::new(Mutex::new(MediaBag::new())),
            logs: Arc::new(Mutex::new(Vec::new())),
            user_data_dir: None,
            verbosity: Arc::new(RwLock::new(1)),
        }
    }

    /// Create a new pure context with a specific user data directory.
    pub fn with_user_data_dir(user_data_dir: PathBuf) -> Self {
        Self {
            files: Arc::new(Mutex::new(HashMap::new())),
            media_bag: Arc::new(Mutex::new(MediaBag::new())),
            logs: Arc::new(Mutex::new(Vec::new())),
            user_data_dir: Some(user_data_dir),
            verbosity: Arc::new(RwLock::new(1)),
        }
    }

    /// Add a file to the in-memory storage.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to store the file under.
    /// * `content` - The file contents.
    ///
    /// # Example
    ///
    /// ```
    /// use clmd::context::PureContext;
    ///
    /// let mut ctx = PureContext::new();
    /// ctx.add_file("test.md", b"# Hello");
    /// ```
    pub fn add_file<P: AsRef<Path>>(&mut self, path: P, content: impl Into<Vec<u8>>) {
        let canonical = common::canonicalize_path(path.as_ref());
        let mut files = self.files.lock().unwrap();
        files.insert(canonical, content.into());
    }

    /// Add a text file to the in-memory storage.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to store the file under.
    /// * `content` - The file contents as a string.
    ///
    /// # Example
    ///
    /// ```
    /// use clmd::context::PureContext;
    ///
    /// let mut ctx = PureContext::new();
    /// ctx.add_text_file("test.md", "# Hello");
    /// ```
    pub fn add_text_file<P: AsRef<Path>>(
        &mut self,
        path: P,
        content: impl Into<String>,
    ) {
        self.add_file(path, content.into().into_bytes());
    }

    /// Remove a file from the in-memory storage.
    ///
    /// # Arguments
    ///
    /// * `path` - The path of the file to remove.
    ///
    /// # Returns
    ///
    /// The file contents if it existed, None otherwise.
    pub fn remove_file<P: AsRef<Path>>(&mut self, path: P) -> Option<Vec<u8>> {
        let canonical = common::canonicalize_path(path.as_ref());
        let mut files = self.files.lock().unwrap();
        files.remove(&canonical)
    }

    /// Get a list of all files in the storage.
    pub fn list_files(&self) -> Vec<String> {
        let files = self.files.lock().unwrap();
        files.keys().cloned().collect()
    }

    /// Clear all files from the storage.
    pub fn clear_files(&mut self) {
        let mut files = self.files.lock().unwrap();
        files.clear();
    }

    /// Check if a file exists in the storage.
    pub fn has_file<P: AsRef<Path>>(&self, path: P) -> bool {
        let canonical = common::canonicalize_path(path.as_ref());
        let files = self.files.lock().unwrap();
        files.contains_key(&canonical)
    }

    /// Get the number of files in the storage.
    pub fn file_count(&self) -> usize {
        let files = self.files.lock().unwrap();
        files.len()
    }

    /// Get all log messages without clearing them.
    pub fn peek_logs(&self) -> Vec<LogMessage> {
        self.logs.lock().unwrap().clone()
    }

    /// Clear all log messages.
    pub fn clear_logs(&mut self) {
        let mut logs = self.logs.lock().unwrap();
        logs.clear();
    }

    /// Get the number of log messages.
    pub fn log_count(&self) -> usize {
        let logs = self.logs.lock().unwrap();
        logs.len()
    }

    /// Check if any errors were logged.
    pub fn has_errors(&self) -> bool {
        let logs = self.logs.lock().unwrap();
        logs.iter().any(|log| log.level == LogLevel::Error)
    }

    /// Check if any warnings were logged.
    pub fn has_warnings(&self) -> bool {
        let logs = self.logs.lock().unwrap();
        logs.iter().any(|log| log.level == LogLevel::Warning)
    }

    /// Get all error messages.
    pub fn get_errors(&self) -> Vec<LogMessage> {
        let logs = self.logs.lock().unwrap();
        logs.iter()
            .filter(|log| log.level == LogLevel::Error)
            .cloned()
            .collect()
    }

    /// Get all warning messages.
    pub fn get_warnings(&self) -> Vec<LogMessage> {
        let logs = self.logs.lock().unwrap();
        logs.iter()
            .filter(|log| log.level == LogLevel::Warning)
            .cloned()
            .collect()
    }
}

impl Default for PureContext {
    fn default() -> Self {
        Self::new()
    }
}

impl Context for PureContext {
    fn read_file(&self, path: &Path) -> ClmdResult<Vec<u8>> {
        let canonical = common::canonicalize_path(path);
        let files = self.files.lock().unwrap();
        files.get(&canonical).cloned().ok_or_else(|| {
            ClmdError::io_error(format!("File not found: {}", path.display()))
        })
    }

    fn write_file(&self, path: &Path, content: &[u8]) -> ClmdResult<()> {
        let canonical = common::canonicalize_path(path);
        let mut files = self.files.lock().unwrap();
        files.insert(canonical, content.to_vec());
        Ok(())
    }

    fn file_exists(&self, path: &Path) -> bool {
        let canonical = common::canonicalize_path(path);
        let files = self.files.lock().unwrap();
        files.contains_key(&canonical)
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
            bag.insert_opt(&PathBuf::from(&new_path), mime_type, data);
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

    #[test]
    fn test_pure_context_new() {
        let ctx = PureContext::new();
        assert_eq!(ctx.get_verbosity(), 1);
        assert_eq!(ctx.file_count(), 0);
    }

    #[test]
    fn test_pure_context_default() {
        let ctx: PureContext = Default::default();
        assert_eq!(ctx.file_count(), 0);
    }

    #[test]
    fn test_add_file() {
        let mut ctx = PureContext::new();
        ctx.add_file("test.md", b"# Hello");

        assert_eq!(ctx.file_count(), 1);
        assert!(ctx.has_file("test.md"));
        assert!(!ctx.has_file("other.md"));
    }

    #[test]
    fn test_add_text_file() {
        let mut ctx = PureContext::new();
        ctx.add_text_file("test.md", "# Hello World");

        let content = ctx.read_file(Path::new("test.md")).unwrap();
        assert_eq!(content, b"# Hello World");
    }

    #[test]
    fn test_read_file() {
        let mut ctx = PureContext::new();
        ctx.add_file("test.md", b"# Hello World");

        let content = ctx.read_file(Path::new("test.md")).unwrap();
        assert_eq!(content, b"# Hello World");
    }

    #[test]
    fn test_read_nonexistent_file() {
        let ctx = PureContext::new();

        let result = ctx.read_file(Path::new("nonexistent.md"));
        assert!(result.is_err());
    }

    #[test]
    fn test_write_file() {
        let ctx = PureContext::new();

        ctx.write_file(Path::new("output.md"), b"# Output").unwrap();

        assert!(ctx.has_file("output.md"));
        let content = ctx.read_file(Path::new("output.md")).unwrap();
        assert_eq!(content, b"# Output");
    }

    #[test]
    fn test_file_exists() {
        let mut ctx = PureContext::new();
        ctx.add_file("exists.md", b"");

        assert!(ctx.file_exists(Path::new("exists.md")));
        assert!(!ctx.file_exists(Path::new("notexists.md")));
    }

    #[test]
    fn test_remove_file() {
        let mut ctx = PureContext::new();
        ctx.add_file("test.md", b"content");

        let removed = ctx.remove_file("test.md");
        assert_eq!(removed, Some(b"content".to_vec()));
        assert!(!ctx.has_file("test.md"));

        let not_removed = ctx.remove_file("nonexistent.md");
        assert_eq!(not_removed, None);
    }

    #[test]
    fn test_list_files() {
        let mut ctx = PureContext::new();
        ctx.add_file("a.md", b"");
        ctx.add_file("b.md", b"");
        ctx.add_file("c.md", b"");

        let files = ctx.list_files();
        assert_eq!(files.len(), 3);
        assert!(files.contains(&"a.md".to_string()));
        assert!(files.contains(&"b.md".to_string()));
        assert!(files.contains(&"c.md".to_string()));
    }

    #[test]
    fn test_clear_files() {
        let mut ctx = PureContext::new();
        ctx.add_file("a.md", b"");
        ctx.add_file("b.md", b"");

        ctx.clear_files();
        assert_eq!(ctx.file_count(), 0);
    }

    #[test]
    fn test_verbosity() {
        let ctx = PureContext::new();
        assert_eq!(ctx.get_verbosity(), 1);

        ctx.set_verbosity(2);
        assert_eq!(ctx.get_verbosity(), 2);

        ctx.set_verbosity(0);
        assert_eq!(ctx.get_verbosity(), 0);
    }

    #[test]
    fn test_log_messages() {
        let ctx = PureContext::new();
        ctx.log(LogLevel::Info, "Test message");

        let logs = ctx.get_logs();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].level, LogLevel::Info);
        assert_eq!(logs[0].message, "Test message");
    }

    #[test]
    fn test_log_verbosity_filtering() {
        let ctx = PureContext::new();
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
    fn test_log_count() {
        let ctx = PureContext::new();
        assert_eq!(ctx.log_count(), 0);

        ctx.log(LogLevel::Info, "Message 1");
        ctx.log(LogLevel::Info, "Message 2");

        assert_eq!(ctx.log_count(), 2);
    }

    #[test]
    fn test_clear_logs() {
        let mut ctx = PureContext::new();
        ctx.log(LogLevel::Info, "Message");
        assert_eq!(ctx.log_count(), 1);

        ctx.clear_logs();
        assert_eq!(ctx.log_count(), 0);
    }

    #[test]
    fn test_has_errors() {
        let ctx = PureContext::new();
        assert!(!ctx.has_errors());

        ctx.log(LogLevel::Warning, "Warning");
        assert!(!ctx.has_errors());

        ctx.log(LogLevel::Error, "Error");
        assert!(ctx.has_errors());
    }

    #[test]
    fn test_has_warnings() {
        let ctx = PureContext::new();
        assert!(!ctx.has_warnings());

        ctx.log(LogLevel::Info, "Info");
        assert!(!ctx.has_warnings());

        ctx.log(LogLevel::Warning, "Warning");
        assert!(ctx.has_warnings());
    }

    #[test]
    fn test_get_errors() {
        let ctx = PureContext::new();
        ctx.log(LogLevel::Error, "Error 1");
        ctx.log(LogLevel::Info, "Info");
        ctx.log(LogLevel::Error, "Error 2");

        let errors = ctx.get_errors();
        assert_eq!(errors.len(), 2);
        assert!(errors.iter().all(|e| e.level == LogLevel::Error));
    }

    #[test]
    fn test_get_warnings() {
        let ctx = PureContext::new();
        ctx.log(LogLevel::Warning, "Warning 1");
        ctx.log(LogLevel::Info, "Info");
        ctx.log(LogLevel::Warning, "Warning 2");

        let warnings = ctx.get_warnings();
        assert_eq!(warnings.len(), 2);
        assert!(warnings.iter().all(|w| w.level == LogLevel::Warning));
    }

    #[test]
    fn test_media_bag_operations() {
        let ctx = PureContext::new();
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
    fn test_user_data_dir() {
        let ctx = PureContext::new();
        assert!(ctx.get_user_data_dir().is_none());

        let ctx_with_dir = PureContext::with_user_data_dir(PathBuf::from("/data"));
        assert_eq!(
            ctx_with_dir.get_user_data_dir(),
            Some(PathBuf::from("/data"))
        );
    }

    #[test]
    fn test_path_normalization() {
        let mut ctx = PureContext::new();
        ctx.add_file("path/to/file.md", b"content");

        // Should be able to access with different path separators
        assert!(ctx.has_file("path/to/file.md"));
        assert!(ctx.has_file("path\\to\\file.md"));

        // Read should work with either separator
        let content1 = ctx.read_file(Path::new("path/to/file.md")).unwrap();
        let content2 = ctx.read_file(Path::new("path\\to\\file.md")).unwrap();
        assert_eq!(content1, content2);
    }
}
