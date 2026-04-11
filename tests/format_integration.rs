//! Formatter integration spec tests
//!
//! These tests verify the end-to-end formatting functionality using the public API.

use clmd::options::format::FormatOptions;
use clmd::parse::parse_document;
use clmd::render::commonmark::Formatter;
use clmd::Options;
use std::fs;

mod test_utils;
use test_utils::spec_parser::parse_formatter_spec_file;

/// Apply spec options to ParseOptions and FormatOptions
fn apply_spec_options(
    parse_options: &mut Options,
    options: &mut FormatOptions,
    option_str: &str,
) {
    match option_str {
        // Format options
        "margin[80]" => options.right_margin = 80,
        "margin[100]" => options.right_margin = 100,
        "margin[120]" => options.right_margin = 120,

        _ => {
            // Handle margin[N] format
            if option_str.starts_with("margin[") {
                if let Some(end) = option_str.find(']') {
                    if let Ok(width) = option_str[7..end].parse::<usize>() {
                        options.right_margin = width;
                    }
                }
            }
        }
    }
}

/// Format markdown input using the given options
fn format_markdown(
    input: &str,
    parse_options: &Options,
    format_options: &FormatOptions,
) -> String {
    let (arena, root) = parse_document(input, parse_options);

    let formatter = Formatter::with_options(format_options.clone());

    formatter.render(&arena, root)
}

/// Run a single formatter spec example
fn run_formatter_example(example: &test_utils::spec_parser::FormatterSpecExample) {
    let mut parse_options = Options::default();
    let mut format_options = FormatOptions::default();

    for opt in &example.options {
        apply_spec_options(&mut parse_options, &mut format_options, opt);
    }

    let output = format_markdown(&example.input, &parse_options, &format_options);

    let expected = example.expected_output.replace("\r\n", "\n");
    let actual = output.replace("\r\n", "\n");

    // Normalize for comparison
    let expected_normalized = expected.trim_end();
    let actual_normalized = actual.trim_end();

    assert_eq!(
        actual_normalized,
        expected_normalized,
        "Test {}:{} failed\nOptions: {:?}\nInput:\n{}\n\nExpected:\n{}\n\nActual:\n{}",
        example.section,
        example.number,
        example.options,
        example.input,
        expected,
        actual
    );
}

/// Run formatter spec tests from a file
fn run_formatter_spec_file(spec_file: &str) {
    let content = fs::read_to_string(spec_file)
        .unwrap_or_else(|_| panic!("Failed to read {}", spec_file));

    let examples = parse_formatter_spec_file(&content);
    println!("Found {} examples in {}", examples.len(), spec_file);

    let mut passed = 0;
    let mut failed = 0;
    let mut failures: Vec<(String, usize, String, String, String)> = Vec::new();

    for example in &examples {
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            run_formatter_example(example);
        })) {
            Ok(_) => passed += 1,
            Err(_) => {
                failed += 1;
                if failures.len() < 10 {
                    let mut parse_options = Options::default();
                    let mut format_options = FormatOptions::default();
                    for opt in &example.options {
                        apply_spec_options(&mut parse_options, &mut format_options, opt);
                    }
                    let output =
                        format_markdown(&example.input, &parse_options, &format_options);
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
    }

    println!("\n=== Formatter Integration Spec Test Results ===");
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
fn test_formatter_integration_spec() {
    run_formatter_spec_file("tests/fixtures/formatter_integration_spec.md");
}
