//! Format-specific renderers for the CommonMark AST.
//!
//! This module provides renderers for various output formats including HTML and PDF.

/// HTML renderer for Arena-based AST.
pub mod html;

/// PDF renderer for Arena-based AST.
pub mod pdf;

// Re-export commonly used types from html
pub use html::{
    render as render_html, render_with_highlighter as render_html_with_highlighter,
};
