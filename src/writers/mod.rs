//! Document writers for various output formats.
//!
//! This module provides a unified interface for writing documents to different
//! formats, inspired by Pandoc's Writer system.
//!
//! # Example
//!
//! ```
//! use clmd::{parse_document, Options};
//! use clmd::writers::{Writer, HtmlWriter};
//!
//! let (arena, root) = parse_document("# Hello", &Options::default());
//! let writer = HtmlWriter::new();
//! let output = writer.write(&arena, root, &Options::default()).unwrap();
//! assert!(output.contains("<h1>"));
//! ```

use crate::arena::{NodeArena, NodeId};
use crate::error::{ParseError, ParseResult};
use crate::options::Options;
use std::collections::HashMap;
use std::fmt;

/// Error type for writer operations.
#[derive(Debug, Clone)]
pub enum WriterError {
    /// Formatting error
    FormatError(String),
    /// Unsupported feature
    UnsupportedFeature(String),
    /// IO error
    IoError(String),
}

impl fmt::Display for WriterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WriterError::FormatError(msg) => write!(f, "Format error: {}", msg),
            WriterError::UnsupportedFeature(feature) => {
                write!(f, "Unsupported feature: {}", feature)
            }
            WriterError::IoError(msg) => write!(f, "IO error: {}", msg),
        }
    }
}

impl std::error::Error for WriterError {}

/// Result type for writer operations.
pub type WriterResult<T> = Result<T, WriterError>;

/// Trait for document writers.
///
/// Implement this trait to add support for new output formats.
pub trait Writer {
    /// Write a document to the output format.
    ///
    /// # Arguments
    ///
    /// * `arena` - The arena containing AST nodes
    /// * `root` - The root node ID
    /// * `options` - Rendering options
    ///
    /// # Returns
    ///
    /// The output string, or a `WriterError` if rendering fails.
    fn write(
        &self,
        arena: &NodeArena,
        root: NodeId,
        options: &Options,
    ) -> WriterResult<String>;

    /// Get the format name this writer supports.
    fn format_name(&self) -> &'static str;
}

/// Writer for HTML format.
#[derive(Debug, Clone, Copy)]
pub struct HtmlWriter;

impl HtmlWriter {
    /// Create a new HTML writer.
    pub fn new() -> Self {
        Self
    }
}

impl Default for HtmlWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl Writer for HtmlWriter {
    fn write(
        &self,
        arena: &NodeArena,
        root: NodeId,
        options: &Options,
    ) -> WriterResult<String> {
        let flags = crate::parser::options_to_flags(options);
        let html = crate::render::html::render(arena, root, flags);
        Ok(html)
    }

    fn format_name(&self) -> &'static str {
        "html"
    }
}

/// Writer for XML format.
#[derive(Debug, Clone, Copy)]
pub struct XmlWriter;

impl XmlWriter {
    /// Create a new XML writer.
    pub fn new() -> Self {
        Self
    }
}

impl Default for XmlWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl Writer for XmlWriter {
    fn write(
        &self,
        arena: &NodeArena,
        root: NodeId,
        options: &Options,
    ) -> WriterResult<String> {
        let mut output = String::new();
        crate::format_xml(arena, root, options, &mut output)
            .map_err(|e| WriterError::FormatError(e.to_string()))?;
        Ok(output)
    }

    fn format_name(&self) -> &'static str {
        "xml"
    }
}

/// Writer for CommonMark format.
#[derive(Debug, Clone, Copy)]
pub struct CommonMarkWriter;

impl CommonMarkWriter {
    /// Create a new CommonMark writer.
    pub fn new() -> Self {
        Self
    }
}

impl Default for CommonMarkWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl Writer for CommonMarkWriter {
    fn write(
        &self,
        arena: &NodeArena,
        root: NodeId,
        options: &Options,
    ) -> WriterResult<String> {
        let mut output = String::new();
        crate::format_commonmark(arena, root, options, &mut output)
            .map_err(|e| WriterError::FormatError(e.to_string()))?;
        Ok(output)
    }

    fn format_name(&self) -> &'static str {
        "commonmark"
    }
}

/// Registry of available writers.
pub struct WriterRegistry {
    writers: HashMap<String, Box<dyn Writer>>,
}

impl std::fmt::Debug for WriterRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WriterRegistry")
            .field("formats", &self.supported_formats())
            .finish()
    }
}

impl WriterRegistry {
    /// Create a new writer registry with default writers.
    pub fn new() -> Self {
        let mut registry = Self {
            writers: HashMap::new(),
        };
        registry.register(Box::new(HtmlWriter::new()));
        registry.register(Box::new(XmlWriter::new()));
        registry.register(Box::new(CommonMarkWriter::new()));
        registry
    }

    /// Register a new writer.
    pub fn register(&mut self, writer: Box<dyn Writer>) {
        let name = writer.format_name().to_string();
        self.writers.insert(name, writer);
    }

    /// Get a writer by format name.
    pub fn get(&self, format: &str) -> Option<&dyn Writer> {
        self.writers.get(format).map(|w| w.as_ref())
    }

    /// Check if a format is supported.
    pub fn supports(&self, format: &str) -> bool {
        self.writers.contains_key(format)
    }

    /// Get a list of supported formats.
    pub fn supported_formats(&self) -> Vec<&str> {
        self.writers.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for WriterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Write a document using the specified format.
///
/// # Arguments
///
/// * `arena` - The arena containing AST nodes
/// * `root` - The root node ID
/// * `format` - The output format (e.g., "html", "xml")
/// * `options` - Rendering options
///
/// # Returns
///
/// The output string, or a `WriterError` if the format is not supported.
pub fn write_document(
    arena: &NodeArena,
    root: NodeId,
    format: &str,
    options: &Options,
) -> WriterResult<String> {
    let registry = WriterRegistry::new();
    let writer = registry.get(format).ok_or_else(|| {
        WriterError::FormatError(format!("Unknown writer format: {}", format))
    })?;
    writer.write(arena, root, options)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_writer() {
        let (arena, root) =
            crate::parser::parse_document("# Hello", &Options::default());
        let writer = HtmlWriter::new();
        let output = writer.write(&arena, root, &Options::default()).unwrap();

        assert!(output.contains("<h1>"));
        assert!(output.contains("Hello"));
    }

    #[test]
    fn test_xml_writer() {
        let (arena, root) =
            crate::parser::parse_document("# Hello", &Options::default());
        let writer = XmlWriter::new();
        let output = writer.write(&arena, root, &Options::default()).unwrap();

        assert!(output.contains("<?xml"));
        assert!(output.contains("<document>"));
    }

    #[test]
    fn test_commonmark_writer() {
        let (arena, root) =
            crate::parser::parse_document("# Hello", &Options::default());
        let writer = CommonMarkWriter::new();
        let output = writer.write(&arena, root, &Options::default()).unwrap();

        assert!(output.contains("# Hello"));
    }

    #[test]
    fn test_writer_registry() {
        let registry = WriterRegistry::new();
        assert!(registry.supports("html"));
        assert!(registry.supports("xml"));
        assert!(registry.supports("commonmark"));
        assert!(!registry.supports("unknown"));

        let formats = registry.supported_formats();
        assert!(formats.contains(&"html"));
        assert!(formats.contains(&"xml"));
        assert!(formats.contains(&"commonmark"));
    }

    #[test]
    fn test_write_document() {
        let (arena, root) = crate::parser::parse_document("# Test", &Options::default());
        let output = write_document(&arena, root, "html", &Options::default()).unwrap();
        assert!(output.contains("<h1>"));
    }

    #[test]
    fn test_write_document_unknown_format() {
        let (arena, root) = crate::parser::parse_document("# Test", &Options::default());
        let result = write_document(&arena, root, "unknown", &Options::default());
        assert!(result.is_err());
    }
}
