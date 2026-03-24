//! Render compatibility module
//!
//! Provides integration between the new AST system and the existing renderers.

use crate::ast::node::Node;
use crate::node::{Node as OldNode, SourcePos as OldSourcePos};
use std::cell::RefCell;
use std::rc::Rc;

/// Convert a new-style AST node to an old-style node
///
/// This is useful for integrating with existing renderers that expect the old node format.
pub fn to_old_node(_node: &Node) -> Option<OldNode> {
    // This is a placeholder implementation
    // In a full implementation, we would need type information to properly convert
    None
}

/// Adapter for rendering new-style AST nodes
///
/// This provides a bridge between the new AST system and existing renderers.
pub struct RenderAdapter {
    root: Rc<RefCell<Node>>,
}

impl RenderAdapter {
    /// Create a new render adapter
    pub fn new(root: Rc<RefCell<Node>>) -> Self {
        Self { root }
    }

    /// Get the root node
    pub fn root(&self) -> &Rc<RefCell<Node>> {
        &self.root
    }

    /// Render the AST to HTML using the existing HTML renderer
    ///
    /// This is a placeholder - full implementation would integrate with html.rs
    pub fn to_html(&self) -> String {
        // Placeholder implementation
        String::from("<html></html>")
    }

    /// Render the AST to XML
    ///
    /// This is a placeholder - full implementation would integrate with xml.rs
    pub fn to_xml(&self) -> String {
        // Placeholder implementation
        String::from("<?xml version=\"1.0\"?><document></document>")
    }
}

/// Extension trait for Node to provide rendering capabilities
pub trait RenderExt {
    /// Render this node to HTML
    fn render_html(&self) -> String;

    /// Render this node to XML
    fn render_xml(&self) -> String;

    /// Get the source position as old-style
    fn source_pos_old(&self) -> OldSourcePos;
}

impl RenderExt for Node {
    fn render_html(&self) -> String {
        // Placeholder - would integrate with actual HTML renderer
        String::new()
    }

    fn render_xml(&self) -> String {
        // Placeholder - would integrate with actual XML renderer
        String::new()
    }

    fn source_pos_old(&self) -> OldSourcePos {
        let pos = self.source_pos();
        OldSourcePos {
            start_line: pos.start_line,
            start_column: pos.start_column,
            end_line: pos.end_line,
            end_column: pos.end_column,
        }
    }
}

/// Utility functions for rendering
pub mod util {

    /// Escape HTML special characters
    pub fn escape_html(text: &str) -> String {
        text.chars()
            .map(|c| match c {
                '<' => "&lt;".to_string(),
                '>' => "&gt;".to_string(),
                '&' => "&amp;".to_string(),
                '"' => "&quot;".to_string(),
                '\'' => "&#x27;".to_string(),
                _ => c.to_string(),
            })
            .collect()
    }

    /// Generate an HTML tag
    pub fn html_tag(tag: &str, content: &str, attrs: &[(&str, &str)]) -> String {
        let mut result = String::with_capacity(tag.len() + content.len() + 10);
        result.push('<');
        result.push_str(tag);

        for (key, value) in attrs {
            result.push(' ');
            result.push_str(key);
            result.push_str("=\"");
            result.push_str(&escape_html(value));
            result.push('"');
        }

        if content.is_empty() {
            result.push_str(" />");
        } else {
            result.push('>');
            result.push_str(content);
            result.push_str("</");
            result.push_str(tag);
            result.push('>');
        }

        result
    }

    /// Generate an XML tag
    pub fn xml_tag(tag: &str, content: &str, attrs: &[(&str, &str)]) -> String {
        html_tag(tag, content, attrs)
    }
}

#[cfg(test)]
mod tests {
    use super::util::*;
    use super::*;

    #[test]
    fn test_escape_html() {
        assert_eq!(escape_html("<div>"), "&lt;div&gt;");
        assert_eq!(escape_html("&"), "&amp;");
        assert_eq!(escape_html("\"test\""), "&quot;test&quot;");
    }

    #[test]
    fn test_html_tag() {
        let tag = html_tag("p", "Hello", &[]);
        assert_eq!(tag, "<p>Hello</p>");

        let tag_with_attrs = html_tag("a", "Link", &[("href", "https://example.com")]);
        assert_eq!(tag_with_attrs, "<a href=\"https://example.com\">Link</a>");

        let empty_tag = html_tag("br", "", &[]);
        assert_eq!(empty_tag, "<br />");
    }

    #[test]
    fn test_render_adapter() {
        let root = Rc::new(RefCell::new(Node::new()));
        let adapter = RenderAdapter::new(root);

        assert!(adapter.to_html().contains("html"));
        assert!(adapter.to_xml().contains("xml"));
    }
}
