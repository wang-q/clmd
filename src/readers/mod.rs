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
//! use clmd::readers::{ReaderRegistry, ReaderOptions, ReaderInput};
//!
//! let registry = ReaderRegistry::with_defaults();
//! assert!(registry.get("markdown").is_some());
//!
//! // Read from text input
//! let input = ReaderInput::text("# Hello World");
//! ```

use crate::arena::{NodeArena, NodeId};
use crate::error::ClmdError;
use crate::parser::options::Options;

pub mod registry;

pub use registry::{
    default_registry, get_reader, get_reader_by_extension, get_reader_by_path,
    CommonMarkReader, HtmlReader, MarkdownReader, ReaderRegistry,
};

/// Input type for readers.
///
/// Readers can accept either text or binary input, depending on the format.
/// This enum is similar to Pandoc's Reader type which handles both text and
/// bytestring inputs.
#[derive(Debug, Clone)]
pub enum ReaderInput {
    /// Text input (UTF-8 encoded).
    Text(String),
    /// Binary input.
    Binary(Vec<u8>),
}

impl ReaderInput {
    /// Create a text input.
    pub fn text<S: Into<String>>(text: S) -> Self {
        Self::Text(text.into())
    }

    /// Create a binary input.
    pub fn binary<B: Into<Vec<u8>>>(bytes: B) -> Self {
        Self::Binary(bytes.into())
    }

    /// Read input from a file, detecting whether it's text or binary.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read.
    pub fn from_file(path: &std::path::Path) -> Result<Self, ClmdError> {
        use std::fs;
        use std::io::Read;

        let mut file = fs::File::open(path).map_err(|e| {
            ClmdError::io_error(format!("Cannot open file {}: {}", path.display(), e))
        })?;

        let mut contents = Vec::new();
        file.read_to_end(&mut contents).map_err(|e| {
            ClmdError::io_error(format!("Cannot read file {}: {}", path.display(), e))
        })?;

        // Try to decode as UTF-8, fall back to binary if it fails
        match String::from_utf8(contents.clone()) {
            Ok(text) => Ok(Self::Text(text)),
            Err(_) => Ok(Self::Binary(contents)),
        }
    }

    /// Check if this is text input.
    pub fn is_text(&self) -> bool {
        matches!(self, Self::Text(_))
    }

    /// Check if this is binary input.
    pub fn is_binary(&self) -> bool {
        matches!(self, Self::Binary(_))
    }

    /// Get the text content if this is text input.
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text(text) => Some(text),
            Self::Binary(_) => None,
        }
    }

    /// Get the binary content if this is binary input.
    pub fn as_binary(&self) -> Option<&[u8]> {
        match self {
            Self::Text(_) => None,
            Self::Binary(bytes) => Some(bytes),
        }
    }

    /// Convert to text, decoding if necessary.
    ///
    /// For binary input, this attempts UTF-8 decoding and may fail.
    pub fn to_text(self) -> Result<String, ClmdError> {
        match self {
            Self::Text(text) => Ok(text),
            Self::Binary(bytes) => String::from_utf8(bytes)
                .map_err(|e| ClmdError::encoding_error(format!("Invalid UTF-8: {}", e))),
        }
    }

    /// Convert to binary.
    ///
    /// For text input, this encodes as UTF-8.
    pub fn to_binary(self) -> Vec<u8> {
        match self {
            Self::Text(text) => text.into_bytes(),
            Self::Binary(bytes) => bytes,
        }
    }

    /// Get the size of the input in bytes.
    pub fn len(&self) -> usize {
        match self {
            Self::Text(text) => text.len(),
            Self::Binary(bytes) => bytes.len(),
        }
    }

    /// Check if the input is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl From<String> for ReaderInput {
    fn from(text: String) -> Self {
        Self::Text(text)
    }
}

impl From<&str> for ReaderInput {
    fn from(text: &str) -> Self {
        Self::Text(text.to_string())
    }
}

impl From<Vec<u8>> for ReaderInput {
    fn from(bytes: Vec<u8>) -> Self {
        Self::Binary(bytes)
    }
}

impl From<&[u8]> for ReaderInput {
    fn from(bytes: &[u8]) -> Self {
        Self::Binary(bytes.to_vec())
    }
}

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
    /// Source name (for error messages).
    pub source_name: Option<String>,
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
            source_name: None,
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

    /// Set source name for error messages.
    pub fn with_source_name<S: Into<String>>(mut self, name: S) -> Self {
        self.source_name = Some(name.into());
        self
    }
}

/// A reader that can parse a specific document format.
///
/// This trait is inspired by Pandoc's Reader type, which supports both
/// text and binary input formats.
pub trait Reader: Send + Sync {
    /// Get the name of this reader.
    fn name(&self) -> &'static str;

    /// Get the file extensions supported by this reader.
    fn extensions(&self) -> &[&'static str];

    /// Check if this reader supports binary input.
    ///
    /// Default is `false` - most readers only support text input.
    fn supports_binary(&self) -> bool {
        false
    }

    /// Read a document from text input.
    ///
    /// # Errors
    ///
    /// Returns an error if the document cannot be parsed.
    fn read_text<'c>(
        &self,
        input: &str,
        options: &ReaderOptions<'c>,
    ) -> Result<(NodeArena, NodeId), ClmdError>;

    /// Read a document from binary input.
    ///
    /// Default implementation returns an error. Readers that support
    /// binary formats (like DOCX, ODT) should override this.
    ///
    /// # Errors
    ///
    /// Returns an error if the document cannot be parsed.
    fn read_binary<'c>(
        &self,
        _input: &[u8],
        _options: &ReaderOptions<'c>,
    ) -> Result<(NodeArena, NodeId), ClmdError> {
        Err(ClmdError::unknown_reader(format!(
            "Reader '{}' does not support binary input",
            self.name()
        )))
    }

    /// Read a document from any input type.
    ///
    /// This method automatically dispatches to `read_text` or `read_binary`
    /// based on the input type.
    ///
    /// # Errors
    ///
    /// Returns an error if the document cannot be parsed.
    fn read<'c>(
        &self,
        input: &ReaderInput,
        options: &ReaderOptions<'c>,
    ) -> Result<(NodeArena, NodeId), ClmdError> {
        match input {
            ReaderInput::Text(text) => self.read_text(text, options),
            ReaderInput::Binary(bytes) => self.read_binary(bytes, options),
        }
    }

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
pub fn read_markdown(
    input: &str,
    _options: &ReaderOptions<'_>,
) -> Result<(NodeArena, NodeId), ClmdError> {
    use crate::parser::options::Options;
    use crate::parser::parse_document;

    let options = Options::default();
    let (arena, root) = parse_document(input, &options);
    Ok((arena, root))
}

/// Read an HTML document.
pub fn read_html(
    input: &str,
    _options: &ReaderOptions<'_>,
) -> Result<(NodeArena, NodeId), ClmdError> {
    use crate::from::html::convert;

    // Convert HTML to Markdown first, then parse
    let markdown = convert(input);
    let options = crate::parser::options::Options::default();
    let (arena, root) = crate::parser::parse_document(&markdown, &options);
    Ok((arena, root))
}

/// Read a CommonMark document.
pub fn read_commonmark(
    input: &str,
    options: &ReaderOptions<'_>,
) -> Result<(NodeArena, NodeId), ClmdError> {
    // CommonMark is a subset of Markdown, so we use the same parser
    read_markdown(input, options)
}
