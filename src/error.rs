//! Error types for the clmd parser
//!
//! This module defines error types that can occur during parsing,
//! providing detailed information about what went wrong and where.

use std::error::Error;
use std::fmt;

/// A reference to a broken link that could not be resolved.
///
/// This struct is passed to the broken link callback when a reference
/// style link cannot be resolved to a definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrokenLinkReference {
    /// The normalized form of the reference label (lowercase, collapsed whitespace)
    pub normalized: String,
    /// The original reference label as it appeared in the source
    pub original: String,
    /// The position where the reference occurred
    pub position: Position,
}

impl BrokenLinkReference {
    /// Create a new broken link reference
    pub fn new(
        normalized: impl Into<String>,
        original: impl Into<String>,
        position: Position,
    ) -> Self {
        Self {
            normalized: normalized.into(),
            original: original.into(),
            position,
        }
    }
}

/// The result of resolving a broken link reference.
///
/// When a broken link callback provides a resolution, it returns this
/// struct with the URL and optional title to use for the link.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedReference {
    /// The URL to use for the link
    pub url: String,
    /// The optional title for the link
    pub title: Option<String>,
}

impl ResolvedReference {
    /// Create a new resolved reference with just a URL
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            title: None,
        }
    }

    /// Create a new resolved reference with URL and title
    pub fn with_title(url: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            title: Some(title.into()),
        }
    }
}

/// Callback trait for handling broken link references.
///
/// Implement this trait to provide custom handling for broken links,
/// such as looking up references in an external database or generating
/// placeholder URLs.
///
/// # Example
///
/// ```
/// use clmd::error::{BrokenLinkCallback, BrokenLinkReference, ResolvedReference};
///
/// struct MyLinkResolver;
///
/// impl BrokenLinkCallback for MyLinkResolver {
///     fn resolve(&self, broken_link: &BrokenLinkReference) -> Option<ResolvedReference> {
///         // Example: resolve wiki-style links
///         if broken_link.normalized.starts_with("wiki:") {
///             let page = &broken_link.normalized[5..];
///             Some(ResolvedReference::new(format!("/wiki/{}", page)))
///         } else {
///             None
///         }
///     }
/// }
/// ```
pub trait BrokenLinkCallback: Send + Sync {
    /// Resolve a broken link reference.
    ///
    /// # Arguments
    ///
    /// * `broken_link` - Information about the broken link
    ///
    /// # Returns
    ///
    /// Some(ResolvedReference) if the link could be resolved, None to leave it as is
    fn resolve(&self, broken_link: &BrokenLinkReference) -> Option<ResolvedReference>;
}

/// A simple broken link callback that always returns None.
#[derive(Debug, Clone, Copy)]
pub struct DefaultBrokenLinkCallback;

impl BrokenLinkCallback for DefaultBrokenLinkCallback {
    fn resolve(&self, _broken_link: &BrokenLinkReference) -> Option<ResolvedReference> {
        None
    }
}

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
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// Input exceeds maximum allowed size
    InputTooLarge {
        /// Size of the input in bytes
        size: usize,
        /// Maximum allowed size in bytes
        max_size: usize,
    },

    /// Line exceeds maximum allowed length
    LineTooLong {
        /// Line number where the error occurred
        line: usize,
        /// Length of the line in bytes
        length: usize,
        /// Maximum allowed length in bytes
        max_length: usize,
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

    /// Broken link reference (link to non-existent definition)
    BrokenLinkReference {
        /// The reference label that could not be found
        label: String,
        /// Position where the reference occurred
        position: Position,
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

    /// Too many list items
    TooManyListItems {
        /// Number of list items found
        count: usize,
        /// Maximum allowed
        max_count: usize,
        /// Position where the limit was exceeded
        position: Position,
    },

    /// Too many links
    TooManyLinks {
        /// Number of links found
        count: usize,
        /// Maximum allowed
        max_count: usize,
        /// Position where the limit was exceeded
        position: Position,
    },

    /// Invalid footnote definition
    InvalidFootnote {
        /// The footnote label that caused the error
        label: String,
        /// Position where the error occurred
        position: Position,
        /// Detailed error message
        message: String,
    },

    /// Duplicate footnote definition
    DuplicateFootnote {
        /// The footnote label that was duplicated
        label: String,
        /// Position of the first definition
        first_position: Position,
        /// Position of the duplicate definition
        duplicate_position: Position,
    },

    /// Invalid YAML front matter
    InvalidFrontMatter {
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
            ParseError::LineTooLong {
                line,
                length,
                max_length,
            } => {
                write!(
                    f,
                    "Line {} too long: {} bytes (maximum allowed: {} bytes)",
                    line, length, max_length
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
            ParseError::BrokenLinkReference { label, position } => {
                write!(
                    f,
                    "Broken link reference '{}' at {}: no definition found",
                    label, position
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
            ParseError::TooManyListItems {
                count,
                max_count,
                position,
            } => {
                write!(
                    f,
                    "Too many list items at {}: {} (maximum allowed: {})",
                    position, count, max_count
                )
            }
            ParseError::TooManyLinks {
                count,
                max_count,
                position,
            } => {
                write!(
                    f,
                    "Too many links at {}: {} (maximum allowed: {})",
                    position, count, max_count
                )
            }
            ParseError::InvalidFootnote {
                label,
                position,
                message,
            } => {
                write!(
                    f,
                    "Invalid footnote '{}' at {}: {}",
                    label, position, message
                )
            }
            ParseError::DuplicateFootnote {
                label,
                first_position,
                duplicate_position,
            } => {
                write!(
                    f,
                    "Duplicate footnote '{}' at {}: already defined at {}",
                    label, duplicate_position, first_position
                )
            }
            ParseError::InvalidFrontMatter { position, message } => {
                write!(f, "Invalid front matter at {}: {}", position, message)
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

/// Default maximum input size: 10MB
const DEFAULT_MAX_INPUT_SIZE: usize = 10 * 1024 * 1024;

/// Default maximum nesting depth
const DEFAULT_MAX_NESTING_DEPTH: usize = 100;

/// Default maximum line length
const DEFAULT_MAX_LINE_LENGTH: usize = 10_000;

/// Default maximum list items
const DEFAULT_MAX_LIST_ITEMS: usize = 10_000;

/// Default maximum links
const DEFAULT_MAX_LINKS: usize = 10_000;

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
            max_input_size: DEFAULT_MAX_INPUT_SIZE,
            max_nesting_depth: DEFAULT_MAX_NESTING_DEPTH,
            max_line_length: DEFAULT_MAX_LINE_LENGTH,
            max_list_items: DEFAULT_MAX_LIST_ITEMS,
            max_links: DEFAULT_MAX_LINKS,
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
            max_size: DEFAULT_MAX_INPUT_SIZE,
        };
        assert!(err.to_string().contains("Input too large"));
        assert!(err.to_string().contains("20000000"));
        assert!(err.to_string().contains("10485760")); // 10MB in bytes
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
        assert_eq!(limits.max_input_size, DEFAULT_MAX_INPUT_SIZE);
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

    #[test]
    fn test_line_too_long_error() {
        let err = ParseError::LineTooLong {
            line: 10,
            length: 15000,
            max_length: 10000,
        };
        let msg = err.to_string();
        assert!(msg.contains("Line 10 too long"));
        assert!(msg.contains("15000"));
        assert!(msg.contains("10000"));
    }

    #[test]
    fn test_broken_link_reference_error() {
        let pos = Position::new(5, 10, 100);
        let err = ParseError::BrokenLinkReference {
            label: "missing-ref".to_string(),
            position: pos,
        };
        let msg = err.to_string();
        assert!(msg.contains("Broken link reference"));
        assert!(msg.contains("missing-ref"));
        assert!(msg.contains("no definition found"));
    }

    #[test]
    fn test_too_many_list_items_error() {
        let pos = Position::new(100, 1, 5000);
        let err = ParseError::TooManyListItems {
            count: 15000,
            max_count: 10000,
            position: pos,
        };
        let msg = err.to_string();
        assert!(msg.contains("Too many list items"));
        assert!(msg.contains("15000"));
        assert!(msg.contains("10000"));
    }

    #[test]
    fn test_too_many_links_error() {
        let pos = Position::new(50, 5, 2500);
        let err = ParseError::TooManyLinks {
            count: 15000,
            max_count: 10000,
            position: pos,
        };
        let msg = err.to_string();
        assert!(msg.contains("Too many links"));
        assert!(msg.contains("15000"));
        assert!(msg.contains("10000"));
    }

    #[test]
    fn test_invalid_footnote_error() {
        let pos = Position::new(20, 5, 800);
        let err = ParseError::InvalidFootnote {
            label: "bad-footnote".to_string(),
            position: pos,
            message: "invalid characters".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("Invalid footnote"));
        assert!(msg.contains("bad-footnote"));
        assert!(msg.contains("invalid characters"));
    }

    #[test]
    fn test_duplicate_footnote_error() {
        let first_pos = Position::new(10, 1, 200);
        let dup_pos = Position::new(30, 1, 600);
        let err = ParseError::DuplicateFootnote {
            label: "dup-footnote".to_string(),
            first_position: first_pos,
            duplicate_position: dup_pos,
        };
        let msg = err.to_string();
        assert!(msg.contains("Duplicate footnote"));
        assert!(msg.contains("dup-footnote"));
        assert!(msg.contains("line 30, column 1"));
        assert!(msg.contains("line 10, column 1"));
    }

    #[test]
    fn test_invalid_front_matter_error() {
        let pos = Position::new(1, 1, 0);
        let err = ParseError::InvalidFrontMatter {
            position: pos,
            message: "malformed YAML".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("Invalid front matter"));
        assert!(msg.contains("malformed YAML"));
    }

    // Tests for broken link callback
    #[test]
    fn test_broken_link_reference_struct() {
        let pos = Position::new(5, 10, 100);
        let broken = BrokenLinkReference::new("wiki:page", "Wiki:Page", pos);
        assert_eq!(broken.normalized, "wiki:page");
        assert_eq!(broken.original, "Wiki:Page");
        assert_eq!(broken.position.line, 5);
    }

    #[test]
    fn test_resolved_reference() {
        let resolved = ResolvedReference::new("https://example.com");
        assert_eq!(resolved.url, "https://example.com");
        assert_eq!(resolved.title, None);

        let resolved_with_title =
            ResolvedReference::with_title("https://example.com", "Example");
        assert_eq!(resolved_with_title.url, "https://example.com");
        assert_eq!(resolved_with_title.title, Some("Example".to_string()));
    }

    #[test]
    fn test_default_broken_link_callback() {
        let callback = DefaultBrokenLinkCallback;
        let broken = BrokenLinkReference::new("test", "test", Position::start());
        assert!(callback.resolve(&broken).is_none());
    }

    #[test]
    fn test_custom_broken_link_callback() {
        struct WikiLinkResolver;
        impl BrokenLinkCallback for WikiLinkResolver {
            fn resolve(
                &self,
                broken_link: &BrokenLinkReference,
            ) -> Option<ResolvedReference> {
                if broken_link.normalized.starts_with("wiki:") {
                    let page = &broken_link.normalized[5..];
                    Some(ResolvedReference::new(format!("/wiki/{}", page)))
                } else {
                    None
                }
            }
        }

        let resolver = WikiLinkResolver;

        // Should resolve wiki links
        let wiki_link =
            BrokenLinkReference::new("wiki:home", "wiki:home", Position::start());
        let resolved = resolver.resolve(&wiki_link);
        assert!(resolved.is_some());
        assert_eq!(resolved.unwrap().url, "/wiki/home");

        // Should not resolve other links
        let other_link = BrokenLinkReference::new("other", "other", Position::start());
        assert!(resolver.resolve(&other_link).is_none());
    }
}
