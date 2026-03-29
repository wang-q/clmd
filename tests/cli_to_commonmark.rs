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
fn test_to_commonmark_basic() {
    let output = run_with_stdin(&["to", "commonmark"], b"# Hello\n\nWorld");

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();
    // CommonMark renderer outputs heading with newline after #
    assert!(cm.contains("# "));
    assert!(cm.contains("Hello"));
    assert!(cm.contains("World"));
}

#[test]
fn test_to_cm_alias() {
    let output = run_with_stdin(&["to", "cm"], b"# Test");

    assert!(output.status.success());
    let cm = String::from_utf8(output.stdout).unwrap();
    // CommonMark renderer outputs heading with newline after #
    assert!(cm.contains("# "));
    assert!(cm.contains("Test"));
}

#[test]
fn test_to_commonmark_from_file() {
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "**bold** text").unwrap();

    let output = clmd_bin()
        .args(["to", "commonmark", temp_file.path().to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
}

#[test]
fn test_to_commonmark_output_to_file() {
    let input = b"# Heading";
    let output_file = NamedTempFile::new().unwrap();

    let output = run_with_stdin(
        &[
            "to",
            "commonmark",
            "-o",
            output_file.path().to_str().unwrap(),
        ],
        input,
    );

    assert!(output.status.success());
    let cm = std::fs::read_to_string(output_file.path()).unwrap();
    // CommonMark renderer outputs heading with newline after #
    assert!(cm.contains("# "));
    assert!(cm.contains("Heading"));
}
