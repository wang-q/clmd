use md::{markdown_to_html, options};

#[test]
fn debug_setext_88() {
    let input = "Foo\n= =\n\nFoo\n--- -";
    let result = markdown_to_html(input, options::DEFAULT);
    println!("Input: {:?}", input);
    println!("Result: {:?}", result);
    // Expected: "<p>Foo\n= =</p>\n<p>Foo</p>\n<hr />"
}

#[test]
fn debug_setext_simple() {
    let input = "Foo\n===";
    let result = markdown_to_html(input, options::DEFAULT);
    println!("Input: {:?}", input);
    println!("Result: {:?}", result);
    // Expected: "<h1>Foo</h1>"
}

#[test]
fn debug_setext_with_space() {
    let input = "Foo\n= =";
    let result = markdown_to_html(input, options::DEFAULT);
    println!("Input: {:?}", input);
    println!("Result: {:?}", result);
    // Expected: "<p>Foo\n= =</p>"
}
