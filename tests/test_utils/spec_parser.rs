//! Parser for flexmark-java AST spec files
//!
//! Spec files use a custom format within markdown code blocks:
//! ```text
//! ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ example Section: Number
//! input markdown
//! .
//! expected html
//! .
//! expected ast (optional)
//! ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
//! ```

/// A single test example from a spec file
#[derive(Debug, Clone)]
pub struct SpecExample {
    /// Section name in the spec file
    pub section: String,
    /// Test case number
    pub number: usize,
    /// Input markdown text
    pub input: String,
    /// Expected HTML output
    pub expected_html: String,
    /// Expected AST output (optional)
    pub expected_ast: Option<String>,
    /// Test options
    pub options: Vec<String>,
}

/// A single formatter test example from a spec file
/// Formatter specs have Markdown output instead of HTML
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FormatterSpecExample {
    /// Section name in the spec file
    pub section: String,
    /// Test case number
    pub number: usize,
    /// Input markdown text
    pub input: String,
    /// Expected Markdown output (not HTML)
    pub expected_output: String,
    /// Test options
    pub options: Vec<String>,
}

/// A single CLI test example from a spec file
/// CLI specs test command-line interface behavior
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CliSpecExample {
    /// Section name in the spec file
    pub section: String,
    /// Test case number
    pub number: usize,
    /// CLI command (e.g., "extract links", "fmt", "to html")
    pub command: String,
    /// Command-line arguments
    pub args: Vec<String>,
    /// Input text (stdin)
    pub input: String,
    /// Expected stdout output
    pub expected_output: String,
    /// Expected exit code (0 for success)
    pub expected_exit_code: i32,
}

/// A single API test example from a spec file
/// API specs test library function calls
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ApiSpecExample {
    /// Section name in the spec file
    pub section: String,
    /// Test case number
    pub number: usize,
    /// API function to call (e.g., "html", "commonmark")
    pub function: String,
    /// Input markdown text
    pub input: String,
    /// Expected output
    pub expected_output: String,
    /// Test options
    pub options: Vec<String>,
}

/// Parse a spec file content and extract all test examples
pub fn parse_spec_file(content: &str) -> Vec<SpecExample> {
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

                // Check if there's AST output (until the closing fence)
                // AST is optional - if the next line is the closing fence, there's no AST
                let mut ast_lines = Vec::new();
                let expected_ast = if i < lines.len()
                    && lines[i].starts_with("````````````````````````````````")
                {
                    // No AST section, closing fence immediately
                    None
                } else {
                    // Collect AST until the closing fence
                    while i < lines.len()
                        && !lines[i].starts_with("````````````````````````````````")
                    {
                        ast_lines.push(lines[i]);
                        i += 1;
                    }
                    if ast_lines.is_empty() {
                        None
                    } else {
                        Some(ast_lines.join("\n"))
                    }
                };

                // Skip the closing fence
                i += 1;

                let input = input_lines.join("\n");
                let expected_html = html_lines.join("\n");

                examples.push(SpecExample {
                    section,
                    number,
                    input,
                    expected_html,
                    expected_ast,
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

/// Parse a formatter spec file content and extract all test examples
/// Formatter specs have Markdown output instead of HTML
#[allow(dead_code)]
pub fn parse_formatter_spec_file(content: &str) -> Vec<FormatterSpecExample> {
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

                // Collect expected output (Markdown) until we hit a line with just "."
                // OR until we hit the closing fence
                let mut output_lines = Vec::new();
                while i < lines.len()
                    && lines[i] != "."
                    && !lines[i].starts_with("````````````````````````````````")
                {
                    output_lines.push(lines[i]);
                    i += 1;
                }

                // Skip the "." separator if present
                if i < lines.len() && lines[i] == "." {
                    i += 1;
                }

                // Skip AST section if present (until the closing fence)
                while i < lines.len()
                    && !lines[i].starts_with("````````````````````````````````")
                {
                    i += 1;
                }

                // Skip the closing fence
                i += 1;

                let input = input_lines.join("\n");
                let expected_output = output_lines.join("\n");

                examples.push(FormatterSpecExample {
                    section,
                    number,
                    input,
                    expected_output,
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
///
/// Supports formats:
/// - `example Section: Number [options]`
/// - `example(Section: Number) options(opt1, opt2)`
fn parse_example_header(line: &str) -> Option<(String, usize, Vec<String>)> {
    // Remove the backticks and "example" keyword
    let prefix = "```````````````````````````````` example";
    if !line.starts_with(prefix) {
        return None;
    }

    let rest = line[prefix.len()..].trim();

    // Check for parenthesis format: example(Section: Number) options(...)
    if rest.starts_with('(') {
        // Format: example(Section: Number) or example(Section: Number) options(...)
        let close_paren = rest.find(')')?;
        let section_part = &rest[1..close_paren];

        // Parse options if present after the closing parenthesis
        let after_paren = &rest[close_paren + 1..].trim();
        let options = if after_paren.starts_with("options(") {
            let opts_start = after_paren.find('(')? + 1;
            let opts_end = after_paren.rfind(')')?;
            let opts_str = &after_paren[opts_start..opts_end];
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

        return Some((section, number, options));
    }

    // Original format: "Section: Number" or "Section: Number [opt1, opt2]"
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

/// Parse a CLI spec file content and extract all test examples
/// CLI specs use format: cli(command: subcommand) args(arg1, arg2)
#[allow(dead_code)]
pub fn parse_cli_spec_file(content: &str) -> Vec<CliSpecExample> {
    let mut examples = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    let mut example_number = 0;

    while i < lines.len() {
        let line = lines[i];

        if line.starts_with("```````````````````````````````` cli") {
            if let Some((command, args, exit_code)) = parse_cli_header(line) {
                example_number += 1;
                i += 1;

                let mut input_lines = Vec::new();
                while i < lines.len() && lines[i] != "." {
                    input_lines.push(lines[i]);
                    i += 1;
                }

                i += 1;

                let mut output_lines = Vec::new();
                while i < lines.len()
                    && !lines[i].starts_with("````````````````````````````````")
                {
                    output_lines.push(lines[i]);
                    i += 1;
                }

                i += 1;

                let input = input_lines.join("\n");
                let expected_output = output_lines.join("\n");

                let section = command.clone();

                examples.push(CliSpecExample {
                    section,
                    number: example_number,
                    command,
                    args,
                    input,
                    expected_output,
                    expected_exit_code: exit_code,
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

/// Parse CLI header line
/// Format: cli(command: subcommand) args(arg1, arg2) exit(code)
#[allow(dead_code)]
fn parse_cli_header(line: &str) -> Option<(String, Vec<String>, i32)> {
    let prefix = "```````````````````````````````` cli";
    if !line.starts_with(prefix) {
        return None;
    }

    let rest = line[prefix.len()..].trim();

    if !rest.starts_with('(') {
        return None;
    }

    let close_paren = rest.find(')')?;
    let command_part = &rest[1..close_paren];

    let command = command_part.trim().to_string();

    let mut args = Vec::new();
    let mut exit_code = 0;

    let after_paren = rest[close_paren + 1..].trim();

    let mut remaining = after_paren;
    while !remaining.is_empty() {
        remaining = remaining.trim_start();
        if remaining.starts_with("args(") {
            let args_start = 5;
            if let Some(args_end) = remaining[args_start..].find(')') {
                let args_str = &remaining[args_start..args_start + args_end];
                args = args_str
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                remaining = &remaining[args_start + args_end + 1..];
            } else {
                break;
            }
        } else if remaining.starts_with("exit(") {
            let exit_start = 5;
            if let Some(exit_end) = remaining[exit_start..].find(')') {
                let exit_str = &remaining[exit_start..exit_start + exit_end];
                exit_code = exit_str.trim().parse().unwrap_or(0);
                remaining = &remaining[exit_start + exit_end + 1..];
            } else {
                break;
            }
        } else {
            break;
        }
    }

    Some((command, args, exit_code))
}

/// Parse an API spec file content and extract all test examples
/// API specs use format: api(function) options(...)
#[allow(dead_code)]
pub fn parse_api_spec_file(content: &str) -> Vec<ApiSpecExample> {
    let mut examples = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    let mut example_number = 0;

    while i < lines.len() {
        let line = lines[i];

        if line.starts_with("```````````````````````````````` api") {
            if let Some((function, options)) = parse_api_header(line) {
                example_number += 1;
                i += 1;

                let mut input_lines = Vec::new();
                while i < lines.len() && lines[i] != "." {
                    input_lines.push(lines[i]);
                    i += 1;
                }

                i += 1;

                let mut output_lines = Vec::new();
                while i < lines.len()
                    && !lines[i].starts_with("````````````````````````````````")
                {
                    output_lines.push(lines[i]);
                    i += 1;
                }

                i += 1;

                let input = input_lines.join("\n");
                let expected_output = output_lines.join("\n");

                let section = function.clone();

                examples.push(ApiSpecExample {
                    section,
                    number: example_number,
                    function,
                    input,
                    expected_output,
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

/// Parse API header line
/// Format: api(function) options(opt1, opt2)
#[allow(dead_code)]
fn parse_api_header(line: &str) -> Option<(String, Vec<String>)> {
    let prefix = "```````````````````````````````` api";
    if !line.starts_with(prefix) {
        return None;
    }

    let rest = line[prefix.len()..].trim();

    if !rest.starts_with('(') {
        return None;
    }

    let close_paren = rest.find(')')?;
    let function = rest[1..close_paren].trim().to_string();

    let mut options = Vec::new();
    let after_paren = rest[close_paren + 1..].trim();

    if after_paren.starts_with("options(") {
        let opts_start = 8;
        if let Some(opts_end) = after_paren[opts_start..].find(')') {
            let opts_str = &after_paren[opts_start..opts_start + opts_end];
            options = opts_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
    }

    Some((function, options))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_example() {
        let spec = r#"```````````````````````````````` example Basic: 1
Hello
.
<p>Hello</p>
.
Document[0, 5]
  Paragraph[0, 5]
    Text[0, 5] chars:[0, 5, "Hello"]
````````````````````````````````
"#;

        let examples = parse_spec_file(spec);
        assert_eq!(examples.len(), 1);
        assert_eq!(examples[0].section, "Basic");
        assert_eq!(examples[0].number, 1);
        assert_eq!(examples[0].input, "Hello");
        assert_eq!(examples[0].expected_html, "<p>Hello</p>");
        assert!(examples[0].expected_ast.is_some());
    }

    #[test]
    fn test_parse_multiple_examples() {
        let spec = r#"```````````````````````````````` example Section1: 1
Input 1
.
<p>Output 1</p>
.
Document[0, 7]
````````````````````````````````

```````````````````````````````` example Section2: 1
Input 2
.
<p>Output 2</p>
.
Document[0, 7]
````````````````````````````````
"#;

        let examples = parse_spec_file(spec);
        assert_eq!(examples.len(), 2);
        assert_eq!(examples[0].section, "Section1");
        assert_eq!(examples[1].section, "Section2");
    }

    #[test]
    fn test_parse_with_options() {
        let spec = r#"```````````````````````````````` example Test: 1 [option1, option2]
Input
.
<p>Output</p>
.
Document[0, 5]
````````````````````````````````
"#;

        let examples = parse_spec_file(spec);
        assert_eq!(examples.len(), 1);
        assert_eq!(examples[0].options, vec!["option1", "option2"]);
    }
}
