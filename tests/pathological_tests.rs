//! Pathological Tests
//!
//! Tests for extreme input cases that could cause performance issues
//! or stack overflows. Based on cmark's pathological_tests.py.

use clmd::markdown_to_html;
use clmd::parse::options::Options;
use std::time::{Duration, Instant};

const TIMEOUT: Duration = Duration::from_secs(5);

/// Helper function to convert markdown to HTML with default options
fn md_to_html(input: &str) -> String {
    markdown_to_html(input, &Options::default())
}

/// Helper function to run a test with timeout
fn run_with_timeout<F>(test_fn: F, description: &str) -> Result<String, String>
where
    F: FnOnce() -> String,
{
    let start = Instant::now();
    let result = test_fn();
    let elapsed = start.elapsed();

    if elapsed > TIMEOUT {
        Err(format!("{} [TIMEOUT after {:?}]", description, elapsed))
    } else {
        Ok(result)
    }
}

/// Test nested strong emphasis
/// Pattern: *a **a * repeated many times
#[test]
#[ignore] // Heavy test, run with --ignored
fn test_nested_strong_emph() {
    let input = ("*a **a ".repeat(1000)) + "b" + &(" a** a*".repeat(1000));

    let result = run_with_timeout(|| md_to_html(&input), "nested strong emph");

    assert!(result.is_ok(), "{}", result.unwrap_err());
    let html = result.unwrap();
    // Should complete without stack overflow
    assert!(html.contains("<p>") || html.contains("<em>"));
}

/// Test many emph closers with no openers
#[test]
fn test_many_emph_closers() {
    let input = "a_ ".repeat(1000);

    let result = run_with_timeout(|| md_to_html(&input), "many emph closers");

    assert!(result.is_ok(), "{}", result.unwrap_err());
    let html = result.unwrap();
    assert!(html.contains("<p>"));
}

/// Test many emph openers with no closers
#[test]
fn test_many_emph_openers() {
    let input = "_a ".repeat(1000);

    let result = run_with_timeout(|| md_to_html(&input), "many emph openers");

    assert!(result.is_ok(), "{}", result.unwrap_err());
    let html = result.unwrap();
    assert!(html.contains("<p>"));
}

/// Test deeply nested brackets
#[test]
fn test_deeply_nested_brackets() {
    let input = "[".repeat(1000) + "text" + &"]".repeat(1000);

    let result = run_with_timeout(|| md_to_html(&input), "deeply nested brackets");

    assert!(result.is_ok(), "{}", result.unwrap_err());
    let html = result.unwrap();
    assert!(html.contains("<p>"));
}

/// Test deeply nested parentheses in links
#[test]
fn test_deeply_nested_parens() {
    let input = "(".repeat(1000) + "text" + &")".repeat(1000);

    let result = run_with_timeout(|| md_to_html(&input), "deeply nested parens");

    assert!(result.is_ok(), "{}", result.unwrap_err());
    let html = result.unwrap();
    assert!(html.contains("<p>"));
}

/// Test deeply nested blockquotes
#[test]
#[ignore] // Heavy test, run with --ignored
fn test_deeply_nested_blockquotes() {
    let input = "> ".repeat(1000) + "text\n";

    let result = run_with_timeout(|| md_to_html(&input), "deeply nested blockquotes");

    assert!(result.is_ok(), "{}", result.unwrap_err());
    let html = result.unwrap();
    assert!(html.contains("<blockquote>"));
}

/// Test deeply nested lists
#[test]
#[ignore] // Heavy test, run with --ignored
fn test_deeply_nested_lists() {
    let mut input = String::new();
    for i in 1..=100 {
        input.push_str(&"  ".repeat(i));
        input.push_str("- item\n");
    }

    let result = run_with_timeout(|| md_to_html(&input), "deeply nested lists");

    assert!(result.is_ok(), "{}", result.unwrap_err());
    let html = result.unwrap();
    assert!(html.contains("<ul>"));
}

/// Test long line
#[test]
fn test_long_line() {
    let input = "a".repeat(100000);

    let result = run_with_timeout(|| md_to_html(&input), "long line");

    assert!(result.is_ok(), "{}", result.unwrap_err());
    let html = result.unwrap();
    assert!(html.contains("<p>"));
}

/// Test many blank lines
#[test]
fn test_many_blank_lines() {
    let input = "\n".repeat(10000);

    let result = run_with_timeout(|| md_to_html(&input), "many blank lines");

    assert!(result.is_ok(), "{}", result.unwrap_err());
}

/// Test many inline backticks
#[test]
fn test_many_inline_backticks() {
    let input = "`code` ".repeat(10000);

    let result = run_with_timeout(|| md_to_html(&input), "many inline backticks");

    assert!(result.is_ok(), "{}", result.unwrap_err());
    let html = result.unwrap();
    assert!(html.contains("<code>"));
}

/// Test many autolinks
#[test]
fn test_many_autolinks() {
    let input = "<http://example.com> ".repeat(10000);

    let result = run_with_timeout(|| md_to_html(&input), "many autolinks");

    assert!(result.is_ok(), "{}", result.unwrap_err());
    let html = result.unwrap();
    assert!(html.contains("<a href"));
}

/// Test many HTML entities
#[test]
fn test_many_entities() {
    let input = "&amp; ".repeat(10000);

    let result = run_with_timeout(|| md_to_html(&input), "many entities");

    assert!(result.is_ok(), "{}", result.unwrap_err());
    let html = result.unwrap();
    assert!(html.contains("&amp;"));
}

/// Test many hard line breaks
#[test]
fn test_many_hard_breaks() {
    let input = "line  \n".repeat(10000);

    let result = run_with_timeout(|| md_to_html(&input), "many hard breaks");

    assert!(result.is_ok(), "{}", result.unwrap_err());
    let html = result.unwrap();
    assert!(html.contains("<br"));
}

/// Test emphasis with many asterisks
#[test]
fn test_many_asterisks() {
    let input = "*".repeat(10000);

    let result = run_with_timeout(|| md_to_html(&input), "many asterisks");

    assert!(result.is_ok(), "{}", result.unwrap_err());
    // Just verify it completes without panic - the output may vary
}

/// Test link reference definitions
#[test]
fn test_many_link_refs() {
    let mut input = String::new();
    for i in 0..1000 {
        input.push_str(&format!("[ref{}]: /url{}\n", i, i));
    }
    input.push_str("[ref0]\n");

    let result = run_with_timeout(|| md_to_html(&input), "many link refs");

    assert!(result.is_ok(), "{}", result.unwrap_err());
    let html = result.unwrap();
    assert!(html.contains("<a href"));
}

/// Test large document
#[test]
#[ignore] // Heavy test, run with --ignored
fn test_large_document() {
    let mut input = String::new();
    for _ in 0..1000 {
        input.push_str("# Heading\n\nParagraph with **bold** and *italic* text.\n\n");
        input.push_str("- List item 1\n- List item 2\n- List item 3\n\n");
        input.push_str("> Blockquote\n\n");
        input.push_str("```\ncode block\n```\n\n");
    }

    let result = run_with_timeout(|| md_to_html(&input), "large document");

    assert!(result.is_ok(), "{}", result.unwrap_err());
    let html = result.unwrap();
    assert!(html.contains("<h1>"));
    assert!(html.contains("<strong>"));
    assert!(html.contains("<em>"));
}
