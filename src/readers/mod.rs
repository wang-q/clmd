//! Reader modules for clmd.
//!
//! This module provides a unified interface for reading different document
//! formats into clmd's AST, inspired by Pandoc's reader architecture.
//!
//! # Supported Formats
//!
//! - Markdown (CommonMark and GFM)
//! - HTML
//! - CommonMark
//!
//! # Example
//!
//! ```
//! use clmd::readers::{ReaderRegistry, ReaderOptions};
//!
//! let registry = ReaderRegistry::new();
//! let reader = registry.get("markdown").unwrap();
//! ```

use crate::arena::{NodeArena, NodeId};
use crate::error::ClmdError;
use crate::parser::options::Options;

pub mod registry;

pub use registry::ReaderRegistry;

/// Options for reading documents.
#[derive(Debug, Clone)]
pub struct ReaderOptions<'c> {
    /// Base options.
    pub options: Options<'c>,
    /// Whether to allow raw HTML.
    pub allow_raw_html: bool,
    /// Whether to allow dangerous URLs (javascript: etc).
    pub allow_dangerous_urls: bool,
    /// File scope for includes.
    pub file_scope: bool,
    /// Tab stop width.
    pub tab_stop: usize,
    /// Whether to track changes.
    pub track_changes: bool,
    /// Default image extension.
    pub default_image_extension: String,
}

impl<'c> Default for ReaderOptions<'c> {
    fn default() -> Self {
        Self {
            options: Options::default(),
            allow_raw_html: true,
            allow_dangerous_urls: false,
            file_scope: false,
            tab_stop: 4,
            track_changes: false,
            default_image_extension: String::new(),
        }
    }
}

impl<'c> ReaderOptions<'c> {
    /// Create new reader options with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set whether to allow raw HTML.
    pub fn with_raw_html(mut self, allow: bool) -> Self {
        self.allow_raw_html = allow;
        self
    }

    /// Set whether to allow dangerous URLs.
    pub fn with_dangerous_urls(mut self, allow: bool) -> Self {
        self.allow_dangerous_urls = allow;
        self
    }

    /// Set file scope for includes.
    pub fn with_file_scope(mut self, scope: bool) -> Self {
        self.file_scope = scope;
        self
    }

    /// Set tab stop width.
    pub fn with_tab_stop(mut self, stop: usize) -> Self {
        self.tab_stop = stop;
        self
    }

    /// Set default image extension.
    pub fn with_default_image_extension<S: Into<String>>(mut self, ext: S) -> Self {
        self.default_image_extension = ext.into();
        self
    }
}

/// A reader that can parse a specific document format.
pub trait Reader: Send + Sync {
    /// Get the name of this reader.
    fn name(&self) -> &'static str;

    /// Get the file extensions supported by this reader.
    fn extensions(&self) -> &[&'static str];

    /// Read a document from a string.
    ///
    /// # Errors
    ///
    /// Returns an error if the document cannot be parsed.
    fn read<'c>(&self, input: &str, options: &ReaderOptions<'c>) -> Result<(NodeArena, NodeId), ClmdError>;

    /// Check if this reader supports the given file extension.
    fn supports_extension(&self, ext: &str) -> bool {
        self.extensions()
            .iter()
            .any(|e| e.eq_ignore_ascii_case(ext))
    }
}

/// Type alias for a boxed reader.
pub type BoxedReader = Box<dyn Reader>;

/// Read a Markdown document.
pub fn read_markdown(input: &str, _options: &ReaderOptions<'_>) -> Result<(NodeArena, NodeId), ClmdError> {
    use crate::parser::parse_document;
    use crate::parser::options::Options;

    let options = Options::default();
    let (arena, root) = parse_document(input, &options);
    Ok((arena, root))
}

/// Read an HTML document.
pub fn read_html(input: &str, _options: &ReaderOptions<'_>) -> Result<(NodeArena, NodeId), ClmdError> {
    use crate::from::html::convert;

    // Convert HTML to Markdown first, then parse
    let markdown = convert(input);
    let options = crate::parser::options::Options::default();
    let (arena, root) = crate::parser::parse_document(&markdown, &options);
    Ok((arena, root))
}

/// Read a CommonMark document.
pub fn read_commonmark(input: &str, options: &ReaderOptions<'_>) -> Result<(NodeArena, NodeId), ClmdError> {
    // CommonMark is a subset of Markdown, so we use the same parser
    read_markdown(input, options)
}
