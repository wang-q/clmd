//! Formatter utility functions
//!
//! This module provides utility functions for formatting Markdown content,
//! inspired by flexmark-java's FormatterUtils class.

use crate::core::nodes::{NodeCodeBlock, NodeList};
use crate::formatter::context::NodeFormatterContext;
use crate::formatter::writer::MarkdownWriter;
use crate::options::format::{BulletMarker, CodeFenceMarker, NumberedMarker};

/// Render a list
///
/// This function is currently a stub. List rendering is fully implemented
/// in the main formatter. This function may be completed in the future
/// for alternative list rendering workflows.
pub fn render_list(
    _list: &NodeList,
    _context: &mut dyn NodeFormatterContext,
    _writer: &mut MarkdownWriter,
) {
    // List rendering is fully implemented in commonmark_formatter.rs
    // This stub exists for potential future use in alternative workflows
    unimplemented!(
        "render_list is not yet fully implemented. Use the main formatter for list rendering."
    )
}

/// Get the bullet marker character
pub fn get_bullet_marker(marker: BulletMarker) -> char {
    match marker {
        BulletMarker::Dash => '-',
        BulletMarker::Asterisk => '*',
        BulletMarker::Plus => '+',
        BulletMarker::Any => '-', // Default to dash
    }
}

/// Get the numbered list delimiter
pub fn get_numbered_delimiter(marker: NumberedMarker) -> char {
    match marker {
        NumberedMarker::Period => '.',
        NumberedMarker::Paren => ')',
        NumberedMarker::Any => '.', // Default to period
    }
}

/// Get the code fence marker string
pub fn get_code_fence_marker(marker: CodeFenceMarker, length: usize) -> String {
    let ch = match marker {
        CodeFenceMarker::BackTick => '`',
        CodeFenceMarker::Tilde => '~',
        CodeFenceMarker::Any => '`', // Default to backtick
    };
    ch.to_string().repeat(length)
}

/// Render a code block
pub fn render_code_block(
    code_block: &NodeCodeBlock,
    context: &mut dyn NodeFormatterContext,
    writer: &mut MarkdownWriter,
) {
    let options = context.get_formatter_options();

    if code_block.fenced {
        // Fenced code block
        let marker = get_code_fence_marker(
            options.fenced_code_marker_type,
            options.fenced_code_marker_length,
        );

        writer.append(&marker);

        // Add info string if present
        if !code_block.info.is_empty() {
            if options.fenced_code_space_before_info {
                writer.space();
            }
            writer.append(&code_block.info);
        }

        writer.line();

        // Write the code content
        if !code_block.literal.is_empty() {
            writer.open_pre_formatted();
            writer.append(&code_block.literal);
            writer.close_pre_formatted();
            writer.line();
        }

        // Closing fence
        writer.append(&marker);
    } else {
        // Indented code block
        let indent = 4;

        let indent_str = " ".repeat(indent);

        writer.push_prefix(&indent_str);
        writer.open_pre_formatted();
        writer.append(&code_block.literal);
        writer.close_pre_formatted();
        writer.pop_prefix();
    }
}

/// Strip soft line breaks from text
pub fn strip_soft_line_breaks(text: &str, replacement: &str) -> String {
    text.replace('\n', replacement)
}

/// Get the appropriate number of blank lines between blocks
pub fn get_blank_lines_between_blocks(
    prev_block_type: &str,
    next_block_type: &str,
    context: &dyn NodeFormatterContext,
) -> usize {
    let options = context.get_formatter_options();

    // Default to one blank line between most blocks
    let blank_lines = match (prev_block_type, next_block_type) {
        ("Document", _) => 0,
        (_, "Document") => 0,
        ("Paragraph", "Paragraph") => 1,
        ("Heading", _) => 1,
        (_, "Heading") => 1,
        ("CodeBlock", _) => 1,
        (_, "CodeBlock") => 1,
        ("List", "List") => 1,
        ("BlockQuote", "BlockQuote") => 0, // Block quotes can be adjacent
        _ => 1,
    };

    blank_lines.min(options.max_blank_lines)
}

/// Repeat a string n times
pub fn repeat_string(s: &str, n: usize) -> String {
    s.repeat(n)
}

/// Pad a string to a minimum width
pub fn pad_to_width(s: &str, width: usize, pad_char: char) -> String {
    if s.len() >= width {
        s.to_string()
    } else {
        let padding = width - s.len();
        format!("{}{}", s, pad_char.to_string().repeat(padding))
    }
}

/// Truncate text to fit within a width
pub fn truncate_to_width(text: &str, max_width: usize, suffix: &str) -> String {
    if text.len() <= max_width {
        text.to_string()
    } else {
        let suffix_len = suffix.len();
        let truncate_at = max_width.saturating_sub(suffix_len);
        format!("{}{}", &text[..truncate_at], suffix)
    }
}

/// Wrap text at a specific width
pub fn wrap_text(text: &str, width: usize) -> Vec<String> {
    if width == 0 || text.len() <= width {
        return vec![text.to_string()];
    }

    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line.push_str(word);
        } else if current_line.len() + 1 + word.len() <= width {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            lines.push(current_line);
            current_line = word.to_string();
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
}

/// Check if a character is a whitespace character
pub fn is_whitespace(ch: char) -> bool {
    ch.is_whitespace()
}

/// Trim trailing whitespace from a string
pub fn trim_trailing_whitespace(s: &str) -> &str {
    s.trim_end_matches(|c: char| c.is_whitespace())
}

/// Get the indentation level of a line
pub fn get_indent_level(line: &str) -> usize {
    line.chars().take_while(|c| c.is_whitespace()).count()
}

/// Remove common leading indentation from multiple lines
pub fn remove_common_indent(lines: &[&str]) -> Vec<String> {
    if lines.is_empty() {
        return Vec::new();
    }

    // Find minimum indentation (excluding empty lines)
    let min_indent = lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| get_indent_level(line))
        .min()
        .unwrap_or(0);

    lines
        .iter()
        .map(|line| {
            if line.len() >= min_indent {
                line[min_indent..].to_string()
            } else {
                line.to_string()
            }
        })
        .collect()
}

/// Format a heading marker
pub fn format_heading_marker(
    level: u8,
    setext: bool,
    setext_char: Option<char>,
) -> String {
    if setext {
        let ch = setext_char.unwrap_or('=');
        ch.to_string().repeat(level as usize)
    } else {
        "#".repeat(level as usize).to_string()
    }
}

/// Check if a string is a valid URL
pub fn is_valid_url(s: &str) -> bool {
    s.starts_with("http://")
        || s.starts_with("https://")
        || s.starts_with("ftp://")
        || s.starts_with("mailto:")
        || s.starts_with("/")
        || s.starts_with("#")
}

/// Normalize line endings to LF
pub fn normalize_line_endings(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_bullet_marker() {
        assert_eq!(get_bullet_marker(BulletMarker::Dash), '-');
        assert_eq!(get_bullet_marker(BulletMarker::Asterisk), '*');
        assert_eq!(get_bullet_marker(BulletMarker::Plus), '+');
    }

    #[test]
    fn test_get_code_fence_marker() {
        assert_eq!(get_code_fence_marker(CodeFenceMarker::BackTick, 3), "```");
        assert_eq!(get_code_fence_marker(CodeFenceMarker::Tilde, 4), "~~~~");
    }

    #[test]
    fn test_strip_soft_line_breaks() {
        assert_eq!(strip_soft_line_breaks("Hello\nWorld", " "), "Hello World");
        assert_eq!(strip_soft_line_breaks("Hello\n\nWorld", ""), "HelloWorld");
    }

    #[test]
    fn test_escape_markdown() {
        // Test moved to escaping.rs module
        // This test is kept for backward compatibility
        let escaped =
            crate::formatter::escaping::escape_text("Hello *world*", &MockContext);
        assert!(escaped.contains("\\*"));
    }

    // Mock context for testing
    struct MockContext;

    impl NodeFormatterContext for MockContext {
        fn get_markdown_writer(
            &mut self,
        ) -> &mut crate::formatter::writer::MarkdownWriter {
            panic!("Not implemented")
        }

        fn render(&mut self, _node_id: crate::core::arena::NodeId) {
            panic!("Not implemented")
        }

        fn render_children(&mut self, _node_id: crate::core::arena::NodeId) {
            panic!("Not implemented")
        }

        fn get_formatting_phase(&self) -> crate::formatter::phase::FormattingPhase {
            crate::formatter::phase::FormattingPhase::Document
        }

        fn delegate_render(&mut self) {}

        fn get_formatter_options(&self) -> &crate::options::format::FormatOptions {
            panic!("Not implemented")
        }

        fn get_render_purpose(&self) -> crate::formatter::purpose::RenderPurpose {
            crate::formatter::purpose::RenderPurpose::Format
        }

        fn get_arena(&self) -> &crate::core::arena::NodeArena {
            panic!("Not implemented")
        }

        fn get_current_node(&self) -> Option<crate::core::arena::NodeId> {
            None
        }

        fn get_nodes_of_type(
            &self,
            _node_type: crate::formatter::node::NodeValueType,
        ) -> Vec<crate::core::arena::NodeId> {
            vec![]
        }

        fn get_nodes_of_types(
            &self,
            _node_types: &[crate::formatter::node::NodeValueType],
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
    }

    #[test]
    fn test_wrap_text() {
        let lines = wrap_text("Hello world this is a long text", 10);
        assert!(!lines.is_empty());
        assert!(lines.iter().all(|line| line.len() <= 10));
    }

    #[test]
    fn test_pad_to_width() {
        assert_eq!(pad_to_width("Hello", 10, ' '), "Hello     ");
        assert_eq!(pad_to_width("Hello", 3, ' '), "Hello");
    }

    #[test]
    fn test_truncate_to_width() {
        assert_eq!(truncate_to_width("Hello World", 8, "..."), "Hello...");
        assert_eq!(truncate_to_width("Hi", 10, "..."), "Hi");
    }

    #[test]
    fn test_remove_common_indent() {
        let lines = vec!["    Hello", "    World", ""];
        let result = remove_common_indent(&lines);
        assert_eq!(result, vec!["Hello", "World", ""]);
    }

    #[test]
    fn test_format_heading_marker() {
        assert_eq!(format_heading_marker(2, false, None), "##");
        assert_eq!(format_heading_marker(1, true, Some('=')), "=");
    }

    #[test]
    fn test_is_valid_url() {
        assert!(is_valid_url("https://example.com"));
        assert!(is_valid_url("/path/to/file"));
        assert!(is_valid_url("#anchor"));
        assert!(!is_valid_url("not a url"));
    }

    #[test]
    fn test_normalize_line_endings() {
        assert_eq!(normalize_line_endings("Hello\r\nWorld"), "Hello\nWorld");
        assert_eq!(normalize_line_endings("Hello\rWorld"), "Hello\nWorld");
    }
}
