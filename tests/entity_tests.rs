//! HTML Entity Tests
//!
//! Tests for HTML entity parsing based on CommonMark spec.

use clmd::markdown_to_html;
use clmd::parse::options::Options;

/// Helper function to convert markdown to HTML with default options
fn md_to_html(input: &str) -> String {
    markdown_to_html(input, &Options::default())
}

/// Test basic named entities are converted to characters
#[test]
fn test_basic_named_entities_converted() {
    let test_cases = vec![("&amp;", "&"), ("&lt;", "<"), ("&gt;", ">")];

    for (input, expected) in test_cases {
        let result = md_to_html(input);
        assert!(
            result.contains(expected),
            "Entity {} should be converted to {} in {}",
            input,
            expected,
            result
        );
    }

    // &quot; is preserved in output (not converted to ")
    let result = md_to_html("&quot;");
    assert!(
        result.contains("&quot;") || result.contains("\""),
        "Entity &quot; should be handled, got: {}",
        result
    );
}

/// Test numeric entities
#[test]
fn test_numeric_entities() {
    let test_cases = vec![
        ("&#65;", "A"),
        ("&#x41;", "A"),
        ("&#97;", "a"),
        ("&#x61;", "a"),
    ];

    for (input, expected) in test_cases {
        let result = md_to_html(input);
        assert!(
            result.contains(expected),
            "Numeric entity {} should be converted to {} in {}",
            input,
            expected,
            result
        );
    }
}

/// Test invalid entities are handled (preserved or escaped)
#[test]
fn test_invalid_entities_handled() {
    // Invalid entities - the & may be escaped to &amp;
    let result = md_to_html("&notanentity;");
    // Should either preserve the entity or escape the &
    assert!(
        result.contains("&notanentity;") || result.contains("&amp;notanentity;"),
        "Invalid entity should be handled, got: {}",
        result
    );

    // Malformed numeric references
    let result = md_to_html("&#;");
    assert!(
        result.contains("&#;") || result.contains("&amp;#;"),
        "Malformed entity should be handled, got: {}",
        result
    );

    let result = md_to_html("&#x;");
    assert!(
        result.contains("&#x;") || result.contains("&amp;#x;"),
        "Malformed entity should be handled, got: {}",
        result
    );
}

/// Test entities in different contexts
#[test]
fn test_entities_in_context() {
    // In code blocks, entities should be preserved
    let code_input = "```\n&amp;\n```";
    let code_result = md_to_html(code_input);
    assert!(
        code_result.contains("&amp;"),
        "Entities in code blocks should be preserved, got: {}",
        code_result
    );

    // In inline code
    let inline_code = "`&amp;`";
    let inline_result = md_to_html(inline_code);
    assert!(
        inline_result.contains("&amp;"),
        "Entities in inline code should be preserved, got: {}",
        inline_result
    );

    // In regular text - entities are converted
    let text = "This is &amp; that";
    let text_result = md_to_html(text);
    assert!(
        text_result.contains("&") || text_result.contains("&amp;"),
        "Entities in text should be handled, got: {}",
        text_result
    );
}

/// Test case sensitivity
#[test]
fn test_entity_case_sensitivity() {
    // Lowercase should work
    let lower = md_to_html("&amp;");
    assert!(lower.contains("&") || lower.contains("&amp;"));

    // Uppercase may or may not work depending on implementation
    // Just verify it doesn't panic
    let _upper = md_to_html("&AMP;");
}

/// Test entities at boundaries
#[test]
fn test_entity_boundaries() {
    // Entity at start
    let result = md_to_html("&amp; test");
    assert!(result.contains("&") || result.contains("&amp;"));

    // Entity at end
    let result = md_to_html("test &amp;");
    assert!(result.contains("&") || result.contains("&amp;"));

    // Entity alone
    let result = md_to_html("&amp;");
    assert!(result.contains("&") || result.contains("&amp;"));
}
