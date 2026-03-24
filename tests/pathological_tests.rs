//! Pathological Tests
//!
//! Tests for extreme input cases that could cause performance issues
//! or stack overflows. Based on cmark's pathological_tests.py.

use clmd::{markdown_to_html, options};
use std::time::{Duration, Instant};

const TIMEOUT: Duration = Duration::from_secs(5);

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

    let result = run_with_timeout(
        || markdown_to_html(&input, options::DEFAULT),
        "nested strong emph",
    );

    assert!(result.is_ok(), "{}", result.unwrap_err());
    let html = result.unwrap();
    // Should complete without stack overflow
    assert!(html.contains("<p>") || html.contains("<em>"));
}

/// Test many emph closers with no openers
#[test]
fn test_many_emph_closers() {
    let input = "a_ ".repeat(1000);

    let result = run_with_timeout(
        || markdown_to_html(&input, options::DEFAULT),
        "many emph closers",
    );

    assert!(result.is_ok(), "{}", result.unwrap_err());
}

/// Test many emph openers with no closers
#[test]
fn test_many_emph_openers() {
    let input = "_a ".repeat(1000);

    let result = run_with_timeout(
        || markdown_to_html(&input, options::DEFAULT),
        "many emph openers",
    );

    assert!(result.is_ok(), "{}", result.unwrap_err());
}

/// Test many link closers with no openers
#[test]
fn test_many_link_closers() {
    let input = "a]".repeat(1000);

    let result = run_with_timeout(
        || markdown_to_html(&input, options::DEFAULT),
        "many link closers",
    );

    assert!(result.is_ok(), "{}", result.unwrap_err());
}

/// Test many link openers with no closers
#[test]
fn test_many_link_openers() {
    let input = "[a".repeat(1000);

    let result = run_with_timeout(
        || markdown_to_html(&input, options::DEFAULT),
        "many link openers",
    );

    assert!(result.is_ok(), "{}", result.unwrap_err());
}

/// Test mismatched openers and closers
#[test]
fn test_mismatched_openers_closers() {
    let input = "*a_ ".repeat(500);

    let result = run_with_timeout(
        || markdown_to_html(&input, options::DEFAULT),
        "mismatched openers and closers",
    );

    assert!(result.is_ok(), "{}", result.unwrap_err());
}

/// Test nested brackets
#[test]
#[ignore] // Heavy test
fn test_nested_brackets() {
    let input = "[".repeat(1000) + "a" + &"]".repeat(1000);

    let result = run_with_timeout(
        || markdown_to_html(&input, options::DEFAULT),
        "nested brackets",
    );

    assert!(result.is_ok(), "{}", result.unwrap_err());
}

/// Test nested block quotes
#[test]
#[ignore] // Heavy test
fn test_nested_block_quotes() {
    let input = "> ".repeat(1000) + "a";

    let result = run_with_timeout(
        || markdown_to_html(&input, options::DEFAULT),
        "nested block quotes",
    );

    assert!(result.is_ok(), "{}", result.unwrap_err());
}

/// Test deeply nested lists
#[test]
#[ignore] // Heavy test
fn test_deeply_nested_lists() {
    let mut input = String::new();
    for i in 0..100 {
        input.push_str(&"  ".repeat(i));
        input.push_str("* a\n");
    }

    let result = run_with_timeout(
        || markdown_to_html(&input, options::DEFAULT),
        "deeply nested lists",
    );

    assert!(result.is_ok(), "{}", result.unwrap_err());
}

/// Test backticks pattern
#[test]
fn test_backticks() {
    let mut input = String::new();
    for i in 1..500 {
        input.push('e');
        input.push_str(&"`".repeat(i));
    }

    let result =
        run_with_timeout(|| markdown_to_html(&input, options::DEFAULT), "backticks");

    assert!(result.is_ok(), "{}", result.unwrap_err());
}

/// Test unclosed links A
#[test]
#[ignore] // Heavy test
fn test_unclosed_links_a() {
    let input = "[a](<b".repeat(1000);

    let result = run_with_timeout(
        || markdown_to_html(&input, options::DEFAULT),
        "unclosed links A",
    );

    assert!(result.is_ok(), "{}", result.unwrap_err());
}

/// Test unclosed links B
#[test]
#[ignore] // Heavy test
fn test_unclosed_links_b() {
    let input = "[a](b".repeat(1000);

    let result = run_with_timeout(
        || markdown_to_html(&input, options::DEFAULT),
        "unclosed links B",
    );

    assert!(result.is_ok(), "{}", result.unwrap_err());
}

/// Test unclosed HTML comments
#[test]
#[ignore] // Heavy test
fn test_unclosed_comments() {
    let input = "</".to_string() + &"<!--".repeat(10000);

    let result = run_with_timeout(
        || markdown_to_html(&input, options::DEFAULT),
        "unclosed comments",
    );

    assert!(result.is_ok(), "{}", result.unwrap_err());
}

/// Test empty lines in deeply nested lists
#[test]
#[ignore] // Heavy test
fn test_empty_lines_nested_lists() {
    let input = "- ".repeat(1000) + "x" + &"\n".repeat(1000);

    let result = run_with_timeout(
        || markdown_to_html(&input, options::DEFAULT),
        "empty lines in nested lists",
    );

    assert!(result.is_ok(), "{}", result.unwrap_err());
}

/// Test U+0000 in input
#[test]
fn test_null_character() {
    let input = "abc\u{0000}de\u{0000}";

    let result = markdown_to_html(input, options::DEFAULT);
    // Should handle null characters without crashing
    assert!(result.contains("abc") || result.contains("\u{FFFD}"));
}

/// Test normal input for comparison
#[test]
fn test_normal_input() {
    let input =
        "# Hello\n\nThis is a **test** paragraph with [a link](http://example.com).";

    let start = Instant::now();
    let result = markdown_to_html(input, options::DEFAULT);
    let elapsed = start.elapsed();

    assert!(result.contains("<h1>"));
    assert!(result.contains("<strong>"));
    assert!(result.contains("<a href"));
    assert!(
        elapsed < Duration::from_millis(100),
        "Normal input should be fast"
    );
}

/// Run all pathological tests with shorter versions for CI
#[test]
fn test_pathological_quick() {
    // Quick versions of pathological tests for regular CI runs
    let tests = vec![
        ("emph closers", "a_ ".repeat(100)),
        ("emph openers", "_a ".repeat(100)),
        ("link closers", "a]".repeat(100)),
        ("link openers", "[a".repeat(100)),
        ("mismatched", "*a_ ".repeat(50)),
    ];

    for (name, input) in tests {
        let start = Instant::now();
        let _result = markdown_to_html(&input, options::DEFAULT);
        let elapsed = start.elapsed();

        assert!(
            elapsed < Duration::from_secs(1),
            "{} test took too long: {:?}",
            name,
            elapsed
        );
    }
}
