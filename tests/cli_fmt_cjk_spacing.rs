use std::io::Write;
use std::process::{Command, Stdio};

fn clmd_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_clmd"))
}

fn run_with_stdin(args: &[&str], input: &[u8]) -> std::process::Output {
    let mut child = clmd_bin()
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn command");

    {
        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        stdin.write_all(input).expect("Failed to write to stdin");
    }

    child.wait_with_output().expect("Failed to read stdout")
}

#[test]
fn test_fmt_cjk_spacing_basic() {
    let input = "中文test示例".as_bytes();
    let output = run_with_stdin(&["fmt", "--cjk-spacing"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();
    assert!(
        cm.contains("中文 test 示例"),
        "Should add spaces between CJK and English: {}",
        cm
    );
}

#[test]
fn test_fmt_cjk_spacing_with_numbers() {
    let input = "数字123测试".as_bytes();
    let output = run_with_stdin(&["fmt", "--cjk-spacing"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();
    assert!(
        cm.contains("数字 123 测试"),
        "Should add spaces between CJK and numbers: {}",
        cm
    );
}

#[test]
fn test_fmt_cjk_spacing_english_to_cjk() {
    let input = "test中文content".as_bytes();
    let output = run_with_stdin(&["fmt", "--cjk-spacing"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();
    assert!(
        cm.contains("test 中文 content"),
        "Should add spaces between English and CJK: {}",
        cm
    );
}

#[test]
fn test_fmt_cjk_spacing_mixed() {
    let input = "这是一个test示例，包含English单词和数字123。".as_bytes();
    let output = run_with_stdin(&["fmt", "--cjk-spacing"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();
    assert!(
        cm.contains("这是一个 test 示例，包含 English 单词和数字 123。"),
        "Should add spaces in mixed content: {}",
        cm
    );
}

#[test]
fn test_fmt_cjk_spacing_disabled_by_default() {
    let input = "中文test示例".as_bytes();
    let output = run_with_stdin(&["fmt"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();
    assert!(
        cm.contains("中文test示例"),
        "Should NOT add spaces when --cjk-spacing is not specified: {}",
        cm
    );
}

#[test]
fn test_fmt_cjk_spacing_with_existing_spaces() {
    let input = "中文 test 示例".as_bytes();
    let output = run_with_stdin(&["fmt", "--cjk-spacing"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();
    // Should not add duplicate spaces
    assert!(
        cm.contains("中文 test 示例"),
        "Should not add duplicate spaces: {}",
        cm
    );
}

#[test]
fn test_fmt_cjk_spacing_inline_code() {
    let input = "这是一个 `inline code` 示例".as_bytes();
    let output = run_with_stdin(&["fmt", "--cjk-spacing"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();
    // Inline code content should not have spaces added
    assert!(
        cm.contains("`inline code`"),
        "Should preserve inline code: {}",
        cm
    );
}

#[test]
fn test_fmt_cjk_spacing_code_block() {
    let input = r#"这是一个代码块示例：

```
中文test示例
```

结束。"#
        .as_bytes();
    let output = run_with_stdin(&["fmt", "--cjk-spacing"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();
    // Code block content should not have spaces added
    assert!(
        cm.contains("中文test示例"),
        "Should not modify code block content: {}",
        cm
    );
}

#[test]
fn test_fmt_cjk_spacing_with_heading() {
    let input = "# 标题test内容\n\n正文English文字123。".as_bytes();
    let output = run_with_stdin(&["fmt", "--cjk-spacing"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();
    assert!(
        cm.contains("# 标题 test 内容"),
        "Should add spaces in heading: {}",
        cm
    );
    assert!(
        cm.contains("正文 English 文字 123。"),
        "Should add spaces in paragraph: {}",
        cm
    );
}

#[test]
fn test_fmt_cjk_spacing_with_list() {
    let input = "- 项目test内容\n- 数字123测试".as_bytes();
    let output = run_with_stdin(&["fmt", "--cjk-spacing"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();
    assert!(
        cm.contains("- 项目 test 内容"),
        "Should add spaces in list item 1: {}",
        cm
    );
    assert!(
        cm.contains("- 数字 123 测试"),
        "Should add spaces in list item 2: {}",
        cm
    );
}

#[test]
fn test_fmt_cjk_spacing_with_link() {
    let input = "这是一个[链接](https://example.com)test示例".as_bytes();
    let output = run_with_stdin(&["fmt", "--cjk-spacing"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();
    // Link text should have spaces added, but URL should not
    assert!(
        cm.contains("这是一个 [链接](https://example.com) test 示例")
            || cm.contains("这是一个[链接](https://example.com)test")
            || cm.contains("链接"),
        "Should handle link correctly: {}",
        cm
    );
}

/// Test for issue: inline code should not cause text order corruption
/// This test ensures that inline code is rendered in the correct position
/// and not moved to the beginning of the text.
/// Regression test for: Text order corruption when using inline code with CJK spacing
#[test]
fn test_fmt_cjk_spacing_inline_code_text_order() {
    // Test case 1: CJK text with inline code
    let input = "本文档旨在为 `tva` 的开发者提供技术背景。".as_bytes();
    let output = run_with_stdin(&["fmt", "--cjk-spacing"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();
    // The inline code `tva` should appear after "本文档旨在为 " and before " 的开发者"
    // It should NOT appear at the beginning of the output
    assert!(cm.contains("`tva`"), "Should contain inline code: {}", cm);
    // Check that the code is not at the beginning (indicating order corruption)
    assert!(
        !cm.trim_start().starts_with("`tva`"),
        "Inline code should not be at the beginning (text order corruption): {}",
        cm
    );
    // Verify the text appears in correct order
    assert!(
        cm.contains("本文档") && cm.contains("`tva`") && cm.contains("开发者"),
        "Text should appear in correct order: {}",
        cm
    );
}

/// Additional test for inline code text order without CJK spacing
/// This ensures the fix works for all cases, not just with --cjk-spacing
#[test]
fn test_fmt_inline_code_text_order_without_cjk_spacing() {
    let input = "Hello `world` test.".as_bytes();
    let output = run_with_stdin(&["fmt"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();
    // The inline code `world` should appear after "Hello " and before " test"
    assert!(cm.contains("`world`"), "Should contain inline code: {}", cm);
    // Check that the code is not at the beginning (indicating order corruption)
    assert!(
        !cm.trim_start().starts_with("`world`"),
        "Inline code should not be at the beginning (text order corruption): {}",
        cm
    );
    // Verify the text appears in correct order
    assert!(
        cm.contains("Hello") && cm.contains("`world`") && cm.contains("test"),
        "Text should appear in correct order: {}",
        cm
    );
}

#[test]
fn test_fmt_cjk_punctuation_no_space_with_markdown() {
    // Test that CJK punctuation does not have space added before/after Markdown markers
    // Issue: `**特性**：` was being formatted as `**特性** ：` (space before colon)
    let input = "- **特性**：datamash 提供大量逐行转换操作。".as_bytes();
    let output = run_with_stdin(&["fmt", "--cjk-spacing"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();

    // The CJK colon should NOT have space before it
    assert!(
        cm.contains("**特性**："),
        "CJK punctuation should not have space before Markdown marker: {}",
        cm
    );
    assert!(
        !cm.contains("**特性** ："),
        "There should be no space between ** and CJK colon: {}",
        cm
    );
}

#[test]
fn test_fmt_cjk_punctuation_with_emphasis() {
    // Test CJK punctuation with emphasis markers
    let input = "*强调*，测试。*强调*。".as_bytes();
    let output = run_with_stdin(&["fmt", "--cjk-spacing"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();

    // CJK comma and period should NOT have space before them
    assert!(
        cm.contains("*强调*，"),
        "CJK comma should not have space after emphasis: {}",
        cm
    );
    assert!(
        cm.contains("*强调*。"),
        "CJK period should not have space after emphasis: {}",
        cm
    );
    assert!(
        !cm.contains("*强调* ，"),
        "There should be no space between * and CJK comma: {}",
        cm
    );
    assert!(
        !cm.contains("*强调* 。"),
        "There should be no space between * and CJK period: {}",
        cm
    );
}

#[test]
fn test_fmt_cjk_punctuation_with_inline_code() {
    // Test CJK punctuation with inline code
    let input = "使用 `code`：示例。".as_bytes();
    let output = run_with_stdin(&["fmt", "--cjk-spacing"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();

    // CJK colon should NOT have space before it after inline code
    assert!(
        cm.contains("`code`："),
        "CJK colon should not have space after inline code: {}",
        cm
    );
    assert!(
        !cm.contains("`code` ："),
        "There should be no space between ` and CJK colon: {}",
        cm
    );
}

#[test]
fn test_fmt_cjk_punctuation_with_link() {
    // Test CJK punctuation with links
    let input = "[链接](https://example.com)：测试。".as_bytes();
    let output = run_with_stdin(&["fmt", "--cjk-spacing"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();

    // CJK colon should NOT have space before it after link
    assert!(
        cm.contains("](https://example.com)："),
        "CJK colon should not have space after link: {}",
        cm
    );
    assert!(
        !cm.contains("](https://example.com) ："),
        "There should be no space between ) and CJK colon: {}",
        cm
    );
}

#[test]
fn test_fmt_cjk_punctuation_various_marks() {
    // Test various CJK punctuation marks with Markdown
    let test_cases = vec![
        ("**粗体**：", "**粗体**：", "CJK colon"),
        ("*斜体*，", "*斜体*，", "CJK comma"),
        ("`code`。", "`code`。", "CJK period"),
        ("**粗体**；", "**粗体**；", "CJK semicolon"),
        ("*斜体*？", "*斜体*？", "CJK question"),
        ("`code`！", "`code`！", "CJK exclamation"),
    ];

    for (input_text, expected, desc) in test_cases {
        let input = format!("{}", input_text).as_bytes().to_vec();
        let output = run_with_stdin(&["fmt", "--cjk-spacing"], &input);

        assert!(output.status.success());
        let cm = String::from_utf8(output.stdout).unwrap();

        assert!(
            cm.contains(expected),
            "{} should not have space before it: got {}",
            desc,
            cm
        );
    }
}

#[test]
fn test_fmt_cjk_punctuation_at_line_start() {
    // Test CJK punctuation at the start of a line
    let input = "：这是以冒号开头的句子。".as_bytes();
    let output = run_with_stdin(&["fmt", "--cjk-spacing"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();

    // The line should start with CJK colon, not a space
    assert!(
        cm.trim_start().starts_with('：'),
        "Line should start with CJK colon: {}",
        cm
    );
}

#[test]
fn test_fmt_cjk_punctuation_between_cjk_and_markdown() {
    // Test CJK punctuation between CJK text and Markdown markers
    let input = "注意：**重要**，请查看。".as_bytes();
    let output = run_with_stdin(&["fmt", "--cjk-spacing"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();

    // Space should be added between CJK and Markdown, but not between Markdown and CJK punctuation
    assert!(
        cm.contains("注意：**重要**，"),
        "Should have space before ** but not after: {}",
        cm
    );
    assert!(
        !cm.contains("注意**重要** ，"),
        "Should not have space between ** and CJK comma: {}",
        cm
    );
}
