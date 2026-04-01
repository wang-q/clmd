//! Tests for shortcodes extension

use crate::core::arena::{Node, NodeArena, TreeOps};
use crate::core::nodes::{NodeShortCode, NodeValue};
use crate::ext::shortcodes::parse_shortcode;
use crate::ext::shortcodes_data::lookup_shortcode;
use crate::render;

#[test]
fn test_lookup_shortcode() {
    assert_eq!(lookup_shortcode("+1"), Some("👍"));
    assert_eq!(lookup_shortcode("thumbsup"), Some("👍"));
    assert_eq!(lookup_shortcode("smile"), Some("😄"));
    assert_eq!(lookup_shortcode("heart"), Some("❤️"));
    assert_eq!(lookup_shortcode("nonexistent"), None);
}

#[test]
fn test_parse_shortcode_valid() {
    assert_eq!(parse_shortcode(":thumbsup:", 0), Some(("👍", 10)));
    assert_eq!(parse_shortcode(":smile:", 0), Some(("😄", 7)));
    assert_eq!(parse_shortcode(":+1:", 0), Some(("👍", 4)));
    assert_eq!(parse_shortcode(":heart:", 0), Some(("❤️", 7)));
}

#[test]
fn test_parse_shortcode_with_offset() {
    let text = "Hello :thumbsup: world";
    assert_eq!(parse_shortcode(text, 6), Some(("👍", 10)));
}

#[test]
fn test_parse_shortcode_invalid() {
    assert_eq!(parse_shortcode("not a shortcode", 0), None);
    assert_eq!(parse_shortcode(":", 0), None);
    assert_eq!(parse_shortcode(":a:", 0), None); // Too short
    assert_eq!(parse_shortcode(":invalid:", 0), None); // Unknown code
    assert_eq!(parse_shortcode(":no closing", 0), None);
}

#[test]
fn test_shortcode_html_rendering() {
    let mut arena = NodeArena::new();
    let root = arena.alloc(Node::with_value(NodeValue::Document));
    let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
    let text1 = arena.alloc(Node::with_value(NodeValue::make_text("Great job ")));
    let shortcode = arena.alloc(Node::with_value(NodeValue::ShortCode(Box::new(
        NodeShortCode {
            code: "thumbsup".to_string(),
            emoji: "👍".to_string(),
        },
    ))));
    let text2 = arena.alloc(Node::with_value(NodeValue::make_text("!")));

    TreeOps::append_child(&mut arena, root, para);
    TreeOps::append_child(&mut arena, para, text1);
    TreeOps::append_child(&mut arena, para, shortcode);
    TreeOps::append_child(&mut arena, para, text2);

    let html = render::html::render(&arena, root, 0);
    assert!(html.contains("👍"), "HTML should contain emoji: {}", html);
    assert!(
        !html.contains(":thumbsup:"),
        "HTML should not contain shortcode: {}",
        html
    );
}

#[test]
fn test_shortcode_commonmark_rendering() {
    let mut arena = NodeArena::new();
    let root = arena.alloc(Node::with_value(NodeValue::Document));
    let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
    let text1 = arena.alloc(Node::with_value(NodeValue::make_text("Great job ")));
    let shortcode = arena.alloc(Node::with_value(NodeValue::ShortCode(Box::new(
        NodeShortCode {
            code: "thumbsup".to_string(),
            emoji: "👍".to_string(),
        },
    ))));
    let text2 = arena.alloc(Node::with_value(NodeValue::make_text("!")));

    TreeOps::append_child(&mut arena, root, para);
    TreeOps::append_child(&mut arena, para, text1);
    TreeOps::append_child(&mut arena, para, shortcode);
    TreeOps::append_child(&mut arena, para, text2);

    let cm = render::commonmark::render(&arena, root, 0, 0);
    assert!(
        cm.contains(":thumbsup:"),
        "CommonMark should preserve shortcode: {}",
        cm
    );
}

#[test]
fn test_shortcode_xml_rendering() {
    let mut arena = NodeArena::new();
    let root = arena.alloc(Node::with_value(NodeValue::Document));
    let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
    let shortcode = arena.alloc(Node::with_value(NodeValue::ShortCode(Box::new(
        NodeShortCode {
            code: "thumbsup".to_string(),
            emoji: "👍".to_string(),
        },
    ))));

    TreeOps::append_child(&mut arena, root, para);
    TreeOps::append_child(&mut arena, para, shortcode);

    let xml = render::xml::render(&arena, root, 0);
    assert!(
        xml.contains("<shortcode"),
        "XML should contain shortcode tag: {}",
        xml
    );
    assert!(
        xml.contains("code=\"thumbsup\""),
        "XML should contain code attribute: {}",
        xml
    );
    assert!(xml.contains("👍"), "XML should contain emoji: {}", xml);
}

#[test]
fn test_multiple_shortcodes() {
    let mut arena = NodeArena::new();
    let root = arena.alloc(Node::with_value(NodeValue::Document));
    let para = arena.alloc(Node::with_value(NodeValue::Paragraph));

    let shortcode1 = arena.alloc(Node::with_value(NodeValue::ShortCode(Box::new(
        NodeShortCode {
            code: "smile".to_string(),
            emoji: "😄".to_string(),
        },
    ))));
    let shortcode2 = arena.alloc(Node::with_value(NodeValue::ShortCode(Box::new(
        NodeShortCode {
            code: "heart".to_string(),
            emoji: "❤️".to_string(),
        },
    ))));

    TreeOps::append_child(&mut arena, root, para);
    TreeOps::append_child(&mut arena, para, shortcode1);
    TreeOps::append_child(&mut arena, para, shortcode2);

    let html = render::html::render(&arena, root, 0);
    assert!(
        html.contains("😄"),
        "HTML should contain first emoji: {}",
        html
    );
    assert!(
        html.contains("❤️"),
        "HTML should contain second emoji: {}",
        html
    );
}

#[test]
fn test_shortcode_special_chars() {
    // Test shortcodes with + and - characters
    assert_eq!(parse_shortcode(":+1:", 0), Some(("👍", 4)));
    assert_eq!(parse_shortcode(":-1:", 0), Some(("👎", 4)));
    assert_eq!(lookup_shortcode("+1"), Some("👍"));
    assert_eq!(lookup_shortcode("-1"), Some("👎"));
}
