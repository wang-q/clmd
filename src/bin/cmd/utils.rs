use std::fs;
use std::io::{self, Read, Write};

use regex::Regex;

/// Read input from file or stdin
pub fn read_input(input_path: Option<&str>) -> anyhow::Result<String> {
    let content = if let Some(path) = input_path {
        fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Failed to read file '{}': {}", path, e))?
    } else {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .map_err(|e| anyhow::anyhow!("Failed to read stdin: {}", e))?;
        buffer
    };

    // Remove UTF-8 BOM if present
    let content = content.strip_prefix('\u{FEFF}').unwrap_or(&content);
    Ok(content.to_string())
}

/// Write output to file or stdout
pub fn write_output(output_path: Option<&str>, content: &str) -> anyhow::Result<()> {
    if let Some(path) = output_path {
        if path == "-" {
            io::stdout()
                .write_all(content.as_bytes())
                .map_err(|e| anyhow::anyhow!("Failed to write stdout: {}", e))
        } else {
            fs::write(path, content)
                .map_err(|e| anyhow::anyhow!("Failed to write file '{}': {}", path, e))
        }
    } else {
        io::stdout()
            .write_all(content.as_bytes())
            .map_err(|e| anyhow::anyhow!("Failed to write stdout: {}", e))
    }
}

/// Count words in text
pub fn count_words(text: &str) -> usize {
    text.split_whitespace().count()
}

/// Count characters (excluding whitespace)
pub fn count_chars(text: &str) -> usize {
    text.chars().filter(|c| !c.is_whitespace()).count()
}

/// Generate slug from heading text for anchor links
pub fn slugify(text: &str) -> String {
    text.to_lowercase()
        .replace(|c: char| !c.is_alphanumeric() && c != ' ', "")
        .replace(' ', "-")
}

/// TOC boundary information
pub struct TocBoundaries {
    pub marker_line: String,
    pub content_end: usize,
}

/// Find TOC marker and content boundaries
pub fn find_toc_boundaries(input: &str) -> Option<TocBoundaries> {
    let re = Regex::new(r"^\[TOC\s*[^\]]*\]\s*:\s*#(?:\s+.*)?$").unwrap();

    let mut search_start = 0;
    while search_start < input.len() {
        let remaining = &input[search_start..];
        if let Some(newline_pos) = remaining.find('\n') {
            let line_end = search_start + newline_pos;
            let line = &input[search_start..line_end];
            let trimmed = line.trim();

            if re.is_match(trimmed) {
                let marker_end = line_end;

                let rest = &input[marker_end..];
                let content_start = marker_end
                    + rest
                        .find(|c: char| c != '\n' && c != '\r')
                        .unwrap_or(rest.len());

                let content_rest = &input[content_start..];
                let mut content_end = content_start;

                for line in content_rest.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with('-')
                        || trimmed
                            .chars()
                            .next()
                            .map(|c| c.is_ascii_digit())
                            .unwrap_or(false)
                    {
                        content_end += line.len() + 1;
                    } else if trimmed.is_empty() {
                        break;
                    } else {
                        break;
                    }
                }

                return Some(TocBoundaries {
                    marker_line: line.to_string(),
                    content_end,
                });
            }

            search_start = line_end + 1;
        } else {
            let line = remaining;
            let trimmed = line.trim();

            if re.is_match(trimmed) {
                let marker_end = input.len();

                return Some(TocBoundaries {
                    marker_line: line.to_string(),
                    content_end: marker_end,
                });
            }
            break;
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("Hello, World!"), "hello-world");
        assert_eq!(slugify("Test 123"), "test-123");
    }

    #[test]
    fn test_find_toc_boundaries_simple() {
        let input = "# Title\n\n[TOC]: #\n\n## Section\n\nContent";
        let boundaries = find_toc_boundaries(input).unwrap();
        assert_eq!(boundaries.marker_line, "[TOC]: #");
    }

    #[test]
    fn test_find_toc_boundaries_with_content() {
        let input =
            "# Title\n\n[TOC]: #\n\n- [Section](#section)\n\n## Section\n\nContent";
        let boundaries = find_toc_boundaries(input).unwrap();
        assert!(boundaries.content_end > 0);
    }

    #[test]
    fn test_find_toc_boundaries_with_options() {
        let input = "[TOC levels=2-4 numbered]: #\n\n- Item";
        let boundaries = find_toc_boundaries(input).unwrap();
        assert!(boundaries.marker_line.contains("levels=2-4"));
        assert!(boundaries.marker_line.contains("numbered"));
    }

    #[test]
    fn test_find_toc_boundaries_none() {
        let input = "# Title\n\n## Section\n\nContent";
        assert!(find_toc_boundaries(input).is_none());
    }

    #[test]
    fn test_read_write_roundtrip() {
        let content = "Hello, world!";
        let path = std::env::temp_dir().join("clmd_test.txt");

        write_output(Some(path.to_str().unwrap()), content).unwrap();
        let result = read_input(Some(path.to_str().unwrap())).unwrap();
        assert_eq!(result, content);

        std::fs::remove_file(path).ok();
    }
}
