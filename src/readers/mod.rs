//! Document readers for various input formats.
//!
//! This module provides a unified interface for reading documents from different
//! formats, inspired by Pandoc's Reader system. Readers convert input data into
//! the internal AST representation.
//!
//! # Example
//!
//! ```
//! use clmd::readers::{ReaderRegistry, Reader};
//! use clmd::Options;
//!
//! let registry = ReaderRegistry::new();
//! let reader = registry.get("markdown").unwrap();
//!
//! let options = Options::default();
//! let (arena, root) = reader.read("# Hello World", &options).unwrap();
//! ```

use crate::arena::NodeArena;
use crate::error::{ClmdError, ClmdResult};
use crate::options::Options;
use crate::parser;
use std::collections::HashMap;
use std::fmt::Debug;

/// A document reader that can parse input into an AST.
///
/// Readers are responsible for converting input data from a specific format
/// into the internal AST representation used by clmd.
///
/// # Example
///
/// ```ignore
/// use clmd::readers::Reader;
/// use clmd::Options;
///
/// fn use_reader<R: Reader>(reader: &R, input: &str) {
///     let options = Options::default();
///     let (arena, root) = reader.read(input, &options).unwrap();
///     // Process the AST...
/// }
/// ```ignore
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
        options: &Options,
    ) -> ClmdResult<(NodeArena, crate::arena::NodeId)>;

    /// Get the format name this reader supports.
    fn format(&self) -> &'static str;

    /// Get the file extensions this reader can handle.
    fn extensions(&self) -> &[&'static str];

    /// Check if this reader supports a specific file extension.
    fn supports_extension(&self, ext: &str) -> bool {
        self.extensions().contains(&ext.to_lowercase().as_str())
    }
}

/// A registry of available document readers.
///
/// The registry allows dynamic lookup of readers by format name or file extension.
/// It supports registering custom readers at runtime.
///
/// # Example
///
/// ```ignore
/// use clmd::readers::ReaderRegistry;
///
/// let mut registry = ReaderRegistry::new();
///
/// // Get a reader by format name
/// if let Some(reader) = registry.get("markdown") {
///     println!("Found reader for markdown");
/// }
///
/// // Get a reader by file extension
/// if let Some(reader) = registry.get_by_extension("md") {
///     println!("Found reader for .md files");
/// }
/// ```ignore
#[derive(Debug, Default)]
pub struct ReaderRegistry {
    readers: HashMap<&'static str, Box<dyn Reader>>,
    extension_map: HashMap<&'static str, &'static str>,
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
    ///
    /// # Example
    ///
    /// ```
    /// use clmd::readers::{ReaderRegistry, MarkdownReader};
    ///
    /// let mut registry = ReaderRegistry::empty();
    /// registry.register(Box::new(MarkdownReader));
    /// ```
    pub fn register(&mut self, reader: Box<dyn Reader>) {
        let format = reader.format();

        // Register extensions
        for ext in reader.extensions() {
            self.extension_map.insert(ext, format);
        }

        // Register the reader
        self.readers.insert(format, reader);
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
            .get(ext.as_str())
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
    pub fn detect_format(&self, path: &std::path::Path) -> Option<&'static str> {
        path.extension()
            .and_then(|e| e.to_str())
            .and_then(|ext| self.extension_map.get(ext).copied())
    }

    /// Get all registered format names.
    pub fn formats(&self) -> Vec<&'static str> {
        self.readers.keys().copied().collect()
    }

    /// Get all registered file extensions.
    pub fn extensions(&self) -> Vec<&'static str> {
        self.extension_map.keys().copied().collect()
    }

    /// Check if a format is supported.
    pub fn supports_format(&self, format: &str) -> bool {
        self.readers.contains_key(format)
    }

    /// Check if an extension is supported.
    pub fn supports_extension(&self, extension: &str) -> bool {
        let ext = extension.to_lowercase();
        self.extension_map.contains_key(ext.as_str())
    }

    /// Register default readers.
    fn register_default_readers(&mut self) {
        self.register(Box::new(MarkdownReader));
        self.register(Box::new(HtmlReader));
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
        options: &Options,
    ) -> ClmdResult<(NodeArena, crate::arena::NodeId)> {
        Ok(parser::parse_document(input, options))
    }

    fn format(&self) -> &'static str {
        "markdown"
    }

    fn extensions(&self) -> &[&'static str] {
        &["md", "markdown", "mkd", "mdown"]
    }
}

/// HTML document reader.
///
/// Reads HTML and converts it to Markdown AST.
#[derive(Debug, Clone, Copy)]
pub struct HtmlReader;

impl Reader for HtmlReader {
    fn read(
        &self,
        input: &str,
        _options: &Options,
    ) -> ClmdResult<(NodeArena, crate::arena::NodeId)> {
        // Convert HTML to Markdown, then parse
        let markdown = crate::from::html_to_markdown(input);
        Ok(parser::parse_document(&markdown, &Options::default()))
    }

    fn format(&self) -> &'static str {
        "html"
    }

    fn extensions(&self) -> &[&'static str] {
        &["html", "htm"]
    }
}

/// Read a document from a string with format detection.
///
/// # Arguments
///
/// * `input` - The input string
/// * `format` - Optional format hint (if None, attempts to detect from content)
/// * `options` - Parsing options
///
/// # Returns
///
/// A tuple of (arena, root_node_id) on success, or an error on failure.
///
/// # Example
///
/// ```ignore
/// use clmd::readers::read_document;
/// use clmd::Options;
///
/// let options = Options::default();
/// let (arena, root) = read_document("# Hello", Some("markdown"), &options).unwrap();
/// ```ignore
pub fn read_document(
    input: &str,
    format: Option<&str>,
    options: &Options,
) -> ClmdResult<(NodeArena, crate::arena::NodeId)> {
    let registry = ReaderRegistry::new();

    let format = format.unwrap_or("markdown");

    let reader = registry
        .get(format)
        .ok_or_else(|| ClmdError::unknown_reader(format))?;

    reader.read(input, options)
}

/// Read a document from a file.
///
/// # Arguments
///
/// * `path` - The file path
/// * `format` - Optional format override (if None, detects from extension)
/// * `options` - Parsing options
///
/// # Returns
///
/// A tuple of (arena, root_node_id) on success, or an error on failure.
///
/// # Example
///
/// ```ignore
/// use clmd::readers::read_file;
/// use clmd::Options;
///
/// let options = Options::default();
/// let (arena, root) = read_file("document.md", None, &options).unwrap();
/// ```ignore
pub fn read_file(
    path: &std::path::Path,
    format: Option<&str>,
    options: &Options,
) -> ClmdResult<(NodeArena, crate::arena::NodeId)> {
    use std::fs;

    let content = fs::read_to_string(path)
        .map_err(|e| ClmdError::io_error(format!("Failed to read file: {}", e)))?;

    // Detect format from extension if not specified
    let format = match format {
        Some(f) => Some(f),
        None => {
            let registry = ReaderRegistry::new();
            registry.detect_format(path)
        }
    };

    read_document(&content, format, options)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nodes::NodeValue;

    #[test]
    fn test_markdown_reader() {
        let reader = MarkdownReader;
        let options = Options::default();

        let (arena, root) = reader.read("# Hello", &options).unwrap();
        let node = arena.get(root);
        assert!(matches!(node.value, NodeValue::Document));
    }

    #[test]
    fn test_html_reader() {
        let reader = HtmlReader;
        let options = Options::default();

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
    fn test_read_document() {
        let options = Options::default();
        let (arena, root) = read_document("# Test", Some("markdown"), &options).unwrap();

        let node = arena.get(root);
        assert!(matches!(node.value, NodeValue::Document));
    }

    #[test]
    fn test_read_document_unknown_format() {
        let options = Options::default();
        let result = read_document("# Test", Some("unknown"), &options);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown reader"));
    }
}
