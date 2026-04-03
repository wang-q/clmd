//! Format-specific renderers for the CommonMark AST.
//!
//! This module provides renderers for various output formats including HTML, LaTeX, and PDF.

/// HTML renderer for Arena-based AST.
pub mod html;

/// LaTeX renderer for Arena-based AST.
pub mod latex;

/// PDF renderer for Arena-based AST.
pub mod pdf;

// Re-export commonly used types from html
pub use html::{
    render as render_html, render_with_highlighter as render_html_with_highlighter,
};
