//! Document writer trait and implementations for clmd.
//!
//! This module provides a unified interface for writing documents to different
//! formats, inspired by Pandoc's Writer system. Writers convert the internal AST
//! representation into various output formats.
//!
//! # Example
//!
//! ```ignore
//! use clmd::writer::{Writer, HtmlWriter};
//! use clmd::options::{WriterOptions, OutputFormat};
//! use clmd::context::{ClmdContext, PureContext};
//! use clmd::arena::{NodeArena, NodeId};
//!
//! fn write_document(writer: &dyn Writer, arena: &NodeArena, root: NodeId) -> String {
//!     let ctx = PureContext::new();
//!     let options = WriterOptions::default();
//!     writer.write(arena, root, &ctx, &options).unwrap()
//! }
//! ```

use crate::arena::{NodeArena, NodeId};
use crate::options::{WriterOptions, OutputFormat};
use crate::context::ClmdContext;
use crate::error::ClmdResult;
use std::fmt::Debug;
use std::path::Path;

/// A document writer that can render AST to a specific format.
///
/// Writers are responsible for converting the internal AST representation
/// into various output formats. This trait is designed to work with the
/// `ClmdContext` abstraction for IO operations.
///
/// # Example
///
/// ```ignore
/// use clmd::writer::{Writer, HtmlWriter};
/// use clmd::options::WriterOptions;
/// use clmd::context::IoContext;
///
/// fn use_writer(writer: &dyn Writer, arena: &clmd::arena::NodeArena, root: clmd::arena::NodeId) {
///     let ctx = IoContext::new();
///     let options = WriterOptions::default();
///     let output = writer.write(arena, root, &ctx, &options).unwrap();
///     // Use the output...
/// }
/// ```ignore
pub trait Writer: Send + Sync + Debug {
    /// Write the AST to the output format.
    ///
    /// # Arguments
    ///
    /// * `arena` - The node arena containing the AST
    /// * `root` - The root node ID
    /// * `ctx` - The context for IO operations and logging
    /// * `options` - Rendering options
    ///
    /// # Returns
    ///
    /// The rendered output as a String on success, or an error on failure.
    fn write(
        &self,
        arena: &NodeArena,
        root: NodeId,
        ctx: &dyn ClmdContext<Error = crate::error::ClmdError>,
        options: &WriterOptions,
    ) -> ClmdResult<String>;

    /// Write the AST to a file.
    ///
    /// This is a convenience method that renders the document and writes it to a file.
    ///
    /// # Arguments
    ///
    /// * `arena` - The node arena containing the AST
    /// * `root` - The root node ID
    /// * `path` - The path to write to
    /// * `ctx` - The context for IO operations and logging
    /// * `options` - Rendering options
    ///
    /// # Returns
    ///
    /// Ok(()) on success, or an error on failure.
    fn write_to_file(
        &self,
        arena: &NodeArena,
        root: NodeId,
        path: &Path,
        ctx: &dyn ClmdContext<Error = crate::error::ClmdError>,
        options: &WriterOptions,
    ) -> ClmdResult<()> {
        let content = self.write(arena, root, ctx, options)?;
        ctx.write_file(path, content.as_bytes())?;
        Ok(())
    }

    /// Get the format name this writer supports.
    fn format(&self) -> OutputFormat;

    /// Get the file extensions this writer can handle.
    fn extensions(&self) -> &[&'static str];

    /// Check if this writer supports a specific file extension.
    fn supports_extension(&self, ext: &str) -> bool {
        self.extensions().contains(&ext.to_lowercase().as_str())
    }

    /// Get the format name as a string.
    fn format_name(&self) -> &'static str {
        self.format().as_str()
    }

    /// Check if this format is a binary format.
    fn is_binary(&self) -> bool {
        self.format().is_binary()
    }
}

/// HTML document writer.
///
/// Renders the AST to HTML format.
#[derive(Debug, Clone, Copy, Default)]
pub struct HtmlWriter;

impl HtmlWriter {
    /// Create a new HTML writer.
    pub fn new() -> Self {
        Self
    }
}

impl Writer for HtmlWriter {
    fn write(
        &self,
        arena: &NodeArena,
        root: NodeId,
        ctx: &dyn ClmdContext<Error = crate::error::ClmdError>,
        options: &WriterOptions,
    ) -> ClmdResult<String> {
        ctx.info("Rendering to HTML");

        // Build HTML render options
        let mut html_options: u32 = 0;
        if options.output_sourcepos {
            html_options |= crate::parser::OPT_SOURCEPOS;
        }
        if options.extensions.contains(crate::extensions::Extensions::TAGFILTER) {
            html_options |= crate::parser::OPT_TAGFILTER;
        }

        Ok(crate::render::html::render(arena, root, html_options))
    }

    fn format(&self) -> OutputFormat {
        OutputFormat::Html
    }

    fn extensions(&self) -> &[&'static str] {
        &["html", "htm"]
    }
}

/// XHTML document writer.
///
/// Renders the AST to XHTML format.
#[derive(Debug, Clone, Copy, Default)]
pub struct XhtmlWriter;

impl XhtmlWriter {
    /// Create a new XHTML writer.
    pub fn new() -> Self {
        Self
    }
}

impl Writer for XhtmlWriter {
    fn write(
        &self,
        arena: &NodeArena,
        root: NodeId,
        ctx: &dyn ClmdContext<Error = crate::error::ClmdError>,
        options: &WriterOptions,
    ) -> ClmdResult<String> {
        ctx.info("Rendering to XHTML");

        // For now, use HTML renderer with XHTML doctype
        let mut html_options: u32 = 0;
        if options.output_sourcepos {
            html_options |= crate::parser::OPT_SOURCEPOS;
        }
        if options.extensions.contains(crate::extensions::Extensions::TAGFILTER) {
            html_options |= crate::parser::OPT_TAGFILTER;
        }
        let mut html = crate::render::html::render(arena, root, html_options);

        // Replace HTML5 doctype with XHTML doctype
        if html.starts_with("<!DOCTYPE html>") {
            html = html.replacen(
                "<!DOCTYPE html>",
                r#"<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.0 Transitional//EN" "http://www.w3.org/TR/xhtml1/DTD/xhtml1-transitional.dtd">"#,
                1,
            );
        }

        Ok(html)
    }

    fn format(&self) -> OutputFormat {
        OutputFormat::Xhtml
    }

    fn extensions(&self) -> &[&'static str] {
        &["xhtml"]
    }
}

/// CommonMark document writer.
///
/// Renders the AST to CommonMark format.
#[derive(Debug, Clone, Copy, Default)]
pub struct CommonMarkWriter;

impl CommonMarkWriter {
    /// Create a new CommonMark writer.
    pub fn new() -> Self {
        Self
    }
}

impl Writer for CommonMarkWriter {
    fn write(
        &self,
        arena: &NodeArena,
        root: NodeId,
        ctx: &dyn ClmdContext<Error = crate::error::ClmdError>,
        options: &WriterOptions,
    ) -> ClmdResult<String> {
        ctx.info("Rendering to CommonMark");

        let width = if options.wrap == crate::options::WrapOption::Auto {
            options.columns
        } else {
            0
        };

        // Build CommonMark render options
        let mut cm_options: u32 = 0;
        if options.hardbreaks {
            cm_options |= 1; // Placeholder for hardbreaks option
        }

        Ok(crate::render::commonmark::render(arena, root, cm_options, width))
    }

    fn format(&self) -> OutputFormat {
        OutputFormat::CommonMark
    }

    fn extensions(&self) -> &[&'static str] {
        &["md", "markdown", "mkd"]
    }
}

/// XML document writer.
///
/// Renders the AST to XML format for debugging.
#[derive(Debug, Clone, Copy, Default)]
pub struct XmlWriter;

impl XmlWriter {
    /// Create a new XML writer.
    pub fn new() -> Self {
        Self
    }
}

impl Writer for XmlWriter {
    fn write(
        &self,
        arena: &NodeArena,
        root: NodeId,
        ctx: &dyn ClmdContext<Error = crate::error::ClmdError>,
        _options: &WriterOptions,
    ) -> ClmdResult<String> {
        ctx.info("Rendering to XML");

        Ok(crate::render::renderer::render_to_xml(arena, root, 0))
    }

    fn format(&self) -> OutputFormat {
        OutputFormat::Xml
    }

    fn extensions(&self) -> &[&'static str] {
        &["xml"]
    }
}

/// LaTeX document writer.
///
/// Renders the AST to LaTeX format.
#[derive(Debug, Clone, Copy, Default)]
pub struct LatexWriter;

impl LatexWriter {
    /// Create a new LaTeX writer.
    pub fn new() -> Self {
        Self
    }
}

impl Writer for LatexWriter {
    fn write(
        &self,
        arena: &NodeArena,
        root: NodeId,
        ctx: &dyn ClmdContext<Error = crate::error::ClmdError>,
        _options: &WriterOptions,
    ) -> ClmdResult<String> {
        ctx.info("Rendering to LaTeX");

        // Build LaTeX render options (placeholder)
        let latex_options: u32 = 0;
        Ok(crate::render::latex::render(arena, root, latex_options))
    }

    fn format(&self) -> OutputFormat {
        OutputFormat::Latex
    }

    fn extensions(&self) -> &[&'static str] {
        &["tex", "latex"]
    }
}

/// Man page document writer.
///
/// Renders the AST to Man page format.
#[derive(Debug, Clone, Copy, Default)]
pub struct ManWriter;

impl ManWriter {
    /// Create a new Man page writer.
    pub fn new() -> Self {
        Self
    }
}

impl Writer for ManWriter {
    fn write(
        &self,
        arena: &NodeArena,
        root: NodeId,
        ctx: &dyn ClmdContext<Error = crate::error::ClmdError>,
        _options: &WriterOptions,
    ) -> ClmdResult<String> {
        ctx.info("Rendering to Man page");

        // Build Man page render options (placeholder)
        let man_options: u32 = 0;
        Ok(crate::render::man::render(arena, root, man_options))
    }

    fn format(&self) -> OutputFormat {
        OutputFormat::Man
    }

    fn extensions(&self) -> &[&'static str] {
        &["man"]
    }
}

/// Plain text document writer.
///
/// Renders the AST to plain text format.
#[derive(Debug, Clone, Copy, Default)]
pub struct PlainWriter;

impl PlainWriter {
    /// Create a new Plain text writer.
    pub fn new() -> Self {
        Self
    }
}

impl Writer for PlainWriter {
    fn write(
        &self,
        arena: &NodeArena,
        root: NodeId,
        ctx: &dyn ClmdContext<Error = crate::error::ClmdError>,
        _options: &WriterOptions,
    ) -> ClmdResult<String> {
        ctx.info("Rendering to plain text");

        // For now, use CommonMark renderer and strip formatting
        // In the future, this should have its own proper plain text renderer
        let cm = crate::render::commonmark::render(arena, root, 0, 0);
        Ok(cm)
    }

    fn format(&self) -> OutputFormat {
        OutputFormat::Plain
    }

    fn extensions(&self) -> &[&'static str] {
        &["txt", "text"]
    }
}

/// Writer registry for looking up writers by format or extension.
///
/// This registry provides a way to dynamically look up writers at runtime
/// based on format name or file extension.
#[derive(Debug, Default)]
pub struct WriterRegistry {
    writers: Vec<Box<dyn Writer>>,
}

impl WriterRegistry {
    /// Create a new registry with all default writers.
    pub fn new() -> Self {
        let mut registry = Self::empty();
        registry.register_default_writers();
        registry
    }

    /// Create an empty registry.
    pub fn empty() -> Self {
        Self {
            writers: Vec::new(),
        }
    }

    /// Register a writer.
    pub fn register(&mut self, writer: Box<dyn Writer>) {
        self.writers.push(writer);
    }

    /// Get a writer by format.
    pub fn get(&self, format: OutputFormat) -> Option<&dyn Writer> {
        self.writers
            .iter()
            .find(|w| w.format() == format)
            .map(|w| w.as_ref())
    }

    /// Get a writer by format name.
    pub fn get_by_name(&self, name: &str) -> Option<&dyn Writer> {
        let format = name.parse::<OutputFormat>().ok()?;
        self.get(format)
    }

    /// Get a writer by file extension.
    pub fn get_by_extension(&self, extension: &str) -> Option<&dyn Writer> {
        let ext = extension.to_lowercase();
        self.writers
            .iter()
            .find(|w| w.supports_extension(&ext))
            .map(|w| w.as_ref())
    }

    /// Detect format from a file path and return the appropriate writer.
    pub fn detect_from_path(&self, path: &Path) -> Option<&dyn Writer> {
        path.extension()
            .and_then(|e| e.to_str())
            .and_then(|ext| self.get_by_extension(ext))
    }

    /// Register default writers.
    fn register_default_writers(&mut self) {
        self.register(Box::new(HtmlWriter::new()));
        self.register(Box::new(XhtmlWriter::new()));
        self.register(Box::new(CommonMarkWriter::new()));
        self.register(Box::new(XmlWriter::new()));
        self.register(Box::new(LatexWriter::new()));
        self.register(Box::new(ManWriter::new()));
        self.register(Box::new(PlainWriter::new()));
    }
}

impl Clone for WriterRegistry {
    fn clone(&self) -> Self {
        // Create a new registry with default writers
        Self::new()
    }
}

/// Helper function to write a document with automatic format detection.
///
/// # Arguments
///
/// * `arena` - The node arena containing the AST
/// * `root` - The root node ID
/// * `ctx` - The context for IO operations
/// * `options` - Rendering options (format is used to select writer)
///
/// # Returns
///
/// The rendered output as a String on success, or an error on failure.
pub fn write_document(
    arena: &NodeArena,
    root: NodeId,
    ctx: &dyn ClmdContext<Error = crate::error::ClmdError>,
    options: &WriterOptions,
) -> ClmdResult<String> {
    let registry = WriterRegistry::new();

    let writer = registry.get(options.output_format).ok_or_else(|| {
        crate::error::ClmdError::io_error(format!(
            "No writer available for format: {:?}",
            options.output_format
        ))
    })?;

    writer.write(arena, root, ctx, options)
}

/// Helper function to write a document to a file with automatic format detection.
///
/// # Arguments
///
/// * `arena` - The node arena containing the AST
/// * `root` - The root node ID
/// * `path` - The path to write to
/// * `ctx` - The context for IO operations
/// * `options` - Rendering options (can be overridden by file extension)
///
/// # Returns
///
/// Ok(()) on success, or an error on failure.
pub fn write_document_to_file(
    arena: &NodeArena,
    root: NodeId,
    path: &Path,
    ctx: &dyn ClmdContext<Error = crate::error::ClmdError>,
    options: &WriterOptions,
) -> ClmdResult<()> {
    // Try to detect format from file extension
    let registry = WriterRegistry::new();

    let writer = if let Some(writer) = registry.detect_from_path(path) {
        ctx.info(&format!(
            "Auto-detected format from file extension: {}",
            writer.format_name()
        ));
        writer
    } else {
        // Fall back to the format specified in options
        registry.get(options.output_format).ok_or_else(|| {
            crate::error::ClmdError::io_error(format!(
                "No writer available for format: {:?}",
                options.output_format
            ))
        })?
    };

    writer.write_to_file(arena, root, path, ctx, options)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arena::{Node, NodeArena, TreeOps};
    use crate::context::PureContext;
    use crate::nodes::NodeValue;

    fn create_test_document() -> (NodeArena, NodeId) {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("Hello World")));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, text);

        (arena, root)
    }

    #[test]
    fn test_html_writer() {
        let ctx = PureContext::new();
        let options = WriterOptions::default();
        let writer = HtmlWriter::new();
        let (arena, root) = create_test_document();

        let output = writer.write(&arena, root, &ctx, &options).unwrap();

        assert!(output.contains("<p>"));
        assert!(output.contains("Hello World"));
        assert!(output.contains("</p>"));
    }

    #[test]
    fn test_commonmark_writer() {
        let ctx = PureContext::new();
        let options = WriterOptions::default();
        let writer = CommonMarkWriter::new();
        let (arena, root) = create_test_document();

        let output = writer.write(&arena, root, &ctx, &options).unwrap();

        assert!(output.contains("Hello World"));
    }

    #[test]
    fn test_xml_writer() {
        let ctx = PureContext::new();
        let options = WriterOptions::default();
        let writer = XmlWriter::new();
        let (arena, root) = create_test_document();

        let output = writer.write(&arena, root, &ctx, &options).unwrap();

        assert!(output.contains("<?xml"));
        assert!(output.contains("<document>"));
    }

    #[test]
    fn test_writer_extensions() {
        let writer = HtmlWriter::new();

        assert!(writer.supports_extension("html"));
        assert!(writer.supports_extension("htm"));
        assert!(writer.supports_extension("HTML")); // case insensitive
        assert!(!writer.supports_extension("txt"));
    }

    #[test]
    fn test_writer_format() {
        let html_writer = HtmlWriter::new();
        assert_eq!(html_writer.format(), OutputFormat::Html);

        let commonmark_writer = CommonMarkWriter::new();
        assert_eq!(commonmark_writer.format(), OutputFormat::CommonMark);

        let xml_writer = XmlWriter::new();
        assert_eq!(xml_writer.format(), OutputFormat::Xml);
    }

    #[test]
    fn test_writer_is_binary() {
        assert!(!HtmlWriter::new().is_binary());
        assert!(!CommonMarkWriter::new().is_binary());
        // PDF would be binary, but we don't have a PDF writer yet
    }

    #[test]
    fn test_writer_registry() {
        let registry = WriterRegistry::new();

        // Get by format
        assert!(registry.get(OutputFormat::Html).is_some());
        assert!(registry.get(OutputFormat::CommonMark).is_some());
        assert!(registry.get(OutputFormat::Xml).is_some());
        assert!(registry.get(OutputFormat::Latex).is_some());
        assert!(registry.get(OutputFormat::Man).is_some());

        // Get by extension
        assert!(registry.get_by_extension("html").is_some());
        assert!(registry.get_by_extension("md").is_some());
        assert!(registry.get_by_extension("xml").is_some());
        assert!(registry.get_by_extension("unknown").is_none());
    }

    #[test]
    fn test_writer_registry_detect_from_path() {
        let registry = WriterRegistry::new();

        assert!(registry.detect_from_path(Path::new("test.html")).is_some());
        assert!(registry.detect_from_path(Path::new("test.md")).is_some());
        assert!(registry
            .detect_from_path(Path::new("test.unknown"))
            .is_none());
    }

    #[test]
    fn test_write_document() {
        let ctx = PureContext::new();
        let options = WriterOptions::default();
        let (arena, root) = create_test_document();

        let output = write_document(&arena, root, &ctx, &options).unwrap();

        assert!(!output.is_empty());
    }

    #[test]
    fn test_write_document_to_file() {
        let ctx = PureContext::new();
        let options = WriterOptions::default();
        let (arena, root) = create_test_document();

        write_document_to_file(&arena, root, Path::new("output.html"), &ctx, &options)
            .unwrap();

        assert!(ctx.has_file("output.html"));
    }
}
