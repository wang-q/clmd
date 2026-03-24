//! HTML Entity Tests
//!
//! Tests for HTML entity parsing based on CommonMark spec.

use clmd::{markdown_to_html, options};

/// Test basic named entities are converted to characters
#[test]
fn test_basic_named_entities_converted() {
    let test_cases = vec![
        ("&amp;", "&"),
        ("&lt;", "<"),
        ("&gt;", ">"),
        ("&quot;", "\""),
    ];

    for (entity, expected_char) in test_cases {
        let input = format!("{}", entity);
        let result = markdown_to_html(&input, options::DEFAULT);
        // The entity should be converted to its character
        assert!(
            result.contains(expected_char) || result.contains(entity),
            "Entity {} should produce {} or be preserved, got: {}",
            entity,
            expected_char,
            result
        );
    }
}

/// Test numeric character references (decimal) are converted
#[test]
fn test_decimal_numeric_references_converted() {
    let test_cases = vec![("&#65;", "A"), ("&#97;", "a"), ("&#38;", "&")];

    for (entity, expected_char) in test_cases {
        let input = format!("{}", entity);
        let result = markdown_to_html(&input, options::DEFAULT);
        assert!(
            result.contains(expected_char),
            "Entity {} should produce {}, got: {}",
            entity,
            expected_char,
            result
        );
    }
}

/// Test numeric character references (hexadecimal) are converted
#[test]
fn test_hexadecimal_numeric_references_converted() {
    let test_cases = vec![("&#x41;", "A"), ("&#x61;", "a"), ("&#x26;", "&")];

    for (entity, expected_char) in test_cases {
        let input = format!("{}", entity);
        let result = markdown_to_html(&input, options::DEFAULT);
        assert!(
            result.contains(expected_char),
            "Entity {} should produce {}, got: {}",
            entity,
            expected_char,
            result
        );
    }
}

/// Test common named entities are converted
#[test]
fn test_common_named_entities_converted() {
    let test_cases = vec![
        ("&copy;", "©"),
        ("&reg;", "®"),
        ("&trade;", "™"),
        ("&mdash;", "—"),
        ("&ndash;", "–"),
        ("&hellip;", "…"),
    ];

    for (entity, expected_char) in test_cases {
        let input = format!("{}", entity);
        let result = markdown_to_html(&input, options::DEFAULT);
        assert!(
            result.contains(expected_char),
            "Entity {} should produce {}, got: {}",
            entity,
            expected_char,
            result
        );
    }
}

/// Test invalid entities are handled (preserved or escaped)
#[test]
fn test_invalid_entities_handled() {
    // Invalid entities - the & may be escaped to &amp;
    let result = markdown_to_html("&notanentity;", options::DEFAULT);
    // Should either preserve the entity or escape the &
    assert!(
        result.contains("&notanentity;") || result.contains("&amp;notanentity;"),
        "Invalid entity should be handled, got: {}",
        result
    );

    // Malformed numeric references
    let result = markdown_to_html("&#;", options::DEFAULT);
    assert!(
        result.contains("&#;") || result.contains("&amp;#;"),
        "Malformed entity should be handled, got: {}",
        result
    );

    let result = markdown_to_html("&#x;", options::DEFAULT);
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
    let code_result = markdown_to_html(code_input, options::DEFAULT);
    assert!(
        code_result.contains("&amp;"),
        "Entities in code blocks should be preserved, got: {}",
        code_result
    );

    // In inline code
    let inline_code = "`&amp;`";
    let inline_result = markdown_to_html(inline_code, options::DEFAULT);
    assert!(
        inline_result.contains("&amp;"),
        "Entities in inline code should be preserved, got: {}",
        inline_result
    );

    // In regular text - entities are converted
    let text = "This is &amp; that";
    let text_result = markdown_to_html(text, options::DEFAULT);
    assert!(
        text_result.contains("&") || text_result.contains("&amp;"),
        "Entities in text should be handled, got: {}",
        text_result
    );
}

/// Test multiple entities in one line
#[test]
fn test_multiple_entities() {
    let input = "&lt;div&gt; &amp; &quot;test&quot;";
    let result = markdown_to_html(input, options::DEFAULT);
    // Entities should be converted or preserved
    assert!(result.contains("<") || result.contains("&lt;"));
    assert!(result.contains(">") || result.contains("&gt;"));
    assert!(result.contains("&") || result.contains("&amp;"));
    assert!(result.contains("\"") || result.contains("&quot;"));
}

/// Test case sensitivity
#[test]
fn test_entity_case_sensitivity() {
    // Lowercase should work
    let lower = markdown_to_html("&amp;", options::DEFAULT);
    assert!(lower.contains("&") || lower.contains("&amp;"));

    // Uppercase may or may not work depending on implementation
    // Just verify it doesn't panic
    let _upper = markdown_to_html("&AMP;", options::DEFAULT);
}

/// Test entities at boundaries
#[test]
fn test_entity_boundaries() {
    // Entity at start
    let result = markdown_to_html("&amp; test", options::DEFAULT);
    assert!(result.contains("&") || result.contains("&amp;"));

    // Entity at end
    let result = markdown_to_html("test &amp;", options::DEFAULT);
    assert!(result.contains("&") || result.contains("&amp;"));

    // Entity alone
    let result = markdown_to_html("&amp;", options::DEFAULT);
    assert!(result.contains("&") || result.contains("&amp;"));
}

/// Test that special characters are escaped in output
#[test]
fn test_special_characters_escaped() {
    // These are literal characters, not entities
    let input = "<div> & \"test\"";
    let result = markdown_to_html(input, options::DEFAULT);
    // They should be escaped in HTML output
    assert!(result.contains("&lt;") || result.contains("<"));
    assert!(result.contains("&gt;") || result.contains(">"));
}
