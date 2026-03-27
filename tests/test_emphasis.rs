// Test specific emphasis cases

// Allow deprecated API usage in tests until all tests are migrated
#![allow(deprecated)]

use clmd::{markdown_to_html, options};

/// Test logging macro - only prints when VERBOSE_TESTS is set
macro_rules! test_log {
    ($($arg:tt)*) => {
        if std::env::var("VERBOSE_TESTS").is_ok() {
            std::println!($($arg)*);
        }
    };
}

#[test]
fn test_basic_emphasis() {
    // Test 1: Basic emphasis with asterisk
    let result = markdown_to_html("*foo bar*", options::DEFAULT);
    test_log!("Test 1 - Basic emphasis: '{}'", result);
    assert!(result.contains("<em>"), "Should contain <em> tag");

    // Test 2: Basic strong with double asterisk
    let result = markdown_to_html("**foo bar**", options::DEFAULT);
    test_log!("Test 2 - Basic strong: '{}'", result);
    assert!(result.contains("<strong>"), "Should contain <strong> tag");

    // Test 3: Nested emphasis and strong
    let result = markdown_to_html("**foo *bar* baz**", options::DEFAULT);
    test_log!("Test 3 - Nested: '{}'", result);
    assert!(result.contains("<strong>"), "Should contain <strong> tag");
    assert!(result.contains("<em>"), "Should contain <em> tag");
}

#[test]
fn test_emphasis_rules() {
    // Test 4: Intraword emphasis with asterisks is not allowed
    let result = markdown_to_html("foo*bar*baz", options::DEFAULT);
    test_log!("Test 4 - Intraword asterisk: '{}'", result);
    // According to CommonMark, intraword emphasis with * is NOT allowed
    // So the output should NOT have <em> tags

    // Test 5: Intraword emphasis with underscores
    let result = markdown_to_html("foo_bar_baz", options::DEFAULT);
    test_log!("Test 5 - Intraword underscore: '{}'", result);
    // Intraword emphasis with _ is also NOT allowed

    // Test 6: Emphasis at word boundaries
    let result = markdown_to_html("_foo_", options::DEFAULT);
    test_log!("Test 6 - Underscore emphasis: '{}'", result);
    assert!(result.contains("<em>"), "Should contain <em> tag");
}

#[test]
fn test_emphasis_complex() {
    // Test 7: Multiple emphasis markers
    let result = markdown_to_html("*foo *bar**", options::DEFAULT);
    test_log!("Test 7 - Multiple: '{}'", result);

    // Test 8: Emphasis with punctuation
    let result = markdown_to_html("*(*foo*)*", options::DEFAULT);
    test_log!("Test 8 - With punctuation: '{}'", result);

    // Test 9: Empty emphasis
    let result = markdown_to_html("**", options::DEFAULT);
    test_log!("Test 9 - Empty: '{}'", result);
}

#[test]
fn test_commonmark_emphasis_examples() {
    // From CommonMark spec test cases

    // Example 360: rule 1
    let result = markdown_to_html("*foo bar*", options::DEFAULT);
    test_log!("Spec 360: '{}'", result);
    assert_eq!(result, "<p><em>foo bar</em></p>");

    // Example 361: rule 2 - intraword not allowed
    let result = markdown_to_html("a * foo bar*", options::DEFAULT);
    test_log!("Spec 361: '{}'", result);
    // Should NOT be emphasis because of space after *

    // Example 362
    let result = markdown_to_html("a*\"foo\"*", options::DEFAULT);
    test_log!("Spec 362: '{}'", result);
}
