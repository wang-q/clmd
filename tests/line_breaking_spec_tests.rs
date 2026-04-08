//! Line breaking spec tests
//!
//! These tests verify the line breaking behavior of the CommonMark formatter.

use clmd::options::format::FormatOptions;
use clmd::options::Options as ParseOptions;
use clmd::parse::parse_document;
use clmd::render::commonmark::{CommonMarkNodeFormatter, Formatter};
use std::fs;

mod test_utils;
use test_utils::spec_parser::parse_formatter_spec_file;

/// Apply spec options to FormatOptions
fn apply_spec_options(options: &mut FormatOptions, option_str: &str) {
    match option_str {
        // Margin options (formatting width)
        opt if opt.starts_with("margin[") => {
            if let Some(end) = opt.find(']') {
                if let Ok(width) = opt[7..end].parse::<usize>() {
                    options.right_margin = width;
                }
            }
        }

        _ => {
            // Unknown option - ignore
        }
    }
}

/// Format markdown input using the given options
fn format_markdown(input: &str, options: &FormatOptions) -> String {
    // Parse the input
    let parse_options = ParseOptions::default();
    let (arena, root) = parse_document(input, &parse_options);

    // Format using the formatter
    let mut formatter = Formatter::with_options(options.clone());
    formatter.add_node_formatter(Box::new(CommonMarkNodeFormatter::with_options(
        options.clone(),
    )));

    formatter.render(&arena, root)
}

/// Run a single formatter spec example
fn run_formatter_example(example: &test_utils::spec_parser::FormatterSpecExample) {
    let mut options = FormatOptions::default();

    // Apply spec options
    for opt in &example.options {
        apply_spec_options(&mut options, opt);
    }

    // Format the input
    let output = format_markdown(&example.input, &options);

    // Normalize line endings for comparison
    let expected = example.expected_output.replace("\r\n", "\n");
    let actual = output.replace("\r\n", "\n");

    // Trim trailing whitespace and normalize for comparison
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
                    let mut options = FormatOptions::default();
                    for opt in &example.options {
                        apply_spec_options(&mut options, opt);
                    }
                    let output = format_markdown(&example.input, &options);
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

    println!("\n=== Line Breaking Spec Test Results ===");
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
fn test_line_breaking_spec() {
    run_formatter_spec_file("tests/fixtures/line_breaking_spec.md");
}
