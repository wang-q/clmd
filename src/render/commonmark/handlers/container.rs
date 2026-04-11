//! Container element handlers for CommonMark formatting
//!
//! This module contains handlers for container-level elements like paragraphs,
//! headings (ATX and Setext style), and block quotes.

use crate::core::nodes::NodeValue;
use crate::options::format::HeadingStyle;
use crate::render::commonmark::core::NodeFormatterContext;
use crate::render::commonmark::handler_utils::calculate_block_quote_prefixes;
use crate::render::commonmark::handlers::block::calculate_heading_content_length;
use crate::render::commonmark::handlers::list::calculate_list_item_prefixes;
use crate::render::commonmark::writer::MarkdownWriter;

/// Render paragraph opening
///
/// Sets up paragraph line breaking if enabled based on formatter options.
pub fn render_paragraph_open(
    _value: &NodeValue,
    ctx: &mut dyn NodeFormatterContext,
    _writer: &mut MarkdownWriter,
) {
    if let NodeValue::Paragraph = _value {
        let options = ctx.get_formatter_options();
        // Only enable line breaking if right_margin is set
        if options.right_margin > 0 {
            // Calculate nesting levels
            let _list_nesting = ctx.get_list_nesting_level();
            let block_quote_nesting = ctx.get_block_quote_nesting_level();

            // Determine which prefix to use and calculate marker width
            let (prefix, marker_width) = if ctx.is_parent_list_item() {
                // List item: use continuation prefix and calculate marker width
                let (_, cont_prefix) = calculate_list_item_prefixes(ctx);
                let marker_width = cont_prefix.len();
                (cont_prefix, marker_width)
            } else if block_quote_nesting > 0 {
                // Block quote: use block quote prefix
                let (_, cont_prefix) = calculate_block_quote_prefixes(ctx);
                let marker_width = cont_prefix.len();
                (cont_prefix, marker_width)
            } else {
                // Regular paragraph: no prefix
                (String::new(), 0)
            };

            // Calculate available width considering marker width
            let max_width = options.right_margin.saturating_sub(marker_width);
            // Ensure max_width is at least MIN_LINE_BREAKING_WIDTH to avoid degenerate cases
            let max_width = max_width
                .max(crate::render::commonmark::handler_utils::MIN_LINE_BREAKING_WIDTH);

            // Start the new paragraph line breaker
            ctx.start_paragraph_line_breaking(max_width, prefix);
        }
    }
}

/// Render paragraph closing
///
/// Finishes line breaking and outputs result, then adds appropriate spacing
/// based on list context.
pub fn render_paragraph_close(
    _value: &NodeValue,
    ctx: &mut dyn NodeFormatterContext,
    writer: &mut MarkdownWriter,
) {
    let is_in_list_item = ctx.is_parent_list_item();
    let is_in_tight_list = ctx.is_in_tight_list();
    let has_next_sibling = ctx.has_next_sibling();

    // Check if we were collecting text for paragraph line breaking
    if ctx.is_paragraph_line_breaking() {
        if let Some(formatted_text) = ctx.finish_paragraph_line_breaking() {
            // Output the formatted text with optimal line breaks
            writer.append_raw(&formatted_text);
        }
    } else {
        // Flush any remaining text in the word wrap buffer
        // This ensures trailing spaces are preserved
        writer.flush_word_wrap_buffer();
    }

    // Add line break after paragraph
    if is_in_list_item {
        // Paragraph is inside a list item
        if is_in_tight_list {
            // Tight list: minimal spacing
            if has_next_sibling {
                // More content follows, add single line
                writer.line();
            }
            // If last child, let the list item handler manage spacing
        } else {
            // Loose list: blank line between paragraphs
            if has_next_sibling {
                writer.blank_line();
            } else {
                // Last paragraph in list item
                writer.line();
            }
        }
    } else if is_in_tight_list {
        // Paragraph in tight list context but not directly in item
        // (e.g., nested content)
        if has_next_sibling {
            writer.line();
        }
    } else {
        // Normal paragraph outside lists
        // Add blank line after paragraph
        writer.blank_line();
    }
}

/// Render heading opening
///
/// Outputs the heading prefix based on style (ATX or Setext).
pub fn render_heading_open(
    value: &NodeValue,
    ctx: &mut dyn NodeFormatterContext,
    writer: &mut MarkdownWriter,
) {
    if let NodeValue::Heading(heading) = value {
        let options = ctx.get_formatter_options();

        // Determine heading style
        let use_setext = match options.heading_style {
            HeadingStyle::Setext => heading.level <= 2,
            HeadingStyle::Atx => false,
            HeadingStyle::AsIs => heading.setext,
        };

        if use_setext {
            // Setext style - no prefix needed, marker comes after content
            // Store that we're using setext style for the close handler
        } else {
            // ATX style
            let hashes = "#".repeat(heading.level as usize);
            if options.space_after_atx_marker {
                writer.append_raw(format!("{} ", hashes));
            } else {
                writer.append_raw(hashes);
            }
        }
    }
}

/// Render heading closing
///
/// Outputs Setext underline or ATX trailing markers, then adds blank line.
pub fn render_heading_close(
    value: &NodeValue,
    ctx: &mut dyn NodeFormatterContext,
    writer: &mut MarkdownWriter,
) {
    if let NodeValue::Heading(heading) = value {
        let options = ctx.get_formatter_options();
        let heading_style = options.heading_style;
        let min_setext_marker_length = options.min_setext_marker_length;
        let setext_equalize_marker = options.setext_heading_equalize_marker;

        // Determine heading style
        let use_setext = match heading_style {
            HeadingStyle::Setext => heading.level <= 2,
            HeadingStyle::Atx => false,
            HeadingStyle::AsIs => heading.setext,
        };

        if use_setext {
            // Add Setext underline
            let marker = if heading.level == 1 { '=' } else { '-' };
            writer.line();

            // Calculate underline length based on content
            // We need to get the content that was just rendered
            let content_len = if let Some(node_id) = ctx.get_current_node() {
                // Calculate content length from children
                calculate_heading_content_length(ctx, node_id)
            } else {
                min_setext_marker_length
            };

            // Calculate marker length
            let underline_len = if setext_equalize_marker {
                // Equalize marker to match content length
                content_len.max(min_setext_marker_length)
            } else {
                // Use minimum marker length
                min_setext_marker_length
            };

            writer.append(marker.to_string().repeat(underline_len));
        } else {
            // ATX style - handle trailing markers
            match options.atx_heading_trailing_marker {
                crate::options::format::TrailingMarker::Add => {
                    // Add trailing hashes
                    let hashes = "#".repeat(heading.level as usize);
                    writer.append_raw(format!(" {}", hashes));
                }
                crate::options::format::TrailingMarker::Remove => {
                    // No trailing hashes
                }
                crate::options::format::TrailingMarker::AsIs => {
                    // Keep as-is (no additional handling needed)
                }
                crate::options::format::TrailingMarker::Equalize => {
                    // Equalize trailing marker to match opening
                    let hashes = "#".repeat(heading.level as usize);
                    writer.append_raw(format!(" {}", hashes));
                }
            }
        }
    }
    // Heading closing - add blank line after heading
    writer.blank_line();
}

/// Render block quote opening
///
/// Pushes the block quote prefix and increments nesting.
pub fn render_block_quote_open(
    _value: &NodeValue,
    ctx: &mut dyn NodeFormatterContext,
    writer: &mut MarkdownWriter,
) {
    writer.push_prefix("> ");
    ctx.set_in_block_quote(true);
    ctx.increment_block_quote_nesting();
}

/// Render block quote closing
///
/// Pops the prefix and decrements nesting.
pub fn render_block_quote_close(
    _value: &NodeValue,
    ctx: &mut dyn NodeFormatterContext,
    writer: &mut MarkdownWriter,
) {
    writer.pop_prefix();
    ctx.decrement_block_quote_nesting();
    if ctx.get_block_quote_nesting_level() == 0 {
        ctx.set_in_block_quote(false);
    }
}
