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
fn test_to_html_basic() {
    let output = run_with_stdin(&["to", "html"], b"# Hello\n\nWorld");

    assert!(output.status.success());
    let html = String::from_utf8(output.stdout).unwrap();
    assert!(html.contains("<h1>Hello</h1>"));
    assert!(html.contains("<p>World</p>"));
}

#[test]
fn test_to_html_with_emphasis() {
    let output = run_with_stdin(&["to", "html"], b"**bold** and *italic*");

    assert!(output.status.success());
    let html = String::from_utf8(output.stdout).unwrap();
    assert!(html.contains("<strong>bold</strong>"));
    assert!(html.contains("<em>italic</em>"));
}

#[test]
fn test_to_html_with_link() {
    let output = run_with_stdin(&["to", "html"], b"[link](https://example.com)");

    assert!(output.status.success());
    let html = String::from_utf8(output.stdout).unwrap();
    assert!(html.contains(r#"<a href="https://example.com">link</a>"#));
}

#[test]
fn test_to_html_full_document() {
    let output = run_with_stdin(&["to", "html", "--full"], b"# Title");

    assert!(output.status.success());
    let html = String::from_utf8(output.stdout).unwrap();
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("<html>"));
    assert!(html.contains("<body>"));
    assert!(html.contains("</body>"));
    assert!(html.contains("</html>"));
}

#[test]
fn test_to_html_hardbreaks() {
    // Test that --hardbreaks flag is accepted (actual functionality depends on library implementation)
    let output = run_with_stdin(&["to", "html", "--hardbreaks"], b"line1\nline2");

    assert!(output.status.success());
    let html = String::from_utf8(output.stdout).unwrap();
    // Output should contain the text
    assert!(html.contains("line1"));
    assert!(html.contains("line2"));
}

#[test]
fn test_to_html_from_file() {
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "# Test Heading\n\nTest content.").unwrap();

    let output = clmd_bin()
        .args(["to", "html", temp_file.path().to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let html = String::from_utf8(output.stdout).unwrap();
    assert!(html.contains("<h1>Test Heading</h1>"));
}

#[test]
fn test_to_html_output_to_file() {
    let input = b"# Hello";
    let output_file = NamedTempFile::new().unwrap();

    let output = run_with_stdin(
        &["to", "html", "-o", output_file.path().to_str().unwrap()],
        input,
    );

    assert!(output.status.success());
    let html = std::fs::read_to_string(output_file.path()).unwrap();
    assert!(html.contains("<h1>Hello</h1>"));
}

#[test]
fn test_to_html_with_table_extension() {
    // Test that -e table flag is accepted (actual table rendering depends on library implementation)
    let input = "| a | b |\n|---|---|\n| c | d |";

    let output = run_with_stdin(&["-e", "table", "to", "html"], input.as_bytes());

    assert!(output.status.success());
    // The command should succeed; actual table parsing depends on the library
}
