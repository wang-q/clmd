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
//! ```ignore
//! use clmd::{markdown_to_html, parser::options::Options};
//!
//! let options = Options::default();
//! let html = markdown_to_html("# Hello\n\nWorld", &options);
//! assert!(html.contains("<h1>Hello</h1>"));
//! assert!(html.contains("<p>World</p>"));
//! ```

pub mod commonmark;
pub mod docx;
pub mod html;
pub mod latex;
pub mod man;
pub mod pdf;
pub mod renderer;
pub mod table_formatter;
pub mod typst;
pub mod xml;

// Re-export renderer types
pub use renderer::{
    render, render_to_commonmark, render_to_html, render_to_latex, render_to_man,
    render_to_xml, OutputFormat, Renderer,
};

// Re-export escape_html from html_utils for backward compatibility
pub use crate::html_utils::escape_html;

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
