//! CLI spec tests
//!
//! These tests verify the CLI commands using spec files.

mod test_utils;
use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};
use test_utils::spec_parser::{parse_cli_spec_file, CliSpecExample};

fn clmd_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_clmd"))
}

fn run_cli_command(example: &CliSpecExample) -> (String, i32) {
    let mut cmd = clmd_bin();

    let parts: Vec<&str> = example.command.split_whitespace().collect();
    for part in parts {
        cmd.arg(part);
    }

    for arg in &example.args {
        cmd.arg(arg);
    }

    let mut child = cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn command");

    {
        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        stdin
            .write_all(example.input.as_bytes())
            .expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to read stdout");

    let stdout = String::from_utf8(output.stdout).unwrap_or_default();
    let exit_code = output.status.code().unwrap_or(-1);

    (stdout, exit_code)
}

fn run_cli_spec_file(spec_file: &str) {
    let content = fs::read_to_string(spec_file)
        .unwrap_or_else(|_| panic!("Failed to read {}", spec_file));

    let examples = parse_cli_spec_file(&content);
    println!("Found {} examples in {}", examples.len(), spec_file);

    let mut passed = 0;
    let mut failed = 0;
    let mut failures: Vec<(String, usize, String, String, String, i32)> = Vec::new();

    for example in &examples {
        let (stdout, exit_code) = run_cli_command(example);

        let expected = example.expected_output.replace("\r\n", "\n");
        let actual = stdout.replace("\r\n", "\n");

        let expected_normalized = expected.trim_end();
        let actual_normalized = actual.trim_end();

        if actual_normalized == expected_normalized && exit_code == example.expected_exit_code {
            passed += 1;
        } else {
            failed += 1;
            if failures.len() < 10 {
                failures.push((
                    example.section.clone(),
                    example.number,
                    example.input.clone(),
                    example.expected_output.clone(),
                    stdout,
                    exit_code,
                ));
            }
        }
    }

    println!("\n=== CLI Spec Test Results ===");
    println!(
        "Passed: {}/{} ({:.1}%)",
        passed,
        examples.len(),
        (passed as f64 / examples.len() as f64) * 100.0
    );
    println!(
        "Failed: {}/{} ({:.1}%)",
        failed,
        examples.len(),
        (failed as f64 / examples.len() as f64) * 100.0
    );

    if !failures.is_empty() {
        println!("\n=== Failed Tests ===");
        for (section, number, input, expected, actual, exit_code) in &failures {
            println!("\n{}:{}", section, number);
            println!("Input:\n{}", input);
            println!("Expected:\n{}", expected);
            println!("Actual:\n{}", actual);
            println!("Exit code: {}", exit_code);
        }
    }

    assert!(
        failed == 0,
        "{} tests failed out of {}",
        failed,
        examples.len()
    );
}

#[test]
fn test_cli_extract_spec() {
    run_cli_spec_file("tests/fixtures/cli_extract_spec.md");
}

#[test]
fn test_cli_fmt_cjk_spacing_spec() {
    run_cli_spec_file("tests/fixtures/cli_fmt_cjk_spacing_spec.md");
}

#[test]
fn test_cli_fmt_line_breaking_spec() {
    run_cli_spec_file("tests/fixtures/cli_fmt_line_breaking_spec.md");
}

#[test]
fn test_cli_fmt_spec() {
    run_cli_spec_file("tests/fixtures/cli_fmt_spec.md");
}
