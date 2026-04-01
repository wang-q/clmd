//! Common types and utilities for the context system.
//!
//! This module provides shared types and helper functions used by both
//! [`IoContext`](crate::context::IoContext) and [`PureContext`](crate::context::PureContext).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use crate::context::mediabag::MediaBag;

/// The ClmdContext trait defines the interface for context operations.
///
/// This trait abstracts over IO operations, logging, and resource management,
/// allowing for both real IO operations and pure/mock implementations.
pub trait ClmdContext {
    /// The error type returned by operations.
    type Error;

    /// Read a file's contents.
    fn read_file(&self, path: &Path) -> Result<Vec<u8>, Self::Error>;

    /// Write content to a file.
    fn write_file(&self, path: &Path, content: &[u8]) -> Result<(), Self::Error>;

    /// Check if a file exists.
    fn file_exists(&self, path: &Path) -> bool;

    /// Get the modification time of a file.
    fn get_modification_time(&self, path: &Path) -> Result<SystemTime, Self::Error>;

    /// Find a file in the search path.
    fn find_file(&self, filename: &str) -> Option<PathBuf>;

    /// Report a log message.
    fn report(&self, level: LogLevel, message: String);

    /// Get all logged messages.
    fn get_logs(&self) -> Vec<LogMessage>;

    /// Get the current verbosity level.
    fn get_verbosity(&self) -> Verbosity;

    /// Set the verbosity level.
    fn set_verbosity(&mut self, verbosity: Verbosity);

    /// Get a reference to the common state.
    fn get_state(&self) -> &CommonState;

    /// Get a mutable reference to the common state.
    fn get_state_mut(&mut self) -> &mut CommonState;

    /// Insert media into the media bag.
    fn insert_media(
        &self,
        path: &Path,
        mime_type: Option<&str>,
        data: Vec<u8>,
    ) -> Result<String, Self::Error>;

    /// Lookup media in the media bag.
    fn lookup_media(&self, path: &Path) -> Option<crate::context::mediabag::MediaItem>;

    /// Get the current time.
    fn get_current_time(&self) -> SystemTime;

    /// Get random bytes.
    fn get_random_bytes(&self, len: usize) -> Vec<u8>;
}

/// Common state shared between context implementations.
#[derive(Debug, Clone)]
pub struct CommonState {
    /// The user data directory.
    pub user_data_dir: Option<PathBuf>,
    /// The media bag for storing binary resources.
    pub media_bag: Arc<Mutex<MediaBag>>,
    /// The verbosity level.
    pub verbosity: Verbosity,
    /// Log messages.
    pub logs: Arc<Mutex<Vec<LogMessage>>>,
    /// Search path for files.
    pub search_path: Vec<PathBuf>,
    /// Environment variables.
    pub env_vars: HashMap<String, String>,
}

impl Default for CommonState {
    fn default() -> Self {
        Self {
            user_data_dir: None,
            media_bag: Arc::new(Mutex::new(MediaBag::new())),
            verbosity: Verbosity::Normal,
            logs: Arc::new(Mutex::new(Vec::new())),
            search_path: Vec::new(),
            env_vars: std::env::vars().collect(),
        }
    }
}

impl CommonState {
    /// Find a file in the search path.
    pub fn find_file(&self, filename: &str) -> Option<PathBuf> {
        // First check if it's an absolute path
        let path = Path::new(filename);
        if path.is_absolute() && path.exists() {
            return Some(path.to_path_buf());
        }

        // Search in the search path
        for dir in &self.search_path {
            let full_path = dir.join(filename);
            if full_path.exists() {
                return Some(full_path);
            }
        }

        // Check in user data directory
        if let Some(user_dir) = &self.user_data_dir {
            let full_path = user_dir.join(filename);
            if full_path.exists() {
                return Some(full_path);
            }
        }

        None
    }

    /// Log a message.
    pub fn log(&self, level: LogLevel, message: String) {
        if level.should_log(self.verbosity.as_u8()) {
            let mut logs = self.logs.lock().unwrap();
            logs.push(LogMessage {
                level,
                message,
                timestamp: SystemTime::now(),
            });
        }
    }

    /// Get all logs.
    pub fn get_logs(&self) -> Vec<LogMessage> {
        self.logs.lock().unwrap().clone()
    }

    /// Clear all logs.
    pub fn clear_logs(&self) {
        self.logs.lock().unwrap().clear();
    }

    /// Check if any errors were logged.
    pub fn has_errors(&self) -> bool {
        self.logs
            .lock()
            .unwrap()
            .iter()
            .any(|log| log.level == LogLevel::Error)
    }

    /// Check if any warnings were logged.
    pub fn has_warnings(&self) -> bool {
        self.logs
            .lock()
            .unwrap()
            .iter()
            .any(|log| log.level == LogLevel::Warning)
    }
}

/// A log message.
#[derive(Debug, Clone)]
pub struct LogMessage {
    /// The log level.
    pub level: LogLevel,
    /// The log message.
    pub message: String,
    /// The timestamp.
    pub timestamp: SystemTime,
}

/// Log level for context operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    /// Debug level.
    Debug,
    /// Info level.
    Info,
    /// Warning level.
    Warning,
    /// Error level.
    Error,
}

impl LogLevel {
    /// Check if this level should be logged given a verbosity.
    pub fn should_log(&self, verbosity: u8) -> bool {
        match self {
            LogLevel::Error => true,
            LogLevel::Warning => verbosity >= 1,
            LogLevel::Info => verbosity >= 1,
            LogLevel::Debug => verbosity >= 2,
        }
    }
}

/// Verbosity level for context operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verbosity {
    /// Quiet mode - only errors.
    Quiet,
    /// Normal mode - errors and warnings.
    Normal,
    /// Info mode - errors, warnings, and info.
    Info,
    /// Debug mode - all messages.
    Debug,
}

impl Verbosity {
    /// Convert to a numeric value.
    pub fn as_u8(&self) -> u8 {
        match self {
            Verbosity::Quiet => 0,
            Verbosity::Normal => 1,
            Verbosity::Info => 1,
            Verbosity::Debug => 2,
        }
    }
}

impl Default for Verbosity {
    fn default() -> Self {
        Verbosity::Normal
    }
}

/// Get the default user data directory.
pub fn default_user_data_dir() -> Option<PathBuf> {
    // Try XDG_DATA_HOME first
    if let Ok(xdg_data) = std::env::var("XDG_DATA_HOME") {
        let path = PathBuf::from(xdg_data).join("clmd");
        return Some(path);
    }

    // Try home directory
    if let Some(home) = dirs::home_dir() {
        // Check for XDG default location
        let xdg_path = home.join(".local").join("share").join("clmd");
        if xdg_path.exists() {
            return Some(xdg_path);
        }

        // Check for legacy location
        let legacy_path = home.join(".clmd");
        if legacy_path.exists() {
            return Some(legacy_path);
        }

        // Return XDG default as the preferred location
        return Some(xdg_path);
    }

    None
}

/// Check if a string is a data URI.
pub fn is_data_uri(s: &str) -> bool {
    s.starts_with("data:")
}

/// Canonicalize a path.
pub fn canonicalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// Generate a hash-based path for data URIs.
///
/// This is used by `insert_media` implementations to generate a unique
/// path for data URI content based on the content hash.
///
/// # Arguments
///
/// * `data` - The binary content to hash
/// * `mime_type` - Optional MIME type for determining file extension
///
/// # Returns
///
/// A hash-based path string (e.g., "abc123.png")
pub fn generate_hash_path(data: &[u8], mime_type: Option<&str>) -> String {
    let hash = format!("{:x}", md5::compute(data));
    let ext = mime_type
        .and_then(|m| m.split('/').nth(1))
        .map(|s| s.split(';').next().unwrap_or(s))
        .map(|s| format!(".{}", s))
        .unwrap_or_default();
    format!("{}{}", hash, ext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_should_log() {
        assert!(LogLevel::Error.should_log(0));
        assert!(!LogLevel::Warning.should_log(0));
        assert!(LogLevel::Warning.should_log(1));
        assert!(!LogLevel::Debug.should_log(1));
        assert!(LogLevel::Debug.should_log(2));
    }

    #[test]
    fn test_verbosity_as_u8() {
        assert_eq!(Verbosity::Quiet.as_u8(), 0);
        assert_eq!(Verbosity::Normal.as_u8(), 1);
        assert_eq!(Verbosity::Info.as_u8(), 1);
        assert_eq!(Verbosity::Debug.as_u8(), 2);
    }

    #[test]
    fn test_is_data_uri() {
        assert!(is_data_uri("data:image/png;base64,abc"));
        assert!(!is_data_uri("https://example.com"));
    }

    #[test]
    fn test_canonicalize_path() {
        let path = Path::new("dir\\file.txt");
        assert_eq!(canonicalize_path(path), "dir/file.txt");
    }
}
