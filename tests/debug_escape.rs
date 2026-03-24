use md::{markdown_to_html, options};

#[test]
fn debug_setext_102() {
    let input = "\\> foo\n------";
    let result = markdown_to_html(input, options::DEFAULT);
    println!("Input: {:?}", input);
    println!("Result: {:?}", result);
    // Expected: "<h2>&gt; foo</h2>"
}
