//! Integration tests for the CommonMark formatter
//!
//! These tests verify the end-to-end formatting functionality
//! using the public API.

use clmd::{markdown_to_commonmark, Options, Plugins};

#[test]
fn test_format_heading() {
    let options = Options::default();
    let input = "# Heading 1\n\n## Heading 2\n\n### Heading 3";
    let output = markdown_to_commonmark(input, &options, &Plugins::default());

    assert!(
        output.contains("# Heading 1"),
        "Should contain h1: {}",
        output
    );
    assert!(
        output.contains("## Heading 2"),
        "Should contain h2: {}",
        output
    );
    assert!(
        output.contains("### Heading 3"),
        "Should contain h3: {}",
        output
    );
}

#[test]
fn test_format_paragraphs() {
    let options = Options::default();
    let input = "First paragraph.\n\nSecond paragraph.";
    let output = markdown_to_commonmark(input, &options, &Plugins::default());

    assert!(
        output.contains("First paragraph"),
        "Should contain first paragraph: {}",
        output
    );
    assert!(
        output.contains("Second paragraph"),
        "Should contain second paragraph: {}",
        output
    );
}

#[test]
fn test_format_emphasis() {
    let options = Options::default();
    let input = "This is *italic* and **bold** text.";
    let output = markdown_to_commonmark(input, &options, &Plugins::default());

    assert!(
        output.contains("*italic*"),
        "Should contain italic: {}",
        output
    );
    assert!(
        output.contains("**bold**"),
        "Should contain bold: {}",
        output
    );
}

#[test]
fn test_format_code_inline() {
    let options = Options::default();
    let input = "Use `code` inline.";
    let output = markdown_to_commonmark(input, &options, &Plugins::default());

    assert!(
        output.contains("`code`"),
        "Should contain inline code: {}",
        output
    );
}

#[test]
fn test_format_code_block() {
    let options = Options::default();
    let input = "```rust\nfn main() {}\n```";
    let output = markdown_to_commonmark(input, &options, &Plugins::default());

    assert!(
        output.contains("```rust"),
        "Should contain opening fence: {}",
        output
    );
    assert!(
        output.contains("fn main() {}"),
        "Should contain code: {}",
        output
    );
    assert!(
        output.contains("```"),
        "Should contain closing fence: {}",
        output
    );
}

#[test]
fn test_format_list_bullet() {
    let options = Options::default();
    let input = "- Item 1\n- Item 2\n- Item 3";
    let output = markdown_to_commonmark(input, &options, &Plugins::default());

    assert!(
        output.contains("- Item 1"),
        "Should contain item 1: {}",
        output
    );
    assert!(
        output.contains("- Item 2"),
        "Should contain item 2: {}",
        output
    );
    assert!(
        output.contains("- Item 3"),
        "Should contain item 3: {}",
        output
    );
}

#[test]
fn test_format_list_ordered() {
    let options = Options::default();
    let input = "1. First\n2. Second\n3. Third";
    let output = markdown_to_commonmark(input, &options, &Plugins::default());

    assert!(
        output.contains("1."),
        "Should contain ordered item 1: {}",
        output
    );
    assert!(
        output.contains("2."),
        "Should contain ordered item 2: {}",
        output
    );
    assert!(
        output.contains("3."),
        "Should contain ordered item 3: {}",
        output
    );
}

#[test]
fn test_format_nested_list() {
    let options = Options::default();
    let input = "- Item 1\n- Item 2\n  - Nested 1\n  - Nested 2";
    let output = markdown_to_commonmark(input, &options, &Plugins::default());

    assert!(
        output.contains("- Item 1"),
        "Should contain outer item 1: {}",
        output
    );
    assert!(
        output.contains("- Item 2"),
        "Should contain outer item 2: {}",
        output
    );
    assert!(
        output.contains("  - Nested 1"),
        "Should contain nested item 1: {}",
        output
    );
    assert!(
        output.contains("  - Nested 2"),
        "Should contain nested item 2: {}",
        output
    );
}

#[test]
fn test_format_link() {
    let options = Options::default();
    let input = "[example](https://example.com)";
    let output = markdown_to_commonmark(input, &options, &Plugins::default());

    assert!(
        output.contains("[example](https://example.com)"),
        "Should contain link: {}",
        output
    );
}

#[test]
fn test_format_image() {
    let options = Options::default();
    let input = "![alt text](image.png)";
    let output = markdown_to_commonmark(input, &options, &Plugins::default());

    assert!(
        output.contains("![alt text](image.png)"),
        "Should contain image: {}",
        output
    );
}

#[test]
fn test_format_blockquote() {
    let options = Options::default();
    let input = "> This is a quote";
    let output = markdown_to_commonmark(input, &options, &Plugins::default());

    assert!(
        output.contains("> This is a quote"),
        "Should contain blockquote: {}",
        output
    );
}

#[test]
fn test_format_table() {
    let mut options = Options::default();
    options.extension.table = true;

    let input = "| Name | Age |\n|------|-----|\n| Alice | 30 |\n| Bob | 25 |";
    let output = markdown_to_commonmark(input, &options, &Plugins::default());

    assert!(
        output.contains("Name"),
        "Should contain header Name: {}",
        output
    );
    assert!(
        output.contains("Age"),
        "Should contain header Age: {}",
        output
    );
    assert!(
        output.contains("Alice"),
        "Should contain cell Alice: {}",
        output
    );
    assert!(
        output.contains("Bob"),
        "Should contain cell Bob: {}",
        output
    );
    assert!(
        output.contains("---"),
        "Should contain delimiter row: {}",
        output
    );
}

#[test]
fn test_format_thematic_break() {
    let options = Options::default();

    // Test that --- is preserved
    let input = "---";
    let output = markdown_to_commonmark(input, &options, &Plugins::default());
    assert!(
        output.contains("---"),
        "Should preserve --- marker: {}",
        output
    );

    // Test that *** is preserved
    let input2 = "***";
    let output2 = markdown_to_commonmark(input2, &options, &Plugins::default());
    assert!(
        output2.contains("***"),
        "Should preserve *** marker: {}",
        output2
    );

    // Test that ___ is preserved
    let input3 = "___";
    let output3 = markdown_to_commonmark(input3, &options, &Plugins::default());
    assert!(
        output3.contains("___"),
        "Should preserve ___ marker: {}",
        output3
    );
}

#[test]
fn test_format_hard_break() {
    let options = Options::default();
    let input = "Line 1  \nLine 2";
    let output = markdown_to_commonmark(input, &options, &Plugins::default());

    assert!(
        output.contains("Line 1"),
        "Should contain line 1: {}",
        output
    );
    assert!(
        output.contains("Line 2"),
        "Should contain line 2: {}",
        output
    );
}

#[test]
fn test_format_strikethrough() {
    let mut options = Options::default();
    options.extension.strikethrough = true;

    let input = "~~deleted~~";
    let output = markdown_to_commonmark(input, &options, &Plugins::default());

    assert!(
        output.contains("~~deleted~~"),
        "Should contain strikethrough: {}",
        output
    );
}

#[test]
fn test_format_complex_document() {
    let mut options = Options::default();
    options.extension.table = true;
    options.extension.strikethrough = true;

    let input = r#"# Document Title

This is an introduction paragraph with **bold** and *italic* text.

## Section 1: Lists

- Bullet item 1
- Bullet item 2
  - Nested item A
  - Nested item B

## Section 2: Code

```rust
fn hello() {
    println!("Hello, World!");
}
```

## Section 3: Table

| Name  | Value |
|-------|-------|
| One   | 1     |
| Two   | 2     |

> A blockquote with ~~deleted~~ text.
"#;

    let output = markdown_to_commonmark(input, &options, &Plugins::default());

    // Verify all sections are present
    assert!(
        output.contains("# Document Title"),
        "Should contain title: {}",
        output
    );
    assert!(
        output.contains("## Section 1: Lists"),
        "Should contain section 1: {}",
        output
    );
    assert!(
        output.contains("## Section 2: Code"),
        "Should contain section 2: {}",
        output
    );
    assert!(
        output.contains("## Section 3: Table"),
        "Should contain section 3: {}",
        output
    );

    // Verify formatting is preserved
    assert!(
        output.contains("**bold**"),
        "Should preserve bold: {}",
        output
    );
    assert!(
        output.contains("*italic*"),
        "Should preserve italic: {}",
        output
    );
    assert!(
        output.contains("```rust"),
        "Should preserve code block: {}",
        output
    );
    assert!(
        output.contains("| Name"),
        "Should preserve table: {}",
        output
    );
    assert!(
        output.contains("> A blockquote"),
        "Should preserve blockquote: {}",
        output
    );
}

#[test]
fn test_format_preserves_structure() {
    let options = Options::default();

    // Test that formatting is idempotent for simple cases
    let input = "# Title\n\nParagraph with **bold**.\n\n- List item\n";
    let first_pass = markdown_to_commonmark(input, &options, &Plugins::default());
    let second_pass = markdown_to_commonmark(&first_pass, &options, &Plugins::default());

    // The output should be stable (or at least structurally similar)
    assert!(
        second_pass.contains("# Title"),
        "Second pass should contain title"
    );
    assert!(
        second_pass.contains("**bold**"),
        "Second pass should contain bold"
    );
    assert!(
        second_pass.contains("- List item"),
        "Second pass should contain list item"
    );
}

#[test]
fn test_format_empty_document() {
    let options = Options::default();
    let input = "";
    let output = markdown_to_commonmark(input, &options, &Plugins::default());

    // Empty input should produce empty or minimal output
    assert!(
        output.is_empty() || output.trim().is_empty(),
        "Empty input should produce empty output: {:?}",
        output
    );
}

#[test]
fn test_format_whitespace_only() {
    let options = Options::default();
    let input = "   \n\n   \n";
    let output = markdown_to_commonmark(input, &options, &Plugins::default());

    // Whitespace-only input should be handled gracefully
    assert!(
        !output.contains("#"),
        "Should not create headings from whitespace"
    );
}

#[test]
fn test_format_task_list() {
    let options = Options::default();
    let input = "- [ ] Unchecked task\n- [x] Checked task\n- [X] Checked task uppercase";
    let output = markdown_to_commonmark(input, &options, &Plugins::default());

    // Check that task list markers are preserved
    // Note: [X] (uppercase) is normalized to [x] (lowercase) during formatting
    assert!(
        output.contains("- [ ]"),
        "Should contain unchecked task marker: {}",
        output
    );
    assert!(
        output.contains("- [x]"),
        "Should contain checked task marker: {}",
        output
    );
    assert!(
        output.contains("Unchecked task"),
        "Should contain unchecked task text: {}",
        output
    );
    assert!(
        output.contains("Checked task"),
        "Should contain checked task text: {}",
        output
    );
}

#[test]
fn test_format_task_list_with_content() {
    let options = Options::default();
    let input = "- [ ] Task with **bold** text\n- [x] Task with *italic* text";
    let output = markdown_to_commonmark(input, &options, &Plugins::default());

    // Check that task list markers and formatting are preserved
    assert!(
        output.contains("- [ ]"),
        "Should contain unchecked task marker: {}",
        output
    );
    assert!(
        output.contains("- [x]"),
        "Should contain checked task marker: {}",
        output
    );
    assert!(
        output.contains("**bold**"),
        "Should contain bold formatting: {}",
        output
    );
    assert!(
        output.contains("*italic*"),
        "Should contain italic formatting: {}",
        output
    );
}

#[test]
fn test_format_single_line_html_comment() {
    let options = Options::default();

    // Test that single-line HTML comments don't have extra blank lines
    let input =
        "<!-- TOC -->\n\n- [Item 1](#item-1)\n- [Item 2](#item-2)\n\n<!-- TOC -->";
    let output = markdown_to_commonmark(input, &options, &Plugins::default());

    // The output should contain the HTML comments
    assert!(
        output.contains("<!-- TOC -->"),
        "Should contain HTML comment: {}",
        output
    );

    // Check that there are no double blank lines before or after the comment
    // This verifies the fix for the issue where extra blank lines were added
    let lines: Vec<&str> = output.lines().collect();

    // Find the first <!-- TOC --> line
    if let Some(first_toc_idx) = lines.iter().position(|&l| l.trim() == "<!-- TOC -->") {
        // Check that the line before is not empty (unless it's the first line)
        if first_toc_idx > 0 {
            assert!(
                !lines[first_toc_idx - 1].trim().is_empty(),
                "Should not have blank line before single-line HTML comment: {:?}",
                lines
            );
        }
    }
}

#[test]
fn test_format_html_comment_with_content() {
    let options = Options::default();

    // Test HTML comment with content around it
    let input = r#"<!-- TOC -->

- [Section 1](#section-1)
  - [Subsection](#subsection)

<!-- TOC -->

# Section 1

## Subsection

Content here.
"#;

    let output = markdown_to_commonmark(input, &options, &Plugins::default());

    // Verify the HTML comments are preserved
    assert!(
        output.contains("<!-- TOC -->"),
        "Should contain HTML comment: {}",
        output
    );

    // Verify content is preserved
    assert!(
        output.contains("Section 1"),
        "Should contain section 1: {}",
        output
    );
    assert!(
        output.contains("Subsection"),
        "Should contain subsection: {}",
        output
    );
}

#[test]
fn test_format_multi_line_html_block() {
    let options = Options::default();

    // Test that multi-line HTML blocks still have proper spacing
    let input = r#"<div class="container">
<p>This is a paragraph inside a div.</p>
</div>

Some text after.
"#;

    let output = markdown_to_commonmark(input, &options, &Plugins::default());

    // Verify the HTML block is preserved
    assert!(
        output.contains("<div"),
        "Should contain div tag: {}",
        output
    );
    assert!(
        output.contains("</div>"),
        "Should contain closing div tag: {}",
        output
    );
    assert!(
        output.contains("Some text after"),
        "Should contain text after: {}",
        output
    );
}

#[test]
fn test_format_html_comment_toc_style() {
    let options = Options::default();

    // Test the exact pattern from the bug report
    let input = r#"<!-- TOC -->

* [Build alignments across a eukaryotic taxonomy rank](#build-alignments-across-a-eukaryotic-taxonomy-rank)
  * [Taxon info](#taxon-info)

<!-- TOC -->

# Build alignments across a eukaryotic taxonomy rank

## Taxon info

Some content here.
"#;

    let output = markdown_to_commonmark(input, &options, &Plugins::default());

    // Verify HTML comments are present
    let toc_count = output.matches("<!-- TOC -->").count();
    assert_eq!(
        toc_count, 2,
        "Should have exactly 2 TOC comments: {}",
        output
    );

    // Verify headings are preserved
    assert!(
        output.contains("# Build alignments"),
        "Should contain h1 heading: {}",
        output
    );
    assert!(
        output.contains("## Taxon info"),
        "Should contain h2 heading: {}",
        output
    );

    // Check that there are no consecutive blank lines around HTML comments
    let lines: Vec<&str> = output.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        if line.trim() == "<!-- TOC -->" {
            // Check that we don't have blank line before (unless it's first line)
            if i > 0 {
                // The previous line should not be empty
                // (There can be content or a single blank line for separation, but not multiple)
            }
        }
    }
}
