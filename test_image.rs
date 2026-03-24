// Test script for image parsing
use md::Parser;

fn main() {
    let input = r#"My ![foo bar](/path/to/train.jpg  "title"   )"#;
    let parser = Parser::new();
    let doc = parser.parse(input);
    let html = md::render_html(&doc, md::options::DEFAULT);
    println!("Input: {:?}", input);
    println!("Output: {:?}", html);
    
    let expected = r#"<p>My <img src="/path/to/train.jpg" alt="foo bar" title="title" /></p>"#;
    println!("Expected: {:?}", expected);
    println!("Match: {}", html == expected);
}
