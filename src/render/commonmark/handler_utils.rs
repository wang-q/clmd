//! Handler factory functions and context helpers
//!
//! This module provides convenience functions to reduce boilerplate when
//! creating NodeFormattingHandler instances and working with formatter context.

use crate::core::nodes::NodeValue;
use crate::options::format::FormatOptions;
use crate::render::commonmark::context::NodeFormatterContext;
use crate::render::commonmark::node::{NodeFormattingHandler, NodeValueType};
use crate::render::commonmark::writer::MarkdownWriter;
use std::fmt;

// ============================================================================
// Error Types
// ============================================================================

/// Errors that can occur during formatting
#[derive(Debug, Clone)]
pub enum FormatterError {
    /// A feature is not yet implemented
    NotImplemented(String),
    /// An invalid node was encountered
    InvalidNode(String),
    /// An error occurred during rendering
    RenderError(String),
}

impl fmt::Display for FormatterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FormatterError::NotImplemented(msg) => write!(f, "Not implemented: {}", msg),
            FormatterError::InvalidNode(msg) => write!(f, "Invalid node: {}", msg),
            FormatterError::RenderError(msg) => write!(f, "Render error: {}", msg),
        }
    }
}

impl std::error::Error for FormatterError {}

// ============================================================================
// Handler Factory Functions
// ============================================================================

/// Create a simple handler for a node type that only needs open formatting
///
/// This is a convenience function to reduce boilerplate when creating
/// NodeFormattingHandler instances.
///
/// # Example
///
/// ```ignore
/// use clmd::render::commonmark::handler_utils::create_simple_handler;
/// use clmd::render::commonmark::node::NodeValueType;
///
/// let handler = create_simple_handler(NodeValueType::Document, |value, ctx, writer| {
///     // Handle document node
/// });
/// ```
#[inline]
pub fn create_simple_handler<F>(
    node_type: NodeValueType,
    handler: F,
) -> NodeFormattingHandler
where
    F: Fn(&NodeValue, &mut dyn NodeFormatterContext, &mut MarkdownWriter)
        + Send
        + Sync
        + 'static,
{
    NodeFormattingHandler::new(node_type, Box::new(handler))
}

/// Create a handler with both open and close formatting
///
/// This is a convenience function to reduce boilerplate when creating
/// NodeFormattingHandler instances that need both open and close handlers.
///
/// # Example
///
/// ```ignore
/// use clmd::render::commonmark::handler_utils::create_handler_with_close;
/// use clmd::render::commonmark::node::NodeValueType;
///
/// let handler = create_handler_with_close(
///     NodeValueType::Paragraph,
///     |value, ctx, writer| {
///         // Opening logic
///     },
///     |value, ctx, writer| {
///         // Closing logic
///     },
/// );
/// ```
#[inline]
pub fn create_handler_with_close<Open, Close>(
    node_type: NodeValueType,
    on_open: Open,
    on_close: Close,
) -> NodeFormattingHandler
where
    Open: Fn(&NodeValue, &mut dyn NodeFormatterContext, &mut MarkdownWriter)
        + Send
        + Sync
        + 'static,
    Close: Fn(&NodeValue, &mut dyn NodeFormatterContext, &mut MarkdownWriter)
        + Send
        + Sync
        + 'static,
{
    NodeFormattingHandler::with_close(node_type, Box::new(on_open), Box::new(on_close))
}

// ============================================================================
// Context Helper Functions
// ============================================================================

/// Check if we should add a blank line after a block element
///
/// This helper function encapsulates the common logic for determining
/// whether a blank line should be added after a block element based on
/// the current context and whether there are more siblings.
#[inline]
pub fn should_add_blank_line_after_block(
    ctx: &dyn NodeFormatterContext,
    has_next_sibling: bool,
) -> bool {
    !ctx.is_in_tight_list() && has_next_sibling
}

/// Check if we're in a list context (either as a list item or in a tight list)
///
/// This helper function checks if the current context is within a list,
/// either directly as a list item or indirectly in a tight list.
#[inline]
pub fn is_in_list_context(ctx: &dyn NodeFormatterContext) -> bool {
    ctx.is_parent_list_item() || ctx.is_in_tight_list()
}

/// Get formatter options from context
///
/// This is a convenience wrapper to reduce verbosity when accessing
/// formatter options from the context.
#[inline]
pub fn get_options(ctx: &dyn NodeFormatterContext) -> &FormatOptions {
    ctx.get_formatter_options()
}

/// Check if paragraph line breaking is active
///
/// This is a convenience wrapper to reduce verbosity when checking
/// if paragraph line breaking is currently active.
#[inline]
pub fn is_line_breaking_active(ctx: &dyn NodeFormatterContext) -> bool {
    ctx.is_paragraph_line_breaking()
}

// ============================================================================
// CJK Character Handling
// ============================================================================

/// Check if a character is a CJK (Chinese, Japanese, Korean) character
///
/// This function uses the text module's CJK detection.
#[inline]
pub fn is_cjk_char(c: char) -> bool {
    crate::text::char::is_cjk(c)
}

/// Check if previous and next siblings are markdown markers
///
/// This checks if the previous and next siblings are Emph or Strong markers,
/// which should not have spaces around them in CJK text.
/// Note: Code and Link should have spaces around them in CJK text.
pub fn check_sibling_markers(ctx: &dyn NodeFormatterContext) -> (bool, bool) {
    use crate::core::nodes::NodeValue;

    let mut prev_is_marker = false;
    let mut next_is_marker = false;

    if let Some(current_node) = ctx.get_current_node() {
        let arena = ctx.get_arena();
        let node = arena.get(current_node);

        // Check if parent is Emph or Strong (for text inside emphasis)
        let parent_is_emph_or_strong = if let Some(parent_id) = node.parent {
            let parent_node = arena.get(parent_id);
            matches!(parent_node.value, NodeValue::Emph | NodeValue::Strong)
        } else {
            false
        };

        // Check previous sibling - only Emph and Strong
        // Code and Link should have spaces around them
        if let Some(prev_id) = node.prev {
            let prev_node = arena.get(prev_id);
            prev_is_marker =
                matches!(prev_node.value, NodeValue::Emph | NodeValue::Strong);
        } else if parent_is_emph_or_strong {
            // If no previous sibling but parent is a marker,
            // then the previous node is the parent's opening marker
            prev_is_marker = true;
        }

        // Check next sibling - only Emph and Strong
        // Code and Link should have spaces around them
        if let Some(next_id) = node.next {
            let next_node = arena.get(next_id);
            next_is_marker =
                matches!(next_node.value, NodeValue::Emph | NodeValue::Strong);
        } else if parent_is_emph_or_strong {
            // If no next sibling but parent is a marker,
            // then the next node is the parent's closing marker
            next_is_marker = true;
        }
    }

    (prev_is_marker, next_is_marker)
}

/// Check if previous sibling is a Link (for CJK spacing)
pub fn prev_is_link(ctx: &dyn NodeFormatterContext) -> bool {
    use crate::core::nodes::NodeValue;

    if let Some(current_node) = ctx.get_current_node() {
        let arena = ctx.get_arena();
        let node = arena.get(current_node);

        if let Some(prev_id) = node.prev {
            let prev_node = arena.get(prev_id);
            return matches!(prev_node.value, NodeValue::Link(_));
        }
    }

    false
}

/// Check if text ends with CJK character
#[allow(dead_code)]
pub fn ends_with_cjk(text: &str) -> bool {
    text.chars()
        .rev()
        .find(|c| !c.is_whitespace())
        .map_or(false, is_cjk_char)
}

/// Check if text starts with CJK character
#[allow(dead_code)]
pub fn starts_with_cjk(text: &str) -> bool {
    text.chars()
        .find(|c| !c.is_whitespace())
        .map_or(false, is_cjk_char)
}

/// Adjust spacing around markdown markers for CJK text
///
/// In CJK typography, markdown markers like **, *, ` should not have spaces
/// around them when adjacent to CJK characters.
/// This function removes spaces between CJK characters and markdown markers.
pub fn adjust_cjk_marker_spacing(
    text: &str,
    prev_is_marker: bool,
    next_is_marker: bool,
) -> String {
    let mut result = text.to_string();

    // Only adjust if the text is purely whitespace - in that case, we may want to remove it
    // if it's between CJK and a marker
    if result.trim().is_empty() {
        // Text is only whitespace - check if we should keep it
        // If previous is marker and next would be CJK, or vice versa,
        // we might want to remove it
        return result;
    }

    // If previous node is a marker and this text starts with CJK,
    // remove leading space
    if prev_is_marker {
        // Check if the first non-whitespace character is CJK
        if let Some(first_char) = result.chars().find(|c| !c.is_whitespace()) {
            if is_cjk_char(first_char) {
                result = result.trim_start().to_string();
            }
        }
    }

    // If next node is a marker and this text ends with CJK,
    // remove trailing space
    if next_is_marker {
        // Check if the last non-whitespace character is CJK
        if let Some(last_char) = result.chars().rev().find(|c| !c.is_whitespace()) {
            if is_cjk_char(last_char) {
                result = result.trim_end().to_string();
            }
        }
    }

    result
}

// ============================================================================
// Paragraph Spacing Utilities
// ============================================================================

/// Calculate the prefixes for block quote line breaking
///
/// Returns (first_line_prefix, continuation_prefix) where:
/// - first_line_prefix is empty (the block quote marker is already output by BlockQuote handler)
/// - continuation_prefix is the block quote marker for subsequent lines
pub fn calculate_block_quote_prefixes(
    ctx: &dyn NodeFormatterContext,
) -> (String, String) {
    let nesting_level = ctx.get_block_quote_nesting_level();

    let cont_prefix = "> ".repeat(nesting_level);

    let first_prefix = String::new();

    (first_prefix, cont_prefix)
}

/// Determine how to render a paragraph based on context
///
/// This function implements flexmark-java's paragraph rendering logic:
/// - In tight lists: minimal spacing
/// - In loose lists: blank line between paragraphs
/// - In list items: special handling for first/last paragraphs
/// - Normal paragraphs: blank line after
///
/// Returns true if a blank line should be added, false for just a line break.
pub fn should_render_loose_paragraph(
    ctx: &dyn NodeFormatterContext,
    is_last_paragraph: bool,
) -> bool {
    let is_in_list_item = ctx.is_parent_list_item();
    let is_in_tight_list = ctx.is_in_tight_list();
    let has_next_sibling = ctx.has_next_sibling();

    if is_in_list_item {
        if is_in_tight_list {
            false
        } else {
            !is_last_paragraph && has_next_sibling
        }
    } else if is_in_tight_list {
        false
    } else {
        !is_last_paragraph
    }
}

/// Render paragraph spacing based on context
///
/// This is the main entry point for paragraph spacing, implementing
/// the logic from flexmark-java's paragraph rendering.
pub fn render_paragraph_spacing(
    ctx: &dyn NodeFormatterContext,
    writer: &mut MarkdownWriter,
    is_last: bool,
) {
    if should_render_loose_paragraph(ctx, is_last) {
        writer.blank_line();
    } else {
        writer.line();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_simple_handler() {
        let handler =
            create_simple_handler(NodeValueType::Document, |_value, _ctx, _writer| {
                // Test handler
            });
        assert_eq!(handler.node_type, NodeValueType::Document);
    }

    #[test]
    fn test_create_handler_with_close() {
        let handler = create_handler_with_close(
            NodeValueType::Paragraph,
            |_value, _ctx, _writer| {},
            |_value, _ctx, _writer| {},
        );
        assert_eq!(handler.node_type, NodeValueType::Paragraph);
    }
}
