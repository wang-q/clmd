//! Handler factory functions and context helpers
//!
//! This module provides convenience functions for working with formatter context.

use crate::render::commonmark::core::NodeFormatterContext;

// ============================================================================
// Constants
// ============================================================================

/// Minimum width for paragraph line breaking to avoid degenerate cases
pub const MIN_LINE_BREAKING_WIDTH: usize = 20;

/// Minimum length for fenced code block markers
pub const MIN_FENCE_LENGTH: usize = 3;

/// Number of spaces for indented code blocks
pub const INDENTED_CODE_SPACES: &str = "    ";

/// Block quote prefix pattern
pub const BLOCK_QUOTE_PREFIX: &str = "> ";

// ============================================================================
// CJK Character Handling
// ============================================================================

/// Check if a character is a CJK (Chinese, Japanese, Korean) character
///
/// This function uses the text module's CJK detection.
#[inline]
pub fn is_cjk_char(c: char) -> bool {
    crate::text::unicode::is_cjk(c)
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

    let cont_prefix = BLOCK_QUOTE_PREFIX.repeat(nesting_level);

    let first_prefix = String::new();

    (first_prefix, cont_prefix)
}

// ============================================================================
// Emphasis Handler Helpers
// ============================================================================

/// Check if an emphasis node contains nested conflicting emphasis type.
///
/// Used by Emph and Strong handlers to determine whether to use atomic
/// unit handling or fall back to per-word marker handling.
///
/// Returns `(is_nested, marker)` where:
/// - `is_nested`: true if the current node has a direct child matching `target_discriminant`
/// - `marker`: clone of the input marker string for convenience
pub fn check_nested_emphasis_conflict(
    ctx: &dyn NodeFormatterContext,
    target_discriminant: std::mem::Discriminant<crate::core::nodes::NodeValue>,
    marker: &str,
) -> (bool, String) {
    let is_nested = ctx
        .get_current_node()
        .and_then(|node_id| {
            if ctx.has_child_of_type(node_id, target_discriminant) {
                Some(true)
            } else {
                None
            }
        })
        .unwrap_or(false);
    (is_nested, marker.to_string())
}

// ============================================================================
// Text Preprocessing Pipeline
// ============================================================================

/// Preprocess text content for CommonMark output with full CJK support.
///
/// Applies the complete text preprocessing pipeline:
/// 1. Skip task list markers when inside task items
/// 2. Detect sibling markdown markers for spacing adjustment
/// 3. Detect previous link nodes for CJK spacing
/// 4. Add CJK character spacing
/// 5. Adjust spacing around markdown markers
/// 6. Add leading space after links for ASCII text
pub fn preprocess_text(text: &str, ctx: &dyn NodeFormatterContext) -> String {
    use crate::render::commonmark::handlers::list::{
        is_in_task_list_item, skip_task_marker,
    };

    let processed_text = if is_in_task_list_item(ctx) {
        skip_task_marker(text)
    } else {
        text.to_string()
    };

    let (prev_is_marker, next_is_marker) = check_sibling_markers(ctx);
    let prev_is_link_node = prev_is_link(ctx);

    let cjk_text = crate::text::unicode::add_cjk_spacing(&processed_text);

    let mut final_text =
        adjust_cjk_marker_spacing(&cjk_text, prev_is_marker, next_is_marker);

    if prev_is_link_node && !final_text.is_empty() {
        if let Some(first_char) = final_text.chars().next() {
            if first_char.is_ascii_alphanumeric() {
                final_text = format!(" {}", final_text);
            }
        }
    }

    final_text
}
