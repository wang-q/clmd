//! Logging system for clmd.
//!
//! This module provides a structured logging system for tracking operations,
//! warnings, and errors during document processing.

use std::fmt;
use std::sync::{Arc, Mutex};

/// Log level for messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LogLevel {
    /// Debug information - verbose details for debugging.
    Debug,
    /// General information - normal operation messages.
    Info,
    /// Warning - potential issues that don't prevent operation.
    Warning,
    /// Error - problems that prevented an operation from completing.
    Error,
}

impl LogLevel {
    /// Get the string representation of the log level.
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warning => "WARN",
            LogLevel::Error => "ERROR",
        }
    }

    /// Check if this level should be logged given a verbosity setting.
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

/// A single log entry.
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// The log level.
    pub level: LogLevel,
    /// The log message.
    pub message: String,
    /// Optional source/component that generated the log.
    pub source: Option<String>,
    /// Timestamp when the log was created.
    pub timestamp: std::time::Instant,
}

impl LogEntry {
    /// Create a new log entry.
    pub fn new(level: LogLevel, message: impl Into<String>) -> Self {
        Self {
            level,
            message: message.into(),
            source: None,
            timestamp: std::time::Instant::now(),
        }
    }
}

impl fmt::Display for LogEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.source {
            Some(source) => write!(f, "[{}] [{}] {}", self.level, source, self.message),
            None => write!(f, "[{}] {}", self.level, self.message),
        }
    }
}

/// A logger that collects log entries.
#[derive(Debug, Clone)]
pub struct Logger {
    entries: Arc<Mutex<Vec<LogEntry>>>,
    verbosity: Arc<Mutex<u8>>,
}

impl Logger {
    /// Create a new logger with default settings.
    pub fn new() -> Self {
        Self {
            entries: Arc::new(Mutex::new(Vec::new())),
            verbosity: Arc::new(Mutex::new(1)),
        }
    }

    /// Create a new logger with a specific verbosity level.
    pub fn with_verbosity(verbosity: u8) -> Self {
        Self {
            entries: Arc::new(Mutex::new(Vec::new())),
            verbosity: Arc::new(Mutex::new(verbosity)),
        }
    }

    /// Get the current verbosity level.
    pub fn verbosity(&self) -> u8 {
        *self.verbosity.lock().unwrap()
    }

    /// Set the verbosity level.
    pub fn set_verbosity(&self, verbosity: u8) {
        *self.verbosity.lock().unwrap() = verbosity;
    }

    /// Log a message at the specified level.
    pub fn log(&self, level: LogLevel, message: impl Into<String>) {
        if level.should_log(self.verbosity()) {
            let entry = LogEntry::new(level, message);
            self.entries.lock().unwrap().push(entry);
        }
    }

    /// Log a debug message.
    pub fn debug(&self, message: impl Into<String>) {
        self.log(LogLevel::Debug, message);
    }

    /// Log an info message.
    pub fn info(&self, message: impl Into<String>) {
        self.log(LogLevel::Info, message);
    }

    /// Log a warning message.
    pub fn warn(&self, message: impl Into<String>) {
        self.log(LogLevel::Warning, message);
    }

    /// Log an error message.
    pub fn error(&self, message: impl Into<String>) {
        self.log(LogLevel::Error, message);
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}
