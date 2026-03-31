//! Writer modules for clmd.
//!
//! This module provides a unified interface for writing clmd's AST to
//! different output formats, inspired by Pandoc's writer architecture.
//!
//! # Supported Formats
//!
//! - HTML
//! - CommonMark
//! - LaTeX
//! - Man page
//! - XML
//!
//! # Example
//!
//! ```
//! use clmd::writers::{WriterRegistry, WriterOptions};
//!
//! let registry = WriterRegistry::with_defaults();
//! assert!(registry.get("html").is_some());
//! ```

use crate::arena::{NodeArena, NodeId};
use crate::error::ClmdError;
use crate::parser::options::Options;

pub mod registry;

pub use registry::{
    default_registry, get_writer, get_writer_by_extension, get_writer_by_path,
    CommonMarkWriter, HtmlWriter, LatexWriter, ManWriter, WriterRegistry, XmlWriter,
};

/// Options for writing documents.
#[derive(Debug, Clone)]
pub struct WriterOptions<'c> {
    /// Base options.
    pub options: Options<'c>,
    /// Whether to produce standalone output.
    pub standalone: bool,
    /// Template to use for standalone output.
    pub template: Option<String>,
    /// CSS styles to include.
    pub css: Vec<String>,
    /// Whether to include syntax highlighting.
    pub highlight: bool,
    /// Tab stop width.
    pub tab_stop: usize,
    /// Column width for wrapping.
    pub columns: usize,
    /// Whether to wrap text.
    pub wrap_text: bool,
    /// DPI for images.
    pub dpi: usize,
    /// Email obfuscation method.
    pub email_obfuscation: EmailObfuscation,
}

/// Email obfuscation methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmailObfuscation {
    /// No obfuscation.
    None,
    /// Use JavaScript obfuscation.
    Javascript,
    /// Use reference-style obfuscation.
    References,
}

impl<'c> Default for WriterOptions<'c> {
    fn default() -> Self {
        Self {
            options: Options::default(),
            standalone: false,
            template: None,
            css: Vec::new(),
            highlight: true,
            tab_stop: 4,
            columns: 80,
            wrap_text: true,
            dpi: 96,
            email_obfuscation: EmailObfuscation::None,
        }
    }
}

impl<'c> WriterOptions<'c> {
    /// Create new writer options with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set whether to produce standalone output.
    pub fn with_standalone(mut self, standalone: bool) -> Self {
        self.standalone = standalone;
        self
    }

    /// Set the template for standalone output.
    pub fn with_template<S: Into<String>>(mut self, template: S) -> Self {
        self.template = Some(template.into());
        self
    }

    /// Add a CSS style.
    pub fn with_css<S: Into<String>>(mut self, css: S) -> Self {
        self.css.push(css.into());
        self
    }

    /// Set whether to include syntax highlighting.
    pub fn with_highlight(mut self, highlight: bool) -> Self {
        self.highlight = highlight;
        self
    }

    /// Set tab stop width.
    pub fn with_tab_stop(mut self, stop: usize) -> Self {
        self.tab_stop = stop;
        self
    }

    /// Set column width.
    pub fn with_columns(mut self, columns: usize) -> Self {
        self.columns = columns;
        self
    }

    /// Set whether to wrap text.
    pub fn with_wrap_text(mut self, wrap: bool) -> Self {
        self.wrap_text = wrap;
        self
    }

    /// Set DPI for images.
    pub fn with_dpi(mut self, dpi: usize) -> Self {
        self.dpi = dpi;
        self
    }

    /// Set email obfuscation method.
    pub fn with_email_obfuscation(mut self, method: EmailObfuscation) -> Self {
        self.email_obfuscation = method;
        self
    }
}

/// A writer that can render to a specific output format.
pub trait Writer: Send + Sync {
    /// Get the name of this writer.
    fn name(&self) -> &'static str;

    /// Get the file extensions supported by this writer.
    fn extensions(&self) -> &[&'static str];

    /// Write a document to a string.
    ///
    /// # Errors
    ///
    /// Returns an error if the document cannot be rendered.
    fn write<'c>(
        &self,
        arena: &NodeArena,
        root: NodeId,
        options: &WriterOptions<'c>,
    ) -> Result<String, ClmdError>;

    /// Check if this writer supports the given file extension.
    fn supports_extension(&self, ext: &str) -> bool {
        self.extensions()
            .iter()
            .any(|e| e.eq_ignore_ascii_case(ext))
    }
}

/// Type alias for a boxed writer.
pub type BoxedWriter = Box<dyn Writer>;

/// Write a document to HTML.
pub fn write_html(
    arena: &NodeArena,
    root: NodeId,
    _options: &WriterOptions<'_>,
) -> Result<String, ClmdError> {
    use crate::render::html;

    let options_flags = 0; // TODO: Convert from WriterOptions
    Ok(html::render(arena, root, options_flags))
}

/// Write a document to CommonMark.
pub fn write_commonmark(
    arena: &NodeArena,
    root: NodeId,
    options: &WriterOptions<'_>,
) -> Result<String, ClmdError> {
    use crate::render::commonmark;

    let options_flags = 0; // TODO: Convert from WriterOptions
    let wrap_width = options.columns;
    Ok(commonmark::render(arena, root, options_flags, wrap_width))
}

/// Write a document to XML.
pub fn write_xml(
    arena: &NodeArena,
    root: NodeId,
    _options: &WriterOptions<'_>,
) -> Result<String, ClmdError> {
    use crate::render::xml;

    let options_flags = 0; // TODO: Convert from WriterOptions
    Ok(xml::render(arena, root, options_flags))
}

/// Write a document to LaTeX.
pub fn write_latex(
    _arena: &NodeArena,
    _root: NodeId,
    _options: &WriterOptions<'_>,
) -> Result<String, ClmdError> {
    // TODO: Implement LaTeX writer
    Err(ClmdError::not_implemented("LaTeX writer"))
}

/// Write a document to Man page format.
pub fn write_man(
    _arena: &NodeArena,
    _root: NodeId,
    _options: &WriterOptions<'_>,
) -> Result<String, ClmdError> {
    // TODO: Implement Man page writer
    Err(ClmdError::not_implemented("Man page writer"))
}
