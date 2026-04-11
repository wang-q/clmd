//! Rendering modules for Arena-based AST
//!
//! This module provides output generation for documents parsed using the Arena-based parser.
//! Supported formats: HTML, XML, CommonMark, and LaTeX.
//!
//! # Overview
//!
//! Each renderer traverses the AST and generates output in its respective format:
//!
//! - **HTML**: Web-ready markup
//! - **XML**: Structured data representation
//! - **CommonMark**: Round-trip Markdown format
//! - **LaTeX**: Typesetting for academic documents
//!
//! # Example
//!
//! ```ignore
//! use clmd::{markdown_to_html, options::Options};
//!
//! let options = Options::default();
//! let html = markdown_to_html("# Hello\n\nWorld", &options);
//! assert!(html.contains("<h1>Hello</h1>"));
//! assert!(html.contains("<p>World</p>"));
//! ```

use crate::core::arena::{NodeArena, NodeId};
use crate::options::Options;

pub mod commonmark;
pub mod html;

pub use crate::options::format as options;

/// Common trait for all renderers
///
/// This trait defines the interface that all renderers must implement.
/// Renderers convert the AST (represented using `NodeValue`) into various output formats.
///
/// # Example
///
/// ```ignore
/// use clmd::render::Renderer;
/// use clmd::arena::NodeArena;
/// use clmd::arena::NodeId;
///
/// fn render_document<R: Renderer>(renderer: &R, arena: &NodeArena, root: NodeId) -> String {
///     renderer.render(arena, root, 0)
/// }
/// ```
pub trait Renderer {
    /// Render the document tree to a string
    ///
    /// # Arguments
    ///
    /// * `arena` - The node arena containing the AST
    /// * `root` - The root node ID
    /// * `options` - Rendering options
    ///
    /// # Returns
    ///
    /// The rendered output as a String
    fn render(&self, arena: &NodeArena, root: NodeId, options: u32) -> String;
}

/// Render to HTML format
///
/// This is a convenience function that uses the HTML renderer.
///
/// # Example
///
/// ```ignore
/// use clmd::{parse_document, render::render_to_html, Options};
/// use clmd::Arena;
///
/// let options = Options::default();
/// let (arena, doc) = parse_document("# Hello", &options);
/// let html = render_to_html(&arena, doc, &options);
/// assert!(html.contains("<h1>"));
/// ```
pub fn render_to_html(arena: &NodeArena, root: NodeId, options: &Options) -> String {
    html::render(arena, root, options)
}

/// Render to XML format
///
/// This is a convenience function that uses the XML renderer.
pub fn render_to_xml(arena: &NodeArena, root: NodeId) -> String {
    crate::io::writer::xml::render(arena, root, 0)
}

/// Render to CommonMark format
///
/// This is a convenience function that uses the CommonMark renderer.
pub fn render_to_commonmark(arena: &NodeArena, root: NodeId, width: usize) -> String {
    commonmark::render(arena, root, width)
}

/// Render to CommonMark format with formatter options
///
/// This is a convenience function that uses the new formatter with custom options.
///
/// # Example
///
/// ```ignore
/// use clmd::{parse_document, render::render_to_commonmark_with_options};
/// use clmd::options::format::FormatterOptions;
///
/// let options = FormatterOptions::new()
///     .with_heading_style(clmd::options::format::HeadingStyle::Atx)
///     .with_right_margin(80);
/// let (arena, root) = parse_document("# Hello", &Default::default());
/// let cm = render_to_commonmark_with_options(&arena, root, options);
/// ```
pub fn render_to_commonmark_with_options(
    arena: &NodeArena,
    root: NodeId,
    options: options::FormatOptions,
) -> String {
    let formatter = commonmark::Formatter::with_options(options);
    formatter.render(arena, root)
}

/// Render to LaTeX format
///
/// This is a convenience function that uses the LaTeX renderer.
pub fn render_to_latex(arena: &NodeArena, root: NodeId, options: u32) -> String {
    crate::io::writer::latex::render(arena, root, options)
}

/// Available output formats
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// HTML output
    Html,
    /// XML output (for debugging)
    Xml,
    /// CommonMark output
    CommonMark,
    /// LaTeX output
    Latex,
}

/// Render to the specified format
///
/// # Arguments
///
/// * `format` - The desired output format
/// * `arena` - The node arena containing the AST
/// * `root` - The root node ID
/// * `options` - Rendering options
/// * `width` - Line width for CommonMark wrapping (0 = no wrapping)
///
/// # Returns
///
/// The rendered output as a String
///
/// # Example
///
/// ```ignore
/// use clmd::{parse_document, render, OutputFormat, options::Options, Arena};
///
/// let mut arena = Arena::new();
/// let options = Options::default();
/// let doc = parse_document(&mut arena, "# Hello", &options);
/// let html = render(OutputFormat::Html, &arena, doc, &options, 0);
/// assert!(html.contains("<h1>"));
/// ```
pub fn render(
    format: OutputFormat,
    arena: &NodeArena,
    root: NodeId,
    options: &Options,
    width: usize,
) -> String {
    match format {
        OutputFormat::Html => render_to_html(arena, root, options),
        OutputFormat::Xml => render_to_xml(arena, root),
        OutputFormat::CommonMark => render_to_commonmark(arena, root, width),
        OutputFormat::Latex => render_to_latex(arena, root, 0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::arena::{Node, NodeArena, TreeOps};
    use crate::core::nodes::NodeValue;
    use crate::text::html_utils::escape_html;

    #[test]
    fn test_escape_html() {
        assert_eq!(escape_html("<script>"), "&lt;script&gt;");
        assert_eq!(escape_html("&"), "&amp;");
        assert_eq!(escape_html("\"test\""), "&quot;test&quot;");
    }

    #[test]
    fn test_output_format_enum() {
        assert_eq!(OutputFormat::Html as u8, 0);
        assert_eq!(OutputFormat::Xml as u8, 1);
        assert_eq!(OutputFormat::CommonMark as u8, 2);
        assert_eq!(OutputFormat::Latex as u8, 3);
    }

    #[test]
    fn test_render_to_html() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("Hello")));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, text);

        let options = Options::default();
        let html = render_to_html(&arena, root, &options);
        assert!(html.contains("<p>Hello</p>"));
    }

    #[test]
    fn test_render_dispatch() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("Hello")));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, text);

        let options = Options::default();
        let html = render(OutputFormat::Html, &arena, root, &options, 0);
        println!("HTML output: {:?}", html);
        assert!(html.contains("<p>Hello</p>"));

        let xml = render(OutputFormat::Xml, &arena, root, &options, 0);
        println!("XML output: {:?}", xml);
        assert!(xml.contains("<paragraph>"));
    }

    #[test]
    fn test_render_all_formats() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("Hello")));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, text);

        let options = Options::default();

        // Test all output formats
        let html = render(OutputFormat::Html, &arena, root, &options, 0);
        assert!(html.contains("<p>Hello</p>"));

        let xml = render(OutputFormat::Xml, &arena, root, &options, 0);
        assert!(xml.contains("<paragraph>"));

        let commonmark = render(OutputFormat::CommonMark, &arena, root, &options, 0);
        assert!(commonmark.contains("Hello"));

        let latex = render(OutputFormat::Latex, &arena, root, &options, 0);
        assert!(latex.contains("\\par") || latex.contains("Hello"));
    }

    #[test]
    fn test_render_empty_document() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));

        TreeOps::append_child(&mut arena, root, para);

        let options = Options::default();
        let html = render_to_html(&arena, root, &options);
        // Document with empty paragraph should produce valid HTML
        assert!(!html.is_empty());
    }

    #[test]
    fn test_output_format_debug() {
        assert!(format!("{:?}", OutputFormat::Html).contains("Html"));
        assert!(format!("{:?}", OutputFormat::Xml).contains("Xml"));
        assert!(format!("{:?}", OutputFormat::CommonMark).contains("CommonMark"));
        assert!(format!("{:?}", OutputFormat::Latex).contains("Latex"));
    }

    #[test]
    fn test_renderer_trait_methods() {
        // Test that Renderer functions exist and can be called
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("Test")));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, text);

        let options = Options::default();

        // Test render_to_string via HtmlRenderer
        let html = render_to_html(&arena, root, &options);
        assert!(!html.is_empty());
    }
}
