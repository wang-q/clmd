//! Block element handlers for CommonMark formatting
//!
//! This module contains handlers for block-level elements like paragraphs,
//! headings, block quotes, code blocks, and thematic breaks.

use crate::core::arena::NodeId;
use crate::core::nodes::{NodeCodeBlock, NodeHtmlBlock};
use crate::core::traverse::TraverseExt;
use crate::options::format::FormatOptions;
use crate::render::commonmark::core::NodeFormatterContext;
use crate::render::commonmark::escaping::compute_fence_length;
use crate::render::commonmark::handler_utils::{INDENTED_CODE_SPACES, MIN_FENCE_LENGTH};
use crate::render::commonmark::writer::MarkdownWriter;

/// Render a code block (fenced or indented)
pub fn render_code_block(
    code_block: &NodeCodeBlock,
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
pub fn render_fenced_code_block(
    code_block: &NodeCodeBlock,
    _ctx: &dyn NodeFormatterContext,
    writer: &mut MarkdownWriter,
    options: &FormatOptions,
) {
    use crate::options::format::CodeFenceMarker;

    let fence_char = match options.fenced_code_marker_type {
        CodeFenceMarker::Tilde => '~',
        _ => '`',
    };

    let base_length = options.fenced_code_marker_length.max(MIN_FENCE_LENGTH);
    let fence_len = if fence_char == '`' {
        compute_fence_length(&code_block.literal, base_length)
    } else {
        base_length
    };

    let fence = fence_char.to_string().repeat(fence_len);
    writer.append(&fence);

    if !code_block.info.is_empty() {
        let clean_info = code_block.info.trim_end_matches(fence_char);
        if !clean_info.is_empty() {
            if options.fenced_code_space_before_info {
                writer.append(" ");
            }
            writer.append(clean_info);
        }
    }
    writer.line();

    let code_content = if options.fenced_code_minimize_indent {
        minimize_indent(&code_block.literal)
    } else {
        code_block.literal.clone()
    };

    let mut lines = code_content.split('\n').peekable();
    while let Some(line) = lines.next() {
        if !line.is_empty() {
            writer.append_raw(line);
        }
        if lines.peek().is_some() || !line.is_empty() {
            if line.is_empty() && lines.peek().is_some() {
                writer.force_newline();
            } else {
                writer.line();
            }
        }
    }

    let closing_fence_len = if options.fenced_code_match_closing_marker {
        fence_len
    } else {
        base_length
    };
    writer.append(fence_char.to_string().repeat(closing_fence_len));
    writer.blank_line();
}

/// Render an indented code block
pub fn render_indented_code_block(
    code_block: &NodeCodeBlock,
    _ctx: &dyn NodeFormatterContext,
    writer: &mut MarkdownWriter,
    options: &FormatOptions,
) {
    let code_content = if options.indented_code_minimize_indent {
        minimize_indent(&code_block.literal)
    } else {
        code_block.literal.clone()
    };

    let mut lines = code_content.split('\n').peekable();
    while let Some(line) = lines.next() {
        if !line.is_empty() {
            writer.append_raw(INDENTED_CODE_SPACES);
            writer.append_raw(line);
        }
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
pub fn minimize_indent(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return content.to_string();
    }

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

    let min_indent = match min_indent {
        Some(indent) => indent,
        None => return content.to_string(),
    };

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

/// Calculate the content length of a heading for Setext underline
///
/// This function calculates the visible content length of a heading,
/// accounting for Unicode character widths and inline formatting.
pub fn calculate_heading_content_length(
    ctx: &mut dyn NodeFormatterContext,
    node_id: NodeId,
) -> usize {
    use crate::core::nodes::NodeValue;

    let arena = ctx.get_arena();

    arena.children_iter(node_id).fold(0, |length, child_id| {
        let child = arena.get(child_id);

        let child_length = match &child.value {
            NodeValue::Text(text) => crate::text::unicode::width(text.as_ref()) as usize,
            NodeValue::Code(code) => crate::text::unicode::width(&code.literal) as usize,
            NodeValue::Emph | NodeValue::Strong => {
                calculate_child_content_length(arena, child_id)
            }
            NodeValue::Link(_) | NodeValue::Image(_) => {
                calculate_child_content_length(arena, child_id)
            }
            NodeValue::SoftBreak | NodeValue::HardBreak => 1,
            _ => calculate_child_content_length(arena, child_id),
        };

        length + child_length
    })
}

/// Calculate content length from children recursively
pub fn calculate_child_content_length(
    arena: &crate::core::arena::NodeArena,
    node_id: NodeId,
) -> usize {
    use crate::core::nodes::NodeValue;

    arena.children_iter(node_id).fold(0, |length, child_id| {
        let child = arena.get(child_id);

        let child_length = match &child.value {
            NodeValue::Text(text) => crate::text::unicode::width(text.as_ref()) as usize,
            NodeValue::Code(code) => crate::text::unicode::width(&code.literal) as usize,
            NodeValue::SoftBreak => 1,
            _ => calculate_child_content_length(arena, child_id),
        };

        length + child_length
    })
}

/// Render an HTML block
///
/// This function handles the rendering of HTML blocks:
/// - For single-line HTML comments, outputs without extra blank lines
/// - For multi-line HTML blocks, adds blank lines before and after
/// - Outputs the HTML content as-is
pub fn render_html_block(
    html: &NodeHtmlBlock,
    _ctx: &dyn NodeFormatterContext,
    writer: &mut MarkdownWriter,
) {
    let content = &html.literal;

    let is_single_line_comment = content.trim().starts_with("<!--")
        && content.trim().ends_with("-->")
        && !content.trim().contains('\n');

    if is_single_line_comment {
        writer.append_raw(content.trim());
        writer.line();
    } else {
        writer.blank_line();

        for line in content.split('\n') {
            writer.append_raw(line);
            writer.line();
        }

        writer.tail_blank_line();
    }
}
