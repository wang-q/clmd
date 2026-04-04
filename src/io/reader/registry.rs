//! Reader registry for managing document format readers.
//!
//! This module provides a centralized registry for document readers,
//! allowing formats to be looked up by name or file extension.
//!
//! # Example
//!
//! ```ignore
//! use clmd::io::reader::{ReaderRegistry, Reader, ReaderOptions};
//!
//! let registry = ReaderRegistry::new();
//!
//! // Get a reader by name
//! if let Some(reader) = registry.get("markdown") {
//!     println!("Found reader: {}", reader.format());
//! }
//!
//! // Get a reader by file extension
//! if let Some(reader) = registry.get_by_extension("md") {
//!     println!("Found reader for .md files");
//! }
//! ```

use std::collections::HashMap;
use std::fmt::Debug;

use crate::core::arena::{NodeArena, NodeId};
use crate::core::error::ClmdResult;
use crate::options::{InputFormat, ReaderOptions};
use crate::parse;

/// A boxed reader trait object.
pub type BoxedReader = Box<dyn Reader>;

/// A document reader that can parse input into an AST.
///
/// Readers are responsible for converting input data from a specific format
/// into the internal AST representation used by clmd.
pub trait Reader: Send + Sync + Debug {
    /// Read input and parse it into an AST.
    ///
    /// # Arguments
    ///
    /// * `input` - The input string to parse
    /// * `options` - Parsing options
    ///
    /// # Returns
    ///
    /// A tuple of (arena, root_node_id) on success, or an error on failure.
    fn read(
        &self,
        input: &str,
        options: &ReaderOptions,
    ) -> ClmdResult<(NodeArena, NodeId)>;

    /// Get the format name this reader supports.
    fn format(&self) -> &'static str;

    /// Get the file extensions this reader can handle.
    fn extensions(&self) -> &[&'static str];

    /// Check if this reader supports a specific file extension.
    fn supports_extension(&self, ext: &str) -> bool {
        self.extensions().contains(&ext.to_lowercase().as_str())
    }

    /// Get the input format this reader handles.
    fn input_format(&self) -> InputFormat;
}

/// A registry of available document readers.
///
/// The registry allows dynamic lookup of readers by format name or file extension.
/// It supports registering custom readers at runtime.
#[derive(Debug, Default)]
pub struct ReaderRegistry {
    readers: HashMap<String, BoxedReader>,
    extension_map: HashMap<String, String>,
}

impl ReaderRegistry {
    /// Create a new registry with default readers.
    pub fn new() -> Self {
        let mut registry = Self::empty();
        registry.register_default_readers();
        registry
    }

    /// Create an empty registry.
    pub fn empty() -> Self {
        Self {
            readers: HashMap::new(),
            extension_map: HashMap::new(),
        }
    }

    /// Register a reader.
    ///
    /// # Arguments
    ///
    /// * `reader` - The reader to register
    pub fn register(&mut self, reader: BoxedReader) {
        let format = reader.format();

        // Register extensions
        for ext in reader.extensions() {
            self.extension_map.insert(ext.to_lowercase(), format.to_string());
        }

        // Register the reader
        self.readers.insert(format.to_string(), reader);
    }

    /// Get a reader by format name.
    ///
    /// # Arguments
    ///
    /// * `format` - The format name (e.g., "markdown", "html")
    ///
    /// # Returns
    ///
    /// Some(reader) if found, None otherwise.
    pub fn get(&self, format: &str) -> Option<&dyn Reader> {
        self.readers.get(format).map(|r| r.as_ref())
    }

    /// Get a reader by file extension.
    ///
    /// # Arguments
    ///
    /// * `extension` - The file extension (e.g., "md", "html")
    ///
    /// # Returns
    ///
    /// Some(reader) if found, None otherwise.
    pub fn get_by_extension(&self, extension: &str) -> Option<&dyn Reader> {
        let ext = extension.to_lowercase();
        self.extension_map
            .get(&ext)
            .and_then(|format| self.get(format))
    }

    /// Detect the format from a file path.
    ///
    /// # Arguments
    ///
    /// * `path` - The file path
    ///
    /// # Returns
    ///
    /// Some(format_name) if detected, None otherwise.
    pub fn detect_format(&self, path: &std::path::Path) -> Option<&str> {
        path.extension()
            .and_then(|e| e.to_str())
            .and_then(|ext| self.extension_map.get(ext).map(|s| s.as_str()))
    }

    /// Get all registered format names.
    pub fn formats(&self) -> Vec<&str> {
        self.readers.keys().map(|s| s.as_str()).collect()
    }

    /// Get all registered file extensions.
    pub fn extensions(&self) -> Vec<&str> {
        self.extension_map.keys().map(|s| s.as_str()).collect()
    }

    /// Check if a format is supported.
    pub fn supports_format(&self, format: &str) -> bool {
        self.readers.contains_key(format)
    }

    /// Check if an extension is supported.
    pub fn supports_extension(&self, extension: &str) -> bool {
        let ext = extension.to_lowercase();
        self.extension_map.contains_key(&ext)
    }

    /// Register default readers.
    fn register_default_readers(&mut self) {
        self.register(Box::new(MarkdownReader));
        self.register(Box::new(HtmlReader));
        self.register(Box::new(BibTeXReader));
        self.register(Box::new(LaTeXReader));
    }
}

impl Clone for ReaderRegistry {
    fn clone(&self) -> Self {
        // Create a new registry with default readers
        // This is a limitation - custom readers won't be cloned
        Self::new()
    }
}

/// Markdown document reader.
///
/// Reads CommonMark and GFM formatted Markdown.
#[derive(Debug, Clone, Copy)]
pub struct MarkdownReader;

impl Reader for MarkdownReader {
    fn read(
        &self,
        input: &str,
        options: &ReaderOptions,
    ) -> ClmdResult<(NodeArena, NodeId)> {
        // Convert ReaderOptions to parser Options
        let parser_options = options.to_parser_options();
        Ok(parse::parse_document(input, &parser_options))
    }

    fn format(&self) -> &'static str {
        "markdown"
    }

    fn extensions(&self) -> &[&'static str] {
        &["md", "markdown", "mkd", "mdown"]
    }

    fn input_format(&self) -> InputFormat {
        InputFormat::Markdown
    }
}

/// HTML document reader.
///
/// Reads HTML and converts to Markdown AST.
#[derive(Debug, Clone, Copy)]
pub struct HtmlReader;

impl Reader for HtmlReader {
    fn read(
        &self,
        input: &str,
        options: &ReaderOptions,
    ) -> ClmdResult<(NodeArena, NodeId)> {
        // Use the HTML reader from the html module
        super::html::HtmlReader.read(input, options)
    }

    fn format(&self) -> &'static str {
        "html"
    }

    fn extensions(&self) -> &[&'static str] {
        &["html", "htm"]
    }

    fn input_format(&self) -> InputFormat {
        InputFormat::Html
    }
}

/// BibTeX document reader.
///
/// Reads BibTeX bibliography files.
#[derive(Debug, Clone, Copy)]
pub struct BibTeXReader;

impl Reader for BibTeXReader {
    fn read(
        &self,
        input: &str,
        _options: &ReaderOptions,
    ) -> ClmdResult<(NodeArena, NodeId)> {
        // Use the BibTeX reader from the bibtex module
        super::bibtex::BibTeXReader.read(input, _options)
    }

    fn format(&self) -> &'static str {
        "bibtex"
    }

    fn extensions(&self) -> &[&'static str] {
        &["bib", "bibtex"]
    }

    fn input_format(&self) -> InputFormat {
        InputFormat::Bibtex
    }
}

/// LaTeX document reader.
///
/// Reads LaTeX documents and converts to Markdown AST.
#[derive(Debug, Clone, Copy)]
pub struct LaTeXReader;

impl Reader for LaTeXReader {
    fn read(
        &self,
        input: &str,
        options: &ReaderOptions,
    ) -> ClmdResult<(NodeArena, NodeId)> {
        // Use the LaTeX reader from the latex module
        super::latex::LaTeXReader.read(input, options)
    }

    fn format(&self) -> &'static str {
        "latex"
    }

    fn extensions(&self) -> &[&'static str] {
        &["tex", "latex"]
    }

    fn input_format(&self) -> InputFormat {
        InputFormat::Latex
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nodes::NodeValue;

    #[test]
    fn test_registry_new() {
        let registry = ReaderRegistry::new();
        assert!(!registry.formats().is_empty());
    }

    #[test]
    fn test_registry_empty() {
        let registry = ReaderRegistry::empty();
        assert!(registry.formats().is_empty());
    }

    #[test]
    fn test_markdown_reader() {
        let reader = MarkdownReader;
        let options = ReaderOptions::default();

        let (arena, root) = reader.read("# Hello", &options).unwrap();
        let node = arena.get(root);
        assert!(matches!(node.value, NodeValue::Document));
    }

    #[test]
    fn test_html_reader() {
        let reader = HtmlReader;
        let options = ReaderOptions::default();

        let (arena, root) = reader.read("<h1>Hello</h1>", &options).unwrap();
        let node = arena.get(root);
        assert!(matches!(node.value, NodeValue::Document));
    }

    #[test]
    fn test_reader_registry() {
        let registry = ReaderRegistry::new();

        assert!(registry.supports_format("markdown"));
        assert!(registry.supports_format("html"));
        assert!(!registry.supports_format("pdf"));

        assert!(registry.supports_extension("md"));
        assert!(registry.supports_extension("html"));
        assert!(!registry.supports_extension("pdf"));
    }

    #[test]
    fn test_registry_get() {
        let registry = ReaderRegistry::new();

        let reader = registry.get("markdown");
        assert!(reader.is_some());
        assert_eq!(reader.unwrap().format(), "markdown");

        let reader = registry.get("unknown");
        assert!(reader.is_none());
    }

    #[test]
    fn test_registry_get_by_extension() {
        let registry = ReaderRegistry::new();

        let reader = registry.get_by_extension("md");
        assert!(reader.is_some());

        let reader = registry.get_by_extension("unknown");
        assert!(reader.is_none());
    }

    #[test]
    fn test_detect_format() {
        let registry = ReaderRegistry::new();

        let path = std::path::Path::new("test.md");
        assert_eq!(registry.detect_format(path), Some("markdown"));

        let path = std::path::Path::new("test.html");
        assert_eq!(registry.detect_format(path), Some("html"));

        let path = std::path::Path::new("test");
        assert_eq!(registry.detect_format(path), None);
    }

    #[test]
    fn test_reader_input_format() {
        let markdown_reader = MarkdownReader;
        assert_eq!(markdown_reader.input_format(), InputFormat::Markdown);

        let html_reader = HtmlReader;
        assert_eq!(html_reader.input_format(), InputFormat::Html);
    }
}
