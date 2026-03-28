//! Renderer trait and utilities
//!
//! This module provides a common interface for all renderers in the clmd crate.
//! All renderers use the `NodeValue` enum from `node_value` module for AST representation.

use crate::arena::{NodeArena, NodeId};
use crate::nodes::NodeValue;

/// Common trait for all renderers
///
/// This trait defines the interface that all renderers must implement.
/// Renderers convert the AST (represented using `NodeValue`) into various output formats.
///
/// # Example
///
/// ```ignore
/// use clmd::{Renderer, NodeArena, NodeId};
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

pub use super::commonmark;
/// Re-export all renderers
pub use super::html;
pub use super::latex;
pub use super::man;
pub use super::xml;

/// Render to HTML format
///
/// This is a convenience function that uses the HTML renderer.
///
/// # Example
///
/// ```ignore
/// use clmd::{parse_document, render_to_html, parser::options::Options, Arena};
///
/// let mut arena = Arena::new();
/// let options = Options::default();
/// let doc = parse_document(&mut arena, "# Hello", &options);
/// let html = render_to_html(&arena, doc, 0);
/// assert!(html.contains("<h1>"));
/// ```
pub fn render_to_html(arena: &NodeArena, root: NodeId, options: u32) -> String {
    html::render(arena, root, options)
}

/// Render to XML format
///
/// This is a convenience function that uses the XML renderer.
pub fn render_to_xml(arena: &NodeArena, root: NodeId, _options: u32) -> String {
    let mut output = String::new();
    output.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    output.push_str("<!DOCTYPE document SYSTEM \"CommonMark.dtd\">\n");
    render_node_xml(arena, root, &mut output);
    output
}

/// Recursively render a node to XML
fn render_node_xml(arena: &NodeArena, node_id: NodeId, output: &mut String) {
    let node = arena.get(node_id);
    let tag_name = node.value.xml_node_name();

    output.push('<');
    output.push_str(tag_name);

    // Handle leaf nodes with content
    if node.value.is_leaf() {
        match &node.value {
            NodeValue::Text(text) => {
                if !text.is_empty() {
                    output.push('>');
                    output.push_str(&escape_xml(text));
                    output.push_str("</");
                    output.push_str(tag_name);
                    output.push('>');
                } else {
                    output.push_str(" />");
                }
            }
            NodeValue::Code(code) => {
                if !code.literal.is_empty() {
                    output.push('>');
                    output.push_str(&escape_xml(&code.literal));
                    output.push_str("</");
                    output.push_str(tag_name);
                    output.push('>');
                } else {
                    output.push_str(" />");
                }
            }
            NodeValue::CodeBlock(code) => {
                if !code.literal.is_empty() {
                    output.push('>');
                    output.push_str(&escape_xml(&code.literal));
                    output.push_str("</");
                    output.push_str(tag_name);
                    output.push('>');
                } else {
                    output.push_str(" />");
                }
            }
            _ => {
                output.push_str(" />");
            }
        }
    } else {
        output.push('>');

        // Render children
        let mut child_id = node.first_child;
        while let Some(child) = child_id {
            render_node_xml(arena, child, output);
            child_id = arena.get(child).next;
        }

        output.push_str("</");
        output.push_str(tag_name);
        output.push('>');
    }
}

/// Escape XML special characters
fn escape_xml(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&apos;"),
            _ => result.push(c),
        }
    }
    result
}

/// Render to CommonMark format
///
/// This is a convenience function that uses the CommonMark renderer.
pub fn render_to_commonmark(arena: &NodeArena, root: NodeId, options: u32) -> String {
    commonmark::render(arena, root, options)
}

/// Render to LaTeX format
///
/// This is a convenience function that uses the LaTeX renderer.
pub fn render_to_latex(arena: &NodeArena, root: NodeId, options: u32) -> String {
    latex::render(arena, root, options)
}

/// Render to Man page format
///
/// This is a convenience function that uses the Man page renderer.
pub fn render_to_man(arena: &NodeArena, root: NodeId, options: u32) -> String {
    man::render(arena, root, options)
}

/// Available output formats
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
    /// Man page output
    Man,
}

/// Render to the specified format
///
/// # Arguments
///
/// * `format` - The desired output format
/// * `arena` - The node arena containing the AST
/// * `root` - The root node ID
/// * `options` - Rendering options
///
/// # Returns
///
/// The rendered output as a String
///
/// # Example
///
/// ```ignore
/// use clmd::{parse_document, render, OutputFormat, parser::options::Options, Arena};
///
/// let mut arena = Arena::new();
/// let options = Options::default();
/// let doc = parse_document(&mut arena, "# Hello", &options);
/// let html = render(OutputFormat::Html, &arena, doc, 0);
/// assert!(html.contains("<h1>"));
/// ```
pub fn render(
    format: OutputFormat,
    arena: &NodeArena,
    root: NodeId,
    options: u32,
) -> String {
    match format {
        OutputFormat::Html => render_to_html(arena, root, options),
        OutputFormat::Xml => render_to_xml(arena, root, options),
        OutputFormat::CommonMark => render_to_commonmark(arena, root, options),
        OutputFormat::Latex => render_to_latex(arena, root, options),
        OutputFormat::Man => render_to_man(arena, root, options),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arena::{Node, NodeArena, TreeOps};
    use crate::nodes::NodeValue;

    #[test]
    fn test_output_format_enum() {
        assert_eq!(OutputFormat::Html as u8, 0);
        assert_eq!(OutputFormat::Xml as u8, 1);
        assert_eq!(OutputFormat::CommonMark as u8, 2);
        assert_eq!(OutputFormat::Latex as u8, 3);
        assert_eq!(OutputFormat::Man as u8, 4);
    }

    #[test]
    fn test_render_to_html() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("Hello")));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, text);

        let html = render_to_html(&arena, root, 0);
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

        let html = render(OutputFormat::Html, &arena, root, 0);
        println!("HTML output: {:?}", html);
        assert!(html.contains("<p>Hello</p>"));

        let xml = render(OutputFormat::Xml, &arena, root, 0);
        println!("XML output: {:?}", xml);
        assert!(xml.contains("<paragraph>"));
    }
}
