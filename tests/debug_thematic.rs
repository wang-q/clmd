use md::{markdown_to_html, options};

#[test]
fn debug_thematic_in_list() {
    let input = "- Foo\n- * * *";
    let expected = "<ul>\n<li>Foo</li>\n<li>\n<hr />\n</li>\n</ul>";
    let result = markdown_to_html(input, options::DEFAULT);

    println!("Input: {:?}", input);
    println!("Expected: {:?}", expected);
    println!("Got:      {:?}", result);

    assert_eq!(result, expected);
}

#[test]
fn debug_thematic_in_list_loose() {
    let input = "- Foo\n\n- * * *";
    let result = markdown_to_html(input, options::DEFAULT);

    println!("Loose list input: {:?}", input);
    println!("Result: {:?}", result);
}

#[test]
fn debug_simple_thematic() {
    let input = "* * *";
    let result = markdown_to_html(input, options::DEFAULT);

    println!("Simple thematic: {:?}", input);
    println!("Result: {:?}", result);
}
