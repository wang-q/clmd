//! CLI integration tests for line breaking
//!
//! These tests verify the `clmd fmt -w <width>` command behavior.

use clmd::text::unicode_width;
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

// =============================================================================
// Link Protection Tests
// =============================================================================

#[test]
fn test_cli_link_not_split() {
    let input = "这是一个链接 [示例](https://example.com) 测试。".as_bytes();
    let output = run_with_stdin(&["fmt", "--width", "40"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();

    // Link should not be split
    assert!(
        cm.contains("[示例](https://example.com)"),
        "Link should not be split: {}",
        cm
    );
}

#[test]
fn test_cli_long_link_not_split() {
    let input =
        "[Issue #Tests-Fail-JavaSwingTimers](https://github.com/vsch/idea-multimarkdown/issues/Tests-Fail-JavaSwingTimers)"
            .as_bytes();
    let output = run_with_stdin(&["fmt", "--width", "30"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();

    // Long link should not be split
    assert!(
        cm.contains("[Issue #Tests-Fail-JavaSwingTimers](https://github.com"),
        "Long link should not be split: {}",
        cm
    );
}

#[test]
fn test_cli_image_link_not_split() {
    let input =
        "![Issue #Tests-Fail-JavaSwingTimers](https://github.com/vsch/idea-multimarkdown/issues/Tests-Fail-JavaSwingTimers)"
            .as_bytes();
    let output = run_with_stdin(&["fmt", "--width", "30"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();

    // Image link should not be split
    assert!(
        cm.contains("![Issue #Tests-Fail-JavaSwingTimers](https://github.com"),
        "Image link should not be split: {}",
        cm
    );
}

#[test]
fn test_cli_link_with_cjk_text() {
    let input = "我们旨在重现 [eBay TSV Utilities](https://github.com/eBay/tsv-utils/blob/master/docs/comparative-benchmarks-2017.md) 使用的严格基准测试策略。".as_bytes();
    let output = run_with_stdin(&["fmt", "--width", "60"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();

    // Link should not be split
    assert!(
        cm.contains("[eBay TSV Utilities](https://github.com/eBay/tsv-utils"),
        "Link should not be split: {}",
        cm
    );
}

// =============================================================================
// CJK Punctuation Tests
// =============================================================================

#[test]
fn test_cli_cjk_comma_not_at_line_end() {
    let input =
        "- **建议**: 增强 `tva filter` 或新增 `tva search`，集成 `aho-corasick` crate 以支持高性能的多模式匹配。"
            .as_bytes();
    let output = run_with_stdin(&["fmt", "--width", "100"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();

    // CJK comma should not be at line end
    assert!(
        !cm.contains("，\n"),
        "CJK comma should not be at line end: {}",
        cm
    );
}

#[test]
fn test_cli_cjk_semicolon_not_at_line_end() {
    let input =
        "- **特色功能**: 支持日期补全 (`--dates`)，自动填充缺失的日期并设为 0；支持间隙压缩 (`--compress-gaps`)，隐藏连续的 0 值。"
            .as_bytes();
    let output = run_with_stdin(&["fmt", "--width", "100"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();

    // CJK semicolon should not be at line end
    assert!(
        !cm.contains("；\n"),
        "CJK semicolon should not be at line end: {}",
        cm
    );
}

// =============================================================================
// Markdown Marker Tests
// =============================================================================

#[test]
fn test_cli_emphasis_not_split() {
    let input = "这是一个 **强调文本** 和 *斜体* 的测试。".as_bytes();
    let output = run_with_stdin(&["fmt", "--width", "50"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();

    // Emphasis markers should not be split
    assert!(
        cm.contains("**强调文本**"),
        "Emphasis should not be split: {}",
        cm
    );
    assert!(cm.contains("*斜体*"), "Italic should not be split: {}", cm);
}

#[test]
fn test_cli_inline_code_not_split() {
    let input = "这是行内代码 `code example` 测试。".as_bytes();
    let output = run_with_stdin(&["fmt", "--width", "50"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();

    // Inline code should not be split
    assert!(
        cm.contains("`code example`"),
        "Inline code should not be split: {}",
        cm
    );
}

// =============================================================================
// English Punctuation Tests
// =============================================================================

#[test]
fn test_cli_no_space_after_opening_paren() {
    let input = "**HEPMASS** (\n  4.8GB)".as_bytes();
    let output = run_with_stdin(&["fmt"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();

    // No space after opening parenthesis
    assert!(
        !cm.contains("( 4.8GB)"),
        "There should be no space after `(`: {}",
        cm
    );
}

#[test]
fn test_cli_closing_paren_not_alone() {
    let input = "这是一个测试 (with English parentheses) 和更多内容。".as_bytes();
    let output = run_with_stdin(&["fmt", "--width", "30"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();

    // Closing paren should not be alone on a line
    let lines: Vec<&str> = cm.lines().collect();
    for line in &lines {
        // Skip lines that are just whitespace or empty
        if line.trim().is_empty() {
            continue;
        }
        // A line should not start with just ")"
        if line.trim().starts_with(')') && !line.trim().starts_with(") ") {
            // This is OK if it's part of a list or other structure
            if !line.starts_with("  ")
                && !line.starts_with("- ")
                && !line.starts_with("> ")
            {
                panic!("Closing paren should not be alone on a line: {}", line);
            }
        }
    }
}

// =============================================================================
// Line Balance Tests
// =============================================================================

#[test]
fn test_cli_line_balance() {
    let input =
        "这是一个比较长的段落，用于测试行长度是否均衡，不应该出现第一行很短而第二行很长的情况。".as_bytes();
    let output = run_with_stdin(&["fmt", "--width", "50"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();

    let lines: Vec<&str> = cm.lines().collect();
    if lines.len() > 1 {
        // Check that lines are reasonably balanced
        let first_line_len = unicode_width::width(lines[0]);
        let _second_line_len = unicode_width::width(lines[1]);

        // First line should not be too short (less than 50% of width)
        assert!(
            first_line_len >= 20,
            "First line too short: {} (width: {})",
            lines[0],
            first_line_len
        );
    }
}

// =============================================================================
// List Item Tests
// =============================================================================

#[test]
fn test_cli_list_item_wrapping() {
    let input = "* Paragraph with hard break and more text.".as_bytes();
    let output = run_with_stdin(&["fmt", "--width", "30"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();

    // Check that list item is properly wrapped
    // Note: formatter normalizes list markers to '-'
    let lines: Vec<&str> = cm.lines().collect();
    assert!(
        lines[0].starts_with("- "),
        "First line should start with '- ' (normalized from '* '): got '{}'",
        lines[0]
    );

    if lines.len() > 1 {
        assert!(
            lines[1].starts_with("  "),
            "Continuation line should be indented"
        );
    }
}

#[test]
fn test_cli_ordered_list_item_wrapping() {
    let input = "1. Paragraph with soft break and more text.".as_bytes();
    let output = run_with_stdin(&["fmt", "--width", "30"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();

    // Check that ordered list item is properly wrapped
    let lines: Vec<&str> = cm.lines().collect();
    assert!(
        lines[0].starts_with("1. "),
        "First line should start with '1. '"
    );

    if lines.len() > 1 {
        assert!(
            lines[1].starts_with("   "),
            "Continuation line should be indented"
        );
    }
}

// =============================================================================
// Block Quote Tests
// =============================================================================

#[test]
fn test_cli_block_quote_wrapping() {
    let input = "> This is a blockquote with some text.".as_bytes();
    let output = run_with_stdin(&["fmt", "--width", "25"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();

    // Check that block quote is properly wrapped
    let lines: Vec<&str> = cm.lines().collect();
    for line in &lines {
        // Skip empty lines (formatter may add trailing newline)
        if line.is_empty() {
            continue;
        }
        assert!(
            line.starts_with("> "),
            "Line should start with '> ': got '{}'",
            line
        );
    }
}

// =============================================================================
// Mixed Content Tests
// =============================================================================

#[test]
fn test_cli_mixed_content() {
    let input = "这是一个测试，包含 **强调**、`代码` 和 [链接](https://example.com)。"
        .as_bytes();
    let output = run_with_stdin(&["fmt", "--width", "50"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();

    // All elements should be preserved
    assert!(cm.contains("**强调**"), "Emphasis should be preserved");
    assert!(cm.contains("`代码`"), "Code should be preserved");
    assert!(
        cm.contains("[链接](https://example.com)"),
        "Link should be preserved"
    );
}
