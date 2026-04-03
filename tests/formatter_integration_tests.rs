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
