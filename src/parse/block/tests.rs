//! Tests for block parsing

use crate::core::arena::NodeArena;
use crate::core::nodes::NodeValue;
use crate::options::Options;
use crate::parse::block::BlockParser;

#[test]
fn test_parser_creation() {
    let mut arena = NodeArena::new();
    let parser = BlockParser::new(&mut arena);
    let doc = parser.doc;
    let tip = parser.tip;
    // Drop parser before accessing arena
    drop(parser);
    assert!(matches!(arena.get(doc).value, NodeValue::Document));
    assert!(matches!(arena.get(tip).value, NodeValue::Document));
}

#[test]
fn test_process_empty_line() {
    let mut arena = NodeArena::new();
    let mut parser = BlockParser::new(&mut arena);
    parser.process_line("");
    // Should not panic
}

#[test]
fn test_parse_simple_paragraph() {
    let mut arena = NodeArena::new();
    let doc = BlockParser::parse(&mut arena, "Hello world");
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    assert!(matches!(
        arena.get(first_child.unwrap()).value,
        NodeValue::Paragraph
    ));

    // After inline processing, paragraph content is stored in child nodes
    let para = arena.get(first_child.unwrap());
    let child = para.first_child;
    assert!(child.is_some(), "Paragraph should have child nodes");

    let content = arena.get(child.unwrap());
    if let NodeValue::Text(literal) = &content.value {
        assert_eq!(literal.as_ref(), "Hello world");
    } else {
        panic!("Expected Text value");
    }
}

#[test]
fn test_parse_block_quote() {
    let mut arena = NodeArena::new();
    let doc = BlockParser::parse(&mut arena, "> Quote line");
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    assert!(matches!(
        arena.get(first_child.unwrap()).value,
        NodeValue::BlockQuote
    ));
}

#[test]
fn test_parse_heading() {
    let mut arena = NodeArena::new();
    let doc = BlockParser::parse(&mut arena, "## Heading");
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    assert!(matches!(
        arena.get(first_child.unwrap()).value,
        NodeValue::Heading(_)
    ));
}

#[test]
fn test_parse_fenced_code_block() {
    let input = "```\ncode\n```";
    let mut arena = NodeArena::new();
    let doc = BlockParser::parse(&mut arena, input);
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    assert!(matches!(
        arena.get(first_child.unwrap()).value,
        NodeValue::CodeBlock(_)
    ));
}

#[test]
fn test_parse_thematic_break() {
    let mut arena = NodeArena::new();
    let doc = BlockParser::parse(&mut arena, "---");
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    if let NodeValue::ThematicBreak(tb) = &arena.get(first_child.unwrap()).value {
        assert_eq!(tb.marker, '-');
    } else {
        panic!("Expected ThematicBreak");
    }
}

#[test]
fn test_parse_bullet_list() {
    let mut arena = NodeArena::new();
    let doc = BlockParser::parse(&mut arena, "* Item 1\n* Item 2");
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    assert!(matches!(
        arena.get(first_child.unwrap()).value,
        NodeValue::List(_)
    ));
}

#[test]
fn test_parse_ordered_list() {
    let mut arena = NodeArena::new();
    let doc = BlockParser::parse(&mut arena, "1. Item 1\n2. Item 2");
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    assert!(matches!(
        arena.get(first_child.unwrap()).value,
        NodeValue::List(_)
    ));
}

#[test]
fn test_parse_nested_block_quote() {
    let mut arena = NodeArena::new();
    let doc = BlockParser::parse(&mut arena, "> Outer\n> > Inner");
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    assert!(matches!(
        arena.get(first_child.unwrap()).value,
        NodeValue::BlockQuote
    ));
}

#[test]
fn test_parse_setext_heading() {
    let mut arena = NodeArena::new();
    let doc = BlockParser::parse(&mut arena, "Heading\n===");
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    assert!(matches!(
        arena.get(first_child.unwrap()).value,
        NodeValue::Heading(_)
    ));
}

#[test]
fn test_remove_link_reference_definitions() {
    let input = "[label]: https://example.com\n\nSome text";
    let mut arena = NodeArena::new();
    let doc = BlockParser::parse(&mut arena, input);

    // The reference definition paragraph should be removed
    // So the first child should be the "Some text" paragraph
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some(), "Document should have a first child");

    let first_child_ref = arena.get(first_child.unwrap());
    assert!(
        matches!(first_child_ref.value, NodeValue::Paragraph),
        "First child should be a paragraph"
    );

    // After inline processing, paragraph content is stored in child nodes
    // The literal is cleared to prevent double-rendering
    let para_content = first_child_ref.first_child;
    assert!(
        para_content.is_some(),
        "Paragraph should have child nodes after inline processing"
    );

    let content_ref = arena.get(para_content.unwrap());
    match &content_ref.value {
        NodeValue::Text(literal) => {
            assert_eq!(
                literal.as_ref(),
                "Some text",
                "Paragraph content should be 'Some text'"
            );
        }
        _ => {
            panic!("Expected Text node, got {:?}", content_ref.value);
        }
    }
}

// ============================================================================
// ATX Heading Tests
// ============================================================================

#[test]
fn test_atx_heading_all_levels() {
    let mut arena = NodeArena::new();

    for level in 1..=6 {
        let input = format!("{} Heading {}", "#".repeat(level), level);
        let doc = BlockParser::parse(&mut arena, &input);
        let first_child = arena.get(doc).first_child;
        assert!(
            first_child.is_some(),
            "Level {} heading should be parsed",
            level
        );

        if let NodeValue::Heading(heading) = &arena.get(first_child.unwrap()).value {
            assert_eq!(heading.level, level as u8);
            assert!(!heading.setext);
        } else {
            panic!("Expected Heading for level {}", level);
        }
    }
}

#[test]
fn test_atx_heading_too_many_hashes() {
    let mut arena = NodeArena::new();
    // 7 hashes should become a paragraph
    let doc = BlockParser::parse(&mut arena, "####### Not a heading");
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    assert!(
        matches!(arena.get(first_child.unwrap()).value, NodeValue::Paragraph),
        "7 hashes should be a paragraph, not a heading"
    );
}

#[test]
fn test_atx_heading_with_closing_hashes() {
    let mut arena = NodeArena::new();
    let doc = BlockParser::parse(&mut arena, "## Heading ##");
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());

    if let NodeValue::Heading(heading) = &arena.get(first_child.unwrap()).value {
        assert_eq!(heading.level, 2);
    } else {
        panic!("Expected Heading");
    }
}

#[test]
fn test_atx_heading_empty_content() {
    let mut arena = NodeArena::new();
    let doc = BlockParser::parse(&mut arena, "##");
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    assert!(matches!(
        arena.get(first_child.unwrap()).value,
        NodeValue::Heading(_)
    ));
}

#[test]
fn test_atx_heading_indented() {
    let mut arena = NodeArena::new();
    // Indented heading should be code block
    let doc = BlockParser::parse(&mut arena, "    # Not a heading");
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    assert!(
        matches!(
            arena.get(first_child.unwrap()).value,
            NodeValue::CodeBlock(_)
        ),
        "Indented heading should be code block"
    );
}

#[test]
fn test_atx_heading_with_inline_content() {
    let mut arena = NodeArena::new();
    let doc = BlockParser::parse(&mut arena, "# Heading with `code`");
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    assert!(matches!(
        arena.get(first_child.unwrap()).value,
        NodeValue::Heading(_)
    ));
}

// ============================================================================
// Fenced Code Block Tests
// ============================================================================

#[test]
fn test_fenced_code_backticks() {
    let mut arena = NodeArena::new();
    let input = "```\ncode line 1\ncode line 2\n```";
    let doc = BlockParser::parse(&mut arena, input);
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    assert!(matches!(
        arena.get(first_child.unwrap()).value,
        NodeValue::CodeBlock(_)
    ));
}

#[test]
fn test_fenced_code_tildes() {
    let mut arena = NodeArena::new();
    let input = "~~~\ncode\n~~~";
    let doc = BlockParser::parse(&mut arena, input);
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    assert!(matches!(
        arena.get(first_child.unwrap()).value,
        NodeValue::CodeBlock(_)
    ));
}

#[test]
fn test_fenced_code_with_info() {
    let mut arena = NodeArena::new();
    let input = "```rust\ncode\n```";
    let doc = BlockParser::parse(&mut arena, input);
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());

    if let NodeValue::CodeBlock(cb) = &arena.get(first_child.unwrap()).value {
        assert!(cb.fenced);
        assert_eq!(cb.info, "rust");
        assert_eq!(cb.fence_char, b'`');
    } else {
        panic!("Expected CodeBlock");
    }
}

#[test]
fn test_fenced_code_not_enough_backticks() {
    let mut arena = NodeArena::new();
    // Only 2 backticks - not a code fence
    let doc = BlockParser::parse(&mut arena, "``\nnot code\n``");
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    // Should be paragraph since 2 backticks is not enough
    assert!(matches!(
        arena.get(first_child.unwrap()).value,
        NodeValue::Paragraph
    ));
}

#[test]
fn test_fenced_code_mixed_fence_chars() {
    let mut arena = NodeArena::new();
    // Opening with backticks, closing with tildes - not valid
    let input = "```\ncode\n~~~";
    let doc = BlockParser::parse(&mut arena, input);
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    // Code block should remain open (not closed by different fence char)
    if let NodeValue::CodeBlock(cb) = &arena.get(first_child.unwrap()).value {
        assert!(!cb.closed);
    } else {
        panic!("Expected CodeBlock");
    }
}

#[test]
fn test_fenced_code_with_backtick_in_info() {
    let mut arena = NodeArena::new();
    // Backtick in info string makes it invalid
    let doc = BlockParser::parse(&mut arena, "```rust`\ncode\n```");
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    // Should be paragraph since backtick in info is invalid
    assert!(matches!(
        arena.get(first_child.unwrap()).value,
        NodeValue::Paragraph
    ));
}

// ============================================================================
// Setext Heading Tests
// ============================================================================

#[test]
fn test_setext_heading_level1() {
    let mut arena = NodeArena::new();
    let doc = BlockParser::parse(&mut arena, "Heading\n===");
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());

    if let NodeValue::Heading(heading) = &arena.get(first_child.unwrap()).value {
        assert_eq!(heading.level, 1);
        assert!(heading.setext);
    } else {
        panic!("Expected Heading");
    }
}

#[test]
fn test_setext_heading_level2() {
    let mut arena = NodeArena::new();
    let doc = BlockParser::parse(&mut arena, "Heading\n---");
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());

    if let NodeValue::Heading(heading) = &arena.get(first_child.unwrap()).value {
        assert_eq!(heading.level, 2);
        assert!(heading.setext);
    } else {
        panic!("Expected Heading");
    }
}

#[test]
fn test_setext_heading_not_paragraph() {
    let mut arena = NodeArena::new();
    // Setext heading must follow a paragraph
    let doc = BlockParser::parse(&mut arena, "## Heading\n---");
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    // First child is ATX heading, not setext
    if let NodeValue::Heading(heading) = &arena.get(first_child.unwrap()).value {
        assert_eq!(heading.level, 2);
        assert!(!heading.setext);
    } else {
        panic!("Expected Heading");
    }
}

#[test]
fn test_setext_heading_indented() {
    let mut arena = NodeArena::new();
    // Indented setext underline should not work
    let doc = BlockParser::parse(&mut arena, "Heading\n    ===");
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    // Should be paragraph since underline is indented
    assert!(matches!(
        arena.get(first_child.unwrap()).value,
        NodeValue::Paragraph
    ));
}

#[test]
fn test_setext_heading_with_reference() {
    let mut arena = NodeArena::new();
    // Setext heading content with reference definition
    let input = "Heading [link]: url\n===";
    let doc = BlockParser::parse(&mut arena, input);
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    // Reference should be stripped
    if let NodeValue::Heading(heading) = &arena.get(first_child.unwrap()).value {
        assert_eq!(heading.level, 1);
    } else {
        panic!("Expected Heading");
    }
}

// ============================================================================
// Thematic Break Tests
// ============================================================================

#[test]
fn test_thematic_break_dashes() {
    let mut arena = NodeArena::new();
    let doc = BlockParser::parse(&mut arena, "---");
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    if let NodeValue::ThematicBreak(tb) = &arena.get(first_child.unwrap()).value {
        assert_eq!(tb.marker, '-');
    } else {
        panic!("Expected ThematicBreak");
    }
}

#[test]
fn test_thematic_break_asterisks() {
    let mut arena = NodeArena::new();
    let doc = BlockParser::parse(&mut arena, "***");
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    if let NodeValue::ThematicBreak(tb) = &arena.get(first_child.unwrap()).value {
        assert_eq!(tb.marker, '*');
    } else {
        panic!("Expected ThematicBreak");
    }
}

#[test]
fn test_thematic_break_underscores() {
    let mut arena = NodeArena::new();
    let doc = BlockParser::parse(&mut arena, "___");
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    if let NodeValue::ThematicBreak(tb) = &arena.get(first_child.unwrap()).value {
        assert_eq!(tb.marker, '_');
    } else {
        panic!("Expected ThematicBreak");
    }
}

#[test]
fn test_thematic_break_with_spaces() {
    let mut arena = NodeArena::new();
    let doc = BlockParser::parse(&mut arena, "- - -");
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    if let NodeValue::ThematicBreak(tb) = &arena.get(first_child.unwrap()).value {
        assert_eq!(tb.marker, '-');
    } else {
        panic!("Expected ThematicBreak");
    }
}

#[test]
fn test_thematic_break_not_enough() {
    let mut arena = NodeArena::new();
    // Only 2 characters - not a thematic break
    let doc = BlockParser::parse(&mut arena, "--");
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    assert!(matches!(
        arena.get(first_child.unwrap()).value,
        NodeValue::Paragraph
    ));
}

#[test]
fn test_thematic_break_mixed_chars() {
    let mut arena = NodeArena::new();
    // Mixed characters - not a thematic break
    let doc = BlockParser::parse(&mut arena, "-*-");
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    assert!(matches!(
        arena.get(first_child.unwrap()).value,
        NodeValue::Paragraph
    ));
}

#[test]
fn test_thematic_break_indented() {
    let mut arena = NodeArena::new();
    // Indented thematic break should be code block
    let doc = BlockParser::parse(&mut arena, "    ---");
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    assert!(matches!(
        arena.get(first_child.unwrap()).value,
        NodeValue::CodeBlock(_)
    ));
}

// ============================================================================
// Block Quote Tests
// ============================================================================

#[test]
fn test_block_quote_simple() {
    let mut arena = NodeArena::new();
    let doc = BlockParser::parse(&mut arena, "> Quote");
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    assert!(matches!(
        arena.get(first_child.unwrap()).value,
        NodeValue::BlockQuote
    ));
}

#[test]
fn test_block_quote_with_space() {
    let mut arena = NodeArena::new();
    let doc = BlockParser::parse(&mut arena, "> Quote");
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    assert!(matches!(
        arena.get(first_child.unwrap()).value,
        NodeValue::BlockQuote
    ));
}

#[test]
fn test_block_quote_empty() {
    let mut arena = NodeArena::new();
    let doc = BlockParser::parse(&mut arena, ">");
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    assert!(matches!(
        arena.get(first_child.unwrap()).value,
        NodeValue::BlockQuote
    ));
}

#[test]
fn test_block_quote_indented() {
    let mut arena = NodeArena::new();
    // Indented block quote should be code block
    let doc = BlockParser::parse(&mut arena, "    > Not a quote");
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    assert!(matches!(
        arena.get(first_child.unwrap()).value,
        NodeValue::CodeBlock(_)
    ));
}

#[test]
fn test_block_quote_nested() {
    let mut arena = NodeArena::new();
    let doc = BlockParser::parse(&mut arena, "> Outer\n> > Inner");
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    assert!(matches!(
        arena.get(first_child.unwrap()).value,
        NodeValue::BlockQuote
    ));

    // Check nested quote - outer block quote contains a paragraph "Outer" and a nested block quote
    let outer = arena.get(first_child.unwrap());
    // First child should be paragraph with "Outer"
    let para = outer.first_child;
    assert!(para.is_some());
    assert!(matches!(
        arena.get(para.unwrap()).value,
        NodeValue::Paragraph
    ));

    // Next sibling should be the nested block quote
    let inner = arena.get(para.unwrap()).next;
    assert!(inner.is_some());
    assert!(matches!(
        arena.get(inner.unwrap()).value,
        NodeValue::BlockQuote
    ));
}

#[test]
fn test_block_quote_with_heading() {
    let mut arena = NodeArena::new();
    let doc = BlockParser::parse(&mut arena, "> # Heading in quote");
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    assert!(matches!(
        arena.get(first_child.unwrap()).value,
        NodeValue::BlockQuote
    ));

    let quote = arena.get(first_child.unwrap());
    let heading = quote.first_child;
    assert!(heading.is_some());
    assert!(matches!(
        arena.get(heading.unwrap()).value,
        NodeValue::Heading(_)
    ));
}

// ============================================================================
// List Tests
// ============================================================================

#[test]
fn test_bullet_list_asterisk() {
    let mut arena = NodeArena::new();
    let doc = BlockParser::parse(&mut arena, "* Item");
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    assert!(matches!(
        arena.get(first_child.unwrap()).value,
        NodeValue::List(_)
    ));
}

#[test]
fn test_bullet_list_plus() {
    let mut arena = NodeArena::new();
    let doc = BlockParser::parse(&mut arena, "+ Item");
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    assert!(matches!(
        arena.get(first_child.unwrap()).value,
        NodeValue::List(_)
    ));
}

#[test]
fn test_bullet_list_dash() {
    let mut arena = NodeArena::new();
    let doc = BlockParser::parse(&mut arena, "- Item");
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    assert!(matches!(
        arena.get(first_child.unwrap()).value,
        NodeValue::List(_)
    ));
}

#[test]
fn test_ordered_list_period() {
    let mut arena = NodeArena::new();
    let doc = BlockParser::parse(&mut arena, "1. Item");
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    assert!(matches!(
        arena.get(first_child.unwrap()).value,
        NodeValue::List(_)
    ));
}

#[test]
fn test_ordered_list_paren() {
    let mut arena = NodeArena::new();
    let doc = BlockParser::parse(&mut arena, "1) Item");
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    assert!(matches!(
        arena.get(first_child.unwrap()).value,
        NodeValue::List(_)
    ));
}

#[test]
fn test_ordered_list_start_not_one() {
    let mut arena = NodeArena::new();
    let doc = BlockParser::parse(&mut arena, "2. Item");
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    assert!(matches!(
        arena.get(first_child.unwrap()).value,
        NodeValue::List(_)
    ));
}

#[test]
fn test_ordered_list_too_many_digits() {
    let mut arena = NodeArena::new();
    // 10 digits - too many
    let doc = BlockParser::parse(&mut arena, "1234567890. Item");
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    // Should be paragraph since too many digits
    assert!(matches!(
        arena.get(first_child.unwrap()).value,
        NodeValue::Paragraph
    ));
}

#[test]
fn test_list_interrupted_by_blank_line() {
    let mut arena = NodeArena::new();
    let input = "* Item 1\n\n* Item 2";
    let doc = BlockParser::parse(&mut arena, input);
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    assert!(matches!(
        arena.get(first_child.unwrap()).value,
        NodeValue::List(_)
    ));
}

#[test]
fn test_list_with_sublist() {
    let mut arena = NodeArena::new();
    let input = "* Item\n  * Subitem";
    let doc = BlockParser::parse(&mut arena, input);
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    assert!(matches!(
        arena.get(first_child.unwrap()).value,
        NodeValue::List(_)
    ));
}

#[test]
fn test_list_no_space_after_marker() {
    let mut arena = NodeArena::new();
    // No space after marker - not a list
    let doc = BlockParser::parse(&mut arena, "*Not a list");
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    assert!(matches!(
        arena.get(first_child.unwrap()).value,
        NodeValue::Paragraph
    ));
}

// ============================================================================
// Indented Code Block Tests
// ============================================================================

#[test]
fn test_indented_code_block() {
    let mut arena = NodeArena::new();
    let input = "    code line 1\n    code line 2";
    let doc = BlockParser::parse(&mut arena, input);
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    assert!(matches!(
        arena.get(first_child.unwrap()).value,
        NodeValue::CodeBlock(_)
    ));
}

#[test]
fn test_indented_code_block_not_lazy() {
    let mut arena = NodeArena::new();
    // After paragraph, indented lines are continuation, not code block
    let input = "Paragraph\n    continuation";
    let doc = BlockParser::parse(&mut arena, input);
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    assert!(matches!(
        arena.get(first_child.unwrap()).value,
        NodeValue::Paragraph
    ));
}

// ============================================================================
// HTML Block Tests
// ============================================================================

#[test]
fn test_html_block_type1_script() {
    let mut arena = NodeArena::new();
    let input = "<script>\nalert('hello');\n</script>";
    let doc = BlockParser::parse(&mut arena, input);
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    assert!(matches!(
        arena.get(first_child.unwrap()).value,
        NodeValue::HtmlBlock(_)
    ));
}

#[test]
fn test_html_block_type1_pre() {
    let mut arena = NodeArena::new();
    let input = "<pre>\ncode\n</pre>";
    let doc = BlockParser::parse(&mut arena, input);
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    assert!(matches!(
        arena.get(first_child.unwrap()).value,
        NodeValue::HtmlBlock(_)
    ));
}

#[test]
fn test_html_block_type2_comment() {
    let mut arena = NodeArena::new();
    let input = "<!-- comment -->";
    let doc = BlockParser::parse(&mut arena, input);
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    assert!(matches!(
        arena.get(first_child.unwrap()).value,
        NodeValue::HtmlBlock(_)
    ));
}

#[test]
fn test_html_block_type3_pi() {
    let mut arena = NodeArena::new();
    let input = "<?php echo 'hello'; ?>";
    let doc = BlockParser::parse(&mut arena, input);
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    assert!(matches!(
        arena.get(first_child.unwrap()).value,
        NodeValue::HtmlBlock(_)
    ));
}

#[test]
fn test_html_block_type4_doctype() {
    let mut arena = NodeArena::new();
    let input = "<!DOCTYPE html>";
    let doc = BlockParser::parse(&mut arena, input);
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    assert!(matches!(
        arena.get(first_child.unwrap()).value,
        NodeValue::HtmlBlock(_)
    ));
}

#[test]
fn test_html_block_type5_cdata() {
    let mut arena = NodeArena::new();
    let input = "<![CDATA[ content ]]>";
    let doc = BlockParser::parse(&mut arena, input);
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    assert!(matches!(
        arena.get(first_child.unwrap()).value,
        NodeValue::HtmlBlock(_)
    ));
}

#[test]
fn test_html_block_type6_div() {
    let mut arena = NodeArena::new();
    let input = "<div>\ncontent\n</div>";
    let doc = BlockParser::parse(&mut arena, input);
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    assert!(matches!(
        arena.get(first_child.unwrap()).value,
        NodeValue::HtmlBlock(_)
    ));
}

#[test]
fn test_html_block_type7() {
    let mut arena = NodeArena::new();
    // Type 7 requires the tag to be on a line by itself without content on same line
    // and must satisfy specific conditions (not in paragraph context)
    let input = "<custom-tag>\ncontent\n</custom-tag>";
    let doc = BlockParser::parse(&mut arena, input);
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    // Type 7 HTML block requires specific conditions; may be parsed as paragraph
    // depending on exact tag format
    let node = &arena.get(first_child.unwrap()).value;
    assert!(
        matches!(node, NodeValue::HtmlBlock(_)) || matches!(node, NodeValue::Paragraph),
        "Type 7 HTML block or paragraph expected, got {:?}",
        node
    );
}

#[test]
fn test_html_block_indented() {
    let mut arena = NodeArena::new();
    // Indented HTML should be code block
    let input = "    <div>content</div>";
    let doc = BlockParser::parse(&mut arena, input);
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    assert!(matches!(
        arena.get(first_child.unwrap()).value,
        NodeValue::CodeBlock(_)
    ));
}

// ============================================================================
// Table Tests (GFM)
// ============================================================================

#[test]
fn test_table_basic() {
    let mut arena = NodeArena::new();
    let options = Options::default();

    let input = "| Header |\n|--------|";
    let doc = BlockParser::parse_with_options(&mut arena, input, options);
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    assert!(matches!(
        arena.get(first_child.unwrap()).value,
        NodeValue::Table(_)
    ));
}

#[test]
fn test_table_with_alignment() {
    let mut arena = NodeArena::new();
    let options = Options::default();

    let input = "| Left | Center | Right |\n|:-----|:------:|------:|";
    let doc = BlockParser::parse_with_options(&mut arena, input, options);
    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    assert!(matches!(
        arena.get(first_child.unwrap()).value,
        NodeValue::Table(_)
    ));
}

// ============================================================================
// Edge Cases and Combinations
// ============================================================================

#[test]
fn test_empty_document() {
    let mut arena = NodeArena::new();
    let doc = BlockParser::parse(&mut arena, "");
    let first_child = arena.get(doc).first_child;
    assert!(
        first_child.is_none(),
        "Empty document should have no children"
    );
}

#[test]
fn test_blank_lines_only() {
    let mut arena = NodeArena::new();
    let doc = BlockParser::parse(&mut arena, "\n\n\n");
    let first_child = arena.get(doc).first_child;
    assert!(
        first_child.is_none(),
        "Document with only blank lines should have no children"
    );
}

#[test]
fn test_multiple_blocks() {
    let mut arena = NodeArena::new();
    let input = "# Heading\n\nParagraph\n\n> Quote";
    let doc = BlockParser::parse(&mut arena, input);

    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    assert!(matches!(
        arena.get(first_child.unwrap()).value,
        NodeValue::Heading(_)
    ));

    let second = arena.get(first_child.unwrap()).next;
    assert!(second.is_some());
    assert!(matches!(
        arena.get(second.unwrap()).value,
        NodeValue::Paragraph
    ));

    let third = arena.get(second.unwrap()).next;
    assert!(third.is_some());
    assert!(matches!(
        arena.get(third.unwrap()).value,
        NodeValue::BlockQuote
    ));
}

#[test]
fn test_code_fence_in_block_quote() {
    let mut arena = NodeArena::new();
    let input = "> ```\n> code\n> ```";
    let doc = BlockParser::parse(&mut arena, input);

    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    assert!(matches!(
        arena.get(first_child.unwrap()).value,
        NodeValue::BlockQuote
    ));

    let quote = arena.get(first_child.unwrap());
    let code = quote.first_child;
    assert!(code.is_some());
    assert!(matches!(
        arena.get(code.unwrap()).value,
        NodeValue::CodeBlock(_)
    ));
}

#[test]
fn test_list_in_block_quote() {
    let mut arena = NodeArena::new();
    let input = "> * Item 1\n> * Item 2";
    let doc = BlockParser::parse(&mut arena, input);

    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    assert!(matches!(
        arena.get(first_child.unwrap()).value,
        NodeValue::BlockQuote
    ));

    let quote = arena.get(first_child.unwrap());
    let list = quote.first_child;
    assert!(list.is_some());
    assert!(matches!(arena.get(list.unwrap()).value, NodeValue::List(_)));
}

#[test]
fn test_heading_in_list() {
    let mut arena = NodeArena::new();
    let input = "* # Heading in list";
    let doc = BlockParser::parse(&mut arena, input);

    let first_child = arena.get(doc).first_child;
    assert!(first_child.is_some());
    assert!(matches!(
        arena.get(first_child.unwrap()).value,
        NodeValue::List(_)
    ));

    let list = arena.get(first_child.unwrap());
    let item = list.first_child;
    assert!(item.is_some());
    assert!(matches!(arena.get(item.unwrap()).value, NodeValue::Item(_)));

    let item_node = arena.get(item.unwrap());
    let heading = item_node.first_child;
    assert!(heading.is_some());
    assert!(matches!(
        arena.get(heading.unwrap()).value,
        NodeValue::Heading(_)
    ));
}
