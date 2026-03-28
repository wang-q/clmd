//! Tests for the CLMD library
//!
//! This module contains tests for the public API of the CLMD library.

use crate::{format_html, markdown_to_html, parse_document, version, Arena, Options};

#[test]
fn test_markdown_to_html_basic() {
    let options = Options::default();
    let html = markdown_to_html("Hello world", &options);
    println!("HTML output bytes: {:?}", html.as_bytes());
    assert_eq!(html, "<p>Hello world</p>");
}

#[test]
fn test_markdown_to_html_heading() {
    let options = Options::default();
    let html = markdown_to_html("# Heading 1\n\n## Heading 2", &options);
    assert!(html.contains("<h1>"));
    assert!(html.contains("<h2>"));
}

#[test]
fn test_markdown_to_html_emphasis() {
    let options = Options::default();
    let html = markdown_to_html("*italic* and **bold**", &options);
    assert!(html.contains("<em>italic</em>"));
    assert!(html.contains("<strong>bold</strong>"));
}

#[test]
fn test_markdown_to_html_link() {
    let options = Options::default();
    let html = markdown_to_html("[link](https://example.com)", &options);
    assert!(html.contains("<a href=\"https://example.com\">"));
}

#[test]
fn test_markdown_to_html_code_inline() {
    let options = Options::default();
    let html = markdown_to_html("Use `code` here", &options);
    assert!(html.contains("<code>code</code>"));
}

#[test]
fn test_markdown_to_html_code_block() {
    let options = Options::default();
    let html = markdown_to_html("```rust\nfn main() {}\n```", &options);
    assert!(html.contains("<pre>"));
    assert!(html.contains("<code"));
    assert!(html.contains("fn main() {}"));
}

#[test]
fn test_markdown_to_html_blockquote() {
    let options = Options::default();
    let html = markdown_to_html("> Quote", &options);
    assert!(html.contains("<blockquote>"));
    assert!(html.contains("Quote"));
}

#[test]
fn test_markdown_to_html_list() {
    let options = Options::default();
    let html = markdown_to_html("- Item 1\n- Item 2", &options);
    assert!(html.contains("<ul>"));
    assert!(html.contains("Item 1"));
    assert!(html.contains("Item 2"));
}

#[test]
fn test_markdown_to_html_ordered_list() {
    let options = Options::default();
    let html = markdown_to_html("1. First\n2. Second", &options);
    assert!(html.contains("<ol>"));
    assert!(html.contains("First"));
    assert!(html.contains("Second"));
}

#[test]
fn test_markdown_to_html_thematic_break() {
    let options = Options::default();
    let html = markdown_to_html("---", &options);
    assert!(html.contains("<hr"));
}

#[test]
fn test_markdown_to_html_image() {
    let options = Options::default();
    let html = markdown_to_html("![alt text](image.png)", &options);
    assert!(html.contains("<img"));
    assert!(html.contains("src=\"image.png\""));
    assert!(html.contains("alt=\"alt text\""));
}

#[test]
fn test_parse_and_render_roundtrip() {
    let options = Options::default();
    let input = "# Title\n\nParagraph with text.";
    let arena = Arena::new();
    let doc = parse_document(&arena, input, &options);
    let mut html = String::new();
    format_html(doc, &options, &mut html).unwrap();
    assert!(html.contains("<h1>"));
    assert!(html.contains("Paragraph"));
}

#[test]
fn test_version() {
    let v = version();
    assert!(!v.is_empty());
}
