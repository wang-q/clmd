use clmd::{markdown_to_html, options};

#[test]
fn test_basic_link_reference() {
    let input = "[foo]\n\n[foo]: /bar\n";
    let result = markdown_to_html(input, options::DEFAULT);
    println!("Input: {:?}", input);
    println!("Output: {:?}", result);
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
    println!("Input: {:?}", input);
    println!("Output: {:?}", result);
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
    println!("Input: {:?}", input);
    println!("Output: {:?}", result);
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
    println!("Input: {:?}", input);
    println!("Output: {:?}", result);
    assert!(
        result.contains("<a href=\"/bar\" title=\"title\">foo</a>"),
        "Expected inline link with title, got: {}",
        result
    );
}
