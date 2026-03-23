use md::{markdown_to_html, options};

#[test]
fn debug_simple2() {
    // Test case with emphasis outside link
    let input = "*[foo*](/uri)";
    let result = markdown_to_html(input, options::DEFAULT);
    println!("Input: {:?}", input);
    println!("Result: {:?}", result);

    // Expected: <p>*<a href="/uri">foo*</a></p>
    // The first * should be plain text
    // The link text should contain foo*
    assert!(result.contains("foo*"), "Link text should contain foo*");
}

#[test]
fn debug_emphasis_only() {
    // Simple emphasis case
    let input = "*foo*";
    let result = markdown_to_html(input, options::DEFAULT);
    println!("Input: {:?}", input);
    println!("Result: {:?}", result);

    // Expected: <p><em>foo</em></p>
    assert!(result.contains("<em>"), "Should contain emphasis");
}
