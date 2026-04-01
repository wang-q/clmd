//! Format-specific renderers for the CommonMark AST.
//!
//! This module provides renderers for various output formats including HTML, XML,
//! LaTeX, Man pages, Typst, DOCX, and PDF.

/// HTML renderer for Arena-based AST.
pub mod html;

/// XML renderer for Arena-based AST.
pub mod xml;

/// LaTeX renderer for Arena-based AST.
pub mod latex;

/// Man page renderer for Arena-based AST.
pub mod man;

/// Typst renderer for Arena-based AST.
pub mod typst;

/// DOCX renderer for Arena-based AST.
pub mod docx;

/// PDF renderer for Arena-based AST.
pub mod pdf;

// Re-export commonly used types from html
pub use html::{
    render as render_html,
    render_with_highlighter as render_html_with_highlighter,
};

// Re-export commonly used types from xml
pub use xml::{
    format_document as format_xml,
    format_document_with_plugins as format_xml_with_plugins,
    render as render_xml,
};
