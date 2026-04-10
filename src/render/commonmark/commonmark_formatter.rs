//! CommonMark node formatter implementation
//!
//! This module provides a NodeFormatter implementation for CommonMark output,
//! migrating the existing commonmark.rs functionality to the formatter framework.
//!
//! # Example
//!
//! ```ignore
//! use clmd::render::commonmark::CommonMarkNodeFormatter;
//! use clmd::options::format::FormatOptions;
//!
//! let formatter = CommonMarkNodeFormatter::new();
//! let options = FormatOptions::new().with_right_margin(80);
//! let formatter = CommonMarkNodeFormatter::with_options(options);
//! ```

use crate::core::arena::NodeId;
use crate::core::nodes::NodeValue;
use crate::options::format::{FormatOptions, HeadingStyle};
use crate::render::commonmark::context::NodeFormatterContext;
use crate::render::commonmark::escaping::{compute_fence_length, escape_text};
use crate::render::commonmark::handler_utils::{
    adjust_cjk_marker_spacing, calculate_block_quote_prefixes, check_sibling_markers,
    create_handler_with_close, create_simple_handler, prev_is_link,
};
use crate::render::commonmark::handlers::block::{
    calculate_heading_content_length, render_code_block, render_html_block,
};
use crate::render::commonmark::handlers::inline::{render_image_url, render_link_url};
use crate::render::commonmark::handlers::list::{
    calculate_effective_list_tightness, calculate_list_item_prefixes,
    count_list_ancestors, format_list_item_marker_with_number_and_options,
    get_item_number_in_list, is_empty_list_item, is_in_task_list_item,
    is_task_item_checked, skip_task_marker,
};
use crate::render::commonmark::handlers::table::{
    collect_cell_text_content, render_formatted_table,
};
use crate::render::commonmark::node::{
    NodeFormatter, NodeFormattingHandler, NodeValueType,
};
use crate::render::commonmark::phase::FormattingPhase;
use crate::render::commonmark::phased::PhasedNodeFormatter;
use crate::render::commonmark::writer::MarkdownWriter;

/// CommonMark node formatter
///
/// This formatter implements the NodeFormatter trait to provide CommonMark output.
/// It supports all standard CommonMark elements plus GFM extensions.
///
/// The formatter uses a multi-phase rendering approach:
/// 1. **Collect phase**: Gathers reference links and other metadata
/// 2. **Document phase**: Performs the main rendering
///
/// # Supported Elements
///
/// ## Block Elements
/// - Document, Paragraph, Heading (ATX style)
/// - BlockQuote, CodeBlock (fenced)
/// - List (ordered/unordered), Item
/// - ThematicBreak, HtmlBlock
///
/// ## Inline Elements
/// - Text (with proper escaping)
/// - Code (inline), Emph, Strong
/// - Link, Image
/// - Strikethrough (GFM)
/// - SoftBreak, HardBreak
/// - HtmlInline
///
/// ## GFM Extensions
/// - Table (with alignment)
/// - FootnoteReference, FootnoteDefinition
/// - TaskItem (checkboxes)
#[derive(Debug)]
pub struct CommonMarkNodeFormatter {
    /// Formatter options for customizing output
    options: FormatOptions,
}

impl CommonMarkNodeFormatter {
    /// Create a new CommonMark formatter with default options
    ///
    /// # Example
    ///
    /// ```ignore
    /// use clmd::render::commonmark::CommonMarkNodeFormatter;
    ///
    /// let formatter = CommonMarkNodeFormatter::new();
    /// ```
    pub fn new() -> Self {
        Self::with_options(FormatOptions::default())
    }

    /// Create a new CommonMark formatter with custom options
    ///
    /// # Arguments
    ///
    /// * `options` - Formatter options for customizing output
    ///
    /// # Example
    ///
    /// ```ignore
    /// use clmd::render::commonmark::CommonMarkNodeFormatter;
    /// use clmd::options::format::FormatOptions;
    ///
    /// let options = FormatOptions::new()
    ///     .with_right_margin(80)
    ///     .with_keep_hard_line_breaks(true);
    /// let formatter = CommonMarkNodeFormatter::with_options(options);
    /// ```
    pub fn with_options(options: FormatOptions) -> Self {
        Self { options }
    }

    /// Get the formatter options
    pub fn options(&self) -> &FormatOptions {
        &self.options
    }
}

impl Default for CommonMarkNodeFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeFormatter for CommonMarkNodeFormatter {
    fn get_node_formatting_handlers(&self) -> Vec<NodeFormattingHandler> {
        vec![
            // Document - simple handler with no-op
            create_simple_handler(NodeValueType::Document, |_value, _ctx, _writer| {
                // Document is handled at the top level
            }),
            // Block elements
            // Paragraph - handler with close
            create_handler_with_close(
                NodeValueType::Paragraph,
                |_value, ctx, _writer| {
                    // Paragraph opening - start paragraph line breaking if enabled
                    if let NodeValue::Paragraph = _value {
                        let options = ctx.get_formatter_options();
                        // Only enable line breaking if right_margin is set
                        if options.right_margin > 0 {
                            // Calculate nesting levels
                            let _list_nesting = ctx.get_list_nesting_level();
                            let block_quote_nesting =
                                ctx.get_block_quote_nesting_level();

                            // Determine which prefix to use and calculate marker width
                            let (prefix, marker_width) = if ctx.is_parent_list_item() {
                                // List item: use continuation prefix and calculate marker width
                                let (_, cont_prefix) = calculate_list_item_prefixes(ctx);
                                let marker_width = cont_prefix.len();
                                (cont_prefix, marker_width)
                            } else if block_quote_nesting > 0 {
                                // Block quote: use block quote prefix
                                let (_, cont_prefix) =
                                    calculate_block_quote_prefixes(ctx);
                                let marker_width = cont_prefix.len();
                                (cont_prefix, marker_width)
                            } else {
                                // Regular paragraph: no prefix
                                (String::new(), 0)
                            };

                            // Calculate available width considering marker width
                            let max_width =
                                options.right_margin.saturating_sub(marker_width);
                            // Ensure max_width is at least MIN_LINE_BREAKING_WIDTH to avoid degenerate cases
                            let max_width = max_width.max(crate::render::commonmark::handler_utils::MIN_LINE_BREAKING_WIDTH);

                            // Start the new paragraph line breaker
                            ctx.start_paragraph_line_breaking(max_width, prefix);
                        }
                    }
                },
                |_value, ctx, writer| {
                    // Paragraph closing - finish line breaking and output result
                    let is_in_list_item = ctx.is_parent_list_item();
                    let is_in_tight_list = ctx.is_in_tight_list();
                    let has_next_sibling = ctx.has_next_sibling();

                    // Check if we were collecting text for paragraph line breaking
                    if ctx.is_paragraph_line_breaking() {
                        if let Some(formatted_text) =
                            ctx.finish_paragraph_line_breaking()
                        {
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
                },
            ),
            // Heading - handler with close
            create_handler_with_close(
                NodeValueType::Heading,
                |value, ctx, writer| {
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
                },
                |value, ctx, writer| {
                    if let NodeValue::Heading(heading) = value {
                        let options = ctx.get_formatter_options();
                        let heading_style = options.heading_style;
                        let min_setext_marker_length = options.min_setext_marker_length;
                        let setext_equalize_marker =
                            options.setext_heading_equalize_marker;

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
                            let content_len =
                                if let Some(node_id) = ctx.get_current_node() {
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
                },
            ),
            // BlockQuote - handler with close
            create_handler_with_close(
                NodeValueType::BlockQuote,
                |_value, ctx, writer| {
                    writer.push_prefix("> ");
                    ctx.set_in_block_quote(true);
                    ctx.increment_block_quote_nesting();
                },
                |_value, ctx, writer| {
                    writer.pop_prefix();
                    ctx.decrement_block_quote_nesting();
                    if ctx.get_block_quote_nesting_level() == 0 {
                        ctx.set_in_block_quote(false);
                    }
                },
            ),
            // CodeBlock - simple handler
            create_simple_handler(NodeValueType::CodeBlock, |value, ctx, writer| {
                if let NodeValue::CodeBlock(code_block) = value {
                    render_code_block(code_block, ctx, writer);
                }
            }),
            // List - handler with close
            create_handler_with_close(
                NodeValueType::List,
                |value, ctx, _writer| {
                    if let NodeValue::List(list) = value {
                        // Determine effective tightness based on list_spacing option and content
                        let effective_tight = if let Some(node_id) =
                            ctx.get_current_node()
                        {
                            calculate_effective_list_tightness(
                                ctx.get_arena(),
                                node_id,
                                list,
                                ctx.get_formatter_options(),
                            )
                        } else {
                            // Fallback to simple option-based logic
                            match ctx.get_formatter_options().list_spacing {
                                crate::options::format::ListSpacing::Tight => true,
                                crate::options::format::ListSpacing::Loose => false,
                                crate::options::format::ListSpacing::AsIs => list.tight,
                                crate::options::format::ListSpacing::Loosen => {
                                    list.tight
                                }
                                crate::options::format::ListSpacing::Tighten => true,
                            }
                        };
                        ctx.set_tight_list(effective_tight);
                        ctx.increment_list_nesting();
                    }
                },
                |_value, ctx, writer| {
                    ctx.decrement_list_nesting();
                    if ctx.get_list_nesting_level() == 0 {
                        ctx.set_tight_list(false);
                        // Add blank line after list ends to separate from following content
                        writer.blank_line();
                    }
                },
            ),
            // Item - handler with close
            create_handler_with_close(
                NodeValueType::Item,
                |value, ctx, writer| {
                    let options = ctx.get_formatter_options();

                    // Check if this item should be removed (empty item removal)
                    if options.list_remove_empty_items {
                        if let Some(node_id) = ctx.get_current_node() {
                            if is_empty_list_item(ctx.get_arena(), node_id) {
                                // Skip rendering this empty item
                                // Note: We still need to handle the closing, but we can
                                // mark it to not output anything
                                return;
                            }
                        }
                    }

                    // Get the parent list to determine the marker and nesting level
                    let (marker, nesting_level) = if let Some(parent_id) =
                        ctx.get_current_node_parent()
                    {
                        let arena = ctx.get_arena();
                        let parent = arena.get(parent_id);
                        if let NodeValue::List(list) = &parent.value {
                            // Find the nesting level by counting list ancestors
                            let level = count_list_ancestors(arena, parent_id);
                            // For ordered lists, calculate the item number based on position
                            let item_number = get_item_number_in_list(
                                arena,
                                parent_id,
                                ctx.get_current_node(),
                            );

                            // Apply list renumbering if configured
                            let effective_number = if options.list_renumber_items {
                                // Renumber starting from 1
                                item_number
                            } else {
                                // Use original list start + offset
                                list.start + item_number - 1
                            };

                            let marker = format_list_item_marker_with_number_and_options(
                                list,
                                effective_number,
                                options,
                            );
                            (marker, level)
                        } else {
                            ("- ".to_string(), 0)
                        }
                    } else {
                        ("- ".to_string(), 0)
                    };

                    // Check if this specific item is a task list item
                    let is_task_list = if let NodeValue::Item(item_data) = value {
                        item_data.is_task_list
                    } else {
                        false
                    };

                    // Calculate total indentation for nested lists
                    // CommonMark requires 4-space indentation for nested lists
                    let total_indent = if nesting_level == 0 {
                        0
                    } else {
                        // Each nesting level adds 4 spaces
                        nesting_level * 4
                    };

                    let indent_str = " ".repeat(total_indent);

                    // Output the list marker directly (not as a prefix)
                    // This avoids the prefix stacking issue with nested lists
                    writer.append_raw(&indent_str);
                    writer.append_raw(&marker);

                    // If this is a task list item, render the task marker
                    if is_task_list {
                        // Determine if this is a checked task
                        let task_marker = if is_task_item_checked(
                            ctx.get_arena(),
                            ctx.get_current_node(),
                        ) {
                            "[x] "
                        } else {
                            "[ ] "
                        };
                        writer.append_raw(task_marker);
                    }
                },
                |_value, ctx, writer| {
                    // Add line break after each list item
                    // In tight lists, just add a single line break
                    // In loose lists, add a blank line
                    // For the last item, we handle it differently based on context
                    let _options = ctx.get_formatter_options();

                    // Check if this is the last item in the list
                    let is_last_item = ctx
                        .get_current_node()
                        .map(|id| {
                            let arena = ctx.get_arena();
                            arena.get(id).next.is_none()
                        })
                        .unwrap_or(true);

                    if ctx.is_in_tight_list() {
                        // Tight list: single line break between items
                        // Don't add blank line after last item (handled by list close)
                        if !is_last_item {
                            writer.line();
                        }
                    } else {
                        // Loose list: blank line between items
                        // Don't add blank line after last item (handled by list close)
                        if !is_last_item {
                            writer.blank_line();
                        } else {
                            writer.line();
                        }
                    }
                },
            ),
            // ThematicBreak - simple handler
            create_simple_handler(
                NodeValueType::ThematicBreak,
                |value, _ctx, writer| {
                    if let NodeValue::ThematicBreak(tb) = value {
                        // Use the original marker from the parsed document
                        writer.append(tb.marker.to_string().repeat(3));
                    } else {
                        // Fallback to default marker
                        writer.append("---");
                    }
                    writer.line();
                },
            ),
            // HtmlBlock - simple handler
            create_simple_handler(NodeValueType::HtmlBlock, |value, ctx, writer| {
                if let NodeValue::HtmlBlock(html) = value {
                    render_html_block(html, ctx, writer);
                }
            }),
            // Inline elements
            // Text - simple handler
            create_simple_handler(NodeValueType::Text, |value, ctx, writer| {
                if let NodeValue::Text(text) = value {
                    let text_str = text.as_ref();

                    // Check if we're in a task list item and need to skip the task marker
                    let processed_text = if is_in_task_list_item(ctx) {
                        skip_task_marker(text_str)
                    } else {
                        text_str.to_string()
                    };

                    // Check if previous and next siblings are markdown markers
                    let (prev_is_marker, next_is_marker) = check_sibling_markers(ctx);

                    // Check if previous sibling is a Link (for CJK spacing)
                    let prev_is_link_node = prev_is_link(ctx);

                    // Apply CJK spacing by default
                    // NOTE: We apply CJK spacing even when using paragraph line breaking
                    // to ensure proper spacing around markdown markers
                    let cjk_text =
                        crate::text::cjk_spacing::add_cjk_spacing(&processed_text);

                    // Adjust spacing around markdown markers for CJK text
                    // This removes spaces between CJK characters and markdown markers
                    let mut final_text = adjust_cjk_marker_spacing(
                        &cjk_text,
                        prev_is_marker,
                        next_is_marker,
                    );

                    // If previous sibling is a Link and this text starts with ASCII,
                    // add a leading space for CJK spacing
                    if prev_is_link_node && !final_text.is_empty() {
                        if let Some(first_char) = final_text.chars().next() {
                            if first_char.is_ascii_alphanumeric() {
                                final_text = format!(" {}", final_text);
                            }
                        }
                    }

                    // Check if we're using paragraph line breaking
                    if ctx.is_paragraph_line_breaking() {
                        // Add text to paragraph line breaker
                        ctx.add_paragraph_text(&final_text);
                    } else {
                        // Use context-aware escaping
                        let escaped = escape_text(&final_text, ctx);
                        // Use append_with_wrap for text wrapping when right_margin is set
                        // This enables line folding at the specified width
                        writer.append_with_wrap(&escaped);
                    }
                }
            }),
            // Code - simple handler
            create_simple_handler(NodeValueType::Code, |value, ctx, writer| {
                if let NodeValue::Code(code) = value {
                    // Calculate the required fence length based on content
                    // Need to account for backticks in the content
                    let fence_len = compute_fence_length(&code.literal, 1);
                    let backticks = "`".repeat(fence_len);

                    // Determine if we need padding (spaces around content)
                    // Padding is needed when:
                    // 1. Content starts or ends with a backtick
                    // 2. Content starts or ends with a space
                    let needs_leading_space =
                        code.literal.starts_with('`') || code.literal.starts_with(' ');
                    let needs_trailing_space =
                        code.literal.ends_with('`') || code.literal.ends_with(' ');

                    // Check if we're using the new paragraph line breaking system
                    if ctx.is_paragraph_line_breaking() {
                        // Add inline code as unbreakable unit
                        let prefix = backticks.clone();
                        let suffix = backticks.clone();
                        let content = if needs_leading_space || needs_trailing_space {
                            let mut c = String::new();
                            if needs_leading_space {
                                c.push(' ');
                            }
                            c.push_str(&code.literal);
                            if needs_trailing_space {
                                c.push(' ');
                            }
                            c
                        } else {
                            code.literal.to_string()
                        };
                        ctx.add_paragraph_unbreakable_unit(
                            crate::render::commonmark::line_breaking::UnitKind::InlineCode,
                            &prefix,
                            &content,
                            &suffix,
                        );
                    } else {
                        writer.append(&backticks);

                        if needs_leading_space {
                            writer.append(" ");
                        }

                        // For code content, we don't escape - output as-is
                        writer.append_raw(&code.literal);

                        if needs_trailing_space {
                            writer.append(" ");
                        }

                        writer.append(&backticks);
                    }
                }
            }),
            // Emph - handler with close
            create_handler_with_close(
                NodeValueType::Emph,
                |_value, ctx, _writer| {
                    if ctx.is_paragraph_line_breaking() {
                        // Check if this emphasis contains nested strong
                        // If so, don't use atomic handling to avoid conflicts
                        if let Some(node_id) = ctx.get_current_node() {
                            if ctx.has_child_of_type(node_id, NodeValueType::Strong) {
                                // Nested emphasis - use normal word handling
                                ctx.add_paragraph_word("*");
                            } else {
                                // For paragraph line breaking, collect entire emphasis as a unit
                                ctx.set_skip_children(true);
                            }
                        }
                    } else {
                        _writer.append("*");
                    }
                },
                |_value, ctx, writer| {
                    if ctx.is_paragraph_line_breaking() {
                        // Check if this emphasis contains nested strong
                        if let Some(node_id) = ctx.get_current_node() {
                            if ctx.has_child_of_type(node_id, NodeValueType::Strong) {
                                // Nested emphasis - use normal word handling
                                ctx.add_paragraph_word("*");
                            } else {
                                // Collect emphasis content from children
                                let content = ctx.render_children_to_string(node_id);
                                // Build complete emphasis as unbreakable unit
                                let full_emph = format!("*{}*", content);
                                ctx.add_paragraph_unbreakable_unit(
                                    crate::render::commonmark::line_breaking::UnitKind::Emph,
                                    "",
                                    &full_emph,
                                    "",
                                );
                                ctx.set_skip_children(false);
                            }
                        }
                    } else {
                        writer.append("*");
                    }
                },
            ),
            // Strong - handler with close
            create_handler_with_close(
                NodeValueType::Strong,
                |_value, ctx, _writer| {
                    if ctx.is_paragraph_line_breaking() {
                        // Check if this strong contains nested emphasis
                        if let Some(node_id) = ctx.get_current_node() {
                            if ctx.has_child_of_type(node_id, NodeValueType::Emph) {
                                // Nested strong - use normal word handling
                                ctx.add_paragraph_word("**");
                            } else {
                                // For paragraph line breaking, collect entire strong as a unit
                                ctx.set_skip_children(true);
                            }
                        }
                    } else {
                        // Flush any pending text in word wrap buffer to ensure
                        // ends_with_whitespace check works correctly
                        _writer.flush_word_wrap_buffer();
                        // Ensure there's a space before the marker if not at start of line
                        // and previous content doesn't end with whitespace
                        // But don't add space if previous char is '*' (for *** emphasis)
                        if !_writer.is_beginning_of_line()
                            && !_writer.ends_with_whitespace()
                            && !_writer.ends_with_char('*')
                        {
                            _writer.append_raw(" ");
                        }
                        _writer.append("**");
                    }
                },
                |_value, ctx, writer| {
                    if ctx.is_paragraph_line_breaking() {
                        // Check if this strong contains nested emphasis
                        if let Some(node_id) = ctx.get_current_node() {
                            if ctx.has_child_of_type(node_id, NodeValueType::Emph) {
                                // Nested strong - use normal word handling
                                ctx.add_paragraph_word("**");
                            } else {
                                // Collect strong content from children
                                let content = ctx.render_children_to_string(node_id);
                                // Build complete strong as unbreakable unit
                                let full_strong = format!("**{}**", content);
                                ctx.add_paragraph_unbreakable_unit(
                                    crate::render::commonmark::line_breaking::UnitKind::Strong,
                                    "",
                                    &full_strong,
                                    "",
                                );
                                ctx.set_skip_children(false);
                            }
                        }
                    } else {
                        writer.append("**");
                    }
                },
            ),
            NodeFormattingHandler::with_close(
                NodeValueType::Link,
                Box::new(
                    |_value: &NodeValue,
                     ctx: &mut dyn NodeFormatterContext,
                     _writer: &mut MarkdownWriter| {
                        // When line breaking is active, we need to handle link specially
                        if ctx.is_paragraph_line_breaking() {
                            // For paragraph line breaking, we collect the entire link as a unit
                            // Skip rendering children here - we'll collect them in the close handler
                            ctx.set_skip_children(true);
                        } else {
                            _writer.append("[");
                        }
                    },
                ),
                Box::new(
                    |value: &NodeValue,
                     ctx: &mut dyn NodeFormatterContext,
                     writer: &mut MarkdownWriter| {
                        if let NodeValue::Link(link) = value {
                            if ctx.is_paragraph_line_breaking() {
                                // Collect link text from children
                                if let Some(node_id) = ctx.get_current_node() {
                                    let link_text =
                                        ctx.render_children_to_string(node_id);
                                    // Build the complete link as an unbreakable unit
                                    // Format: [link_text](url "title")
                                    let full_link = if link.title.is_empty() {
                                        format!("[{}]({})", link_text, link.url)
                                    } else {
                                        format!(
                                            "[{}]({} \"{}\")",
                                            link_text, link.url, link.title
                                        )
                                    };
                                    // Add as an unbreakable unit
                                    // This ensures the link is not split across lines
                                    ctx.add_paragraph_unbreakable_unit(
                                        crate::render::commonmark::line_breaking::UnitKind::Link,
                                        "",
                                        &full_link,
                                        "",
                                    );
                                }
                                // Re-enable children rendering
                                ctx.set_skip_children(false);
                            } else {
                                // Close the link text bracket
                                writer.append("]");
                                // Then add the URL/title
                                render_link_url(&link.url, &link.title, ctx, writer);
                            }
                        }
                    },
                ),
            ),
            NodeFormattingHandler::with_close(
                NodeValueType::Image,
                Box::new(
                    |_value: &NodeValue,
                     ctx: &mut dyn NodeFormatterContext,
                     _writer: &mut MarkdownWriter| {
                        // When line breaking is active, we need to handle image specially
                        if ctx.is_paragraph_line_breaking() {
                            // For paragraph line breaking, we collect the entire image as a unit
                            // Skip rendering children here - we'll collect them in the close handler
                            ctx.set_skip_children(true);
                        } else {
                            _writer.append("![");
                        }
                    },
                ),
                Box::new(
                    |value: &NodeValue,
                     ctx: &mut dyn NodeFormatterContext,
                     writer: &mut MarkdownWriter| {
                        if let NodeValue::Image(link) = value {
                            if ctx.is_paragraph_line_breaking() {
                                // Collect alt text from children
                                if let Some(node_id) = ctx.get_current_node() {
                                    let alt_text =
                                        ctx.render_children_to_string(node_id);
                                    // Build the complete image as an unbreakable unit
                                    // Format: ![alt_text](url "title")
                                    let full_image = if link.title.is_empty() {
                                        format!("![{}]({})", alt_text, link.url)
                                    } else {
                                        format!(
                                            "![{}]({} \"{}\")",
                                            alt_text, link.url, link.title
                                        )
                                    };
                                    // Add as a single word (unbreakable)
                                    ctx.add_paragraph_word(&full_image);
                                }
                                // Re-enable children rendering
                                ctx.set_skip_children(false);
                            } else {
                                // Close the image alt bracket
                                writer.append("]");
                                // Then add the URL/title
                                render_image_url(&link.url, &link.title, ctx, writer);
                            }
                        }
                    },
                ),
            ),
            // Strikethrough - handler with close
            create_handler_with_close(
                NodeValueType::Strikethrough,
                |_value, _ctx, writer| {
                    writer.append("~~");
                },
                |_value, _ctx, writer| {
                    writer.append("~~");
                },
            ),
            // SoftBreak - simple handler
            create_simple_handler(NodeValueType::SoftBreak, |_value, ctx, writer| {
                // Based on flexmark-java's SoftLineBreak handling:
                // 1. In tight lists, soft breaks become spaces
                // 2. If keepSoftLineBreaks is enabled, preserve the break
                // 3. Otherwise, convert to space (for wrapping) or line break
                let options = ctx.get_formatter_options();

                // Check if we're using the new paragraph line breaking system
                if ctx.is_paragraph_line_breaking() {
                    // Add space for soft break
                    ctx.add_paragraph_text(" ");
                } else if ctx.is_in_tight_list() {
                    // In tight lists, soft breaks become spaces
                    writer.append(" ");
                } else if options.keep_soft_line_breaks {
                    // Preserve soft line breaks if configured
                    writer.line();
                } else {
                    // Default: convert soft break to space for wrapping
                    // The formatter will handle line wrapping based on right_margin
                    let right_margin = options.right_margin;
                    if right_margin > 0 {
                        // With right margin set, use space for potential wrapping
                        writer.append(" ");
                    } else {
                        // Without right margin, use line break
                        writer.line();
                    }
                }
            }),
            // HardBreak - simple handler
            create_simple_handler(NodeValueType::HardBreak, |_value, ctx, writer| {
                // Based on flexmark-java's HardLineBreak handling:
                // 1. If keepHardLineBreaks is enabled, preserve as \\ at end of line
                // 2. Otherwise, use two spaces at end of line (standard Markdown)
                let options = ctx.get_formatter_options();

                // Check if we're using the new paragraph line breaking system
                if ctx.is_paragraph_line_breaking() {
                    // Add hard break to paragraph line breaker
                    ctx.add_paragraph_hard_break();
                } else if options.keep_hard_line_breaks {
                    // Use backslash style for hard breaks
                    writer.append("\\");
                    writer.line();
                } else {
                    // Standard: two spaces at end of line
                    writer.append("  ");
                    writer.line();
                }
            }),
            // HtmlInline - simple handler
            create_simple_handler(NodeValueType::HtmlInline, |value, _ctx, writer| {
                if let NodeValue::HtmlInline(html) = value {
                    writer.append(html);
                }
            }),
            // Table elements
            // Table - handler with close
            create_handler_with_close(
                NodeValueType::Table,
                |value, ctx, _writer| {
                    // Table opening - start collecting data
                    if let NodeValue::Table(table) = value {
                        ctx.start_table_collection(table.alignments.clone());
                    }
                },
                |_value, ctx, writer| {
                    // Table closing - format and output using table.rs
                    if let Some((rows, alignments)) = ctx.take_table_data() {
                        render_formatted_table(&rows, &alignments, writer);
                    }
                },
            ),
            // TableRow - handler with close
            create_handler_with_close(
                NodeValueType::TableRow,
                |_value, ctx, _writer| {
                    // Row opening - add new row to collection
                    ctx.add_table_row();
                },
                |_value, _ctx, _writer| {
                    // Row closing - nothing to do
                },
            ),
            // TableCell - handler with close
            create_handler_with_close(
                NodeValueType::TableCell,
                |_value, ctx, _writer| {
                    // Cell opening - if collecting table, skip rendering children
                    // They will be collected on close
                    if ctx.is_collecting_table() {
                        ctx.set_skip_children(true);
                    }
                },
                |_value, ctx, _writer| {
                    // Cell closing - collect text content directly without full rendering
                    if ctx.is_collecting_table() {
                        if let Some(node_id) = ctx.get_current_node() {
                            let content =
                                collect_cell_text_content(ctx.get_arena(), node_id);
                            ctx.add_table_cell(content);
                        }
                    }
                },
            ),
            // Footnote elements
            create_simple_handler(
                NodeValueType::FootnoteReference,
                |value, _ctx, writer| {
                    if let NodeValue::FootnoteReference(footnote) = value {
                        writer.append(format!("[^{}]", footnote.name));
                    }
                },
            ),
            create_simple_handler(
                NodeValueType::FootnoteDefinition,
                |value, _ctx, writer| {
                    if let NodeValue::FootnoteDefinition(footnote) = value {
                        writer.append(format!("[^{}]:", footnote.name));
                    }
                },
            ),
            // Task items
            create_simple_handler(NodeValueType::TaskItem, |value, _ctx, writer| {
                if let NodeValue::TaskItem(task) = value {
                    if task.symbol.is_some() {
                        writer.append_raw("[x] ");
                    } else {
                        writer.append_raw("[ ] ");
                    }
                }
            }),
            // Shortcode emoji
            create_simple_handler(NodeValueType::ShortCode, |value, _ctx, writer| {
                if let NodeValue::ShortCode(shortcode) = value {
                    // Output the original shortcode format
                    writer.append(format!(":{}:", shortcode.code));
                }
            }),
        ]
    }
}

impl PhasedNodeFormatter for CommonMarkNodeFormatter {
    fn get_formatting_phases(&self) -> Vec<FormattingPhase> {
        // Support all formatting phases as defined in flexmark-java
        vec![
            FormattingPhase::Collect,
            FormattingPhase::DocumentFirst,
            FormattingPhase::DocumentTop,
            FormattingPhase::Document,
            FormattingPhase::DocumentBottom,
        ]
    }

    fn render_document(
        &self,
        context: &mut dyn NodeFormatterContext,
        writer: &mut MarkdownWriter,
        _root: NodeId,
        phase: FormattingPhase,
    ) {
        match phase {
            FormattingPhase::Collect => {
                // In the Collect phase, gather reference links and other information
                // needed for the main rendering pass
                // This includes collecting unused references for sorting
                self.collect_document_info(context);
            }
            FormattingPhase::DocumentFirst => {
                // First pass over the document - can be used for initialization
                // or pre-processing before the main rendering
            }
            FormattingPhase::DocumentTop => {
                // Render elements at the top of the document
                // This is where collected references can be placed if configured
                self.render_document_top_elements(context, writer);
            }
            FormattingPhase::Document => {
                // Main document rendering is handled by the node handlers
                // This phase is processed through the regular node traversal
            }
            FormattingPhase::DocumentBottom => {
                // Render elements at the bottom of the document
                // This is where footnotes or references can be placed
                self.render_document_bottom_elements(context, writer);
            }
        }
    }
}

impl CommonMarkNodeFormatter {
    /// Collect information about the document during the Collect phase
    fn collect_document_info(&self, _context: &mut dyn NodeFormatterContext) {
        // Placeholder for collecting reference links, footnotes, etc.
        // This information can be used during rendering to:
        // - Identify unused references
        // - Sort references
        // - Generate footnote numbers
    }

    /// Render elements that should appear at the top of the document
    fn render_document_top_elements(
        &self,
        context: &mut dyn NodeFormatterContext,
        _writer: &mut MarkdownWriter,
    ) {
        // Placeholder for rendering elements at document top
        // This can include:
        // - References placed at document top
        // - Table of contents
        // - Other collected elements

        // Check if we should place references at document top
        let options = context.get_formatter_options();
        if options.reference_placement
            == crate::options::format::ElementPlacement::DocumentTop
        {
            // Render collected references at top
            // This would require reference collection to be implemented
        }
    }

    /// Render elements that should appear at the bottom of the document
    fn render_document_bottom_elements(
        &self,
        context: &mut dyn NodeFormatterContext,
        _writer: &mut MarkdownWriter,
    ) {
        // Placeholder for rendering elements at document bottom
        // This can include:
        // - Footnotes
        // - References placed at document bottom
        // - Other collected elements

        // Check if we should place references at document bottom
        let options = context.get_formatter_options();
        if options.reference_placement
            == crate::options::format::ElementPlacement::DocumentBottom
        {
            // Render collected references at bottom
            // This would require reference collection to be implemented
        }

        // Render footnotes if any were collected
        // This would require footnote collection to be implemented
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::commonmark::escaping::{escape_string, escape_text, escape_url};

    // Mock context for testing
    struct MockContext;

    impl NodeFormatterContext for MockContext {
        fn get_markdown_writer(
            &mut self,
        ) -> &mut crate::render::commonmark::writer::MarkdownWriter {
            panic!("Not implemented")
        }

        fn render(&mut self, _node_id: crate::core::arena::NodeId) {
            panic!("Not implemented")
        }

        fn render_children(&mut self, _node_id: crate::core::arena::NodeId) {
            panic!("Not implemented")
        }

        fn get_formatting_phase(
            &self,
        ) -> crate::render::commonmark::phase::FormattingPhase {
            crate::render::commonmark::phase::FormattingPhase::Document
        }

        fn delegate_render(&mut self) {}

        fn get_formatter_options(&self) -> &crate::options::format::FormatOptions {
            panic!("Not implemented")
        }

        fn get_render_purpose(
            &self,
        ) -> crate::render::commonmark::purpose::RenderPurpose {
            crate::render::commonmark::purpose::RenderPurpose::Format
        }

        fn get_arena(&self) -> &crate::core::arena::NodeArena {
            panic!("Not implemented")
        }

        fn get_current_node(&self) -> Option<crate::core::arena::NodeId> {
            None
        }

        fn get_nodes_of_type(
            &self,
            _node_type: crate::render::commonmark::node::NodeValueType,
        ) -> Vec<crate::core::arena::NodeId> {
            vec![]
        }

        fn get_nodes_of_types(
            &self,
            _node_types: &[crate::render::commonmark::node::NodeValueType],
        ) -> Vec<crate::core::arena::NodeId> {
            vec![]
        }

        fn get_block_quote_like_prefix_predicate(&self) -> Box<dyn Fn(char) -> bool> {
            Box::new(|c| c == '>')
        }

        fn get_block_quote_like_prefix_chars(&self) -> &str {
            ">"
        }

        fn transform_non_translating(&self, text: &str) -> String {
            text.to_string()
        }

        fn transform_translating(&self, text: &str) -> String {
            text.to_string()
        }

        fn create_sub_context(&self) -> Box<dyn NodeFormatterContext> {
            panic!("Not implemented")
        }

        fn is_in_tight_list(&self) -> bool {
            false
        }

        fn set_tight_list(&mut self, _tight: bool) {}

        fn get_list_nesting_level(&self) -> usize {
            0
        }

        fn increment_list_nesting(&mut self) {}

        fn decrement_list_nesting(&mut self) {}

        fn is_in_block_quote(&self) -> bool {
            false
        }

        fn set_in_block_quote(&mut self, _in_block_quote: bool) {}

        fn get_block_quote_nesting_level(&self) -> usize {
            0
        }

        fn increment_block_quote_nesting(&mut self) {}

        fn decrement_block_quote_nesting(&mut self) {}

        fn start_table_collection(
            &mut self,
            _alignments: Vec<crate::core::nodes::TableAlignment>,
        ) {
        }

        fn add_table_row(&mut self) {}

        fn add_table_cell(&mut self, _content: String) {}

        fn take_table_data(
            &mut self,
        ) -> Option<(Vec<Vec<String>>, Vec<crate::core::nodes::TableAlignment>)>
        {
            None
        }

        fn is_collecting_table(&self) -> bool {
            false
        }

        fn set_skip_children(&mut self, _skip: bool) {}

        fn render_children_to_string(
            &mut self,
            _node_id: crate::core::arena::NodeId,
        ) -> String {
            String::new()
        }

        fn start_paragraph_line_breaking(&mut self, _max_width: usize, _prefix: String) {
        }

        fn finish_paragraph_line_breaking(&mut self) -> Option<String> {
            None
        }

        fn add_paragraph_text(&mut self, _text: &str) {}

        fn add_paragraph_word(&mut self, _text: &str) {}

        fn start_paragraph_unit(
            &mut self,
            _kind: crate::render::commonmark::line_breaking::UnitKind,
            _marker_width: usize,
        ) -> Option<crate::render::commonmark::line_breaking::UnitHandle> {
            None
        }

        fn end_paragraph_unit(
            &mut self,
            _handle: crate::render::commonmark::line_breaking::UnitHandle,
            _content_width: usize,
            _marker_width: usize,
        ) {
        }

        fn add_paragraph_unbreakable_unit(
            &mut self,
            _kind: crate::render::commonmark::line_breaking::UnitKind,
            _prefix: &str,
            _content: &str,
            _suffix: &str,
        ) {
        }

        fn add_paragraph_hard_break(&mut self) {}

        fn add_paragraph_atomic(
            &mut self,
            _content: &str,
            _kind: crate::render::commonmark::AtomicKind,
        ) {
        }

        fn is_paragraph_line_breaking(&self) -> bool {
            false
        }

        fn remove_paragraph_trailing_space(&mut self) {}

        fn paragraph_ends_with_whitespace(&self) -> bool {
            false
        }

        fn paragraph_ends_with_cjk(&self) -> bool {
            false
        }
    }

    #[test]
    fn test_commonmark_formatter_creation() {
        let formatter = CommonMarkNodeFormatter::new();
        let handlers = formatter.get_node_formatting_handlers();
        assert!(!handlers.is_empty());
        assert_eq!(handlers.len(), 26); // All node types including TableRow, TableCell, and ShortCode
    }

    #[test]
    fn test_commonmark_formatter_default() {
        let formatter: CommonMarkNodeFormatter = Default::default();
        let handlers = formatter.get_node_formatting_handlers();
        assert!(!handlers.is_empty());
    }

    #[test]
    fn test_commonmark_formatter_with_options() {
        let options = FormatOptions::new()
            .with_right_margin(80)
            .with_keep_hard_line_breaks(true);
        let formatter = CommonMarkNodeFormatter::with_options(options);
        let handlers = formatter.get_node_formatting_handlers();
        assert!(!handlers.is_empty());
        assert_eq!(formatter.options().right_margin, 80);
    }

    #[test]
    fn test_escape_text() {
        let ctx = MockContext;
        assert_eq!(escape_text("*text*", &ctx), "\\*text\\*");
        assert_eq!(escape_text("_text_", &ctx), "\\_text\\_");
        assert_eq!(escape_text("[link]", &ctx), "\\[link\\]");
        assert_eq!(escape_text("(paren)", &ctx), "(paren)"); // parentheses are not escaped
        assert_eq!(escape_text("`code`", &ctx), "\\`code\\`");
    }

    #[test]
    fn test_escape_text_no_special_chars() {
        let ctx = MockContext;
        assert_eq!(escape_text("plain text", &ctx), "plain text");
        assert_eq!(escape_text("123", &ctx), "123");
    }

    #[test]
    fn test_phased_formatter_phases() {
        let formatter = CommonMarkNodeFormatter::new();
        let phases = formatter.get_formatting_phases();
        // Now supports all 5 formatting phases as per flexmark-java
        assert_eq!(phases.len(), 5);
        assert!(phases.contains(&FormattingPhase::Collect));
        assert!(phases.contains(&FormattingPhase::DocumentFirst));
        assert!(phases.contains(&FormattingPhase::DocumentTop));
        assert!(phases.contains(&FormattingPhase::Document));
        assert!(phases.contains(&FormattingPhase::DocumentBottom));
    }

    #[test]
    fn test_formatter_options_accessor() {
        let options = FormatOptions::new().with_right_margin(100);
        let formatter = CommonMarkNodeFormatter::with_options(options);
        assert_eq!(formatter.options().right_margin, 100);
    }

    #[test]
    fn test_escape_string() {
        assert_eq!(escape_string("title"), "title");
        // escape_string replaces " with \" first, then \ with \\
        // Note: The order causes double-escaping of backslashes before quotes
        // ti"tle -> ti\"tle -> ti\\\"tle (quote escaped, then backslash escaped)
        assert_eq!(escape_string("ti\"tle"), "ti\\\\\"tle");
        // ti\tle -> ti\\tle (backslash escaped)
        assert_eq!(escape_string("ti\\tle"), "ti\\\\tle");
    }

    #[test]
    fn test_escape_url() {
        assert_eq!(escape_url("https://example.com"), "https://example.com");
        assert_eq!(escape_url("url with space"), "url\\ with\\ space");
        assert_eq!(escape_url("(paren)"), "\\(paren\\)");
    }

    #[test]
    fn test_render_document_with_nested_lists() {
        use crate::core::arena::{Node, NodeArena, TreeOps};
        use crate::core::nodes::{ListDelimType, ListType, NodeList, NodeValue};
        use crate::render::commonmark::{FormatOptions, Formatter};

        let mut arena = NodeArena::new();

        // Create: - Item 1
        //         - Item 2
        //           - Nested
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        let list = arena.alloc(Node::with_value(NodeValue::List(NodeList {
            list_type: ListType::Bullet,
            marker_offset: 0,
            padding: 0,
            start: 1,
            delimiter: ListDelimType::Period,
            bullet_char: b'-',
            tight: true,
            is_task_list: false,
        })));

        let item1 = arena.alloc(Node::with_value(NodeValue::Item(NodeList {
            list_type: ListType::Bullet,
            marker_offset: 0,
            padding: 0,
            start: 0,
            delimiter: ListDelimType::Period,
            bullet_char: 0,
            tight: true,
            is_task_list: false,
        })));
        let text1 = arena.alloc(Node::with_value(NodeValue::make_text("Item 1")));
        let para1 = arena.alloc(Node::with_value(NodeValue::Paragraph));
        TreeOps::append_child(&mut arena, para1, text1);
        TreeOps::append_child(&mut arena, item1, para1);

        let item2 = arena.alloc(Node::with_value(NodeValue::Item(NodeList {
            list_type: ListType::Bullet,
            marker_offset: 0,
            padding: 0,
            start: 0,
            delimiter: ListDelimType::Period,
            bullet_char: 0,
            tight: true,
            is_task_list: false,
        })));
        let text2 = arena.alloc(Node::with_value(NodeValue::make_text("Item 2")));
        let para2 = arena.alloc(Node::with_value(NodeValue::Paragraph));
        TreeOps::append_child(&mut arena, para2, text2);
        TreeOps::append_child(&mut arena, item2, para2);

        let nested_list = arena.alloc(Node::with_value(NodeValue::List(NodeList {
            list_type: ListType::Bullet,
            marker_offset: 0,
            padding: 0,
            start: 1,
            delimiter: ListDelimType::Period,
            bullet_char: b'-',
            tight: true,
            is_task_list: false,
        })));

        let nested_item = arena.alloc(Node::with_value(NodeValue::Item(NodeList {
            list_type: ListType::Bullet,
            marker_offset: 0,
            padding: 0,
            start: 0,
            delimiter: ListDelimType::Period,
            bullet_char: 0,
            tight: true,
            is_task_list: false,
        })));
        let nested_text = arena.alloc(Node::with_value(NodeValue::make_text("Nested")));
        let nested_para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        TreeOps::append_child(&mut arena, nested_para, nested_text);
        TreeOps::append_child(&mut arena, nested_item, nested_para);
        TreeOps::append_child(&mut arena, nested_list, nested_item);
        TreeOps::append_child(&mut arena, item2, nested_list);

        TreeOps::append_child(&mut arena, list, item1);
        TreeOps::append_child(&mut arena, list, item2);
        TreeOps::append_child(&mut arena, root, list);

        let options = FormatOptions::new();
        let mut formatter = Formatter::with_options(options);
        formatter.add_node_formatter(Box::new(CommonMarkNodeFormatter::new()));

        let result = formatter.render(&arena, root);

        assert!(
            result.contains("- Item 1"),
            "Should contain first item: {}",
            result
        );
        assert!(
            result.contains("- Item 2"),
            "Should contain second item: {}",
            result
        );
        assert!(
            result.contains("  - Nested"),
            "Should contain nested item with indent: {}",
            result
        );
    }

    #[test]
    fn test_render_code_block_with_backticks() {
        use crate::core::arena::{Node, NodeArena, TreeOps};
        use crate::core::nodes::{NodeCodeBlock, NodeValue};
        use crate::render::commonmark::{FormatOptions, Formatter};

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        let code_block = arena.alloc(Node::with_value(NodeValue::CodeBlock(Box::new(
            NodeCodeBlock {
                fenced: true,
                fence_char: b'`',
                fence_length: 3,
                fence_offset: 0,
                info: "rust".to_string(),
                literal: "fn main() {}".to_string(),
                closed: true,
            },
        ))));

        TreeOps::append_child(&mut arena, root, code_block);

        let options = FormatOptions::new();
        let mut formatter = Formatter::with_options(options);
        formatter.add_node_formatter(Box::new(CommonMarkNodeFormatter::new()));

        let result = formatter.render(&arena, root);

        assert!(
            result.contains("```rust"),
            "Should contain opening fence with info: {}",
            result
        );
        assert!(
            result.contains("fn main() {}"),
            "Should contain code content: {}",
            result
        );
        assert!(
            result.contains("```"),
            "Should contain closing fence: {}",
            result
        );
    }

    #[test]
    fn test_render_heading_atx() {
        use crate::core::arena::{Node, NodeArena, TreeOps};
        use crate::core::nodes::{NodeHeading, NodeValue};
        use crate::render::commonmark::{FormatOptions, Formatter};

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        let heading = arena.alloc(Node::with_value(NodeValue::Heading(NodeHeading {
            level: 2,
            setext: false,
            closed: false,
        })));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("Section Title")));
        TreeOps::append_child(&mut arena, heading, text);
        TreeOps::append_child(&mut arena, root, heading);

        let options = FormatOptions::new();
        let mut formatter = Formatter::with_options(options);
        formatter.add_node_formatter(Box::new(CommonMarkNodeFormatter::new()));

        let result = formatter.render(&arena, root);

        assert!(
            result.contains("## Section Title"),
            "Should contain ATX heading: {}",
            result
        );
    }

    #[test]
    fn test_render_blockquote() {
        use crate::core::arena::{Node, NodeArena, TreeOps};
        use crate::core::nodes::NodeValue;
        use crate::render::commonmark::{FormatOptions, Formatter};

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        let blockquote = arena.alloc(Node::with_value(NodeValue::BlockQuote));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("Quoted text")));

        TreeOps::append_child(&mut arena, para, text);
        TreeOps::append_child(&mut arena, blockquote, para);
        TreeOps::append_child(&mut arena, root, blockquote);

        let options = FormatOptions::new();
        let mut formatter = Formatter::with_options(options);
        formatter.add_node_formatter(Box::new(CommonMarkNodeFormatter::new()));

        let result = formatter.render(&arena, root);

        assert!(
            result.contains("> Quoted text"),
            "Should contain blockquote: {}",
            result
        );
    }

    #[test]
    fn test_render_link_and_image() {
        use crate::core::arena::{Node, NodeArena, TreeOps};
        use crate::core::nodes::{NodeLink, NodeValue};
        use crate::render::commonmark::{FormatOptions, Formatter};

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));

        // Create link: [example](https://example.com)
        let link = arena.alloc(Node::with_value(NodeValue::Link(Box::new(NodeLink {
            url: "https://example.com".to_string(),
            title: "".to_string(),
        }))));
        let link_text = arena.alloc(Node::with_value(NodeValue::make_text("example")));
        TreeOps::append_child(&mut arena, link, link_text);
        TreeOps::append_child(&mut arena, para, link);

        // Create image: ![alt](image.png)
        let image =
            arena.alloc(Node::with_value(NodeValue::Image(Box::new(NodeLink {
                url: "image.png".to_string(),
                title: "".to_string(),
            }))));
        let image_alt = arena.alloc(Node::with_value(NodeValue::make_text("alt")));
        TreeOps::append_child(&mut arena, image, image_alt);
        TreeOps::append_child(&mut arena, para, image);

        TreeOps::append_child(&mut arena, root, para);

        let options = FormatOptions::new();
        let mut formatter = Formatter::with_options(options);
        formatter.add_node_formatter(Box::new(CommonMarkNodeFormatter::new()));

        let result = formatter.render(&arena, root);

        assert!(
            result.contains("[example](https://example.com)"),
            "Should contain link: {}",
            result
        );
        assert!(
            result.contains("![alt](image.png)"),
            "Should contain image: {}",
            result
        );
    }

    #[test]
    fn test_render_emphasis_and_strong() {
        use crate::core::arena::{Node, NodeArena, TreeOps};
        use crate::core::nodes::NodeValue;
        use crate::render::commonmark::{FormatOptions, Formatter};

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));

        // Create: *emphasis* and **strong**
        let emph = arena.alloc(Node::with_value(NodeValue::Emph));
        let emph_text = arena.alloc(Node::with_value(NodeValue::make_text("emphasis")));
        TreeOps::append_child(&mut arena, emph, emph_text);
        TreeOps::append_child(&mut arena, para, emph);

        let strong = arena.alloc(Node::with_value(NodeValue::Strong));
        let strong_text = arena.alloc(Node::with_value(NodeValue::make_text("strong")));
        TreeOps::append_child(&mut arena, strong, strong_text);
        TreeOps::append_child(&mut arena, para, strong);

        TreeOps::append_child(&mut arena, root, para);

        let options = FormatOptions::new();
        let mut formatter = Formatter::with_options(options);
        formatter.add_node_formatter(Box::new(CommonMarkNodeFormatter::new()));

        let result = formatter.render(&arena, root);

        assert!(
            result.contains("*emphasis*"),
            "Should contain emphasis: {}",
            result
        );
        assert!(
            result.contains("**strong**"),
            "Should contain strong: {}",
            result
        );
    }

    #[test]
    fn test_is_task_item_checked() {
        use crate::core::arena::{Node, NodeArena, TreeOps};
        use crate::core::nodes::{ListDelimType, ListType, NodeList, NodeValue};

        let mut arena = NodeArena::new();

        // Create a task list item with [x]
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let list = arena.alloc(Node::with_value(NodeValue::List(NodeList {
            list_type: ListType::Bullet,
            marker_offset: 0,
            padding: 0,
            start: 1,
            delimiter: ListDelimType::Period,
            bullet_char: b'-',
            tight: true,
            is_task_list: true,
        })));

        let item = arena.alloc(Node::with_value(NodeValue::Item(NodeList {
            list_type: ListType::Bullet,
            marker_offset: 0,
            padding: 0,
            start: 0,
            delimiter: ListDelimType::Period,
            bullet_char: 0,
            tight: true,
            is_task_list: true,
        })));

        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text =
            arena.alloc(Node::with_value(NodeValue::make_text("[x] Checked task")));

        TreeOps::append_child(&mut arena, para, text);
        TreeOps::append_child(&mut arena, item, para);
        TreeOps::append_child(&mut arena, list, item);
        TreeOps::append_child(&mut arena, root, list);

        // Test that is_task_item_checked returns true for [x]
        assert!(
            is_task_item_checked(&arena, Some(item)),
            "Should detect checked task item"
        );

        // Create another task list item with [ ]
        let item2 = arena.alloc(Node::with_value(NodeValue::Item(NodeList {
            list_type: ListType::Bullet,
            marker_offset: 0,
            padding: 0,
            start: 0,
            delimiter: ListDelimType::Period,
            bullet_char: 0,
            tight: true,
            is_task_list: true,
        })));

        let para2 = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text2 =
            arena.alloc(Node::with_value(NodeValue::make_text("[ ] Unchecked task")));

        TreeOps::append_child(&mut arena, para2, text2);
        TreeOps::append_child(&mut arena, item2, para2);
        TreeOps::append_child(&mut arena, list, item2);

        // Test that is_task_item_checked returns false for [ ]
        assert!(
            !is_task_item_checked(&arena, Some(item2)),
            "Should detect unchecked task item"
        );

        // Test with [X] (uppercase)
        let item3 = arena.alloc(Node::with_value(NodeValue::Item(NodeList {
            list_type: ListType::Bullet,
            marker_offset: 0,
            padding: 0,
            start: 0,
            delimiter: ListDelimType::Period,
            bullet_char: 0,
            tight: true,
            is_task_list: true,
        })));

        let para3 = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text3 = arena.alloc(Node::with_value(NodeValue::make_text(
            "[X] Checked task uppercase",
        )));

        TreeOps::append_child(&mut arena, para3, text3);
        TreeOps::append_child(&mut arena, item3, para3);
        TreeOps::append_child(&mut arena, list, item3);

        // Test that is_task_item_checked returns true for [X]
        assert!(
            is_task_item_checked(&arena, Some(item3)),
            "Should detect checked task item with uppercase X"
        );

        // Test with None
        assert!(
            !is_task_item_checked(&arena, None),
            "Should return false for None"
        );
    }

    #[test]
    fn test_render_task_list() {
        use crate::core::arena::{Node, NodeArena, TreeOps};
        use crate::core::nodes::{ListDelimType, ListType, NodeList, NodeValue};
        use crate::render::commonmark::{FormatOptions, Formatter};

        let mut arena = NodeArena::new();

        // Create a task list
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let list = arena.alloc(Node::with_value(NodeValue::List(NodeList {
            list_type: ListType::Bullet,
            marker_offset: 0,
            padding: 0,
            start: 1,
            delimiter: ListDelimType::Period,
            bullet_char: b'-',
            tight: true,
            is_task_list: true,
        })));

        // First item: [ ] Unchecked
        let item1 = arena.alloc(Node::with_value(NodeValue::Item(NodeList {
            list_type: ListType::Bullet,
            marker_offset: 0,
            padding: 0,
            start: 0,
            delimiter: ListDelimType::Period,
            bullet_char: 0,
            tight: true,
            is_task_list: true,
        })));
        let para1 = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text1 =
            arena.alloc(Node::with_value(NodeValue::make_text("[ ] Unchecked task")));
        TreeOps::append_child(&mut arena, para1, text1);
        TreeOps::append_child(&mut arena, item1, para1);
        TreeOps::append_child(&mut arena, list, item1);

        // Second item: [x] Checked
        let item2 = arena.alloc(Node::with_value(NodeValue::Item(NodeList {
            list_type: ListType::Bullet,
            marker_offset: 0,
            padding: 0,
            start: 0,
            delimiter: ListDelimType::Period,
            bullet_char: 0,
            tight: true,
            is_task_list: true,
        })));
        let para2 = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text2 =
            arena.alloc(Node::with_value(NodeValue::make_text("[x] Checked task")));
        TreeOps::append_child(&mut arena, para2, text2);
        TreeOps::append_child(&mut arena, item2, para2);
        TreeOps::append_child(&mut arena, list, item2);

        TreeOps::append_child(&mut arena, root, list);

        let options = FormatOptions::new();
        let mut formatter = Formatter::with_options(options);
        formatter.add_node_formatter(Box::new(CommonMarkNodeFormatter::new()));

        let result = formatter.render(&arena, root);

        // Check that task list markers are rendered
        assert!(
            result.contains("- [ ]"),
            "Should contain unchecked task marker: {}",
            result
        );
        assert!(
            result.contains("- [x]"),
            "Should contain checked task marker: {}",
            result
        );
        assert!(
            result.contains("Unchecked task"),
            "Should contain unchecked task text: {}",
            result
        );
        assert!(
            result.contains("Checked task"),
            "Should contain checked task text: {}",
            result
        );
    }

    // ========================================================================
    // Boundary Condition Tests
    // ========================================================================

    #[test]
    fn test_render_empty_document() {
        use crate::core::arena::{Node, NodeArena};
        use crate::core::nodes::NodeValue;
        use crate::render::commonmark::{FormatOptions, Formatter};

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        let options = FormatOptions::new();
        let mut formatter = Formatter::with_options(options);
        formatter.add_node_formatter(Box::new(CommonMarkNodeFormatter::new()));

        let result = formatter.render(&arena, root);
        assert!(result.is_empty() || result.trim().is_empty());
    }

    #[test]
    fn test_render_empty_paragraph() {
        use crate::core::arena::{Node, NodeArena, TreeOps};
        use crate::core::nodes::NodeValue;
        use crate::render::commonmark::{FormatOptions, Formatter};

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        TreeOps::append_child(&mut arena, root, para);

        let options = FormatOptions::new();
        let mut formatter = Formatter::with_options(options);
        formatter.add_node_formatter(Box::new(CommonMarkNodeFormatter::new()));

        let result = formatter.render(&arena, root);
        assert!(result.is_empty() || result.trim().is_empty());
    }

    #[test]
    fn test_render_special_characters_in_text() {
        use crate::core::arena::{Node, NodeArena, TreeOps};
        use crate::core::nodes::NodeValue;
        use crate::render::commonmark::{FormatOptions, Formatter};

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("*_`[]<>#\\|")));
        TreeOps::append_child(&mut arena, para, text);
        TreeOps::append_child(&mut arena, root, para);

        let options = FormatOptions::new();
        let mut formatter = Formatter::with_options(options);
        formatter.add_node_formatter(Box::new(CommonMarkNodeFormatter::new()));

        let result = formatter.render(&arena, root);
        assert!(result.contains("\\*"));
        assert!(result.contains("\\_"));
        assert!(result.contains("\\`"));
    }

    #[test]
    fn test_render_unicode_text() {
        use crate::core::arena::{Node, NodeArena, TreeOps};
        use crate::core::nodes::NodeValue;
        use crate::render::commonmark::{FormatOptions, Formatter};

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::make_text(
            "中文测试 日本語 한국어",
        )));
        TreeOps::append_child(&mut arena, para, text);
        TreeOps::append_child(&mut arena, root, para);

        let options = FormatOptions::new();
        let mut formatter = Formatter::with_options(options);
        formatter.add_node_formatter(Box::new(CommonMarkNodeFormatter::new()));

        let result = formatter.render(&arena, root);
        assert!(result.contains("中文测试"));
        assert!(result.contains("日本語"));
        assert!(result.contains("한국어"));
    }

    #[test]
    fn test_render_very_long_text() {
        use crate::core::arena::{Node, NodeArena, TreeOps};
        use crate::core::nodes::NodeValue;
        use crate::render::commonmark::{FormatOptions, Formatter};

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let long_text = "a".repeat(10000);
        let text =
            arena.alloc(Node::with_value(NodeValue::make_text(long_text.clone())));
        TreeOps::append_child(&mut arena, para, text);
        TreeOps::append_child(&mut arena, root, para);

        let options = FormatOptions::new();
        let mut formatter = Formatter::with_options(options);
        formatter.add_node_formatter(Box::new(CommonMarkNodeFormatter::new()));

        let result = formatter.render(&arena, root);
        assert!(result.starts_with(&long_text));
        assert!(result.len() >= 10000);
    }

    #[test]
    fn test_render_deeply_nested_structure() {
        use crate::core::arena::{Node, NodeArena, TreeOps};
        use crate::core::nodes::NodeValue;
        use crate::render::commonmark::{FormatOptions, Formatter};

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        let mut current = root;
        for _ in 0..10 {
            let blockquote = arena.alloc(Node::with_value(NodeValue::BlockQuote));
            TreeOps::append_child(&mut arena, current, blockquote);
            current = blockquote;
        }

        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("Deep")));
        TreeOps::append_child(&mut arena, para, text);
        TreeOps::append_child(&mut arena, current, para);

        let options = FormatOptions::new();
        let mut formatter = Formatter::with_options(options);
        formatter.add_node_formatter(Box::new(CommonMarkNodeFormatter::new()));

        let result = formatter.render(&arena, root);
        assert!(result.contains("Deep"));
        assert!(result.contains(">"));
    }

    #[test]
    fn test_render_code_block_empty() {
        use crate::core::arena::{Node, NodeArena, TreeOps};
        use crate::core::nodes::{NodeCodeBlock, NodeValue};
        use crate::render::commonmark::{FormatOptions, Formatter};

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        let code_block = arena.alloc(Node::with_value(NodeValue::CodeBlock(Box::new(
            NodeCodeBlock {
                fenced: true,
                fence_char: b'`',
                fence_length: 3,
                fence_offset: 0,
                info: String::new(),
                literal: String::new(),
                closed: true,
            },
        ))));
        TreeOps::append_child(&mut arena, root, code_block);

        let options = FormatOptions::new();
        let mut formatter = Formatter::with_options(options);
        formatter.add_node_formatter(Box::new(CommonMarkNodeFormatter::new()));

        let result = formatter.render(&arena, root);
        assert!(result.contains("```"));
    }

    #[test]
    fn test_render_link_empty_url() {
        use crate::core::arena::{Node, NodeArena, TreeOps};
        use crate::core::nodes::{NodeLink, NodeValue};
        use crate::render::commonmark::{FormatOptions, Formatter};

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));

        let link = arena.alloc(Node::with_value(NodeValue::Link(Box::new(NodeLink {
            url: String::new(),
            title: String::new(),
        }))));
        let link_text =
            arena.alloc(Node::with_value(NodeValue::make_text("empty link")));
        TreeOps::append_child(&mut arena, link, link_text);
        TreeOps::append_child(&mut arena, para, link);
        TreeOps::append_child(&mut arena, root, para);

        let options = FormatOptions::new();
        let mut formatter = Formatter::with_options(options);
        formatter.add_node_formatter(Box::new(CommonMarkNodeFormatter::new()));

        let result = formatter.render(&arena, root);
        assert!(result.contains("[empty link]()"));
    }
}
