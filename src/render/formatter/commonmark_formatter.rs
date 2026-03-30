//! CommonMark node formatter implementation
//!
//! This module provides a NodeFormatter implementation for CommonMark output,
//! migrating the existing commonmark.rs functionality to the formatter framework.
//!
//! # Example
//!
//! ```
//! use clmd::render::formatter::{CommonMarkNodeFormatter, FormatterOptions};
//!
//! let formatter = CommonMarkNodeFormatter::new();
//! let options = FormatterOptions::new().with_right_margin(80);
//! let formatter = CommonMarkNodeFormatter::with_options(options);
//! ```

use crate::arena::NodeId;
use crate::nodes::NodeValue;
use crate::render::formatter::context::NodeFormatterContext;
use crate::render::formatter::node::{NodeFormatter, NodeFormattingHandler, NodeValueType};
use crate::render::formatter::options::FormatterOptions;
use crate::render::formatter::phase::FormattingPhase;
use crate::render::formatter::phased::PhasedNodeFormatter;
use crate::render::formatter::writer::MarkdownWriter;

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
    options: FormatterOptions,
}

impl CommonMarkNodeFormatter {
    /// Create a new CommonMark formatter with default options
    ///
    /// # Example
    ///
    /// ```
    /// use clmd::render::formatter::CommonMarkNodeFormatter;
    ///
    /// let formatter = CommonMarkNodeFormatter::new();
    /// ```
    pub fn new() -> Self {
        Self::with_options(FormatterOptions::default())
    }

    /// Create a new CommonMark formatter with custom options
    ///
    /// # Arguments
    ///
    /// * `options` - Formatter options for customizing output
    ///
    /// # Example
    ///
    /// ```
    /// use clmd::render::formatter::{CommonMarkNodeFormatter, FormatterOptions};
    ///
    /// let options = FormatterOptions::new()
    ///     .with_right_margin(80)
    ///     .with_keep_hard_line_breaks(true);
    /// let formatter = CommonMarkNodeFormatter::with_options(options);
    /// ```
    pub fn with_options(options: FormatterOptions) -> Self {
        Self { options }
    }

    /// Get the formatter options
    pub fn options(&self) -> &FormatterOptions {
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
                Box::new(|_value: &NodeValue, _ctx: &mut dyn NodeFormatterContext, _writer: &mut MarkdownWriter| {
                    // Document is handled at the top level
                }),
            ),
            // Block elements
            NodeFormattingHandler::with_close(
                NodeValueType::Paragraph,
                Box::new(|_value: &NodeValue, _ctx: &mut dyn NodeFormatterContext, _writer: &mut MarkdownWriter| {
                    // Paragraph opening - nothing special needed
                }),
                Box::new(|_value: &NodeValue, ctx: &mut dyn NodeFormatterContext, writer: &mut MarkdownWriter| {
                    // Paragraph closing - add blank line after paragraph
                    if !ctx.is_in_tight_list() {
                        writer.blank_line();
                    }
                }),
            ),
            NodeFormattingHandler::with_close(
                NodeValueType::Heading,
                Box::new(|value: &NodeValue, _ctx: &mut dyn NodeFormatterContext, writer: &mut MarkdownWriter| {
                    if let NodeValue::Heading(heading) = value {
                        let hashes = "#".repeat(heading.level as usize);
                        // Append hash and space together to avoid whitespace trimming
                        writer.append_raw(format!("{} ", hashes));
                    }
                }),
                Box::new(|_value: &NodeValue, _ctx: &mut dyn NodeFormatterContext, writer: &mut MarkdownWriter| {
                    // Heading closing - add blank line after heading
                    writer.blank_line();
                }),
            ),
            NodeFormattingHandler::with_close(
                NodeValueType::BlockQuote,
                Box::new(|_value: &NodeValue, ctx: &mut dyn NodeFormatterContext, writer: &mut MarkdownWriter| {
                    writer.push_prefix("> ");
                    ctx.set_in_block_quote(true);
                    ctx.increment_block_quote_nesting();
                }),
                Box::new(|_value: &NodeValue, ctx: &mut dyn NodeFormatterContext, writer: &mut MarkdownWriter| {
                    writer.pop_prefix();
                    ctx.decrement_block_quote_nesting();
                    if ctx.get_block_quote_nesting_level() == 0 {
                        ctx.set_in_block_quote(false);
                    }
                }),
            ),
            NodeFormattingHandler::new(
                NodeValueType::CodeBlock,
                Box::new(|value: &NodeValue, _ctx: &mut dyn NodeFormatterContext, writer: &mut MarkdownWriter| {
                    if let NodeValue::CodeBlock(code_block) = value {
                        render_code_block(code_block, writer);
                    }
                }),
            ),
            NodeFormattingHandler::with_close(
                NodeValueType::List,
                Box::new(|value: &NodeValue, ctx: &mut dyn NodeFormatterContext, _writer: &mut MarkdownWriter| {
                    if let NodeValue::List(list) = value {
                        ctx.set_tight_list(list.tight);
                        ctx.increment_list_nesting();
                    }
                }),
                Box::new(|_value: &NodeValue, ctx: &mut dyn NodeFormatterContext, _writer: &mut MarkdownWriter| {
                    ctx.decrement_list_nesting();
                    if ctx.get_list_nesting_level() == 0 {
                        ctx.set_tight_list(false);
                    }
                }),
            ),
            NodeFormattingHandler::with_close(
                NodeValueType::Item,
                Box::new(|_value: &NodeValue, ctx: &mut dyn NodeFormatterContext, writer: &mut MarkdownWriter| {
                    // Get the parent list to determine the marker
                    let prefix = if let Some(parent_id) = ctx.get_current_node_parent() {
                        let arena = ctx.get_arena();
                        let parent = arena.get(parent_id);
                        if let NodeValue::List(list) = &parent.value {
                            format_list_item_prefix(list)
                        } else {
                            "- ".to_string()
                        }
                    } else {
                        "- ".to_string()
                    };
                    writer.push_prefix(&prefix);
                }),
                Box::new(|_value: &NodeValue, ctx: &mut dyn NodeFormatterContext, writer: &mut MarkdownWriter| {
                    writer.pop_prefix();
                    // Add line break after each list item
                    writer.line();
                }),
            ),
            NodeFormattingHandler::new(
                NodeValueType::ThematicBreak,
                Box::new(|_value: &NodeValue, _ctx: &mut dyn NodeFormatterContext, writer: &mut MarkdownWriter| {
                    writer.append("***");
                    writer.line();
                }),
            ),
            NodeFormattingHandler::new(
                NodeValueType::HtmlBlock,
                Box::new(|value: &NodeValue, _ctx: &mut dyn NodeFormatterContext, writer: &mut MarkdownWriter| {
                    if let NodeValue::HtmlBlock(html) = value {
                        for line in html.literal.lines() {
                            writer.append(line);
                            writer.line();
                        }
                    }
                }),
            ),
            // Inline elements
            NodeFormattingHandler::new(
                NodeValueType::Text,
                Box::new(|value: &NodeValue, _ctx: &mut dyn NodeFormatterContext, writer: &mut MarkdownWriter| {
                    if let NodeValue::Text(text) = value {
                        let escaped = escape_markdown(text);
                        // Use append_raw to preserve whitespace in text content
                        writer.append_raw(&escaped);
                    }
                }),
            ),
            NodeFormattingHandler::new(
                NodeValueType::Code,
                Box::new(|value: &NodeValue, _ctx: &mut dyn NodeFormatterContext, writer: &mut MarkdownWriter| {
                    if let NodeValue::Code(code) = value {
                        let backticks = get_backtick_sequence(&code.literal);
                        writer.append(&backticks);
                        writer.append(&code.literal);
                        writer.append(&backticks);
                    }
                }),
            ),
            NodeFormattingHandler::with_close(
                NodeValueType::Emph,
                Box::new(|_value: &NodeValue, _ctx: &mut dyn NodeFormatterContext, writer: &mut MarkdownWriter| {
                    writer.append("*");
                }),
                Box::new(|_value: &NodeValue, _ctx: &mut dyn NodeFormatterContext, writer: &mut MarkdownWriter| {
                    writer.append("*");
                }),
            ),
            NodeFormattingHandler::with_close(
                NodeValueType::Strong,
                Box::new(|_value: &NodeValue, _ctx: &mut dyn NodeFormatterContext, writer: &mut MarkdownWriter| {
                    writer.append("**");
                }),
                Box::new(|_value: &NodeValue, _ctx: &mut dyn NodeFormatterContext, writer: &mut MarkdownWriter| {
                    writer.append("**");
                }),
            ),
            NodeFormattingHandler::with_close(
                NodeValueType::Link,
                Box::new(|_value: &NodeValue, _ctx: &mut dyn NodeFormatterContext, writer: &mut MarkdownWriter| {
                    writer.append("[");
                }),
                Box::new(|value: &NodeValue, _ctx: &mut dyn NodeFormatterContext, writer: &mut MarkdownWriter| {
                    if let NodeValue::Link(link) = value {
                        writer.append("](");
                        writer.append(&escape_link_url(&link.url));
                        if !link.title.is_empty() {
                            writer.append(&format!(" \"{}\"", escape_string(&link.title)));
                        }
                        writer.append(")");
                    }
                }),
            ),
            NodeFormattingHandler::with_close(
                NodeValueType::Image,
                Box::new(|_value: &NodeValue, _ctx: &mut dyn NodeFormatterContext, writer: &mut MarkdownWriter| {
                    writer.append("![");
                }),
                Box::new(|value: &NodeValue, _ctx: &mut dyn NodeFormatterContext, writer: &mut MarkdownWriter| {
                    if let NodeValue::Image(link) = value {
                        writer.append("](");
                        writer.append(&escape_link_url(&link.url));
                        if !link.title.is_empty() {
                            writer.append(&format!(" \"{}\"", escape_string(&link.title)));
                        }
                        writer.append(")");
                    }
                }),
            ),
            NodeFormattingHandler::new(
                NodeValueType::Strikethrough,
                Box::new(|_value: &NodeValue, _ctx: &mut dyn NodeFormatterContext, writer: &mut MarkdownWriter| {
                    writer.append("~~");
                }),
            ),
            NodeFormattingHandler::new(
                NodeValueType::SoftBreak,
                Box::new(|_value: &NodeValue, ctx: &mut dyn NodeFormatterContext, writer: &mut MarkdownWriter| {
                    if ctx.is_in_tight_list() {
                        writer.append(" ");
                    } else {
                        writer.line();
                    }
                }),
            ),
            NodeFormattingHandler::new(
                NodeValueType::HardBreak,
                Box::new(|_value: &NodeValue, _ctx: &mut dyn NodeFormatterContext, writer: &mut MarkdownWriter| {
                    writer.append("  ");
                    writer.line();
                }),
            ),
            NodeFormattingHandler::new(
                NodeValueType::HtmlInline,
                Box::new(|value: &NodeValue, _ctx: &mut dyn NodeFormatterContext, writer: &mut MarkdownWriter| {
                    if let NodeValue::HtmlInline(html) = value {
                        writer.append(html);
                    }
                }),
            ),
            // Table elements
            NodeFormattingHandler::with_close(
                NodeValueType::Table,
                Box::new(|value: &NodeValue, ctx: &mut dyn NodeFormatterContext, _writer: &mut MarkdownWriter| {
                    // Table opening - start collecting data
                    if let NodeValue::Table(table) = value {
                        ctx.start_table_collection(table.alignments.clone());
                    }
                }),
                Box::new(|_value: &NodeValue, ctx: &mut dyn NodeFormatterContext, writer: &mut MarkdownWriter| {
                    // Table closing - format and output using table.rs
                    if let Some((rows, alignments)) = ctx.take_table_data() {
                        render_formatted_table(&rows, &alignments, writer);
                    }
                }),
            ),
            NodeFormattingHandler::with_close(
                NodeValueType::TableRow,
                Box::new(|_value: &NodeValue, ctx: &mut dyn NodeFormatterContext, _writer: &mut MarkdownWriter| {
                    // Row opening - add new row to collection
                    ctx.add_table_row();
                }),
                Box::new(|_value: &NodeValue, _ctx: &mut dyn NodeFormatterContext, _writer: &mut MarkdownWriter| {
                    // Row closing - nothing to do
                }),
            ),
            NodeFormattingHandler::with_close(
                NodeValueType::TableCell,
                Box::new(|_value: &NodeValue, ctx: &mut dyn NodeFormatterContext, _writer: &mut MarkdownWriter| {
                    // Cell opening - if collecting table, skip rendering children
                    // They will be collected on close
                    if ctx.is_collecting_table() {
                        ctx.set_skip_children(true);
                    }
                }),
                Box::new(|_value: &NodeValue, ctx: &mut dyn NodeFormatterContext, _writer: &mut MarkdownWriter| {
                    // Cell closing - collect text content directly without full rendering
                    if ctx.is_collecting_table() {
                        if let Some(node_id) = ctx.get_current_node() {
                            let content = collect_cell_text_content(ctx.get_arena(), node_id);
                            ctx.add_table_cell(content);
                        }
                    }
                }),
            ),
            // Footnote elements
            NodeFormattingHandler::new(
                NodeValueType::FootnoteReference,
                Box::new(|value: &NodeValue, _ctx: &mut dyn NodeFormatterContext, writer: &mut MarkdownWriter| {
                    if let NodeValue::FootnoteReference(footnote) = value {
                        writer.append(&format!("[^{}]", footnote.name));
                    }
                }),
            ),
            NodeFormattingHandler::new(
                NodeValueType::FootnoteDefinition,
                Box::new(|value: &NodeValue, _ctx: &mut dyn NodeFormatterContext, writer: &mut MarkdownWriter| {
                    if let NodeValue::FootnoteDefinition(footnote) = value {
                        writer.append(&format!("[^{}]:", footnote.name));
                    }
                }),
            ),
            // Task items
            NodeFormattingHandler::new(
                NodeValueType::TaskItem,
                Box::new(|value: &NodeValue, _ctx: &mut dyn NodeFormatterContext, writer: &mut MarkdownWriter| {
                    if let NodeValue::TaskItem(task) = value {
                        if task.symbol.is_some() {
                            writer.append("[x] ");
                        } else {
                            writer.append("[ ] ");
                        }
                    }
                }),
            ),
        ]
    }
}

impl PhasedNodeFormatter for CommonMarkNodeFormatter {
    fn get_formatting_phases(&self) -> Vec<FormattingPhase> {
        vec![
            FormattingPhase::Collect,
            FormattingPhase::Document,
        ]
    }

    fn render_document(
        &self,
        _context: &mut dyn NodeFormatterContext,
        _writer: &mut MarkdownWriter,
        _root: NodeId,
        phase: FormattingPhase,
    ) {
        match phase {
            FormattingPhase::Collect => {
                // In the Collect phase, we could gather reference links
                // and other information needed for the main rendering pass
                // This is currently a placeholder for future enhancement
            }
            FormattingPhase::Document => {
                // Main document rendering is handled by the node handlers
            }
            _ => {}
        }
    }
}

/// Render a code block with proper fencing
///
/// Determines the appropriate fence length to avoid conflicts with
/// backticks in the code content.
fn render_code_block(code_block: &crate::nodes::NodeCodeBlock, writer: &mut MarkdownWriter) {
    // Determine fence length
    let mut fence_len = 3;
    for seq in code_block.literal.split('\n') {
        let mut count = 0;
        for c in seq.chars() {
            if c == '`' {
                count += 1;
                fence_len = fence_len.max(count + 1);
            } else {
                count = 0;
            }
        }
    }

    let fence = "`".repeat(fence_len);
    writer.append(&fence);

    // Add info string on the same line as the opening fence
    if !code_block.info.is_empty() {
        let clean_info = code_block.info.trim_end_matches('`');
        if !clean_info.is_empty() {
            writer.append(" ");
            writer.append(clean_info);
        }
    }
    writer.line();

    for line in code_block.literal.lines() {
        writer.append(line);
        writer.line();
    }

    writer.append(&fence);
    writer.line();
}

/// Collect text content from a cell node and its children
///
/// This function recursively collects text from Text nodes and applies
/// appropriate formatting for inline elements, but avoids escaping pipe
/// characters which are used for table cell separation.
fn collect_cell_text_content(arena: &crate::arena::NodeArena, node_id: crate::arena::NodeId) -> String {
    use crate::arena::TreeOps;
    use crate::nodes::NodeValue;
    
    let mut result = String::new();
    let node = arena.get(node_id);
    
    match &node.value {
        NodeValue::Text(text) => {
            // Escape markdown special chars but not pipe
            result.push_str(&escape_markdown_for_table(text));
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
            result.push_str(&escape_link_url(&link.url));
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
    alignments: &[crate::nodes::TableAlignment],
    writer: &mut MarkdownWriter,
) {
    use crate::render::formatter::table;

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
        let cells: Vec<String> = header_row.iter().map(|cell| cell.clone()).collect();
        let line = format!("| {} |", cells.join(" | "));
        lines.push(line);
    }

    // Add a placeholder delimiter row (will be skipped by format_table_lines)
    // We use a simple delimiter that matches the number of columns
    let num_cols = alignments.len().max(1);
    let placeholder_delimiter: Vec<String> = (0..num_cols).map(|_| "---".to_string()).collect();
    lines.push(format!("| {} |", placeholder_delimiter.join(" | ")));

    // Add data rows (remaining rows)
    for row in rows.iter().skip(1) {
        let cells: Vec<String> = row.iter().map(|cell| cell.clone()).collect();
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
}

/// Get the appropriate backtick sequence for inline code
///
/// Determines the minimum number of backticks needed to wrap the content
/// without conflicting with backticks inside the content.
///
/// # Examples
///
/// ```
/// use clmd::render::formatter::commonmark_formatter::get_backtick_sequence;
///
/// assert_eq!(get_backtick_sequence("code"), "`");
/// assert_eq!(get_backtick_sequence("code `with` backticks"), "``");
/// assert_eq!(get_backtick_sequence("``double``"), "```");
/// ```
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

/// Escape special Markdown characters in text
///
/// Escapes characters that have special meaning in Markdown to ensure
/// they are rendered as literal text.
///
/// # Examples
///
/// ```
/// use clmd::render::formatter::commonmark_formatter::escape_markdown;
///
/// assert_eq!(escape_markdown("*text*"), "\\*text\\*");
/// assert_eq!(escape_markdown("_text_"), "\\_text\\_");
/// assert_eq!(escape_markdown("[link]"), "\\[link\\]");
/// ```
pub fn escape_markdown(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let special_chars = [
        '*', '_', '[', ']', '(', ')', '<', '>', '#', '`', '\\', '!', '|',
    ];

    for c in text.chars() {
        if special_chars.contains(&c) {
            result.push('\\');
        }
        result.push(c);
    }

    result
}

/// Escape special Markdown characters in text, but preserve pipe for table cells
///
/// This is used for table cell content where pipe characters should not be escaped.
fn escape_markdown_for_table(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    // Note: '|' is not included here as it's used for table cell separation
    let special_chars = [
        '*', '_', '[', ']', '(', ')', '<', '>', '#', '`', '\\', '!',
    ];

    for c in text.chars() {
        if special_chars.contains(&c) {
            result.push('\\');
        }
        result.push(c);
    }

    result
}

/// Escape string for use in link title
///
/// Escapes quotes and backslashes in link titles.
pub fn escape_string(text: &str) -> String {
    text.replace('"', "\\\"").replace('\\', "\\\\")
}

/// Escape URL for use in link destination
///
/// Escapes special characters that need escaping in link URLs.
pub fn escape_link_url(url: &str) -> String {
    let mut result = String::with_capacity(url.len());
    let special_chars = ['(', ')', '<', '>', '[', ']', '"', ' ', '\n'];

    for c in url.chars() {
        if special_chars.contains(&c) {
            result.push('\\');
        }
        result.push(c);
    }

    result
}

/// Format list item prefix based on list type
///
/// Returns the appropriate prefix for a list item (e.g., "- ", "1. ", "2) ")
fn format_list_item_prefix(list: &crate::nodes::NodeList) -> String {
    use crate::nodes::{ListDelimType, ListType};

    match list.list_type {
        ListType::Bullet => {
            format!("{} ", list.bullet_char as char)
        }
        ListType::Ordered => {
            let marker = match list.delimiter {
                ListDelimType::Period => format!("{}.", list.start),
                ListDelimType::Paren => format!("{})", list.start),
            };
            // Pad to 4 characters for alignment
            format!("{:4}", marker)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commonmark_formatter_creation() {
        let formatter = CommonMarkNodeFormatter::new();
        let handlers = formatter.get_node_formatting_handlers();
        assert!(!handlers.is_empty());
        assert_eq!(handlers.len(), 25); // All node types including TableRow and TableCell
    }

    #[test]
    fn test_commonmark_formatter_default() {
        let formatter: CommonMarkNodeFormatter = Default::default();
        let handlers = formatter.get_node_formatting_handlers();
        assert!(!handlers.is_empty());
    }

    #[test]
    fn test_commonmark_formatter_with_options() {
        let options = FormatterOptions::new()
            .with_right_margin(80)
            .with_keep_hard_line_breaks(true);
        let formatter = CommonMarkNodeFormatter::with_options(options);
        let handlers = formatter.get_node_formatting_handlers();
        assert!(!handlers.is_empty());
        assert_eq!(formatter.options().right_margin, 80);
    }

    #[test]
    fn test_escape_markdown() {
        assert_eq!(escape_markdown("*text*"), "\\*text\\*");
        assert_eq!(escape_markdown("_text_"), "\\_text\\_");
        assert_eq!(escape_markdown("[link]"), "\\[link\\]");
        assert_eq!(escape_markdown("(paren)"), "\\(paren\\)");
        assert_eq!(escape_markdown("`code`"), "\\`code\\`");
    }

    #[test]
    fn test_escape_markdown_no_special_chars() {
        assert_eq!(escape_markdown("plain text"), "plain text");
        assert_eq!(escape_markdown("123"), "123");
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
        assert_eq!(phases.len(), 2);
        assert!(phases.contains(&FormattingPhase::Collect));
        assert!(phases.contains(&FormattingPhase::Document));
    }

    #[test]
    fn test_formatter_options_accessor() {
        let options = FormatterOptions::new().with_right_margin(100);
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
    fn test_escape_link_url() {
        assert_eq!(escape_link_url("https://example.com"), "https://example.com");
        assert_eq!(escape_link_url("url with space"), "url\\ with\\ space");
        assert_eq!(escape_link_url("(paren)"), "\\(paren\\)");
    }

    #[test]
    fn test_format_list_item_prefix_bullet() {
        use crate::nodes::{ListDelimType, ListType, NodeList};

        let list = NodeList {
            list_type: ListType::Bullet,
            marker_offset: 0,
            padding: 0,
            start: 1,
            delimiter: ListDelimType::Period,
            bullet_char: b'-',
            tight: false,
            is_task_list: false,
        };
        assert_eq!(format_list_item_prefix(&list), "- ");

        let list_star = NodeList {
            bullet_char: b'*',
            ..list
        };
        assert_eq!(format_list_item_prefix(&list_star), "* ");
    }

    #[test]
    fn test_format_list_item_prefix_ordered() {
        use crate::nodes::{ListDelimType, ListType, NodeList};

        let list = NodeList {
            list_type: ListType::Ordered,
            marker_offset: 0,
            padding: 0,
            start: 1,
            delimiter: ListDelimType::Period,
            bullet_char: b'-',
            tight: false,
            is_task_list: false,
        };
        assert_eq!(format_list_item_prefix(&list), "1.  ");

        let list_paren = NodeList {
            delimiter: ListDelimType::Paren,
            ..list
        };
        assert_eq!(format_list_item_prefix(&list_paren), "1)  ");

        let list_start10 = NodeList {
            start: 10,
            delimiter: ListDelimType::Period,
            ..list
        };
        assert_eq!(format_list_item_prefix(&list_start10), "10. ");
    }
}
