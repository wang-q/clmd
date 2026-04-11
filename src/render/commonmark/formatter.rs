//! Main Formatter entry point
//!
//! This module provides the primary `Formatter` struct for CommonMark output generation,
//! along with the convenience `render()` function. The Formatter coordinates multiple
//! node formatters and manages the rendering process.
//!
//! # Usage
//!
//! ```ignore
//! use clmd::render::commonmark::{Formatter, FormatOptions, CommonMarkNodeFormatter};
//!
//! let mut formatter = Formatter::with_options(FormatOptions::new());
//! formatter.add_node_formatter(Box::new(CommonMarkNodeFormatter::new()));
//!
//! let result = formatter.render(&arena, root);
//! ```

use crate::core::arena::{NodeArena, NodeId};
use crate::options::format::FormatOptions;
use crate::render::commonmark::context::MainFormatterContext;
use crate::render::commonmark::core::{ComposedNodeFormatter, NodeFormatter};
use crate::render::commonmark::writer;

/// Main Markdown formatter
///
/// This is the primary entry point for formatting Markdown documents.
/// It coordinates multiple node formatters and manages the rendering process.
pub struct Formatter {
    /// Formatter options
    options: FormatOptions,
    /// Node formatters
    node_formatters: ComposedNodeFormatter,
}

impl std::fmt::Debug for Formatter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Formatter")
            .field("options", &self.options)
            .field("node_formatters", &self.node_formatters)
            .finish_non_exhaustive()
    }
}

impl Formatter {
    /// Create a new formatter with default options
    pub fn new() -> Self {
        Self::with_options(FormatOptions::default())
    }

    /// Create a new formatter with specific options
    pub fn with_options(options: FormatOptions) -> Self {
        Self {
            options,
            node_formatters: ComposedNodeFormatter::new(),
        }
    }

    /// Add a node formatter
    pub fn add_node_formatter(&mut self, formatter: Box<dyn NodeFormatter>) {
        self.node_formatters.add_formatter(formatter);
    }

    /// Render a document
    ///
    /// This is the main entry point for rendering a document tree to Markdown.
    ///
    /// # Arguments
    ///
    /// * `arena` - The NodeArena containing the AST
    /// * `root` - The root node ID
    ///
    /// # Returns
    ///
    /// The rendered Markdown output as a String
    pub fn render(&self, arena: &NodeArena, root: NodeId) -> String {
        let mut writer = writer::MarkdownWriter::new(self.options.format_flags);
        writer.set_max_trailing_blank_lines(self.options.max_trailing_blank_lines);
        writer.set_right_margin(self.options.right_margin);
        let mut context =
            MainFormatterContext::new(arena, &self.options, &self.node_formatters);

        // Main document rendering
        context.render(root, &mut writer);

        writer.to_string()
    }
}

impl Default for Formatter {
    fn default() -> Self {
        Self::new()
    }
}

/// Render a node tree as CommonMark
///
/// This is a convenience function that uses the CommonMarkNodeFormatter via the
/// Formatter framework, which provides a flexible, node-based approach to rendering
/// CommonMark output.
///
/// # Arguments
///
/// * `arena` - The NodeArena containing the AST
/// * `root` - The root node ID
/// * `wrap_width` - Maximum line width for wrapping (0 = no wrapping)
///
/// # Returns
///
/// The CommonMark output as a String
///
/// # Example
///
/// ```ignore
/// use clmd::render::commonmark::render;
/// use clmd::core::arena::NodeArena;
///
/// let arena = NodeArena::new();
/// let root = /* ... create your AST ... */;
/// let markdown = render(&arena, root, 80);
/// ```
pub fn render(arena: &NodeArena, root: NodeId, wrap_width: usize) -> String {
    let opts = FormatOptions::new().with_right_margin(wrap_width);
    let mut formatter = Formatter::with_options(opts);
    formatter.add_node_formatter(Box::new(
        super::commonmark_formatter::CommonMarkNodeFormatter::new(),
    ));
    formatter.render(arena, root)
}
