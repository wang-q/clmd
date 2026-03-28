use clmd::{markdown_to_html, Options};

fn main() {
    let options = Options::default();
    
    // Test 1: Simple emphasis
    let html1 = markdown_to_html("*italic*", &options);
    println!("Test 1 - *italic*: {}", html1);
    
    // Test 2: Simple strong
    let html2 = markdown_to_html("**bold**", &options);
    println!("Test 2 - **bold**: {}", html2);
    
    // Test 3: Both
    let html3 = markdown_to_html("*italic* and **bold**", &options);
    println!("Test 3 - *italic* and **bold**: {}", html3);
    
    // Test 4: Underscore
    let html4 = markdown_to_html("_italic_ and __bold__", &options);
    println!("Test 4 - _italic_ and __bold__: {}", html4);
}
