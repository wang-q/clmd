//! Error types and parsing limits for the clmd Markdown parser.
//!
//! This module provides comprehensive error handling for parsing, rendering,
//! and document conversion operations, inspired by Pandoc's error system.
//!
//! # Example
//!
//! ```ignore
//! use clmd::core::error::{ClmdError, Position};
//!
//! let error = ClmdError::parse_error(Position::new(1, 10), "Unexpected token");
//! println!("Error: {}", error);
//! ```

use std::fmt;
use std::io;
use thiserror::Error;

/// Position in source text (line, column, offset).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    /// Line number (1-based).
    pub line: usize,
    /// Column number (1-based).
    pub column: usize,
    /// Byte offset in the source.
    pub offset: usize,
}

impl Position {
    /// Create a new position.
    pub fn new(line: usize, column: usize) -> Self {
        Self {
            line,
            column,
            offset: 0,
        }
    }

    /// Create a new position with offset.
    pub fn with_offset(line: usize, column: usize, offset: usize) -> Self {
        Self {
            line,
            column,
            offset,
        }
    }

    /// Create a position at the start of the document.
    pub fn start() -> Self {
        Self {
            line: 1,
            column: 1,
            offset: 0,
        }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

/// Main error type for clmd operations.
///
/// This enum covers all possible errors that can occur during parsing,
/// rendering, and document conversion.
#[derive(Error, Debug, Clone)]
pub enum ClmdError {
    /// IO error during file operations.
    #[error("IO error: {0}")]
    Io(String),

    /// Parse error with position information.
    #[error("Parse error at {position}: {message}")]
    Parse {
        /// Position where the error occurred.
        position: Position,
        /// Error message.
        message: String,
    },

    /// Unknown format.
    #[error("Unknown format: {0}")]
    UnknownFormat(String),

    /// Unknown reader format.
    #[error("Unknown reader format: {0}")]
    UnknownReader(String),

    /// Unknown writer format.
    #[error("Unknown writer format: {0}")]
    UnknownWriter(String),

    /// Unsupported extension for a format.
    #[error("Extension '{extension}' is not supported for format '{format}'")]
    UnsupportedExtension {
        /// Extension name.
        extension: String,
        /// Format name.
        format: String,
    },

    /// Document transformation error.
    #[error("Transform error: {0}")]
    Transform(String),

    /// Validation error.
    #[error("Validation error: {0}")]
    Validation(String),

    /// Resource not found.
    #[error("Resource not found: {0}")]
    ResourceNotFound(String),

    /// Template error.
    #[error("Template error: {0}")]
    Template(String),

    /// Filter error.
    #[error("Filter error: {0}")]
    Filter(String),

    /// Configuration error.
    #[error("Configuration error: {0}")]
    Config(String),

    /// Encoding error.
    #[error("Encoding error: {0}")]
    Encoding(String),

    /// Feature not enabled.
    #[error(
        "Feature '{0}' is not enabled. Enable it with the corresponding feature flag."
    )]
    FeatureNotEnabled(String),

    /// Parser limit exceeded.
    #[error("Parser limit exceeded: {kind} (limit: {limit}, actual: {actual})")]
    LimitExceeded {
        /// Kind of limit exceeded.
        kind: LimitKind,
        /// Limit value.
        limit: usize,
        /// Actual value.
        actual: usize,
    },

    /// Generic error with message.
    #[error("{0}")]
    Other(String),

    /// Circular reference error.
    #[error("Circular reference detected: {0}")]
    CircularReference(String),

    /// Invalid argument error.
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    /// Unsupported operation error.
    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),

    /// Timeout error.
    #[error("Operation timed out: {0}")]
    Timeout(String),

    /// Network error.
    #[error("Network error: {0}")]
    Network(String),

    /// Permission denied error.
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Already exists error.
    #[error("Resource already exists: {0}")]
    AlreadyExists(String),

    /// Not supported error.
    #[error("Not supported: {0}")]
    NotSupported(String),

    /// Warning (non-fatal error).
    #[error("Warning: {0}")]
    Warning(String),

    /// Sandbox security error.
    #[error("Sandbox error: {0}")]
    Sandbox(String),

    /// Input too large error.
    #[error("Input too large: {size} bytes (max: {max_size})")]
    InputTooLarge {
        /// Actual input size.
        size: usize,
        /// Maximum allowed size.
        max_size: usize,
    },
}

impl ClmdError {
    /// Create a parse error at a specific position.
    pub fn parse_error<P: Into<Position>, S: Into<String>>(
        position: P,
        message: S,
    ) -> Self {
        Self::Parse {
            position: position.into(),
            message: message.into(),
        }
    }

    /// Create an IO error.
    pub fn io_error<S: Into<String>>(message: S) -> Self {
        Self::Io(message.into())
    }

    /// Create a generic error.
    pub fn other<S: Into<String>>(message: S) -> Self {
        Self::Other(message.into())
    }

    /// Create an unknown format error.
    pub fn unknown_format<S: Into<String>>(format: S) -> Self {
        Self::UnknownFormat(format.into())
    }

    /// Create an unknown reader error.
    pub fn unknown_reader<S: Into<String>>(format: S) -> Self {
        Self::UnknownReader(format.into())
    }

    /// Create an unknown writer error.
    pub fn unknown_writer<S: Into<String>>(format: S) -> Self {
        Self::UnknownWriter(format.into())
    }

    /// Create an input too large error.
    pub fn input_too_large(size: usize, max_size: usize) -> Self {
        Self::InputTooLarge { size, max_size }
    }

    /// Create an unsupported extension error.
    pub fn unsupported_extension<E: Into<String>, F: Into<String>>(
        extension: E,
        format: F,
    ) -> Self {
        Self::UnsupportedExtension {
            extension: extension.into(),
            format: format.into(),
        }
    }

    /// Create a transform error.
    pub fn transform_error<S: Into<String>>(message: S) -> Self {
        Self::Transform(message.into())
    }

    /// Create a validation error.
    pub fn validation_error<S: Into<String>>(message: S) -> Self {
        Self::Validation(message.into())
    }

    /// Create a resource not found error.
    pub fn resource_not_found<S: Into<String>>(resource: S) -> Self {
        Self::ResourceNotFound(resource.into())
    }

    /// Create a template error.
    pub fn template_error<S: Into<String>>(message: S) -> Self {
        Self::Template(message.into())
    }

    /// Create a filter error.
    pub fn filter_error<S: Into<String>>(message: S) -> Self {
        Self::Filter(message.into())
    }

    /// Create a config error.
    pub fn config_error<S: Into<String>>(message: S) -> Self {
        Self::Config(message.into())
    }

    /// Create an encoding error.
    pub fn encoding_error<S: Into<String>>(message: S) -> Self {
        Self::Encoding(message.into())
    }

    /// Create a feature not enabled error.
    pub fn feature_not_enabled<S: Into<String>>(feature: S) -> Self {
        Self::FeatureNotEnabled(feature.into())
    }

    /// Create a not implemented error.
    pub fn not_implemented<S: Into<String>>(feature: S) -> Self {
        Self::Other(format!("Not implemented: {}", feature.into()))
    }

    /// Create a limit exceeded error.
    pub fn limit_exceeded(kind: LimitKind, limit: usize, actual: usize) -> Self {
        Self::LimitExceeded {
            kind,
            limit,
            actual,
        }
    }

    /// Create a circular reference error.
    pub fn circular_reference<S: Into<String>>(message: S) -> Self {
        Self::CircularReference(message.into())
    }

    /// Create an invalid argument error.
    pub fn invalid_argument<S: Into<String>>(message: S) -> Self {
        Self::InvalidArgument(message.into())
    }

    /// Create an unsupported operation error.
    pub fn unsupported_operation<S: Into<String>>(message: S) -> Self {
        Self::UnsupportedOperation(message.into())
    }

    /// Create a timeout error.
    pub fn timeout<S: Into<String>>(message: S) -> Self {
        Self::Timeout(message.into())
    }

    /// Create a network error.
    pub fn network<S: Into<String>>(message: S) -> Self {
        Self::Network(message.into())
    }

    /// Create a permission denied error.
    pub fn permission_denied<S: Into<String>>(resource: S) -> Self {
        Self::PermissionDenied(resource.into())
    }

    /// Create an already exists error.
    pub fn already_exists<S: Into<String>>(resource: S) -> Self {
        Self::AlreadyExists(resource.into())
    }

    /// Create a not supported error.
    pub fn not_supported<S: Into<String>>(feature: S) -> Self {
        Self::NotSupported(feature.into())
    }

    /// Create a warning.
    pub fn warning<S: Into<String>>(message: S) -> Self {
        Self::Warning(message.into())
    }

    /// Create a sandbox error.
    pub fn sandbox_error<S: Into<String>>(message: S) -> Self {
        Self::Sandbox(message.into())
    }

    /// Get the position of the error, if available.
    pub fn position(&self) -> Option<Position> {
        match self {
            Self::Parse { position, .. } => Some(*position),
            _ => None,
        }
    }

    /// Check if this is a parse error.
    pub fn is_parse_error(&self) -> bool {
        matches!(self, Self::Parse { .. })
    }

    /// Check if this is an IO error.
    pub fn is_io_error(&self) -> bool {
        matches!(self, Self::Io(_))
    }

    /// Check if this is a limit exceeded error.
    pub fn is_limit_exceeded(&self) -> bool {
        matches!(
            self,
            Self::LimitExceeded { .. } | Self::InputTooLarge { .. }
        )
    }

    /// Check if this is a circular reference error.
    pub fn is_circular_reference(&self) -> bool {
        matches!(self, Self::CircularReference(_))
    }

    /// Check if this is an invalid argument error.
    pub fn is_invalid_argument(&self) -> bool {
        matches!(self, Self::InvalidArgument(_))
    }

    /// Check if this is an unsupported operation error.
    pub fn is_unsupported_operation(&self) -> bool {
        matches!(self, Self::UnsupportedOperation(_))
    }

    /// Check if this is a timeout error.
    pub fn is_timeout(&self) -> bool {
        matches!(self, Self::Timeout(_))
    }

    /// Check if this is a network error.
    pub fn is_network(&self) -> bool {
        matches!(self, Self::Network(_))
    }

    /// Check if this is a permission denied error.
    pub fn is_permission_denied(&self) -> bool {
        matches!(self, Self::PermissionDenied(_))
    }

    /// Check if this is an already exists error.
    pub fn is_already_exists(&self) -> bool {
        matches!(self, Self::AlreadyExists(_))
    }

    /// Check if this is a not supported error.
    pub fn is_not_supported(&self) -> bool {
        matches!(self, Self::NotSupported(_))
    }

    /// Check if this is a warning.
    pub fn is_warning(&self) -> bool {
        matches!(self, Self::Warning(_))
    }

    /// Check if this error is fatal (not a warning).
    pub fn is_fatal(&self) -> bool {
        !self.is_warning()
    }

    /// Get the exit code for this error.
    ///
    /// These exit codes are compatible with Pandoc's exit codes where applicable.
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Io(_) => 1,
            Self::Parse { .. } => 64,
            Self::UnknownFormat(_) => 20,
            Self::UnknownReader(_) => 21,
            Self::UnknownWriter(_) => 22,
            Self::UnsupportedExtension { .. } => 23,
            Self::Transform(_) => 83,
            Self::Validation(_) => 97,
            Self::ResourceNotFound(_) => 99,
            Self::Template(_) => 5,
            Self::Filter(_) => 83,
            Self::Config(_) => 6,
            Self::Encoding(_) => 92,
            Self::FeatureNotEnabled(_) => 89,
            Self::LimitExceeded { .. } => 93,
            Self::CircularReference(_) => 67,
            Self::InvalidArgument(_) => 3,
            Self::UnsupportedOperation(_) => 84,
            Self::Timeout(_) => 98,
            Self::Network(_) => 91,
            Self::PermissionDenied(_) => 77,
            Self::AlreadyExists(_) => 75,
            Self::NotSupported(_) => 85,
            Self::Warning(_) => 0, // Warnings don't cause non-zero exit
            Self::Sandbox(_) => 77, // Permission denied
            Self::Other(_) => 1,
            Self::InputTooLarge { .. } => 93, // Same as LimitExceeded
        }
    }
}

impl From<io::Error> for ClmdError {
    fn from(err: io::Error) -> Self {
        Self::Io(err.to_string())
    }
}

impl From<std::fmt::Error> for ClmdError {
    fn from(err: std::fmt::Error) -> Self {
        Self::Io(err.to_string())
    }
}

/// Kind of parser limit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LimitKind {
    /// Maximum input size.
    InputSize,
    /// Maximum line length.
    LineLength,
    /// Maximum nesting depth.
    NestingDepth,
    /// Maximum number of list items.
    ListItems,
    /// Maximum number of links.
    Links,
    /// Maximum number of emphasis markers.
    Emphasis,
    /// Maximum table cells.
    TableCells,
    /// Maximum table rows.
    TableRows,
}

impl fmt::Display for LimitKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InputSize => write!(f, "input size"),
            Self::LineLength => write!(f, "line length"),
            Self::NestingDepth => write!(f, "nesting depth"),
            Self::ListItems => write!(f, "list items"),
            Self::Links => write!(f, "links"),
            Self::Emphasis => write!(f, "emphasis"),
            Self::TableCells => write!(f, "table cells"),
            Self::TableRows => write!(f, "table rows"),
        }
    }
}

/// Result type for clmd operations.
pub type ClmdResult<T> = Result<T, ClmdError>;

/// Log level for messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum LogLevel {
    /// Debug - detailed debug information.
    Debug,
    /// Info - normal informational output.
    Info,
    /// Warning - errors and warnings.
    #[default]
    Warning,
    /// Error - only errors.
    Error,
    /// Silent - no output.
    Silent,
}

/// A log message with metadata.
#[derive(Debug, Clone)]
pub struct LogMessage {
    /// The severity level of this message.
    pub level: LogLevel,
    /// The message content.
    pub message: String,
    /// Source file (if applicable).
    pub source: Option<String>,
    /// Line number (if applicable).
    pub line: Option<usize>,
    /// Column number (if applicable).
    pub column: Option<usize>,
}

impl LogMessage {
    /// Create a new log message.
    pub fn new<S: Into<String>>(level: LogLevel, message: S) -> Self {
        Self {
            level,
            message: message.into(),
            source: None,
            line: None,
            column: None,
        }
    }

    /// Create an error message.
    pub fn error<S: Into<String>>(message: S) -> Self {
        Self::new(LogLevel::Error, message)
    }

    /// Create a warning message.
    pub fn warning<S: Into<String>>(message: S) -> Self {
        Self::new(LogLevel::Warning, message)
    }

    /// Create an info message.
    pub fn info<S: Into<String>>(message: S) -> Self {
        Self::new(LogLevel::Info, message)
    }

    /// Create a debug message.
    pub fn debug<S: Into<String>>(message: S) -> Self {
        Self::new(LogLevel::Debug, message)
    }

    /// Set the source file.
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Set the position.
    pub fn with_position(mut self, line: usize, column: usize) -> Self {
        self.line = Some(line);
        self.column = Some(column);
        self
    }
}

/// Parser limits for security and resource control.
///
/// These limits help prevent denial-of-service attacks and excessive
/// resource consumption when parsing untrusted input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParserLimits {
    /// Maximum input size in bytes (0 = unlimited).
    pub max_input_size: usize,
    /// Maximum line length in bytes (0 = unlimited).
    pub max_line_length: usize,
    /// Maximum nesting depth for block elements (0 = unlimited).
    pub max_nesting_depth: usize,
    /// Maximum number of list items (0 = unlimited).
    pub max_list_items: usize,
    /// Maximum number of links (0 = unlimited).
    pub max_links: usize,
    /// Maximum number of emphasis markers (0 = unlimited).
    pub max_emphasis: usize,
    /// Maximum table cells (0 = unlimited).
    pub max_table_cells: usize,
    /// Maximum table rows (0 = unlimited).
    pub max_table_rows: usize,
}

impl ParserLimits {
    /// Create a new limits configuration with default values.
    pub const fn new() -> Self {
        Self {
            max_input_size: 0,
            max_line_length: 0,
            max_nesting_depth: 0,
            max_list_items: 0,
            max_links: 0,
            max_emphasis: 0,
            max_table_cells: 0,
            max_table_rows: 0,
        }
    }

    /// Set maximum input size.
    pub fn with_max_input_size(mut self, size: usize) -> Self {
        self.max_input_size = size;
        self
    }

    /// Set maximum line length.
    pub fn with_max_line_length(mut self, length: usize) -> Self {
        self.max_line_length = length;
        self
    }

    /// Set maximum nesting depth.
    pub fn with_max_nesting_depth(mut self, depth: usize) -> Self {
        self.max_nesting_depth = depth;
        self
    }
}

impl Default for ParserLimits {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position() {
        let pos = Position::new(10, 5);
        assert_eq!(pos.line, 10);
        assert_eq!(pos.column, 5);
        assert_eq!(pos.to_string(), "10:5");
    }

    #[test]
    fn test_clmd_error_parse() {
        let err = ClmdError::parse_error(Position::new(5, 10), "Unexpected token");
        assert!(err.is_parse_error());
        assert_eq!(err.position(), Some(Position::new(5, 10)));
        assert!(err.to_string().contains("Parse error at 5:10"));
    }

    #[test]
    fn test_clmd_error_io() {
        let err = ClmdError::io_error("File not found");
        assert!(err.is_io_error());
        assert!(err.to_string().contains("IO error"));
    }

    #[test]
    fn test_clmd_error_unknown_reader() {
        let err = ClmdError::unknown_reader("custom-format");
        assert!(err
            .to_string()
            .contains("Unknown reader format: custom-format"));
    }

    #[test]
    fn test_clmd_error_unknown_writer() {
        let err = ClmdError::unknown_writer("pdf");
        assert!(err.to_string().contains("Unknown writer format: pdf"));
    }

    #[test]
    fn test_clmd_error_unsupported_extension() {
        let err = ClmdError::unsupported_extension("table", "plain");
        assert!(err
            .to_string()
            .contains("Extension 'table' is not supported for format 'plain'"));
    }

    #[test]
    fn test_clmd_error_limit_exceeded() {
        let err = ClmdError::limit_exceeded(LimitKind::InputSize, 1024, 2048);
        assert!(err.is_limit_exceeded());
        assert!(err
            .to_string()
            .contains("Parser limit exceeded: input size"));
    }

    #[test]
    fn test_parser_limits_default() {
        let limits = ParserLimits::default();
        assert_eq!(limits.max_input_size, 0);
        assert_eq!(limits.max_line_length, 0);
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let clmd_err: ClmdError = io_err.into();
        assert!(matches!(clmd_err, ClmdError::Io(_)));
    }

    #[test]
    fn test_new_error_types() {
        let err = ClmdError::circular_reference("link A -> B -> A");
        assert!(err.is_circular_reference());
        assert!(err.to_string().contains("Circular reference"));

        let err = ClmdError::invalid_argument("negative value");
        assert!(err.is_invalid_argument());
        assert!(err.to_string().contains("Invalid argument"));

        let err = ClmdError::timeout("operation took too long");
        assert!(err.is_timeout());
        assert!(err.to_string().contains("timed out"));

        let err = ClmdError::permission_denied("/etc/passwd");
        assert!(err.is_permission_denied());
        assert!(err.to_string().contains("Permission denied"));

        let err = ClmdError::warning("this is a warning");
        assert!(err.is_warning());
        assert!(!err.is_fatal());
    }

    #[test]
    fn test_error_exit_codes() {
        assert_eq!(ClmdError::io_error("test").exit_code(), 1);
        assert_eq!(
            ClmdError::parse_error(Position::new(1, 1), "test").exit_code(),
            64
        );
        assert_eq!(ClmdError::unknown_reader("fmt").exit_code(), 21);
        assert_eq!(ClmdError::unknown_writer("fmt").exit_code(), 22);
        assert_eq!(ClmdError::circular_reference("test").exit_code(), 67);
        assert_eq!(ClmdError::timeout("test").exit_code(), 98);
        assert_eq!(ClmdError::warning("test").exit_code(), 0);
    }
}
