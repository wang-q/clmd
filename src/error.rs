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

    /// Deprecated feature error.
    #[error("Deprecated feature: {0}")]
    Deprecated(String),

    /// Warning (non-fatal error).
    #[error("Warning: {0}")]
    Warning(String),

    /// Sandbox security error.
    #[error("Sandbox error: {0}")]
    Sandbox(String),
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

    /// Create a deprecated feature error.
    pub fn deprecated<S: Into<String>>(feature: S) -> Self {
        Self::Deprecated(feature.into())
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
        matches!(self, Self::LimitExceeded { .. })
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

    /// Check if this is a deprecated feature error.
    pub fn is_deprecated(&self) -> bool {
        matches!(self, Self::Deprecated(_))
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
            Self::Deprecated(_) => 2,
            Self::Warning(_) => 0, // Warnings don't cause non-zero exit
            Self::Sandbox(_) => 77, // Permission denied
            Self::Other(_) => 1,
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

/// Error renderer for displaying errors with context.
///
/// This struct provides methods for rendering errors in various formats,
/// including human-readable messages with source context.
#[derive(Debug, Clone, Copy)]
pub struct ErrorRenderer {
    /// Whether to use colors in output.
    use_colors: bool,
    /// Context lines to show around errors.
    context_lines: usize,
}

impl ErrorRenderer {
    /// Create a new error renderer.
    pub fn new() -> Self {
        Self {
            use_colors: true,
            context_lines: 2,
        }
    }

    /// Create a renderer without colors.
    pub fn no_colors() -> Self {
        Self {
            use_colors: false,
            context_lines: 2,
        }
    }

    /// Set whether to use colors.
    pub fn with_colors(mut self, use_colors: bool) -> Self {
        self.use_colors = use_colors;
        self
    }

    /// Set the number of context lines.
    pub fn with_context_lines(mut self, lines: usize) -> Self {
        self.context_lines = lines;
        self
    }

    /// Render an error with source context.
    ///
    /// # Arguments
    ///
    /// * `error` - The error to render
    /// * `source` - The source text (optional)
    /// * `source_name` - Name of the source file (optional)
    pub fn render(
        &self,
        error: &ClmdError,
        source: Option<&str>,
        source_name: Option<&str>,
    ) -> String {
        let mut output = String::new();

        // Add source name if available
        if let Some(name) = source_name {
            if self.use_colors {
                output.push_str(&format!("\x1b[1m{}:\x1b[0m ", name));
            } else {
                output.push_str(&format!("{}: ", name));
            }
        }

        // Add error message
        if self.use_colors {
            if error.is_warning() {
                output.push_str(&format!("\x1b[33mwarning:\x1b[0m {}\n", error));
            } else {
                output.push_str(&format!("\x1b[31merror:\x1b[0m {}\n", error));
            }
        } else {
            if error.is_warning() {
                output.push_str(&format!("warning: {}\n", error));
            } else {
                output.push_str(&format!("error: {}\n", error));
            }
        }

        // Add source context if available and error has position
        if let (Some(pos), Some(src)) = (error.position(), source) {
            output.push_str(&self.render_source_context(src, pos));
        }

        output
    }

    /// Render source context around a position.
    fn render_source_context(&self, source: &str, pos: Position) -> String {
        let mut output = String::new();
        let lines: Vec<&str> = source.lines().collect();

        if pos.line == 0 || pos.line > lines.len() {
            return output;
        }

        let start_line = pos.line.saturating_sub(self.context_lines).max(1);
        let end_line = (pos.line + self.context_lines).min(lines.len());

        // Calculate line number width for padding
        let line_num_width = end_line.to_string().len();

        for line_num in start_line..=end_line {
            let line = lines[line_num - 1];
            let is_error_line = line_num == pos.line;

            // Line number
            if self.use_colors && is_error_line {
                output.push_str(&format!(
                    "\x1b[34m{:>width$} |\x1b[0m ",
                    line_num,
                    width = line_num_width
                ));
            } else {
                output.push_str(&format!(
                    "{:>width$} | ",
                    line_num,
                    width = line_num_width
                ));
            }

            // Line content
            output.push_str(line);
            output.push('\n');

            // Error indicator
            if is_error_line {
                let padding = line_num_width + 3 + pos.column.saturating_sub(1);
                if self.use_colors {
                    output
                        .push_str(&format!("{}\x1b[31m^\x1b[0m\n", " ".repeat(padding)));
                } else {
                    output.push_str(&format!("{}^\n", " ".repeat(padding)));
                }
            }
        }

        output
    }

    /// Render a simple error message without context.
    pub fn render_simple(&self, error: &ClmdError) -> String {
        if self.use_colors {
            if error.is_warning() {
                format!("\x1b[33mwarning:\x1b[0m {}", error)
            } else {
                format!("\x1b[31merror:\x1b[0m {}", error)
            }
        } else {
            if error.is_warning() {
                format!("warning: {}", error)
            } else {
                format!("error: {}", error)
            }
        }
    }
}

impl Default for ErrorRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Collect multiple errors and warnings.
#[derive(Debug, Clone, Default)]
pub struct ErrorCollector {
    errors: Vec<ClmdError>,
    warnings: Vec<ClmdError>,
}

impl ErrorCollector {
    /// Create a new error collector.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an error.
    pub fn add_error(&mut self, error: ClmdError) {
        self.errors.push(error);
    }

    /// Add a warning.
    pub fn add_warning(&mut self, warning: ClmdError) {
        self.warnings.push(warning);
    }

    /// Check if there are any errors.
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Check if there are any warnings.
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Check if there are any errors or warnings.
    pub fn has_any(&self) -> bool {
        self.has_errors() || self.has_warnings()
    }

    /// Get the number of errors.
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Get the number of warnings.
    pub fn warning_count(&self) -> usize {
        self.warnings.len()
    }

    /// Get all errors.
    pub fn errors(&self) -> &[ClmdError] {
        &self.errors
    }

    /// Get all warnings.
    pub fn warnings(&self) -> &[ClmdError] {
        &self.warnings
    }

    /// Get the first error, if any.
    pub fn first_error(&self) -> Option<&ClmdError> {
        self.errors.first()
    }

    /// Take all errors (consumes self).
    pub fn take_errors(self) -> Vec<ClmdError> {
        self.errors
    }

    /// Take all warnings (consumes self).
    pub fn take_warnings(self) -> Vec<ClmdError> {
        self.warnings
    }

    /// Convert to a single result.
    ///
    /// Returns Ok if there are no errors, Err with the first error otherwise.
    pub fn into_result<T>(self, value: T) -> ClmdResult<T> {
        if let Some(err) = self.errors.into_iter().next() {
            Err(err)
        } else {
            Ok(value)
        }
    }

    /// Render all errors and warnings.
    pub fn render_all(
        &self,
        renderer: &ErrorRenderer,
        source: Option<&str>,
        source_name: Option<&str>,
    ) -> String {
        let mut output = String::new();

        for error in &self.errors {
            output.push_str(&renderer.render(error, source, source_name));
            output.push('\n');
        }

        for warning in &self.warnings {
            output.push_str(&renderer.render(warning, source, source_name));
            output.push('\n');
        }

        output
    }

    /// Clear all errors and warnings.
    pub fn clear(&mut self) {
        self.errors.clear();
        self.warnings.clear();
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

    #[test]
    fn test_error_renderer() {
        let renderer = ErrorRenderer::new();
        let err = ClmdError::parse_error(Position::new(2, 5), "unexpected token");
        let source = "Line 1\nLine 2 with error\nLine 3";

        let output = renderer.render(&err, Some(source), Some("test.md"));
        assert!(output.contains("error:"));
        assert!(output.contains("test.md"));
    }

    #[test]
    fn test_error_renderer_no_colors() {
        let renderer = ErrorRenderer::no_colors();
        let err = ClmdError::io_error("file not found");

        let output = renderer.render_simple(&err);
        assert!(output.contains("error:"));
        assert!(!output.contains('\x1b')); // No ANSI codes
    }

    #[test]
    fn test_error_collector() {
        let mut collector = ErrorCollector::new();

        assert!(!collector.has_errors());
        assert!(!collector.has_any());

        collector.add_error(ClmdError::io_error("error 1"));
        collector.add_warning(ClmdError::warning("warning 1"));

        assert!(collector.has_errors());
        assert!(collector.has_warnings());
        assert!(collector.has_any());
        assert_eq!(collector.error_count(), 1);
        assert_eq!(collector.warning_count(), 1);

        let result: ClmdResult<i32> = collector.into_result(42);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_collector_success() {
        let collector = ErrorCollector::new();
        let result: ClmdResult<i32> = collector.into_result(42);
        assert_eq!(result.unwrap(), 42);
    }
}
