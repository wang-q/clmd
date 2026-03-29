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
fn test_toc_basic() {
    let input = r#"# Title

## Section 1

Content here.

## Section 2

More content.
"#;

    let output = run_with_stdin(&["toc"], input.as_bytes());

    assert!(output.status.success());
    let toc = String::from_utf8(output.stdout).unwrap();
    assert!(toc.contains("- Title"));
    assert!(toc.contains("  - Section 1"));
    assert!(toc.contains("  - Section 2"));
}

#[test]
fn test_toc_with_links() {
    let input = r#"# My Title

## First Section
"#;

    let output = run_with_stdin(&["toc", "--links"], input.as_bytes());

    assert!(output.status.success());
    let toc = String::from_utf8(output.stdout).unwrap();
    assert!(toc.contains("[My Title](#my-title)"));
    assert!(toc.contains("[First Section](#first-section)"));
}

#[test]
fn test_toc_numbered() {
    let input = r#"# Title

## Section 1

### Subsection

## Section 2
"#;

    let output = run_with_stdin(&["toc", "--numbered"], input.as_bytes());

    assert!(output.status.success());
    let toc = String::from_utf8(output.stdout).unwrap();
    assert!(toc.contains("1 Title"));
    assert!(toc.contains("1.1 Section 1"));
    assert!(toc.contains("1.1.1 Subsection"));
    assert!(toc.contains("1.2 Section 2"));
}

#[test]
fn test_toc_level_filter() {
    let input = r#"# Title

## Section 1

### Deep Section

## Section 2
"#;

    let output = run_with_stdin(&["toc", "-l", "1-2"], input.as_bytes());

    assert!(output.status.success());
    let toc = String::from_utf8(output.stdout).unwrap();
    assert!(toc.contains("- Title"));
    assert!(toc.contains("  - Section 1"));
    assert!(toc.contains("  - Section 2"));
    assert!(!toc.contains("Deep Section"));
}

#[test]
fn test_toc_single_level() {
    let input = r#"# Title

## Section 1

### Deep

## Section 2
"#;

    let output = run_with_stdin(&["toc", "-l", "2"], input.as_bytes());

    assert!(output.status.success());
    let toc = String::from_utf8(output.stdout).unwrap();
    assert!(!toc.contains("Title"));
    assert!(toc.contains("- Section 1"));
    assert!(toc.contains("- Section 2"));
}

#[test]
fn test_toc_empty_document() {
    let output = run_with_stdin(&["toc"], b"No headings here.");

    assert!(output.status.success());
    let toc = String::from_utf8(output.stdout).unwrap();
    assert!(toc.is_empty());
}

#[test]
fn test_toc_from_file() {
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "# Heading\n\nContent.").unwrap();

    let output = clmd_bin()
        .args(["toc", temp_file.path().to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let toc = String::from_utf8(output.stdout).unwrap();
    assert!(toc.contains("- Heading"));
}

#[test]
fn test_toc_output_to_file() {
    let input = b"# Title\n\n## Section";
    let output_file = NamedTempFile::new().unwrap();

    let output =
        run_with_stdin(&["toc", "-o", output_file.path().to_str().unwrap()], input);

    assert!(output.status.success());
    let toc = std::fs::read_to_string(output_file.path()).unwrap();
    assert!(toc.contains("- Title"));
    assert!(toc.contains("  - Section"));
}

#[test]
fn test_toc_with_special_characters() {
    let input = "# Title: Subtitle\n\n## Section & More\n\n### Code `inline`";

    let output = run_with_stdin(&["toc", "--links"], input.as_bytes());

    assert!(output.status.success());
    let toc = String::from_utf8(output.stdout).unwrap();
    // Check that special characters are handled in links
    assert!(toc.contains("(#"));
}
