//! Writer registry for managing output format writers.
//!
//! This module provides a centralized registry for document writers,
//! allowing formats to be looked up by name or file extension.
//!
//! # Example
//!
//! ```ignore
//! use clmd::io::writer::{WriterRegistry, Writer};
//! use clmd::options::WriterOptions;
//!
//! let registry = WriterRegistry::new();
//!
//! // Get a writer by format name
//! if let Some(writer) = registry.get_by_name("html") {
//!     println!("Found writer: {}", writer.format());
//! }
//!
//! // Get a writer by file extension
//! if let Some(writer) = registry.get_by_extension("html") {
//!     println!("Found writer for .html files");
//! }
//! ```

use std::collections::HashMap;
use std::path::Path;

use crate::context::ClmdContext;
use crate::core::arena::{NodeArena, NodeId};
use crate::core::error::{ClmdError, ClmdResult};
use crate::options::{OutputFormat, WriterOptions};

// Import writers from their modules
use crate::io::writer::bibtex::BibTeXWriter;
use crate::io::writer::beamer::BeamerWriter;
use crate::io::writer::revealjs::RevealJsWriter;
use crate::io::writer::rtf::RtfWriter;

/// Type alias for boxed writer trait objects.
pub type BoxedWriter = Box<dyn Writer>;

/// A document writer that can render AST to a specific format.
///
/// Writers are responsible for converting the internal AST representation
/// into the target output format.
///
/// # Example
///
/// ```ignore
/// use clmd::io::writer::Writer;
/// use clmd::options::WriterOptions;
/// use clmd::context::PureContext;
///
/// fn use_writer<W: Writer>(writer: &W, arena: &NodeArena, root: NodeId) {
///     let ctx = PureContext::new();
///     let options = WriterOptions::default();
///     let output = writer.write(arena, root, &ctx, &options).unwrap();
///     println!("{}", output);
/// }
/// ```
pub trait Writer: Send + Sync + std::fmt::Debug {
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
        ctx: &dyn ClmdContext<Error = ClmdError>,
        options: &WriterOptions,
    ) -> ClmdResult<String>;

    /// Get the format this writer supports.
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
    /// This is a convenience method that writes the rendered output directly to a file.
    ///
    /// # Arguments
    ///
    /// * `arena` - The arena containing the AST nodes
    /// * `root` - The root node ID
    /// * `path` - The output file path
    /// * `ctx` - The context for IO operations
    /// * `options` - Rendering options
    ///
    /// # Returns
    ///
    /// Ok on success, or an error on failure.
    fn write_to_file(
        &self,
        arena: &NodeArena,
        root: NodeId,
        path: &std::path::Path,
        ctx: &dyn ClmdContext<Error = ClmdError>,
        options: &WriterOptions,
    ) -> ClmdResult<()> {
        let content = self.write(arena, root, ctx, options)?;
        ctx.write_file(path, content.as_bytes())
    }
}

/// A registry of document writers.
///
/// The registry maintains a mapping from format names to writer implementations,
/// and provides methods for looking up writers by name or file extension.
#[derive(Default)]
pub struct WriterRegistry {
    /// Map from format name to writer.
    writers: HashMap<OutputFormat, BoxedWriter>,
    /// Map from file extension to format name.
    extension_map: HashMap<String, OutputFormat>,
}

impl std::fmt::Debug for WriterRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WriterRegistry")
            .field("writers", &self.writers.keys().collect::<Vec<_>>())
            .field("extension_map", &self.extension_map)
            .finish()
    }
}

impl WriterRegistry {
    /// Create a new empty registry.
    pub fn empty() -> Self {
        Self {
            writers: HashMap::new(),
            extension_map: HashMap::new(),
        }
    }

    /// Create a new registry with all built-in writers registered.
    pub fn new() -> Self {
        let mut registry = Self::empty();
        registry.register_builtin_writers();
        registry
    }

    /// Register a writer.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use clmd::io::writer::{WriterRegistry, HtmlWriter};
    ///
    /// let mut registry = WriterRegistry::empty();
    /// registry.register(Box::new(HtmlWriter));
    /// ```
    pub fn register(&mut self, writer: BoxedWriter) {
        let format = writer.format();
        let extensions: Vec<_> = writer.extensions().to_vec();

        self.writers.insert(format, writer);

        // Register extensions
        for ext in extensions {
            self.extension_map.insert(ext.to_lowercase(), format);
        }
    }

    /// Get a writer by format.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use clmd::io::writer::WriterRegistry;
    /// use clmd::options::OutputFormat;
    ///
    /// let registry = WriterRegistry::new();
    /// if let Some(writer) = registry.get(OutputFormat::Html) {
    ///     println!("Found writer: {}", writer.format());
    /// }
    /// ```
    pub fn get(&self, format: OutputFormat) -> Option<&dyn Writer> {
        self.writers.get(&format).map(|w| w.as_ref())
    }

    /// Get a writer by format name.
    ///
    /// # Arguments
    ///
    /// * `name` - The format name (e.g., "html", "markdown")
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
        self.extension_map
            .get(&ext)
            .and_then(|format| self.get(*format))
    }

    /// Get a writer by file path.
    ///
    /// This extracts the extension from the path and looks up the corresponding
    /// writer.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use clmd::io::writer::WriterRegistry;
    /// use std::path::Path;
    ///
    /// let registry = WriterRegistry::new();
    /// if let Some(writer) = registry.get_by_path(Path::new("document.html")) {
    ///     println!("Found writer for document.html");
    /// }
    /// ```
    pub fn get_by_path(&self, path: &Path) -> Option<&dyn Writer> {
        path.extension()
            .and_then(|e| e.to_str())
            .and_then(|ext| self.get_by_extension(ext))
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
        path.extension()
            .and_then(|e| e.to_str())
            .and_then(|ext| self.extension_map.get(&ext.to_lowercase()).copied())
    }

    /// Check if a writer is registered for the given format name.
    pub fn supports_format(&self, name: &str) -> bool {
        name.parse::<OutputFormat>()
            .ok()
            .and_then(|f| self.get(f))
            .is_some()
    }

    /// Check if an extension is supported.
    pub fn supports_extension(&self, extension: &str) -> bool {
        self.get_by_extension(extension).is_some()
    }

    /// Get a list of all registered format names.
    pub fn formats(&self) -> Vec<&'static str> {
        self.writers.keys().map(|f| f.as_str()).collect()
    }

    /// Get a list of all registered file extensions.
    pub fn extensions(&self) -> Vec<&'static str> {
        self.extension_map
            .keys()
            .filter_map(|k| {
                // Get the extension string from the format
                self.extension_map.get(k).map(|f| {
                    // Find the matching extension from the writer
                    if let Some(writer) = self.get(*f) {
                        writer
                            .extensions()
                            .iter()
                            .find(|e| e.to_lowercase() == *k)
                            .copied()
                    } else {
                        None
                    }
                })?
            })
            .collect()
    }

    /// Get the number of registered writers.
    pub fn len(&self) -> usize {
        self.writers.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.writers.is_empty()
    }

    /// Remove a writer from the registry.
    pub fn remove(&mut self, format: OutputFormat) -> Option<BoxedWriter> {
        // Remove from extension map
        self.extension_map.retain(|_, v| *v != format);

        // Remove writer
        self.writers.remove(&format)
    }

    /// Clear all writers from the registry.
    pub fn clear(&mut self) {
        self.writers.clear();
        self.extension_map.clear();
    }

    /// Register all built-in writers.
    fn register_builtin_writers(&mut self) {
        self.register(Box::new(HtmlWriter));
        self.register(Box::new(CommonMarkWriter));
        self.register(Box::new(XmlWriter));
        self.register(Box::new(LatexWriter));
        self.register(Box::new(ManWriter));
        self.register(Box::new(TypstWriter));
        self.register(Box::new(PdfWriter));
        self.register(Box::new(BibTeXWriter));
        self.register(Box::new(RtfWriter));
        self.register(Box::new(BeamerWriter));
        self.register(Box::new(RevealJsWriter));
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
        _ctx: &dyn ClmdContext<Error = ClmdError>,
        options: &WriterOptions,
    ) -> ClmdResult<String> {
        let mut render_options = crate::options::Options::default();
        render_options.render.sourcepos = options.output_sourcepos;
        render_options.extension.tagfilter = options
            .extensions
            .contains(crate::ext::flags::ExtensionFlags::TAGFILTER);
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
        _ctx: &dyn ClmdContext<Error = ClmdError>,
        options: &WriterOptions,
    ) -> ClmdResult<String> {
        let width = if options.wrap == crate::options::WrapOption::Auto {
            options.width
        } else {
            0
        };
        Ok(crate::render::commonmark::render(arena, root, width))
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
        _ctx: &dyn ClmdContext<Error = ClmdError>,
        options: &WriterOptions,
    ) -> ClmdResult<String> {
        crate::io::writer::xml::write_xml(arena, root, options)
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

/// LaTeX document writer.
///
/// Renders documents to LaTeX format.
#[derive(Debug, Clone, Copy)]
pub struct LatexWriter;

impl Writer for LatexWriter {
    fn write(
        &self,
        arena: &NodeArena,
        root: NodeId,
        _ctx: &dyn ClmdContext<Error = ClmdError>,
        options: &WriterOptions,
    ) -> ClmdResult<String> {
        crate::io::writer::latex::write_latex(arena, root, options)
    }

    fn format(&self) -> OutputFormat {
        OutputFormat::Latex
    }

    fn extensions(&self) -> &[&'static str] {
        &["tex", "latex"]
    }

    fn mime_type(&self) -> &'static str {
        "application/x-latex"
    }
}

/// Man page document writer.
///
/// Renders documents to Unix man page (groff) format.
#[derive(Debug, Clone, Copy)]
pub struct ManWriter;

impl Writer for ManWriter {
    fn write(
        &self,
        arena: &NodeArena,
        root: NodeId,
        _ctx: &dyn ClmdContext<Error = ClmdError>,
        options: &WriterOptions,
    ) -> ClmdResult<String> {
        crate::io::writer::man::write_man(arena, root, options)
    }

    fn format(&self) -> OutputFormat {
        OutputFormat::Man
    }

    fn extensions(&self) -> &[&'static str] {
        &["man", "1", "2", "3", "4", "5", "6", "7", "8", "9"]
    }

    fn mime_type(&self) -> &'static str {
        "application/x-troff-man"
    }
}

/// Typst document writer.
///
/// Renders documents to Typst format.
#[derive(Debug, Clone, Copy)]
pub struct TypstWriter;

impl Writer for TypstWriter {
    fn write(
        &self,
        arena: &NodeArena,
        root: NodeId,
        _ctx: &dyn ClmdContext<Error = ClmdError>,
        options: &WriterOptions,
    ) -> ClmdResult<String> {
        crate::io::writer::typst::write_typst(arena, root, options)
    }

    fn format(&self) -> OutputFormat {
        OutputFormat::Typst
    }

    fn extensions(&self) -> &[&'static str] {
        &["typ", "typst"]
    }

    fn mime_type(&self) -> &'static str {
        "text/typst"
    }
}

/// PDF document writer.
///
/// Renders documents to PDF format.
#[derive(Debug, Clone, Copy)]
pub struct PdfWriter;

impl Writer for PdfWriter {
    fn write(
        &self,
        arena: &NodeArena,
        root: NodeId,
        _ctx: &dyn ClmdContext<Error = ClmdError>,
        options: &WriterOptions,
    ) -> ClmdResult<String> {
        crate::io::writer::pdf::write_pdf(arena, root, options)
    }

    fn format(&self) -> OutputFormat {
        OutputFormat::Pdf
    }

    fn extensions(&self) -> &[&'static str] {
        &["pdf"]
    }

    fn mime_type(&self) -> &'static str {
        "application/pdf"
    }
}

/// Get the default writer registry.
///
/// This is a lazily initialized global registry containing all built-in writers.
///
/// # Example
///
/// ```ignore
/// use clmd::io::writer::default_registry;
///
/// let registry = default_registry();
/// if let Some(writer) = registry.get_by_name("html") {
///     println!("HTML writer is available");
/// }
/// ```
pub fn default_registry() -> &'static WriterRegistry {
    use std::sync::OnceLock;
    static REGISTRY: OnceLock<WriterRegistry> = OnceLock::new();
    REGISTRY.get_or_init(WriterRegistry::new)
}

/// Get a writer by name from the default registry.
///
/// This is a convenience function that looks up a writer in the default registry.
///
/// # Example
///
/// ```ignore
/// use clmd::io::writer::get_writer;
///
/// if let Some(writer) = get_writer("html") {
///     println!("Found HTML writer");
/// }
/// ```
pub fn get_writer(name: &str) -> Option<&'static dyn Writer> {
    default_registry().get_by_name(name)
}

/// Get a writer by file extension from the default registry.
///
/// This is a convenience function that looks up a writer by extension
/// in the default registry.
///
/// # Example
///
/// ```ignore
/// use clmd::io::writer::get_writer_by_extension;
///
/// if let Some(writer) = get_writer_by_extension("html") {
///     println!("Found writer for .html files");
/// }
/// ```
pub fn get_writer_by_extension(ext: &str) -> Option<&'static dyn Writer> {
    default_registry().get_by_extension(ext)
}

/// Get a writer by file path from the default registry.
///
/// This is a convenience function that looks up a writer by file path
/// in the default registry.
///
/// # Example
///
/// ```ignore
/// use clmd::io::writer::get_writer_by_path;
/// use std::path::Path;
///
/// if let Some(writer) = get_writer_by_path(Path::new("document.html")) {
///     println!("Found writer for document.html");
/// }
/// ```
pub fn get_writer_by_path(path: &Path) -> Option<&'static dyn Writer> {
    default_registry().get_by_path(path)
}

/// Write a document to a string.
///
/// # Arguments
///
/// * `arena` - The arena containing the AST nodes
/// * `root` - The root node ID
/// * `format` - The output format name
/// * `ctx` - The context for IO operations
/// * `options` - Rendering options
///
/// # Returns
///
/// The rendered output as a string on success, or an error on failure.
pub fn write_document(
    arena: &NodeArena,
    root: NodeId,
    format: &str,
    ctx: &dyn ClmdContext<Error = ClmdError>,
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
pub fn write_file(
    arena: &NodeArena,
    root: NodeId,
    path: &Path,
    format: Option<&str>,
    ctx: &dyn ClmdContext<Error = ClmdError>,
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
    use crate::io::test_utils::{create_heading, create_test_arena};

    #[test]
    fn test_registry_empty() {
        let registry = WriterRegistry::empty();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_registry_new() {
        let registry = WriterRegistry::new();
        assert!(!registry.is_empty());
        assert!(registry.supports_format("html"));
        assert!(registry.supports_format("markdown"));
        assert!(registry.supports_format("xml"));
        assert!(registry.supports_format("pdf"));

        assert!(registry.supports_extension("html"));
        assert!(registry.supports_extension("md"));
        assert!(registry.supports_extension("pdf"));
    }

    #[test]
    fn test_registry_get() {
        let registry = WriterRegistry::new();

        let writer = registry.get(OutputFormat::Html);
        assert!(writer.is_some());
        assert_eq!(writer.unwrap().format(), OutputFormat::Html);

        let writer = registry.get_by_name("html");
        assert!(writer.is_some());

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
    fn test_registry_get_by_path() {
        let registry = WriterRegistry::new();

        let writer = registry.get_by_path(Path::new("test.html"));
        assert!(writer.is_some());

        let writer = registry.get_by_path(Path::new("no_extension"));
        assert!(writer.is_none());
    }

    #[test]
    fn test_detect_format() {
        let registry = WriterRegistry::new();

        // Note: "html" extension is registered by both HtmlWriter and RevealJsWriter
        // The last one registered wins (RevealJs in this case)
        let path = Path::new("test.html");
        let format = registry.detect_format(path);
        assert!(
            format == Some(OutputFormat::Html) || format == Some(OutputFormat::RevealJs)
        );

        let path = Path::new("test.md");
        assert_eq!(registry.detect_format(path), Some(OutputFormat::Markdown));

        let path = Path::new("test.revealjs");
        assert_eq!(registry.detect_format(path), Some(OutputFormat::RevealJs));

        let path = Path::new("test");
        assert_eq!(registry.detect_format(path), None);
    }

    #[test]
    fn test_html_writer() {
        let ctx = PureContext::new();
        let writer = HtmlWriter;
        let options = WriterOptions::default();
        let (mut arena, root) = create_test_arena();
        create_heading(&mut arena, root, 1, "Hello");

        let output = writer.write(&arena, root, &ctx, &options).unwrap();
        assert!(output.contains("<h1>"));
    }

    #[test]
    fn test_commonmark_writer() {
        let ctx = PureContext::new();
        let writer = CommonMarkWriter;
        let options = WriterOptions::default();
        let (mut arena, root) = create_test_arena();
        create_heading(&mut arena, root, 1, "Hello");

        let output = writer.write(&arena, root, &ctx, &options).unwrap();
        assert!(output.contains("# Hello"));
    }

    #[test]
    fn test_xml_writer() {
        let ctx = PureContext::new();
        let writer = XmlWriter;
        let options = WriterOptions::default();
        let (mut arena, root) = create_test_arena();
        create_heading(&mut arena, root, 1, "Hello");

        let output = writer.write(&arena, root, &ctx, &options).unwrap();
        assert!(output.contains("<?xml"));
    }

    #[test]
    fn test_write_document() {
        let ctx = PureContext::new();
        let options = WriterOptions::default();
        let (mut arena, root) = create_test_arena();
        create_heading(&mut arena, root, 1, "Hello");

        let output = write_document(&arena, root, "html", &ctx, &options).unwrap();
        assert!(output.contains("<h1>"));
    }

    #[test]
    fn test_write_document_unknown_format() {
        let ctx = PureContext::new();
        let options = WriterOptions::default();
        let (mut arena, root) = create_test_arena();
        create_heading(&mut arena, root, 1, "Hello");

        let result = write_document(&arena, root, "unknown", &ctx, &options);
        assert!(result.is_err());
    }

    #[test]
    fn test_default_registry() {
        let registry = default_registry();
        assert!(registry.supports_format("html"));
        assert!(registry.supports_format("markdown"));
    }

    #[test]
    fn test_get_writer() {
        let writer = get_writer("html");
        assert!(writer.is_some());

        assert!(get_writer("unknown").is_none());
    }

    #[test]
    fn test_get_writer_by_extension() {
        let writer = get_writer_by_extension("html");
        assert!(writer.is_some());

        assert!(get_writer_by_extension("unknown").is_none());
    }

    #[test]
    fn test_html_writer_trait() {
        let writer = HtmlWriter;
        assert_eq!(writer.format(), OutputFormat::Html);
        assert!(writer.extensions().contains(&"html"));
        assert!(writer.extensions().contains(&"htm"));
        assert!(!writer.extensions().contains(&"md"));
        assert_eq!(writer.mime_type(), "text/html");
    }

    #[test]
    fn test_commonmark_writer_trait() {
        let writer = CommonMarkWriter;
        assert_eq!(writer.format(), OutputFormat::Markdown);
        assert!(writer.extensions().contains(&"md"));
        assert!(writer.extensions().contains(&"markdown"));
        assert!(!writer.extensions().contains(&"html"));
        assert_eq!(writer.mime_type(), "text/markdown");
    }

    #[test]
    fn test_xml_writer_trait() {
        let writer = XmlWriter;
        assert_eq!(writer.format(), OutputFormat::Xml);
        assert!(writer.extensions().contains(&"xml"));
        assert!(!writer.extensions().contains(&"html"));
        assert_eq!(writer.mime_type(), "application/xml");
    }

    #[test]
    fn test_latex_writer_trait() {
        let writer = LatexWriter;
        assert_eq!(writer.format(), OutputFormat::Latex);
        assert!(writer.extensions().contains(&"tex"));
        assert!(writer.extensions().contains(&"latex"));
        assert_eq!(writer.mime_type(), "application/x-latex");
    }

    #[test]
    fn test_man_writer_trait() {
        let writer = ManWriter;
        assert_eq!(writer.format(), OutputFormat::Man);
        assert!(writer.extensions().contains(&"man"));
        assert!(writer.extensions().contains(&"1"));
        assert_eq!(writer.mime_type(), "application/x-troff-man");
    }
}
