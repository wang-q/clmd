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
fn test_basic_link_reference() {
    let input = "[foo]\n\n[foo]: /bar\n";
    let result = markdown_to_html(input, options::DEFAULT);
    test_log!("Input: {:?}", input);
    test_log!("Output: {:?}", result);
    assert!(
        result.contains("<a href=\"/bar\">foo</a>"),
        "Expected link reference to be resolved, got: {}",
        result
    );
}

#[test]
fn test_link_reference_with_title() {
    let input = "[foo]\n\n[foo]: /bar \"title\"\n";
    let result = markdown_to_html(input, options::DEFAULT);
    test_log!("Input: {:?}", input);
    test_log!("Output: {:?}", result);
    assert!(
        result.contains("<a href=\"/bar\" title=\"title\">foo</a>"),
        "Expected link reference with title, got: {}",
        result
    );
}

#[test]
fn test_inline_link() {
    let input = "[foo](/bar)\n";
    let result = markdown_to_html(input, options::DEFAULT);
    test_log!("Input: {:?}", input);
    test_log!("Output: {:?}", result);
    assert!(
        result.contains("<a href=\"/bar\">foo</a>"),
        "Expected inline link, got: {}",
        result
    );
}

#[test]
fn test_inline_link_with_title() {
    let input = "[foo](/bar \"title\")\n";
    let result = markdown_to_html(input, options::DEFAULT);
    test_log!("Input: {:?}", input);
    test_log!("Output: {:?}", result);
    assert!(
        result.contains("<a href=\"/bar\" title=\"title\">foo</a>"),
        "Expected inline link with title, got: {}",
        result
    );
}
