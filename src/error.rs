//! Error types for the clmd parser
//!
//! This module defines error types that can occur during parsing,
//! providing detailed information about what went wrong and where.

use std::error::Error;
use std::fmt;

/// Position in the source document
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed)
    pub column: usize,
    /// Byte offset from the start of the document
    pub offset: usize,
}

impl Position {
    /// Create a new position
    pub fn new(line: usize, column: usize, offset: usize) -> Self {
        Position {
            line,
            column,
            offset,
        }
    }

    /// Create a position at the start of the document
    pub fn start() -> Self {
        Position::new(1, 1, 0)
    }
}

impl Default for Position {
    fn default() -> Self {
        Position::new(0, 0, 0)
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}, column {}", self.line, self.column)
    }
}

/// Errors that can occur during parsing
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// Input exceeds maximum allowed size
    InputTooLarge {
        /// Size of the input in bytes
        size: usize,
        /// Maximum allowed size in bytes
        max_size: usize,
    },

    /// Nesting depth exceeds maximum allowed
    NestingTooDeep {
        /// Current nesting depth
        depth: usize,
        /// Maximum allowed depth
        max_depth: usize,
        /// Position where the error occurred
        position: Position,
    },

    /// Invalid reference definition
    InvalidReference {
        /// The reference label that caused the error
        label: String,
        /// Position where the error occurred
        position: Position,
        /// Detailed error message
        message: String,
    },

    /// Invalid URL in link or image
    InvalidUrl {
        /// The URL that caused the error
        url: String,
        /// Position where the error occurred
        position: Position,
        /// Detailed error message
        message: String,
    },

    /// Invalid HTML tag
    InvalidHtml {
        /// The HTML that caused the error
        html: String,
        /// Position where the error occurred
        position: Position,
        /// Detailed error message
        message: String,
    },

    /// Malformed table
    InvalidTable {
        /// Position where the error occurred
        position: Position,
        /// Detailed error message
        message: String,
    },

    /// Invalid character encoding
    InvalidEncoding {
        /// Position where the error occurred
        position: Position,
        /// Detailed error message
        message: String,
    },

    /// Generic parse error
    ParseError {
        /// Position where the error occurred
        position: Position,
        /// Detailed error message
        message: String,
    },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::InputTooLarge { size, max_size } => {
                write!(
                    f,
                    "Input too large: {} bytes (maximum allowed: {} bytes)",
                    size, max_size
                )
            }
            ParseError::NestingTooDeep {
                depth,
                max_depth,
                position,
            } => {
                write!(
                    f,
                    "Nesting too deep at {}: depth {} (maximum allowed: {})",
                    position, depth, max_depth
                )
            }
            ParseError::InvalidReference {
                label,
                position,
                message,
            } => {
                write!(
                    f,
                    "Invalid reference '{}' at {}: {}",
                    label, position, message
                )
            }
            ParseError::InvalidUrl {
                url,
                position,
                message,
            } => {
                write!(f, "Invalid URL '{}' at {}: {}", url, position, message)
            }
            ParseError::InvalidHtml {
                html,
                position,
                message,
            } => {
                write!(f, "Invalid HTML '{}' at {}: {}", html, position, message)
            }
            ParseError::InvalidTable { position, message } => {
                write!(f, "Invalid table at {}: {}", position, message)
            }
            ParseError::InvalidEncoding { position, message } => {
                write!(f, "Invalid encoding at {}: {}", position, message)
            }
            ParseError::ParseError { position, message } => {
                write!(f, "Parse error at {}: {}", position, message)
            }
        }
    }
}

impl Error for ParseError {}

/// Result type alias for parse operations
pub type ParseResult<T> = Result<T, ParseError>;

/// Configuration for parser limits and validation
#[derive(Debug, Clone, Copy)]
pub struct ParserLimits {
    /// Maximum input size in bytes (default: 10MB)
    pub max_input_size: usize,
    /// Maximum nesting depth (default: 100)
    pub max_nesting_depth: usize,
    /// Maximum line length in bytes (default: 10000)
    pub max_line_length: usize,
    /// Maximum number of list items (default: 10000)
    pub max_list_items: usize,
    /// Maximum number of links (default: 10000)
    pub max_links: usize,
}

impl Default for ParserLimits {
    fn default() -> Self {
        ParserLimits {
            max_input_size: 10 * 1024 * 1024, // 10MB
            max_nesting_depth: 100,
            max_line_length: 10000,
            max_list_items: 10000,
            max_links: 10000,
        }
    }
}

impl ParserLimits {
    /// Create a new ParserLimits with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum input size
    pub fn max_input_size(mut self, size: usize) -> Self {
        self.max_input_size = size;
        self
    }

    /// Set maximum nesting depth
    pub fn max_nesting_depth(mut self, depth: usize) -> Self {
        self.max_nesting_depth = depth;
        self
    }

    /// Set maximum line length
    pub fn max_line_length(mut self, length: usize) -> Self {
        self.max_line_length = length;
        self
    }

    /// Set maximum number of list items
    pub fn max_list_items(mut self, count: usize) -> Self {
        self.max_list_items = count;
        self
    }

    /// Set maximum number of links
    pub fn max_links(mut self, count: usize) -> Self {
        self.max_links = count;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_display() {
        let pos = Position::new(10, 5, 100);
        assert_eq!(pos.to_string(), "line 10, column 5");
    }

    #[test]
    fn test_parse_error_display() {
        let err = ParseError::InputTooLarge {
            size: 20_000_000,
            max_size: 10_000_000,
        };
        assert!(err.to_string().contains("Input too large"));
        assert!(err.to_string().contains("20000000"));
        assert!(err.to_string().contains("10000000"));
    }

    #[test]
    fn test_nesting_too_deep_error() {
        let pos = Position::new(5, 10, 100);
        let err = ParseError::NestingTooDeep {
            depth: 150,
            max_depth: 100,
            position: pos,
        };
        let msg = err.to_string();
        assert!(msg.contains("Nesting too deep"));
        assert!(msg.contains("line 5, column 10"));
        assert!(msg.contains("150"));
        assert!(msg.contains("100"));
    }

    #[test]
    fn test_invalid_reference_error() {
        let pos = Position::new(3, 15, 50);
        let err = ParseError::InvalidReference {
            label: "bad-ref".to_string(),
            position: pos,
            message: "unclosed bracket".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("Invalid reference"));
        assert!(msg.contains("bad-ref"));
        assert!(msg.contains("unclosed bracket"));
    }

    #[test]
    fn test_parser_limits_default() {
        let limits = ParserLimits::default();
        assert_eq!(limits.max_input_size, 10 * 1024 * 1024);
        assert_eq!(limits.max_nesting_depth, 100);
        assert_eq!(limits.max_line_length, 10000);
        assert_eq!(limits.max_list_items, 10000);
        assert_eq!(limits.max_links, 10000);
    }

    #[test]
    fn test_parser_limits_builder() {
        let limits = ParserLimits::new()
            .max_input_size(5 * 1024 * 1024)
            .max_nesting_depth(50)
            .max_line_length(5000);

        assert_eq!(limits.max_input_size, 5 * 1024 * 1024);
        assert_eq!(limits.max_nesting_depth, 50);
        assert_eq!(limits.max_line_length, 5000);
    }

    #[test]
    fn test_parse_result_type() {
        fn may_fail() -> ParseResult<i32> {
            Ok(42)
        }

        fn always_fails() -> ParseResult<i32> {
            Err(ParseError::ParseError {
                position: Position::start(),
                message: "test error".to_string(),
            })
        }

        assert_eq!(may_fail().unwrap(), 42);
        assert!(always_fails().is_err());
    }
}
