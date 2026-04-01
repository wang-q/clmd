//! Command-style tests for clmd
//!
//! These tests are inspired by pandoc's test framework and use
//! markdown files to define test cases.

mod command;

use command::{load_command_tests, run_command_test};

/// Run all command tests from the tests/command directory
#[test]
fn test_command_tests() {
    let tests = load_command_tests("tests/command");

    if tests.is_empty() {
        println!("No command tests found in tests/command/");
        return;
    }

    println!("\n=== Command Tests ===");
    println!("Found {} test cases\n", tests.len());

    let mut passed = 0;
    let mut failed = 0;
    let mut failures: Vec<String> = Vec::new();

    for test in &tests {
        let result = run_command_test(test);

        if result.passed {
            passed += 1;
            println!("✓ {}", result.name);
        } else {
            failed += 1;
            println!("\n{}", result.format());
            failures.push(result.format());
        }
    }

    println!("\n=== Results ===");
    println!("Passed: {}/{}", passed, tests.len());
    println!("Failed: {}/{}", failed, tests.len());

    // Print summary of failures
    if !failures.is_empty() {
        println!("\n=== Failed Tests Summary ===");
        for failure in &failures {
            println!("{}", failure);
        }
    }

    // Assert that all tests passed
    assert_eq!(
        failed,
        0,
        "{} command test(s) failed out of {}",
        failed,
        tests.len()
    );
}

/// Test specific command test files
#[test]
fn test_headers() {
    run_tests_from_file("tests/command/headers.md");
}

#[test]
fn test_paragraphs() {
    run_tests_from_file("tests/command/paragraphs.md");
}

#[test]
fn test_emphasis() {
    run_tests_from_file("tests/command/emphasis.md");
}

fn run_tests_from_file(path: &str) {
    use command::parse_command_test_file;
    use std::fs;

    let content =
        fs::read_to_string(path).unwrap_or_else(|_| panic!("Failed to read {}", path));
    let filename = std::path::Path::new(path)
        .file_stem()
        .unwrap()
        .to_string_lossy();
    let tests = parse_command_test_file(&content, &filename);

    if tests.is_empty() {
        println!("No tests found in {}", path);
        return;
    }

    println!("\n=== {} Tests ===", filename);

    let mut passed = 0;
    let mut failed = 0;

    for test in &tests {
        let result = run_command_test(test);

        if result.passed {
            passed += 1;
            println!("✓ {}", result.name);
        } else {
            failed += 1;
            println!("\n{}", result.format());
        }
    }

    println!("\nResults: {}/{} passed", passed, tests.len());

    assert_eq!(
        failed, 0,
        "{} test(s) failed in {}",
        failed, filename
    );
}
