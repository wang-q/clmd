use md::{markdown_to_html, options};

#[test]
fn debug_emphasis_link() {
    // Test #521: *[foo*](/uri)
    let input = "*[foo*](/uri)";
    let expected = "<p>*<a href=\"/uri\">foo*</a></p>";
    let result = markdown_to_html(input, options::DEFAULT);

    println!("Input: {:?}", input);
    println!("Expected: {:?}", expected);
    println!("Got:      {:?}", result);

    assert_eq!(result, expected);
}

#[test]
fn debug_emphasis_link_ref() {
    // Test #534: *[foo*][ref]
    let input = "*[foo*][ref]\n\n[ref]: /uri";
    let expected = "<p>*<a href=\"/uri\">foo*</a></p>";
    let result = markdown_to_html(input, options::DEFAULT);

    println!("Input: {:?}", input);
    println!("Expected: {:?}", expected);
    println!("Got:      {:?}", result);

    assert_eq!(result, expected);
}
