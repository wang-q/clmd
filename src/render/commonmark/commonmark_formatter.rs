//! CommonMark node formatter implementation
//!
//! This module provides a NodeFormatter implementation for CommonMark output,
//! migrating the existing commonmark.rs functionality to the formatter framework.
//!
//! # Example
//!
//! ```ignore
//! use clmd::render::commonmark::{CommonMarkNodeFormatter, FormatterOptions};
//!
//! let formatter = CommonMarkNodeFormatter::new();
//! let options = FormatOptions::new().with_right_margin(80);
//! let formatter = CommonMarkNodeFormatter::with_options(options);
//! ```

use crate::core::arena::NodeId;
use crate::core::nodes::NodeValue;
use crate::options::format::{FormatOptions, HeadingStyle};
use crate::render::commonmark::context::NodeFormatterContext;
use crate::render::commonmark::escaping::{
    compute_fence_length, escape_markdown_for_table_simple, escape_string, escape_text,
    escape_url,
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
    /// use clmd::render::commonmark::{CommonMarkNodeFormatter, FormatterOptions};
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
            // Document
            NodeFormattingHandler::new(
                NodeValueType::Document,
                Box::new(
                    |_value: &NodeValue,
                     _ctx: &mut dyn NodeFormatterContext,
                     _writer: &mut MarkdownWriter| {
                        // Document is handled at the top level
                    },
                ),
            ),
            // Block elements
            NodeFormattingHandler::with_close(
                NodeValueType::Paragraph,
                Box::new(
                    |_value: &NodeValue,
                     ctx: &mut dyn NodeFormatterContext,
                     _writer: &mut MarkdownWriter| {
                        // Paragraph opening - start paragraph line breaking if enabled
                        if let NodeValue::Paragraph = _value {
                            let options = ctx.get_formatter_options();
                            // Only enable line breaking if right_margin is set
                            if options.right_margin > 0 {
                                // Calculate nesting levels
                                let list_nesting = ctx.get_list_nesting_level();
                                let block_quote_nesting =
                                    ctx.get_block_quote_nesting_level();

                                // Determine which prefix to use and calculate marker width
                                let (prefix, marker_width) = if ctx.is_parent_list_item()
                                {
                                    // List item: use continuation prefix and calculate marker width
                                    let (_, cont_prefix) =
                                        calculate_list_item_prefixes(ctx);
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
                                // Ensure max_width is at least 20 to avoid degenerate cases
                                let max_width = max_width.max(20);

                                // Start the new paragraph line breaker
                                ctx.start_paragraph_line_breaking(max_width, prefix);
                            }
                        }
                    },
                ),
                Box::new(
                    |_value: &NodeValue,
                     ctx: &mut dyn NodeFormatterContext,
                     writer: &mut MarkdownWriter| {
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
            ),
            NodeFormattingHandler::with_close(
                NodeValueType::Heading,
                Box::new(
                    |value: &NodeValue,
                     ctx: &mut dyn NodeFormatterContext,
                     writer: &mut MarkdownWriter| {
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
                ),
                Box::new(
                    |value: &NodeValue,
                     ctx: &mut dyn NodeFormatterContext,
                     writer: &mut MarkdownWriter| {
                        if let NodeValue::Heading(heading) = value {
                            let options = ctx.get_formatter_options();
                            let heading_style = options.heading_style;
                            let min_setext_marker_length =
                                options.min_setext_marker_length;
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
            ),
            NodeFormattingHandler::with_close(
                NodeValueType::BlockQuote,
                Box::new(
                    |_value: &NodeValue,
                     ctx: &mut dyn NodeFormatterContext,
                     writer: &mut MarkdownWriter| {
                        writer.push_prefix("> ");
                        ctx.set_in_block_quote(true);
                        ctx.increment_block_quote_nesting();
                    },
                ),
                Box::new(
                    |_value: &NodeValue,
                     ctx: &mut dyn NodeFormatterContext,
                     writer: &mut MarkdownWriter| {
                        writer.pop_prefix();
                        ctx.decrement_block_quote_nesting();
                        if ctx.get_block_quote_nesting_level() == 0 {
                            ctx.set_in_block_quote(false);
                        }
                    },
                ),
            ),
            NodeFormattingHandler::new(
                NodeValueType::CodeBlock,
                Box::new(
                    |value: &NodeValue,
                     ctx: &mut dyn NodeFormatterContext,
                     writer: &mut MarkdownWriter| {
                        if let NodeValue::CodeBlock(code_block) = value {
                            render_code_block(code_block, ctx, writer);
                        }
                    },
                ),
            ),
            NodeFormattingHandler::with_close(
                NodeValueType::List,
                Box::new(
                    |value: &NodeValue,
                     ctx: &mut dyn NodeFormatterContext,
                     _writer: &mut MarkdownWriter| {
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
                                    crate::options::format::ListSpacing::AsIs => {
                                        list.tight
                                    }
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
                ),
                Box::new(
                    |_value: &NodeValue,
                     ctx: &mut dyn NodeFormatterContext,
                     writer: &mut MarkdownWriter| {
                        ctx.decrement_list_nesting();
                        if ctx.get_list_nesting_level() == 0 {
                            ctx.set_tight_list(false);
                            // Add blank line after list ends to separate from following content
                            writer.blank_line();
                        }
                    },
                ),
            ),
            NodeFormattingHandler::with_close(
                NodeValueType::Item,
                Box::new(
                    |value: &NodeValue,
                     ctx: &mut dyn NodeFormatterContext,
                     writer: &mut MarkdownWriter| {
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

                                let marker =
                                    format_list_item_marker_with_number_and_options(
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
                ),
                Box::new(
                    |_value: &NodeValue,
                     ctx: &mut dyn NodeFormatterContext,
                     writer: &mut MarkdownWriter| {
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
            ),
            NodeFormattingHandler::new(
                NodeValueType::ThematicBreak,
                Box::new(
                    |value: &NodeValue,
                     _ctx: &mut dyn NodeFormatterContext,
                     writer: &mut MarkdownWriter| {
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
            ),
            NodeFormattingHandler::new(
                NodeValueType::HtmlBlock,
                Box::new(
                    |value: &NodeValue,
                     ctx: &mut dyn NodeFormatterContext,
                     writer: &mut MarkdownWriter| {
                        if let NodeValue::HtmlBlock(html) = value {
                            render_html_block(html, ctx, writer);
                        }
                    },
                ),
            ),
            // Inline elements
            NodeFormattingHandler::new(
                NodeValueType::Text,
                Box::new(
                    |value: &NodeValue,
                     ctx: &mut dyn NodeFormatterContext,
                     writer: &mut MarkdownWriter| {
                        if let NodeValue::Text(text) = value {
                            let text_str = text.as_ref();

                            // Check if we're in a task list item and need to skip the task marker
                            let processed_text = if is_in_task_list_item(ctx) {
                                skip_task_marker(text_str)
                            } else {
                                text_str.to_string()
                            };

                            // Apply CJK spacing by default
                            // NOTE: We apply CJK spacing even when using paragraph line breaking
                            // to ensure proper spacing around markdown markers
                            let final_text = crate::text::cjk_spacing::add_cjk_spacing(
                                &processed_text,
                            );

                            // Check if we're using the new paragraph line breaking system
                            if ctx.is_paragraph_line_breaking() {
                                // Add text to paragraph line breaker
                                ctx.add_paragraph_text(&final_text);
                            } else if ctx.is_collecting_line_breaking() {
                                // Add text to legacy line breaking context
                                ctx.add_line_breaking_text(&final_text);
                            } else {
                                // Use context-aware escaping
                                let escaped = escape_text(&final_text, ctx);
                                // Use append_with_wrap for text wrapping when right_margin is set
                                // This enables line folding at the specified width
                                writer.append_with_wrap(&escaped);
                            }
                        }
                    },
                ),
            ),
            NodeFormattingHandler::new(
                NodeValueType::Code,
                Box::new(
                    |value: &NodeValue,
                     ctx: &mut dyn NodeFormatterContext,
                     writer: &mut MarkdownWriter| {
                        if let NodeValue::Code(code) = value {
                            // Calculate the required fence length based on content
                            // Need to account for backticks in the content
                            let fence_len = compute_fence_length(&code.literal, 1);
                            let backticks = "`".repeat(fence_len);

                            // Determine if we need padding (spaces around content)
                            // Padding is needed when:
                            // 1. Content starts or ends with a backtick
                            // 2. Content starts or ends with a space
                            let needs_leading_space = code.literal.starts_with('`')
                                || code.literal.starts_with(' ');
                            let needs_trailing_space = code.literal.ends_with('`')
                                || code.literal.ends_with(' ');

                            // Check if we're using the new paragraph line breaking system
                            if ctx.is_paragraph_line_breaking() {
                                // Add inline code as unbreakable unit
                                let prefix = backticks.clone();
                                let suffix = backticks.clone();
                                let content =
                                    if needs_leading_space || needs_trailing_space {
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
                            } else if ctx.is_collecting_line_breaking() {
                                // Add code span as an inline element to preserve surrounding spaces
                                let mut code_text = backticks.clone();
                                if needs_leading_space {
                                    code_text.push(' ');
                                }
                                code_text.push_str(&code.literal);
                                if needs_trailing_space {
                                    code_text.push(' ');
                                }
                                code_text.push_str(&backticks);
                                ctx.add_line_breaking_inline_element(&code_text);
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
                    },
                ),
            ),
            NodeFormattingHandler::with_close(
                NodeValueType::Emph,
                Box::new(
                    |_value: &NodeValue,
                     ctx: &mut dyn NodeFormatterContext,
                     writer: &mut MarkdownWriter| {
                        let marker = "*";
                        if ctx.is_paragraph_line_breaking() {
                            // Remove trailing space before markdown marker for CJK spacing
                            ctx.remove_paragraph_trailing_space();
                            // Use add_paragraph_word to prevent internal breaks
                            ctx.add_paragraph_word(marker);
                        } else if ctx.is_collecting_line_breaking() {
                            ctx.add_line_breaking_word_text(marker);
                        } else {
                            writer.append(marker);
                        }
                    },
                ),
                Box::new(
                    |_value: &NodeValue,
                     ctx: &mut dyn NodeFormatterContext,
                     writer: &mut MarkdownWriter| {
                        let marker = "*";
                        if ctx.is_paragraph_line_breaking() {
                            // Use add_paragraph_word to prevent internal breaks
                            ctx.add_paragraph_word(marker);
                        } else if ctx.is_collecting_line_breaking() {
                            ctx.add_line_breaking_marker_end(marker);
                        } else {
                            writer.append(marker);
                        }
                    },
                ),
            ),
            NodeFormattingHandler::with_close(
                NodeValueType::Strong,
                Box::new(
                    |_value: &NodeValue,
                     ctx: &mut dyn NodeFormatterContext,
                     writer: &mut MarkdownWriter| {
                        let marker = "**";
                        if ctx.is_paragraph_line_breaking() {
                            // Remove trailing space before markdown marker for CJK spacing
                            ctx.remove_paragraph_trailing_space();
                            // Use add_paragraph_word to prevent internal breaks
                            ctx.add_paragraph_word(marker);
                        } else if ctx.is_collecting_line_breaking() {
                            ctx.add_line_breaking_word_text(marker);
                        } else {
                            writer.append(marker);
                        }
                    },
                ),
                Box::new(
                    |_value: &NodeValue,
                     ctx: &mut dyn NodeFormatterContext,
                     writer: &mut MarkdownWriter| {
                        let marker = "**";
                        if ctx.is_paragraph_line_breaking() {
                            // Use add_paragraph_word to prevent internal breaks
                            ctx.add_paragraph_word(marker);
                        } else if ctx.is_collecting_line_breaking() {
                            ctx.add_line_breaking_marker_end(marker);
                        } else {
                            writer.append(marker);
                        }
                    },
                ),
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
                        } else if ctx.is_collecting_line_breaking() {
                            ctx.add_line_breaking_word_text("[");
                            // Enter link text mode to prevent line breaks inside link text
                            ctx.enter_link_text();
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
                            } else if ctx.is_collecting_line_breaking() {
                                // Exit link text mode before closing bracket
                                ctx.exit_link_text();
                                // Close the link text bracket and add URL as words
                                ctx.add_line_breaking_word_text("]");
                                ctx.add_line_breaking_word_text("(");
                                // Use add_line_breaking_url to add URL as a single word (not a markdown marker)
                                // This allows long URLs to break lines properly without being split
                                ctx.add_line_breaking_url(&link.url);
                                if !link.title.is_empty() {
                                    ctx.add_line_breaking_word_text(&format!(
                                        " \"{}\"",
                                        link.title
                                    ));
                                }
                                ctx.add_line_breaking_link_close(")");
                                // Exit link URL mode after closing parenthesis
                                ctx.exit_link_url();
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
                        } else if ctx.is_collecting_line_breaking() {
                            ctx.add_line_breaking_word_text("![");
                            // Enter link text mode to prevent line breaks inside alt text
                            ctx.enter_link_text();
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
                            } else if ctx.is_collecting_line_breaking() {
                                // Exit link text mode before closing bracket
                                ctx.exit_link_text();
                                // Close the image alt bracket and add URL as words
                                ctx.add_line_breaking_word_text("]");
                                ctx.add_line_breaking_word_text("(");
                                // Use add_line_breaking_url to add URL as a single word (not a markdown marker)
                                ctx.add_line_breaking_url(&link.url);
                                if !link.title.is_empty() {
                                    ctx.add_line_breaking_word_text(&format!(
                                        " \"{}\"",
                                        link.title
                                    ));
                                }
                                ctx.add_line_breaking_link_close(")");
                                // Exit link URL mode after closing parenthesis
                                ctx.exit_link_url();
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
            NodeFormattingHandler::with_close(
                NodeValueType::Strikethrough,
                Box::new(
                    |_value: &NodeValue,
                     _ctx: &mut dyn NodeFormatterContext,
                     writer: &mut MarkdownWriter| {
                        writer.append("~~");
                    },
                ),
                Box::new(
                    |_value: &NodeValue,
                     _ctx: &mut dyn NodeFormatterContext,
                     writer: &mut MarkdownWriter| {
                        writer.append("~~");
                    },
                ),
            ),
            NodeFormattingHandler::new(
                NodeValueType::SoftBreak,
                Box::new(
                    |_value: &NodeValue,
                     ctx: &mut dyn NodeFormatterContext,
                     writer: &mut MarkdownWriter| {
                        // Based on flexmark-java's SoftLineBreak handling:
                        // 1. In tight lists, soft breaks become spaces
                        // 2. If keepSoftLineBreaks is enabled, preserve the break
                        // 3. Otherwise, convert to space (for wrapping) or line break
                        let options = ctx.get_formatter_options();

                        // Check if we're using the new paragraph line breaking system
                        if ctx.is_paragraph_line_breaking() {
                            // Add space for soft break
                            ctx.add_paragraph_text(" ");
                        } else if ctx.is_collecting_line_breaking() {
                            // Add space to line breaking context for proper wrapping
                            ctx.add_line_breaking_text(" ");
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
                    },
                ),
            ),
            NodeFormattingHandler::new(
                NodeValueType::HardBreak,
                Box::new(
                    |_value: &NodeValue,
                     ctx: &mut dyn NodeFormatterContext,
                     writer: &mut MarkdownWriter| {
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
                    },
                ),
            ),
            NodeFormattingHandler::new(
                NodeValueType::HtmlInline,
                Box::new(
                    |value: &NodeValue,
                     _ctx: &mut dyn NodeFormatterContext,
                     writer: &mut MarkdownWriter| {
                        if let NodeValue::HtmlInline(html) = value {
                            writer.append(html);
                        }
                    },
                ),
            ),
            // Table elements
            NodeFormattingHandler::with_close(
                NodeValueType::Table,
                Box::new(
                    |value: &NodeValue,
                     ctx: &mut dyn NodeFormatterContext,
                     _writer: &mut MarkdownWriter| {
                        // Table opening - start collecting data
                        if let NodeValue::Table(table) = value {
                            ctx.start_table_collection(table.alignments.clone());
                        }
                    },
                ),
                Box::new(
                    |_value: &NodeValue,
                     ctx: &mut dyn NodeFormatterContext,
                     writer: &mut MarkdownWriter| {
                        // Table closing - format and output using table.rs
                        if let Some((rows, alignments)) = ctx.take_table_data() {
                            render_formatted_table(&rows, &alignments, writer);
                        }
                    },
                ),
            ),
            NodeFormattingHandler::with_close(
                NodeValueType::TableRow,
                Box::new(
                    |_value: &NodeValue,
                     ctx: &mut dyn NodeFormatterContext,
                     _writer: &mut MarkdownWriter| {
                        // Row opening - add new row to collection
                        ctx.add_table_row();
                    },
                ),
                Box::new(
                    |_value: &NodeValue,
                     _ctx: &mut dyn NodeFormatterContext,
                     _writer: &mut MarkdownWriter| {
                        // Row closing - nothing to do
                    },
                ),
            ),
            NodeFormattingHandler::with_close(
                NodeValueType::TableCell,
                Box::new(
                    |_value: &NodeValue,
                     ctx: &mut dyn NodeFormatterContext,
                     _writer: &mut MarkdownWriter| {
                        // Cell opening - if collecting table, skip rendering children
                        // They will be collected on close
                        if ctx.is_collecting_table() {
                            ctx.set_skip_children(true);
                        }
                    },
                ),
                Box::new(
                    |_value: &NodeValue,
                     ctx: &mut dyn NodeFormatterContext,
                     _writer: &mut MarkdownWriter| {
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
            ),
            // Footnote elements
            NodeFormattingHandler::new(
                NodeValueType::FootnoteReference,
                Box::new(
                    |value: &NodeValue,
                     _ctx: &mut dyn NodeFormatterContext,
                     writer: &mut MarkdownWriter| {
                        if let NodeValue::FootnoteReference(footnote) = value {
                            writer.append(format!("[^{}]", footnote.name));
                        }
                    },
                ),
            ),
            NodeFormattingHandler::new(
                NodeValueType::FootnoteDefinition,
                Box::new(
                    |value: &NodeValue,
                     _ctx: &mut dyn NodeFormatterContext,
                     writer: &mut MarkdownWriter| {
                        if let NodeValue::FootnoteDefinition(footnote) = value {
                            writer.append(format!("[^{}]:", footnote.name));
                        }
                    },
                ),
            ),
            // Task items
            NodeFormattingHandler::new(
                NodeValueType::TaskItem,
                Box::new(
                    |value: &NodeValue,
                     _ctx: &mut dyn NodeFormatterContext,
                     writer: &mut MarkdownWriter| {
                        if let NodeValue::TaskItem(task) = value {
                            if task.symbol.is_some() {
                                writer.append_raw("[x] ");
                            } else {
                                writer.append_raw("[ ] ");
                            }
                        }
                    },
                ),
            ),
            // Shortcode emoji
            NodeFormattingHandler::new(
                NodeValueType::ShortCode,
                Box::new(
                    |value: &NodeValue,
                     _ctx: &mut dyn NodeFormatterContext,
                     writer: &mut MarkdownWriter| {
                        if let NodeValue::ShortCode(shortcode) = value {
                            // Output the original shortcode format
                            writer.append(format!(":{}:", shortcode.code));
                        }
                    },
                ),
            ),
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

/// Render a code block with proper fencing
///
/// Determines the appropriate fence length to avoid conflicts with
/// backticks in the code content. Supports various formatting options
/// including custom fence length, space before info, and matching closing marker.
fn render_code_block(
    code_block: &crate::core::nodes::NodeCodeBlock,
    ctx: &dyn NodeFormatterContext,
    writer: &mut MarkdownWriter,
) {
    let options = ctx.get_formatter_options();

    if code_block.fenced {
        render_fenced_code_block(code_block, ctx, writer, options);
    } else {
        render_indented_code_block(code_block, ctx, writer, options);
    }
}

/// Render a fenced code block
fn render_fenced_code_block(
    code_block: &crate::core::nodes::NodeCodeBlock,
    _ctx: &dyn NodeFormatterContext,
    writer: &mut MarkdownWriter,
    options: &crate::options::format::FormatOptions,
) {
    // Determine the fence character and base length
    let fence_char = match options.fenced_code_marker_type {
        crate::options::format::CodeFenceMarker::Tilde => '~',
        _ => '`',
    };

    // Calculate the required fence length
    let base_length = options.fenced_code_marker_length.max(3);
    let fence_len = if fence_char == '`' {
        // For backticks, need to ensure fence doesn't appear in content
        compute_fence_length(&code_block.literal, base_length)
    } else {
        // For tildes, just use the base length
        base_length
    };

    let fence = fence_char.to_string().repeat(fence_len);
    writer.append(&fence);

    // Add info string on the same line as the opening fence
    if !code_block.info.is_empty() {
        let clean_info = code_block.info.trim_end_matches(fence_char);
        if !clean_info.is_empty() {
            // Add space before info if configured
            if options.fenced_code_space_before_info {
                writer.append(" ");
            }
            writer.append(clean_info);
        }
    }
    writer.line();

    // Process code content with optional indent minimization
    let code_content = if options.fenced_code_minimize_indent {
        minimize_indent(&code_block.literal)
    } else {
        code_block.literal.clone()
    };

    // Use split('\n') instead of lines() to preserve empty lines in code blocks
    let mut lines = code_content.split('\n').peekable();
    while let Some(line) = lines.next() {
        // For non-empty lines, use append_raw to preserve leading whitespace
        if !line.is_empty() {
            writer.append_raw(line);
        }
        // Add line break after every line, including the last one
        // This ensures the closing fence is on its own line
        if lines.peek().is_some() || !line.is_empty() {
            // Check if we need to force a newline for empty lines
            if line.is_empty() && lines.peek().is_some() {
                // Force output a newline by temporarily resetting beginning_of_line
                writer.force_newline();
            } else {
                writer.line();
            }
        }
    }

    // Add closing fence - match opening length if configured
    let closing_fence_len = if options.fenced_code_match_closing_marker {
        fence_len
    } else {
        base_length
    };
    writer.append(fence_char.to_string().repeat(closing_fence_len));
    writer.blank_line();
}

/// Render an indented code block
fn render_indented_code_block(
    code_block: &crate::core::nodes::NodeCodeBlock,
    _ctx: &dyn NodeFormatterContext,
    writer: &mut MarkdownWriter,
    options: &crate::options::format::FormatOptions,
) {
    // Process code content with optional indent minimization
    let code_content = if options.indented_code_minimize_indent {
        minimize_indent(&code_block.literal)
    } else {
        code_block.literal.clone()
    };

    // Use split('\n') instead of lines() to preserve empty lines in code blocks
    let mut lines = code_content.split('\n').peekable();
    while let Some(line) = lines.next() {
        // Add 4-space indent for indented code blocks
        if !line.is_empty() {
            writer.append_raw("    ");
            writer.append_raw(line);
        }
        // Add line break if this is not the last line
        if lines.peek().is_some() {
            if line.is_empty() {
                writer.force_newline();
            } else {
                writer.line();
            }
        }
    }
    writer.blank_line();
}

/// Minimize the indentation of code block content
///
/// This function finds the common leading whitespace across all non-empty lines
/// and removes it, reducing the overall indentation while preserving relative
/// indentation within the code block.
fn minimize_indent(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return content.to_string();
    }

    // Find the minimum indentation across all non-empty lines
    let mut min_indent: Option<usize> = None;

    for line in &lines {
        if line.is_empty() {
            continue;
        }

        let indent = line.len() - line.trim_start().len();
        min_indent = Some(match min_indent {
            Some(current) => current.min(indent),
            None => indent,
        });
    }

    // If no minimum indent found (all lines empty), return original
    let min_indent = match min_indent {
        Some(indent) => indent,
        None => return content.to_string(),
    };

    // Remove the common indentation from each line
    lines
        .iter()
        .map(|line| {
            if line.is_empty() {
                *line
            } else {
                &line[min_indent.min(line.len())..]
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Render an HTML block with proper formatting
///
/// This function handles HTML blocks according to flexmark-java's approach:
/// - In translation mode, wraps content with non-translating markers
/// - In normal mode, outputs the HTML content as-is
fn render_html_block(
    html: &crate::core::nodes::NodeHtmlBlock,
    _ctx: &dyn NodeFormatterContext,
    writer: &mut MarkdownWriter,
) {
    let content = &html.literal;

    // Check if this is a single-line HTML comment (like <!-- TOC -->)
    let is_single_line_comment = content.trim().starts_with("<!--")
        && content.trim().ends_with("-->")
        && !content.trim().contains('\n');

    if is_single_line_comment {
        // For single-line HTML comments, output without extra blank lines
        writer.append_raw(content.trim());
        writer.line();
    } else {
        // For multi-line HTML blocks, add blank lines before and after
        writer.blank_line();

        // Output the HTML content line by line
        // Use split('\n') instead of lines() to preserve empty lines
        for line in content.split('\n') {
            writer.append_raw(line);
            writer.line();
        }

        // Add trailing blank line after HTML block
        writer.tail_blank_line();
    }
}

/// Collect text content from a cell node and its children
///
/// This function recursively collects text from Text nodes and applies
/// appropriate formatting for inline elements, but avoids escaping pipe
/// characters which are used for table cell separation.
fn collect_cell_text_content(
    arena: &crate::core::arena::NodeArena,
    node_id: crate::core::arena::NodeId,
) -> String {
    use crate::core::nodes::NodeValue;

    let mut result = String::new();
    let node = arena.get(node_id);

    match &node.value {
        NodeValue::Text(text) => {
            // Escape markdown special chars but not pipe
            result.push_str(&escape_markdown_for_table_simple(text));
        }
        NodeValue::Code(code) => {
            // Inline code
            let backticks = get_backtick_sequence(&code.literal);
            result.push_str(&backticks);
            result.push_str(&code.literal);
            result.push_str(&backticks);
        }
        NodeValue::Emph => {
            result.push('*');
            // Recursively collect children
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                result.push_str(&collect_cell_text_content(arena, child_id));
                child_opt = arena.get(child_id).next;
            }
            result.push('*');
        }
        NodeValue::Strong => {
            result.push_str("**");
            // Recursively collect children
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                result.push_str(&collect_cell_text_content(arena, child_id));
                child_opt = arena.get(child_id).next;
            }
            result.push_str("**");
        }
        NodeValue::Strikethrough => {
            result.push_str("~~");
            // Recursively collect children
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                result.push_str(&collect_cell_text_content(arena, child_id));
                child_opt = arena.get(child_id).next;
            }
            result.push_str("~~");
        }
        NodeValue::Link(link) => {
            result.push('[');
            // Recursively collect children for link text
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                result.push_str(&collect_cell_text_content(arena, child_id));
                child_opt = arena.get(child_id).next;
            }
            result.push_str("](");
            result.push_str(&escape_url(&link.url));
            if !link.title.is_empty() {
                result.push_str(&format!(" \"{}\"", escape_string(&link.title)));
            }
            result.push(')');
        }
        NodeValue::SoftBreak => {
            result.push(' ');
        }
        NodeValue::HardBreak => {
            result.push_str("  ");
        }
        _ => {
            // For other node types, just recursively collect children
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                result.push_str(&collect_cell_text_content(arena, child_id));
                child_opt = arena.get(child_id).next;
            }
        }
    }

    result
}

/// Render a formatted table using table.rs
///
/// Takes collected row and cell data, builds table lines,
/// and uses table::format_table_lines for Unicode-aware formatting.
fn render_formatted_table(
    rows: &[Vec<String>],
    alignments: &[crate::core::nodes::TableAlignment],
    writer: &mut MarkdownWriter,
) {
    use crate::render::commonmark::table;

    // Filter out empty rows (e.g., header end markers)
    let rows: Vec<&Vec<String>> = rows.iter().filter(|row| !row.is_empty()).collect();

    if rows.is_empty() {
        return;
    }

    // Build table lines from collected data
    // format_table_lines expects: [header_row, delimiter_row, data_rows...]
    // It will skip the delimiter row at index 1 and generate its own
    let mut lines: Vec<String> = Vec::new();

    // Add header row (first row)
    if let Some(header_row) = rows.first() {
        let cells: Vec<String> = header_row.to_vec();
        let line = format!("| {} |", cells.join(" | "));
        lines.push(line);
    }

    // Add a placeholder delimiter row (will be skipped by format_table_lines)
    // We use a simple delimiter that matches the number of columns
    let num_cols = alignments.len().max(1);
    let placeholder_delimiter: Vec<String> =
        (0..num_cols).map(|_| "---".to_string()).collect();
    lines.push(format!("| {} |", placeholder_delimiter.join(" | ")));

    // Add data rows (remaining rows)
    for row in rows.iter().skip(1) {
        let cells: Vec<String> = row.to_vec();
        let line = format!("| {} |", cells.join(" | "));
        lines.push(line);
    }

    // Convert to &str slice for format_table_lines
    let line_refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();

    // Format the table using table.rs
    let formatted = table::format_table_lines(&line_refs, alignments);

    // Write the formatted table
    for line in formatted.lines() {
        writer.append(line);
        writer.line();
    }

    // Add blank line after table to separate from following content
    writer.blank_line();
}

/// Get the appropriate backtick sequence for inline code
///
/// Determines the minimum number of backticks needed to wrap the content
/// without conflicting with backticks inside the content.
///
/// # Examples
///
/// ```ignore
/// use clmd::render::commonmark::commonmark_formatter::get_backtick_sequence;
///
/// assert_eq!(get_backtick_sequence("code"), "`");
/// assert_eq!(get_backtick_sequence("code `with` backticks"), "``");
/// assert_eq!(get_backtick_sequence("``double``"), "```");
/// ```ignore
pub fn get_backtick_sequence(content: &str) -> String {
    let mut max_backticks = 0;
    let mut current = 0;

    for c in content.chars() {
        if c == '`' {
            current += 1;
            max_backticks = max_backticks.max(current);
        } else {
            current = 0;
        }
    }

    let count = (max_backticks + 1).max(1);
    "`".repeat(count)
}

/// Format list item marker with specific item number and options
///
/// This version respects the formatter options for marker style.
fn format_list_item_marker_with_number_and_options(
    list: &crate::core::nodes::NodeList,
    item_number: usize,
    options: &FormatOptions,
) -> String {
    use crate::core::nodes::{ListDelimType, ListType};
    use crate::options::format::{BulletMarker, NumberedMarker};

    match list.list_type {
        ListType::Bullet => {
            // Choose bullet character based on options
            let bullet_char = match options.list_bullet_marker {
                BulletMarker::Dash => '-',
                BulletMarker::Asterisk => '*',
                BulletMarker::Plus => '+',
                BulletMarker::Any => list.bullet_char as char,
            };
            format!("{} ", bullet_char)
        }
        ListType::Ordered => {
            // Choose delimiter based on options
            let delimiter = match options.list_numbered_marker {
                NumberedMarker::Period => '.',
                NumberedMarker::Paren => ')',
                NumberedMarker::Any => match list.delimiter {
                    ListDelimType::Period => '.',
                    ListDelimType::Paren => ')',
                },
            };
            let marker = format!("{}{}", item_number, delimiter);
            // Add a single space after the marker
            format!("{} ", marker)
        }
    }
}

/// Count the number of list ancestors for a given node
///
/// This is used to determine the nesting level of a list item.
/// Returns 0 for top-level items, 1 for items in nested lists, etc.
fn count_list_ancestors(
    arena: &crate::core::arena::NodeArena,
    list_node_id: crate::core::arena::NodeId,
) -> usize {
    use crate::core::nodes::NodeValue;

    let mut count: usize = 0;
    let mut current = list_node_id;

    // Count how many list ancestors this list has
    while let Some(parent_id) = arena.get(current).parent {
        let parent = arena.get(parent_id);
        if matches!(parent.value, NodeValue::List(..)) {
            count += 1;
        }
        current = parent_id;
    }

    count
}

/// Check if a task list item is checked
///
/// This function examines the content of a task list item to determine
/// if it starts with [x] or [X] (checked) or [ ] (unchecked).
fn is_task_item_checked(
    arena: &crate::core::arena::NodeArena,
    item_node_id: Option<crate::core::arena::NodeId>,
) -> bool {
    use crate::core::nodes::NodeValue;

    let item_id = match item_node_id {
        Some(id) => id,
        None => return false,
    };

    let item = arena.get(item_id);

    // Look for the first text node in the item's children
    let mut child_id = item.first_child;
    while let Some(child) = child_id {
        let child_node = arena.get(child);

        // Check if this is a paragraph
        if matches!(child_node.value, NodeValue::Paragraph) {
            // Look for text inside the paragraph
            let mut para_child_id = child_node.first_child;
            while let Some(para_child) = para_child_id {
                let para_child_node = arena.get(para_child);
                if let NodeValue::Text(text) = &para_child_node.value {
                    // Check if the text starts with [x] or [X]
                    let text_str = text.as_ref();
                    return text_str.starts_with("[x]") || text_str.starts_with("[X]");
                }
                para_child_id = para_child_node.next;
            }
        }

        child_id = child_node.next;
    }

    false
}

/// Check if the current context is inside a task list item
///
/// This checks if the current node's parent is a list item that is part of a task list.
fn is_in_task_list_item(ctx: &dyn NodeFormatterContext) -> bool {
    use crate::core::nodes::NodeValue;

    if let Some(current_node) = ctx.get_current_node() {
        let arena = ctx.get_arena();
        let node = arena.get(current_node);

        // Check if the current node is inside an Item
        if let Some(parent_id) = node.parent {
            let parent = arena.get(parent_id);

            // Check if parent is a Paragraph inside an Item
            if matches!(parent.value, NodeValue::Paragraph) {
                if let Some(grandparent_id) = parent.parent {
                    let grandparent = arena.get(grandparent_id);
                    if let NodeValue::Item(item_data) = &grandparent.value {
                        return item_data.is_task_list;
                    }
                }
            }

            // Or if parent is directly an Item
            if let NodeValue::Item(item_data) = &parent.value {
                return item_data.is_task_list;
            }
        }
    }

    false
}

/// Skip the task list marker from the beginning of text
///
/// If the text starts with "[ ] " or "[x] " (or "[X] "), remove it.
fn skip_task_marker(text: &str) -> String {
    if let Some(rest) = text.strip_prefix("[ ] ") {
        rest.to_string()
    } else if let Some(rest) = text.strip_prefix("[x] ") {
        rest.to_string()
    } else if let Some(rest) = text.strip_prefix("[X] ") {
        rest.to_string()
    } else {
        text.to_string()
    }
}

/// Calculate the prefixes for list item line breaking
///
/// Returns (first_line_prefix, continuation_prefix) where:
/// - first_line_prefix is empty (the list marker is already output by Item handler)
/// - continuation_prefix is the indentation to align with the list marker
fn calculate_list_item_prefixes(ctx: &dyn NodeFormatterContext) -> (String, String) {
    use crate::core::nodes::NodeValue;
    use crate::text::unicode_width;

    // Get the current node (Paragraph) and find its parent Item
    if let Some(current_node) = ctx.get_current_node() {
        let arena = ctx.get_arena();
        let node = arena.get(current_node);

        if let Some(parent_id) = node.parent {
            let parent = arena.get(parent_id);

            // Check if parent is an Item
            if let NodeValue::Item(_item_data) = &parent.value {
                // Get the grandparent (List) to determine the marker
                if let Some(grandparent_id) = parent.parent {
                    let grandparent = arena.get(grandparent_id);

                    if let NodeValue::List(list) = &grandparent.value {
                        // Calculate the item number for ordered lists
                        let item_number = get_item_number_in_list(
                            arena,
                            grandparent_id,
                            Some(parent_id),
                        );

                        // Get the list marker
                        let marker = format_list_item_marker_with_number_and_options(
                            list,
                            item_number,
                            ctx.get_formatter_options(),
                        );

                        // Calculate marker width
                        let marker_width = unicode_width::width(&marker) as usize;

                        // Calculate nesting level for additional indentation
                        let nesting_level = count_list_ancestors(arena, grandparent_id);
                        let indent_width = nesting_level * 4;

                        // First line prefix is empty (marker already output)
                        let first_prefix = String::new();

                        // Continuation prefix aligns with the content after the marker
                        let cont_prefix = " ".repeat(indent_width + marker_width);

                        return (first_prefix, cont_prefix);
                    }
                }
            }
        }
    }

    // Fallback: no special prefixes
    (String::new(), String::new())
}

/// Calculate the prefixes for block quote line breaking
///
/// Returns (first_line_prefix, continuation_prefix) where:
/// - first_line_prefix is empty (the block quote marker is already output by BlockQuote handler)
/// - continuation_prefix is the block quote marker for subsequent lines
fn calculate_block_quote_prefixes(ctx: &dyn NodeFormatterContext) -> (String, String) {
    let nesting_level = ctx.get_block_quote_nesting_level();

    // Build the continuation prefix: "> " repeated for each nesting level
    // This is needed for continuation lines
    let cont_prefix = "> ".repeat(nesting_level);

    // First line prefix is empty because BlockQuote handler already outputs the marker
    let first_prefix = String::new();

    (first_prefix, cont_prefix)
}

/// Calculate the content length of a heading for Setext underline
///
/// This function calculates the visible content length of a heading,
/// accounting for Unicode character widths and inline formatting.
fn calculate_heading_content_length(
    ctx: &mut dyn NodeFormatterContext,
    node_id: crate::core::arena::NodeId,
) -> usize {
    use crate::core::nodes::NodeValue;

    let arena = ctx.get_arena();
    let node = arena.get(node_id);

    let mut length = 0;

    // Recursively calculate content length from children
    let mut child_opt = node.first_child;
    while let Some(child_id) = child_opt {
        let child = arena.get(child_id);

        match &child.value {
            NodeValue::Text(text) => {
                // Use Unicode width for accurate character counting
                length += crate::text::unicode_width::width(text.as_ref()) as usize;
            }
            NodeValue::Code(code) => {
                // Code spans: count the literal content
                length += crate::text::unicode_width::width(&code.literal) as usize;
            }
            NodeValue::Emph | NodeValue::Strong => {
                // Recursively count children (emphasis markers don't add to visible length)
                length += calculate_child_content_length(arena, child_id);
            }
            NodeValue::Link(_link) => {
                // For links, count the link text (children), not the URL
                length += calculate_child_content_length(arena, child_id);
            }
            NodeValue::Image(_link) => {
                // For images, count the alt text (children)
                length += calculate_child_content_length(arena, child_id);
            }
            NodeValue::SoftBreak => {
                // Soft breaks in headings become spaces
                length += 1;
            }
            NodeValue::HardBreak => {
                // Hard breaks shouldn't normally appear in headings
                length += 1;
            }
            _ => {
                // For other nodes, recursively count children
                length += calculate_child_content_length(arena, child_id);
            }
        }

        child_opt = child.next;
    }

    length
}

/// Calculate content length from children recursively
fn calculate_child_content_length(
    arena: &crate::core::arena::NodeArena,
    node_id: crate::core::arena::NodeId,
) -> usize {
    use crate::core::nodes::NodeValue;

    let node = arena.get(node_id);
    let mut length = 0;

    let mut child_opt = node.first_child;
    while let Some(child_id) = child_opt {
        let child = arena.get(child_id);

        match &child.value {
            NodeValue::Text(text) => {
                length += crate::text::unicode_width::width(text.as_ref()) as usize;
            }
            NodeValue::Code(code) => {
                length += crate::text::unicode_width::width(&code.literal) as usize;
            }
            NodeValue::SoftBreak => {
                length += 1;
            }
            _ => {
                length += calculate_child_content_length(arena, child_id);
            }
        }

        child_opt = child.next;
    }

    length
}

/// Get the 1-based item number of a node within its parent list
///
/// This is used to determine the correct number for ordered list items.
fn get_item_number_in_list(
    arena: &crate::core::arena::NodeArena,
    list_node_id: crate::core::arena::NodeId,
    item_node_id: Option<crate::core::arena::NodeId>,
) -> usize {
    use crate::core::nodes::NodeValue;

    let item_id = match item_node_id {
        Some(id) => id,
        None => return 1, // Default to 1 if no item specified
    };

    let list = arena.get(list_node_id);
    let mut item_number: usize = 0;

    // Count how many Item siblings come before this item (including this item)
    if let Some(first_child) = list.first_child {
        let mut current = first_child;
        loop {
            if matches!(arena.get(current).value, NodeValue::Item(..)) {
                item_number += 1;
                if current == item_id {
                    break;
                }
            }
            if let Some(next) = arena.get(current).next {
                current = next;
            } else {
                break;
            }
        }
    }

    // If we didn't find the item, return 1 as default
    if item_number == 0 {
        1
    } else {
        item_number
    }
}

/// Check if a list item is empty (has no content or only whitespace)
///
/// This is used for the listRemoveEmptyItems option.
fn is_empty_list_item(
    arena: &crate::core::arena::NodeArena,
    item_node_id: crate::core::arena::NodeId,
) -> bool {
    use crate::core::nodes::NodeValue;

    let item = arena.get(item_node_id);

    // Check if the item has any children with content
    let mut child_id = item.first_child;
    while let Some(child) = child_id {
        let child_node = arena.get(child);

        match &child_node.value {
            NodeValue::Text(text) => {
                if !text.trim().is_empty() {
                    return false;
                }
            }
            NodeValue::Paragraph | NodeValue::Heading(_) => {
                // These containers might have content in their children
                if !is_empty_container(arena, child) {
                    return false;
                }
            }
            // Other node types are considered content
            _ => return false,
        }

        child_id = child_node.next;
    }

    // No content found
    true
}

/// Check if a container node is empty (has no meaningful content)
fn is_empty_container(
    arena: &crate::core::arena::NodeArena,
    node_id: crate::core::arena::NodeId,
) -> bool {
    use crate::core::nodes::NodeValue;

    let node = arena.get(node_id);

    let mut child_id = node.first_child;
    while let Some(child) = child_id {
        let child_node = arena.get(child);

        match &child_node.value {
            NodeValue::Text(text) => {
                if !text.trim().is_empty() {
                    return false;
                }
            }
            NodeValue::Paragraph | NodeValue::Heading(_) => {
                if !is_empty_container(arena, child) {
                    return false;
                }
            }
            _ => return false,
        }

        child_id = child_node.next;
    }

    true
}

/// Check if a list should be considered "loose" based on CommonMark spec
///
/// According to CommonMark, a list is loose if any of its constituent list
/// items are separated by blank lines, or if any of its constituent list
/// items directly contain two block-level elements with a blank line between them.
///
/// This function performs a more thorough check than just looking at the list's
/// tight flag, by analyzing the actual structure of the list items.
fn is_list_loose(
    arena: &crate::core::arena::NodeArena,
    list_node_id: crate::core::arena::NodeId,
) -> bool {
    use crate::core::nodes::NodeValue;

    let list = arena.get(list_node_id);

    // Iterate through all children (list items)
    let mut prev_item_had_blank_line = false;
    let mut child_id = list.first_child;

    while let Some(item_id) = child_id {
        let item = arena.get(item_id);

        if matches!(item.value, NodeValue::Item(..)) {
            // Check if this item contains blank lines
            if item_contains_blank_lines(arena, item_id) {
                return true;
            }

            // Check if there's a blank line between this item and the previous one
            if prev_item_had_blank_line {
                return true;
            }

            // Check for blank line after this item
            prev_item_had_blank_line = has_trailing_blank_line(arena, item_id);
        }

        child_id = item.next;
    }

    false
}

/// Check if a list item contains blank lines between its block-level children
///
/// This is part of the CommonMark definition of a loose list.
/// In this implementation, we check if the item has multiple block-level
/// children, which indicates a loose list structure.
fn item_contains_blank_lines(
    arena: &crate::core::arena::NodeArena,
    item_node_id: crate::core::arena::NodeId,
) -> bool {
    use crate::core::nodes::NodeValue;

    let item = arena.get(item_node_id);

    // Count block-level children, but treat nested lists specially
    // A list item is loose only if it contains multiple non-list block-level children
    // or if there are blank lines between elements
    let mut non_list_block_count = 0;
    let mut child_id = item.first_child;

    while let Some(child) = child_id {
        let child_node = arena.get(child);

        // Check if this child is a block-level element
        match &child_node.value {
            NodeValue::List(_) => {
                // Nested lists are treated as part of the list item content
                // They don't make the list loose on their own
            }
            NodeValue::Paragraph
            | NodeValue::Heading(_)
            | NodeValue::BlockQuote
            | NodeValue::CodeBlock(_)
            | NodeValue::HtmlBlock(_) => {
                non_list_block_count += 1;
                // If we have more than one non-list block-level child, it's loose
                if non_list_block_count > 1 {
                    return true;
                }
            }
            _ => {}
        }

        child_id = child_node.next;
    }

    // A list item with one non-list block and a nested list is not loose
    // (e.g., a paragraph followed by a sublist is still tight)
    false
}

/// Check if a node has trailing blank lines
///
/// This checks if there's a blank line after this node in the document.
/// In this implementation, we check if the next sibling is an empty paragraph
/// or if there's no next sibling (end of container).
fn has_trailing_blank_line(
    _arena: &crate::core::arena::NodeArena,
    node_id: crate::core::arena::NodeId,
) -> bool {
    // For now, we use a simplified check
    // In a full implementation, this would analyze the source for blank lines
    // The tight flag in the list itself is the primary indicator
    let _ = node_id;
    false
}

/// Calculate the effective tightness of a list based on options and content
///
/// This function determines whether a list should be rendered as tight or loose
/// based on the formatter options and the actual list content.
fn calculate_effective_list_tightness(
    arena: &crate::core::arena::NodeArena,
    list_node_id: crate::core::arena::NodeId,
    _list: &crate::core::nodes::NodeList,
    options: &crate::options::format::FormatOptions,
) -> bool {
    use crate::options::format::ListSpacing;

    match options.list_spacing {
        ListSpacing::Tight => true,
        ListSpacing::Loose => false,
        ListSpacing::AsIs => {
            // Use the list's own tight flag, but verify it matches the content
            let content_is_loose = is_list_loose(arena, list_node_id);
            !content_is_loose
        }
        ListSpacing::Loosen => {
            // Loosen if content indicates loose list
            let content_is_loose = is_list_loose(arena, list_node_id);
            !content_is_loose // Return tight=true if content is not loose
        }
        ListSpacing::Tighten => {
            // Always tighten
            true
        }
    }
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
fn should_render_loose_paragraph(
    ctx: &dyn NodeFormatterContext,
    is_last_paragraph: bool,
) -> bool {
    let is_in_list_item = ctx.is_parent_list_item();
    let is_in_tight_list = ctx.is_in_tight_list();
    let has_next_sibling = ctx.has_next_sibling();

    if is_in_list_item {
        // Paragraph is inside a list item
        if is_in_tight_list {
            // Tight list: never use blank lines
            false
        } else {
            // Loose list: use blank line if not the last paragraph and has siblings
            !is_last_paragraph && has_next_sibling
        }
    } else if is_in_tight_list {
        // Paragraph in tight list context but not directly in item
        false
    } else {
        // Normal paragraph outside lists
        // Use blank line if not the last paragraph
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

/// Check if a URL is a reference-style link label
///
/// Reference-style links use a label instead of a direct URL.
/// The label is case-insensitive and can contain letters, numbers, spaces, and punctuation.
fn is_reference_label(url: &str) -> bool {
    // A reference label is not a URL if:
    // 1. It doesn't contain :// (protocol separator)
    // 2. It doesn't start with /, ./, or ../
    // 3. It doesn't start with # (fragment) or ? (query)
    // 4. It doesn't look like an absolute URL (contains :// or starts with common schemes)

    // If it contains ://, it's definitely a URL
    if url.contains("://") {
        return false;
    }

    // If it starts with /, ./, or ../, it's a path (not a reference label)
    if url.starts_with('/') || url.starts_with("./") || url.starts_with("../") {
        return false;
    }

    // If it starts with # or ?, it's a fragment or query (not a reference label)
    if url.starts_with('#') || url.starts_with('?') {
        return false;
    }

    // If it's empty, it's not a reference label
    if url.is_empty() {
        return false;
    }

    // Check for common URL schemes at the start followed by :
    // This catches cases like "http:" without //
    let schemes = [
        "http:",
        "https:",
        "ftp:",
        "mailto:",
        "file:",
        "data:",
        "javascript:",
        "vbscript:",
    ];
    for scheme in &schemes {
        if url.starts_with(scheme) {
            return false;
        }
    }

    // Check if it looks like a domain (contains . and no spaces)
    // e.g., "example.com" should be treated as a URL, not a reference label
    if url.contains('.')
        && !url.contains(' ')
        && !url.contains('[')
        && !url.contains(']')
    {
        // Check if it looks like a domain name (has a dot and valid domain characters)
        let domain_part: String = url
            .chars()
            .take_while(|&c| c != '/' && c != '?' && c != '#')
            .collect();
        if domain_part.contains('.')
            && domain_part
                .chars()
                .all(|c| c.is_alphanumeric() || c == '.' || c == '-' || c == '_')
        {
            return false;
        }
    }

    // If we get here, it's likely a reference label
    true
}

/// Render a link URL, handling reference-style links
///
/// This function determines whether a link is inline (with direct URL)
/// or reference-style (with label), and renders accordingly.
fn render_link_url(
    url: &str,
    title: &str,
    _ctx: &dyn NodeFormatterContext,
    writer: &mut MarkdownWriter,
) {
    if is_reference_label(url) {
        // Reference-style link: [text][label] or [text][]
        // The label is already in the URL field
        writer.append("[");
        writer.append(url);
        writer.append("]");
    } else {
        // Inline link: [text](url "title")
        writer.append("(");
        writer.append(escape_url(url));
        if !title.is_empty() {
            writer.append(format!(" \"{}\"", escape_string(title)));
        }
        writer.append(")");
    }
}

/// Render an image URL, handling reference-style images
///
/// Similar to render_link_url but for images.
fn render_image_url(
    url: &str,
    title: &str,
    _ctx: &dyn NodeFormatterContext,
    writer: &mut MarkdownWriter,
) {
    if is_reference_label(url) {
        // Reference-style image: ![alt][label] or ![alt][]
        writer.append("[");
        writer.append(url);
        writer.append("]");
    } else {
        // Inline image: ![alt](url "title")
        writer.append("(");
        writer.append(escape_url(url));
        if !title.is_empty() {
            writer.append(format!(" \"{}\"", escape_string(title)));
        }
        writer.append(")");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::commonmark::escaping::escape_text;

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

        fn start_line_breaking(&mut self, _ideal_width: usize, _max_width: usize) {}

        fn start_line_breaking_with_prefixes(
            &mut self,
            _ideal_width: usize,
            _max_width: usize,
            _first_line_prefix: String,
            _continuation_prefix: String,
        ) {
        }

        fn add_line_breaking_word(
            &mut self,
            _word: crate::render::commonmark::line_breaking::Word,
        ) {
        }

        fn add_line_breaking_text(&mut self, _text: &str) {}

        fn add_line_breaking_word_text(&mut self, _text: &str) {}

        fn add_line_breaking_marker_end(&mut self, _text: &str) {}

        fn add_line_breaking_inline_element(&mut self, _text: &str) {}

        fn add_line_breaking_link_close(&mut self, _text: &str) {}

        fn add_line_breaking_url(&mut self, _text: &str) {}

        fn finish_line_breaking(&mut self) -> Option<String> {
            None
        }

        fn is_collecting_line_breaking(&self) -> bool {
            false
        }

        fn get_line_breaking_context(
            &self,
        ) -> Option<&crate::render::commonmark::line_breaking::LineBreakingContext>
        {
            None
        }

        fn reset_line_breaking_space(&mut self) {}

        fn enter_link_text(&mut self) {}

        fn exit_link_text(&mut self) {}

        fn exit_link_url(&mut self) {}

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

        fn is_paragraph_line_breaking(&self) -> bool {
            false
        }

        fn remove_paragraph_trailing_space(&mut self) {}
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
    fn test_get_backtick_sequence() {
        assert_eq!(get_backtick_sequence("code"), "`");
        assert_eq!(get_backtick_sequence("code `with` backticks"), "``");
        assert_eq!(get_backtick_sequence("``double``"), "```");
        assert_eq!(get_backtick_sequence("```triple```"), "````");
    }

    #[test]
    fn test_get_backtick_sequence_empty() {
        assert_eq!(get_backtick_sequence(""), "`");
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
    fn test_count_list_ancestors() {
        use crate::core::arena::{Node, NodeArena, TreeOps};
        use crate::core::nodes::{ListDelimType, ListType, NodeList, NodeValue};

        let mut arena = NodeArena::new();

        // Create a nested list structure:
        // Document
        // └── List (outer)
        //     ├── Item 1
        //     └── Item 2
        //         └── List (inner)
        //             └── Item 3

        let root = arena.alloc(Node::with_value(NodeValue::Document));

        let outer_list = arena.alloc(Node::with_value(NodeValue::List(NodeList {
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

        let inner_list = arena.alloc(Node::with_value(NodeValue::List(NodeList {
            list_type: ListType::Bullet,
            marker_offset: 0,
            padding: 0,
            start: 1,
            delimiter: ListDelimType::Period,
            bullet_char: b'-',
            tight: true,
            is_task_list: false,
        })));

        let item3 = arena.alloc(Node::with_value(NodeValue::Item(NodeList {
            list_type: ListType::Bullet,
            marker_offset: 0,
            padding: 0,
            start: 0,
            delimiter: ListDelimType::Period,
            bullet_char: 0,
            tight: true,
            is_task_list: false,
        })));

        TreeOps::append_child(&mut arena, root, outer_list);
        TreeOps::append_child(&mut arena, outer_list, item1);
        TreeOps::append_child(&mut arena, outer_list, item2);
        TreeOps::append_child(&mut arena, item2, inner_list);
        TreeOps::append_child(&mut arena, inner_list, item3);

        // outer_list has 0 list ancestors (it's top-level)
        assert_eq!(count_list_ancestors(&arena, outer_list), 0);

        // inner_list has 1 list ancestor (outer_list)
        assert_eq!(count_list_ancestors(&arena, inner_list), 1);
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
}
