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
                let mut output_lines = Vec::new();
                while i < lines.len() && lines[i] != "." {
                    output_lines.push(lines[i]);
                    i += 1;
                }

                // Skip the "." separator
                i += 1;

                // Check if there's AST output (until the closing fence)
                // AST is optional - if the next line is the closing fence, there's no AST
                // Skip AST section if present (until the closing fence)
                // AST is optional - if the next line is the closing fence, there's no AST
                if i < lines.len()
                    && !lines[i].starts_with("````````````````````````````````")
                {
                    // Skip AST lines until the closing fence
                    while i < lines.len()
                        && !lines[i].starts_with("````````````````````````````````")
                    {
                        i += 1;
                    }
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
