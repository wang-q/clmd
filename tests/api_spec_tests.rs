//! API spec tests
//!
//! These tests verify the library API using spec files.

mod test_utils;
use clmd::{markdown_to_html, Options, Plugins};
use std::fs;
use test_utils::spec_parser::{parse_api_spec_file, ApiSpecExample};

fn run_api_example(example: &ApiSpecExample) -> String {
    match example.function.as_str() {
        "html" => markdown_to_html(&example.input, &Options::default(), &Plugins::default()),
        _ => panic!("Unknown API function: {}", example.function),
    }
}

fn run_api_spec_file(spec_file: &str) {
    let content = fs::read_to_string(spec_file)
        .unwrap_or_else(|_| panic!("Failed to read {}", spec_file));

    let examples = parse_api_spec_file(&content);
    println!("Found {} examples in {}", examples.len(), spec_file);

    let mut passed = 0;
    let mut failed = 0;
    let mut failures: Vec<(String, usize, String, String, String)> = Vec::new();

    for example in &examples {
        let output = run_api_example(example);

        let expected = example.expected_output.replace("\r\n", "\n");
        let actual = output.replace("\r\n", "\n");

        let expected_normalized = expected.trim_end();
        let actual_normalized = actual.trim_end();

        if actual_normalized == expected_normalized {
            passed += 1;
        } else {
            failed += 1;
            if failures.len() < 10 {
                failures.push((
                    example.section.clone(),
                    example.number,
                    example.input.clone(),
                    example.expected_output.clone(),
                    output,
                ));
            }
        }
    }

    println!("\n=== API Spec Test Results ===");
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
        for (section, number, input, expected, actual) in &failures {
            println!("\n{}:{}", section, number);
            println!("Input:\n{}", input);
            println!("Expected:\n{}", expected);
            println!("Actual:\n{}", actual);
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
fn test_integration_spec() {
    run_api_spec_file("tests/fixtures/integration_spec.md");
}
