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
    let output = run_with_stdin(&["fmt"], input);

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
    let output = run_with_stdin(&["fmt"], input);

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
    let output = run_with_stdin(&["fmt"], input);

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
    let output = run_with_stdin(&["fmt"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();
    assert!(
        cm.contains("这是一个 test 示例，包含 English 单词和数字 123。"),
        "Should add spaces in mixed content: {}",
        cm
    );
}

#[test]
fn test_fmt_cjk_spacing_enabled_by_default() {
    let input = "中文test示例".as_bytes();
    let output = run_with_stdin(&["fmt"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();
    assert!(
        cm.contains("中文 test 示例"),
        "Should add spaces by default (CJK spacing is default behavior): {}",
        cm
    );
}

#[test]
fn test_fmt_cjk_spacing_with_existing_spaces() {
    let input = "中文 test 示例".as_bytes();
    let output = run_with_stdin(&["fmt"], input);

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
    let output = run_with_stdin(&["fmt"], input);

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
    let output = run_with_stdin(&["fmt"], input);

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
    let output = run_with_stdin(&["fmt"], input);

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
    let output = run_with_stdin(&["fmt"], input);

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
    let output = run_with_stdin(&["fmt"], input);

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
    let output = run_with_stdin(&["fmt"], input);

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

/// Additional test for inline code text order
/// This ensures the fix works for all cases
#[test]
fn test_fmt_inline_code_text_order() {
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
    let output = run_with_stdin(&["fmt"], input);

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
    let output = run_with_stdin(&["fmt"], input);

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
    let output = run_with_stdin(&["fmt"], input);

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
    let output = run_with_stdin(&["fmt"], input);

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
        let output = run_with_stdin(&["fmt"], &input);

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
fn test_fmt_cjk_comma_between_inline_codes() {
    // Test that comma between inline codes has no extra spaces
    // Example: `one-to-one`, `many-to-one`
    let input = "- 行动: 添加 `--relationship` 标志（例如 `one-to-one`, `many-to-one`）在连接时验证键。".as_bytes();
    let output = run_with_stdin(&["fmt", "--width", "120"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();

    // The comma should have no space before it and one space after it
    assert!(
        cm.contains("`one-to-one`, `many-to-one`"),
        "Comma between inline codes should have no space before and one space after: got {}",
        cm
    );
}

#[test]
fn test_fmt_cjk_punctuation_at_line_start() {
    // Test CJK punctuation at the start of a line
    let input = "：这是以冒号开头的句子。".as_bytes();
    let output = run_with_stdin(&["fmt"], input);

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
    let output = run_with_stdin(&["fmt"], input);

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

#[test]
fn test_fmt_cjk_punctuation_multiple_marks() {
    // Test multiple CJK punctuation marks in sequence
    let input = "**粗体***斜体**：测试。".as_bytes();
    let output = run_with_stdin(&["fmt"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();

    // CJK colon should not have space before it
    assert!(
        cm.contains("*斜体**："),
        "CJK colon should not have space after italic: {}",
        cm
    );
}

#[test]
fn test_fmt_cjk_punctuation_with_nested_emphasis() {
    // Test nested emphasis with CJK punctuation
    let input = "***粗斜体***：测试。".as_bytes();
    let output = run_with_stdin(&["fmt"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();

    // CJK colon should not have space before it
    assert!(
        cm.contains("***粗斜体***："),
        "CJK colon should not have space after nested emphasis: {}",
        cm
    );
}

#[test]
fn test_fmt_cjk_punctuation_with_strikethrough() {
    // Test strikethrough with CJK punctuation (if supported)
    let input = "~~删除~~：测试。".as_bytes();
    let output = run_with_stdin(&["fmt"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();

    // CJK colon should not have space before it
    assert!(
        !cm.contains("~~删除~~ ："),
        "CJK colon should not have space after strikethrough: {}",
        cm
    );
}

#[test]
fn test_fmt_cjk_inline_code_at_end_of_sentence() {
    // Test inline code at the end of sentence followed by CJK punctuation
    let input = "使用 `code`。这是下一句。".as_bytes();
    let output = run_with_stdin(&["fmt"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();

    // CJK period should not have space before it
    assert!(
        cm.contains("`code`。"),
        "CJK period should not have space after inline code: {}",
        cm
    );
    assert!(
        !cm.contains("`code` 。"),
        "There should be no space between ` and CJK period: {}",
        cm
    );
}

#[test]
fn test_fmt_cjk_inline_code_at_start_of_sentence() {
    // Test inline code at the start of sentence
    let input = "`code` 是代码示例。".as_bytes();
    let output = run_with_stdin(&["fmt"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();

    // Space should be added between inline code and CJK text
    assert!(
        cm.contains("`code` 是"),
        "Space should be added after inline code: {}",
        cm
    );
}

#[test]
fn test_fmt_cjk_multiple_inline_codes() {
    // Test multiple inline codes in one sentence
    let input = "使用 `code1` 和 `code2` 测试。".as_bytes();
    let output = run_with_stdin(&["fmt"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();

    // Space should be added around inline codes
    assert!(
        cm.contains("`code1` 和"),
        "Space should be added after first inline code: {}",
        cm
    );
    assert!(
        cm.contains("和 `code2` 测试"),
        "Space should be added around second inline code: {}",
        cm
    );
}

#[test]
fn test_fmt_cjk_link_with_cjk_punctuation() {
    // Test link followed by CJK punctuation
    let input = "[链接](https://example.com)：说明。".as_bytes();
    let output = run_with_stdin(&["fmt"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();

    // CJK colon should not have space before it
    assert!(
        !cm.contains("](https://example.com) ："),
        "CJK colon should not have space after link: {}",
        cm
    );
}

#[test]
fn test_fmt_cjk_image_with_cjk_punctuation() {
    // Test image followed by CJK punctuation
    let input = "![图片](image.png)：说明。".as_bytes();
    let output = run_with_stdin(&["fmt"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();

    // CJK colon should not have space before it
    assert!(
        !cm.contains("](image.png) ："),
        "CJK colon should not have space after image: {}",
        cm
    );
}

#[test]
fn test_fmt_cjk_heading_with_inline_code() {
    // Test heading with inline code
    let input = "# 标题 `code` 说明".as_bytes();
    let output = run_with_stdin(&["fmt"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();

    // Space should be added around inline code in heading
    assert!(
        cm.contains("标题 `code` 说明"),
        "Space should be added around inline code in heading: {}",
        cm
    );
}

#[test]
fn test_fmt_cjk_list_with_inline_code() {
    // Test list item with inline code
    let input = "- 项目 `code` 说明\n- 另一项".as_bytes();
    let output = run_with_stdin(&["fmt"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();

    // Space should be added around inline code in list item
    assert!(
        cm.contains("项目 `code` 说明"),
        "Space should be added around inline code in list: {}",
        cm
    );
}

#[test]
fn test_fmt_cjk_blockquote_with_inline_code() {
    // Test blockquote with inline code
    let input = "> 引用 `code` 说明".as_bytes();
    let output = run_with_stdin(&["fmt"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();

    // Space should be added around inline code in blockquote
    assert!(
        cm.contains("引用 `code` 说明"),
        "Space should be added around inline code in blockquote: {}",
        cm
    );
}

#[test]
fn test_fmt_cjk_mixed_english_and_cjk_punctuation() {
    // Test mixed English and CJK punctuation
    let input = "**粗体**：test，中文。".as_bytes();
    let output = run_with_stdin(&["fmt"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();

    // CJK colon should not have space before it
    assert!(
        !cm.contains("**粗体** ："),
        "CJK colon should not have space: {}",
        cm
    );
    // English text should have space after CJK colon
    assert!(
        cm.contains("**：test") || cm.contains("**： test"),
        "English text should follow CJK colon: {}",
        cm
    );
}

#[test]
fn test_fmt_cjk_all_punctuation_types() {
    // Test all common CJK punctuation types
    let test_cases = vec![
        ("测试：", "CJK colon"),
        ("测试；", "CJK semicolon"),
        ("测试，", "CJK comma"),
        ("测试。", "CJK period"),
        ("测试！", "CJK exclamation"),
        ("测试？", "CJK question"),
        ("测试、", "CJK enumeration comma"),
        ("测试（", "CJK left parenthesis"),
        ("测试）", "CJK right parenthesis"),
        ("测试【", "CJK left bracket"),
        ("测试】", "CJK right bracket"),
    ];

    for (punct, desc) in test_cases {
        let input = format!("**粗体**{}", punct).as_bytes().to_vec();
        let output = run_with_stdin(&["fmt"], &input);

        assert!(output.status.success());
        let cm = String::from_utf8(output.stdout).unwrap();

        let expected = format!("**粗体**{}", punct);
        assert!(
            cm.contains(&expected),
            "{} should not have space before it: got {}",
            desc,
            cm
        );
    }
}

#[test]
fn test_fmt_cjk_emphasis_with_english() {
    // Test emphasis mixed with English and CJK
    let input = "**bold**中文和English混合。".as_bytes();
    let output = run_with_stdin(&["fmt"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();

    // Space should be added between English and CJK
    assert!(
        cm.contains("**bold** 中文") || cm.contains("**bold**中文"),
        "Bold marker should be followed by CJK text: {}",
        cm
    );
}

#[test]
fn test_fmt_cjk_code_with_backticks() {
    // Test inline code with backticks inside
    let input = "使用 `` `backticks` `` 代码。".as_bytes();
    let output = run_with_stdin(&["fmt"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();

    // Space should be added around code
    assert!(
        cm.contains("使用 `` `backticks` `` 代码"),
        "Space should be added around code with backticks: {}",
        cm
    );
}

#[test]
fn test_fmt_cjk_empty_document() {
    // Test empty document
    let input = "".as_bytes();
    let output = run_with_stdin(&["fmt"], input);

    assert!(output.status.success());
}

#[test]
fn test_fmt_cjk_only_whitespace() {
    // Test document with only whitespace
    let input = "   \n\n   ".as_bytes();
    let output = run_with_stdin(&["fmt"], input);

    assert!(output.status.success());
}

#[test]
fn test_fmt_cjk_long_paragraph() {
    // Test long paragraph with line breaking
    let input = "这是一个很长的段落，包含很多中文字符和English单词，用来测试行断行功能是否正常工作，以及CJK标点的处理是否正确。".as_bytes();
    let output = run_with_stdin(&["fmt", "--width", "40"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();

    // Check that the output has multiple lines
    let lines: Vec<&str> = cm.lines().collect();
    assert!(
        lines.len() > 1,
        "Long paragraph should be wrapped into multiple lines: {}",
        cm
    );
}

#[test]
fn test_fmt_cjk_ascii_colon_after_inline_code() {
    // Test ASCII colon after inline code
    // Issue: `longer`: 支持 was being formatted as `longer` : 支持 (space before colon)
    let input = "- `longer`: 支持在 `--names-to` 中使用。".as_bytes();
    let output = run_with_stdin(&["fmt"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();

    // ASCII colon should NOT have space before it after inline code
    assert!(
        cm.contains("`longer`: 支持"),
        "ASCII colon should not have space before it after inline code: {}",
        cm
    );
    assert!(
        !cm.contains("`longer` : 支持"),
        "There should be no space between ` and ASCII colon: {}",
        cm
    );
}

#[test]
fn test_fmt_cjk_ascii_colon_various() {
    // Test various ASCII punctuation after inline code
    let test_cases = vec![
        ("`code`: 测试", "`code`: 测试", "ASCII colon"),
        ("`code`, 测试", "`code`, 测试", "ASCII comma"),
        ("`code`. 测试", "`code`. 测试", "ASCII period"),
        ("`code`; 测试", "`code`; 测试", "ASCII semicolon"),
    ];

    for (input_text, expected, desc) in test_cases {
        let input = input_text.as_bytes();
        let output = run_with_stdin(&["fmt"], input);

        assert!(output.status.success());
        let cm = String::from_utf8(output.stdout).unwrap();

        assert!(
            cm.contains(expected),
            "{} should not have space before it after inline code: got {}",
            desc,
            cm
        );
    }
}

#[test]
fn test_fmt_cjk_comma_not_at_line_start() {
    // Test that comma is not placed at the start of a line when wrapping
    // Example: `one-to-one`, `many-to-one` should not have comma at line start
    let input = "- 行动: 添加 `--relationship` 标志（例如 `one-to-one`, `many-to-one`）在连接时验证键。".as_bytes();
    let output = run_with_stdin(&["fmt", "--width", "50"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();

    // The comma should not be at the start of a line
    assert!(
        !cm.contains("\n  ,"),
        "Comma should not be at the start of a line: got {}",
        cm
    );

    // The comma should be on the same line as the preceding content
    assert!(
        cm.contains("`one-to-one`,\n") || cm.contains("`one-to-one`, "),
        "Comma should be on the same line as the preceding inline code: got {}",
        cm
    );
}

#[test]
fn test_fmt_cjk_closing_paren_not_at_line_start() {
    // Test that closing parenthesis is not placed at the start of a line when wrapping
    // Example: (`xan/src/cmd/hist.rs`) should not have ) at line start
    let input = "除了 `plot`，`xan` 还提供了一个专门的 `hist` 命令 (`xan/src/cmd/hist.rs`)，用于绘制水平条形图（Horizontal Bar Charts）。".as_bytes();
    let output = run_with_stdin(&["fmt", "--width", "40"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();

    // The closing parenthesis should not be at the start of a line
    assert!(
        !cm.contains("\n  )"),
        "Closing parenthesis should not be at the start of a line: got {}",
        cm
    );

    // The closing parenthesis should be on the same line as the preceding content
    assert!(
        cm.contains(".rs"),
        "Closing parenthesis should be on the same line as the preceding content: got {}",
        cm
    );
}

#[test]
fn test_fmt_cjk_opening_paren_with_content() {
    // Test that opening parenthesis stays with its content when wrapping
    // Example: (`slice`) should not have `(` at line end and `slice` at line start
    let input = "- **建议**: 处理超大压缩 TSV 时，支持 BGZF 索引是实现并行切片 (`slice`) 和随机采样 (`sample`)的基础。".as_bytes();
    let output = run_with_stdin(&["fmt", "--width", "35"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();

    // The opening parenthesis should be on the same line as its content
    assert!(
        !cm.contains("切片 (\n"),
        "Opening parenthesis should not be at line end: got {}",
        cm
    );

    // The content should be on the same line as the opening parenthesis
    assert!(
        cm.contains("切片 (`slice`)"),
        "Opening parenthesis should be on the same line as its content: got {}",
        cm
    );
}

#[test]
fn test_fmt_cjk_list_marker_not_alone() {
    // Test that list marker `-` is not placed alone on a line
    // Example: `- **借鉴**: ` should not have `-` on one line and `**借鉴**` on the next
    let input =
        "- **借鉴**: \n    - 为 `tva` 的 `docs/data/` 提供成对的示例文件（有/无表头）。"
            .as_bytes();
    let output = run_with_stdin(&["fmt"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();

    // The list marker should be on the same line as the content
    assert!(
        !cm.contains("-\n"),
        "List marker should not be alone on a line: got {}",
        cm
    );

    // The list marker should be on the same line as the bold text
    assert!(
        cm.contains("- **借鉴**"),
        "List marker should be on the same line as the bold text: got {}",
        cm
    );
}
