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
    assert!(cm.contains("```rust"), "Should contain opening fence: {}", cm);
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
    assert!(cm.contains("- Item 1"), "Should contain outer item 1: {}", cm);
    assert!(cm.contains("- Item 2"), "Should contain outer item 2: {}", cm);
    assert!(cm.contains("  - Nested 1"), "Should contain nested item 1: {}", cm);
    assert!(cm.contains("  - Nested 2"), "Should contain nested item 2: {}", cm);
}

#[test]
fn test_fmt_link() {
    let input = b"[example](https://example.com)";
    let output = run_with_stdin(&["fmt"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();
    assert!(cm.contains("[example](https://example.com)"), "Should contain link: {}", cm);
}

#[test]
fn test_fmt_image() {
    let input = b"![alt text](image.png)";
    let output = run_with_stdin(&["fmt"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();
    assert!(cm.contains("![alt text](image.png)"), "Should contain image: {}", cm);
}

#[test]
fn test_fmt_blockquote() {
    let input = b"> This is a quote";
    let output = run_with_stdin(&["fmt"], input);

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();
    assert!(cm.contains("> This is a quote"), "Should contain blockquote: {}", cm);
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
    assert!(cm.contains("# Document Title"), "Should contain title: {}", cm);
    assert!(cm.contains("## Section 1: Lists"), "Should contain section 1: {}", cm);
    assert!(cm.contains("## Section 2: Code"), "Should contain section 2: {}", cm);
    assert!(cm.contains("## Section 3: Table"), "Should contain section 3: {}", cm);

    // Verify formatting is preserved
    assert!(cm.contains("**bold**"), "Should preserve bold: {}", cm);
    assert!(cm.contains("*italic*"), "Should preserve italic: {}", cm);
    assert!(cm.contains("```rust"), "Should preserve code block: {}", cm);
    assert!(cm.contains("| Name"), "Should preserve table: {}", cm);
    assert!(cm.contains("> A blockquote"), "Should preserve blockquote: {}", cm);
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
    assert!(cm.is_empty() || cm.trim().is_empty(), "Empty input should produce empty output: {:?}", cm);
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
