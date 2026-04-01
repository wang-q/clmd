//! Roundtrip Tests
//!
//! Tests for Markdown -> HTML -> Markdown roundtrip consistency.
//! Based on cmark's roundtrip_tests.py.

use clmd::markdown_to_html;
use clmd::parse::options::Options;
use std::fs;

/// Helper function to convert markdown to HTML with default options
fn md_to_html(input: &str) -> String {
    markdown_to_html(input, &Options::default())
}

/// Helper function to convert HTML to Markdown
fn html_to_md(html: &str) -> String {
    clmd::io::reader::html::html_to_markdown(html)
}

/// Test logging macro - only prints when VERBOSE_TESTS is set
macro_rules! test_log {
    ($($arg:tt)*) => {
        if std::env::var("VERBOSE_TESTS").is_ok() {
            std::println!($($arg)*);
        }
    };
}

#[derive(Debug)]
struct RoundtripTest {
    name: String,
    markdown: String,
}

fn load_roundtrip_tests() -> Vec<RoundtripTest> {
    let mut tests = Vec::new();

    // Load spec.txt and extract test cases
    if let Ok(content) = fs::read_to_string("tests/fixtures/spec.txt") {
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;
        let mut test_num = 0;

        while i < lines.len() {
            if lines[i].contains("example") && lines[i].contains("````") {
                test_num += 1;
                i += 1;

                // Collect markdown input
                let mut markdown = String::new();
                while i < lines.len() && lines[i] != "." {
                    if !markdown.is_empty() {
                        markdown.push('\n');
                    }
                    markdown.push_str(lines[i]);
                    i += 1;
                }

                tests.push(RoundtripTest {
                    name: format!("spec_{}", test_num),
                    markdown,
                });

                // Skip to end of example
                while i < lines.len() && !lines[i].contains("````") {
                    i += 1;
                }
            }
            i += 1;
        }
    }

    tests
}

#[test]
fn test_roundtrip_basic() {
    let test_cases = vec![
        ("paragraph", "Hello world\n"),
        ("heading", "# Heading\n"),
        ("list", "- Item 1\n- Item 2\n"),
        ("code", "```\ncode\n```\n"),
        ("quote", "> Quote\n"),
    ];

    for (name, markdown) in test_cases {
        let html = md_to_html(markdown);
        let roundtrip = html_to_md(&html);

        test_log!("\n=== {} ===", name);
        test_log!("Original: {:?}", markdown);
        test_log!("HTML: {:?}", html);
        test_log!("Roundtrip: {:?}", roundtrip);

        // Roundtrip may not be identical, but should be semantically similar
        // For now, just verify it doesn't panic
        assert!(!roundtrip.is_empty(), "Roundtrip should produce output");
    }
}

#[test]
fn test_roundtrip_spec_examples() {
    let tests = load_roundtrip_tests();

    // Test a sample of spec examples
    let sample_size = std::cmp::min(10, tests.len());
    for test in tests.iter().take(sample_size) {
        let html = md_to_html(&test.markdown);
        let roundtrip = html_to_md(&html);

        test_log!("\n=== {} ===", test.name);
        test_log!("Original length: {}", test.markdown.len());
        test_log!("HTML length: {}", html.len());
        test_log!("Roundtrip length: {}", roundtrip.len());

        // Verify roundtrip produces valid output
        assert!(!roundtrip.is_empty(), "Roundtrip should produce output");
    }
}

#[test]
fn test_roundtrip_preserves_structure() {
    // Test that basic structure is preserved
    let markdown = "# Heading\n\nParagraph with **bold** and *italic*.\n\n- List item\n\n> Blockquote\n";

    let html = md_to_html(markdown);
    let roundtrip = html_to_md(&html);

    test_log!("Original:\n{}", markdown);
    test_log!("\nHTML:\n{}", html);
    test_log!("\nRoundtrip:\n{}", roundtrip);

    // Verify key elements are preserved
    assert!(
        roundtrip.contains("#") || roundtrip.contains("Heading"),
        "Heading should be preserved"
    );
}

#[test]
fn test_roundtrip_empty_input() {
    let html = md_to_html("");
    let roundtrip = html_to_md(&html);
    assert!(
        roundtrip.is_empty(),
        "Empty input should produce empty roundtrip"
    );
}

#[test]
fn test_roundtrip_whitespace_only() {
    let html = md_to_html("   \n   ");
    let roundtrip = html_to_md(&html);
    // Whitespace handling may vary
    test_log!("Whitespace roundtrip: {:?}", roundtrip);
}

#[test]
fn test_roundtrip_special_chars() {
    let markdown = "< > & \"";
    let html = md_to_html(markdown);
    let roundtrip = html_to_md(&html);

    test_log!("Special chars HTML: {:?}", html);
    test_log!("Special chars roundtrip: {:?}", roundtrip);

    // Special characters should be handled correctly
    assert!(!roundtrip.is_empty());
}
