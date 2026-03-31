//! Error types and parsing limits for the clmd Markdown parser.
//!
//! This module provides comprehensive error handling for parsing, rendering,
//! and document conversion operations, inspired by Pandoc's error system.
//!
//! # Example
//!
//! ```
//! use clmd::error::{ClmdError, Position};
//!
//! let error = ClmdError::parse_error(Position::new(1, 10), "Unexpected token");
//! println!("Error: {}", error);
//! ```

use std::fmt;
use std::io;
use thiserror::Error;

/// Position in source text (line, column).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    /// Line number (1-based).
    pub line: usize,
    /// Column number (1-based).
    pub column: usize,
}

impl Position {
    /// Create a new position.
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }

    /// Create a position at the start of the document.
    pub fn start() -> Self {
        Self { line: 1, column: 1 }
    }

    /// Create a position from a byte offset in the source.
    pub fn from_offset(source: &str, offset: usize) -> Self {
        let mut line = 1;
        let mut column = 1;

        for (i, c) in source.char_indices() {
            if i >= offset {
                break;
            }
            if c == '\n' {
                line += 1;
                column = 1;
            } else {
                column += 1;
            }
        }

        Self { line, column }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

/// Range in source text (start, end).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Range {
    /// Start position.
    pub start: Position,
    /// End position.
    pub end: Position,
}

impl Range {
    /// Create a new range.
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    /// Create a range from byte offsets.
    pub fn from_offsets(source: &str, start_offset: usize, end_offset: usize) -> Self {
        Self {
            start: Position::from_offset(source, start_offset),
            end: Position::from_offset(source, end_offset),
        }
    }
}

impl fmt::Display for Range {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.start == self.end {
            write!(f, "{}", self.start)
        } else {
            write!(f, "{}-{}", self.start, self.end)
        }
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

    /// Get the exit code for this error.
    ///
    /// These exit codes are compatible with Pandoc's exit codes where applicable.
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Io(_) => 1,
            Self::Parse { .. } => 64,
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
            Self::Other(_) => 1,
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

    /// Create an unknown reader error.
    pub fn unknown_reader<S: Into<String>>(format: S) -> Self {
        Self::UnknownReader(format.into())
    }

    /// Create an unknown writer error.
    pub fn unknown_writer<S: Into<String>>(format: S) -> Self {
        Self::UnknownWriter(format.into())
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
        matches!(self, Self::LimitExceeded { .. })
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

/// Legacy error type for backward compatibility.
#[derive(Error, Debug, Clone)]
pub enum ParseError {
    /// Parse error with position.
    #[error("Parse error at {position}: {message}")]
    ParseError {
        /// Position where the error occurred.
        position: Position,
        /// Error message.
        message: String,
    },

    /// IO error.
    #[error("IO error: {0}")]
    IoError(String),

    /// Limit exceeded.
    #[error("Limit exceeded: {kind} (limit: {limit}, actual: {actual})")]
    LimitExceeded {
        /// Kind of limit.
        kind: LimitKind,
        /// Limit value.
        limit: usize,
        /// Actual value.
        actual: usize,
    },

    /// Input too large.
    #[error("Input too large: {size} bytes (max: {max_size})")]
    InputTooLarge {
        /// Actual input size.
        size: usize,
        /// Maximum allowed size.
        max_size: usize,
    },
}

impl ParseError {
    /// Create a new parse error.
    pub fn new<S: Into<String>>(position: Position, message: S) -> Self {
        Self::ParseError {
            position,
            message: message.into(),
        }
    }
}

impl From<ClmdError> for ParseError {
    fn from(err: ClmdError) -> Self {
        match err {
            ClmdError::Parse { position, message } => {
                Self::ParseError { position, message }
            }
            ClmdError::Io(msg) => Self::IoError(msg),
            ClmdError::LimitExceeded {
                kind,
                limit,
                actual,
            } => Self::LimitExceeded {
                kind,
                limit,
                actual,
            },
            _ => Self::IoError(err.to_string()),
        }
    }
}

/// Legacy result type for backward compatibility.
pub type ParseResult<T> = Result<T, ParseError>;

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

    /// Create limits with conservative defaults.
    pub const fn conservative() -> Self {
        Self {
            max_input_size: 10 * 1024 * 1024, // 10MB
            max_line_length: 100_000,         // 100KB per line
            max_nesting_depth: 100,
            max_list_items: 50_000,
            max_links: 50_000,
            max_emphasis: 50_000,
            max_table_cells: 100_000,
            max_table_rows: 10_000,
        }
    }

    /// Create limits with strict defaults.
    pub const fn strict() -> Self {
        Self {
            max_input_size: 1024 * 1024, // 1MB
            max_line_length: 10_000,     // 10KB per line
            max_nesting_depth: 50,
            max_list_items: 10_000,
            max_links: 10_000,
            max_emphasis: 10_000,
            max_table_cells: 10_000,
            max_table_rows: 1_000,
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

    /// Check if input size is within limits.
    pub fn check_input_size(&self, size: usize) -> ClmdResult<()> {
        if self.max_input_size > 0 && size > self.max_input_size {
            Err(ClmdError::limit_exceeded(
                LimitKind::InputSize,
                self.max_input_size,
                size,
            ))
        } else {
            Ok(())
        }
    }

    /// Check if line length is within limits.
    pub fn check_line_length(&self, length: usize) -> ClmdResult<()> {
        if self.max_line_length > 0 && length > self.max_line_length {
            Err(ClmdError::limit_exceeded(
                LimitKind::LineLength,
                self.max_line_length,
                length,
            ))
        } else {
            Ok(())
        }
    }

    /// Check if nesting depth is within limits.
    pub fn check_nesting_depth(&self, depth: usize) -> ClmdResult<()> {
        if self.max_nesting_depth > 0 && depth > self.max_nesting_depth {
            Err(ClmdError::limit_exceeded(
                LimitKind::NestingDepth,
                self.max_nesting_depth,
                depth,
            ))
        } else {
            Ok(())
        }
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
    fn test_position_from_offset() {
        let source = "Line 1\nLine 2\nLine 3";
        let pos = Position::from_offset(source, 10);
        assert_eq!(pos.line, 2);
        assert_eq!(pos.column, 4);
    }

    #[test]
    fn test_range() {
        let start = Position::new(1, 10);
        let end = Position::new(1, 20);
        let range = Range::new(start, end);
        assert_eq!(range.to_string(), "1:10-1:20");
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
    fn test_parser_limits_conservative() {
        let limits = ParserLimits::conservative();
        assert_eq!(limits.max_input_size, 10 * 1024 * 1024);
        assert_eq!(limits.max_nesting_depth, 100);
    }

    #[test]
    fn test_parser_limits_strict() {
        let limits = ParserLimits::strict();
        assert_eq!(limits.max_input_size, 1024 * 1024);
        assert_eq!(limits.max_nesting_depth, 50);
    }

    #[test]
    fn test_parser_limits_check_input_size() {
        let limits = ParserLimits::strict();
        assert!(limits.check_input_size(100).is_ok());
        assert!(limits.check_input_size(2 * 1024 * 1024).is_err());
    }

    #[test]
    fn test_parser_limits_check_line_length() {
        let limits = ParserLimits::strict();
        assert!(limits.check_line_length(100).is_ok());
        assert!(limits.check_line_length(20_000).is_err());
    }

    #[test]
    fn test_parser_limits_check_nesting_depth() {
        let limits = ParserLimits::strict();
        assert!(limits.check_nesting_depth(25).is_ok());
        assert!(limits.check_nesting_depth(100).is_err());
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let clmd_err: ClmdError = io_err.into();
        assert!(matches!(clmd_err, ClmdError::Io(_)));
    }

    #[test]
    fn test_parse_error_legacy() {
        let err = ParseError::new(Position::new(1, 5), "test error");
        assert!(err.to_string().contains("Parse error at 1:5"));
    }
}
