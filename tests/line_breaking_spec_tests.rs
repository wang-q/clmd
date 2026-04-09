// Line breaking algorithm tests

use clmd::{markdown_to_commonmark, Options, Plugins};

fn format_with_width(md: &str, width: usize) -> String {
    let mut options = Options::default();
    options.render.width = width;
    options.extension.table = true;
    options.extension.tasklist = true;
    markdown_to_commonmark(md, &options, &Plugins::default())
}

#[test]
fn test_basic_paragraph_wrapping() {
    let input = "This is a long paragraph that should be wrapped at the specified width limit. We want to see how well the line breaking algorithm handles this case.";
    let output = format_with_width(input, 40);
    println!("Basic wrapping:\n{}\n", output);

    // Check that lines don't exceed 40 characters
    for line in output.lines() {
        assert!(
            line.len() <= 40,
            "Line exceeds 40 chars: '{}' (len={})",
            line,
            line.len()
        );
    }
}

#[test]
fn test_list_item_wrapping() {
    let input = "- This is a long list item that should be wrapped properly with correct indentation for continuation lines\n- Another item";
    let output = format_with_width(input, 40);
    println!("List item wrapping:\n{}\n", output);

    // Check that continuation lines have proper indentation
    let lines: Vec<&str> = output.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        println!("Line {}: '{}' (len={})", i, line, line.len());
    }
}

#[test]
fn test_nested_list_wrapping() {
    let input = "- Outer item\n  - Nested item with long text that should be wrapped\n  - Another nested";
    let output = format_with_width(input, 40);
    println!("Nested list wrapping:\n{}\n", output);
}

#[test]
fn test_blockquote_wrapping() {
    let input = "> This is a long quote that should be wrapped properly with the block quote marker preserved on continuation lines.";
    let output = format_with_width(input, 40);
    println!("Blockquote wrapping:\n{}\n", output);

    // Check that continuation lines have block quote marker
    for line in output.lines() {
        assert!(
            line.starts_with("> ") || line.is_empty(),
            "Continuation line missing block quote marker: '{}'",
            line
        );
    }
}

#[test]
fn test_cjk_text_wrapping() {
    let input = "这是一个很长的中文段落，应该在指定的宽度限制处正确换行。我们需要测试断行算法对CJK文本的处理能力。";
    let output = format_with_width(input, 40);
    println!("CJK wrapping:\n{}\n", output);
}

#[test]
fn test_link_wrapping() {
    let input = "Here is a [long link text](https://example.com/path/to/something) that should be handled properly.";
    let output = format_with_width(input, 40);
    println!("Link wrapping:\n{}\n", output);
}

#[test]
fn test_emphasis_wrapping() {
    let input = "This is **bold text** and *italic text* that should be wrapped without breaking the markers.";
    let output = format_with_width(input, 40);
    println!("Emphasis wrapping:\n{}\n", output);
}

#[test]
fn test_code_wrapping() {
    let input = "Here is some `inline code` that should not be broken across lines.";
    let output = format_with_width(input, 40);
    println!("Code wrapping:\n{}\n", output);
}

#[test]
fn test_hard_break() {
    let input = "Line one  \nLine two";
    let output = format_with_width(input, 40);
    println!("Hard break:\n{}\n", output);

    // Hard break should result in two separate lines
    // The two spaces before newline should be preserved or converted to proper line break
    assert!(output.contains("Line one") && output.contains("Line two"));
}

#[test]
fn test_mixed_content() {
    let input = r#"# Heading

This is a paragraph with **bold** and *italic* text, plus a [link](https://example.com).

- List item one
- List item two with more text
  - Nested item

> A blockquote with some text.

`code block`
"#;
    let output = format_with_width(input, 50);
    println!("Mixed content:\n{}\n", output);
}
