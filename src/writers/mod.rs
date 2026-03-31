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
//! use clmd::writers::{WriterRegistry, WriterOptions, WriterOutput};
//!
//! let registry = WriterRegistry::with_defaults();
//! assert!(registry.get("html").is_some());
//!
//! // Write output to file
//! let output = WriterOutput::text("<h1>Hello</h1>");
//! output.write_to_file("output.html").unwrap();
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

/// Output type for writers.
///
/// Writers can produce either text or binary output, depending on the format.
/// This enum is similar to Pandoc's Writer type which handles both text and
/// bytestring outputs.
#[derive(Debug, Clone)]
pub enum WriterOutput {
    /// Text output (UTF-8 encoded).
    Text(String),
    /// Binary output.
    Binary(Vec<u8>),
}

impl WriterOutput {
    /// Create a text output.
    pub fn text<S: Into<String>>(text: S) -> Self {
        Self::Text(text.into())
    }

    /// Create a binary output.
    pub fn binary<B: Into<Vec<u8>>>(bytes: B) -> Self {
        Self::Binary(bytes.into())
    }

    /// Check if this is text output.
    pub fn is_text(&self) -> bool {
        matches!(self, Self::Text(_))
    }

    /// Check if this is binary output.
    pub fn is_binary(&self) -> bool {
        matches!(self, Self::Binary(_))
    }

    /// Get the text content if this is text output.
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text(text) => Some(text),
            Self::Binary(_) => None,
        }
    }

    /// Get the binary content if this is binary output.
    pub fn as_binary(&self) -> Option<&[u8]> {
        match self {
            Self::Text(_) => None,
            Self::Binary(bytes) => Some(bytes),
        }
    }

    /// Convert to text, decoding if necessary.
    ///
    /// For binary output, this attempts UTF-8 decoding and may fail.
    pub fn to_text(self) -> Result<String, ClmdError> {
        match self {
            Self::Text(text) => Ok(text),
            Self::Binary(bytes) => String::from_utf8(bytes)
                .map_err(|e| ClmdError::encoding_error(format!("Invalid UTF-8: {}", e))),
        }
    }

    /// Convert to binary.
    ///
    /// For text output, this encodes as UTF-8.
    pub fn to_binary(self) -> Vec<u8> {
        match self {
            Self::Text(text) => text.into_bytes(),
            Self::Binary(bytes) => bytes,
        }
    }

    /// Get the size of the output in bytes.
    pub fn len(&self) -> usize {
        match self {
            Self::Text(text) => text.len(),
            Self::Binary(bytes) => bytes.len(),
        }
    }

    /// Check if the output is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Write the output to a file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written.
    pub fn write_to_file<P: AsRef<std::path::Path>>(
        &self,
        path: P,
    ) -> Result<(), ClmdError> {
        use std::fs;
        use std::io::Write;

        let path = path.as_ref();

        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                ClmdError::io_error(format!(
                    "Cannot create directory {}: {}",
                    parent.display(),
                    e
                ))
            })?;
        }

        let mut file = fs::File::create(path).map_err(|e| {
            ClmdError::io_error(format!("Cannot create file {}: {}", path.display(), e))
        })?;

        match self {
            Self::Text(text) => {
                file.write_all(text.as_bytes()).map_err(|e| {
                    ClmdError::io_error(format!(
                        "Cannot write to file {}: {}",
                        path.display(),
                        e
                    ))
                })?;
            }
            Self::Binary(bytes) => {
                file.write_all(bytes).map_err(|e| {
                    ClmdError::io_error(format!(
                        "Cannot write to file {}: {}",
                        path.display(),
                        e
                    ))
                })?;
            }
        }

        Ok(())
    }

    /// Write the output to stdout.
    ///
    /// # Errors
    ///
    /// Returns an error if writing to stdout fails.
    pub fn write_to_stdout(&self) -> Result<(), ClmdError> {
        use std::io::{self, Write};

        let stdout = io::stdout();
        let mut handle = stdout.lock();

        match self {
            Self::Text(text) => {
                handle.write_all(text.as_bytes()).map_err(|e| {
                    ClmdError::io_error(format!("Cannot write to stdout: {}", e))
                })?;
            }
            Self::Binary(bytes) => {
                handle.write_all(bytes).map_err(|e| {
                    ClmdError::io_error(format!("Cannot write to stdout: {}", e))
                })?;
            }
        }

        Ok(())
    }
}

impl From<String> for WriterOutput {
    fn from(text: String) -> Self {
        Self::Text(text)
    }
}

impl From<&str> for WriterOutput {
    fn from(text: &str) -> Self {
        Self::Text(text.to_string())
    }
}

impl From<Vec<u8>> for WriterOutput {
    fn from(bytes: Vec<u8>) -> Self {
        Self::Binary(bytes)
    }
}

impl From<&[u8]> for WriterOutput {
    fn from(bytes: &[u8]) -> Self {
        Self::Binary(bytes.to_vec())
    }
}

/// A writer that can render to a specific output format.
///
/// This trait is inspired by Pandoc's Writer type, which supports both
/// text and binary output formats.
pub trait Writer: Send + Sync {
    /// Get the name of this writer.
    fn name(&self) -> &'static str;

    /// Get the file extensions supported by this writer.
    fn extensions(&self) -> &[&'static str];

    /// Check if this writer produces binary output.
    ///
    /// Default is `false` - most writers produce text output.
    fn produces_binary(&self) -> bool {
        false
    }

    /// Write a document to text output.
    ///
    /// # Errors
    ///
    /// Returns an error if the document cannot be rendered.
    fn write_text<'c>(
        &self,
        arena: &NodeArena,
        root: NodeId,
        options: &WriterOptions<'c>,
    ) -> Result<String, ClmdError>;

    /// Write a document to binary output.
    ///
    /// Default implementation encodes text as UTF-8. Writers that produce
    /// native binary formats (like DOCX) should override this.
    ///
    /// # Errors
    ///
    /// Returns an error if the document cannot be rendered.
    fn write_binary<'c>(
        &self,
        arena: &NodeArena,
        root: NodeId,
        options: &WriterOptions<'c>,
    ) -> Result<Vec<u8>, ClmdError> {
        self.write_text(arena, root, options)
            .map(|text| text.into_bytes())
    }

    /// Write a document to any output type.
    ///
    /// This method automatically produces text or binary output based on
    /// the writer's capabilities.
    ///
    /// # Errors
    ///
    /// Returns an error if the document cannot be rendered.
    fn write<'c>(
        &self,
        arena: &NodeArena,
        root: NodeId,
        options: &WriterOptions<'c>,
    ) -> Result<WriterOutput, ClmdError> {
        if self.produces_binary() {
            self.write_binary(arena, root, options)
                .map(WriterOutput::Binary)
        } else {
            self.write_text(arena, root, options)
                .map(WriterOutput::Text)
        }
    }

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
