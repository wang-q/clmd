//! Context system for clmd, inspired by Pandoc's PandocMonad.
//!
//! This module provides a unified abstraction for IO operations, logging,
//! and resource management. It allows for both real IO operations and
//! pure/mock implementations for testing.
//!
//! # Example
//!
//! ```
//! use clmd::context::{Context, IoContext};
//! use std::path::Path;
//!
//! let ctx = IoContext::new();
//! // Use ctx for file operations, logging, etc.
//! ```

use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::error::ClmdResult;
use crate::mediabag::MediaBag;

mod io;
mod pure;

pub use io::IoContext;
pub use pure::PureContext;

/// Log level for context logging.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
}

impl LogMessage {
    /// Create a new log message.
    pub fn new(level: LogLevel, message: impl Into<String>) -> Self {
        Self {
            level,
            message: message.into(),
            source: None,
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
        }
    }
}

/// Context trait for clmd operations.
///
/// This trait abstracts over IO operations, logging, and resource management,
/// allowing for both real IO and pure/mock implementations.
///
/// # Safety
///
/// Implementations must be thread-safe (Send + Sync).
pub trait Context: Send + Sync {
    /// Read a file into memory.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the file.
    ///
    /// # Returns
    ///
    /// The file contents as bytes, or an error.
    fn read_file(&self, path: &Path) -> ClmdResult<Vec<u8>>;

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
    fn write_file(&self, path: &Path, content: &[u8]) -> ClmdResult<()>;

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

    /// Log a message.
    ///
    /// # Arguments
    ///
    /// * `level` - The log level.
    /// * `message` - The message to log.
    fn log(&self, level: LogLevel, message: &str);

    /// Get all logged messages.
    ///
    /// # Returns
    ///
    /// A vector of all log messages.
    fn get_logs(&self) -> Vec<LogMessage>;

    /// Get the media bag.
    ///
    /// # Returns
    ///
    /// A reference to the media bag.
    fn get_media_bag(&self) -> Arc<Mutex<MediaBag>>;

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
    ) -> ClmdResult<String>;

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

    /// Get the user data directory.
    ///
    /// # Returns
    ///
    /// The path to the user data directory, if available.
    fn get_user_data_dir(&self) -> Option<std::path::PathBuf>;

    /// Get the verbosity level.
    ///
    /// # Returns
    ///
    /// The current verbosity level (0 = quiet, 1 = normal, 2 = verbose).
    fn get_verbosity(&self) -> u8;

    /// Set the verbosity level.
    ///
    /// # Arguments
    ///
    /// * `level` - The verbosity level.
    fn set_verbosity(&self, level: u8);
}

/// Common functionality for context implementations.
pub(crate) mod common {
    use super::*;

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
