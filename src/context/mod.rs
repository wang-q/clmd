//! Context system for clmd, inspired by Pandoc's PandocMonad.
//!
//! This module provides a unified abstraction for IO operations, logging,
//! and resource management. It allows for both real IO operations and
//! pure/mock implementations for testing.
//!
//! The design follows Pandoc's PandocMonad typeclass pattern, providing:
//! - A trait-based abstraction for IO operations
//! - Support for both pure and IO-based implementations
//! - Common state management (verbosity, resource paths, etc.)
//! - Logging and reporting capabilities
//!
//! # Example
//!
//! ```
//! use clmd::context::{ClmdContext, IoContext, LogLevel};
//! use std::path::Path;
//!
//! fn process_document<C: ClmdContext>(ctx: &C, input: &Path) -> Result<String, C::Error> {
//!     ctx.report(LogLevel::Info, "Processing document".to_string());
//!     let content = ctx.read_file_to_string(input)?;
//!     Ok(content)
//! }
//! ```

use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use crate::error::ClmdResult;
use crate::mediabag::MediaBag;

mod io;
mod pure;

pub use io::IoContext;
pub use pure::PureContext;

/// Log level for context logging.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LogLevel {
    /// Debug level - verbose information for debugging.
    Debug,
    /// Info level - general information.
    Info,
    /// Warning level - potential issues.
    Warning,
    /// Error level - errors that occurred.
    Error,
}

impl LogLevel {
    /// Get the string representation of the log level.
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warning => "WARNING",
            LogLevel::Error => "ERROR",
        }
    }

    /// Check if this level should be logged given a verbosity setting.
    ///
    /// # Arguments
    ///
    /// * `verbosity` - The verbosity level (0=quiet, 1=normal, 2=verbose)
    pub fn should_log(&self, verbosity: u8) -> bool {
        match self {
            LogLevel::Error => true,
            LogLevel::Warning => verbosity >= 1,
            LogLevel::Info => verbosity >= 1,
            LogLevel::Debug => verbosity >= 2,
        }
    }
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A log message.
#[derive(Debug, Clone)]
pub struct LogMessage {
    /// The log level.
    pub level: LogLevel,
    /// The log message.
    pub message: String,
    /// Optional source information.
    pub source: Option<String>,
    /// Timestamp when the log was created.
    pub timestamp: SystemTime,
}

impl LogMessage {
    /// Create a new log message.
    pub fn new(level: LogLevel, message: impl Into<String>) -> Self {
        Self {
            level,
            message: message.into(),
            source: None,
            timestamp: SystemTime::now(),
        }
    }

    /// Create a new log message with source.
    pub fn with_source(
        level: LogLevel,
        message: impl Into<String>,
        source: impl Into<String>,
    ) -> Self {
        Self {
            level,
            message: message.into(),
            source: Some(source.into()),
            timestamp: SystemTime::now(),
        }
    }

    /// Format the log message as a string.
    pub fn format(&self) -> String {
        match &self.source {
            Some(source) => format!("[{}] [{}] {}", self.level, source, self.message),
            None => format!("[{}] {}", self.level, self.message),
        }
    }
}

impl fmt::Display for LogMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format())
    }
}

/// Verbosity level for logging.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Verbosity {
    /// Quiet mode - only errors.
    Quiet = 0,
    /// Normal mode - errors, warnings, and info.
    #[default]
    Normal = 1,
    /// Verbose mode - all messages including debug.
    Verbose = 2,
}

impl Verbosity {
    /// Convert to numeric value.
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }

    /// Create from numeric value.
    pub fn from_u8(level: u8) -> Self {
        match level {
            0 => Verbosity::Quiet,
            1 => Verbosity::Normal,
            _ => Verbosity::Verbose,
        }
    }
}

/// Common state shared across context operations.
///
/// This is similar to Pandoc's CommonState, holding configuration
/// that persists across operations.
#[derive(Debug, Clone)]
pub struct CommonState {
    /// The verbosity level.
    pub verbosity: Verbosity,
    /// Resource paths for looking up files.
    pub resource_path: Vec<PathBuf>,
    /// User data directory.
    pub user_data_dir: Option<PathBuf>,
    /// Media bag for storing binary resources.
    pub media_bag: Arc<Mutex<MediaBag>>,
    /// Log messages.
    pub logs: Arc<Mutex<Vec<LogMessage>>>,
    /// Requested output format (for conditional content).
    pub output_format: Option<String>,
    /// Input filename (for error messages).
    pub input_filename: Option<String>,
    /// Source URL for resolving relative URLs.
    pub source_url: Option<String>,
}

impl Default for CommonState {
    fn default() -> Self {
        Self {
            verbosity: Verbosity::Normal,
            resource_path: Vec::new(),
            user_data_dir: None,
            media_bag: Arc::new(Mutex::new(MediaBag::new())),
            logs: Arc::new(Mutex::new(Vec::new())),
            output_format: None,
            input_filename: None,
            source_url: None,
        }
    }
}

impl CommonState {
    /// Create a new common state with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the verbosity level.
    pub fn set_verbosity(&mut self, verbosity: Verbosity) {
        self.verbosity = verbosity;
    }

    /// Add a resource path.
    pub fn add_resource_path(&mut self, path: PathBuf) {
        self.resource_path.push(path);
    }

    /// Find a file in the resource path.
    pub fn find_file(&self, filename: &str) -> Option<PathBuf> {
        // First try as absolute or relative path
        let path = PathBuf::from(filename);
        if path.exists() {
            return Some(path);
        }

        // Then try in resource paths
        for dir in &self.resource_path {
            let full_path = dir.join(filename);
            if full_path.exists() {
                return Some(full_path);
            }
        }

        // Try user data directory
        if let Some(user_data) = &self.user_data_dir {
            let full_path = user_data.join(filename);
            if full_path.exists() {
                return Some(full_path);
            }
        }

        None
    }

    /// Log a message.
    pub fn log(&self, level: LogLevel, message: impl Into<String>) {
        if level.should_log(self.verbosity.as_u8()) {
            let mut logs = self.logs.lock().unwrap();
            logs.push(LogMessage::new(level, message));
        }
    }

    /// Get all logs.
    pub fn get_logs(&self) -> Vec<LogMessage> {
        self.logs.lock().unwrap().clone()
    }

    /// Clear all logs.
    pub fn clear_logs(&self) {
        let mut logs = self.logs.lock().unwrap();
        logs.clear();
    }

    /// Check if there are any error logs.
    pub fn has_errors(&self) -> bool {
        let logs = self.logs.lock().unwrap();
        logs.iter().any(|log| log.level == LogLevel::Error)
    }

    /// Check if there are any warning logs.
    pub fn has_warnings(&self) -> bool {
        let logs = self.logs.lock().unwrap();
        logs.iter().any(|log| log.level == LogLevel::Warning)
    }
}

/// The main context trait for clmd operations.
///
/// This trait abstracts over IO operations, logging, and resource management,
/// allowing for both real IO and pure/mock implementations. It is inspired by
/// Pandoc's PandocMonad typeclass.
///
/// Implementations must be thread-safe (Send + Sync).
///
/// # Type Parameters
///
/// * `Error` - The error type used by this context.
pub trait ClmdContext: Send + Sync {
    /// The error type for this context.
    type Error: std::error::Error + Send + Sync + 'static;

    // =========================================================================
    // File Operations
    // =========================================================================

    /// Read a file into memory as bytes.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the file.
    ///
    /// # Returns
    ///
    /// The file contents as bytes, or an error.
    fn read_file(&self, path: &Path) -> Result<Vec<u8>, Self::Error>;

    /// Read a file into memory as a string.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the file.
    ///
    /// # Returns
    ///
    /// The file contents as a string, or an error.
    fn read_file_to_string(&self, path: &Path) -> Result<String, Self::Error>
    where
        Self: Sized,
    {
        let bytes = self.read_file(path)?;
        String::from_utf8(bytes).map_err(|_| Self::invalid_utf8_error(path))
    }

    /// Read a file into memory as a string (dyn-compatible version).
    ///
    /// This version works with trait objects (`dyn ClmdContext`).
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the file.
    ///
    /// # Returns
    ///
    /// The file contents as a string, or an error.
    fn read_file_to_string_dyn(&self, path: &Path) -> Result<String, Self::Error>;

    /// Create an error for invalid UTF-8.
    ///
    /// This is a helper method for the default implementation of
    /// `read_file_to_string`. Implementations should override this
    /// if they use a custom error type.
    ///
    /// # Arguments
    ///
    /// * `path` - The path that contained invalid UTF-8.
    ///
    /// # Returns
    ///
    /// An error indicating invalid UTF-8 in the file.
    fn invalid_utf8_error(path: &Path) -> Self::Error
    where
        Self: Sized;

    /// Write bytes to a file.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to write to.
    /// * `content` - The content to write.
    ///
    /// # Returns
    ///
    /// Ok on success, or an error.
    fn write_file(&self, path: &Path, content: &[u8]) -> Result<(), Self::Error>;

    /// Write a string to a file.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to write to.
    /// * `content` - The content to write.
    ///
    /// # Returns
    ///
    /// Ok on success, or an error.
    fn write_file_string(&self, path: &Path, content: &str) -> Result<(), Self::Error> {
        self.write_file(path, content.as_bytes())
    }

    /// Check if a file exists.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to check.
    ///
    /// # Returns
    ///
    /// True if the file exists, false otherwise.
    fn file_exists(&self, path: &Path) -> bool;

    /// Get the file modification time.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to check.
    ///
    /// # Returns
    ///
    /// The modification time, or an error.
    fn get_modification_time(&self, path: &Path) -> Result<SystemTime, Self::Error>;

    /// Find a file in the resource path.
    ///
    /// Searches for a file in the configured resource paths.
    ///
    /// # Arguments
    ///
    /// * `filename` - The filename to search for.
    ///
    /// # Returns
    ///
    /// The full path if found, None otherwise.
    fn find_file(&self, filename: &str) -> Option<PathBuf>;

    // =========================================================================
    // Logging and Reporting
    // =========================================================================

    /// Report a log message.
    ///
    /// This is the primary logging method. The message will be logged
    /// based on the current verbosity level.
    ///
    /// # Arguments
    ///
    /// * `level` - The log level.
    /// * `message` - The message to log.
    fn report(&self, level: LogLevel, message: String);

    /// Log a debug message.
    fn debug(&self, message: &str) {
        self.report(LogLevel::Debug, message.to_string());
    }

    /// Log an info message.
    fn info(&self, message: &str) {
        self.report(LogLevel::Info, message.to_string());
    }

    /// Log a warning message.
    fn warn(&self, message: &str) {
        self.report(LogLevel::Warning, message.to_string());
    }

    /// Log an error message.
    fn error(&self, message: &str) {
        self.report(LogLevel::Error, message.to_string());
    }

    /// Get all logged messages.
    fn get_logs(&self) -> Vec<LogMessage>;

    /// Get the current verbosity level.
    fn get_verbosity(&self) -> Verbosity;

    /// Set the verbosity level.
    fn set_verbosity(&mut self, verbosity: Verbosity);

    // =========================================================================
    // State Management
    // =========================================================================

    /// Get the common state.
    fn get_state(&self) -> &CommonState;

    /// Get a mutable reference to the common state.
    fn get_state_mut(&mut self) -> &mut CommonState;

    /// Get the resource path.
    fn get_resource_path(&self) -> &[PathBuf] {
        &self.get_state().resource_path
    }

    /// Add a resource path.
    fn add_resource_path(&mut self, path: PathBuf) {
        self.get_state_mut().add_resource_path(path);
    }

    /// Get the user data directory.
    fn get_user_data_dir(&self) -> Option<&PathBuf> {
        self.get_state().user_data_dir.as_ref()
    }

    /// Set the user data directory.
    fn set_user_data_dir(&mut self, path: Option<PathBuf>) {
        self.get_state_mut().user_data_dir = path;
    }

    /// Get the output format.
    fn get_output_format(&self) -> Option<&str> {
        self.get_state().output_format.as_deref()
    }

    /// Set the output format.
    fn set_output_format(&mut self, format: Option<String>) {
        self.get_state_mut().output_format = format;
    }

    /// Get the input filename.
    fn get_input_filename(&self) -> Option<&str> {
        self.get_state().input_filename.as_deref()
    }

    /// Set the input filename.
    fn set_input_filename(&mut self, filename: Option<String>) {
        self.get_state_mut().input_filename = filename;
    }

    // =========================================================================
    // Media Bag Operations
    // =========================================================================

    /// Get the media bag.
    fn get_media_bag(&self) -> Arc<Mutex<MediaBag>> {
        Arc::clone(&self.get_state().media_bag)
    }

    /// Insert media into the media bag.
    ///
    /// # Arguments
    ///
    /// * `path` - The path/key for the media.
    /// * `mime_type` - Optional MIME type.
    /// * `data` - The media data.
    ///
    /// # Returns
    ///
    /// The canonical path used for storage.
    fn insert_media(
        &self,
        path: &Path,
        mime_type: Option<&str>,
        data: Vec<u8>,
    ) -> Result<String, Self::Error>;

    /// Lookup media in the media bag.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to look up.
    ///
    /// # Returns
    ///
    /// The media item if found, None otherwise.
    fn lookup_media(&self, path: &Path) -> Option<crate::mediabag::MediaItem>;

    // =========================================================================
    // Time and Random (for test isolation)
    // =========================================================================

    /// Get the current time.
    ///
    /// This allows tests to mock time for reproducible results.
    fn get_current_time(&self) -> SystemTime;

    /// Generate random bytes.
    ///
    /// This allows tests to mock randomness for reproducible results.
    ///
    /// # Arguments
    ///
    /// * `len` - The number of bytes to generate.
    ///
    /// # Returns
    ///
    /// A vector of random bytes.
    fn get_random_bytes(&self, len: usize) -> Vec<u8>;
}

/// Common functionality for context implementations.
pub(crate) mod common {
    use super::*;
    use std::path::Path;

    /// Canonicalize a path for use in the media bag.
    ///
    /// This normalizes paths to use forward slashes and removes
    /// redundant components.
    pub fn canonicalize_path(path: &Path) -> String {
        let path_str = path.to_string_lossy();
        // Use forward slashes for cross-platform consistency
        path_str.replace('\\', "/")
    }

    /// Check if a path is a data URI.
    pub fn is_data_uri(path: &str) -> bool {
        path.starts_with("data:")
    }

    /// Get the default user data directory.
    pub fn default_user_data_dir() -> Option<PathBuf> {
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

/// Legacy Context trait for backward compatibility.
///
/// This trait is deprecated in favor of `ClmdContext`.
/// It is kept for backward compatibility with existing code.
#[deprecated(since = "0.2.0", note = "Use ClmdContext instead")]
pub trait Context: Send + Sync {
    /// Read a file into memory.
    fn read_file(&self, path: &Path) -> ClmdResult<Vec<u8>>;

    /// Write bytes to a file.
    fn write_file(&self, path: &Path, content: &[u8]) -> ClmdResult<()>;

    /// Check if a file exists.
    fn file_exists(&self, path: &Path) -> bool;

    /// Log a message.
    fn log(&self, level: LogLevel, message: &str);

    /// Get all logged messages.
    fn get_logs(&self) -> Vec<LogMessage>;

    /// Get the media bag.
    fn get_media_bag(&self) -> Arc<Mutex<MediaBag>>;

    /// Insert media into the media bag.
    fn insert_media(
        &self,
        path: &Path,
        mime_type: Option<&str>,
        data: Vec<u8>,
    ) -> ClmdResult<String>;

    /// Lookup media in the media bag.
    fn lookup_media(&self, path: &Path) -> Option<crate::mediabag::MediaItem>;

    /// Get the user data directory.
    fn get_user_data_dir(&self) -> Option<PathBuf>;

    /// Get the verbosity level.
    fn get_verbosity(&self) -> u8;

    /// Set the verbosity level.
    fn set_verbosity(&self, level: u8);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_log_level_ordering() {
        assert!(LogLevel::Debug < LogLevel::Info);
        assert!(LogLevel::Info < LogLevel::Warning);
        assert!(LogLevel::Warning < LogLevel::Error);
    }

    #[test]
    fn test_log_level_as_str() {
        assert_eq!(LogLevel::Debug.as_str(), "DEBUG");
        assert_eq!(LogLevel::Info.as_str(), "INFO");
        assert_eq!(LogLevel::Warning.as_str(), "WARNING");
        assert_eq!(LogLevel::Error.as_str(), "ERROR");
    }

    #[test]
    fn test_log_level_should_log() {
        // Quiet mode (0) - only errors
        assert!(LogLevel::Error.should_log(0));
        assert!(!LogLevel::Warning.should_log(0));
        assert!(!LogLevel::Info.should_log(0));
        assert!(!LogLevel::Debug.should_log(0));

        // Normal mode (1) - errors, warnings, info
        assert!(LogLevel::Error.should_log(1));
        assert!(LogLevel::Warning.should_log(1));
        assert!(LogLevel::Info.should_log(1));
        assert!(!LogLevel::Debug.should_log(1));

        // Verbose mode (2) - everything
        assert!(LogLevel::Error.should_log(2));
        assert!(LogLevel::Warning.should_log(2));
        assert!(LogLevel::Info.should_log(2));
        assert!(LogLevel::Debug.should_log(2));
    }

    #[test]
    fn test_log_message() {
        let msg = LogMessage::new(LogLevel::Info, "test message");
        assert_eq!(msg.level, LogLevel::Info);
        assert_eq!(msg.message, "test message");
        assert!(msg.source.is_none());
    }

    #[test]
    fn test_log_message_with_source() {
        let msg = LogMessage::with_source(LogLevel::Warning, "test", "parser.rs:42");
        assert_eq!(msg.level, LogLevel::Warning);
        assert_eq!(msg.message, "test");
        assert_eq!(msg.source, Some("parser.rs:42".to_string()));
    }

    #[test]
    fn test_log_message_format() {
        let msg = LogMessage::new(LogLevel::Info, "Hello");
        assert!(msg.format().contains("INFO"));
        assert!(msg.format().contains("Hello"));

        let msg_with_source =
            LogMessage::with_source(LogLevel::Error, "Oops", "main.rs:10");
        assert!(msg_with_source.format().contains("ERROR"));
        assert!(msg_with_source.format().contains("Oops"));
        assert!(msg_with_source.format().contains("main.rs:10"));
    }

    #[test]
    fn test_verbosity() {
        assert_eq!(Verbosity::Quiet.as_u8(), 0);
        assert_eq!(Verbosity::Normal.as_u8(), 1);
        assert_eq!(Verbosity::Verbose.as_u8(), 2);

        assert_eq!(Verbosity::from_u8(0), Verbosity::Quiet);
        assert_eq!(Verbosity::from_u8(1), Verbosity::Normal);
        assert_eq!(Verbosity::from_u8(2), Verbosity::Verbose);
        assert_eq!(Verbosity::from_u8(3), Verbosity::Verbose);
    }

    #[test]
    fn test_common_state_default() {
        let state = CommonState::default();
        assert_eq!(state.verbosity, Verbosity::Normal);
        assert!(state.resource_path.is_empty());
        assert!(state.user_data_dir.is_none());
        assert!(state.output_format.is_none());
        assert!(state.input_filename.is_none());
    }

    #[test]
    fn test_common_state_logging() {
        let state = CommonState::new();
        state.log(LogLevel::Info, "Test message");

        let logs = state.get_logs();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].level, LogLevel::Info);
    }

    #[test]
    fn test_common_state_log_filtering() {
        let mut state = CommonState::new();
        state.set_verbosity(Verbosity::Quiet);

        state.log(LogLevel::Debug, "Debug");
        state.log(LogLevel::Info, "Info");
        state.log(LogLevel::Warning, "Warning");
        state.log(LogLevel::Error, "Error");

        let logs = state.get_logs();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].level, LogLevel::Error);
    }

    #[test]
    fn test_common_state_has_errors_warnings() {
        let state = CommonState::new();
        assert!(!state.has_errors());
        assert!(!state.has_warnings());

        state.log(LogLevel::Warning, "Warning");
        assert!(!state.has_errors());
        assert!(state.has_warnings());

        state.log(LogLevel::Error, "Error");
        assert!(state.has_errors());
        assert!(state.has_warnings());
    }

    #[test]
    fn test_canonicalize_path() {
        let path = PathBuf::from("foo\\bar\\baz");
        assert_eq!(common::canonicalize_path(&path), "foo/bar/baz");

        let path = PathBuf::from("foo/bar/baz");
        assert_eq!(common::canonicalize_path(&path), "foo/bar/baz");
    }

    #[test]
    fn test_is_data_uri() {
        assert!(common::is_data_uri("data:image/png;base64,abc"));
        assert!(!common::is_data_uri("https://example.com/image.png"));
        assert!(!common::is_data_uri("/path/to/file"));
    }
}
