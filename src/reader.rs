//! Document reader trait and implementations for clmd.
//!
//! This module provides a unified interface for reading documents from different
//! formats, inspired by Pandoc's Reader system. Readers convert input data into
//! the internal AST representation.
//!
//! # Example
//!
//! ```
//! use clmd::reader::{Reader, MarkdownReader};
//! use clmd::clmd_options::{ClmdOptions, InputFormat};
//! use clmd::context::IoContext;
//!
//! let ctx = IoContext::new();
//! let options = ClmdOptions::default();
//! let reader = MarkdownReader::new();
//!
//! let (arena, root) = reader.read("# Hello World", &ctx, &options).unwrap();
//! ```

use crate::arena::{NodeArena, NodeId};
use crate::clmd_options::{ClmdOptions, InputFormat};
use crate::context::ClmdContext;
use crate::error::ClmdResult;
use std::fmt::Debug;

/// A document reader that can parse input into an AST.
///
/// Readers are responsible for converting input data from a specific format
/// into the internal AST representation used by clmd. This trait is designed
/// to work with the `ClmdContext` abstraction for IO operations.
///
/// # Example
///
/// ```
/// use clmd::reader::{Reader, MarkdownReader};
/// use clmd::clmd_options::ClmdOptions;
/// use clmd::context::IoContext;
///
/// fn use_reader(reader: &dyn Reader, input: &str) {
///     let ctx = IoContext::new();
///     let options = ClmdOptions::default();
///     let (arena, root) = reader.read(input, &ctx, &options).unwrap();
///     // Process the AST...
/// }
/// ```
pub trait Reader: Send + Sync + Debug {
    /// Read input and parse it into an AST.
    ///
    /// # Arguments
    ///
    /// * `input` - The input string to parse
    /// * `ctx` - The context for IO operations and logging
    /// * `options` - Parsing options
    ///
    /// # Returns
    ///
    /// A tuple of (arena, root_node_id) on success, or an error on failure.
    fn read(
        &self,
        input: &str,
        ctx: &dyn ClmdContext<Error = crate::error::ClmdError>,
        options: &ClmdOptions,
    ) -> ClmdResult<(NodeArena, NodeId)>;

    /// Read input from a file.
    ///
    /// This is a convenience method that reads the file content and then parses it.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the file to read
    /// * `ctx` - The context for IO operations and logging
    /// * `options` - Parsing options
    ///
    /// # Returns
    ///
    /// A tuple of (arena, root_node_id) on success, or an error on failure.
    fn read_file(
        &self,
        path: &std::path::Path,
        ctx: &dyn ClmdContext<Error = crate::error::ClmdError>,
        options: &ClmdOptions,
    ) -> ClmdResult<(NodeArena, NodeId)> {
        let content = ctx.read_file_to_string_dyn(path)?;
        self.read(&content, ctx, options)
    }

    /// Get the format name this reader supports.
    fn format(&self) -> InputFormat;

    /// Get the file extensions this reader can handle.
    fn extensions(&self) -> &[&'static str];

    /// Check if this reader supports a specific file extension.
    fn supports_extension(&self, ext: &str) -> bool {
        self.extensions().contains(&ext.to_lowercase().as_str())
    }

    /// Get the format name as a string.
    fn format_name(&self) -> &'static str {
        self.format().as_str()
    }
}

/// Markdown document reader.
///
/// Reads CommonMark and GFM formatted Markdown.
#[derive(Debug, Clone, Copy, Default)]
pub struct MarkdownReader;

impl MarkdownReader {
    /// Create a new Markdown reader.
    pub fn new() -> Self {
        Self
    }
}

impl Reader for MarkdownReader {
    fn read(
        &self,
        input: &str,
        ctx: &dyn ClmdContext<Error = crate::error::ClmdError>,
        options: &ClmdOptions,
    ) -> ClmdResult<(NodeArena, NodeId)> {
        ctx.info(&format!(
            "Reading Markdown document ({} bytes)",
            input.len()
        ));

        // Convert to legacy options for now
        let legacy_options = options.to_options();
        Ok(crate::parser::parse_document(input, &legacy_options))
    }

    fn format(&self) -> InputFormat {
        InputFormat::Markdown
    }

    fn extensions(&self) -> &[&'static str] {
        &["md", "markdown", "mkd", "mdown"]
    }
}

/// CommonMark document reader.
///
/// Reads strict CommonMark formatted Markdown.
#[derive(Debug, Clone, Copy, Default)]
pub struct CommonMarkReader;

impl CommonMarkReader {
    /// Create a new CommonMark reader.
    pub fn new() -> Self {
        Self
    }
}

impl Reader for CommonMarkReader {
    fn read(
        &self,
        input: &str,
        ctx: &dyn ClmdContext<Error = crate::error::ClmdError>,
        options: &ClmdOptions,
    ) -> ClmdResult<(NodeArena, NodeId)> {
        ctx.info(&format!(
            "Reading CommonMark document ({} bytes)",
            input.len()
        ));

        // Use strict CommonMark options
        let mut legacy_options = options.to_options();
        legacy_options.extension = crate::options::Extension::default();
        Ok(crate::parser::parse_document(input, &legacy_options))
    }

    fn format(&self) -> InputFormat {
        InputFormat::CommonMark
    }

    fn extensions(&self) -> &[&'static str] {
        &["md", "markdown", "cm"]
    }
}

/// GitHub Flavored Markdown reader.
///
/// Reads GFM formatted Markdown with all GFM extensions enabled.
#[derive(Debug, Clone, Copy, Default)]
pub struct GfmReader;

impl GfmReader {
    /// Create a new GFM reader.
    pub fn new() -> Self {
        Self
    }
}

impl Reader for GfmReader {
    fn read(
        &self,
        input: &str,
        ctx: &dyn ClmdContext<Error = crate::error::ClmdError>,
        options: &ClmdOptions,
    ) -> ClmdResult<(NodeArena, NodeId)> {
        ctx.info(&format!("Reading GFM document ({} bytes)", input.len()));

        // Use GFM options
        let mut legacy_options = options.to_options();
        legacy_options.extension = crate::options::Extension::default();
        legacy_options.extension.table = true;
        legacy_options.extension.strikethrough = true;
        legacy_options.extension.tasklist = true;
        legacy_options.extension.autolink = true;
        legacy_options.extension.tagfilter = true;

        Ok(crate::parser::parse_document(input, &legacy_options))
    }

    fn format(&self) -> InputFormat {
        InputFormat::Gfm
    }

    fn extensions(&self) -> &[&'static str] {
        &["md", "markdown", "gfm"]
    }
}

/// HTML document reader.
///
/// Reads HTML and converts it to the internal AST representation.
#[derive(Debug, Clone, Copy, Default)]
pub struct HtmlReader;

impl HtmlReader {
    /// Create a new HTML reader.
    pub fn new() -> Self {
        Self
    }
}

impl Reader for HtmlReader {
    fn read(
        &self,
        input: &str,
        ctx: &dyn ClmdContext<Error = crate::error::ClmdError>,
        _options: &ClmdOptions,
    ) -> ClmdResult<(NodeArena, NodeId)> {
        ctx.info(&format!("Reading HTML document ({} bytes)", input.len()));
        ctx.warn("HTML reader is experimental and may not handle all HTML constructs");

        // For now, use the existing HTML to Markdown conversion
        // In the future, this should have its own proper HTML parser
        let markdown = crate::from::html_to_markdown(input);
        let legacy_options = crate::options::Options::default();
        Ok(crate::parser::parse_document(&markdown, &legacy_options))
    }

    fn format(&self) -> InputFormat {
        InputFormat::Html
    }

    fn extensions(&self) -> &[&'static str] {
        &["html", "htm"]
    }
}

/// Reader registry for looking up readers by format or extension.
///
/// This registry provides a way to dynamically look up readers at runtime
/// based on format name or file extension.
#[derive(Debug, Default)]
pub struct ReaderRegistry {
    readers: Vec<Box<dyn Reader>>,
}

impl ReaderRegistry {
    /// Create a new registry with all default readers.
    pub fn new() -> Self {
        let mut registry = Self::empty();
        registry.register_default_readers();
        registry
    }

    /// Create an empty registry.
    pub fn empty() -> Self {
        Self {
            readers: Vec::new(),
        }
    }

    /// Register a reader.
    pub fn register(&mut self, reader: Box<dyn Reader>) {
        self.readers.push(reader);
    }

    /// Get a reader by format.
    pub fn get(&self, format: InputFormat) -> Option<&dyn Reader> {
        self.readers
            .iter()
            .find(|r| r.format() == format)
            .map(|r| r.as_ref())
    }

    /// Get a reader by format name.
    pub fn get_by_name(&self, name: &str) -> Option<&dyn Reader> {
        let format = name.parse::<InputFormat>().ok()?;
        self.get(format)
    }

    /// Get a reader by file extension.
    pub fn get_by_extension(&self, extension: &str) -> Option<&dyn Reader> {
        let ext = extension.to_lowercase();
        self.readers
            .iter()
            .find(|r| r.supports_extension(&ext))
            .map(|r| r.as_ref())
    }

    /// Detect format from a file path and return the appropriate reader.
    pub fn detect_from_path(&self, path: &std::path::Path) -> Option<&dyn Reader> {
        path.extension()
            .and_then(|e| e.to_str())
            .and_then(|ext| self.get_by_extension(ext))
    }

    /// Register default readers.
    fn register_default_readers(&mut self) {
        self.register(Box::new(MarkdownReader::new()));
        self.register(Box::new(CommonMarkReader::new()));
        self.register(Box::new(GfmReader::new()));
        self.register(Box::new(HtmlReader::new()));
    }
}

impl Clone for ReaderRegistry {
    fn clone(&self) -> Self {
        // Create a new registry with default readers
        Self::new()
    }
}

/// Helper function to read a document with automatic format detection.
///
/// # Arguments
///
/// * `input` - The input string to parse
/// * `ctx` - The context for IO operations
/// * `options` - Parsing options (format is used to select reader)
///
/// # Returns
///
/// A tuple of (arena, root_node_id) on success, or an error on failure.
pub fn read_document(
    input: &str,
    ctx: &dyn ClmdContext<Error = crate::error::ClmdError>,
    options: &ClmdOptions,
) -> ClmdResult<(NodeArena, NodeId)> {
    let registry = ReaderRegistry::new();

    let reader = registry.get(options.input_format).ok_or_else(|| {
        crate::error::ClmdError::io_error(format!(
            "No reader available for format: {:?}",
            options.input_format
        ))
    })?;

    reader.read(input, ctx, options)
}

/// Helper function to read a document from a file with automatic format detection.
///
/// # Arguments
///
/// * `path` - The path to the file
/// * `ctx` - The context for IO operations
/// * `options` - Parsing options (can be overridden by file extension)
///
/// # Returns
///
/// A tuple of (arena, root_node_id) on success, or an error on failure.
pub fn read_document_from_file(
    path: &std::path::Path,
    ctx: &dyn ClmdContext<Error = crate::error::ClmdError>,
    options: &ClmdOptions,
) -> ClmdResult<(NodeArena, NodeId)> {
    // Try to detect format from file extension
    let registry = ReaderRegistry::new();

    let reader = if let Some(reader) = registry.detect_from_path(path) {
        ctx.info(&format!(
            "Auto-detected format from file extension: {}",
            reader.format_name()
        ));
        reader
    } else {
        // Fall back to the format specified in options
        registry.get(options.input_format).ok_or_else(|| {
            crate::error::ClmdError::io_error(format!(
                "No reader available for format: {:?}",
                options.input_format
            ))
        })?
    };

    reader.read_file(path, ctx, options)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clmd_options::ClmdOptions;
    use crate::context::PureContext;

    #[test]
    fn test_markdown_reader() {
        let ctx = PureContext::new();
        let options = ClmdOptions::default();
        let reader = MarkdownReader::new();

        let (arena, root) = reader.read("# Hello", &ctx, &options).unwrap();

        assert!(!arena.is_empty());
        let root_node = arena.get(root);
        assert!(matches!(root_node.value, crate::nodes::NodeValue::Document));
    }

    #[test]
    fn test_commonmark_reader() {
        let ctx = PureContext::new();
        let options = ClmdOptions::default();
        let reader = CommonMarkReader::new();

        let (arena, root) = reader.read("# Hello", &ctx, &options).unwrap();

        assert!(!arena.is_empty());
        let root_node = arena.get(root);
        assert!(matches!(root_node.value, crate::nodes::NodeValue::Document));
    }

    #[test]
    fn test_gfm_reader() {
        let ctx = PureContext::new();
        let options = ClmdOptions::default();
        let reader = GfmReader::new();

        let (arena, _root) = reader
            .read("# Hello\n\n| a | b |\n|---|---|\n| c | d |", &ctx, &options)
            .unwrap();

        assert!(!arena.is_empty());
    }

    #[test]
    fn test_reader_extensions() {
        let reader = MarkdownReader::new();

        assert!(reader.supports_extension("md"));
        assert!(reader.supports_extension("markdown"));
        assert!(reader.supports_extension("MD")); // case insensitive
        assert!(!reader.supports_extension("txt"));
    }

    #[test]
    fn test_reader_format() {
        let markdown_reader = MarkdownReader::new();
        assert_eq!(markdown_reader.format(), InputFormat::Markdown);

        let commonmark_reader = CommonMarkReader::new();
        assert_eq!(commonmark_reader.format(), InputFormat::CommonMark);

        let gfm_reader = GfmReader::new();
        assert_eq!(gfm_reader.format(), InputFormat::Gfm);
    }

    #[test]
    fn test_reader_registry() {
        let registry = ReaderRegistry::new();

        // Get by format
        assert!(registry.get(InputFormat::Markdown).is_some());
        assert!(registry.get(InputFormat::CommonMark).is_some());
        assert!(registry.get(InputFormat::Gfm).is_some());
        assert!(registry.get(InputFormat::Html).is_some());

        // Get by extension
        assert!(registry.get_by_extension("md").is_some());
        assert!(registry.get_by_extension("html").is_some());
        assert!(registry.get_by_extension("unknown").is_none());
    }

    #[test]
    fn test_reader_registry_detect_from_path() {
        let registry = ReaderRegistry::new();

        assert!(registry
            .detect_from_path(std::path::Path::new("test.md"))
            .is_some());
        assert!(registry
            .detect_from_path(std::path::Path::new("test.html"))
            .is_some());
        assert!(registry
            .detect_from_path(std::path::Path::new("test.unknown"))
            .is_none());
    }

    #[test]
    fn test_read_document() {
        let ctx = PureContext::new();
        let options = ClmdOptions::default();

        let (arena, root) = read_document("# Hello World", &ctx, &options).unwrap();

        assert!(!arena.is_empty());
        let root_node = arena.get(root);
        assert!(matches!(root_node.value, crate::nodes::NodeValue::Document));
    }

    #[test]
    fn test_read_document_with_pure_context() {
        let mut ctx = PureContext::new();
        ctx.add_file("test.md", b"# Test Document");

        let options = ClmdOptions::default();

        let (arena, _root) =
            read_document_from_file(std::path::Path::new("test.md"), &ctx, &options)
                .unwrap();

        assert!(!arena.is_empty());
    }
}
