//! CommonMark rendering and formatting
//!
//! This module provides CommonMark output generation and the main Markdown formatter
//! implementation, inspired by flexmark-java's Formatter class.
//!
//! # Submodules
//!
//! - `context`: Main formatter context implementation (MainFormatterContext)
//! - `formatter`: Main Formatter entry point and render() function (pre-configured with CommonMark handlers)
//! - `options`: Formatter configuration options
//! - `core`: Core traits (NodeFormatterContext, NodeFormatter, etc.)
//! - `escaping`: Text escaping utilities
//! - `handler_utils`: Handler factory functions and context helpers
//! - `handlers`: Node type handlers (block, container, inline, list, table, registration)
//! - `line_breaking`: Paragraph line breaking algorithm
//! - `writer`: Markdown output writer

pub mod context;
pub mod core;
pub mod escaping;
pub mod formatter;
pub mod handler_utils;
pub mod handlers;
pub mod line_breaking;
pub mod writer;

// Re-export commonly used types from options
pub use crate::options::format::{
    Alignment, BlockQuoteMarker, BulletMarker, CodeFenceMarker, DiscretionaryText,
    ElementPlacement, ElementPlacementSort, FormatFlags, FormatOptions, HeadingStyle,
    ListSpacing, NumberedMarker, TrailingMarker,
};

// Re-export core types
pub use core::{
    ComposedNodeFormatter, NodeFormatter, NodeFormatterContext, NodeFormatterFn,
    NodeFormattingHandler, NodeType,
};

// Re-export formatter and render function
pub use formatter::{render, Formatter};

// Re-export line breaking types
pub use line_breaking::{AtomicKind, ParagraphLineBreaker};

// Re-export writer
pub use writer::MarkdownWriter;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::arena::{Node, NodeArena, TreeOps};
    use crate::core::nodes::NodeValue;

    #[test]
    fn test_formatter_creation() {
        // Just verify Formatter::new() doesn't panic
        let _formatter = Formatter::new();
    }

    #[test]
    fn test_format_document_with_commonmark_formatter() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let heading = arena.alloc(Node::with_value(NodeValue::Heading(
            crate::core::nodes::NodeHeading {
                level: 1,
                setext: false,
                closed: false,
            },
        )));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("Hello")));

        TreeOps::append_child(&mut arena, root, heading);
        TreeOps::append_child(&mut arena, heading, text);

        let formatter = Formatter::new();

        let result = formatter.render(&arena, root);
        assert!(result.contains("#"), "Should contain heading marker");
        assert!(result.contains("Hello"), "Should contain text content");
        assert!(
            result.contains("# Hello"),
            "Should have proper heading format with space"
        );
    }

    #[test]
    fn test_format_document_paragraphs() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        // First paragraph with "Hello World"
        let para1 = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text1 = arena.alloc(Node::with_value(NodeValue::make_text("Hello World")));
        TreeOps::append_child(&mut arena, para1, text1);
        TreeOps::append_child(&mut arena, root, para1);

        // Second paragraph with "Second paragraph"
        let para2 = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text2 =
            arena.alloc(Node::with_value(NodeValue::make_text("Second paragraph")));
        TreeOps::append_child(&mut arena, para2, text2);
        TreeOps::append_child(&mut arena, root, para2);

        let formatter = Formatter::new();

        let result = formatter.render(&arena, root);
        assert!(
            result.contains("Hello World"),
            "Should contain first paragraph"
        );
        assert!(
            result.contains("Second paragraph"),
            "Should contain second paragraph"
        );
        // Paragraphs should be separated by blank line
        assert!(
            result.contains("Hello World\n\nSecond")
                || result.contains("Hello World\r\n\r\nSecond"),
            "Paragraphs should be separated by blank line. Result: {:?}",
            result
        );
    }

    #[test]
    fn test_format_document_with_emphasis() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));

        // "This is "
        let text1 = arena.alloc(Node::with_value(NodeValue::make_text("This is ")));
        TreeOps::append_child(&mut arena, para, text1);

        // **bold**
        let strong = arena.alloc(Node::with_value(NodeValue::Strong));
        let bold_text = arena.alloc(Node::with_value(NodeValue::make_text("bold")));
        TreeOps::append_child(&mut arena, strong, bold_text);
        TreeOps::append_child(&mut arena, para, strong);

        // " text"
        let text2 = arena.alloc(Node::with_value(NodeValue::make_text(" text")));
        TreeOps::append_child(&mut arena, para, text2);

        TreeOps::append_child(&mut arena, root, para);

        let formatter = Formatter::new();

        let result = formatter.render(&arena, root);
        assert!(
            result.contains("This is"),
            "Should contain 'This is'. Result: {:?}",
            result
        );
        assert!(
            result.contains("**bold**"),
            "Should contain '**bold**'. Result: {:?}",
            result
        );
        assert!(
            result.contains(" text"),
            "Should contain ' text'. Result: {:?}",
            result
        );
        // Check the full format with proper spacing
        assert!(
            result.contains("This is **bold** text"),
            "Should have proper spacing around emphasis. Result: {:?}",
            result
        );
    }

    #[test]
    fn test_format_document_with_emphasis_and_width() {
        use crate::options::format::FormatOptions;

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));

        // "This is "
        let text1 = arena.alloc(Node::with_value(NodeValue::make_text("This is ")));
        TreeOps::append_child(&mut arena, para, text1);

        // *italic*
        let emph = arena.alloc(Node::with_value(NodeValue::Emph));
        let emph_text = arena.alloc(Node::with_value(NodeValue::make_text("italic")));
        TreeOps::append_child(&mut arena, emph, emph_text);
        TreeOps::append_child(&mut arena, para, emph);

        // " text"
        let text2 = arena.alloc(Node::with_value(NodeValue::make_text(" text")));
        TreeOps::append_child(&mut arena, para, text2);

        TreeOps::append_child(&mut arena, root, para);

        // Test with right_margin = 80 (like CLI)
        let opts = FormatOptions::new().with_right_margin(80);
        let formatter = Formatter::with_options(opts);

        let result = formatter.render(&arena, root);
        println!("Result with width 80: {:?}", result);
        println!("Expected: {:?}", "This is *italic* text\n\n");

        // Check the full format with proper spacing
        assert!(
            result == "This is *italic* text\n\n",
            "Should have proper spacing around emphasis. Result: {:?}",
            result
        );
    }

    #[test]
    fn test_format_document_with_table() {
        use crate::core::nodes::{NodeTable, TableAlignment};

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        // Create table
        let table =
            arena.alloc(Node::with_value(NodeValue::Table(Box::new(NodeTable {
                alignments: vec![TableAlignment::None, TableAlignment::None],
                num_columns: 2,
                num_rows: 2,
                num_nonempty_cells: 4,
            }))));

        // Header row
        let header_row = arena.alloc(Node::with_value(NodeValue::TableRow(true)));
        let cell1 = arena.alloc(Node::with_value(NodeValue::TableCell));
        let cell1_text = arena.alloc(Node::with_value(NodeValue::make_text("Name")));
        TreeOps::append_child(&mut arena, cell1, cell1_text);
        TreeOps::append_child(&mut arena, header_row, cell1);

        let cell2 = arena.alloc(Node::with_value(NodeValue::TableCell));
        let cell2_text = arena.alloc(Node::with_value(NodeValue::make_text("Age")));
        TreeOps::append_child(&mut arena, cell2, cell2_text);
        TreeOps::append_child(&mut arena, header_row, cell2);

        TreeOps::append_child(&mut arena, table, header_row);

        // Data row
        let data_row = arena.alloc(Node::with_value(NodeValue::TableRow(false)));
        let cell3 = arena.alloc(Node::with_value(NodeValue::TableCell));
        let cell3_text = arena.alloc(Node::with_value(NodeValue::make_text("Alice")));
        TreeOps::append_child(&mut arena, cell3, cell3_text);
        TreeOps::append_child(&mut arena, data_row, cell3);

        let cell4 = arena.alloc(Node::with_value(NodeValue::TableCell));
        let cell4_text = arena.alloc(Node::with_value(NodeValue::make_text("30")));
        TreeOps::append_child(&mut arena, cell4, cell4_text);
        TreeOps::append_child(&mut arena, data_row, cell4);

        TreeOps::append_child(&mut arena, table, data_row);
        TreeOps::append_child(&mut arena, root, table);

        let formatter = Formatter::new();

        let result = formatter.render(&arena, root);
        println!("Table result: {:?}", result);

        // Check that pipe characters are NOT escaped (allow for variable spacing)
        assert!(
            result.contains("Name") && result.contains("Age"),
            "Table header should contain Name and Age. Result: {:?}",
            result
        );
        // Check for data row
        assert!(
            result.contains("Alice") && result.contains("30"),
            "Table data should contain Alice and 30. Result: {:?}",
            result
        );
        // Check for delimiter row
        assert!(
            result.contains("---"),
            "Table should have delimiter row. Result: {:?}",
            result
        );
    }

    #[test]
    fn test_format_document_with_table_and_code_block() {
        use crate::core::nodes::{NodeCodeBlock, NodeTable, TableAlignment};

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        // Create table
        let table =
            arena.alloc(Node::with_value(NodeValue::Table(Box::new(NodeTable {
                alignments: vec![TableAlignment::None, TableAlignment::None],
                num_columns: 2,
                num_rows: 2,
                num_nonempty_cells: 4,
            }))));

        // Header row
        let header_row = arena.alloc(Node::with_value(NodeValue::TableRow(true)));
        let cell1 = arena.alloc(Node::with_value(NodeValue::TableCell));
        let cell1_text = arena.alloc(Node::with_value(NodeValue::make_text("Name")));
        TreeOps::append_child(&mut arena, cell1, cell1_text);
        TreeOps::append_child(&mut arena, header_row, cell1);

        let cell2 = arena.alloc(Node::with_value(NodeValue::TableCell));
        let cell2_text = arena.alloc(Node::with_value(NodeValue::make_text("Age")));
        TreeOps::append_child(&mut arena, cell2, cell2_text);
        TreeOps::append_child(&mut arena, header_row, cell2);

        TreeOps::append_child(&mut arena, table, header_row);

        // Data row
        let data_row = arena.alloc(Node::with_value(NodeValue::TableRow(false)));
        let cell3 = arena.alloc(Node::with_value(NodeValue::TableCell));
        let cell3_text = arena.alloc(Node::with_value(NodeValue::make_text("Alice")));
        TreeOps::append_child(&mut arena, cell3, cell3_text);
        TreeOps::append_child(&mut arena, data_row, cell3);

        let cell4 = arena.alloc(Node::with_value(NodeValue::TableCell));
        let cell4_text = arena.alloc(Node::with_value(NodeValue::make_text("30")));
        TreeOps::append_child(&mut arena, cell4, cell4_text);
        TreeOps::append_child(&mut arena, data_row, cell4);

        TreeOps::append_child(&mut arena, table, data_row);
        TreeOps::append_child(&mut arena, root, table);

        // Create code block
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

        let formatter = Formatter::new();

        let result = formatter.render(&arena, root);
        println!("Table + code block result: {:?}", result);

        // Check that table is present
        assert!(
            result.contains("Name") && result.contains("Age"),
            "Table header should contain Name and Age. Result: {:?}",
            result
        );

        // Check that code block is present
        assert!(
            result.contains("```rust"),
            "Should contain code block start. Result: {:?}",
            result
        );

        // Check that there's a blank line between table and code block
        // The table should end with a blank line before the code block
        assert!(
            result.contains("30  |\n\n```rust"),
            "Should have blank line between table and code block. Result: {:?}",
            result
        );
    }

    #[test]
    fn test_format_document_with_heading() {
        use crate::core::nodes::NodeHeading;

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        // Create heading
        let heading = arena.alloc(Node::with_value(NodeValue::Heading(NodeHeading {
            level: 1,
            setext: false,
            closed: false,
        })));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("Title")));
        TreeOps::append_child(&mut arena, heading, text);
        TreeOps::append_child(&mut arena, root, heading);

        let formatter = Formatter::new();

        let result = formatter.render(&arena, root);
        assert!(
            result.contains("# Title"),
            "Should contain heading. Result: {:?}",
            result
        );
    }

    #[test]
    fn test_format_document_with_list() {
        use crate::core::nodes::{ListDelimType, ListType, NodeList};

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        // Create bullet list
        let list = arena.alloc(Node::with_value(NodeValue::List(NodeList {
            list_type: ListType::Bullet,
            marker_offset: 0,
            padding: 2,
            start: 0,
            delimiter: ListDelimType::Period,
            bullet_char: b'-',
            tight: true,
            is_task_list: false,
        })));

        let item1 = arena.alloc(Node::with_value(NodeValue::Item(NodeList {
            list_type: ListType::Bullet,
            marker_offset: 0,
            padding: 2,
            start: 0,
            delimiter: ListDelimType::Period,
            bullet_char: b'-',
            tight: true,
            is_task_list: false,
        })));
        let item1_text = arena.alloc(Node::with_value(NodeValue::make_text("Item 1")));
        TreeOps::append_child(&mut arena, item1, item1_text);
        TreeOps::append_child(&mut arena, list, item1);

        let item2 = arena.alloc(Node::with_value(NodeValue::Item(NodeList {
            list_type: ListType::Bullet,
            marker_offset: 0,
            padding: 2,
            start: 0,
            delimiter: ListDelimType::Period,
            bullet_char: b'-',
            tight: true,
            is_task_list: false,
        })));
        let item2_text = arena.alloc(Node::with_value(NodeValue::make_text("Item 2")));
        TreeOps::append_child(&mut arena, item2, item2_text);
        TreeOps::append_child(&mut arena, list, item2);

        TreeOps::append_child(&mut arena, root, list);

        let formatter = Formatter::new();

        let result = formatter.render(&arena, root);
        assert!(
            result.contains("- Item 1"),
            "Should contain list item 1. Result: {:?}",
            result
        );
        assert!(
            result.contains("- Item 2"),
            "Should contain list item 2. Result: {:?}",
            result
        );
    }

    #[test]
    fn test_format_document_with_code_block() {
        use crate::core::nodes::NodeCodeBlock;

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        // Create code block
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

        let formatter = Formatter::new();

        let result = formatter.render(&arena, root);
        assert!(
            result.contains("```rust"),
            "Should contain code block start. Result: {:?}",
            result
        );
        assert!(
            result.contains("fn main() {}"),
            "Should contain code. Result: {:?}",
            result
        );
        assert!(
            result.contains("```"),
            "Should contain code block end. Result: {:?}",
            result
        );
    }

    #[test]
    fn test_format_document_with_link() {
        use crate::core::nodes::NodeLink;

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        // Create paragraph with link
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let link = arena.alloc(Node::with_value(NodeValue::Link(Box::new(NodeLink {
            url: "https://example.com".to_string(),
            title: "Example".to_string(),
        }))));
        let link_text = arena.alloc(Node::with_value(NodeValue::make_text("Example")));
        TreeOps::append_child(&mut arena, link, link_text);
        TreeOps::append_child(&mut arena, para, link);
        TreeOps::append_child(&mut arena, root, para);

        let formatter = Formatter::new();

        let result = formatter.render(&arena, root);
        assert!(
            result.contains("[Example]"),
            "Should contain link text. Result: {:?}",
            result
        );
        assert!(
            result.contains("https://example.com"),
            "Should contain link URL. Result: {:?}",
            result
        );
    }

    #[test]
    fn test_format_document_with_blockquote() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        // Create blockquote
        let quote = arena.alloc(Node::with_value(NodeValue::BlockQuote));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("Quote text")));
        TreeOps::append_child(&mut arena, para, text);
        TreeOps::append_child(&mut arena, quote, para);
        TreeOps::append_child(&mut arena, root, quote);

        let formatter = Formatter::new();

        let result = formatter.render(&arena, root);
        assert!(
            result.contains("> Quote text"),
            "Should contain blockquote. Result: {:?}",
            result
        );
    }

    #[test]
    fn test_format_empty_document() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        let formatter = Formatter::new();

        let result = formatter.render(&arena, root);
        assert!(result.is_empty() || result.trim().is_empty(), "Empty document should produce empty or whitespace-only output. Result: {:?}", result);
    }

    // ========================================================================
    // Integration tests migrated from commonmark_formatter.rs
    // ========================================================================

    #[test]
    fn test_render_nested_lists() {
        use crate::core::nodes::{ListDelimType, ListType, NodeList};

        let mut arena = NodeArena::new();

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

        let formatter = Formatter::new();

        let result = formatter.render(&arena, root);
        assert!(
            result.contains("- Item 1"),
            "Should contain first item. Result: {:?}",
            result
        );
        assert!(
            result.contains("- Item 2"),
            "Should contain second item. Result: {:?}",
            result
        );
        assert!(
            result.contains("  - Nested"),
            "Should contain nested item with indent. Result: {:?}",
            result
        );
    }

    #[test]
    fn test_render_code_block_with_backticks() {
        use crate::core::nodes::NodeCodeBlock;

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

        let formatter = Formatter::new();

        let result = formatter.render(&arena, root);
        assert!(
            result.contains("```rust"),
            "Should contain opening fence. Result: {:?}",
            result
        );
        assert!(
            result.contains("fn main() {}"),
            "Should contain code content. Result: {:?}",
            result
        );
        assert!(
            result.contains("```"),
            "Should contain closing fence. Result: {:?}",
            result
        );
    }

    #[test]
    fn test_render_heading_atx() {
        use crate::core::nodes::NodeHeading;

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

        let formatter = Formatter::new();

        let result = formatter.render(&arena, root);
        assert!(
            result.contains("## Section Title"),
            "Should contain ATX heading. Result: {:?}",
            result
        );
    }

    #[test]
    fn test_render_blockquote_simple() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        let blockquote = arena.alloc(Node::with_value(NodeValue::BlockQuote));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("Quoted text")));

        TreeOps::append_child(&mut arena, para, text);
        TreeOps::append_child(&mut arena, blockquote, para);
        TreeOps::append_child(&mut arena, root, blockquote);

        let formatter = Formatter::new();

        let result = formatter.render(&arena, root);
        assert!(
            result.contains("> Quoted text"),
            "Should contain blockquote. Result: {:?}",
            result
        );
    }

    #[test]
    fn test_render_link_and_image() {
        use crate::core::nodes::NodeLink;

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));

        let link = arena.alloc(Node::with_value(NodeValue::Link(Box::new(NodeLink {
            url: "https://example.com".to_string(),
            title: "".to_string(),
        }))));
        let link_text = arena.alloc(Node::with_value(NodeValue::make_text("example")));
        TreeOps::append_child(&mut arena, link, link_text);
        TreeOps::append_child(&mut arena, para, link);

        let image =
            arena.alloc(Node::with_value(NodeValue::Image(Box::new(NodeLink {
                url: "image.png".to_string(),
                title: "".to_string(),
            }))));
        let image_alt = arena.alloc(Node::with_value(NodeValue::make_text("alt")));
        TreeOps::append_child(&mut arena, image, image_alt);
        TreeOps::append_child(&mut arena, para, image);

        TreeOps::append_child(&mut arena, root, para);

        let formatter = Formatter::new();

        let result = formatter.render(&arena, root);
        assert!(
            result.contains("[example](https://example.com)"),
            "Should contain link. Result: {:?}",
            result
        );
        assert!(
            result.contains("![alt](image.png)"),
            "Should contain image. Result: {:?}",
            result
        );
    }

    #[test]
    fn test_render_emphasis_and_strong_standalone() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));

        let emph = arena.alloc(Node::with_value(NodeValue::Emph));
        let emph_text = arena.alloc(Node::with_value(NodeValue::make_text("emphasis")));
        TreeOps::append_child(&mut arena, emph, emph_text);
        TreeOps::append_child(&mut arena, para, emph);

        let strong = arena.alloc(Node::with_value(NodeValue::Strong));
        let strong_text = arena.alloc(Node::with_value(NodeValue::make_text("strong")));
        TreeOps::append_child(&mut arena, strong, strong_text);
        TreeOps::append_child(&mut arena, para, strong);

        TreeOps::append_child(&mut arena, root, para);

        let formatter = Formatter::new();

        let result = formatter.render(&arena, root);
        assert!(
            result.contains("*emphasis*"),
            "Should contain emphasis. Result: {:?}",
            result
        );
        assert!(
            result.contains("**strong**"),
            "Should contain strong. Result: {:?}",
            result
        );
    }

    #[test]
    fn test_is_task_item_checked_integration() {
        use crate::core::nodes::{ListDelimType, ListType, NodeList};
        use crate::render::commonmark::handlers::list::is_task_item_checked;

        let mut arena = NodeArena::new();

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

        assert!(
            is_task_item_checked(&arena, Some(item)),
            "Should detect [x] as checked"
        );

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

        assert!(
            !is_task_item_checked(&arena, Some(item2)),
            "Should detect [ ] as unchecked"
        );
        assert!(
            !is_task_item_checked(&arena, None),
            "None should return false"
        );

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
            "[X] Checked uppercase",
        )));
        TreeOps::append_child(&mut arena, para3, text3);
        TreeOps::append_child(&mut arena, item3, para3);
        TreeOps::append_child(&mut arena, list, item3);

        assert!(
            is_task_item_checked(&arena, Some(item3)),
            "[X] uppercase should be checked"
        );
    }

    #[test]
    fn test_render_task_list_integration() {
        use crate::core::nodes::{ListDelimType, ListType, NodeList};

        let mut arena = NodeArena::new();

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

        let formatter = Formatter::new();

        let result = formatter.render(&arena, root);
        assert!(
            result.contains("- [ ]"),
            "Unchecked marker. Result: {:?}",
            result
        );
        assert!(
            result.contains("- [x]"),
            "Checked marker. Result: {:?}",
            result
        );
        assert!(
            result.contains("Unchecked task"),
            "Unchecked text. Result: {:?}",
            result
        );
        assert!(
            result.contains("Checked task"),
            "Checked text. Result: {:?}",
            result
        );
    }

    // ========================================================================
    // Boundary Condition Tests
    // ========================================================================

    #[test]
    fn test_render_empty_document_boundary() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        let formatter = Formatter::new();
        let result = formatter.render(&arena, root);
        assert!(result.is_empty() || result.trim().is_empty());
    }

    #[test]
    fn test_render_empty_paragraph_boundary() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        TreeOps::append_child(&mut arena, root, para);

        let formatter = Formatter::new();
        let result = formatter.render(&arena, root);
        assert!(result.is_empty() || result.trim().is_empty());
    }

    #[test]
    fn test_render_special_characters_in_text() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("*_`[]<>#\\|")));
        TreeOps::append_child(&mut arena, para, text);
        TreeOps::append_child(&mut arena, root, para);

        let formatter = Formatter::new();
        let result = formatter.render(&arena, root);
        assert!(result.contains("\\*"));
        assert!(result.contains("\\_"));
        assert!(result.contains("\\`"));
    }

    #[test]
    fn test_render_unicode_text_cjk() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::make_text(
            "中文测试 日本語 한국어",
        )));
        TreeOps::append_child(&mut arena, para, text);
        TreeOps::append_child(&mut arena, root, para);

        let formatter = Formatter::new();
        let result = formatter.render(&arena, root);
        assert!(result.contains("中文测试"));
        assert!(result.contains("日本語"));
        assert!(result.contains("한국어"));
    }

    #[test]
    fn test_render_very_long_text() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let long_text = "a".repeat(10000);
        let text =
            arena.alloc(Node::with_value(NodeValue::make_text(long_text.clone())));
        TreeOps::append_child(&mut arena, para, text);
        TreeOps::append_child(&mut arena, root, para);

        let formatter = Formatter::new();
        let result = formatter.render(&arena, root);
        assert!(result.starts_with(&long_text));
        assert!(result.len() >= 10000);
    }

    #[test]
    fn test_render_deeply_nested_blockquote() {
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

        let formatter = Formatter::new();
        let result = formatter.render(&arena, root);
        assert!(result.contains("Deep"));
        assert!(result.contains(">"));
    }

    #[test]
    fn test_render_code_block_empty() {
        use crate::core::nodes::NodeCodeBlock;

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

        let formatter = Formatter::new();
        let result = formatter.render(&arena, root);
        assert!(result.contains("```"));
    }

    #[test]
    fn test_render_link_empty_url() {
        use crate::core::nodes::NodeLink;

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

        let formatter = Formatter::new();
        let result = formatter.render(&arena, root);
        assert!(result.contains("[empty link]()"));
    }
}
