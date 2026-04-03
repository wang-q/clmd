//! HTML renderer for the CommonMark AST
//!
//! This module provides HTML output generation for documents parsed using the Arena-based parser.
//! It supports all standard CommonMark elements plus GFM extensions.
//!
//! # Example
//!
//! ```ignore
//! use clmd::{parse_document, options::Options};
//! use clmd::render::html::render;
//!
//! let options = Options::default();
//! let arena = parse_document("# Hello\n\nWorld", &options);
//! let html = render(&arena, 0, &options);
//! assert!(html.contains("<h1>Hello</h1>"));
//! assert!(html.contains("<p>World</p>"));
//! ```
//!
//! # Submodules
//!
//! - `escaping`: HTML escaping utilities
//! - `renderer`: Main HTML renderer implementation
//! - `nodes`: Node handling logic
//! - `code`: Code block rendering
//! - `footnote`: Footnote rendering
//! - `tests`: Unit tests

pub mod escaping;
mod renderer;

// Import submodules that extend HtmlRenderer with additional methods
mod code;
mod footnote;
mod nodes;
mod table;
mod tests;

use crate::core::adapter::SyntaxHighlighterAdapter;
use crate::core::arena::{NodeArena, NodeId};
use crate::options::Options;

/// Render a node tree as HTML
///
/// # Arguments
///
/// * `arena` - The NodeArena containing the AST
/// * `root` - The root node ID
/// * `options` - Rendering options
///
/// # Returns
///
/// The HTML output as a String
pub fn render(arena: &NodeArena, root: NodeId, options: &Options) -> String {
    let mut renderer = renderer::HtmlRenderer::new(arena, options, None);
    renderer.render(root)
}

/// Render a node tree as HTML with syntax highlighter
///
/// # Arguments
///
/// * `arena` - The NodeArena containing the AST
/// * `root` - The root node ID
/// * `options` - Rendering options
/// * `highlighter` - Optional syntax highlighter adapter
///
/// # Returns
///
/// The HTML output as a String
pub fn render_with_highlighter<'a>(
    arena: &'a NodeArena,
    root: NodeId,
    options: &'a Options,
    highlighter: Option<&'a dyn SyntaxHighlighterAdapter>,
) -> String {
    let mut renderer = renderer::HtmlRenderer::new(arena, options, highlighter);
    renderer.render(root)
}
