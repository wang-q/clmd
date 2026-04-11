//! Command-style tests inspired by pandoc's test framework
//!
//! This module implements a test framework that reads test cases from markdown files,
//! similar to pandoc's command tests. Each test file can contain multiple test cases
//! defined in code blocks.
//!
//! # Test File Format
//!
//! Test files use markdown code blocks with a special format:
//!
//! ```markdown
//! % clmd -f markdown -t html
//! # Hello World
//! ^D
//! <h1>Hello World</h1>
//! ```
//!
//! - Lines starting with `%` specify the command to run
//! - Input follows until `^D` (EOF marker)
//! - Expected output follows `^D`
//! - Lines starting with `2>` specify expected stderr output
//! - A line starting with `=>` specifies expected exit code (default: 0)
//!
//! # Example Test File
//!
//! ```markdown
//! # Header Tests
//!
//! ```
//! % clmd
//! # ATX Heading
//! ^D
//! <h1>ATX Heading</h1>
//! ```
//!
//! ```
//! % clmd
//! Setext Heading
//! =============
//! ^D
//! <h1>Setext Heading</h1>
//! ```
//! ```

use std::fs;
use std::path::Path;

/// A single command test case
#[derive(Debug, Clone)]
pub struct CommandTest {
    /// Test name/description
    pub name: String,
    /// Command arguments (e.g., ["-f", "markdown", "-t", "html"])
    pub args: Vec<String>,
    /// Input to provide to the command
    pub input: String,
    /// Expected stdout output
    pub expected_stdout: String,
    /// Expected stderr output (optional)
    #[allow(dead_code)]
    pub expected_stderr: Option<String>,
    /// Expected exit code (default: 0)
    #[allow(dead_code)]
    pub expected_exit_code: i32,
    /// Source file for debugging
    pub source_file: String,
    /// Line number for debugging
    pub line_number: usize,
}

/// Parse a command test file and extract all test cases
pub fn parse_command_test_file(content: &str, filename: &str) -> Vec<CommandTest> {
    let mut tests = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    let mut test_number = 0;

    while i < lines.len() {
        let line = lines[i];

        // Look for code block start (``` or ~~~)
        if line.trim_start().starts_with("```") || line.trim_start().starts_with("~~~") {
            let fence = if line.trim_start().starts_with("```") {
                "```"
            } else {
                "~~~"
            };

            // Check if this is a command test (starts with %)
            if i + 1 < lines.len() && lines[i + 1].trim_start().starts_with('%') {
                test_number += 1;

                if let Some(test) = parse_command_test(
                    &lines[i + 1..],
                    filename,
                    i + 1,
                    test_number,
                    fence,
                ) {
                    tests.push(test);
                }

                // Skip to end of code block
                i += 1;
                while i < lines.len() && !lines[i].trim_start().starts_with(fence) {
                    i += 1;
                }
            }
        }

        i += 1;
    }

    tests
}

/// Parse a single command test from lines
fn parse_command_test(
    lines: &[&str],
    filename: &str,
    start_line: usize,
    test_number: usize,
    fence: &str,
) -> Option<CommandTest> {
    if lines.is_empty() {
        return None;
    }

    // First line should be the command (starts with %)
    let first_line = lines[0];
    if !first_line.trim_start().starts_with('%') {
        return None;
    }

    // Parse command arguments
    let cmd_line = first_line.trim_start().trim_start_matches('%').trim();
    let args = parse_command_args(cmd_line);

    // Collect input until ^D
    let mut input_lines = Vec::new();
    let mut i = 1;

    while i < lines.len()
        && !lines[i].trim_start().starts_with(fence)
        && lines[i].trim() != "^D"
    {
        input_lines.push(lines[i]);
        i += 1;
    }

    // Skip ^D line
    if i < lines.len() && lines[i].trim() == "^D" {
        i += 1;
    }

    // Collect expected output
    let mut stdout_lines = Vec::new();
    let mut stderr_lines = Vec::new();
    let mut exit_code = 0;

    while i < lines.len() && !lines[i].trim_start().starts_with(fence) {
        let line = lines[i];

        // Check for exit code marker
        if line.starts_with("=>") {
            if let Some(code_str) = line[2..].trim().strip_prefix("exit") {
                exit_code = code_str.trim().parse().unwrap_or(0);
            } else {
                exit_code = line[2..].trim().parse().unwrap_or(0);
            }
        }
        // Check for stderr marker
        else if line.starts_with("2>") {
            stderr_lines.push(&line[2..]);
        }
        // Regular stdout
        else {
            stdout_lines.push(line);
        }

        i += 1;
    }

    Some(CommandTest {
        name: format!("{} # {}", filename, test_number),
        args,
        input: input_lines.join("\n"),
        expected_stdout: stdout_lines.join("\n"),
        expected_stderr: if stderr_lines.is_empty() {
            None
        } else {
            Some(stderr_lines.join("\n"))
        },
        expected_exit_code: exit_code,
        source_file: filename.to_string(),
        line_number: start_line,
    })
}

/// Parse command arguments from a command line string
fn parse_command_args(cmd_line: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut quote_char = '\0';

    for c in cmd_line.chars() {
        match c {
            '"' | '\'' if !in_quotes => {
                in_quotes = true;
                quote_char = c;
            }
            c if in_quotes && c == quote_char => {
                in_quotes = false;
                quote_char = '\0';
            }
            ' ' if !in_quotes => {
                if !current.is_empty() {
                    args.push(current.clone());
                    current.clear();
                }
            }
            _ => current.push(c),
        }
    }

    if !current.is_empty() {
        args.push(current);
    }

    // Remove "clmd" or "clmd.exe" from the beginning if present
    if !args.is_empty() && (args[0] == "clmd" || args[0].ends_with("clmd")) {
        args.remove(0);
    }

    args
}

/// Load and parse all command test files from a directory
pub fn load_command_tests<P: AsRef<Path>>(dir: P) -> Vec<CommandTest> {
    let mut all_tests = Vec::new();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "md") {
                if let Ok(content) = fs::read_to_string(&path) {
                    let filename =
                        path.file_stem().unwrap_or_default().to_string_lossy();
                    let tests = parse_command_test_file(&content, &filename);
                    all_tests.extend(tests);
                }
            }
        }
    }

    all_tests
}

/// Run a command test and return the result
#[cfg(test)]
pub fn run_command_test(test: &CommandTest) -> CommandTestResult {
    use clmd::{markdown_to_commonmark, markdown_to_html, Options};

    let mut options = Options::default();

    // Parse arguments
    let mut i = 0;
    let mut format = "html".to_string();

    while i < test.args.len() {
        match test.args[i].as_str() {
            "-f" | "--from" => {
                i += 1;
                // Input format - currently only markdown is supported
            }
            "-t" | "--to" => {
                i += 1;
                if i < test.args.len() {
                    format = test.args[i].clone();
                }
            }
            "--wrap" => {
                i += 1;
                // Wrap option
            }
            "--smart" => {
                options.parse.smart = true;
            }
            _ => {}
        }
        i += 1;
    }

    // Run conversion
    let actual = match format.as_str() {
        "commonmark" | "markdown" => markdown_to_commonmark(&test.input, &options),
        _ => markdown_to_html(&test.input, &options),
    };

    // Normalize output
    let actual = actual.trim_end_matches('\n').to_string();
    let expected = test.expected_stdout.trim_end_matches('\n').to_string();

    CommandTestResult {
        name: test.name.clone(),
        passed: actual == expected,
        input: test.input.clone(),
        expected,
        actual,
        source_file: test.source_file.clone(),
        line_number: test.line_number,
    }
}

/// Result of running a command test
#[cfg(test)]
#[derive(Debug)]
pub struct CommandTestResult {
    pub name: String,
    pub passed: bool,
    pub input: String,
    pub expected: String,
    pub actual: String,
    pub source_file: String,
    pub line_number: usize,
}

#[cfg(test)]
impl CommandTestResult {
    /// Format the result for display
    pub fn format(&self) -> String {
        if self.passed {
            format!("✓ {}", self.name)
        } else {
            format!(
                "✗ {} ({}:{})\n  Input: {:?}\n  Expected: {:?}\n  Actual: {:?}",
                self.name,
                self.source_file,
                self.line_number,
                self.input,
                self.expected,
                self.actual
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_command_args() {
        let args = parse_command_args("clmd -f markdown -t html");
        assert_eq!(args, vec!["-f", "markdown", "-t", "html"]);

        let args = parse_command_args("-t commonmark --wrap=preserve");
        assert_eq!(args, vec!["-t", "commonmark", "--wrap=preserve"]);
    }

    #[test]
    fn test_parse_simple_command_test() {
        let content = r#"
```
% clmd
# Hello
^D
<h1>Hello</h1>
```
"#;

        let tests = parse_command_test_file(content, "test.md");
        assert_eq!(tests.len(), 1);
        assert_eq!(tests[0].input, "# Hello");
        assert_eq!(tests[0].expected_stdout, "<h1>Hello</h1>");
    }

    #[test]
    fn test_parse_multiple_tests() {
        let content = r#"
```
% clmd
# Test 1
^D
<h1>Test 1</h1>
```

```
% clmd
## Test 2
^D
<h2>Test 2</h2>
```
"#;

        let tests = parse_command_test_file(content, "test.md");
        assert_eq!(tests.len(), 2);
    }

    #[test]
    fn test_parse_with_options() {
        let content = r#"
```
% clmd -t commonmark --smart
**bold**
^D
**bold**
```
"#;

        let tests = parse_command_test_file(content, "test.md");
        assert_eq!(tests.len(), 1);
        assert_eq!(tests[0].args, vec!["-t", "commonmark", "--smart"]);
    }
}
