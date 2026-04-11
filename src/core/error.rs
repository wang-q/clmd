//! Error types and parsing limits for the clmd Markdown parser.
//!
//! This module provides comprehensive error handling for parsing, rendering,
//! and document conversion operations, inspired by Pandoc's error system.

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
    /// Create a new position with the given line and column.
    pub fn new(line: usize, column: usize) -> Self {
        Self {
            line,
            column,
            offset: 0,
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

    /// Create a limit exceeded error.
    pub fn limit_exceeded(kind: LimitKind, limit: usize, actual: usize) -> Self {
        Self::LimitExceeded {
            kind,
            limit,
            actual,
        }
    }

    /// Create an IO error.
    pub fn io_error<S: Into<String>>(message: S) -> Self {
        Self::Io(message.into())
    }

    /// Create a feature not enabled error.
    pub fn feature_not_enabled<S: Into<String>>(feature: S) -> Self {
        Self::FeatureNotEnabled(feature.into())
    }

    /// Create a resource not found error.
    pub fn resource_not_found<S: Into<String>>(resource: S) -> Self {
        Self::ResourceNotFound(resource.into())
    }

    /// Create an input too large error.
    pub fn input_too_large(size: usize, max_size: usize) -> Self {
        Self::InputTooLarge { size, max_size }
    }

    /// Create an unknown reader error.
    pub fn unknown_reader<S: Into<String>>(format: S) -> Self {
        Self::UnknownReader(format.into())
    }

    /// Create an unknown writer error.
    pub fn unknown_writer<S: Into<String>>(format: S) -> Self {
        Self::UnknownWriter(format.into())
    }

    /// Create a transform error.
    pub fn transform_error<S: Into<String>>(message: S) -> Self {
        Self::Transform(message.into())
    }

    /// Create a config error.
    pub fn config_error<S: Into<String>>(message: S) -> Self {
        Self::Config(message.into())
    }

    /// Create a sandbox error.
    pub fn sandbox_error<S: Into<String>>(message: S) -> Self {
        Self::Sandbox(message.into())
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
    /// Maximum nesting depth.
    NestingDepth,
}

impl fmt::Display for LimitKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InputSize => write!(f, "input size"),
            Self::NestingDepth => write!(f, "nesting depth"),
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
}

impl Default for ParserLimits {
    fn default() -> Self {
        Self::new()
    }
}
