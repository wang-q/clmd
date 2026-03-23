// Debug test for emphasis parsing
use md::{markdown_to_html, options};

#[test]
fn debug_emphasis() {
    // Test basic emphasis
    let result = markdown_to_html("*foo bar*", options::DEFAULT);
    println!("Input: '*foo bar*'");
    println!("Output: '{}'", result);
    println!("Expected: '<p><em>foo bar</em></p>'");
    
    // Check if it contains em tag
    if result.contains("<em>") {
        println!("SUCCESS: Contains <em> tag");
    } else {
        println!("FAILURE: Does not contain <em> tag");
    }
}

#[test]
fn debug_strong() {
    let result = markdown_to_html("**foo bar**", options::DEFAULT);
    println!("Input: '**foo bar**'");
    println!("Output: '{}'", result);
    println!("Expected: '<p><strong>foo bar</strong></p>'");
}
