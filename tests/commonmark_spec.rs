use clmd::{markdown_to_html, options};
use std::collections::HashMap;
use std::fs;

/// Test logging macro - only prints when VERBOSE_TESTS is set
macro_rules! test_log {
    ($($arg:tt)*) => {
        if std::env::var("VERBOSE_TESTS").is_ok() {
            std::println!($($arg)*);
        }
    };
}

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
        if let Some(section) = line.strip_prefix("## ") {
            current_section = section.trim().to_string();
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

/// Known failing test cases that need to be fixed.
/// These are tracked for debugging and future improvement.
const KNOWN_FAILURES: &[usize] = &[
    // Currently no specific known failures tracked
    // Add test numbers here as they are identified
];

/// Check if a test failure is known
fn is_known_failure(test_num: usize) -> bool {
    KNOWN_FAILURES.contains(&test_num)
}

#[test]
fn test_commonmark_spec() {
    let spec_content =
        fs::read_to_string("tests/fixtures/spec.txt").expect("Failed to read spec.txt");

    let tests = parse_spec_tests(&spec_content);
    test_log!("Found {} test cases", tests.len());

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

    test_log!("\n=== CommonMark Spec Test Results ===");
    test_log!(
        "Passed: {}/{} ({:.1}%)",
        passed,
        tests.len(),
        (passed as f64 / tests.len() as f64) * 100.0
    );
    test_log!(
        "Failed: {}/{} ({:.1}%)",
        failed,
        tests.len(),
        (failed as f64 / tests.len() as f64) * 100.0
    );

    if let Some((num, section)) = first_passed {
        test_log!("\nFirst passed test: #{} ({})", num, section);
    }

    // Group failed tests by section
    if !failed_tests.is_empty() {
        let mut failed_by_section: HashMap<String, Vec<usize>> = HashMap::new();
        for (num, section) in &failed_tests {
            failed_by_section
                .entry(section.clone())
                .or_default()
                .push(*num);
        }

        test_log!("\n=== Failed Tests by Section ===");
        let mut sections: Vec<_> = failed_by_section.iter().collect();
        sections.sort_by_key(|(s, _)| s.as_str());
        for (section, tests) in sections {
            test_log!("{}: {} tests", section, tests.len());
        }
    }

    // Print all failed tests by section
    let mut failed_by_section: HashMap<String, Vec<(usize, String, String, String)>> =
        HashMap::new();
    for (num, section, markdown, expected, got) in &failures {
        failed_by_section.entry(section.clone()).or_default().push((
            *num,
            markdown.clone(),
            expected.clone(),
            got.clone(),
        ));
    }

    // Also collect all failures, not just first 10
    for test in &tests {
        let result = markdown_to_html(&test.markdown, options::DEFAULT);
        if result != test.html {
            let expected_normalized = normalize_html(&test.html);
            let result_normalized = normalize_html(&result);
            if expected_normalized != result_normalized {
                failed_by_section
                    .entry(test.section.clone())
                    .or_default()
                    .push((
                        test.number,
                        test.markdown.clone(),
                        test.html.clone(),
                        result,
                    ));
            }
        }
    }

    if !failed_by_section.is_empty() {
        test_log!("\n=== All Failures by Section ===");
        let mut sections: Vec<_> = failed_by_section.iter().collect();
        sections.sort_by_key(|(s, _)| s.as_str());
        for (section, tests) in sections {
            test_log!("\n=== {} ===", section);
            for (num, markdown, expected, got) in tests.iter().take(5) {
                test_log!("\nTest #{}", num);
                test_log!("Input: {:?}", markdown);
                test_log!("Expected: {:?}", expected);
                test_log!("Got: {:?}", got);
            }
            if tests.len() > 5 {
                test_log!("... and {} more", tests.len() - 5);
            }
        }
    }

    // Assert on pass rate to prevent regressions
    const MIN_PASS_RATE: f64 = 1.0;
    let pass_rate = passed as f64 / tests.len() as f64;
    assert!(
        pass_rate >= MIN_PASS_RATE,
        "Pass rate {:.1}% is below the threshold {:.1}%\nPassed: {}/{} tests",
        pass_rate * 100.0,
        MIN_PASS_RATE * 100.0,
        passed,
        tests.len()
    );
}

#[test]
fn test_regression() {
    let spec_content = fs::read_to_string("tests/fixtures/regression.txt")
        .expect("Failed to read regression.txt");

    let tests = parse_spec_tests(&spec_content);
    test_log!("Found {} regression test cases", tests.len());

    let mut passed = 0;
    let mut failed = 0;

    for test in &tests {
        let result = markdown_to_html(&test.markdown, options::DEFAULT);

        if result == test.html {
            passed += 1;
        } else {
            failed += 1;
            test_log!("\nFailed test #{} ({})", test.number, test.section);
            test_log!("Input: {:?}", test.markdown);
            test_log!("Expected: {:?}", test.html);
            test_log!("Got: {:?}", result);
        }
    }

    test_log!("\n=== Regression Test Results ===");
    test_log!(
        "Passed: {}/{} ({:.1}%)",
        passed,
        tests.len(),
        (passed as f64 / tests.len() as f64) * 100.0
    );
    test_log!(
        "Failed: {}/{} ({:.1}%)",
        failed,
        tests.len(),
        (failed as f64 / tests.len() as f64) * 100.0
    );

    assert!(
        passed > 0,
        "No regression tests passed - there may be a fundamental issue"
    );
}

#[test]
fn test_smart_punct() {
    let spec_content = fs::read_to_string("tests/fixtures/smart_punct.txt")
        .expect("Failed to read smart_punct.txt");

    let tests = parse_spec_tests(&spec_content);
    test_log!("Found {} smart punctuation test cases", tests.len());

    let mut passed = 0;
    let mut failed = 0;

    for test in &tests {
        // Smart punctuation requires SMART option
        let result = markdown_to_html(&test.markdown, options::SMART);

        if result == test.html {
            passed += 1;
        } else {
            failed += 1;
            test_log!("\nFailed test #{} ({})", test.number, test.section);
            test_log!("Input: {:?}", test.markdown);
            test_log!("Expected: {:?}", test.html);
            test_log!("Got: {:?}", result);
        }
    }

    test_log!("\n=== Smart Punctuation Test Results ===");
    test_log!(
        "Passed: {}/{} ({:.1}%)",
        passed,
        tests.len(),
        (passed as f64 / tests.len() as f64) * 100.0
    );
    test_log!(
        "Failed: {}/{} ({:.1}%)",
        failed,
        tests.len(),
        (failed as f64 / tests.len() as f64) * 100.0
    );

    assert!(
        passed > 0,
        "No smart punctuation tests passed - there may be a fundamental issue"
    );
}

#[test]
fn test_specific_examples() {
    // Test a few specific examples to verify basic functionality

    // Example 1: Thematic breaks
    let result = markdown_to_html("***\n---\n___\n", options::DEFAULT);
    assert!(
        result.contains("<hr"),
        "Thematic breaks should produce <hr> tags"
    );

    // Example 2: Basic paragraph
    let result = markdown_to_html("Hello world\n", options::DEFAULT);
    assert!(result.contains("<p>"), "Should produce paragraph tags");

    // Example 3: ATX heading
    let result = markdown_to_html("# Heading\n", options::DEFAULT);
    assert!(result.contains("<h1>"), "Should produce h1 tag");

    // Debug test for ATX heading issue #79
    let result = markdown_to_html("## \n#\n### ###", options::DEFAULT);
    test_log!("ATX heading test #79 result: {:?}", result);
    test_log!("Expected: {:?}", "<h2></h2>\n<h1></h1>\n<h3></h3>");

    // Debug test for Setext heading issue #91
    let input = "`Foo\n----\n`\n\n<a title=\"a lot\n---\nof dashes\"/>";
    let result = markdown_to_html(input, options::DEFAULT);
    test_log!("\nSetext heading test #91 result: {:?}", result);
    test_log!(
        "Expected: {:?}",
        "<h2>`Foo</h2>\n<p>`</p>\n<h2>&lt;a title=\"a lot</h2>\n<p>of dashes\"/&gt;</p>"
    );

    // Simpler test case
    let input2 = "<a title=\"a lot\n---\nof dashes\"/>";
    let result2 = markdown_to_html(input2, options::DEFAULT);
    test_log!("\nSimpler test result: {:?}", result2);

    // Even simpler test case - just the heading part
    let input3 = "<a title=\"a lot\n---";
    let result3 = markdown_to_html(input3, options::DEFAULT);
    test_log!("\nEven simpler test result: {:?}", result3);

    // Test without newline
    let input4 = "<a title=\"a lot";
    let result4 = markdown_to_html(input4, options::DEFAULT);
    test_log!("\nWithout newline test result: {:?}", result4);

    // Test fenced code block #126
    let input5 = "```";
    let result5 = markdown_to_html(input5, options::DEFAULT);
    test_log!("\nFenced code block #126 result: {:?}", result5);
    test_log!("Expected: {:?}", "<pre><code></code></pre>");

    // Test fenced code block with content
    let input6 = "```\nfoo\n```";
    let result6 = markdown_to_html(input6, options::DEFAULT);
    test_log!("\nFenced code block with content result: {:?}", result6);
    test_log!("Expected: {:?}", "<pre><code>foo\n</code></pre>");
}
