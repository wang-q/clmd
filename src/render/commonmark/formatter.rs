//! Main Formatter entry point
//!
//! This module provides the primary `Formatter` struct for CommonMark output generation
//! and the convenience `render()` function. The Formatter comes pre-configured with all
//! CommonMark and GFM node handlers, ready to use without additional setup.
//!
//! # Usage
//!
//! ```ignore
//! use clmd::render::commonmark::{Formatter, FormatOptions};
//!
//! let formatter = Formatter::with_options(FormatOptions::new());
//! let result = formatter.render(&arena, root);
//! ```

use crate::core::arena::{NodeArena, NodeId};
use crate::options::format::FormatOptions;
use crate::render::commonmark::context::MainFormatterContext;
use crate::render::commonmark::core::{ComposedNodeFormatter, NodeFormatter};
use crate::render::commonmark::handlers::registration::register_all_handlers;
use crate::render::commonmark::writer;

/// Main Markdown formatter
///
/// This is the primary entry point for formatting Markdown documents.
/// It is pre-configured with all CommonMark and GFM node handlers.
pub struct Formatter {
    options: FormatOptions,
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
    /// Create a new formatter with default options and built-in CommonMark handlers
    pub fn new() -> Self {
        Self::with_options(FormatOptions::default())
    }

    /// Create a new formatter with specific options and built-in CommonMark handlers
    pub fn with_options(options: FormatOptions) -> Self {
        let mut node_formatters = ComposedNodeFormatter::new();
        node_formatters.add_formatter(Box::new(CommonMarkBuiltinFormatter));
        Self {
            options,
            node_formatters,
        }
    }

    /// Add an additional node formatter (for custom/extension handlers)
    pub fn add_node_formatter(&mut self, formatter: Box<dyn NodeFormatter>) {
        self.node_formatters.add_formatter(formatter);
    }

    /// Render a document tree to CommonMark Markdown
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

        context.render(root, &mut writer);

        writer.to_string()
    }
}

impl Default for Formatter {
    fn default() -> Self {
        Self::new()
    }
}

/// Internal builtin formatter that provides all CommonMark/GFM handlers
struct CommonMarkBuiltinFormatter;

impl NodeFormatter for CommonMarkBuiltinFormatter {
    fn get_node_formatting_handlers(
        &self,
    ) -> Vec<crate::render::commonmark::core::NodeFormattingHandler> {
        register_all_handlers()
    }
}

/// Render a node tree as CommonMark
///
/// Convenience function that creates a Formatter with the given wrap width
/// and renders the document.
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
    let formatter = Formatter::with_options(opts);
    formatter.render(arena, root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::commonmark::core::test_utils::MockContext;
    use crate::render::commonmark::escaping::{escape_string, escape_text};

    #[test]
    fn test_formatter_creation_has_handlers() {
        let formatter = Formatter::new();
        assert!(!formatter.node_formatters.get_all_handlers().is_empty());
    }

    #[test]
    fn test_formatter_default() {
        let formatter: Formatter = Default::default();
        assert!(!formatter.node_formatters.get_all_handlers().is_empty());
    }

    #[test]
    fn test_escape_text() {
        let ctx = MockContext::new();
        assert_eq!(escape_text("*text*", &ctx), "\\*text\\*");
        assert_eq!(escape_text("_text_", &ctx), "\\_text\\_");
        assert_eq!(escape_text("[link]", &ctx), "\\[link\\]");
        assert_eq!(escape_text("(paren)", &ctx), "(paren)");
        assert_eq!(escape_text("`code`", &ctx), "\\`code\\`");
    }

    #[test]
    fn test_escape_text_no_special_chars() {
        let ctx = MockContext::new();
        assert_eq!(escape_text("plain text", &ctx), "plain text");
        assert_eq!(escape_text("123", &ctx), "123");
    }

    #[test]
    fn test_escape_string() {
        assert_eq!(escape_string("title"), "title");
        assert_eq!(escape_string("ti\"tle"), "ti\\\\\"tle");
        assert_eq!(escape_string("ti\\tle"), "ti\\\\tle");
    }
}
