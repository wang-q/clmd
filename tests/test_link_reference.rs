use clmd::markdown_to_html;
use clmd::options::Options;

/// Helper function to convert markdown to HTML with default options
fn md_to_html(input: &str) -> String {
    markdown_to_html(input, &Options::default())
}

/// Test logging macro - only prints when VERBOSE_TESTS is set
macro_rules! test_log {
    ($($arg:tt)*) => {
        if std::env::var("VERBOSE_TESTS").is_ok() {
            std::println!($($arg)*);
        }
    };
}

#[test]
fn test_basic_link_reference() {
    let input = "[link][ref]\n\n[ref]: https://example.com";
    let result = md_to_html(input);
    test_log!("Basic link ref result: {}", result);
    assert!(result.contains("<a href=\"https://example.com\">"));
}

#[test]
fn test_link_reference_with_title() {
    let input = "[link][ref]\n\n[ref]: https://example.com \"title\"";
    let result = md_to_html(input);
    test_log!("Link ref with title result: {}", result);
    assert!(result.contains("<a href=\"https://example.com\" title=\"title\">"));
}

#[test]
fn test_link_reference_collapsed() {
    let input = "[ref][]\n\n[ref]: https://example.com";
    let result = md_to_html(input);
    test_log!("Collapsed link ref result: {}", result);
    assert!(result.contains("<a href=\"https://example.com\">"));
}

#[test]
fn test_link_reference_shortcut() {
    let input = "[ref]\n\n[ref]: https://example.com";
    let result = md_to_html(input);
    test_log!("Shortcut link ref result: {}", result);
    assert!(result.contains("<a href=\"https://example.com\">"));
}

#[test]
fn test_link_reference_unused() {
    let input = "[ref]: https://example.com";
    let result = md_to_html(input);
    test_log!("Unused link ref result: {}", result);
    // Should produce empty output or just whitespace
}

#[test]
fn test_link_reference_multiple() {
    let input = "[link1][ref1] [link2][ref2]\n\n[ref1]: https://example.com\n[ref2]: https://example.org";
    let result = md_to_html(input);
    test_log!("Multiple link refs result: {}", result);
    assert!(result.contains("<a href=\"https://example.com\">"));
    assert!(result.contains("<a href=\"https://example.org\">"));
}

#[test]
fn test_link_reference_case_insensitive() {
    let input = "[Link][REF]\n\n[ref]: https://example.com";
    let result = md_to_html(input);
    test_log!("Case insensitive link ref result: {}", result);
    assert!(result.contains("<a href=\"https://example.com\">"));
}

#[test]
fn test_link_reference_not_found() {
    let input = "[link][missing]";
    let result = md_to_html(input);
    test_log!("Missing link ref result: {}", result);
    // Should not create a link
}
