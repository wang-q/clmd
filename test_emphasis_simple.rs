use clmd::markdown_to_html;
use clmd::parser::options::Options;

fn main() {
    let test_cases = vec![
        ("*(*foo*)*", "<p><em>(<em>foo</em>)</em></p>"),
        ("_(_foo_)_", "<p><em>(<em>foo</em>)</em></p>"),
        ("*foo*", "<p><em>foo</em></p>"),
        ("**foo**", "<p><strong>foo</strong></p>"),
        ("*foo**bar*", "<p><em>foo</em><em>bar</em></p>"),
        ("__foo, __bar__, baz__", "<p><strong>foo, <strong>bar</strong>, baz</strong></p>"),
        ("*foo [bar](/url)*", "<p><em>foo <a href=\"/url\">bar</a></em></p>"),
        ("*foo\nbar*", "<p><em>foo\nbar</em></p>"),
    ];

    let options = Options::default();

    for (input, expected) in test_cases {
        let result = markdown_to_html(input, &options);
        if result == expected {
            println!("✓ PASS: {:?}", input);
        } else {
            println!("✗ FAIL: {:?}", input);
            println!("  Expected: {:?}", expected);
            println!("  Got:      {:?}", result);
        }
    }
}
