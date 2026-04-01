//! Logging system for clmd.
//!
//! This module provides a structured logging system for tracking operations,
//! warnings, and errors during document processing. It integrates with the
//! Context system to provide consistent logging across the application.
//!
//! # Example
//!
//! ```ignore
//! use clmd::context::logging::{Logger, LogLevel, LogEntry};
//!
//! let logger = Logger::new();
//! logger.log(LogLevel::Info, "Processing document");
//! logger.warn("Deprecated feature used");
//!
//! for entry in logger.entries() {
//!     println!("[{}] {}", entry.level, entry.message);
//! }
//! ```

use std::fmt;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

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
    pub timestamp: Instant,
}

impl LogEntry {
    /// Create a new log entry.
    pub fn new(level: LogLevel, message: impl Into<String>) -> Self {
        Self {
            level,
            message: message.into(),
            source: None,
            timestamp: Instant::now(),
        }
    }

    /// Create a new log entry with a source.
    pub fn with_source(
        level: LogLevel,
        message: impl Into<String>,
        source: impl Into<String>,
    ) -> Self {
        Self {
            level,
            message: message.into(),
            source: Some(source.into()),
            timestamp: Instant::now(),
        }
    }

    /// Get the age of this log entry.
    pub fn age(&self) -> Duration {
        self.timestamp.elapsed()
    }

    /// Format the log entry as a string.
    pub fn format(&self) -> String {
        match &self.source {
            Some(source) => format!("[{}] [{}] {}", self.level, source, self.message),
            None => format!("[{}] {}", self.level, self.message),
        }
    }
}

impl fmt::Display for LogEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format())
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

    /// Log a message with a source.
    pub fn log_with_source(
        &self,
        level: LogLevel,
        message: impl Into<String>,
        source: impl Into<String>,
    ) {
        if level.should_log(self.verbosity()) {
            let entry = LogEntry::with_source(level, message, source);
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

    /// Get all log entries.
    pub fn entries(&self) -> Vec<LogEntry> {
        self.entries.lock().unwrap().clone()
    }

    /// Get entries filtered by level.
    pub fn entries_at_level(&self, level: LogLevel) -> Vec<LogEntry> {
        self.entries
            .lock()
            .unwrap()
            .iter()
            .filter(|e| e.level == level)
            .cloned()
            .collect()
    }

    /// Get entries with level >= the specified level.
    pub fn entries_at_least(&self, level: LogLevel) -> Vec<LogEntry> {
        self.entries
            .lock()
            .unwrap()
            .iter()
            .filter(|e| e.level >= level)
            .cloned()
            .collect()
    }

    /// Check if there are any error entries.
    pub fn has_errors(&self) -> bool {
        self.entries
            .lock()
            .unwrap()
            .iter()
            .any(|e| e.level == LogLevel::Error)
    }

    /// Check if there are any warning entries.
    pub fn has_warnings(&self) -> bool {
        self.entries
            .lock()
            .unwrap()
            .iter()
            .any(|e| e.level == LogLevel::Warning)
    }

    /// Get the number of log entries.
    pub fn len(&self) -> usize {
        self.entries.lock().unwrap().len()
    }

    /// Check if the logger is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.lock().unwrap().is_empty()
    }

    /// Clear all log entries.
    pub fn clear(&self) {
        self.entries.lock().unwrap().clear();
    }

    /// Format all entries as a string.
    pub fn format_all(&self) -> String {
        self.entries
            .lock()
            .unwrap()
            .iter()
            .map(|e| e.format())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}

/// A progress tracker for long-running operations.
#[derive(Debug)]
pub struct ProgressTracker {
    operation: String,
    total: Option<usize>,
    current: Arc<Mutex<usize>>,
    logger: Logger,
}

impl ProgressTracker {
    /// Create a new progress tracker.
    pub fn new(operation: impl Into<String>, total: Option<usize>) -> Self {
        Self {
            operation: operation.into(),
            total,
            current: Arc::new(Mutex::new(0)),
            logger: Logger::new(),
        }
    }

    /// Get the current progress count.
    pub fn current(&self) -> usize {
        *self.current.lock().unwrap()
    }

    /// Get the total count (if known).
    pub fn total(&self) -> Option<usize> {
        self.total
    }

    /// Get the progress percentage (if total is known).
    pub fn percentage(&self) -> Option<f64> {
        self.total.map(|t| {
            let c = self.current() as f64;
            let t = t as f64;
            (c / t) * 100.0
        })
    }

    /// Increment the progress counter.
    pub fn increment(&self) {
        *self.current.lock().unwrap() += 1;
    }

    /// Increment by a specific amount.
    pub fn increment_by(&self, amount: usize) {
        *self.current.lock().unwrap() += amount;
    }

    /// Log progress at the info level.
    pub fn log_progress(&self) {
        match self.total {
            Some(total) => {
                let current = self.current();
                let pct = (current as f64 / total as f64) * 100.0;
                self.logger.info(format!(
                    "{}: {}/{} ({:.1}%)",
                    self.operation, current, total, pct
                ));
            }
            None => {
                self.logger
                    .info(format!("{}: {}", self.operation, self.current()));
            }
        }
    }

    /// Get the logger for this tracker.
    pub fn logger(&self) -> &Logger {
        &self.logger
    }
}

/// Statistics collector for operations.
#[derive(Debug, Default, Clone, Copy)]
pub struct Stats {
    /// Number of files processed.
    pub files_processed: usize,
    /// Number of files skipped.
    pub files_skipped: usize,
    /// Number of errors encountered.
    pub errors: usize,
    /// Number of warnings generated.
    pub warnings: usize,
    /// Processing time.
    pub duration: Option<Duration>,
}

impl Stats {
    /// Create new empty stats.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a processed file.
    pub fn file_processed(&mut self) {
        self.files_processed += 1;
    }

    /// Record a skipped file.
    pub fn file_skipped(&mut self) {
        self.files_skipped += 1;
    }

    /// Record an error.
    pub fn error(&mut self) {
        self.errors += 1;
    }

    /// Record a warning.
    pub fn warning(&mut self) {
        self.warnings += 1;
    }

    /// Set the duration.
    pub fn set_duration(&mut self, duration: Duration) {
        self.duration = Some(duration);
    }

    /// Get the total number of files (processed + skipped).
    pub fn total_files(&self) -> usize {
        self.files_processed + self.files_skipped
    }

    /// Format stats as a string.
    pub fn format(&self) -> String {
        let mut parts = vec![
            format!("Files processed: {}", self.files_processed),
            format!("Files skipped: {}", self.files_skipped),
            format!("Errors: {}", self.errors),
            format!("Warnings: {}", self.warnings),
        ];

        if let Some(duration) = self.duration {
            parts.push(format!("Duration: {:.2}s", duration.as_secs_f64()));
        }

        parts.join("\n")
    }
}

impl fmt::Display for Stats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_as_str() {
        assert_eq!(LogLevel::Debug.as_str(), "DEBUG");
        assert_eq!(LogLevel::Info.as_str(), "INFO");
        assert_eq!(LogLevel::Warning.as_str(), "WARN");
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
    fn test_log_entry() {
        let entry = LogEntry::new(LogLevel::Info, "Test message");
        assert_eq!(entry.level, LogLevel::Info);
        assert_eq!(entry.message, "Test message");
        assert!(entry.source.is_none());

        let entry_with_source =
            LogEntry::with_source(LogLevel::Warning, "Warning", "parser");
        assert_eq!(entry_with_source.source, Some("parser".to_string()));
    }

    #[test]
    fn test_logger() {
        let logger = Logger::new();

        logger.info("Info message");
        logger.warn("Warning message");
        logger.error("Error message");

        assert_eq!(logger.len(), 3);
        assert!(logger.has_warnings());
        assert!(logger.has_errors());

        let errors = logger.entries_at_level(LogLevel::Error);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_logger_verbosity() {
        let logger = Logger::with_verbosity(0); // Quiet mode

        logger.debug("Debug");
        logger.info("Info");
        logger.warn("Warning");
        logger.error("Error");

        // Only errors should be logged in quiet mode
        assert_eq!(logger.len(), 1);
        assert!(logger.has_errors());
        assert!(!logger.has_warnings());
    }

    #[test]
    fn test_logger_clear() {
        let logger = Logger::new();
        logger.info("Message");
        assert!(!logger.is_empty());

        logger.clear();
        assert!(logger.is_empty());
    }

    #[test]
    fn test_progress_tracker() {
        let tracker = ProgressTracker::new("Processing", Some(100));

        assert_eq!(tracker.current(), 0);
        assert_eq!(tracker.total(), Some(100));

        tracker.increment();
        assert_eq!(tracker.current(), 1);

        tracker.increment_by(9);
        assert_eq!(tracker.current(), 10);

        let pct = tracker.percentage();
        assert!(pct.is_some());
        assert_eq!(pct.unwrap(), 10.0);
    }

    #[test]
    fn test_progress_tracker_unknown_total() {
        let tracker = ProgressTracker::new("Processing", None);
        assert_eq!(tracker.total(), None);
        assert!(tracker.percentage().is_none());
    }

    #[test]
    fn test_stats() {
        let mut stats = Stats::new();

        stats.file_processed();
        stats.file_processed();
        stats.file_skipped();
        stats.error();
        stats.warning();
        stats.warning();

        assert_eq!(stats.files_processed, 2);
        assert_eq!(stats.files_skipped, 1);
        assert_eq!(stats.total_files(), 3);
        assert_eq!(stats.errors, 1);
        assert_eq!(stats.warnings, 2);

        stats.set_duration(Duration::from_secs(5));
        assert!(stats.duration.is_some());

        let formatted = stats.format();
        assert!(formatted.contains("Files processed: 2"));
        assert!(formatted.contains("Duration: 5.00s"));
    }
}
