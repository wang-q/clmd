//! Monad abstraction layer for clmd.
//!
//! This module provides a unified interface for IO and pure operations,
//! inspired by Pandoc's PandocMonad typeclass. This allows for better
//! testability and flexibility in the codebase.
//!
//! # Example
//!
//! ```ignore
//! use clmd::core::{ClmdMonad, ClmdIO, ClmdPure, Verbosity};
//!
//! let monad = ClmdIO::with_verbosity(Verbosity::Info);
//! monad.log_info("Starting document conversion");
//! ```

use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::core::error::{ClmdError, LogLevel, LogMessage};

/// Verbosity level for logging.

/// Type alias for the shared state monad.
pub type SharedMonad = ClmdIO;

/// Create a shared state monad with default configuration.
pub fn share_monad(io: ClmdIO) -> SharedMonad {
    io
}

/// Verbosity level for logging.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Verbosity {
    /// Silent - no output.
    Silent,
    /// Error - only errors.
    Error,
    /// Warning - errors and warnings.
    #[default]
    Warning,
    /// Info - normal informational output.
    Info,
    /// Debug - detailed debug information.
    Debug,
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
    sandbox: Option<crate::core::sandbox::SandboxPolicy>,
}

impl ClmdIO {
    /// Create a new ClmdIO instance.
    pub fn new() -> Self {
        Self {
            verbosity: Verbosity::default(),
            resource_paths: vec![PathBuf::from(".")],
            user_data_dir: dirs::data_dir().map(|d| d.join("clmd")),
            sandbox: None,
        }
    }

    /// Create a new ClmdIO instance with a specific verbosity level.
    pub fn with_verbosity(verbosity: Verbosity) -> Self {
        Self {
            verbosity,
            resource_paths: vec![PathBuf::from(".")],
            user_data_dir: dirs::data_dir().map(|d| d.join("clmd")),
            sandbox: None,
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

    /// Set the sandbox policy.
    pub fn with_sandbox(mut self, sandbox: crate::core::sandbox::SandboxPolicy) -> Self {
        self.sandbox = Some(sandbox);
        self
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
        // Check sandbox policy if set
        if let Some(ref sandbox) = self.sandbox {
            if !sandbox.is_path_allowed(path, &self.resource_paths) {
                return Err(ClmdError::sandbox_error(format!(
                    "Access to path '{}' is not allowed by sandbox policy",
                    path.display()
                )));
            }
        }

        std::fs::read_to_string(path).map_err(|e| {
            ClmdError::io_error(format!(
                "Failed to read file '{}': {}",
                path.display(),
                e
            ))
        })
    }

    fn read_file_bytes(&self, path: &Path) -> Result<Vec<u8>, Self::Error> {
        // Check sandbox policy if set
        if let Some(ref sandbox) = self.sandbox {
            if !sandbox.is_path_allowed(path, &self.resource_paths) {
                return Err(ClmdError::sandbox_error(format!(
                    "Access to path '{}' is not allowed by sandbox policy",
                    path.display()
                )));
            }
        }

        std::fs::read(path).map_err(|e| {
            ClmdError::io_error(format!(
                "Failed to read file '{}': {}",
                path.display(),
                e
            ))
        })
    }

    fn write_file(&self, path: &Path, content: &str) -> Result<(), Self::Error> {
        // Check sandbox policy if set
        if let Some(ref sandbox) = self.sandbox {
            if !sandbox.are_writes_allowed() {
                return Err(ClmdError::sandbox_error(
                    "File writes are not allowed by sandbox policy",
                ));
            }
            if !sandbox.is_path_allowed(path, &self.resource_paths) {
                return Err(ClmdError::sandbox_error(format!(
                    "Access to path '{}' is not allowed by sandbox policy",
                    path.display()
                )));
            }
        }

        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(path, content).map_err(|e| {
            ClmdError::io_error(format!(
                "Failed to write file '{}': {}",
                path.display(),
                e
            ))
        })
    }

    fn write_file_bytes(&self, path: &Path, content: &[u8]) -> Result<(), Self::Error> {
        // Check sandbox policy if set
        if let Some(ref sandbox) = self.sandbox {
            if !sandbox.are_writes_allowed() {
                return Err(ClmdError::sandbox_error(
                    "File writes are not allowed by sandbox policy",
                ));
            }
            if !sandbox.is_path_allowed(path, &self.resource_paths) {
                return Err(ClmdError::sandbox_error(format!(
                    "Access to path '{}' is not allowed by sandbox policy",
                    path.display()
                )));
            }
        }

        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(path, content).map_err(|e| {
            ClmdError::io_error(format!(
                "Failed to write file '{}': {}",
                path.display(),
                e
            ))
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
            // HTTP support is not implemented yet
            // This would require adding a dependency like reqwest or ureq
            Err(ClmdError::feature_not_enabled(
                "HTTP resource fetching is not enabled. \
                To fetch resources from URLs, enable the 'http' feature. \
                Alternatively, download the resource manually and reference it by local path."
            ))
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

            // Provide helpful error message with searched paths
            let searched_paths: Vec<String> = self
                .resource_paths
                .iter()
                .map(|p| format!("  - {}", p.join(path).display()))
                .collect();
            Err(ClmdError::resource_not_found(format!(
                "Resource '{}' not found. Searched in:\n{}\n  - {} (absolute path)",
                url,
                searched_paths.join("\n"),
                path.display()
            )))
        }
    }

    fn report(&self, msg: LogMessage) {
        // Convert LogLevel to Verbosity for comparison
        let msg_verbosity: Verbosity = match msg.level {
            LogLevel::Debug => Verbosity::Debug,
            LogLevel::Info => Verbosity::Info,
            LogLevel::Warning => Verbosity::Warning,
            LogLevel::Error => Verbosity::Error,
            LogLevel::Silent => Verbosity::Silent,
        };

        if self.verbosity >= msg_verbosity {
            if msg.level == LogLevel::Silent {
                return;
            }

            let prefix = match msg.level {
                LogLevel::Error => "ERROR",
                LogLevel::Warning => "WARNING",
                LogLevel::Info => "INFO",
                LogLevel::Debug => "DEBUG",
                _ => "",
            };

            if let (Some(line), Some(col)) = (msg.line, msg.column) {
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
    written_files: std::cell::RefCell<std::collections::HashMap<PathBuf, String>>,
    written_binary_files:
        std::cell::RefCell<std::collections::HashMap<PathBuf, Vec<u8>>>,
    resource_paths: Vec<PathBuf>,
    logs: std::cell::RefCell<Vec<LogMessage>>,
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
            written_files: std::cell::RefCell::new(std::collections::HashMap::new()),
            written_binary_files: std::cell::RefCell::new(
                std::collections::HashMap::new(),
            ),
            resource_paths: vec![PathBuf::from(".")],
            logs: std::cell::RefCell::new(Vec::new()),
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
            written_files: std::cell::RefCell::new(std::collections::HashMap::new()),
            written_binary_files: std::cell::RefCell::new(
                std::collections::HashMap::new(),
            ),
            resource_paths: vec![PathBuf::from(".")],
            logs: std::cell::RefCell::new(Vec::new()),
            timestamp: SystemTime::UNIX_EPOCH,
            current_dir: PathBuf::from("/test"),
            user_data_dir: Some(PathBuf::from("/test/.local/share/clmd")),
        }
    }

    /// Add a text file to the in-memory file system.
    pub fn with_file<S: Into<String>>(
        mut self,
        path: impl AsRef<Path>,
        content: S,
    ) -> Self {
        self.files
            .insert(path.as_ref().to_path_buf(), content.into());
        self
    }

    /// Add a binary file to the in-memory file system.
    pub fn with_binary_file(mut self, path: impl AsRef<Path>, content: Vec<u8>) -> Self {
        self.binary_files
            .insert(path.as_ref().to_path_buf(), content);
        self
    }

    /// Set the verbosity level.
    pub fn with_verbosity(mut self, verbosity: Verbosity) -> Self {
        self.verbosity = verbosity;
        self
    }
}

impl ClmdPure {
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
    pub fn get_logs(&self) -> Vec<LogMessage> {
        self.logs.borrow().clone()
    }

    /// Clear all logged messages.
    pub fn clear_logs(&self) {
        self.logs.borrow_mut().clear();
    }

    /// Check if a message was logged.
    pub fn has_logged(&self, level: Verbosity, message: &str) -> bool {
        let target_level: LogLevel = match level {
            Verbosity::Debug => LogLevel::Debug,
            Verbosity::Info => LogLevel::Info,
            Verbosity::Warning => LogLevel::Warning,
            Verbosity::Error => LogLevel::Error,
            Verbosity::Silent => LogLevel::Silent,
        };
        self.logs
            .borrow()
            .iter()
            .any(|log| log.level == target_level && log.message.contains(message))
    }

    /// Get a written file's content (for testing).
    pub fn get_written_file(&self, path: &Path) -> Option<String> {
        self.written_files.borrow().get(path).cloned()
    }

    /// Get a written binary file's content (for testing).
    pub fn get_written_binary_file(&self, path: &Path) -> Option<Vec<u8>> {
        self.written_binary_files.borrow().get(path).cloned()
    }

    /// Get paths of all written files.
    pub fn get_written_file_paths(&self) -> Vec<PathBuf> {
        self.written_files.borrow().keys().cloned().collect()
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

    fn write_file(&self, path: &Path, content: &str) -> Result<(), Self::Error> {
        // Store the written file for later verification
        self.written_files
            .borrow_mut()
            .insert(path.to_path_buf(), content.to_string());
        Ok(())
    }

    fn write_file_bytes(&self, path: &Path, content: &[u8]) -> Result<(), Self::Error> {
        // Store the written binary file for later verification
        self.written_binary_files
            .borrow_mut()
            .insert(path.to_path_buf(), content.to_vec());
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
        // Store the log message for later verification
        self.logs.borrow_mut().push(msg.clone());

        // Convert LogLevel to Verbosity for comparison
        let msg_verbosity: Verbosity = match msg.level {
            LogLevel::Debug => Verbosity::Debug,
            LogLevel::Info => Verbosity::Info,
            LogLevel::Warning => Verbosity::Warning,
            LogLevel::Error => Verbosity::Error,
            LogLevel::Silent => Verbosity::Silent,
        };

        // Also print if verbosity level allows
        if self.verbosity >= msg_verbosity {
            let prefix = match msg.level {
                LogLevel::Error => "ERROR",
                LogLevel::Warning => "WARNING",
                LogLevel::Info => "INFO",
                LogLevel::Debug => "DEBUG",
                _ => "",
            };

            if msg.level != LogLevel::Silent {
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
        assert_eq!(msg.level, LogLevel::Info);
        assert_eq!(msg.message, "Test message");
        assert!(msg.line.is_none());
        assert!(msg.column.is_none());

        let msg = LogMessage::error("Error").with_position(10, 5);
        assert_eq!(msg.line, Some(10));
        assert_eq!(msg.column, Some(5));
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
        assert_eq!(
            pure.get_current_dir().unwrap(),
            PathBuf::from("/custom/path")
        );
    }

    #[test]
    fn test_share_monad() {
        let io = ClmdIO::new();
        let shared = share_monad(io);

        assert_eq!(shared.get_verbosity(), Verbosity::Warning);
    }

    #[test]
    fn test_clmd_io_with_resource_paths() {
        let io = ClmdIO::new()
            .with_resource_paths(vec![PathBuf::from("/path1"), PathBuf::from("/path2")]);
        let paths = io.get_resource_paths();
        assert_eq!(paths.len(), 2);
        assert_eq!(paths[0], PathBuf::from("/path1"));
        assert_eq!(paths[1], PathBuf::from("/path2"));
    }

    #[test]
    fn test_clmd_io_with_user_data_dir() {
        let io = ClmdIO::new().with_user_data_dir(PathBuf::from("/custom/data"));
        assert_eq!(io.get_user_data_dir(), Some(PathBuf::from("/custom/data")));
    }

    #[test]
    fn test_clmd_io_file_operations() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        let io = ClmdIO::new();

        // Test write_file
        io.write_file(&file_path, "Hello, World!").unwrap();
        assert!(io.file_exists(&file_path));

        // Test read_file
        let content = io.read_file(&file_path).unwrap();
        assert_eq!(content, "Hello, World!");

        // Test read_file_bytes
        let bytes = io.read_file_bytes(&file_path).unwrap();
        assert_eq!(bytes, b"Hello, World!");

        // Test write_file_bytes
        let binary_path = temp_dir.path().join("binary.bin");
        io.write_file_bytes(&binary_path, &[0x00, 0x01, 0x02])
            .unwrap();
        let binary_content = io.read_file_bytes(&binary_path).unwrap();
        assert_eq!(binary_content, vec![0x00, 0x01, 0x02]);

        // Test file_exists for non-existent file
        assert!(!io.file_exists(&temp_dir.path().join("nonexistent.txt")));
    }

    #[test]
    fn test_clmd_io_fetch_resource_local() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("resource.txt");
        std::fs::write(&file_path, "resource content").unwrap();

        let io = ClmdIO::new().with_resource_paths(vec![temp_dir.path().to_path_buf()]);

        // Test fetching from resource path
        let content = io.fetch_resource("resource.txt").unwrap();
        assert_eq!(content, b"resource content");

        // Test fetching with absolute path
        let content = io.fetch_resource(file_path.to_str().unwrap()).unwrap();
        assert_eq!(content, b"resource content");
    }

    #[test]
    fn test_clmd_io_fetch_resource_not_found() {
        let io = ClmdIO::new();

        let result = io.fetch_resource("nonexistent_file_xyz.txt");
        assert!(result.is_err());
    }

    #[test]
    fn test_clmd_io_fetch_resource_http() {
        let io = ClmdIO::new();

        // HTTP fetching should return error (not enabled)
        let result = io.fetch_resource("http://example.com/test.txt");
        assert!(result.is_err());
    }

    #[test]
    fn test_clmd_io_get_current_dir() {
        let io = ClmdIO::new();
        let current_dir = io.get_current_dir().unwrap();
        assert!(current_dir.exists());
    }

    #[test]
    fn test_clmd_io_get_timestamp() {
        let io = ClmdIO::new();
        let timestamp = io.get_timestamp();
        // Just verify it returns a valid timestamp
        let _elapsed = timestamp.elapsed();
    }

    #[test]
    fn test_clmd_io_report() {
        let io = ClmdIO::with_verbosity(Verbosity::Debug);

        // These should not panic
        io.report(LogMessage::error("Test error"));
        io.report(LogMessage::warning("Test warning"));
        io.report(LogMessage::info("Test info"));
        io.report(LogMessage::debug("Test debug"));
    }

    #[test]
    fn test_clmd_io_log_methods() {
        let io = ClmdIO::with_verbosity(Verbosity::Debug);

        // These should not panic
        io.log_error("Error message");
        io.log_warning("Warning message");
        io.log_info("Info message");
        io.log_debug("Debug message");
    }

    #[test]
    fn test_clmd_io_with_sandbox() {
        use crate::core::sandbox::{SandboxMode, SandboxPolicy};

        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "test").unwrap();

        let sandbox = SandboxPolicy::new(SandboxMode::Strict)
            .allow_path(temp_dir.path().to_path_buf())
            .without_writes();

        let io = ClmdIO::new().with_sandbox(sandbox);

        // Should be able to read allowed path
        assert!(io.file_exists(&file_path));

        // Write should be blocked by sandbox
        let result = io.write_file(&temp_dir.path().join("new.txt"), "content");
        assert!(result.is_err());
    }

    #[test]
    fn test_clmd_pure_write_operations() {
        let pure = ClmdPure::new();

        // Test write_file
        pure.write_file(Path::new("/test/output.txt"), "written content")
            .unwrap();

        // Test get_written_file
        let content = pure.get_written_file(Path::new("/test/output.txt"));
        assert_eq!(content, Some("written content".to_string()));

        // Test get_written_file_paths
        let paths = pure.get_written_file_paths();
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], PathBuf::from("/test/output.txt"));

        // Test write_file_bytes
        pure.write_file_bytes(Path::new("/test/binary.bin"), &[0x00, 0x01, 0x02])
            .unwrap();
        let binary_content = pure.get_written_binary_file(Path::new("/test/binary.bin"));
        assert_eq!(binary_content, Some(vec![0x00, 0x01, 0x02]));
    }

    #[test]
    fn test_clmd_pure_log_operations() {
        let pure = ClmdPure::new().with_verbosity(Verbosity::Debug);

        // Test report
        pure.report(LogMessage::info("Test info"));
        pure.report(LogMessage::warning("Test warning"));
        pure.report(LogMessage::error("Test error"));

        // Test get_logs
        let logs = pure.get_logs();
        assert_eq!(logs.len(), 3);

        // Test has_logged
        assert!(pure.has_logged(Verbosity::Info, "Test info"));
        assert!(pure.has_logged(Verbosity::Warning, "warning"));
        assert!(pure.has_logged(Verbosity::Error, "error"));
        assert!(!pure.has_logged(Verbosity::Info, "nonexistent"));

        // Test clear_logs
        pure.clear_logs();
        let logs = pure.get_logs();
        assert!(logs.is_empty());
    }

    #[test]
    fn test_clmd_pure_fetch_resource() {
        let pure = ClmdPure::new().with_file("/test/resource.txt", "resource content");

        let content = pure.fetch_resource("/test/resource.txt").unwrap();
        assert_eq!(content, b"resource content");
    }

    #[test]
    fn test_clmd_pure_read_file_bytes_from_text() {
        let pure = ClmdPure::new().with_file("/test/file.txt", "Hello");

        // Should be able to read text file as bytes
        let bytes = pure.read_file_bytes(Path::new("/test/file.txt")).unwrap();
        assert_eq!(bytes, b"Hello");
    }

    #[test]
    fn test_clmd_pure_get_user_data_dir() {
        let pure = ClmdPure::new();
        assert_eq!(
            pure.get_user_data_dir(),
            Some(PathBuf::from("/test/.local/share/clmd"))
        );
    }

    #[test]
    fn test_clmd_pure_default() {
        let pure: ClmdPure = Default::default();
        assert_eq!(pure.get_verbosity(), Verbosity::Warning);
        assert_eq!(pure.get_timestamp(), SystemTime::UNIX_EPOCH);
        assert_eq!(pure.get_current_dir().unwrap(), PathBuf::from("/test"));
    }

    #[test]
    fn test_verbosity_default() {
        let default: Verbosity = Default::default();
        assert_eq!(default, Verbosity::Warning);
    }

    #[test]
    fn test_verbosity_is_enabled() {
        let io = ClmdIO::with_verbosity(Verbosity::Info);

        assert!(!io.is_verbosity_enabled(Verbosity::Debug));
        assert!(io.is_verbosity_enabled(Verbosity::Info));
        assert!(io.is_verbosity_enabled(Verbosity::Warning));
        assert!(io.is_verbosity_enabled(Verbosity::Error));
        assert!(io.is_verbosity_enabled(Verbosity::Silent));
    }
}
