//! Tests for block parsing

use crate::arena::NodeArena;
use crate::blocks::BlockParser;
use crate::nodes::NodeValue;

#[test]
fn test_parser_creation() {
    let mut arena = NodeArena::new();
    let parser = BlockParser::new(&mut arena);
    let doc = parser.doc;
    let tip = parser.tip;
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
    assert!(matches!(
        arena.get(first_child.unwrap()).value,
        NodeValue::ThematicBreak
    ));
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
                literal.as_ref(), "Some text",
                "Paragraph content should be 'Some text'"
            );
        }
        _ => {
            panic!("Expected Text node, got {:?}", content_ref.value);
        }
    }
}
