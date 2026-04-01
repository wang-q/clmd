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
//! use clmd::options::{WriterOptions, OutputFormat};
//! use clmd::{parse_document, Options};
//!
//! let registry = WriterRegistry::new();
//! let writer = registry.get_by_name("html").unwrap();
//!
//! let options = Options::default();
//! let (arena, root) = parse_document("# Hello World", &options);
//! let writer_options = WriterOptions::default();
//! let output = writer.write(&arena, root, &writer_options).unwrap();
//! ```

use crate::context::ClmdContext;
use crate::core::arena::{NodeArena, NodeId};
use crate::core::error::{ClmdError, ClmdResult};
use crate::ext::flags::ExtensionFlags;
use crate::options::{OutputFormat, WriterOptions};
use std::fmt::Debug;
use std::path::Path;

/// A document writer that can render AST to a specific format.
///
/// Writers are responsible for converting the internal AST representation
/// into the target output format.
///
/// # Example
///
/// ```ignore
/// use clmd::writers::Writer;
/// use clmd::options::WriterOptions;
/// use clmd::context::PureContext;
///
/// fn use_writer<W: Writer>(writer: &W, arena: &NodeArena, root: NodeId) {
///     let ctx = PureContext::new();
///     let options = WriterOptions::default();
///     let output = writer.write(arena, root, &ctx, &options).unwrap();
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
    /// * `ctx` - The context for IO operations
    /// * `options` - Rendering options
    ///
    /// # Returns
    ///
    /// The rendered output as a string on success, or an error on failure.
    fn write(
        &self,
        arena: &NodeArena,
        root: NodeId,
        ctx: &dyn ClmdContext<Error = crate::core::error::ClmdError>,
        options: &WriterOptions,
    ) -> ClmdResult<String>;

    /// Get the format name this writer supports.
    fn format(&self) -> OutputFormat;

    /// Get the file extensions this writer can handle.
    fn extensions(&self) -> &[&'static str];

    /// Check if this writer supports a specific file extension.
    fn supports_extension(&self, ext: &str) -> bool {
        self.extensions().contains(&ext.to_lowercase().as_str())
    }

    /// Get the MIME type for this format.
    fn mime_type(&self) -> &'static str;

    /// Write the AST to a file.
    ///
    /// This is a convenience method that renders the document and writes it to a file.
    fn write_to_file(
        &self,
        arena: &NodeArena,
        root: NodeId,
        path: &Path,
        ctx: &dyn ClmdContext<Error = crate::core::error::ClmdError>,
        options: &WriterOptions,
    ) -> ClmdResult<()> {
        let content = self.write(arena, root, ctx, options)?;
        ctx.write_file(path, content.as_bytes())?;
        Ok(())
    }
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
/// if let Some(writer) = registry.get_by_name("html") {
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
    writers: Vec<Box<dyn Writer>>,
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
            writers: Vec::new(),
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
    ///
    /// # Arguments
    ///
    /// * `name` - The format name (e.g., "html", "commonmark")
    ///
    /// # Returns
    ///
    /// Some(writer) if found, None otherwise.
    pub fn get_by_name(&self, name: &str) -> Option<&dyn Writer> {
        let format = name.parse::<OutputFormat>().ok()?;
        self.get(format)
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
        self.writers
            .iter()
            .find(|w| w.supports_extension(&ext))
            .map(|w| w.as_ref())
    }

    /// Detect the format from a file path.
    ///
    /// # Arguments
    ///
    /// * `path` - The file path
    ///
    /// # Returns
    ///
    /// Some(format) if detected, None otherwise.
    pub fn detect_format(&self, path: &Path) -> Option<OutputFormat> {
        path.extension().and_then(|e| e.to_str()).and_then(|ext| {
            self.writers
                .iter()
                .find(|w| w.supports_extension(ext))
                .map(|w| w.format())
        })
    }

    /// Get all registered format names.
    pub fn formats(&self) -> Vec<&'static str> {
        self.writers.iter().map(|w| w.format().as_str()).collect()
    }

    /// Get all registered file extensions.
    pub fn extensions(&self) -> Vec<&'static str> {
        self.writers
            .iter()
            .flat_map(|w| w.extensions().iter().copied())
            .collect()
    }

    /// Check if a format is supported.
    pub fn supports_format(&self, format: &str) -> bool {
        format
            .parse::<OutputFormat>()
            .ok()
            .and_then(|f| self.get(f))
            .is_some()
    }

    /// Check if an extension is supported.
    pub fn supports_extension(&self, extension: &str) -> bool {
        self.get_by_extension(extension).is_some()
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
        _ctx: &dyn ClmdContext<Error = crate::core::error::ClmdError>,
        options: &WriterOptions,
    ) -> ClmdResult<String> {
        let mut render_options = crate::parser::options::Options::default();
        render_options.render.sourcepos = options.output_sourcepos;
        render_options.extension.tagfilter =
            options.extensions.contains(ExtensionFlags::TAGFILTER);
        Ok(crate::render::html::render(arena, root, &render_options))
    }

    fn format(&self) -> OutputFormat {
        OutputFormat::Html
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
        _ctx: &dyn ClmdContext<Error = crate::core::error::ClmdError>,
        options: &WriterOptions,
    ) -> ClmdResult<String> {
        let width = if options.wrap == crate::options::WrapOption::Auto {
            options.width
        } else {
            0
        };
        Ok(crate::render::commonmark::render(arena, root, 0, width))
    }

    fn format(&self) -> OutputFormat {
        OutputFormat::Markdown
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
        _ctx: &dyn ClmdContext<Error = crate::core::error::ClmdError>,
        _options: &WriterOptions,
    ) -> ClmdResult<String> {
        Ok(crate::render::renderer::render_to_xml(arena, root, 0))
    }

    fn format(&self) -> OutputFormat {
        OutputFormat::Xml
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
/// * `format` - The output format name
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
/// use clmd::options::WriterOptions;
/// use clmd::context::PureContext;
///
/// let ctx = PureContext::new();
/// let options = WriterOptions::default();
/// // let (arena, root) = parse_document("# Hello", &options);
/// // let output = write_document(&arena, root, "html", &ctx, &options).unwrap();
/// ```ignore
pub fn write_document(
    arena: &NodeArena,
    root: NodeId,
    format: &str,
    ctx: &dyn ClmdContext<Error = crate::core::error::ClmdError>,
    options: &WriterOptions,
) -> ClmdResult<String> {
    let registry = WriterRegistry::new();

    let writer = registry
        .get_by_name(format)
        .ok_or_else(|| ClmdError::unknown_writer(format))?;

    writer.write(arena, root, ctx, options)
}

/// Write a document to a file.
///
/// # Arguments
///
/// * `arena` - The arena containing the AST nodes
/// * `root` - The root node ID
/// * `path` - The output file path
/// * `format` - Optional format override (if None, detects from extension)
/// * `ctx` - The context for IO operations
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
/// use clmd::options::WriterOptions;
/// use clmd::context::PureContext;
///
/// let ctx = PureContext::new();
/// let options = WriterOptions::default();
/// // let (arena, root) = parse_document("# Hello", &options);
/// // write_file(&arena, root, "output.html", None, &ctx, &options).unwrap();
/// ```ignore
pub fn write_file(
    arena: &NodeArena,
    root: NodeId,
    path: &Path,
    format: Option<&str>,
    ctx: &dyn ClmdContext<Error = crate::core::error::ClmdError>,
    options: &WriterOptions,
) -> ClmdResult<()> {
    // Create the appropriate writer based on format or file extension
    let content = if let Some(format_name) = format {
        write_document(arena, root, format_name, ctx, options)?
    } else {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("html");
        let registry = WriterRegistry::new();
        let writer = registry
            .get_by_extension(ext)
            .ok_or_else(|| ClmdError::unknown_writer("unknown"))?;
        writer.write(arena, root, ctx, options)?
    };

    ctx.write_file(path, content.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::PureContext;

    fn create_test_document() -> (NodeArena, NodeId) {
        use crate::core::arena::{Node, TreeOps};
        use crate::core::nodes::{NodeHeading, NodeValue};

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let heading = arena.alloc(Node::with_value(NodeValue::Heading(NodeHeading {
            level: 1,
            setext: false,
            closed: false,
        })));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("Hello")));

        TreeOps::append_child(&mut arena, root, heading);
        TreeOps::append_child(&mut arena, heading, text);

        (arena, root)
    }

    #[test]
    fn test_html_writer() {
        let ctx = PureContext::new();
        let writer = HtmlWriter;
        let options = WriterOptions::default();
        let (arena, root) = create_test_document();

        let output = writer.write(&arena, root, &ctx, &options).unwrap();
        assert!(output.contains("<h1>"));
    }

    #[test]
    fn test_commonmark_writer() {
        let ctx = PureContext::new();
        let writer = CommonMarkWriter;
        let options = WriterOptions::default();
        let (arena, root) = create_test_document();

        let output = writer.write(&arena, root, &ctx, &options).unwrap();
        assert!(output.contains("# Hello"));
    }

    #[test]
    fn test_xml_writer() {
        let ctx = PureContext::new();
        let writer = XmlWriter;
        let options = WriterOptions::default();
        let (arena, root) = create_test_document();

        let output = writer.write(&arena, root, &ctx, &options).unwrap();
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

        let writer = registry.get_by_name("html");
        assert!(writer.is_some());
        assert_eq!(writer.unwrap().format(), OutputFormat::Html);

        let writer = registry.get_by_name("unknown");
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

        let path = Path::new("test.html");
        assert_eq!(registry.detect_format(path), Some(OutputFormat::Html));

        let path = Path::new("test.md");
        assert_eq!(registry.detect_format(path), Some(OutputFormat::Markdown));

        let path = Path::new("test");
        assert_eq!(registry.detect_format(path), None);
    }

    #[test]
    fn test_write_document() {
        let ctx = PureContext::new();
        let options = WriterOptions::default();
        let (arena, root) = create_test_document();

        let output = write_document(&arena, root, "html", &ctx, &options).unwrap();
        assert!(output.contains("<h1>"));
    }

    #[test]
    fn test_write_document_unknown_format() {
        let ctx = PureContext::new();
        let options = WriterOptions::default();
        let (arena, root) = create_test_document();

        let result = write_document(&arena, root, "unknown", &ctx, &options);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown writer"));
    }
}
