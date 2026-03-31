//! Pure Context implementation for testing.
//!
//! This module provides the [`PureContext`] struct, which implements
//! the [`ClmdContext`] trait using in-memory storage. This is useful for
//! testing and pure functional code.
//!
//! # Example
//!
//! ```ignore
//! use clmd::context::{ClmdContext, PureContext};
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
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use crate::context::{
    common, ClmdContext, CommonState, LogLevel, LogMessage, Verbosity,
};
use crate::error::ClmdError;
use crate::mediabag::MediaItem;

/// Pure Context for testing and pure functional code.
///
/// This context stores files in memory and performs no actual IO operations.
/// It is useful for testing and for pure functional code that needs to
/// abstract over IO operations.
///
/// # Example
///
/// ```ignore
/// use clmd::context::{ClmdContext, PureContext};
///
/// let mut ctx = PureContext::new();
/// ctx.add_file("input.md", b"# Test");
///
/// let content = ctx.read_file(std::path::Path::new("input.md")).unwrap();
/// assert_eq!(content, b"# Test");
/// ```ignore
#[derive(Debug, Clone)]
pub struct PureContext {
    /// The common state for this context.
    state: CommonState,
    /// In-memory file storage.
    files: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    /// Simulated current time for testing.
    current_time: Arc<Mutex<SystemTime>>,
    /// Simulated random bytes for testing.
    random_bytes: Arc<Mutex<Vec<u8>>>,
}

impl PureContext {
    /// Create a new pure context with default settings.
    pub fn new() -> Self {
        Self {
            state: CommonState::default(),
            files: Arc::new(Mutex::new(HashMap::new())),
            current_time: Arc::new(Mutex::new(SystemTime::UNIX_EPOCH)),
            random_bytes: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Create a new pure context with a specific user data directory.
    pub fn with_user_data_dir(user_data_dir: PathBuf) -> Self {
        Self {
            state: CommonState {
                user_data_dir: Some(user_data_dir),
                ..Default::default()
            },
            files: Arc::new(Mutex::new(HashMap::new())),
            current_time: Arc::new(Mutex::new(SystemTime::UNIX_EPOCH)),
            random_bytes: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Create a new pure context with the given common state.
    pub fn with_state(state: CommonState) -> Self {
        Self {
            state,
            files: Arc::new(Mutex::new(HashMap::new())),
            current_time: Arc::new(Mutex::new(SystemTime::UNIX_EPOCH)),
            random_bytes: Arc::new(Mutex::new(Vec::new())),
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
        self.state.get_logs()
    }

    /// Clear all log messages.
    pub fn clear_logs(&mut self) {
        self.state.clear_logs();
    }

    /// Get the number of log messages.
    pub fn log_count(&self) -> usize {
        self.state.get_logs().len()
    }

    /// Check if any errors were logged.
    pub fn has_errors(&self) -> bool {
        self.state.has_errors()
    }

    /// Check if any warnings were logged.
    pub fn has_warnings(&self) -> bool {
        self.state.has_warnings()
    }

    /// Get all error messages.
    pub fn get_errors(&self) -> Vec<LogMessage> {
        let logs = self.state.get_logs();
        logs.iter()
            .filter(|log| log.level == LogLevel::Error)
            .cloned()
            .collect()
    }

    /// Get all warning messages.
    pub fn get_warnings(&self) -> Vec<LogMessage> {
        let logs = self.state.get_logs();
        logs.iter()
            .filter(|log| log.level == LogLevel::Warning)
            .cloned()
            .collect()
    }

    /// Set the simulated current time for testing.
    pub fn set_current_time(&self, time: SystemTime) {
        let mut current_time = self.current_time.lock().unwrap();
        *current_time = time;
    }

    /// Set the simulated random bytes for testing.
    pub fn set_random_bytes(&self, bytes: Vec<u8>) {
        let mut random_bytes = self.random_bytes.lock().unwrap();
        *random_bytes = bytes;
    }
}

impl Default for PureContext {
    fn default() -> Self {
        Self::new()
    }
}

impl ClmdContext for PureContext {
    type Error = ClmdError;

    fn read_file(&self, path: &Path) -> Result<Vec<u8>, Self::Error> {
        let canonical = common::canonicalize_path(path);
        let files = self.files.lock().unwrap();
        files.get(&canonical).cloned().ok_or_else(|| {
            ClmdError::io_error(format!("File not found: {}", path.display()))
        })
    }

    fn write_file(&self, path: &Path, content: &[u8]) -> Result<(), Self::Error> {
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

    fn get_modification_time(&self, path: &Path) -> Result<SystemTime, Self::Error> {
        // In pure context, return the simulated current time for any existing file
        if self.file_exists(path) {
            let time = *self.current_time.lock().unwrap();
            Ok(time)
        } else {
            Err(ClmdError::io_error(format!(
                "File not found: {}",
                path.display()
            )))
        }
    }

    fn find_file(&self, filename: &str) -> Option<PathBuf> {
        self.state.find_file(filename)
    }

    fn report(&self, level: LogLevel, message: String) {
        self.state.log(level, message);
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
        *self.current_time.lock().unwrap()
    }

    fn get_random_bytes(&self, len: usize) -> Vec<u8> {
        let random_bytes = self.random_bytes.lock().unwrap();
        if random_bytes.is_empty() {
            // Return deterministic bytes if not set
            (0..len).map(|i| (i % 256) as u8).collect()
        } else {
            // Cycle through the provided bytes
            (0..len)
                .map(|i| random_bytes[i % random_bytes.len()])
                .collect()
        }
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

    #[test]
    fn test_pure_context_new() {
        let ctx = PureContext::new();
        assert_eq!(ctx.get_verbosity(), Verbosity::Normal);
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
        let mut ctx = PureContext::new();
        assert_eq!(ctx.get_verbosity(), Verbosity::Normal);

        ctx.set_verbosity(Verbosity::Verbose);
        assert_eq!(ctx.get_verbosity(), Verbosity::Verbose);

        ctx.set_verbosity(Verbosity::Quiet);
        assert_eq!(ctx.get_verbosity(), Verbosity::Quiet);
    }

    #[test]
    fn test_log_messages() {
        let ctx = PureContext::new();
        ctx.info("Test message");

        let logs = ctx.get_logs();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].level, LogLevel::Info);
        assert_eq!(logs[0].message, "Test message");
    }

    #[test]
    fn test_log_verbosity_filtering() {
        let mut ctx = PureContext::new();
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
    fn test_log_count() {
        let ctx = PureContext::new();
        assert_eq!(ctx.log_count(), 0);

        ctx.info("Message 1");
        ctx.info("Message 2");

        assert_eq!(ctx.log_count(), 2);
    }

    #[test]
    fn test_clear_logs() {
        let mut ctx = PureContext::new();
        ctx.info("Message");
        assert_eq!(ctx.log_count(), 1);

        ctx.clear_logs();
        assert_eq!(ctx.log_count(), 0);
    }

    #[test]
    fn test_has_errors() {
        let ctx = PureContext::new();
        assert!(!ctx.has_errors());

        ctx.warn("Warning");
        assert!(!ctx.has_errors());

        ctx.error("Error");
        assert!(ctx.has_errors());
    }

    #[test]
    fn test_has_warnings() {
        let ctx = PureContext::new();
        assert!(!ctx.has_warnings());

        ctx.info("Info");
        assert!(!ctx.has_warnings());

        ctx.warn("Warning");
        assert!(ctx.has_warnings());
    }

    #[test]
    fn test_get_errors() {
        let ctx = PureContext::new();
        ctx.error("Error 1");
        ctx.info("Info");
        ctx.error("Error 2");

        let errors = ctx.get_errors();
        assert_eq!(errors.len(), 2);
        assert!(errors.iter().all(|e| e.level == LogLevel::Error));
    }

    #[test]
    fn test_get_warnings() {
        let ctx = PureContext::new();
        ctx.warn("Warning 1");
        ctx.info("Info");
        ctx.warn("Warning 2");

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
            Some(&PathBuf::from("/data"))
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

    #[test]
    fn test_simulated_time() {
        let mut ctx = PureContext::new();

        // Default is UNIX_EPOCH
        assert_eq!(ctx.get_current_time(), SystemTime::UNIX_EPOCH);

        // Set a specific time
        let test_time = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1000);
        ctx.set_current_time(test_time);
        assert_eq!(ctx.get_current_time(), test_time);

        // Modification time should return the simulated time
        ctx.add_file("test.md", b"content");
        let mtime = ctx.get_modification_time(Path::new("test.md")).unwrap();
        assert_eq!(mtime, test_time);
    }

    #[test]
    fn test_simulated_random_bytes() {
        let ctx = PureContext::new();

        // Default returns deterministic bytes
        let bytes = ctx.get_random_bytes(5);
        assert_eq!(bytes, vec![0, 1, 2, 3, 4]);

        // Set specific random bytes
        ctx.set_random_bytes(vec![0xAB, 0xCD]);
        let bytes = ctx.get_random_bytes(6);
        assert_eq!(bytes, vec![0xAB, 0xCD, 0xAB, 0xCD, 0xAB, 0xCD]);
    }

    #[test]
    fn test_get_modification_time_nonexistent() {
        let ctx = PureContext::new();
        let result = ctx.get_modification_time(Path::new("nonexistent.md"));
        assert!(result.is_err());
    }
}
