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
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn command");

    {
        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        stdin.write_all(input).expect("Failed to write to stdin");
    }

    child.wait_with_output().expect("Failed to read stdout")
}

#[test]
fn test_to_docx_basic() {
    let input = b"# Hello World\n\nThis is a test document.";
    let output_file = NamedTempFile::new().unwrap();

    let output = run_with_stdin(
        &["to", "docx", "-o", output_file.path().to_str().unwrap()],
        input,
    );

    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify file exists and is not empty
    let metadata = std::fs::metadata(output_file.path()).unwrap();
    assert!(metadata.len() > 0, "DOCX file should not be empty");

    // Verify file starts with PK (ZIP magic bytes)
    let content = std::fs::read(output_file.path()).unwrap();
    assert!(content.len() >= 2, "DOCX file should have at least 2 bytes");
    assert_eq!(
        &content[0..2],
        b"PK",
        "DOCX should start with PK magic bytes"
    );
}

#[test]
fn test_to_docx_with_headings() {
    let input =
        b"# Title\n\n## Section 1\n\nContent here.\n\n## Section 2\n\nMore content.";
    let output_file = NamedTempFile::new().unwrap();

    let output = run_with_stdin(
        &["to", "docx", "-o", output_file.path().to_str().unwrap()],
        input,
    );

    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify file is a valid DOCX (ZIP format)
    let content = std::fs::read(output_file.path()).unwrap();
    assert_eq!(&content[0..2], b"PK", "Should be a valid ZIP/DOCX file");
}

#[test]
fn test_to_docx_with_list() {
    let input = b"# Shopping List\n\n- Apples\n- Bananas\n- Oranges";
    let output_file = NamedTempFile::new().unwrap();

    let output = run_with_stdin(
        &["to", "docx", "-o", output_file.path().to_str().unwrap()],
        input,
    );

    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let content = std::fs::read(output_file.path()).unwrap();
    assert_eq!(&content[0..2], b"PK", "Should be a valid DOCX file");
}

#[test]
fn test_to_docx_from_file() {
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "# Document\n\nTest content.").unwrap();

    let output_file = NamedTempFile::new().unwrap();

    let output = clmd_bin()
        .args([
            "to",
            "docx",
            temp_file.path().to_str().unwrap(),
            "-o",
            output_file.path().to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let content = std::fs::read(output_file.path()).unwrap();
    assert_eq!(&content[0..2], b"PK", "Should be a valid DOCX file");
}

#[test]
fn test_to_docx_auto_output_filename() {
    // Create a temporary directory to avoid polluting the working directory
    let temp_dir = tempfile::TempDir::new().unwrap();
    let input_path = temp_dir.path().join("test_input.md");

    // Create input file in the temp directory
    {
        let mut file = std::fs::File::create(&input_path).unwrap();
        write!(file, "# Test Document\n\nContent.").unwrap();
    }

    // Run without -o, should auto-generate output filename
    let output = clmd_bin()
        .args(["to", "docx", input_path.to_str().unwrap()])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute command");

    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Check that output file was created with .docx extension in the temp directory
    let expected_output = temp_dir.path().join("test_input.docx");
    assert!(
        expected_output.exists(),
        "Output file should be created: {:?}",
        expected_output
    );

    // Verify it's a valid DOCX file
    let content = std::fs::read(&expected_output).unwrap();
    assert_eq!(&content[0..2], b"PK", "Should be a valid DOCX file");

    // TempDir will be automatically cleaned up when it goes out of scope
}

#[test]
fn test_to_docx_complex_document() {
    let input = r#"# Project Report

## Executive Summary

This is a comprehensive report about the project.

## Key Findings

- **Performance**: Improved by 50%
- **Cost**: Reduced by 30%
- **Quality**: Maintained high standards

### Detailed Analysis

1. First quarter results were positive
2. Second quarter showed continued growth
3. Third quarter exceeded expectations

## Conclusion

The project was a success.

> "Success is not final, failure is not fatal."

```
Code example here
```
"#;

    let output_file = NamedTempFile::new().unwrap();

    let output = run_with_stdin(
        &["to", "docx", "-o", output_file.path().to_str().unwrap()],
        input.as_bytes(),
    );

    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let content = std::fs::read(output_file.path()).unwrap();
    assert!(
        content.len() > 100,
        "Complex document should produce larger file"
    );
    assert_eq!(&content[0..2], b"PK", "Should be a valid DOCX file");
}

// Regression test for the binary format output issue
#[test]
fn test_to_docx_outputs_binary_not_base64() {
    // This is a regression test for the issue where DOCX output was base64 encoded
    // instead of being written as binary
    let input = b"# Test\n\nContent.";
    let output_file = NamedTempFile::new().unwrap();

    let output = run_with_stdin(
        &["to", "docx", "-o", output_file.path().to_str().unwrap()],
        input,
    );

    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let content = std::fs::read(output_file.path()).unwrap();

    // Check that the file is a valid ZIP (DOCX) format, not base64 text
    // Base64 would be ASCII text, while DOCX is binary ZIP format
    assert_eq!(
        &content[0..2],
        b"PK",
        "Output should be binary ZIP format, not base64"
    );

    // The file should contain XML content (word/document.xml)
    // which is part of the DOCX structure
    let content_str = String::from_utf8_lossy(&content);
    assert!(
        content_str.contains("[Content_Types].xml")
            || content_str.contains("word/document"),
        "DOCX should contain expected internal structure"
    );
}
