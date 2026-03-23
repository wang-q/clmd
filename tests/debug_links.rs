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
fn debug_links_failures() {
    let spec_content = fs::read_to_string("tests/fixtures/spec.txt")
        .expect("Failed to read spec.txt");

    let tests = parse_spec_tests(&spec_content);

    println!("\n=== Links Test Failures ===\n");

    let mut failure_count = 0;
    for test in &tests {
        if test.section != "Links" {
            continue;
        }

        let result = markdown_to_html(&test.markdown, options::DEFAULT);

        if result != test.html {
            failure_count += 1;
            println!("Test #{} (Links)", test.number);
            println!("Input markdown: {:?}", test.markdown);
            println!("Expected: {:?}", test.html);
            println!("Got:      {:?}", result);
            println!("---");
        }
    }

    println!("\nTotal Links failures: {}", failure_count);
}
