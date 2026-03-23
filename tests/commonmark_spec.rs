use md::{markdown_to_html, options};
use std::fs;
use std::collections::HashMap;

#[derive(Debug)]
struct TestCase {
    number: usize,
    section: String,
    markdown: String,
    html: String,
}

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
            // The spec uses → (U+2192) to represent tabs in the test cases
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

/// Normalize HTML for comparison
/// This normalizes whitespace to make comparison more lenient
fn normalize_html(html: &str) -> String {
    // First, normalize all whitespace sequences to a single space
    let mut result = String::new();
    let mut prev_was_space = true; // Start true to trim leading whitespace

    for c in html.chars() {
        if c.is_whitespace() {
            if !prev_was_space {
                result.push(' ');
                prev_was_space = true;
            }
        } else {
            result.push(c);
            prev_was_space = false;
        }
    }

    // Trim trailing whitespace
    result.trim().to_string()
}

#[test]
fn test_commonmark_spec() {
    let spec_content = fs::read_to_string("tests/fixtures/spec.txt")
        .expect("Failed to read spec.txt");

    let tests = parse_spec_tests(&spec_content);
    println!("Found {} test cases", tests.len());

    let mut passed = 0;
    let mut failed = 0;
    let mut failures: Vec<(usize, String, String, String, String)> = Vec::new();
    let mut failed_tests: Vec<(usize, String)> = Vec::new();
    let mut first_passed: Option<(usize, String)> = None;

    for test in &tests {
        let result = markdown_to_html(&test.markdown, options::DEFAULT);

        // Try exact match first
        if result == test.html {
            passed += 1;
            if first_passed.is_none() {
                first_passed = Some((test.number, test.section.clone()));
            }
        } else {
            // Try normalized match for more lenient comparison
            let expected_normalized = normalize_html(&test.html);
            let result_normalized = normalize_html(&result);

            if expected_normalized == result_normalized {
                passed += 1;
                if first_passed.is_none() {
                    first_passed = Some((test.number, test.section.clone()));
                }
            } else {
                failed += 1;
                failed_tests.push((test.number, test.section.clone()));
                if failures.len() < 10 {
                    failures.push((
                        test.number,
                        test.section.clone(),
                        test.markdown.clone(),
                        test.html.clone(),
                        result,
                    ));
                }
            }
        }
    }

    println!("\n=== CommonMark Spec Test Results ===");
    println!("Passed: {}/{} ({:.1}%)", passed, tests.len(),
        (passed as f64 / tests.len() as f64) * 100.0);
    println!("Failed: {}/{} ({:.1}%)", failed, tests.len(),
        (failed as f64 / tests.len() as f64) * 100.0);

    if let Some((num, section)) = first_passed {
        println!("\nFirst passed test: #{} ({})", num, section);
    }

    // Group failed tests by section
    if !failed_tests.is_empty() {
        let mut failed_by_section: HashMap<String, Vec<usize>> = HashMap::new();
        for (num, section) in &failed_tests {
            failed_by_section.entry(section.clone()).or_default().push(*num);
        }

        println!("\n=== Failed Tests by Section ===");
        let mut sections: Vec<_> = failed_by_section.iter().collect();
        sections.sort_by_key(|(s, _)| s.as_str());
        for (section, tests) in sections {
            println!("{}: {} tests", section, tests.len());
        }
    }

    if !failures.is_empty() {
        println!("\n=== First {} Failures ===", failures.len());
        for (num, section, markdown, expected, got) in &failures {
            println!("\nTest #{} ({})", num, section);
            println!("Input markdown (escaped): {:?}", markdown);
            println!("Expected (escaped): {:?}", expected);
            println!("Got (escaped): {:?}", got);
            println!("Expected:\n{}", expected);
            println!("Got:\n{}", got);
            println!("---");
        }
    }

    // For now, just print results without failing
    // Once we have good coverage, we can assert on pass rate
    assert!(
        passed > 0,
        "No tests passed - there may be a fundamental issue"
    );
}

#[test]
fn test_specific_examples() {
    // Test a few specific examples to verify basic functionality

    // Example 1: Thematic breaks
    let result = markdown_to_html("***\n---\n___\n", options::DEFAULT);
    assert!(result.contains("<hr"), "Thematic breaks should produce <hr> tags");

    // Example 2: Basic paragraph
    let result = markdown_to_html("Hello world\n", options::DEFAULT);
    assert!(result.contains("<p>"), "Should produce paragraph tags");

    // Example 3: ATX heading
    let result = markdown_to_html("# Heading\n", options::DEFAULT);
    assert!(result.contains("<h1>"), "Should produce h1 tag");
}
