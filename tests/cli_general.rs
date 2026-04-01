use std::io::Write;
use std::process::{Command, Stdio};

fn clmd_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_clmd"))
}

#[test]
fn test_cli_help() {
    let output = clmd_bin()
        .arg("--help")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let help = String::from_utf8(output.stdout).unwrap();
    assert!(help.contains("clmd: CommonMark Markdown processor"));
    assert!(help.contains("convert"));
    assert!(help.contains("extract"));
    assert!(help.contains("stats"));
    assert!(help.contains("toc"));
}

#[test]
fn test_cli_version() {
    let output = clmd_bin()
        .arg("--version")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let version = String::from_utf8(output.stdout).unwrap();
    assert!(version.contains("clmd"));
}

#[test]
fn test_cli_no_args_shows_help() {
    let output = clmd_bin().output().expect("Failed to execute command");

    // Should fail with exit code 2 (clap's default for missing required arg)
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Usage:"));
}

#[test]
fn test_cli_convert_help() {
    let output = clmd_bin()
        .args(["convert", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let help = String::from_utf8(output.stdout).unwrap();
    assert!(help.contains("to"));
    assert!(help.contains("from"));
}

#[test]
fn test_cli_convert_to_html_help() {
    let output = clmd_bin()
        .args(["convert", "to", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let help = String::from_utf8(output.stdout).unwrap();
    assert!(help.contains("html"));
    assert!(help.contains("--output"));
}

#[test]
fn test_cli_extract_help() {
    let output = clmd_bin()
        .args(["extract", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let help = String::from_utf8(output.stdout).unwrap();
    assert!(help.contains("links"));
    assert!(help.contains("headings"));
    assert!(help.contains("code"));
}

#[test]
fn test_cli_extension_flag() {
    let mut child = clmd_bin()
        .args(["-e", "table", "convert", "to", "html"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn command");

    {
        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        stdin
            .write_all(b"|a|b|\n|---|---|\n|c|d|")
            .expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to read stdout");
    assert!(output.status.success());
}

#[test]
fn test_cli_multiple_extensions() {
    let mut child = clmd_bin()
        .args([
            "-e",
            "table",
            "-e",
            "strikethrough",
            "convert",
            "to",
            "html",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn command");

    {
        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        stdin.write_all(b"test").expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to read stdout");
    assert!(output.status.success());
}

#[test]
fn test_cli_safe_mode() {
    // Test that --safe flag is accepted (actual safe mode functionality depends on library implementation)
    let mut child = clmd_bin()
        .args(["--safe", "convert", "to", "html"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn command");

    {
        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        stdin
            .write_all(b"# Test")
            .expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to read stdout");
    assert!(output.status.success());
}

#[test]
fn test_cli_unknown_extension_warning() {
    let mut child = clmd_bin()
        .args(["-e", "unknown_extension", "convert", "to", "html"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn command");

    {
        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        stdin.write_all(b"test").expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to read stdout");
    assert!(output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Warning: unknown extension"));
}
