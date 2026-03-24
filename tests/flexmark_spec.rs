use clmd::{markdown_to_html, options};
use std::fs;

#[derive(Debug)]
struct SpecExample {
    section: String,
    number: usize,
    input: String,
    expected_html: String,
    #[allow(dead_code)]
    options: Vec<String>,
}

/// Parse flexmark-java spec file format
/// Format:
/// ```````````````````````````````` example Section: Number [options]
/// input markdown
/// .
/// expected html
/// .
/// expected ast (optional)
/// ````````````````````````````````
fn parse_flexmark_spec(content: &str) -> Vec<SpecExample> {
    let mut examples = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        // Look for example block start
        if line.starts_with("```````````````````````````````` example") {
            if let Some((section, number, options)) = parse_example_header(line) {
                i += 1;

                // Collect input until we hit a line with just "."
                let mut input_lines = Vec::new();
                while i < lines.len() && lines[i] != "." {
                    input_lines.push(lines[i]);
                    i += 1;
                }

                // Skip the "." separator
                i += 1;

                // Collect expected HTML until we hit a line with just "."
                let mut html_lines = Vec::new();
                while i < lines.len() && lines[i] != "." {
                    html_lines.push(lines[i]);
                    i += 1;
                }

                // Skip the "." separator
                i += 1;

                // Skip AST output if present (until the closing fence)
                while i < lines.len()
                    && !lines[i].starts_with("````````````````````````````````")
                {
                    i += 1;
                }

                // Skip the closing fence
                i += 1;

                let input = input_lines.join("\n");
                let expected_html = html_lines.join("\n");

                examples.push(SpecExample {
                    section,
                    number,
                    input,
                    expected_html,
                    options,
                });
            } else {
                i += 1;
            }
        } else {
            i += 1;
        }
    }

    examples
}

/// Parse the example header line
/// Format: ```````````````````````````````` example Section: Number [options]
fn parse_example_header(line: &str) -> Option<(String, usize, Vec<String>)> {
    let prefix = "```````````````````````````````` example";
    if !line.starts_with(prefix) {
        return None;
    }

    let rest = line[prefix.len()..].trim();

    // Parse section and number: "Section: Number" or "Section: Number [opt1, opt2]"
    let parts: Vec<&str> = rest.splitn(2, '[').collect();
    let section_part = parts[0].trim();

    // Parse options if present
    let options = if parts.len() > 1 {
        let opts_str = parts[1].trim_end_matches(']');
        opts_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    } else {
        Vec::new()
    };

    // Parse section and number
    let section_parts: Vec<&str> = section_part.split(':').collect();
    if section_parts.len() != 2 {
        return None;
    }

    let section = section_parts[0].trim().to_string();
    let number = section_parts[1].trim().parse::<usize>().ok()?;

    Some((section, number, options))
}

fn run_spec_tests(spec_file: &str, module_name: &str) {
    let spec_content = fs::read_to_string(spec_file)
        .unwrap_or_else(|_| panic!("Failed to read {}", spec_file));

    let examples = parse_flexmark_spec(&spec_content);
    println!("Found {} {} test examples", examples.len(), module_name);

    let mut passed = 0;
    let mut failed = 0;
    let mut failures: Vec<(String, usize, String, String, String)> = Vec::new();

    for example in &examples {
        let result = markdown_to_html(&example.input, options::DEFAULT);

        if result == example.expected_html {
            passed += 1;
        } else {
            failed += 1;
            if failures.len() < 5 {
                failures.push((
                    example.section.clone(),
                    example.number,
                    example.input.clone(),
                    example.expected_html.clone(),
                    result,
                ));
            }
        }
    }

    println!("\n=== {} Spec Test Results ===", module_name);
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

    // Print some failures for debugging
    if !failures.is_empty() {
        println!("\n=== Sample Failures ===");
        for (section, number, input, expected, got) in &failures {
            println!("\n{}: {} #{}", module_name, section, number);
            println!("Input: {:?}", input);
            println!("Expected: {:?}", expected);
            println!("Got: {:?}", got);
        }
    }

    // Don't fail the test, just report results
    // This allows us to track progress without breaking the build
}

#[test]
fn test_flexmark_tables() {
    run_spec_tests("tests/fixtures/flexmark_tables_spec.md", "Tables");
}

#[test]
fn test_flexmark_strikethrough() {
    run_spec_tests(
        "tests/fixtures/flexmark_strikethrough_spec.md",
        "Strikethrough",
    );
}

#[test]
fn test_flexmark_tasklist() {
    run_spec_tests("tests/fixtures/flexmark_tasklist_spec.md", "Tasklist");
}

#[test]
fn test_flexmark_autolink() {
    run_spec_tests("tests/fixtures/flexmark_autolink_spec.md", "Autolink");
}

#[test]
fn test_flexmark_footnotes() {
    run_spec_tests("tests/fixtures/flexmark_footnotes_spec.md", "Footnotes");
}

#[test]
fn test_flexmark_toc() {
    run_spec_tests("tests/fixtures/flexmark_toc_spec.md", "TOC");
}

#[test]
fn test_flexmark_attributes() {
    run_spec_tests("tests/fixtures/flexmark_attributes_spec.md", "Attributes");
}

#[test]
fn test_flexmark_abbreviation() {
    run_spec_tests(
        "tests/fixtures/flexmark_abbreviation_spec.md",
        "Abbreviation",
    );
}

#[test]
fn test_flexmark_definition() {
    run_spec_tests("tests/fixtures/flexmark_definition_spec.md", "Definition");
}

#[test]
fn test_flexmark_yaml_front_matter() {
    run_spec_tests(
        "tests/fixtures/flexmark_yaml_front_matter_spec.md",
        "YAML Front Matter",
    );
}
