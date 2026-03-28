use clmd::markdown_to_html;
use clmd::parser::options::Options;
use std::fs;

fn md_to_html(input: &str) -> String {
    markdown_to_html(input, &Options::default())
}

fn main() {
    let spec_content = fs::read_to_string("tests/fixtures/spec.txt").expect("Failed to read spec.txt");
    
    let mut test_number = 0;
    let lines: Vec<&str> = spec_content.lines().collect();
    let mut i = 0;
    
    let mut passed = 0;
    let mut failed = 0;

    while i < lines.len() {
        let line = lines[i];

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

            let result = md_to_html(&markdown);
            
            if result == html {
                passed += 1;
            } else {
                failed += 1;
                println!("\n=== Test #{} ===", test_number);
                println!("Input: {:?}", markdown);
                println!("Expected: {:?}", html);
                println!("Got: {:?}", result);
            }
        }

        i += 1;
    }
    
    println!("\n=== Summary ===");
    println!("Passed: {}", passed);
    println!("Failed: {}", failed);
    println!("Total: {}", test_number);
    println!("Pass rate: {:.1}%", (passed as f64 / test_number as f64) * 100.0);
}
