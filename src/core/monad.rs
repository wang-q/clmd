//! Monad abstraction layer for clmd.
//!
//! This module provides a unified interface for IO and pure operations,
//! inspired by Pandoc's PandocMonad typeclass. This allows for better
//! testability and flexibility in the codebase.
//!
//! # Example
//!
//! ```
//! use clmd::core::{ClmdMonad, ClmdIO, ClmdPure};
//! use std::path::Path;
//!
//! fn read_document<M: ClmdMonad>(monad: &M, path: &Path) -> Result<String, clmd::error::ClmdError> {
//!     monad.read_file(path)
//! }
//! ```

use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::error::ClmdError;

/// Verbosity level for logging.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Verbosity {
    /// Silent - no output.
    Silent,
    /// Error - only errors.
    Error,
    /// Warning - errors and warnings.
    Warning,
    /// Info - normal informational output.
    Info,
    /// Debug - detailed debug information.
    Debug,
}

impl Default for Verbosity {
    fn default() -> Self {
        Self::Warning
    }
}

/// A log message for reporting progress and issues.
#[derive(Debug, Clone)]
pub struct LogMessage {
    /// The verbosity level of this message.
    pub level: Verbosity,
    /// The message content.
    pub message: String,
    /// Optional source position.
    pub position: Option<(usize, usize)>,
}

impl LogMessage {
    /// Create a new log message.
    pub fn new<S: Into<String>>(level: Verbosity, message: S) -> Self {
        Self {
            level,
            message: message.into(),
            position: None,
        }
    }

    /// Create an error message.
    pub fn error<S: Into<String>>(message: S) -> Self {
        Self::new(Verbosity::Error, message)
    }

    /// Create a warning message.
    pub fn warning<S: Into<String>>(message: S) -> Self {
        Self::new(Verbosity::Warning, message)
    }

    /// Create an info message.
    pub fn info<S: Into<String>>(message: S) -> Self {
        Self::new(Verbosity::Info, message)
    }

    /// Create a debug message.
    pub fn debug<S: Into<String>>(message: S) -> Self {
        Self::new(Verbosity::Debug, message)
    }

    /// Add position information.
    pub fn with_position(mut self, line: usize, column: usize) -> Self {
        self.position = Some((line, column));
        self
    }
}

/// The core monad trait for clmd operations.
///
/// This trait abstracts over IO and pure operations, allowing code to be
/// written generically and tested without actual file system operations.
///
/// # Type Parameters
///
/// - `E`: The error type (defaults to `ClmdError`)
pub trait ClmdMonad {
    /// The error type used by this monad.
    type Error: From<ClmdError> + From<io::Error>;

    /// Read a file from the file system.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read.
    fn read_file(&self, path: &Path) -> Result<String, Self::Error>;

    /// Read a file as bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read.
    fn read_file_bytes(&self, path: &Path) -> Result<Vec<u8>, Self::Error>;

    /// Write content to a file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written.
    fn write_file(&self, path: &Path, content: &str) -> Result<(), Self::Error>;

    /// Write bytes to a file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written.
    fn write_file_bytes(&self, path: &Path, content: &[u8]) -> Result<(), Self::Error>;

    /// Check if a file exists.
    fn file_exists(&self, path: &Path) -> bool;

    /// Get the resource search paths.
    fn get_resource_paths(&self) -> Vec<PathBuf>;

    /// Fetch a resource from a URL or path.
    ///
    /// For local paths, this reads the file. For URLs, this would
    /// perform an HTTP request (if the HTTP feature is enabled).
    ///
    /// # Errors
    ///
    /// Returns an error if the resource cannot be fetched.
    fn fetch_resource(&self, url: &str) -> Result<Vec<u8>, Self::Error>;

    /// Report a log message.
    fn report(&self, msg: LogMessage);

    /// Get the current verbosity level.
    fn get_verbosity(&self) -> Verbosity;

    /// Check if the given verbosity level is enabled.
    fn is_verbosity_enabled(&self, level: Verbosity) -> bool {
        self.get_verbosity() >= level
    }

    /// Get the current timestamp.
    fn get_timestamp(&self) -> SystemTime;

    /// Get the current working directory.
    fn get_current_dir(&self) -> Result<PathBuf, Self::Error>;

    /// Get the user's data directory.
    fn get_user_data_dir(&self) -> Option<PathBuf>;

    /// Log an error message.
    fn log_error<S: Into<String>>(&self, message: S) {
        self.report(LogMessage::error(message));
    }

    /// Log a warning message.
    fn log_warning<S: Into<String>>(&self, message: S) {
        self.report(LogMessage::warning(message));
    }

    /// Log an info message.
    fn log_info<S: Into<String>>(&self, message: S) {
        self.report(LogMessage::info(message));
    }

    /// Log a debug message.
    fn log_debug<S: Into<String>>(&self, message: S) {
        self.report(LogMessage::debug(message));
    }
}

/// The IO implementation of ClmdMonad.
///
/// This is the production implementation that performs actual file system
/// operations and network requests.
#[derive(Debug, Clone)]
pub struct ClmdIO {
    verbosity: Verbosity,
    resource_paths: Vec<PathBuf>,
    user_data_dir: Option<PathBuf>,
}

impl ClmdIO {
    /// Create a new ClmdIO instance.
    pub fn new() -> Self {
        Self {
            verbosity: Verbosity::default(),
            resource_paths: vec![PathBuf::from(".")],
            user_data_dir: dirs::data_dir().map(|d| d.join("clmd")),
        }
    }

    /// Create a new ClmdIO instance with a specific verbosity level.
    pub fn with_verbosity(verbosity: Verbosity) -> Self {
        Self {
            verbosity,
            resource_paths: vec![PathBuf::from(".")],
            user_data_dir: dirs::data_dir().map(|d| d.join("clmd")),
        }
    }

    /// Set the resource search paths.
    pub fn with_resource_paths(mut self, paths: Vec<PathBuf>) -> Self {
        self.resource_paths = paths;
        self
    }

    /// Set the user data directory.
    pub fn with_user_data_dir(mut self, dir: PathBuf) -> Self {
        self.user_data_dir = Some(dir);
        self
    }

    /// Get the default resource paths.
    fn get_default_resource_paths() -> Vec<PathBuf> {
        let mut paths = vec![PathBuf::from(".")];

        // Add user's data directory
        if let Some(data_dir) = dirs::data_dir() {
            paths.push(data_dir.join("clmd"));
        }

        // Add system-wide directories
        #[cfg(target_os = "macos")]
        {
            paths.push(PathBuf::from("/usr/local/share/clmd"));
            paths.push(PathBuf::from("/usr/share/clmd"));
        }

        #[cfg(target_os = "linux")]
        {
            paths.push(PathBuf::from("/usr/local/share/clmd"));
            paths.push(PathBuf::from("/usr/share/clmd"));
        }

        paths
    }
}

impl Default for ClmdIO {
    fn default() -> Self {
        Self::new()
    }
}

impl ClmdMonad for ClmdIO {
    type Error = ClmdError;

    fn read_file(&self, path: &Path) -> Result<String, Self::Error> {
        std::fs::read_to_string(path).map_err(|e| {
            ClmdError::io_error(format!("Failed to read file '{}': {}", path.display(), e))
        })
    }

    fn read_file_bytes(&self, path: &Path) -> Result<Vec<u8>, Self::Error> {
        std::fs::read(path).map_err(|e| {
            ClmdError::io_error(format!("Failed to read file '{}': {}", path.display(), e))
        })
    }

    fn write_file(&self, path: &Path, content: &str) -> Result<(), Self::Error> {
        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(path, content).map_err(|e| {
            ClmdError::io_error(format!("Failed to write file '{}': {}", path.display(), e))
        })
    }

    fn write_file_bytes(&self, path: &Path, content: &[u8]) -> Result<(), Self::Error> {
        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(path, content).map_err(|e| {
            ClmdError::io_error(format!("Failed to write file '{}': {}", path.display(), e))
        })
    }

    fn file_exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn get_resource_paths(&self) -> Vec<PathBuf> {
        self.resource_paths.clone()
    }

    fn fetch_resource(&self, url: &str) -> Result<Vec<u8>, Self::Error> {
        // Check if it's a URL or a local path
        if url.starts_with("http://") || url.starts_with("https://") {
            // For now, return an error - HTTP support would require a feature flag
            Err(ClmdError::feature_not_enabled("http"))
        } else {
            // Try as a local path
            let path = Path::new(url);

            // Try relative to resource paths
            for base in &self.resource_paths {
                let full_path = base.join(path);
                if full_path.exists() {
                    return self.read_file_bytes(&full_path);
                }
            }

            // Try as absolute path
            if path.is_absolute() && path.exists() {
                return self.read_file_bytes(path);
            }

            Err(ClmdError::resource_not_found(url))
        }
    }

    fn report(&self, msg: LogMessage) {
        if self.verbosity >= msg.level {
            let prefix = match msg.level {
                Verbosity::Error => "ERROR",
                Verbosity::Warning => "WARNING",
                Verbosity::Info => "INFO",
                Verbosity::Debug => "DEBUG",
                _ => "",
            };

            if msg.level == Verbosity::Silent {
                return;
            }

            if let Some((line, col)) = msg.position {
                eprintln!("[{}] {} (at {}:{})", prefix, msg.message, line, col);
            } else {
                eprintln!("[{}] {}", prefix, msg.message);
            }
        }
    }

    fn get_verbosity(&self) -> Verbosity {
        self.verbosity
    }

    fn get_timestamp(&self) -> SystemTime {
        SystemTime::now()
    }

    fn get_current_dir(&self) -> Result<PathBuf, Self::Error> {
        std::env::current_dir().map_err(|e| {
            ClmdError::io_error(format!("Failed to get current directory: {}", e))
        })
    }

    fn get_user_data_dir(&self) -> Option<PathBuf> {
        self.user_data_dir.clone()
    }
}

/// The pure implementation of ClmdMonad for testing.
///
/// This implementation uses in-memory storage and does not perform any
/// actual file system operations. It's useful for unit testing.
#[derive(Debug, Clone)]
pub struct ClmdPure {
    verbosity: Verbosity,
    files: std::collections::HashMap<PathBuf, String>,
    binary_files: std::collections::HashMap<PathBuf, Vec<u8>>,
    resource_paths: Vec<PathBuf>,
    logs: Vec<LogMessage>,
    timestamp: SystemTime,
    current_dir: PathBuf,
    user_data_dir: Option<PathBuf>,
}

impl Default for ClmdPure {
    fn default() -> Self {
        Self {
            verbosity: Verbosity::default(),
            files: std::collections::HashMap::new(),
            binary_files: std::collections::HashMap::new(),
            resource_paths: vec![PathBuf::from(".")],
            logs: Vec::new(),
            timestamp: SystemTime::UNIX_EPOCH,
            current_dir: PathBuf::from("/test"),
            user_data_dir: Some(PathBuf::from("/test/.local/share/clmd")),
        }
    }
}

impl ClmdPure {
    /// Create a new ClmdPure instance.
    pub fn new() -> Self {
        Self {
            verbosity: Verbosity::default(),
            files: std::collections::HashMap::new(),
            binary_files: std::collections::HashMap::new(),
            resource_paths: vec![PathBuf::from(".")],
            logs: Vec::new(),
            timestamp: SystemTime::UNIX_EPOCH,
            current_dir: PathBuf::from("/test"),
            user_data_dir: Some(PathBuf::from("/test/.local/share/clmd")),
        }
    }

    /// Add a text file to the in-memory file system.
    pub fn with_file<S: Into<String>>(mut self, path: impl AsRef<Path>, content: S) -> Self {
        self.files.insert(path.as_ref().to_path_buf(), content.into());
        self
    }

    /// Add a binary file to the in-memory file system.
    pub fn with_binary_file(mut self, path: impl AsRef<Path>, content: Vec<u8>) -> Self {
        self.binary_files.insert(path.as_ref().to_path_buf(), content);
        self
    }

    /// Set the verbosity level.
    pub fn with_verbosity(mut self, verbosity: Verbosity) -> Self {
        self.verbosity = verbosity;
        self
    }

    /// Set the current timestamp.
    pub fn with_timestamp(mut self, timestamp: SystemTime) -> Self {
        self.timestamp = timestamp;
        self
    }

    /// Set the current directory.
    pub fn with_current_dir(mut self, dir: impl AsRef<Path>) -> Self {
        self.current_dir = dir.as_ref().to_path_buf();
        self
    }

    /// Get all logged messages.
    pub fn get_logs(&self) -> &[LogMessage] {
        &self.logs
    }

    /// Clear all logged messages.
    pub fn clear_logs(&mut self) {
        self.logs.clear();
    }

    /// Check if a message was logged.
    pub fn has_logged(&self, level: Verbosity, message: &str) -> bool {
        self.logs.iter().any(|log| log.level == level && log.message.contains(message))
    }
}

impl ClmdMonad for ClmdPure {
    type Error = ClmdError;

    fn read_file(&self, path: &Path) -> Result<String, Self::Error> {
        self.files.get(path).cloned().ok_or_else(|| {
            ClmdError::io_error(format!("File not found: {}", path.display()))
        })
    }

    fn read_file_bytes(&self, path: &Path) -> Result<Vec<u8>, Self::Error> {
        self.binary_files
            .get(path)
            .cloned()
            .or_else(|| self.files.get(path).map(|s| s.as_bytes().to_vec()))
            .ok_or_else(|| {
                ClmdError::io_error(format!("File not found: {}", path.display()))
            })
    }

    fn write_file(&self, _path: &Path, _content: &str) -> Result<(), Self::Error> {
        // In pure mode, we don't actually write files
        // This could be extended to track writes if needed
        Ok(())
    }

    fn write_file_bytes(&self, _path: &Path, _content: &[u8]) -> Result<(), Self::Error> {
        Ok(())
    }

    fn file_exists(&self, path: &Path) -> bool {
        self.files.contains_key(path) || self.binary_files.contains_key(path)
    }

    fn get_resource_paths(&self) -> Vec<PathBuf> {
        self.resource_paths.clone()
    }

    fn fetch_resource(&self, url: &str) -> Result<Vec<u8>, Self::Error> {
        // In pure mode, treat as local path
        let path = Path::new(url);
        self.read_file_bytes(path)
    }

    fn report(&self, msg: LogMessage) {
        // Store the log message
        // Note: This requires interior mutability in practice
        // For simplicity, we just print in the pure implementation
        if self.verbosity >= msg.level {
            let prefix = match msg.level {
                Verbosity::Error => "ERROR",
                Verbosity::Warning => "WARNING",
                Verbosity::Info => "INFO",
                Verbosity::Debug => "DEBUG",
                _ => "",
            };

            if msg.level != Verbosity::Silent {
                println!("[PURE {}] {}", prefix, msg.message);
            }
        }
    }

    fn get_verbosity(&self) -> Verbosity {
        self.verbosity
    }

    fn get_timestamp(&self) -> SystemTime {
        self.timestamp
    }

    fn get_current_dir(&self) -> Result<PathBuf, Self::Error> {
        Ok(self.current_dir.clone())
    }

    fn get_user_data_dir(&self) -> Option<PathBuf> {
        self.user_data_dir.clone()
    }
}

/// A reference-counted monad for shared state.
///
/// This is useful when you need to share the monad state across multiple
/// operations, such as in a multi-threaded context.
pub type SharedMonad<M> = std::sync::Arc<M>;

/// Create a shared reference to a monad.
pub fn share_monad<M: ClmdMonad>(monad: M) -> SharedMonad<M> {
    std::sync::Arc::new(monad)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verbosity_ordering() {
        assert!(Verbosity::Silent < Verbosity::Error);
        assert!(Verbosity::Error < Verbosity::Warning);
        assert!(Verbosity::Warning < Verbosity::Info);
        assert!(Verbosity::Info < Verbosity::Debug);
    }

    #[test]
    fn test_log_message() {
        let msg = LogMessage::info("Test message");
        assert_eq!(msg.level, Verbosity::Info);
        assert_eq!(msg.message, "Test message");
        assert!(msg.position.is_none());

        let msg = LogMessage::error("Error").with_position(10, 5);
        assert_eq!(msg.position, Some((10, 5)));
    }

    #[test]
    fn test_clmd_io_default() {
        let io = ClmdIO::default();
        assert_eq!(io.get_verbosity(), Verbosity::Warning);
        assert!(!io.get_resource_paths().is_empty());
    }

    #[test]
    fn test_clmd_io_with_verbosity() {
        let io = ClmdIO::with_verbosity(Verbosity::Debug);
        assert_eq!(io.get_verbosity(), Verbosity::Debug);
        assert!(io.is_verbosity_enabled(Verbosity::Info));
        assert!(io.is_verbosity_enabled(Verbosity::Debug));
    }

    #[test]
    fn test_clmd_pure_file_operations() {
        let pure = ClmdPure::new()
            .with_file("/test/file.txt", "Hello, World!")
            .with_binary_file("/test/image.png", vec![0x89, 0x50, 0x4E, 0x47]);

        assert!(pure.file_exists(Path::new("/test/file.txt")));
        assert!(pure.file_exists(Path::new("/test/image.png")));
        assert!(!pure.file_exists(Path::new("/test/missing.txt")));

        let content = pure.read_file(Path::new("/test/file.txt")).unwrap();
        assert_eq!(content, "Hello, World!");

        let bytes = pure.read_file_bytes(Path::new("/test/image.png")).unwrap();
        assert_eq!(bytes, vec![0x89, 0x50, 0x4E, 0x47]);
    }

    #[test]
    fn test_clmd_pure_missing_file() {
        let pure = ClmdPure::new();

        let result = pure.read_file(Path::new("/test/missing.txt"));
        assert!(result.is_err());
    }

    #[test]
    fn test_clmd_pure_timestamp() {
        let pure = ClmdPure::new().with_timestamp(SystemTime::UNIX_EPOCH);
        assert_eq!(pure.get_timestamp(), SystemTime::UNIX_EPOCH);
    }

    #[test]
    fn test_clmd_pure_current_dir() {
        let pure = ClmdPure::new().with_current_dir("/custom/path");
        assert_eq!(pure.get_current_dir().unwrap(), PathBuf::from("/custom/path"));
    }

    #[test]
    fn test_share_monad() {
        let io = ClmdIO::new();
        let shared = share_monad(io);

        assert_eq!(shared.get_verbosity(), Verbosity::Warning);
    }
}
