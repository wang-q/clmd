/// Parser for flexmark-java AST spec files
///
/// Spec files use a custom format within markdown code blocks:
/// ```markdown
/// ```````````````````````````````` example Section: Number
/// input markdown
/// .
/// expected html
/// .
/// expected ast (optional)
/// ````````````````````````````````
/// ```

#[derive(Debug, Clone)]
pub struct SpecExample {
    pub section: String,
    pub number: usize,
    pub input: String,
    pub expected_html: String,
    pub expected_ast: Option<String>,
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

/// Parse the example header line
/// Format: ```````````````````````````````` example Section: Number [options]
fn parse_example_header(line: &str) -> Option<(String, usize, Vec<String>)> {
    // Remove the backticks and "example" keyword
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

/// Generate Rust test code from parsed spec examples
pub fn generate_rust_tests(examples: &[SpecExample], module_name: &str) -> String {
    let mut output = String::new();

    output.push_str(&format!(
        "// Auto-generated tests from flexmark-java spec file\n"
    ));
    output.push_str(&format!("// Module: {}\n\n", module_name));
    output.push_str("use crate::*;\n\n");

    // Generate test function for each example
    for example in examples {
        let test_name = format!(
            "test_{}_{}_{}",
            module_name,
            sanitize_name(&example.section),
            example.number
        );

        output.push_str("#[test]\n");
        output.push_str(&format!("fn {}() {{\n", test_name));

        // Escape the input for Rust string literal
        let input_escaped = escape_rust_string(&example.input);
        let html_escaped = escape_rust_string(&example.expected_html);

        output.push_str(&format!("    let input = r#\"{}\"#;\n", input_escaped));
        output.push_str(&format!(
            "    let expected_html = r#\"{}\"#;\n",
            html_escaped
        ));

        output.push_str("    let doc = parse_document(input, options::DEFAULT);\n");
        output.push_str("    let html = render_html(&doc, options::DEFAULT);\n");
        output.push_str("    assert_eq!(html, expected_html);\n");

        output.push_str("}\n\n");
    }

    output
}

/// Sanitize a name for use in Rust identifier
fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect()
}

/// Escape a string for use in Rust raw string literal
fn escape_rust_string(s: &str) -> String {
    // For raw string literals r#"..."#, we need to handle the closing sequence
    // If the string contains "#, we need to use more # characters
    let max_hashes = s
        .chars()
        .collect::<Vec<_>>()
        .windows(2)
        .filter(|w| w[0] == '"' && w[1] == '#')
        .count();

    if max_hashes > 0 {
        // Use more # characters to avoid conflict
        let hashes = "#".repeat(max_hashes + 1);
        format!("r{hashes}\"{}\"{hashes}", s)
    } else {
        s.to_string()
    }
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
