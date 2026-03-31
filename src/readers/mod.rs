//! Document readers for various input formats.
//!
//! This module provides a unified interface for reading documents from different
//! formats, inspired by Pandoc's Reader system.
//!
//! # Example
//!
//! ```
//! use clmd::readers::{Reader, MarkdownReader};
//! use clmd::Options;
//!
//! let reader = MarkdownReader::new();
//! let options = Options::default();
//! let document = reader.read("# Hello\n\nWorld", &options).unwrap();
//! ```

use crate::arena::{NodeArena, NodeId};
use crate::error::{ParseError, ParseResult};
use crate::options::Options;
use std::collections::HashMap;

/// A document representation containing the AST and metadata.
#[derive(Debug)]
pub struct Document {
    /// The arena containing all AST nodes.
    pub arena: NodeArena,
    /// The root node ID of the document.
    pub root: NodeId,
    /// Document metadata (from YAML front matter).
    pub metadata: HashMap<String, String>,
}

/// Trait for document readers.
///
/// Implement this trait to add support for new input formats.
pub trait Reader {
    /// Read a document from the input string.
    ///
    /// # Arguments
    ///
    /// * `input` - The input text to parse
    /// * `options` - Parsing options
    ///
    /// # Returns
    ///
    /// A `Document` containing the parsed AST, or a `ParseError` if parsing fails.
    fn read(&self, input: &str, options: &Options) -> ParseResult<Document>;

    /// Get the format name this reader supports.
    fn format_name(&self) -> &'static str;
}

/// Reader for Markdown/CommonMark format.
#[derive(Debug, Clone, Copy)]
pub struct MarkdownReader;

impl MarkdownReader {
    /// Create a new Markdown reader.
    pub fn new() -> Self {
        Self
    }
}

impl Default for MarkdownReader {
    fn default() -> Self {
        Self::new()
    }
}

impl Reader for MarkdownReader {
    fn read(&self, input: &str, options: &Options) -> ParseResult<Document> {
        let (arena, root) = crate::parser::parse_document(input, options);
        Ok(Document {
            arena,
            root,
            metadata: HashMap::new(),
        })
    }

    fn format_name(&self) -> &'static str {
        "markdown"
    }
}

/// Reader for HTML format (converts HTML to Markdown AST).
#[derive(Debug, Clone, Copy)]
pub struct HtmlReader;

impl HtmlReader {
    /// Create a new HTML reader.
    pub fn new() -> Self {
        Self
    }
}

impl Default for HtmlReader {
    fn default() -> Self {
        Self::new()
    }
}

impl Reader for HtmlReader {
    fn read(&self, input: &str, _options: &Options) -> ParseResult<Document> {
        // Convert HTML to Markdown, then parse
        let markdown = crate::from::html_to_markdown(input);
        let (arena, root) =
            crate::parser::parse_document(&markdown, &Options::default());
        Ok(Document {
            arena,
            root,
            metadata: HashMap::new(),
        })
    }

    fn format_name(&self) -> &'static str {
        "html"
    }
}

/// Registry of available readers.
pub struct ReaderRegistry {
    readers: HashMap<String, Box<dyn Reader>>,
}

impl std::fmt::Debug for ReaderRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReaderRegistry")
            .field("formats", &self.supported_formats())
            .finish()
    }
}

impl ReaderRegistry {
    /// Create a new reader registry with default readers.
    pub fn new() -> Self {
        let mut registry = Self {
            readers: HashMap::new(),
        };
        registry.register(Box::new(MarkdownReader::new()));
        registry.register(Box::new(HtmlReader::new()));
        registry
    }

    /// Register a new reader.
    pub fn register(&mut self, reader: Box<dyn Reader>) {
        let name = reader.format_name().to_string();
        self.readers.insert(name, reader);
    }

    /// Get a reader by format name.
    pub fn get(&self, format: &str) -> Option<&dyn Reader> {
        self.readers.get(format).map(|r| r.as_ref())
    }

    /// Check if a format is supported.
    pub fn supports(&self, format: &str) -> bool {
        self.readers.contains_key(format)
    }

    /// Get a list of supported formats.
    pub fn supported_formats(&self) -> Vec<&str> {
        self.readers.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for ReaderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Read a document using the specified format.
///
/// # Arguments
///
/// * `input` - The input text
/// * `format` - The input format (e.g., "markdown", "html")
/// * `options` - Parsing options
///
/// # Returns
///
/// A `Document` or a `ParseError` if the format is not supported.
pub fn read_document(
    input: &str,
    format: &str,
    options: &Options,
) -> ParseResult<Document> {
    let registry = ReaderRegistry::new();
    let reader = registry.get(format).ok_or_else(|| ParseError::ParseError {
        position: crate::error::Position::start(),
        message: format!("Unknown reader format: {}", format),
    })?;
    reader.read(input, options)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_reader() {
        let reader = MarkdownReader::new();
        let options = Options::default();
        let doc = reader.read("# Hello\n\nWorld", &options).unwrap();

        assert_eq!(doc.metadata.len(), 0);
        // Root should be a document node
        let root = doc.arena.get(doc.root);
        assert!(matches!(root.value, crate::nodes::NodeValue::Document));
    }

    #[test]
    fn test_reader_registry() {
        let registry = ReaderRegistry::new();
        assert!(registry.supports("markdown"));
        assert!(registry.supports("html"));
        assert!(!registry.supports("unknown"));

        let formats = registry.supported_formats();
        assert!(formats.contains(&"markdown"));
        assert!(formats.contains(&"html"));
    }

    #[test]
    fn test_read_document() {
        let options = Options::default();
        let doc = read_document("# Test", "markdown", &options).unwrap();
        let root = doc.arena.get(doc.root);
        assert!(matches!(root.value, crate::nodes::NodeValue::Document));
    }

    #[test]
    fn test_read_document_unknown_format() {
        let options = Options::default();
        let result = read_document("# Test", "unknown", &options);
        assert!(result.is_err());
    }
}
