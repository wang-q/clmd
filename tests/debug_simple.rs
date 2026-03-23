use md::{markdown_to_html, options};

#[test]
fn debug_simple() {
    // Simple test case
    let input = "[foo*](/uri)";
    let result = markdown_to_html(input, options::DEFAULT);
    println!("Input: {:?}", input);
    println!("Result: {:?}", result);

    // Expected: <p><a href="/uri">foo*</a></p>
    assert!(result.contains("foo*"), "Link text should contain foo*");
}
