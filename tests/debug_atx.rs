use md::{markdown_to_html, options};

#[test]
fn debug_atx_10() {
    let input = "#\tFoo";
    let result = markdown_to_html(input, options::DEFAULT);
    println!("Input: {:?}", input);
    println!("Result: {:?}", result);
    // Expected: "<h1>Foo</h1>"
}

#[test]
fn debug_atx_simple() {
    let input = "# Hello";
    let result = markdown_to_html(input, options::DEFAULT);
    println!("Input: {:?}", input);
    println!("Result: {:?}", result);
    // Expected: "<h1>Hello</h1>"
}
