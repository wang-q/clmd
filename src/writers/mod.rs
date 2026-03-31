//! Document writers for various output formats.
//!
//! This module provides a unified interface for writing documents to different
//! formats, inspired by Pandoc's Writer system. Writers convert the internal AST
//! representation to the target format.
//!
//! # Example
//!
//! ```ignore
//! use clmd::writers::{WriterRegistry, Writer};
//! use clmd::{Options, parse_document};
//!
//! let registry = WriterRegistry::new();
//! let writer = registry.get("html").unwrap();
//!
//! let options = Options::default();
//! let (arena, root) = parse_document("# Hello World", &options);
//! let output = writer.write(&arena, root, &options).unwrap();
//! ```

use crate::arena::{NodeArena, NodeId};
use crate::error::{ClmdError, ClmdResult};
use crate::options::Options;
use std::collections::HashMap;
use std::fmt::Debug;

/// A document writer that can render AST to a specific format.
///
/// Writers are responsible for converting the internal AST representation
/// into the target output format.
///
/// # Example
///
/// ```ignore
/// use clmd::writers::Writer;
/// use clmd::{Options, parse_document};
///
/// fn use_writer<W: Writer>(writer: &W, input: &str) {
///     let options = Options::default();
///     let (arena, root) = parse_document(input, &options);
///     let output = writer.write(&arena, root, &options).unwrap();
///     println!("{}", output);
/// }
/// ```ignore
pub trait Writer: Send + Sync + Debug {
    /// Write the AST to the output format.
    ///
    /// # Arguments
    ///
    /// * `arena` - The arena containing the AST nodes
    /// * `root` - The root node ID
    /// * `options` - Rendering options
    ///
    /// # Returns
    ///
    /// The rendered output as a string on success, or an error on failure.
    fn write(
        &self,
        arena: &NodeArena,
        root: NodeId,
        options: &Options,
    ) -> ClmdResult<String>;

    /// Get the format name this writer supports.
    fn format(&self) -> &'static str;

    /// Get the file extensions this writer can handle.
    fn extensions(&self) -> &[&'static str];

    /// Check if this writer supports a specific file extension.
    fn supports_extension(&self, ext: &str) -> bool {
        self.extensions().contains(&ext.to_lowercase().as_str())
    }

    /// Get the MIME type for this format.
    fn mime_type(&self) -> &'static str;
}

/// A registry of available document writers.
///
/// The registry allows dynamic lookup of writers by format name or file extension.
/// It supports registering custom writers at runtime.
///
/// # Example
///
/// ```ignore
/// use clmd::writers::WriterRegistry;
///
/// let mut registry = WriterRegistry::new();
///
/// // Get a writer by format name
/// if let Some(writer) = registry.get("html") {
///     println!("Found writer for HTML");
/// }
///
/// // Get a writer by file extension
/// if let Some(writer) = registry.get_by_extension("html") {
///     println!("Found writer for .html files");
/// }
/// ```ignore
#[derive(Debug, Default)]
pub struct WriterRegistry {
    writers: HashMap<&'static str, Box<dyn Writer>>,
    extension_map: HashMap<&'static str, &'static str>,
}

impl WriterRegistry {
    /// Create a new registry with default writers.
    pub fn new() -> Self {
        let mut registry = Self::empty();
        registry.register_default_writers();
        registry
    }

    /// Create an empty registry.
    pub fn empty() -> Self {
        Self {
            writers: HashMap::new(),
            extension_map: HashMap::new(),
        }
    }

    /// Register a writer.
    ///
    /// # Arguments
    ///
    /// * `writer` - The writer to register
    ///
    /// # Example
    ///
    /// ```
    /// use clmd::writers::{WriterRegistry, HtmlWriter};
    ///
    /// let mut registry = WriterRegistry::empty();
    /// registry.register(Box::new(HtmlWriter));
    /// ```
    pub fn register(&mut self, writer: Box<dyn Writer>) {
        let format = writer.format();

        // Register extensions
        for ext in writer.extensions() {
            self.extension_map.insert(ext, format);
        }

        // Register the writer
        self.writers.insert(format, writer);
    }

    /// Get a writer by format name.
    ///
    /// # Arguments
    ///
    /// * `format` - The format name (e.g., "html", "commonmark")
    ///
    /// # Returns
    ///
    /// Some(writer) if found, None otherwise.
    pub fn get(&self, format: &str) -> Option<&dyn Writer> {
        self.writers.get(format).map(|w| w.as_ref())
    }

    /// Get a writer by file extension.
    ///
    /// # Arguments
    ///
    /// * `extension` - The file extension (e.g., "html", "md")
    ///
    /// # Returns
    ///
    /// Some(writer) if found, None otherwise.
    pub fn get_by_extension(&self, extension: &str) -> Option<&dyn Writer> {
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
        self.writers.keys().copied().collect()
    }

    /// Get all registered file extensions.
    pub fn extensions(&self) -> Vec<&'static str> {
        self.extension_map.keys().copied().collect()
    }

    /// Check if a format is supported.
    pub fn supports_format(&self, format: &str) -> bool {
        self.writers.contains_key(format)
    }

    /// Check if an extension is supported.
    pub fn supports_extension(&self, extension: &str) -> bool {
        let ext = extension.to_lowercase();
        self.extension_map.contains_key(ext.as_str())
    }

    /// Register default writers.
    fn register_default_writers(&mut self) {
        self.register(Box::new(HtmlWriter));
        self.register(Box::new(CommonMarkWriter));
        self.register(Box::new(XmlWriter));
    }
}

impl Clone for WriterRegistry {
    fn clone(&self) -> Self {
        // Create a new registry with default writers
        // This is a limitation - custom writers won't be cloned
        Self::new()
    }
}

/// HTML document writer.
///
/// Renders documents to HTML format.
#[derive(Debug, Clone, Copy)]
pub struct HtmlWriter;

impl Writer for HtmlWriter {
    fn write(
        &self,
        arena: &NodeArena,
        root: NodeId,
        options: &Options,
    ) -> ClmdResult<String> {
        let mut output = String::new();
        crate::format_html(arena, root, options, &mut output)
            .map_err(|e| ClmdError::io_error(format!("HTML formatting error: {}", e)))?;
        Ok(output)
    }

    fn format(&self) -> &'static str {
        "html"
    }

    fn extensions(&self) -> &[&'static str] {
        &["html", "htm"]
    }

    fn mime_type(&self) -> &'static str {
        "text/html"
    }
}

/// CommonMark document writer.
///
/// Renders documents to CommonMark (Markdown) format.
#[derive(Debug, Clone, Copy)]
pub struct CommonMarkWriter;

impl Writer for CommonMarkWriter {
    fn write(
        &self,
        arena: &NodeArena,
        root: NodeId,
        options: &Options,
    ) -> ClmdResult<String> {
        let mut output = String::new();
        crate::format_commonmark(arena, root, options, &mut output).map_err(|e| {
            ClmdError::io_error(format!("CommonMark formatting error: {}", e))
        })?;
        Ok(output)
    }

    fn format(&self) -> &'static str {
        "commonmark"
    }

    fn extensions(&self) -> &[&'static str] {
        &["md", "markdown", "mkd", "mdown"]
    }

    fn mime_type(&self) -> &'static str {
        "text/markdown"
    }
}

/// XML document writer.
///
/// Renders documents to CommonMark XML format.
#[derive(Debug, Clone, Copy)]
pub struct XmlWriter;

impl Writer for XmlWriter {
    fn write(
        &self,
        arena: &NodeArena,
        root: NodeId,
        options: &Options,
    ) -> ClmdResult<String> {
        let mut output = String::new();
        crate::format_xml(arena, root, options, &mut output)
            .map_err(|e| ClmdError::io_error(format!("XML formatting error: {}", e)))?;
        Ok(output)
    }

    fn format(&self) -> &'static str {
        "xml"
    }

    fn extensions(&self) -> &[&'static str] {
        &["xml"]
    }

    fn mime_type(&self) -> &'static str {
        "application/xml"
    }
}

/// Write a document to a string.
///
/// # Arguments
///
/// * `arena` - The arena containing the AST nodes
/// * `root` - The root node ID
/// * `format` - The output format
/// * `options` - Rendering options
///
/// # Returns
///
/// The rendered output as a string on success, or an error on failure.
///
/// # Example
///
/// ```ignore
/// use clmd::writers::write_document;
/// use clmd::{Options, parse_document};
///
/// let options = Options::default();
/// let (arena, root) = parse_document("# Hello", &options);
/// let output = write_document(&arena, root, "html", &options).unwrap();
/// assert!(output.contains("<h1>"));
/// ```ignore
pub fn write_document(
    arena: &NodeArena,
    root: NodeId,
    format: &str,
    options: &Options,
) -> ClmdResult<String> {
    let registry = WriterRegistry::new();

    let writer = registry
        .get(format)
        .ok_or_else(|| ClmdError::unknown_writer(format))?;

    writer.write(arena, root, options)
}

/// Write a document to a file.
///
/// # Arguments
///
/// * `arena` - The arena containing the AST nodes
/// * `root` - The root node ID
/// * `path` - The output file path
/// * `format` - Optional format override (if None, detects from extension)
/// * `options` - Rendering options
///
/// # Returns
///
/// Ok on success, or an error on failure.
///
/// # Example
///
/// ```ignore
/// use clmd::writers::write_file;
/// use clmd::{Options, parse_document};
///
/// let options = Options::default();
/// let (arena, root) = parse_document("# Hello", &options);
/// write_file(&arena, root, "output.html", None, &options).unwrap();
/// ```ignore
pub fn write_file(
    arena: &NodeArena,
    root: NodeId,
    path: &std::path::Path,
    format: Option<&str>,
    options: &Options,
) -> ClmdResult<()> {
    use std::fs;

    // Detect format from extension if not specified
    let format = match format {
        Some(f) => f,
        None => {
            let registry = WriterRegistry::new();
            registry
                .detect_format(path)
                .ok_or_else(|| ClmdError::unknown_writer("unknown"))?
        }
    };

    let content = write_document(arena, root, format, options)?;

    fs::write(path, content)
        .map_err(|e| ClmdError::io_error(format!("Failed to write file: {}", e)))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_writer() {
        let writer = HtmlWriter;
        let options = Options::default();
        let (arena, root) = crate::parse_document("# Hello", &options);

        let output = writer.write(&arena, root, &options).unwrap();
        assert!(output.contains("<h1>"));
    }

    #[test]
    fn test_commonmark_writer() {
        let writer = CommonMarkWriter;
        let options = Options::default();
        let (arena, root) = crate::parse_document("# Hello", &options);

        let output = writer.write(&arena, root, &options).unwrap();
        assert!(output.contains("# Hello"));
    }

    #[test]
    fn test_xml_writer() {
        let writer = XmlWriter;
        let options = Options::default();
        let (arena, root) = crate::parse_document("# Hello", &options);

        let output = writer.write(&arena, root, &options).unwrap();
        assert!(output.contains("<?xml"));
    }

    #[test]
    fn test_writer_registry() {
        let registry = WriterRegistry::new();

        assert!(registry.supports_format("html"));
        assert!(registry.supports_format("commonmark"));
        assert!(registry.supports_format("xml"));
        assert!(!registry.supports_format("pdf"));

        assert!(registry.supports_extension("html"));
        assert!(registry.supports_extension("md"));
        assert!(!registry.supports_extension("pdf"));
    }

    #[test]
    fn test_registry_get() {
        let registry = WriterRegistry::new();

        let writer = registry.get("html");
        assert!(writer.is_some());
        assert_eq!(writer.unwrap().format(), "html");

        let writer = registry.get("unknown");
        assert!(writer.is_none());
    }

    #[test]
    fn test_registry_get_by_extension() {
        let registry = WriterRegistry::new();

        let writer = registry.get_by_extension("html");
        assert!(writer.is_some());

        let writer = registry.get_by_extension("unknown");
        assert!(writer.is_none());
    }

    #[test]
    fn test_detect_format() {
        let registry = WriterRegistry::new();

        let path = std::path::Path::new("test.html");
        assert_eq!(registry.detect_format(path), Some("html"));

        let path = std::path::Path::new("test.md");
        assert_eq!(registry.detect_format(path), Some("commonmark"));

        let path = std::path::Path::new("test");
        assert_eq!(registry.detect_format(path), None);
    }

    #[test]
    fn test_write_document() {
        let options = Options::default();
        let (arena, root) = crate::parse_document("# Test", &options);

        let output = write_document(&arena, root, "html", &options).unwrap();
        assert!(output.contains("<h1>"));
    }

    #[test]
    fn test_write_document_unknown_format() {
        let options = Options::default();
        let (arena, root) = crate::parse_document("# Test", &options);

        let result = write_document(&arena, root, "unknown", &options);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown writer"));
    }
}
