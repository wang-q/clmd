//! Rendering modules for Arena-based AST
//!
//! This module provides output generation for documents parsed using the Arena-based parser.
//! Supported formats: HTML, XML, CommonMark, LaTeX, and Man page.
//!
//! # Overview
//!
//! Each renderer traverses the AST and generates output in its respective format:
//!
//! - **HTML**: Web-ready markup
//! - **XML**: Structured data representation
//! - **CommonMark**: Round-trip Markdown format
//! - **LaTeX**: Typesetting for academic documents
//! - **Man**: Unix manual page format
//!
//! # Example
//!
//! ```
//! use clmd::{markdown_to_html, options};
//!
//! let html = markdown_to_html("# Hello\n\nWorld", options::DEFAULT);
//! assert_eq!(html, "<h1>Hello</h1>\n<p>World</p>");
//! ```

pub mod commonmark;
pub mod html;
pub mod latex;
pub mod man;
pub mod renderer;
pub mod xml;

// Re-export renderer types
pub use renderer::{
    CommonMarkRenderer, HtmlRenderer, LatexRenderer, ManRenderer, Renderer,
    StreamingRenderer, XmlRenderer,
};

/// Escape HTML special characters
pub fn escape_html(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            _ => result.push(c),
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_html() {
        assert_eq!(escape_html("<script>"), "&lt;script&gt;");
        assert_eq!(escape_html("&"), "&amp;");
        assert_eq!(escape_html("\"test\""), "&quot;test&quot;");
    }
}
