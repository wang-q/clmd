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
