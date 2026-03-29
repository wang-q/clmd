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
fn test_extract_links_basic() {
    let input = r#"[Link1](https://example.com)

[Link2](https://test.com "title")
"#;

    let output = run_with_stdin(&["extract", "links"], input.as_bytes());

    assert!(output.status.success());
    let result = String::from_utf8(output.stdout).unwrap();
    assert!(result.contains("Link1"));
    assert!(result.contains("https://example.com"));
    assert!(result.contains("Link2"));
    assert!(result.contains("https://test.com"));
}

#[test]
fn test_extract_links_json_format() {
    let input = "[Test](https://example.com)";

    let output =
        run_with_stdin(&["extract", "links", "--format", "json"], input.as_bytes());

    assert!(output.status.success());
    let result = String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&result).expect("Invalid JSON");
    assert!(json.is_array());
    assert_eq!(json[0]["text"], "Test");
    assert_eq!(json[0]["url"], "https://example.com");
}

#[test]
fn test_extract_headings_basic() {
    let input = r#"# Title

## Section 1

### Deep

## Section 2
"#;

    let output = run_with_stdin(&["extract", "headings"], input.as_bytes());

    assert!(output.status.success());
    let result = String::from_utf8(output.stdout).unwrap();
    assert!(result.contains("1\tTitle"));
    assert!(result.contains("2\tSection 1"));
    assert!(result.contains("3\tDeep"));
    assert!(result.contains("2\tSection 2"));
}

#[test]
fn test_extract_headings_level_filter() {
    let input = r#"# Title

## Section 1

### Deep

## Section 2
"#;

    let output = run_with_stdin(&["extract", "headings", "-l", "2"], input.as_bytes());

    assert!(output.status.success());
    let result = String::from_utf8(output.stdout).unwrap();
    assert!(!result.contains("Title"));
    assert!(result.contains("2\tSection 1"));
    assert!(result.contains("2\tSection 2"));
    assert!(!result.contains("Deep"));
}

#[test]
fn test_extract_headings_json_format() {
    let input = "# Heading";

    let output = run_with_stdin(
        &["extract", "headings", "--format", "json"],
        input.as_bytes(),
    );

    assert!(output.status.success());
    let result = String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&result).expect("Invalid JSON");
    assert!(json.is_array());
    assert_eq!(json[0]["level"], 1);
    assert_eq!(json[0]["text"], "Heading");
}

#[test]
fn test_extract_code_basic() {
    let input = r#"Some text.

```rust
fn main() {
    println!("Hello");
}
```

More text.

```python
print("world")
```
"#;

    let output = run_with_stdin(&["extract", "code"], input.as_bytes());

    assert!(output.status.success());
    let result = String::from_utf8(output.stdout).unwrap();
    assert!(result.contains("```rust"));
    assert!(result.contains("fn main()"));
    assert!(result.contains("```python"));
    assert!(result.contains("print"));
}

#[test]
fn test_extract_code_no_language() {
    let input = r#"```
plain code
```"#;

    let output = run_with_stdin(&["extract", "code"], input.as_bytes());

    assert!(output.status.success());
    let result = String::from_utf8(output.stdout).unwrap();
    assert!(result.contains("```"));
    assert!(result.contains("plain code"));
}

#[test]
fn test_extract_from_file() {
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "[Link](https://example.com)").unwrap();

    let output = clmd_bin()
        .args(["extract", "links", temp_file.path().to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let result = String::from_utf8(output.stdout).unwrap();
    assert!(result.contains("Link"));
    assert!(result.contains("https://example.com"));
}

#[test]
fn test_extract_output_to_file() {
    let input = b"# Heading";
    let output_file = NamedTempFile::new().unwrap();

    let output = run_with_stdin(
        &[
            "extract",
            "headings",
            "-o",
            output_file.path().to_str().unwrap(),
        ],
        input,
    );

    assert!(output.status.success());
    let result = std::fs::read_to_string(output_file.path()).unwrap();
    assert!(result.contains("Heading"));
}

#[test]
fn test_extract_no_links() {
    let output = run_with_stdin(&["extract", "links"], b"No links here.");

    assert!(output.status.success());
    let result = String::from_utf8(output.stdout).unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_extract_no_headings() {
    let output = run_with_stdin(&["extract", "headings"], b"No headings here.");

    assert!(output.status.success());
    let result = String::from_utf8(output.stdout).unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_extract_no_code() {
    let output = run_with_stdin(&["extract", "code"], b"No code here.");

    assert!(output.status.success());
    let result = String::from_utf8(output.stdout).unwrap();
    assert!(result.is_empty());
}
