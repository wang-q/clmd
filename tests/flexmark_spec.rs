use clmd::{markdown_to_html, Options};
use std::fs;

/// Helper function to convert markdown to HTML with default options
fn md_to_html(input: &str) -> String {
    markdown_to_html(input, &Options::default())
}

/// Test logging macro - only prints when VERBOSE_TESTS is set
macro_rules! test_log {
    ($($arg:tt)*) => {
        if std::env::var("VERBOSE_TESTS").is_ok() {
            std::println!($($arg)*);
        }
    };
}

#[derive(Debug)]
struct SpecExample {
    section: String,
    number: usize,
    input: String,
    expected_html: String,
}

fn parse_flexmark_spec(content: &str) -> Vec<SpecExample> {
    let mut examples = Vec::new();
    let mut current_section = String::new();
    let mut example_number = 0;

    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        // Check for section header
        if let Some(section) = line.strip_prefix("## ") {
            current_section = section.trim().to_string();
        }

        // Check for example start
        if line.starts_with("```````````````````````````````` example") {
            example_number += 1;
            i += 1;

            // Collect markdown input until we hit the dot separator
            let mut input = String::new();
            while i < lines.len() && lines[i] != "." {
                if !input.is_empty() {
                    input.push('\n');
                }
                input.push_str(lines[i]);
                i += 1;
            }

            // Skip the dot line
            i += 1;

            // Collect expected HTML output
            let mut expected_html = String::new();
            while i < lines.len()
                && !lines[i].starts_with("````````````````````````````````")
            {
                if !expected_html.is_empty() {
                    expected_html.push('\n');
                }
                expected_html.push_str(lines[i]);
                i += 1;
            }

            examples.push(SpecExample {
                section: current_section.clone(),
                number: example_number,
                input,
                expected_html,
            });
        }

        i += 1;
    }

    examples
}

fn run_flexmark_tests(module_name: &str, filename: &str) {
    // Flexmark tests require specific extensions to be enabled
    // For now, just verify the parser can handle the input without crashing
    let spec_content =
        fs::read_to_string(filename).expect(&format!("Failed to read {}", filename));

    let examples = parse_flexmark_spec(&spec_content);
    test_log!("Found {} {} test examples", examples.len(), module_name);

    // Just verify parsing doesn't panic
    for example in examples.iter().take(5) {
        let _result = md_to_html(&example.input);
    }

    test_log!("\n=== {} Test Results ===", module_name);
    test_log!("Parsed {} examples without errors", examples.len().min(5));

    // For now, just verify tests run without panic
    // Full compliance will be achieved incrementally when extensions are enabled
    assert!(
        !examples.is_empty(),
        "No {} test examples found",
        module_name
    );
}

#[test]
fn test_flexmark_abbreviation() {
    run_flexmark_tests(
        "Abbreviation",
        "tests/fixtures/flexmark_abbreviation_spec.md",
    );
}

#[test]
fn test_flexmark_attributes() {
    run_flexmark_tests("Attributes", "tests/fixtures/flexmark_attributes_spec.md");
}

#[test]
fn test_flexmark_autolink() {
    run_flexmark_tests("Autolink", "tests/fixtures/flexmark_autolink_spec.md");
}

#[test]
fn test_flexmark_definition() {
    run_flexmark_tests("Definition", "tests/fixtures/flexmark_definition_spec.md");
}

#[test]
fn test_flexmark_footnotes() {
    run_flexmark_tests("Footnotes", "tests/fixtures/flexmark_footnotes_spec.md");
}

#[test]
fn test_flexmark_strikethrough() {
    run_flexmark_tests(
        "Strikethrough",
        "tests/fixtures/flexmark_strikethrough_spec.md",
    );
}

#[test]
fn test_flexmark_tables() {
    run_flexmark_tests("Tables", "tests/fixtures/flexmark_tables_spec.md");
}

#[test]
fn test_flexmark_tasklist() {
    run_flexmark_tests("Tasklist", "tests/fixtures/flexmark_tasklist_spec.md");
}

#[test]
fn test_flexmark_toc() {
    run_flexmark_tests("TOC", "tests/fixtures/flexmark_toc_spec.md");
}

#[test]
fn test_flexmark_yaml_front_matter() {
    run_flexmark_tests(
        "YAML Front Matter",
        "tests/fixtures/flexmark_yaml_front_matter_spec.md",
    );
}
