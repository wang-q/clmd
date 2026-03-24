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
fn debug_entity_and_numeric_character_references() {
    let spec_content =
        fs::read_to_string("tests/fixtures/spec.txt").expect("Failed to read spec.txt");
    let tests = parse_spec_tests(&spec_content);

    println!("\n=== Entity and numeric character references Failures ===\n");

    for test in &tests {
        if test.section != "Entity and numeric character references" {
            continue;
        }

        let result = markdown_to_html(&test.markdown, options::DEFAULT);

        if result != test.html {
            println!("Test #{} ({})", test.number, test.section);
            println!("Input markdown: {:?}", test.markdown);
            println!("Expected: {:?}", test.html);
            println!("Got:      {:?}", result);
            println!("---");
        }
    }
}

#[test]
fn debug_setext_headings() {
    let spec_content =
        fs::read_to_string("tests/fixtures/spec.txt").expect("Failed to read spec.txt");
    let tests = parse_spec_tests(&spec_content);

    println!("\n=== Setext headings Failures ===\n");

    for test in &tests {
        if test.section != "Setext headings" {
            continue;
        }

        let result = markdown_to_html(&test.markdown, options::DEFAULT);

        if result != test.html {
            println!("Test #{} ({})", test.number, test.section);
            println!("Input markdown: {:?}", test.markdown);
            println!("Expected: {:?}", test.html);
            println!("Got:      {:?}", result);
            println!("---");
        }
    }
}

#[test]
fn debug_atx_headings() {
    let spec_content =
        fs::read_to_string("tests/fixtures/spec.txt").expect("Failed to read spec.txt");
    let tests = parse_spec_tests(&spec_content);

    println!("\n=== ATX headings Failures ===\n");

    for test in &tests {
        if test.section != "ATX headings" {
            continue;
        }

        let result = markdown_to_html(&test.markdown, options::DEFAULT);

        if result != test.html {
            println!("Test #{} ({})", test.number, test.section);
            println!("Input markdown: {:?}", test.markdown);
            println!("Expected: {:?}", test.html);
            println!("Got:      {:?}", result);
            println!("---");
        }
    }
}

#[test]
fn debug_fenced_code_blocks() {
    let spec_content =
        fs::read_to_string("tests/fixtures/spec.txt").expect("Failed to read spec.txt");
    let tests = parse_spec_tests(&spec_content);

    println!("\n=== Fenced code blocks Failures ===\n");

    for test in &tests {
        if test.section != "Fenced code blocks" {
            continue;
        }

        let result = markdown_to_html(&test.markdown, options::DEFAULT);

        if result != test.html {
            println!("Test #{} ({})", test.number, test.section);
            println!("Input markdown: {:?}", test.markdown);
            println!("Expected: {:?}", test.html);
            println!("Got:      {:?}", result);
            println!("---");
        }
    }
}

#[test]
fn debug_backslash_escapes() {
    let spec_content =
        fs::read_to_string("tests/fixtures/spec.txt").expect("Failed to read spec.txt");
    let tests = parse_spec_tests(&spec_content);

    println!("\n=== Backslash escapes Failures ===\n");

    for test in &tests {
        if test.section != "Backslash escapes" {
            continue;
        }

        let result = markdown_to_html(&test.markdown, options::DEFAULT);

        if result != test.html {
            println!("Test #{} ({})", test.number, test.section);
            println!("Input markdown: {:?}", test.markdown);
            println!("Expected: {:?}", test.html);
            println!("Got:      {:?}", result);
            println!("---");
        }
    }
}

#[test]
fn debug_block_quotes() {
    let spec_content =
        fs::read_to_string("tests/fixtures/spec.txt").expect("Failed to read spec.txt");
    let tests = parse_spec_tests(&spec_content);

    println!("\n=== Block quotes Failures ===\n");

    for test in &tests {
        if test.section != "Block quotes" {
            continue;
        }

        let result = markdown_to_html(&test.markdown, options::DEFAULT);

        if result != test.html {
            println!("Test #{} ({})", test.number, test.section);
            println!("Input markdown: {:?}", test.markdown);
            println!("Expected: {:?}", test.html);
            println!("Got:      {:?}", result);
            println!("---");
        }
    }
}

#[test]
fn debug_code_spans() {
    let spec_content =
        fs::read_to_string("tests/fixtures/spec.txt").expect("Failed to read spec.txt");
    let tests = parse_spec_tests(&spec_content);

    println!("\n=== Code spans Failures ===\n");

    for test in &tests {
        if test.section != "Code spans" {
            continue;
        }

        let result = markdown_to_html(&test.markdown, options::DEFAULT);

        if result != test.html {
            println!("Test #{} ({})", test.number, test.section);
            println!("Input markdown: {:?}", test.markdown);
            println!("Expected: {:?}", test.html);
            println!("Got:      {:?}", result);
            println!("---");
        }
    }
}
