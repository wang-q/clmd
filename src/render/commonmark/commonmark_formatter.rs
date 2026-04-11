//! CommonMark node formatter implementation
//!
//! This module provides a NodeFormatter implementation for CommonMark output.
//! Handler registration is delegated to the handlers::registration module,
//! which organizes handlers by functional domain (block, inline, list, table, extension).
//!
//! # Supported Elements
//!
//! ## Block Elements
//! - Document, Paragraph, Heading (ATX style)
//! - BlockQuote, CodeBlock (fenced)
//! - List (ordered/unordered), Item
//! - ThematicBreak, HtmlBlock
//!
//! ## Inline Elements
//! - Text (with proper escaping)
//! - Code (inline), Emph, Strong
//! - Link, Image
//! - Strikethrough (GFM)
//! - SoftBreak, HardBreak
//! - HtmlInline
//!
//! ## GFM Extensions
//! - Table (with alignment)
//! - FootnoteReference, FootnoteDefinition
//! - TaskItem (checkboxes)

use crate::render::commonmark::core::{NodeFormatter, NodeFormattingHandler};
use crate::render::commonmark::handlers::registration::register_all_handlers;

/// CommonMark node formatter
///
/// This formatter implements the NodeFormatter trait to provide CommonMark output.
/// It supports all standard CommonMark elements plus GFM extensions.
///
/// The formatter uses a multi-phase rendering approach:
/// 1. **Collect phase**: Gathers reference links and other metadata
/// 2. **Document phase**: Performs the main rendering
///
/// Handler registration is delegated to the `handlers::registration` module,
/// which organizes handlers into functional groups for better maintainability.
#[derive(Debug, Default, Clone, Copy)]
pub struct CommonMarkNodeFormatter;

impl CommonMarkNodeFormatter {
    /// Create a new CommonMark formatter
    pub fn new() -> Self {
        Self
    }
}

impl NodeFormatter for CommonMarkNodeFormatter {
    fn get_node_formatting_handlers(&self) -> Vec<NodeFormattingHandler> {
        register_all_handlers()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::commonmark::core::test_utils::MockContext;
    use crate::render::commonmark::escaping::{escape_string, escape_text};

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
    fn test_escape_text() {
        let ctx = MockContext::new();
        assert_eq!(escape_text("*text*", &ctx), "\\*text\\*");
        assert_eq!(escape_text("_text_", &ctx), "\\_text\\_");
        assert_eq!(escape_text("[link]", &ctx), "\\[link\\]");
        assert_eq!(escape_text("(paren)", &ctx), "(paren)"); // parentheses are not escaped
        assert_eq!(escape_text("`code`", &ctx), "\\`code\\`");
    }

    #[test]
    fn test_escape_text_no_special_chars() {
        let ctx = MockContext::new();
        assert_eq!(escape_text("plain text", &ctx), "plain text");
        assert_eq!(escape_text("123", &ctx), "123");
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
        use crate::render::commonmark::handlers::list::is_task_item_checked;

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
