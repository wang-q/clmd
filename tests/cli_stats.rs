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
fn test_stats_basic() {
    let output = run_with_stdin(&["stats"], b"# Hello\n\nWorld");

    assert!(output.status.success());
    let stats = String::from_utf8(output.stdout).unwrap();
    assert!(stats.contains("Lines:"));
    assert!(stats.contains("Words:"));
    assert!(stats.contains("Headings:"));
}

#[test]
fn test_stats_json_format() {
    let output = run_with_stdin(&["stats", "--format", "json"], b"# Title\n\nContent");

    assert!(output.status.success());
    let stats = String::from_utf8(output.stdout).unwrap();

    // Parse JSON to verify structure
    let json: serde_json::Value =
        serde_json::from_str(&stats).expect("Invalid JSON output");
    assert!(json.get("basic").is_some());
    assert!(json["basic"].get("lines").is_some());
    assert!(json["basic"].get("words").is_some());
    assert!(json.get("structure").is_some());
    assert!(json["structure"].get("headings").is_some());
}

#[test]
fn test_stats_detailed_counts() {
    let input = r#"# Heading 1
## Heading 2
### Heading 3

Some text with **bold** and *italic*.

[Link](https://example.com)

![Image](image.png)

```rust
fn main() {}
```

> Blockquote

---

- Item 1
- Item 2
"#;

    let output = run_with_stdin(&["stats", "--format", "json"], input.as_bytes());

    assert!(output.status.success());
    let stats = String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stats).unwrap();

    assert_eq!(json["structure"]["headings"]["h1"], 1);
    assert_eq!(json["structure"]["headings"]["h2"], 1);
    assert_eq!(json["structure"]["headings"]["h3"], 1);
    assert_eq!(json["structure"]["links"], 1);
    assert_eq!(json["structure"]["images"], 1);
    assert_eq!(json["code"]["code_blocks"], 1);
    assert_eq!(json["structure"]["blockquotes"], 1);
    assert_eq!(json["structure"]["thematic_breaks"], 1);
    assert_eq!(json["structure"]["lists"], 1);
    assert_eq!(json["structure"]["list_items"], 2);
}

#[test]
fn test_stats_from_file() {
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "# Test\n\nContent here.").unwrap();

    let output = clmd_bin()
        .args(["stats", temp_file.path().to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stats = String::from_utf8(output.stdout).unwrap();
    assert!(stats.contains("Lines:"));
    assert!(stats.contains("Words:"));
}

#[test]
fn test_stats_empty_document() {
    let output = run_with_stdin(&["stats", "--format", "json"], b"");

    assert!(output.status.success());
    let stats = String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stats).unwrap();
    assert_eq!(json["basic"]["lines"], 0);
    assert_eq!(json["basic"]["words"], 0);
}
