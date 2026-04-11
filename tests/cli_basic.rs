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
    assert!(help.contains("to"));
    assert!(help.contains("from"));
    assert!(help.contains("extract"));
    assert!(help.contains("stats"));
    assert!(help.contains("toc"));
    assert!(help.contains("fmt"));
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
fn test_cli_to_help() {
    let output = clmd_bin()
        .args(["to", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let help = String::from_utf8(output.stdout).unwrap();
    assert!(help.contains("html"));
    assert!(help.contains("--output"));
}

#[test]
fn test_cli_to_html_help() {
    let output = clmd_bin()
        .args(["to", "html", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let help = String::from_utf8(output.stdout).unwrap();
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
        .args(["-e", "table", "to", "html"])
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
        .args(["-e", "table", "-e", "strikethrough", "to", "html"])
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
        .args(["--safe", "to", "html"])
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
fn test_cli_all_extensions_enabled() {
    let mut child = clmd_bin()
        .args(["to", "html"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn command");

    {
        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        stdin
            .write_all(b"| table | header |\n|---|---|\n| data | value |")
            .expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to read stdout");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("<table>"),
        "Tables should be enabled by default"
    );
}
