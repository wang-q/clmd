//! Handler registration for CommonMark formatter
//!
//! This module organizes node handler registration by functional domain.
//! Each registration function corresponds to a group of related node types:
//!
//! - **Block handlers**: Document, Paragraph, Heading, BlockQuote, CodeBlock, ThematicBreak, HtmlBlock
//! - **Inline handlers**: Text, Code, Emph, Strong, Link, Image, Strikethrough, SoftBreak, HardBreak, HtmlInline
//! - **List handlers**: List, Item (with task list support)
//! - **Table handlers**: Table, TableRow, TableCell
//! - **Extension handlers**: FootnoteReference, FootnoteDefinition, TaskItem, ShortCode
//!
//! # Usage
//!
//! ```ignore
//! use crate::render::commonmark::handlers::registration::register_all_handlers;
//!
//! let handlers = register_all_handlers();
//! ```

use crate::core::nodes::NodeValue;
use crate::options::format::ListSpacing;
use crate::render::commonmark::core::NodeFormattingHandler;
use crate::render::commonmark::escaping::{compute_fence_length, escape_text};
use crate::render::commonmark::handler_utils::{
    check_nested_emphasis_conflict, preprocess_text,
};
use crate::render::commonmark::handlers::block::{render_code_block, render_html_block};
use crate::render::commonmark::handlers::container::{
    render_block_quote_close, render_block_quote_open, render_heading_close,
    render_heading_open, render_paragraph_close, render_paragraph_open,
};
use crate::render::commonmark::handlers::inline::{render_image_url, render_link_url};
use crate::render::commonmark::handlers::list::{
    calculate_effective_list_tightness, count_list_ancestors,
    format_list_item_marker_with_number_and_options, get_item_number_in_list,
    is_empty_list_item, is_task_item_checked,
};
use crate::render::commonmark::handlers::table::{
    collect_cell_text_content, render_formatted_table,
};
use crate::render::commonmark::line_breaking::AtomicKind;
use crate::render::commonmark::writer::MarkdownWriter;

/// Register all block-level element handlers
///
/// Includes: Document, Paragraph, Heading, BlockQuote, CodeBlock, ThematicBreak, HtmlBlock
pub fn register_block_handlers() -> Vec<NodeFormattingHandler> {
    vec![
        // Document - simple handler with no-op
        NodeFormattingHandler::new(
            std::mem::discriminant(&NodeValue::Document),
            |_value, _ctx, _writer| {
                // Document is handled at the top level
            },
        ),
        // Paragraph - handler with close
        NodeFormattingHandler::with_close(
            std::mem::discriminant(&NodeValue::Paragraph),
            render_paragraph_open,
            render_paragraph_close,
        ),
        // Heading - handler with close
        NodeFormattingHandler::with_close(
            std::mem::discriminant(&NodeValue::Heading(
                crate::core::nodes::NodeHeading::default(),
            )),
            render_heading_open,
            render_heading_close,
        ),
        // BlockQuote - handler with close
        NodeFormattingHandler::with_close(
            std::mem::discriminant(&NodeValue::BlockQuote),
            render_block_quote_open,
            render_block_quote_close,
        ),
        // CodeBlock - simple handler
        NodeFormattingHandler::new(
            std::mem::discriminant(&NodeValue::CodeBlock(Box::default())),
            |value, ctx, writer| {
                if let NodeValue::CodeBlock(code_block) = value {
                    render_code_block(code_block, ctx, writer);
                }
            },
        ),
        // ThematicBreak - simple handler
        NodeFormattingHandler::new(
            std::mem::discriminant(&NodeValue::ThematicBreak(
                crate::core::nodes::NodeThematicBreak::default(),
            )),
            |value, _ctx, writer| {
                if let NodeValue::ThematicBreak(tb) = value {
                    writer.append(tb.marker.to_string().repeat(3));
                } else {
                    writer.append("---");
                }
                writer.line();
            },
        ),
        // HtmlBlock - simple handler
        NodeFormattingHandler::new(
            std::mem::discriminant(&NodeValue::HtmlBlock(Box::default())),
            |value, ctx, writer| {
                if let NodeValue::HtmlBlock(html) = value {
                    render_html_block(html, ctx, writer);
                }
            },
        ),
    ]
}

/// Register all inline element handlers
///
/// Includes: Text, Code, Emph, Strong, Link, Image, Strikethrough, SoftBreak, HardBreak, HtmlInline
pub fn register_inline_handlers() -> Vec<NodeFormattingHandler> {
    vec![
        // Text - simple handler
        NodeFormattingHandler::new(
            std::mem::discriminant(&NodeValue::Text(Box::default())),
            |value, ctx, writer| {
                if let NodeValue::Text(text) = value {
                    let final_text = preprocess_text(text.as_ref(), ctx);

                    if ctx.is_paragraph_line_breaking() {
                        ctx.add_paragraph_text(&final_text);
                    } else {
                        let escaped = escape_text(&final_text, ctx);
                        writer.append_with_wrap(&escaped);
                    }
                }
            },
        ),
        // Code - simple handler
        NodeFormattingHandler::new(
            std::mem::discriminant(&NodeValue::Code(Box::default())),
            |value, ctx, writer| {
                if let NodeValue::Code(code) = value {
                    let fence_len = compute_fence_length(&code.literal, 1);
                    let backticks = "`".repeat(fence_len);

                    let needs_leading_space =
                        code.literal.starts_with('`') || code.literal.starts_with(' ');
                    let needs_trailing_space =
                        code.literal.ends_with('`') || code.literal.ends_with(' ');

                    if ctx.is_paragraph_line_breaking() {
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
                            AtomicKind::Code,
                            &prefix,
                            &content,
                            &suffix,
                        );
                    } else {
                        writer.append(&backticks);

                        if needs_leading_space {
                            writer.append(" ");
                        }

                        writer.append_raw(&code.literal);

                        if needs_trailing_space {
                            writer.append(" ");
                        }

                        writer.append(&backticks);
                    }
                }
            },
        ),
        // Emph - handler with close
        NodeFormattingHandler::with_close(
            std::mem::discriminant(&NodeValue::Emph),
            |_value, ctx, _writer| {
                if ctx.is_paragraph_line_breaking() {
                    let (is_nested, m) = check_nested_emphasis_conflict(
                        ctx,
                        std::mem::discriminant(&NodeValue::Strong),
                        "*",
                    );
                    if is_nested {
                        ctx.add_paragraph_word(&m);
                    } else {
                        ctx.set_skip_children(true);
                    }
                } else {
                    _writer.append("*");
                }
            },
            |_value, ctx, writer| {
                if ctx.is_paragraph_line_breaking() {
                    let (is_nested, m) = check_nested_emphasis_conflict(
                        ctx,
                        std::mem::discriminant(&NodeValue::Strong),
                        "*",
                    );
                    if is_nested {
                        ctx.add_paragraph_word(&m);
                    } else if let Some(node_id) = ctx.get_current_node() {
                        let content = ctx.render_children_to_string(node_id);
                        let full_emph = format!("*{}*", content);
                        ctx.add_paragraph_unbreakable_unit(
                            AtomicKind::Emph,
                            "",
                            &full_emph,
                            "",
                        );
                        ctx.set_skip_children(false);
                    }
                } else {
                    writer.append("*");
                }
            },
        ),
        // Strong - handler with close
        NodeFormattingHandler::with_close(
            std::mem::discriminant(&NodeValue::Strong),
            |_value, ctx, _writer| {
                if ctx.is_paragraph_line_breaking() {
                    let (is_nested, m) = check_nested_emphasis_conflict(
                        ctx,
                        std::mem::discriminant(&NodeValue::Emph),
                        "**",
                    );
                    if is_nested {
                        ctx.add_paragraph_word(&m);
                    } else {
                        ctx.set_skip_children(true);
                    }
                } else {
                    _writer.flush_word_wrap_buffer();
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
                    let (is_nested, m) = check_nested_emphasis_conflict(
                        ctx,
                        std::mem::discriminant(&NodeValue::Emph),
                        "**",
                    );
                    if is_nested {
                        ctx.add_paragraph_word(&m);
                    } else if let Some(node_id) = ctx.get_current_node() {
                        let content = ctx.render_children_to_string(node_id);
                        let full_strong = format!("**{}**", content);
                        ctx.add_paragraph_unbreakable_unit(
                            AtomicKind::Strong,
                            "",
                            &full_strong,
                            "",
                        );
                        ctx.set_skip_children(false);
                    }
                } else {
                    writer.append("**");
                }
            },
        ),
        // Link - handler with close
        NodeFormattingHandler::with_close(
            std::mem::discriminant(&NodeValue::Link(Box::default())),
            |_value: &NodeValue,
             ctx: &mut dyn crate::render::commonmark::core::NodeFormatterContext,
             _writer: &mut MarkdownWriter| {
                if ctx.is_paragraph_line_breaking() {
                    ctx.set_skip_children(true);
                } else {
                    _writer.append("[");
                }
            },
            |value: &NodeValue,
             ctx: &mut dyn crate::render::commonmark::core::NodeFormatterContext,
             writer: &mut MarkdownWriter| {
                if let NodeValue::Link(link) = value {
                    if ctx.is_paragraph_line_breaking() {
                        if let Some(node_id) = ctx.get_current_node() {
                            let link_text = ctx.render_children_to_string(node_id);
                            let full_link = if link.title.is_empty() {
                                format!("[{}]({})", link_text, link.url)
                            } else {
                                format!(
                                    "[{}]({} \"{}\")",
                                    link_text, link.url, link.title
                                )
                            };
                            ctx.add_paragraph_unbreakable_unit(
                                AtomicKind::Link,
                                "",
                                &full_link,
                                "",
                            );
                        }
                        ctx.set_skip_children(false);
                    } else {
                        writer.append("]");
                        render_link_url(&link.url, &link.title, ctx, writer);
                    }
                }
            },
        ),
        // Image - handler with close
        NodeFormattingHandler::with_close(
            std::mem::discriminant(&NodeValue::Image(Box::default())),
            |_value: &NodeValue,
             ctx: &mut dyn crate::render::commonmark::core::NodeFormatterContext,
             _writer: &mut MarkdownWriter| {
                if ctx.is_paragraph_line_breaking() {
                    ctx.set_skip_children(true);
                } else {
                    _writer.append("![");
                }
            },
            |value: &NodeValue,
             ctx: &mut dyn crate::render::commonmark::core::NodeFormatterContext,
             writer: &mut MarkdownWriter| {
                if let NodeValue::Image(link) = value {
                    if ctx.is_paragraph_line_breaking() {
                        if let Some(node_id) = ctx.get_current_node() {
                            let alt_text = ctx.render_children_to_string(node_id);
                            let full_image = if link.title.is_empty() {
                                format!("![{}]({})", alt_text, link.url)
                            } else {
                                format!(
                                    "![{}]({} \"{}\")",
                                    alt_text, link.url, link.title
                                )
                            };
                            ctx.add_paragraph_word(&full_image);
                        }
                        ctx.set_skip_children(false);
                    } else {
                        writer.append("]");
                        render_image_url(&link.url, &link.title, ctx, writer);
                    }
                }
            },
        ),
        // Strikethrough - handler with close
        NodeFormattingHandler::with_close(
            std::mem::discriminant(&NodeValue::Strikethrough),
            |_value, _ctx, writer| {
                writer.append("~~");
            },
            |_value, _ctx, writer| {
                writer.append("~~");
            },
        ),
        // SoftBreak - simple handler
        NodeFormattingHandler::new(
            std::mem::discriminant(&NodeValue::SoftBreak),
            |_value, ctx, writer| {
                let options = ctx.get_formatter_options();

                if ctx.is_paragraph_line_breaking() {
                    ctx.add_paragraph_text(" ");
                } else if ctx.is_in_tight_list() {
                    writer.append(" ");
                } else if options.keep_soft_line_breaks {
                    writer.line();
                } else {
                    let right_margin = options.right_margin;
                    if right_margin > 0 {
                        writer.append(" ");
                    } else {
                        writer.line();
                    }
                }
            },
        ),
        // HardBreak - simple handler
        NodeFormattingHandler::new(
            std::mem::discriminant(&NodeValue::HardBreak),
            |_value, ctx, writer| {
                let options = ctx.get_formatter_options();

                if ctx.is_paragraph_line_breaking() {
                    ctx.add_paragraph_hard_break();
                } else if options.keep_hard_line_breaks {
                    writer.append("\\");
                    writer.line();
                } else {
                    writer.append("  ");
                    writer.line();
                }
            },
        ),
        // HtmlInline - simple handler
        NodeFormattingHandler::new(
            std::mem::discriminant(&NodeValue::HtmlInline(Box::default())),
            |value, _ctx, writer| {
                if let NodeValue::HtmlInline(html) = value {
                    writer.append(html);
                }
            },
        ),
    ]
}

/// Register all list handlers
///
/// Includes: List (ordered/unordered), Item (with task list support)
pub fn register_list_handlers() -> Vec<NodeFormattingHandler> {
    vec![
        // List - handler with close
        NodeFormattingHandler::with_close(
            std::mem::discriminant(&NodeValue::List(
                crate::core::nodes::NodeList::default(),
            )),
            |value, ctx, _writer| {
                if let NodeValue::List(list) = value {
                    let effective_tight = if let Some(node_id) = ctx.get_current_node() {
                        calculate_effective_list_tightness(
                            ctx.get_arena(),
                            node_id,
                            list,
                            ctx.get_formatter_options(),
                        )
                    } else {
                        match ctx.get_formatter_options().list_spacing {
                            ListSpacing::Tight => true,
                            ListSpacing::Loose => false,
                            ListSpacing::AsIs => list.tight,
                            ListSpacing::Loosen => list.tight,
                            ListSpacing::Tighten => true,
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
                    writer.blank_line();
                }
            },
        ),
        // Item - handler with close
        NodeFormattingHandler::with_close(
            std::mem::discriminant(&NodeValue::Item(
                crate::core::nodes::NodeList::default(),
            )),
            |value, ctx, writer| {
                let options = ctx.get_formatter_options();

                if options.list_remove_empty_items {
                    if let Some(node_id) = ctx.get_current_node() {
                        if is_empty_list_item(ctx.get_arena(), node_id) {
                            return;
                        }
                    }
                }

                let (marker, nesting_level) =
                    if let Some(parent_id) = ctx.get_current_node_parent() {
                        let arena = ctx.get_arena();
                        let parent = arena.get(parent_id);
                        if let NodeValue::List(list) = &parent.value {
                            let level = count_list_ancestors(arena, parent_id);
                            let item_number = get_item_number_in_list(
                                arena,
                                parent_id,
                                ctx.get_current_node(),
                            );

                            let effective_number = if options.list_renumber_items {
                                item_number
                            } else {
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

                let is_task_list = if let NodeValue::Item(item_data) = value {
                    item_data.is_task_list
                } else {
                    false
                };

                let total_indent = if nesting_level == 0 {
                    0
                } else {
                    nesting_level * 4
                };

                let indent_str = " ".repeat(total_indent);

                writer.append_raw(&indent_str);
                writer.append_raw(&marker);

                if is_task_list {
                    let task_marker =
                        if is_task_item_checked(ctx.get_arena(), ctx.get_current_node())
                        {
                            "[x] "
                        } else {
                            "[ ] "
                        };
                    writer.append_raw(task_marker);
                }
            },
            |_value, ctx, writer| {
                let _options = ctx.get_formatter_options();

                let is_last_item = ctx
                    .get_current_node()
                    .map(|id| {
                        let arena = ctx.get_arena();
                        arena.get(id).next.is_none()
                    })
                    .unwrap_or(true);

                if ctx.is_in_tight_list() {
                    if !is_last_item {
                        writer.line();
                    }
                } else if !is_last_item {
                    writer.blank_line();
                } else {
                    writer.line();
                }
            },
        ),
    ]
}

/// Register all table handlers
///
/// Includes: Table, TableRow, TableCell
pub fn register_table_handlers() -> Vec<NodeFormattingHandler> {
    vec![
        // Table - handler with close
        NodeFormattingHandler::with_close(
            std::mem::discriminant(&NodeValue::Table(Box::default())),
            |value, ctx, _writer| {
                if let NodeValue::Table(table) = value {
                    ctx.start_table_collection(table.alignments.clone());
                }
            },
            |_value, ctx, writer| {
                if let Some((rows, alignments)) = ctx.take_table_data() {
                    let padding = ctx.get_formatter_options().table_delimiter_padding;
                    render_formatted_table(&rows, &alignments, writer, padding);
                }
            },
        ),
        // TableRow - handler with close
        NodeFormattingHandler::with_close(
            std::mem::discriminant(&NodeValue::TableRow(false)),
            |_value, ctx, _writer| {
                ctx.add_table_row();
            },
            |_value, _ctx, _writer| {},
        ),
        // TableCell - handler with close
        NodeFormattingHandler::with_close(
            std::mem::discriminant(&NodeValue::TableCell),
            |_value, ctx, _writer| {
                if ctx.is_collecting_table() {
                    ctx.set_skip_children(true);
                }
            },
            |_value, ctx, _writer| {
                if ctx.is_collecting_table() {
                    if let Some(node_id) = ctx.get_current_node() {
                        let content =
                            collect_cell_text_content(ctx.get_arena(), node_id);
                        ctx.add_table_cell(content);
                    }
                }
            },
        ),
    ]
}

/// Register all extension handlers (GFM features and custom elements)
///
/// Includes: FootnoteReference, FootnoteDefinition, TaskItem, ShortCode
pub fn register_extension_handlers() -> Vec<NodeFormattingHandler> {
    vec![
        // FootnoteReference - simple handler
        NodeFormattingHandler::new(
            std::mem::discriminant(&NodeValue::FootnoteReference(Box::default())),
            |value, _ctx, writer| {
                if let NodeValue::FootnoteReference(footnote) = value {
                    writer.append(format!("[^{}]", footnote.name));
                }
            },
        ),
        // FootnoteDefinition - simple handler
        NodeFormattingHandler::new(
            std::mem::discriminant(&NodeValue::FootnoteDefinition(Box::default())),
            |value, _ctx, writer| {
                if let NodeValue::FootnoteDefinition(footnote) = value {
                    writer.append(format!("[^{}]:", footnote.name));
                }
            },
        ),
        // TaskItem - simple handler
        NodeFormattingHandler::new(
            std::mem::discriminant(&NodeValue::TaskItem(
                crate::core::nodes::NodeTaskItem::default(),
            )),
            |value, _ctx, writer| {
                if let NodeValue::TaskItem(task) = value {
                    if task.symbol.is_some() {
                        writer.append_raw("[x] ");
                    } else {
                        writer.append_raw("[ ] ");
                    }
                }
            },
        ),
        // ShortCode emoji - simple handler
        NodeFormattingHandler::new(
            std::mem::discriminant(&NodeValue::ShortCode(Box::default())),
            |value, _ctx, writer| {
                if let NodeValue::ShortCode(shortcode) = value {
                    writer.append(format!(":{}:", shortcode.code));
                }
            },
        ),
    ]
}

/// Register all node formatting handlers
///
/// Combines all handler groups into a single vector for the formatter.
/// This is the main entry point for handler registration.
///
/// # Returns
///
/// A vector of `NodeFormattingHandler` instances covering all supported node types.
///
/// # Example
///
/// ```ignore
/// use crate::render::commonmark::handlers::registration::register_all_handlers;
///
/// let handlers = register_all_handlers();
/// assert_eq!(handlers.len(), 26); // Total number of registered handlers
/// ```
pub fn register_all_handlers() -> Vec<NodeFormattingHandler> {
    let mut handlers = Vec::new();
    handlers.extend(register_block_handlers());
    handlers.extend(register_inline_handlers());
    handlers.extend(register_list_handlers());
    handlers.extend(register_table_handlers());
    handlers.extend(register_extension_handlers());
    handlers
}
