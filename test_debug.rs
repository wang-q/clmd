// Debug test for emphasis processing
use clmd::markdown_to_html;
use clmd::parser::options::Options;

fn main() {
    let test_cases = vec![
        ("*(*foo*)*", "<p><em>(<em>foo</em>)</em></p>"),
        ("*foo*", "<p><em>foo</em></p>"),
        ("**foo**", "<p><strong>foo</strong></p>"),
    ];

    let options = Options::default();

    for (input, expected) in test_cases {
        let result = markdown_to_html(input, &options);
        println!("Input:    {:?}", input);
        println!("Expected: {:?}", expected);
        println!("Got:      {:?}", result);
        println!("Match:    {}", result == expected);
        println!();
    }
}
