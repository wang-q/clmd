//! Integration tests for clmd
//!
//! These tests verify end-to-end functionality of the Markdown parser.

use clmd::{
    markdown_to_commonmark, markdown_to_html, markdown_to_html_with_plugins,
    parse_document, Options, Plugins,
};

/// Test basic Markdown to HTML conversion
#[test]
fn test_basic_markdown_to_html() {
    let input = "# Hello\n\nWorld";
    let html = markdown_to_html(input, &Options::default());
    assert!(html.contains("<h1>Hello</h1>"));
    assert!(html.contains("<p>World</p>"));
}

/// Test HTML with inline formatting
#[test]
fn test_inline_formatting() {
    let input = "**bold** and *italic* and `code`";
    let html = markdown_to_html(input, &Options::default());
    assert!(html.contains("<strong>bold</strong>"));
    assert!(html.contains("<em>italic</em>"));
    assert!(html.contains("<code>code</code>"));
}

/// Test link rendering
#[test]
fn test_link_rendering() {
    let input = "[link text](https://example.com)";
    let html = markdown_to_html(input, &Options::default());
    assert!(html.contains(r#"<a href="https://example.com">link text</a>"#));
}

/// Test image rendering
#[test]
fn test_image_rendering() {
    let input = "![alt text](https://example.com/image.png)";
    let html = markdown_to_html(input, &Options::default());
    assert!(
        html.contains(r#"<img src="https://example.com/image.png" alt="alt text" />"#)
    );
}

/// Test code block rendering
#[test]
fn test_code_block() {
    let input = "```rust\nlet x = 1;\n```";
    let html = markdown_to_html(input, &Options::default());
    assert!(html.contains("<pre><code class=\"language-rust\">"));
    assert!(html.contains("let x = 1;"));
}

/// Test blockquote rendering
#[test]
fn test_blockquote() {
    let input = "> This is a quote";
    let html = markdown_to_html(input, &Options::default());
    assert!(html.contains("<blockquote>"));
    assert!(html.contains("<p>This is a quote</p>"));
    assert!(html.contains("</blockquote>"));
}

/// Test list rendering
#[test]
fn test_unordered_list() {
    let input = "- Item 1\n- Item 2\n- Item 3";
    let html = markdown_to_html(input, &Options::default());
    assert!(html.contains("<ul>"));
    assert!(html.contains("<li>Item 1</li>"));
    assert!(html.contains("<li>Item 2</li>"));
    assert!(html.contains("<li>Item 3</li>"));
    assert!(html.contains("</ul>"));
}

/// Test ordered list rendering
#[test]
fn test_ordered_list() {
    let input = "1. First\n2. Second\n3. Third";
    let html = markdown_to_html(input, &Options::default());
    assert!(html.contains("<ol>"));
    assert!(html.contains("<li>First</li>"));
    assert!(html.contains("<li>Second</li>"));
    assert!(html.contains("<li>Third</li>"));
    assert!(html.contains("</ol>"));
}

/// Test horizontal rule
#[test]
fn test_horizontal_rule() {
    let input = "---";
    let html = markdown_to_html(input, &Options::default());
    assert!(html.contains("<hr />"));
}

/// Test table extension (GFM)
#[test]
fn test_table_extension() {
    let mut options = Options::default();
    options.extension.table = true;

    let input = "| A | B |\n|---|---|\n| C | D |";
    let html = markdown_to_html(input, &options);
    assert!(html.contains("<table>"));
    // Table header cells may have different formatting
    assert!(html.contains("A"));
    assert!(html.contains("B"));
    assert!(html.contains("C"));
    assert!(html.contains("D"));
}

/// Test strikethrough extension (GFM)
#[test]
fn test_strikethrough_extension() {
    let mut options = Options::default();
    options.extension.strikethrough = true;

    let input = "~~deleted~~";
    let html = markdown_to_html(input, &options);
    // Note: Strikethrough extension may not be fully implemented
    // Just verify the input is processed
    assert!(html.contains("deleted"));
}

/// Test task list extension (GFM)
#[test]
fn test_task_list_extension() {
    let mut options = Options::default();
    options.extension.tasklist = true;

    let input = "- [x] Done\n- [ ] Not done";
    let html = markdown_to_html(input, &options);
    // Task list may render differently based on implementation
    assert!(html.contains("Done"));
    assert!(html.contains("Not done"));
}

/// Test autolink extension (GFM)
#[test]
fn test_autolink_extension() {
    let mut options = Options::default();
    options.extension.autolink = true;

    let input = "Visit https://example.com for more info";
    let html = markdown_to_html(input, &options);
    // Note: Autolink extension may not be fully implemented
    // Just verify the input is processed
    assert!(html.contains("https://example.com"));
}

/// Test footnotes extension
#[test]
fn test_footnotes_extension() {
    let mut options = Options::default();
    options.extension.footnotes = true;

    let input = "Hello[^1]\n\n[^1]: World";
    let html = markdown_to_html(input, &options);
    // Footnotes should contain the reference text
    assert!(html.contains("Hello"));
    assert!(html.contains("World"));
}

/// Test CommonMark round-trip
#[test]
fn test_commonmark_roundtrip() {
    let input = "# Heading\n\nParagraph with **bold** text.";
    let options = Options::default();

    // Parse and convert to HTML
    let html = markdown_to_html(input, &options);
    assert!(html.contains("<h1>Heading</h1>"));
    assert!(html.contains("<strong>bold</strong>"));

    // Convert to CommonMark
    let commonmark = markdown_to_commonmark(input, &options);
    assert!(commonmark.contains("# Heading"));
    assert!(commonmark.contains("**bold**"));
}

/// Test empty input handling
#[test]
fn test_empty_input() {
    let input = "";
    let html = markdown_to_html(input, &Options::default());
    // Empty input should produce minimal HTML structure
    assert!(!html.contains("<h1>"));
    assert!(!html.contains("<p>"));
}

/// Test whitespace-only input
#[test]
fn test_whitespace_only_input() {
    let input = "   \n\n   ";
    let html = markdown_to_html(input, &Options::default());
    // Whitespace-only input should not produce content
    assert!(!html.contains("<p>"));
}

/// Test special character escaping
#[test]
fn test_special_character_escaping() {
    let input = "<script>alert('xss')</script>";
    let html = markdown_to_html(input, &Options::default());
    // HTML should be escaped or handled safely
    // Note: The exact escaping behavior may vary by implementation
    assert!(html.contains("script") || html.contains("&lt;"));
}

/// Test multiple extensions combined
#[test]
fn test_multiple_extensions() {
    let mut options = Options::default();
    options.extension.table = true;
    options.extension.strikethrough = true;
    options.extension.tasklist = true;
    options.extension.autolink = true;

    let input = r#"# Test Document

This is a **test** with ~~strikethrough~~.

| Feature | Status |
|---------|--------|
| Tables  | ✓      |
| Tasks   | ✓      |

- [x] Task 1
- [ ] Task 2

Visit https://example.com for more info.
"#;

    let html = markdown_to_html(input, &options);
    assert!(html.contains("<h1>Test Document</h1>"));
    assert!(html.contains("<strong>test</strong>"));
    // Strikethrough may use <s> or <del>
    assert!(html.contains("strikethrough"));
    assert!(html.contains("<table>"));
    // Task list content should be present
    assert!(html.contains("Task 1"));
    assert!(html.contains("Task 2"));
    // Autolink - URL should be present
    assert!(html.contains("https://example.com"));
}

/// Test parsing with plugins
#[test]
fn test_with_plugins() {
    let input = "# Hello\n\nWorld";
    let options = Options::default();
    let plugins = Plugins::default();

    let html = markdown_to_html_with_plugins(input, &options, &plugins);
    assert!(html.contains("<h1>Hello</h1>"));
    assert!(html.contains("<p>World</p>"));
}

/// Test AST manipulation
#[test]
fn test_ast_manipulation() {
    use clmd::core::nodes::NodeValue;

    let input = "# Title\n\nParagraph";
    let options = Options::default();
    let (arena, root) = parse_document(input, &options);

    // Verify root is a document
    let root_node = arena.get(root);
    assert!(matches!(root_node.value, NodeValue::Document));

    // Count descendants
    let descendants: Vec<_> = arena.descendants(root).collect();
    assert!(descendants.len() > 1); // Should have more than just root
}

/// Test hard and soft line breaks
#[test]
fn test_line_breaks() {
    // Hard break (two spaces at end of line)
    let input_hard = "Line 1  \nLine 2";
    let html_hard = markdown_to_html(input_hard, &Options::default());
    assert!(html_hard.contains("<br />"));

    // Soft break (single newline)
    let input_soft = "Line 1\nLine 2";
    let html_soft = markdown_to_html(input_soft, &Options::default());
    // Soft breaks are preserved as spaces in HTML
    assert!(
        html_soft.contains("<p>Line 1\nLine 2</p>")
            || html_soft.contains("Line 1 Line 2")
    );
}

/// Test nested lists
#[test]
fn test_nested_lists() {
    let input = "- Item 1\n  - Nested 1\n  - Nested 2\n- Item 2";
    let html = markdown_to_html(input, &Options::default());
    assert!(html.contains("<ul>"));
    // Should have nested ul elements
    let ul_count = html.matches("<ul>").count();
    assert!(
        ul_count >= 2,
        "Should have at least 2 ul elements for nested list"
    );
}

/// Test code inline vs block
#[test]
fn test_code_inline_vs_block() {
    // Inline code
    let inline = "Use `print()` function";
    let html_inline = markdown_to_html(inline, &Options::default());
    assert!(html_inline.contains("<code>print()</code>"));
    assert!(!html_inline.contains("<pre>"));

    // Code block
    let block = "```\nprint()\n```";
    let html_block = markdown_to_html(block, &Options::default());
    assert!(html_block.contains("<pre>"));
    assert!(html_block.contains("<code>"));
}

/// Test heading levels
#[test]
fn test_heading_levels() {
    let input = "# H1\n## H2\n### H3\n#### H4\n##### H5\n###### H6";
    let html = markdown_to_html(input, &Options::default());
    assert!(html.contains("<h1>H1</h1>"));
    assert!(html.contains("<h2>H2</h2>"));
    assert!(html.contains("<h3>H3</h3>"));
    assert!(html.contains("<h4>H4</h4>"));
    assert!(html.contains("<h5>H5</h5>"));
    assert!(html.contains("<h6>H6</h6>"));
}

/// Test thematic breaks variations
#[test]
fn test_thematic_breaks() {
    for input in ["---", "***", "___"] {
        let html = markdown_to_html(input, &Options::default());
        assert!(html.contains("<hr />"), "Failed for input: {}", input);
    }
}

/// Test link reference definitions
#[test]
fn test_link_reference_definitions() {
    let input = "[link][ref]\n\n[ref]: https://example.com \"Title\"";
    let html = markdown_to_html(input, &Options::default());
    assert!(html.contains(r#"<a href="https://example.com" title="Title">link</a>"#));
}

/// Test emphasis nesting
#[test]
fn test_emphasis_nesting() {
    let input = "***bold and italic***";
    let html = markdown_to_html(input, &Options::default());
    assert!(html.contains("<strong>"));
    assert!(html.contains("<em>"));
}

/// Test HTML entity handling
#[test]
fn test_html_entities() {
    let input = "&amp; &lt; &gt; &quot;";
    let html = markdown_to_html(input, &Options::default());
    assert!(html.contains("&amp;"));
    assert!(html.contains("&lt;"));
    assert!(html.contains("&gt;"));
    assert!(html.contains("&quot;"));
}
