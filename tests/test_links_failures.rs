use md::{markdown_to_html, options};
use std::fs;

#[derive(Debug)]
struct TestCase {
    number: usize,
    section: String,
    markdown: String,
    html: String,
}

fn parse_spec_tests(content: &str) -> Vec<TestCase> {
    let mut tests = Vec::new();
    let mut current_section = String::new();
    let mut test_number = 0;

    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        if line.starts_with("## ") {
            current_section = line[3..].trim().to_string();
        }

        if line.contains("example") && line.contains("````") {
            test_number += 1;
            i += 1;

            let mut markdown = String::new();
            while i < lines.len() && lines[i] != "." {
                if !markdown.is_empty() {
                    markdown.push('\n');
                }
                markdown.push_str(lines[i]);
                i += 1;
            }

            i += 1;

            let mut html = String::new();
            while i < lines.len() && !lines[i].contains("````") {
                if !html.is_empty() {
                    html.push('\n');
                }
                html.push_str(lines[i]);
                i += 1;
            }

            let markdown = markdown.replace('→', "\t");
            let html = html.replace('→', "\t");

            tests.push(TestCase {
                number: test_number,
                section: current_section.clone(),
                markdown,
                html,
            });
        }

        i += 1;
    }

    tests
}

#[test]
fn test_links_failures() {
    let spec_content = fs::read_to_string("tests/fixtures/spec.txt")
        .expect("Failed to read spec.txt");

    let tests = parse_spec_tests(&spec_content);

    let mut failures: Vec<(usize, String, String, String, String)> = Vec::new();

    for test in &tests {
        // Only test Links section
        if test.section != "Links" {
            continue;
        }

        let result = markdown_to_html(&test.markdown, options::DEFAULT);

        if result != test.html {
            failures.push((
                test.number,
                test.section.clone(),
                test.markdown.clone(),
                test.html.clone(),
                result,
            ));
        }
    }

    println!("\n=== Links Section Failures ===");
    println!("Total failures: {}", failures.len());

    for (num, section, markdown, expected, got) in &failures[..failures.len().min(20)] {
        println!("\n--- Test #{} ({})", num, section);
        println!("Markdown: {:?}", markdown);
        println!("Expected: {:?}", expected);
        println!("Got:      {:?}", got);
    }
}
