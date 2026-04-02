use std::io::Write;
use std::process::{Command, Stdio};
use tempfile::NamedTempFile;

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
fn test_fmt_basic() {
    let output = run_with_stdin(&["fmt"], b"# Hello\n\nWorld");

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();
    // CommonMark renderer outputs heading with newline after #
    assert!(cm.contains("# "));
    assert!(cm.contains("Hello"));
    assert!(cm.contains("World"));
}

#[test]
fn test_fmt_from_file() {
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "**bold** text").unwrap();

    let output = clmd_bin()
        .args(["fmt", temp_file.path().to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
}

#[test]
fn test_fmt_output_to_file() {
    let input = b"# Heading";
    let output_file = NamedTempFile::new().unwrap();

    let output =
        run_with_stdin(&["fmt", "-o", output_file.path().to_str().unwrap()], input);

    assert!(output.status.success());
    let cm = std::fs::read_to_string(output_file.path()).unwrap();
    // CommonMark renderer outputs heading with newline after #
    assert!(cm.contains("# "));
    assert!(cm.contains("Heading"));
}

#[test]
fn test_fmt_headings() {
    let input = b"# Heading 1\n\n## Heading 2\n\n### Heading 3";
    let output = run_with_stdin(&["fmt"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();
    assert!(cm.contains("# Heading 1"), "Should contain h1: {}", cm);
    assert!(cm.contains("## Heading 2"), "Should contain h2: {}", cm);
    assert!(cm.contains("### Heading 3"), "Should contain h3: {}", cm);
}

#[test]
fn test_fmt_emphasis() {
    let input = b"This is *italic* and **bold** text.";
    let output = run_with_stdin(&["fmt"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();
    assert!(cm.contains("*italic*"), "Should contain italic: {}", cm);
    assert!(cm.contains("**bold**"), "Should contain bold: {}", cm);
}

#[test]
fn test_fmt_code_block() {
    let input = b"```rust\nfn main() {\n    println!(\"Hello\");\n}\n```";
    let output = run_with_stdin(&["fmt"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();
    assert!(
        cm.contains("```rust"),
        "Should contain opening fence: {}",
        cm
    );
    assert!(cm.contains("fn main()"), "Should contain code: {}", cm);
    assert!(cm.contains("```"), "Should contain closing fence: {}", cm);
}

#[test]
fn test_fmt_list_bullet() {
    let input = b"- Item 1\n- Item 2\n- Item 3";
    let output = run_with_stdin(&["fmt"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();
    assert!(cm.contains("- Item 1"), "Should contain item 1: {}", cm);
    assert!(cm.contains("- Item 2"), "Should contain item 2: {}", cm);
    assert!(cm.contains("- Item 3"), "Should contain item 3: {}", cm);
}

#[test]
fn test_fmt_list_ordered() {
    let input = b"1. First\n2. Second\n3. Third";
    let output = run_with_stdin(&["fmt"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();
    assert!(cm.contains("1."), "Should contain ordered item 1: {}", cm);
    assert!(cm.contains("2."), "Should contain ordered item 2: {}", cm);
    assert!(cm.contains("3."), "Should contain ordered item 3: {}", cm);
}

#[test]
fn test_fmt_nested_list() {
    let input = b"- Item 1\n- Item 2\n  - Nested 1\n  - Nested 2";
    let output = run_with_stdin(&["fmt"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();
    assert!(
        cm.contains("- Item 1"),
        "Should contain outer item 1: {}",
        cm
    );
    assert!(
        cm.contains("- Item 2"),
        "Should contain outer item 2: {}",
        cm
    );
    assert!(
        cm.contains("  - Nested 1"),
        "Should contain nested item 1: {}",
        cm
    );
    assert!(
        cm.contains("  - Nested 2"),
        "Should contain nested item 2: {}",
        cm
    );
}

#[test]
fn test_fmt_link() {
    let input = b"[example](https://example.com)";
    let output = run_with_stdin(&["fmt"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();
    assert!(
        cm.contains("[example](https://example.com)"),
        "Should contain link: {}",
        cm
    );
}

#[test]
fn test_fmt_image() {
    let input = b"![alt text](image.png)";
    let output = run_with_stdin(&["fmt"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();
    assert!(
        cm.contains("![alt text](image.png)"),
        "Should contain image: {}",
        cm
    );
}

#[test]
fn test_fmt_blockquote() {
    let input = b"> This is a quote";
    let output = run_with_stdin(&["fmt"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();
    assert!(
        cm.contains("> This is a quote"),
        "Should contain blockquote: {}",
        cm
    );
}

#[test]
fn test_fmt_table() {
    let input = b"| Name | Age |\n|------|-----|\n| Alice | 30 |\n| Bob | 25 |";
    let output = run_with_stdin(&["fmt"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();
    assert!(cm.contains("Name"), "Should contain header Name: {}", cm);
    assert!(cm.contains("Age"), "Should contain header Age: {}", cm);
    assert!(cm.contains("Alice"), "Should contain cell Alice: {}", cm);
    assert!(cm.contains("Bob"), "Should contain cell Bob: {}", cm);
    assert!(cm.contains("---"), "Should contain delimiter row: {}", cm);
}

#[test]
fn test_fmt_thematic_break() {
    let input = b"---";
    let output = run_with_stdin(&["fmt"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();
    assert!(cm.contains("***"), "Should contain thematic break: {}", cm);
}

#[test]
fn test_fmt_complex_document() {
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

> A blockquote with some text.
"#;

    let output = run_with_stdin(&["fmt"], input.as_bytes());

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();

    // Verify all sections are present
    assert!(
        cm.contains("# Document Title"),
        "Should contain title: {}",
        cm
    );
    assert!(
        cm.contains("## Section 1: Lists"),
        "Should contain section 1: {}",
        cm
    );
    assert!(
        cm.contains("## Section 2: Code"),
        "Should contain section 2: {}",
        cm
    );
    assert!(
        cm.contains("## Section 3: Table"),
        "Should contain section 3: {}",
        cm
    );

    // Verify formatting is preserved
    assert!(cm.contains("**bold**"), "Should preserve bold: {}", cm);
    assert!(cm.contains("*italic*"), "Should preserve italic: {}", cm);
    assert!(cm.contains("```rust"), "Should preserve code block: {}", cm);
    assert!(cm.contains("| Name"), "Should preserve table: {}", cm);
    assert!(
        cm.contains("> A blockquote"),
        "Should preserve blockquote: {}",
        cm
    );
}

#[test]
fn test_fmt_with_width_option() {
    let input = b"# Heading\n\nSome text here.";
    let output = run_with_stdin(&["fmt", "--width", "80"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();
    assert!(cm.contains("# Heading"), "Should contain heading: {}", cm);
}

#[test]
fn test_fmt_empty_input() {
    let input = b"";
    let output = run_with_stdin(&["fmt"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();
    // Empty input should produce empty or minimal output
    assert!(
        cm.is_empty() || cm.trim().is_empty(),
        "Empty input should produce empty output: {:?}",
        cm
    );
}

#[test]
fn test_fmt_from_file_with_extensions() {
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "| Name | Age |\n|------|-----|\n| Alice | 30 |").unwrap();

    let output = clmd_bin()
        .args(["fmt", temp_file.path().to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();
    assert!(cm.contains("Alice"), "Should contain table content: {}", cm);
}

// Regression tests for blank line handling

#[test]
fn test_fmt_list_followed_by_heading_has_blank_line() {
    // Regression test: List followed by heading should have blank line
    let input = b"# Title\n- item 1\n- item 2\n## Section";
    let output = run_with_stdin(&["fmt"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();

    // Verify list items are present
    assert!(cm.contains("- item 1"), "Should contain item 1: {}", cm);
    assert!(cm.contains("- item 2"), "Should contain item 2: {}", cm);
    assert!(cm.contains("## Section"), "Should contain heading: {}", cm);

    // Verify there's a blank line between list and heading
    // The output should have "- item 2" followed by blank line, then "## Section"
    let lines: Vec<&str> = cm.lines().collect();
    let item2_idx = lines
        .iter()
        .position(|&l| l == "- item 2")
        .expect("item 2 not found");
    let heading_idx = lines
        .iter()
        .position(|&l| l == "## Section")
        .expect("heading not found");

    assert!(
        heading_idx > item2_idx + 1,
        "There should be at least one blank line between list and heading.\nOutput:\n{}",
        cm
    );

    // Verify the line between is blank (or contains only whitespace)
    if heading_idx == item2_idx + 2 {
        let middle_line = lines[item2_idx + 1];
        assert!(
            middle_line.trim().is_empty(),
            "Line between list and heading should be blank, got: {:?}\nOutput:\n{}",
            middle_line,
            cm
        );
    }
}

#[test]
fn test_fmt_code_block_followed_by_heading_has_blank_line() {
    // Regression test: Code block followed by heading should have blank line
    let input = b"# Title\n\n```\ncode\n```\n## Section";
    let output = run_with_stdin(&["fmt"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();

    // Verify code block and heading are present
    assert!(cm.contains("```"), "Should contain code fence: {}", cm);
    assert!(cm.contains("code"), "Should contain code content: {}", cm);
    assert!(cm.contains("## Section"), "Should contain heading: {}", cm);

    // Verify there's a blank line between code block and heading
    let lines: Vec<&str> = cm.lines().collect();
    let code_fence_idx = lines
        .iter()
        .position(|&l| l == "```")
        .expect("code fence not found");
    let heading_idx = lines
        .iter()
        .position(|&l| l == "## Section")
        .expect("heading not found");

    // Find the closing code fence (second occurrence)
    let closing_fence_idx = lines[code_fence_idx + 1..]
        .iter()
        .position(|&l| l == "```")
        .map(|i| i + code_fence_idx + 1)
        .expect("closing code fence not found");

    assert!(
        heading_idx > closing_fence_idx + 1,
        "There should be at least one blank line between code block and heading.\nOutput:\n{}",
        cm
    );
}

#[test]
fn test_fmt_nested_list_followed_by_heading_has_blank_line() {
    // Regression test: Nested list followed by heading should have blank line
    let input = b"# Title\n- item 1\n  - nested 1\n  - nested 2\n- item 2\n## Section";
    let output = run_with_stdin(&["fmt"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();

    // Verify nested list items are present
    assert!(cm.contains("- item 1"), "Should contain item 1: {}", cm);
    assert!(
        cm.contains("  - nested 1"),
        "Should contain nested 1: {}",
        cm
    );
    assert!(
        cm.contains("  - nested 2"),
        "Should contain nested 2: {}",
        cm
    );
    assert!(cm.contains("- item 2"), "Should contain item 2: {}", cm);
    assert!(cm.contains("## Section"), "Should contain heading: {}", cm);

    // Verify there's a blank line between list and heading
    let lines: Vec<&str> = cm.lines().collect();
    let item2_idx = lines
        .iter()
        .position(|&l| l == "- item 2")
        .expect("item 2 not found");
    let heading_idx = lines
        .iter()
        .position(|&l| l == "## Section")
        .expect("heading not found");

    assert!(
        heading_idx > item2_idx + 1,
        "There should be at least one blank line between nested list and heading.\nOutput:\n{}",
        cm
    );
}

#[test]
fn test_fmt_multiple_lists_followed_by_heading_has_blank_line() {
    // Regression test: Multiple lists followed by heading should have blank line
    let input =
        b"# Title\n- bullet 1\n- bullet 2\n\n1. ordered 1\n2. ordered 2\n## Section";
    let output = run_with_stdin(&["fmt"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();

    // Verify both lists are present
    assert!(cm.contains("- bullet 1"), "Should contain bullet 1: {}", cm);
    assert!(cm.contains("- bullet 2"), "Should contain bullet 2: {}", cm);
    assert!(cm.contains("1."), "Should contain ordered item: {}", cm);
    assert!(cm.contains("## Section"), "Should contain heading: {}", cm);

    // Verify there's a blank line between last list and heading
    let lines: Vec<&str> = cm.lines().collect();
    let heading_idx = lines
        .iter()
        .position(|&l| l == "## Section")
        .expect("heading not found");

    // Find the last list item (could be ordered list)
    let last_list_idx = lines
        .iter()
        .rposition(|&l| {
            l.starts_with("- ")
                || l.trim()
                    .starts_with(|c: char| c.is_ascii_digit() && l.contains('.'))
        })
        .expect("list item not found");

    assert!(
        heading_idx > last_list_idx + 1,
        "There should be at least one blank line between lists and heading.\nOutput:\n{}",
        cm
    );
}
