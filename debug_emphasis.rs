use clmd::Arena;
use clmd::parser::options::Options;
use clmd::parse_document;
use clmd::format_html;

fn main() {
    let input = "*(*foo*)*";
    let arena = Arena::new();
    let options = Options::default();
    let root = parse_document(&arena, input, &options);
    
    let mut html = String::new();
    format_html(root, &options, &mut html).unwrap();
    
    println!("Input: {:?}", input);
    println!("Output: {:?}", html);
    println!("Expected: {:?}", "<p><em>(<em>foo</em>)</em></p>");
}
