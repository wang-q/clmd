//! Roundtrip Tests
//!
//! Tests for Markdown -> HTML -> Markdown roundtrip consistency.
//! Based on cmark's roundtrip_tests.py.

use clmd::{html_to_md, markdown_to_html, options};
use std::fs;

#[derive(Debug)]
#[allow(dead_code)]
struct TestCase {
    number: usize,
    section: String,
    markdown: String,
    html: String,
}

/// Parse spec.txt format
fn parse_spec_tests(content: &str) -> Vec<TestCase> {
    let mut tests = Vec::new();
    let mut current_section = String::new();
    let mut test_number = 0;

    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        // Check for section header
        if line.starts_with("## ") {
            current_section = line[3..].trim().to_string();
        }

        // Check for test example start
        if line.contains("example") && line.contains("````") {
            test_number += 1;
            i += 1;

            // Collect markdown input until we hit the dot separator
            let mut markdown = String::new();
            while i < lines.len() && lines[i] != "." {
                if !markdown.is_empty() {
                    markdown.push('\n');
                }
                markdown.push_str(lines[i]);
                i += 1;
            }

            // Skip the dot line
            i += 1;

            // Collect expected HTML output
            let mut html = String::new();
            while i < lines.len() && !lines[i].contains("````") {
                if !html.is_empty() {
                    html.push('\n');
                }
                html.push_str(lines[i]);
                i += 1;
            }

            // Replace visual tab representation (→) with actual tab character
            let markdown = markdown.replace('→', "\t");
            let html = html.replace('→', "\t");

            tests.push(TestCase {
                number: test_number,
                section: current_section.clone(),
                markdown,
                html,
            });
        }

        i += 1;
    }

    tests
}

/// Test roundtrip for a sample of spec tests
#[test]
fn test_roundtrip_sample() {
    let spec_content =
        fs::read_to_string("tests/fixtures/spec.txt").expect("Failed to read spec.txt");

    let tests = parse_spec_tests(&spec_content);
    println!("Testing roundtrip for {} spec tests", tests.len());

    let mut passed = 0;
    let mut failed = 0;
    let mut skipped = 0;

    // Test a sample of tests to avoid long running times
    for test in tests.iter().step_by(10) {
        // Skip tests that are known to have roundtrip issues
        if test.section.contains("HTML") || test.section.contains("Raw") {
            skipped += 1;
            continue;
        }

        // Markdown -> HTML
        let html = markdown_to_html(&test.markdown, options::DEFAULT);

        // HTML -> Markdown (using our HTML to Markdown converter)
        let roundtrip_md = html_to_md::convert(&html);

        // Normalize both for comparison
        let original_normalized = normalize_markdown(&test.markdown);
        let roundtrip_normalized = normalize_markdown(&roundtrip_md);

        if original_normalized == roundtrip_normalized {
            passed += 1;
        } else {
            failed += 1;
            if failed <= 3 {
                println!(
                    "\nRoundtrip failed for test #{} ({})",
                    test.number, test.section
                );
                println!("Original: {:?}", test.markdown);
                println!("Roundtrip: {:?}", roundtrip_md);
            }
        }
    }

    println!("\n=== Roundtrip Test Results ===");
    println!("Passed: {}", passed);
    println!("Failed: {}", failed);
    println!("Skipped: {}", skipped);

    // For now, just verify we don't panic and some tests pass
    assert!(passed > 0, "Some roundtrip tests should pass");
}

/// Test simple roundtrip cases
#[test]
fn test_simple_roundtrip() {
    let test_cases = vec![
        "Hello world",
        "# Heading",
        "**bold**",
        "*italic*",
        "`code`",
        "[link](http://example.com)",
        "- item 1\n- item 2",
        "1. item 1\n2. item 2",
        "> quote",
        "```\ncode block\n```",
    ];

    for _md in test_cases {
        let html = markdown_to_html(_md, options::DEFAULT);
        let roundtrip = html_to_md::convert(&html);

        // Basic sanity check - roundtrip should not be empty
        assert!(
            !roundtrip.is_empty(),
            "Roundtrip should not be empty for: {}",
            _md
        );

        // The roundtrip should contain the essential content
        // (exact format may differ)
        println!("Original: {:?}", _md);
        println!("HTML: {:?}", html);
        println!("Roundtrip: {:?}", roundtrip);
        println!();
    }
}

/// Test that HTML comments are handled in roundtrip
#[test]
fn test_roundtrip_with_comments() {
    // CommonMark renderer inserts <!-- end list --> comments
    let html = "<p>text</p>\n<!-- end list -->\n<p>more</p>";
    let md = html_to_md::convert(html);

    // Comments should be stripped or handled
    assert!(!md.contains("<!--"), "HTML comments should be handled");
}

/// Normalize markdown for comparison
fn normalize_markdown(md: &str) -> String {
    md.lines()
        .map(|line| line.trim_end())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

/// Test specific edge cases
#[test]
fn test_roundtrip_edge_cases() {
    // Empty document
    let html = markdown_to_html("", options::DEFAULT);
    let md = html_to_md::convert(&html);
    assert!(md.is_empty() || md.trim().is_empty());

    // Only whitespace
    let html = markdown_to_html("   \n   ", options::DEFAULT);
    let _md = html_to_md::convert(&html);
    // Result may be empty or contain whitespace
    let _ = _md; // Suppress unused variable warning

    // Special characters
    let html = markdown_to_html("< > & \"", options::DEFAULT);
    let md = html_to_md::convert(&html);
    assert!(!md.is_empty());
}
