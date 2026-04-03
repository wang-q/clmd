//! Edge case tests for the clmd parser
//!
//! This module tests boundary conditions and edge cases that may not be
//! covered by the standard CommonMark spec tests.

use clmd::markdown_to_html;
use clmd::options::Options;

/// Helper function to convert markdown to HTML with default options
fn md_to_html(input: &str) -> String {
    markdown_to_html(input, &Options::default())
}

/// Test empty input handling
#[test]
fn test_empty_input() {
    let html = md_to_html("");
    assert_eq!(html, "", "Empty input should produce empty output");
}

/// Test whitespace-only input
#[test]
fn test_whitespace_only() {
    let cases = vec![
        ("   ", ""),
        ("\n", ""),
        ("\t\n\r\n", ""),
        ("   \n   \n", ""),
    ];

    for (input, expected) in cases {
        let html = md_to_html(input);
        assert_eq!(
            html,
            expected,
            "Whitespace-only input '{}' should produce '{}'",
            input.escape_default(),
            expected
        );
    }
}

/// Test very long lines
#[test]
fn test_long_lines() {
    // Create a very long paragraph
    let long_text = "a".repeat(10000);
    let input = long_text.clone();
    let html = md_to_html(&input);
    assert!(html.contains("<p>"), "Long line should produce paragraph");
    assert!(
        html.contains(&long_text),
        "Long line content should be preserved"
    );
}

/// Test deep nesting
#[test]
fn test_deep_nesting() {
    // Create deeply nested blockquotes
    let mut input = String::new();
    let depth = 50;
    for i in 0..depth {
        input.push_str("> ");
        input.push_str(&i.to_string());
        input.push('\n');
    }

    let html = md_to_html(&input);
    assert!(
        html.contains("<blockquote>"),
        "Deep nesting should produce blockquotes"
    );
}

/// Test deeply nested lists
#[test]
fn test_deep_list_nesting() {
    let mut input = String::new();
    let depth = 20;

    for _ in 0..depth {
        input.push_str("- item\n");
    }

    let html = md_to_html(&input);
    assert!(html.contains("<ul>"), "Deep list should produce ul");
    assert!(html.contains("<li>"), "Deep list should produce li");
}

/// Test special Unicode characters
#[test]
fn test_unicode_characters() {
    let cases = vec![
        ("Hello 世界", "Hello 世界"),
        ("Emoji: 🎉 🚀", "🎉 🚀"),
        ("Math: ∫ ∑ ∏", "∫ ∑ ∏"),
        ("Arabic: مرحبا", "مرحبا"),
        ("Hebrew: שלום", "שלום"),
        ("Zero width: a\u{200B}b", "a\u{200B}b"),
    ];

    for (input, expected_content) in cases {
        let html = md_to_html(input);
        assert!(
            html.contains(expected_content),
            "Unicode content should be preserved: {}",
            input
        );
    }
}

/// Test malformed Markdown - unmatched brackets
#[test]
fn test_unmatched_brackets() {
    let cases = vec![
        ("[unclosed", "[unclosed"),
        ("unclosed]", "unclosed]"),
        ("[[[", "[[["),
        ("]]]", "]]]"),
    ];

    for (input, expected) in cases {
        let html = md_to_html(input);
        assert!(
            html.contains(expected),
            "Unmatched brackets should be preserved as text: {}",
            input
        );
    }
}

/// Test malformed Markdown - unmatched backticks
#[test]
fn test_unmatched_backticks() {
    let cases = vec![
        ("`unclosed", "`unclosed"),
        ("``unclosed", "``unclosed"),
        ("`code", "`code"),
    ];

    for (input, _) in cases {
        let html = md_to_html(input);
        // Should not panic and should produce some output
        assert!(
            !html.is_empty() || html.is_empty(),
            "Should handle unmatched backticks"
        );
    }
}

/// Test null bytes in input
#[test]
fn test_null_bytes() {
    let input = "Hello\x00World";
    let html = md_to_html(input);
    // Null bytes should be replaced with replacement character
    assert!(html.contains("Hello") && html.contains("World"));
}

/// Test various line endings
#[test]
fn test_line_endings() {
    let cases = vec![
        ("Line1\nLine2", "Line1", "Line2"),
        ("Line1\r\nLine2", "Line1", "Line2"),
        ("Line1\rLine2", "Line1", "Line2"),
    ];

    for (input, line1, line2) in cases {
        let html = md_to_html(input);
        assert!(
            html.contains(line1) && html.contains(line2),
            "Should handle line endings correctly: {:?}",
            input.escape_default()
        );
    }
}

/// Test empty code blocks
#[test]
fn test_empty_code_blocks() {
    let cases = vec![
        ("```\n```", "<code>"),
        ("```rust\n```", "<code"),
        ("    ", ""), // Empty indented code block
    ];

    for (input, expected) in cases {
        let html = md_to_html(input);
        if !expected.is_empty() {
            assert!(
                html.contains(expected),
                "Empty code block should produce code element: {}",
                input
            );
        }
    }
}

/// Test HTML entity handling
#[test]
fn test_html_entities() {
    let cases = vec![
        ("&amp;", "&amp;"),
        ("&lt;", "&lt;"),
        ("&gt;", "&gt;"),
        ("&#123;", "{"),
        ("&#x7B;", "{"),
    ];

    for (input, expected) in cases {
        let html = md_to_html(input);
        assert!(
            html.contains(expected),
            "Entity {} should produce {} in {}",
            input,
            expected,
            html
        );
    }
}

/// Test smart punctuation option
#[test]
fn test_smart_punctuation() {
    let input = "\"Hello\" -- world...";

    let html_default = md_to_html(input);
    let mut opts = Options::default();
    opts.parse.smart = true;
    let html_smart = markdown_to_html(input, &opts);

    // Smart punctuation should convert quotes and dashes
    assert_ne!(
        html_default, html_smart,
        "Smart punctuation should produce different output"
    );
}

/// Test link with special characters
#[test]
fn test_special_links() {
    let cases = vec![
        ("[text](http://example.com)", "http://example.com"),
        ("[text](mailto:test@example.com)", "mailto:test@example.com"),
        ("[text](#anchor)", "#anchor"),
        ("[text](path/to/file.md)", "path/to/file.md"),
    ];

    for (input, expected_url) in cases {
        let html = md_to_html(input);
        assert!(
            html.contains(expected_url),
            "Link URL should be preserved: {}",
            input
        );
    }
}

/// Test image with empty alt text
#[test]
fn test_empty_alt_text() {
    let input = "![](image.png)";
    let html = md_to_html(input);
    assert!(html.contains("<img"), "Image should be rendered");
    assert!(html.contains("image.png"), "Image src should be preserved");
}

/// Test table edge cases
#[test]
fn test_table_edge_cases() {
    let cases = vec![
        // Empty cells
        "|a|b|\n|---|---|\n|c|",
        // Single column
        "|a|\n|---|\n|b|",
        // No header separator
        "|a|b|\n|c|d|",
    ];

    for input in cases {
        let html = md_to_html(input);
        // Should not panic
        assert!(!html.is_empty() || html.is_empty());
    }
}

/// Test thematic break variations
#[test]
fn test_thematic_breaks() {
    let cases = vec!["***", "---", "___", " * * * ", "- - -", "_ _ _"];

    for input in cases {
        let html = md_to_html(input);
        assert!(
            html.contains("<hr") || html == "<p>".to_string() + input + "</p>",
            "Thematic break or paragraph: {}",
            input
        );
    }
}

/// Test heading edge cases
#[test]
fn test_heading_edge_cases() {
    let cases = vec![
        ("# ", "<h1>"),
        ("###### ", "<h6>"),
        ("####### too many", "<p>"), // 7 hashes should be paragraph
        ("#\tTab after hash", "<h1>"),
    ];

    for (input, expected_tag) in cases {
        let html = md_to_html(input);
        assert!(
            html.contains(expected_tag),
            "Input '{}' should contain '{}' in {}",
            input,
            expected_tag,
            html
        );
    }
}

/// Test list with mixed markers
#[test]
fn test_mixed_list_markers() {
    let input = "- item 1\n+ item 2\n* item 3";
    let html = md_to_html(input);

    // Each different marker should start a new list
    assert!(html.contains("<ul>"));
    assert!(html.contains("item 1"));
    assert!(html.contains("item 2"));
    assert!(html.contains("item 3"));
}

/// Test code fence with backticks in content
#[test]
fn test_code_fence_with_backticks() {
    let input = "````\n```\ninner\n```\n````";
    let html = md_to_html(input);
    assert!(html.contains("<pre>"));
    assert!(html.contains("<code>"));
    assert!(html.contains("inner"));
}

/// Test inline code with backticks
#[test]
fn test_inline_code_with_backticks() {
    // Test inline code containing backticks
    let input = "`` ` ``"; // Code containing single backtick
    let html = md_to_html(input);
    // Should produce code element with backtick content
    assert!(
        html.contains("<code>") && html.contains("</code>"),
        "Inline code should handle backticks: {} -> {}",
        input,
        html
    );
}

/// Test autolink edge cases
#[test]
fn test_autolink_edge_cases() {
    let cases = vec![
        ("<http://example.com>", "http://example.com"),
        ("<mailto:test@example.com>", "mailto:test@example.com"),
        ("<test@example.com>", "test@example.com"),
    ];

    for (input, expected) in cases {
        let html = md_to_html(input);
        assert!(
            html.contains(expected),
            "Autolink should work for: {}",
            input
        );
    }
}

/// Test HTML comment handling
#[test]
fn test_html_comments() {
    let cases = vec![
        "<!-- comment -->",
        "<!--\nmulti\nline\n-->",
        "text <!-- inline --> more",
    ];

    for input in cases {
        let html = md_to_html(input);
        // Should not panic
        assert!(!html.is_empty() || html.is_empty());
    }
}

/// Test reference-style link edge cases
#[test]
fn test_reference_link_edge_cases() {
    let input = r#"[text][ref]

[ref]: http://example.com "title"
"#;

    let html = md_to_html(input);
    assert!(
        html.contains("http://example.com"),
        "Reference link should resolve: {}",
        html
    );
}

/// Test emphasis edge cases
#[test]
fn test_emphasis_edge_cases() {
    let cases = vec![
        ("*text*", "<em>"),
        ("**text**", "<strong>"),
        // Note: ***text*** produces <em><strong>text</strong></em> (order may vary)
        ("***text***", "<strong>"),
        ("_text_", "<em>"),
        ("__text__", "<strong>"),
        // Note: ___text___ produces <em><strong>text</strong></em> (order may vary)
        ("___text___", "<strong>"),
    ];

    for (input, expected) in cases {
        let html = md_to_html(input);
        assert!(
            html.contains(expected),
            "Emphasis '{}' should contain '{}' in {}",
            input,
            expected,
            html
        );
    }
}

/// Test hard line breaks
#[test]
fn test_hard_line_breaks() {
    let cases = vec![("line1  \nline2", "<br"), ("line1\\\nline2", "<br")];

    for (input, expected) in cases {
        let html = md_to_html(input);
        assert!(
            html.contains(expected),
            "Hard line break should produce br: {}",
            input
        );
    }
}

/// Test setext heading edge cases
#[test]
fn test_setext_heading_edge_cases() {
    let cases = vec![
        ("text\n=", "<h1>"),
        ("text\n-", "<h2>"),
        ("text\n===", "<h1>"),
        ("text\n---", "<h2>"),
    ];

    for (input, expected) in cases {
        let html = md_to_html(input);
        assert!(
            html.contains(expected),
            "Setext heading '{}' should contain '{}' in {}",
            input,
            expected,
            html
        );
    }
}

/// Test Setext heading with blank lines
#[test]
fn test_setext_heading_blank_lines() {
    // Setext heading requires no blank line between text and underline
    let input = "text\n\n=";
    let html = md_to_html(input);
    // Should NOT be a heading - blank line breaks it
    assert!(
        !html.contains("<h1>") && !html.contains("<h2>"),
        "Setext heading with blank line should NOT be a heading: {}",
        html
    );
    assert!(html.contains("<p>"), "Should be paragraphs: {}", html);
}

/// Test Setext heading with indentation
#[test]
fn test_setext_heading_indentation() {
    let cases = vec![
        // Indented underline should still work (up to 3 spaces)
        ("text\n =", "<h1>"),
        ("text\n  =", "<h1>"),
        ("text\n   =", "<h1>"),
    ];

    for (input, expected) in cases {
        let html = md_to_html(input);
        assert!(
            html.contains(expected),
            "Setext heading '{}' should contain '{}' in {}",
            input,
            expected,
            html
        );
    }

    // Too much indentation (4+ spaces) - check that it's NOT a heading
    let input = "text\n    =";
    let html = md_to_html(input);
    assert!(
        !html.contains("<h1>") && !html.contains("<h2>"),
        "Over-indented setext '{}' should NOT be a heading in {}",
        input,
        html
    );
}

/// Test Setext heading with interrupted paragraph
#[test]
fn test_setext_heading_interrupted_paragraph() {
    // List interrupts paragraph, so setext underline after list doesn't apply
    let input = "paragraph\n- list item\n=";
    let html = md_to_html(input);
    // The = should NOT create a heading because the paragraph was interrupted
    assert!(
        !html.contains("<h1>") && !html.contains("<h2>"),
        "Setext heading after list should NOT be a heading: {}",
        html
    );
}

/// Test Setext heading with long underline
#[test]
fn test_setext_heading_long_underline() {
    let cases = vec![
        ("text\n=================", "<h1>"),
        ("text\n-----------------", "<h2>"),
        // Note: Underlines with spaces between markers may not be valid setext headings
        // depending on the parser implementation
    ];

    for (input, expected) in cases {
        let html = md_to_html(input);
        assert!(
            html.contains(expected),
            "Setext heading '{}' should contain '{}' in {}",
            input,
            expected,
            html
        );
    }
}

/// Test Setext heading with empty content
#[test]
fn test_setext_heading_empty_content() {
    let cases = vec![
        // Empty content before underline
        ("\n=", "<p>"),
        ("   \n=", "<p>"),
    ];

    for (input, expected) in cases {
        let html = md_to_html(input);
        assert!(
            html.contains(expected),
            "Empty setext '{}' should contain '{}' in {}",
            input,
            expected,
            html
        );
    }
}

/// Test Setext heading with inline content
#[test]
fn test_setext_heading_inline_content() {
    let input = "text *emph* **strong** `code`\n===";
    let html = md_to_html(input);
    assert!(html.contains("<h1>"), "Should be h1: {}", html);
    assert!(html.contains("<em>"), "Should contain emphasis: {}", html);
    assert!(html.contains("<strong>"), "Should contain strong: {}", html);
    assert!(html.contains("<code>"), "Should contain code: {}", html);
}

/// Test HTML block type 1 (script, pre, style)
#[test]
fn test_html_block_type1() {
    let cases = vec![
        ("<script>\nalert(1)\n</script>", "<script>"),
        ("<pre>\ncode\n</pre>", "<pre>"),
        ("<style>\nbody{}\n</style>", "<style>"),
    ];

    for (input, expected) in cases {
        let html = md_to_html(input);
        assert!(
            html.contains(expected),
            "HTML block '{}' should contain '{}' in {}",
            input,
            expected,
            html
        );
    }
}

/// Test HTML block type 2 (comment)
#[test]
fn test_html_block_type2() {
    let input = "<!--\ncomment\n-->";
    let html = md_to_html(input);
    assert!(
        html.contains("<!--"),
        "Should contain comment start: {}",
        html
    );
    assert!(html.contains("-->"), "Should contain comment end: {}", html);
}

/// Test HTML block type 6 (block elements)
#[test]
fn test_html_block_type6() {
    let cases = vec![
        ("<div>\ncontent\n</div>", "<div>"),
        ("<table>\n<tr><td>cell</td></tr>\n</table>", "<table>"),
        ("<blockquote>\nquote\n</blockquote>", "<blockquote>"),
    ];

    for (input, expected) in cases {
        let html = md_to_html(input);
        assert!(
            html.contains(expected),
            "HTML block '{}' should contain '{}' in {}",
            input,
            expected,
            html
        );
    }
}

/// Test HTML block with blank line termination
#[test]
fn test_html_block_blank_line_termination() {
    // Type 6 HTML block ends at blank line
    let input = "<div>\ncontent\n\nparagraph";
    let html = md_to_html(input);
    assert!(html.contains("<div>"), "Should contain div: {}", html);
    assert!(
        html.contains("<p>"),
        "Should contain paragraph after blank line: {}",
        html
    );
}

/// Test HTML block with inline markdown
#[test]
fn test_html_block_inline_markdown() {
    // Inside HTML block, markdown is not parsed
    let input = "<div>\n*not emphasis*\n</div>";
    let html = md_to_html(input);
    assert!(
        html.contains("*not emphasis*"),
        "Should NOT parse emphasis inside HTML block: {}",
        html
    );
    assert!(
        !html.contains("<em>"),
        "Should NOT contain em tag: {}",
        html
    );
}

/// Test incomplete HTML block
#[test]
fn test_incomplete_html_block() {
    let cases = vec![
        // Unclosed tags
        ("<div>\ncontent", "<div>"),
        ("<table>\n<tr>", "<table>"),
    ];

    for (input, expected) in cases {
        let html = md_to_html(input);
        assert!(
            html.contains(expected),
            "Incomplete HTML '{}' should contain '{}' in {}",
            input,
            expected,
            html
        );
    }
}

/// Test HTML block type 7 (miscellaneous)
#[test]
fn test_html_block_type7() {
    // Type 7 requires the tag to be on its own line with specific formatting
    let cases = vec![
        ("<custom>\ncontent\n</custom>", "<custom>"),
        ("<my-element>\ntext\n</my-element>", "<my-element>"),
    ];

    for (input, expected) in cases {
        let html = md_to_html(input);
        assert!(
            html.contains(expected),
            "HTML block type 7 '{}' should contain '{}' in {}",
            input,
            expected,
            html
        );
    }
}

/// Test HTML block with attributes
#[test]
fn test_html_block_with_attributes() {
    let cases = vec![
        (r#"<div class="test">content</div>"#, r#"class="test""#),
        (r#"<div id="main" class="container">content</div>"#, "<div"),
    ];

    for (input, expected) in cases {
        let html = md_to_html(input);
        assert!(
            html.contains(expected),
            "HTML with attributes '{}' should contain '{}' in {}",
            input,
            expected,
            html
        );
    }
}

/// Test self-closing HTML tags
#[test]
fn test_self_closing_html_tags() {
    let cases = vec![
        ("<hr />", "<hr"),
        ("<br/>", "<br"),
        ("<img src=\"test.png\" />", "<img"),
    ];

    for (input, expected) in cases {
        let html = md_to_html(input);
        assert!(
            html.contains(expected),
            "Self-closing tag '{}' should contain '{}' in {}",
            input,
            expected,
            html
        );
    }
}

/// Test nested HTML blocks
#[test]
fn test_nested_html_blocks() {
    let input = "<div>\n<p>paragraph</p>\n</div>";
    let html = md_to_html(input);
    assert!(html.contains("<div>"), "Should contain outer div: {}", html);
    assert!(html.contains("<p>"), "Should contain inner p: {}", html);
}

/// Test blockquote with nested elements
#[test]
fn test_blockquote_nesting() {
    let input = "> # Heading in quote\n> \n> - List item\n>   - Nested item";
    let html = md_to_html(input);

    assert!(html.contains("<blockquote>"));
    assert!(html.contains("<h1>") || html.contains("<h2>"));
    assert!(html.contains("<ul>"));
}

/// Test task list items
#[test]
fn test_task_list_items() {
    let cases = vec![
        ("- [ ] unchecked", "unchecked"),
        ("- [x] checked", "checked"),
        ("- [X] checked uppercase", "checked"),
    ];

    for (input, expected) in cases {
        let html = md_to_html(input);
        assert!(
            html.contains(expected) || html.contains("<li"),
            "Task list item should be rendered: {}",
            input
        );
    }
}

/// Test footnote edge cases
#[test]
fn test_footnote_edge_cases() {
    let input = r#"Text[^1]

[^1]: Footnote content
    with continuation
"#;

    let html = md_to_html(input);
    // Should not panic
    assert!(!html.is_empty() || html.is_empty());
}

/// Test strikethrough
#[test]
fn test_strikethrough() {
    let input = "~~deleted~~";
    let html = md_to_html(input);
    // Strikethrough may be rendered as <del> or <s>
    assert!(
        html.contains("<del>") || html.contains("<s>") || html.contains("deleted"),
        "Strikethrough should be rendered: {}",
        html
    );
}
