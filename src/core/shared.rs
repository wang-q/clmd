//! Shared utility functions for document processing.
//!
//! This module provides common utility functions for working with Markdown documents,
//! inspired by Pandoc's Shared module.
//!
//! # Example
//!
//! ```ignore
//! use clmd::shared::stringify;
//!
//! let text = stringify("Hello **world**!");
//! assert_eq!(text, "Hello world!");
//! ```

/// Convert a string to a plain text representation.
///
/// This function removes Markdown formatting and returns plain text.
///
/// # Arguments
///
/// * `input` - The input string with Markdown formatting
///
/// # Returns
///
/// The plain text without Markdown formatting
///
/// # Example
///
/// ```ignore
/// use clmd::shared::stringify;
///
/// let text = stringify("Hello **world**!");
/// assert_eq!(text, "Hello world!");
/// ```ignore
pub fn stringify(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut in_code = false;
    let mut in_link = false;
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '`' => {
                // Toggle code mode
                in_code = !in_code;
            }
            '[' if !in_code => {
                // Start of link text
                in_link = true;
            }
            ']' if in_link => {
                // End of link text
                in_link = false;
                // Skip the URL part if present
                if chars.peek() == Some(&'(') {
                    chars.next(); // consume '('
                                  // Safely skip until we find ')' or reach the end
                    for c in chars.by_ref() {
                        if c == ')' {
                            break;
                        }
                    }
                    // If we didn't find the closing ')', the link syntax is incomplete
                    // We don't need to do anything special here as we've already consumed '['
                    // and the link text was already added to result
                }
            }
            '*' | '_' => {
                // Skip emphasis markers
                if !in_code {
                    continue;
                }
                result.push(c);
            }
            '\\' => {
                // Handle escape sequences
                if let Some(next) = chars.next() {
                    result.push(next);
                }
            }
            _ => {
                result.push(c);
            }
        }
    }

    result
}

/// Normalize whitespace in a string.
///
/// This function collapses consecutive whitespace characters into a single space
/// and trims leading and trailing whitespace.
///
/// # Arguments
///
/// * `input` - The input string
///
/// # Returns
///
/// The normalized string
///
/// # Example
///
/// ```ignore
/// use clmd::shared::normalize_whitespace;
///
/// let text = normalize_whitespace("  Hello   world  ");
/// assert_eq!(text, "Hello world");
/// ```ignore
pub fn normalize_whitespace(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut prev_was_whitespace = true; // Start true to trim leading whitespace

    for c in input.chars() {
        if c.is_whitespace() {
            if !prev_was_whitespace {
                result.push(' ');
                prev_was_whitespace = true;
            }
        } else {
            result.push(c);
            prev_was_whitespace = false;
        }
    }

    // Trim trailing whitespace
    if result.ends_with(' ') {
        result.pop();
    }

    result
}

/// Trim leading and trailing blank lines from a string.
///
/// # Arguments
///
/// * `input` - The input string
///
/// # Returns
///
/// The trimmed string
pub fn trim_blank_lines(input: &str) -> &str {
    let mut result = input;

    // Trim leading blank lines
    loop {
        if result.is_empty() {
            return "";
        }
        if result.starts_with('\n') {
            result = &result[1..];
        } else if result.starts_with("\r\n") {
            result = &result[2..];
        } else if result.trim().is_empty() {
            // Only whitespace remains
            return "";
        } else {
            break;
        }
    }

    // Trim trailing blank lines
    loop {
        if result.is_empty() {
            return "";
        }
        if result.ends_with('\n') {
            // Check if the line before the newline is blank
            if let Some(pos) = result[..result.len() - 1].rfind('\n') {
                if result[pos + 1..result.len() - 1].trim().is_empty() {
                    result = &result[..pos + 1];
                    continue;
                }
            } else if result[..result.len() - 1].trim().is_empty() {
                // The only line is blank
                return "";
            }
            break;
        } else if result.ends_with("\r\n") {
            // Check if the line before the newline is blank
            if let Some(pos) = result[..result.len() - 2].rfind('\n') {
                if result[pos + 1..result.len() - 2].trim().is_empty() {
                    result = &result[..pos + 1];
                    continue;
                }
            } else if result[..result.len() - 2].trim().is_empty() {
                // The only line is blank
                return "";
            }
            break;
        } else {
            break;
        }
    }

    // Remove trailing newline from the result
    if result.ends_with('\n') {
        result = &result[..result.len() - 1];
    } else if result.ends_with("\r\n") {
        result = &result[..result.len() - 2];
    }

    result
}

/// Split a string into lines and preserve line endings.
///
/// # Arguments
///
/// * `input` - The input string
///
/// # Returns
///
/// A vector of tuples containing (line content, line ending)
pub fn split_lines(input: &str) -> Vec<(&str, &str)> {
    let mut result = Vec::new();
    let mut remaining = input;

    while !remaining.is_empty() {
        if let Some(pos) = remaining.find(['\n', '\r']) {
            let line = &remaining[..pos];
            let mut ending = &remaining[pos..pos + 1];

            // Check for \r\n
            if ending == "\r"
                && remaining.len() > pos + 1
                && remaining.as_bytes()[pos + 1] == b'\n'
            {
                ending = &remaining[pos..pos + 2];
            }

            result.push((line, ending));
            remaining = &remaining[pos + ending.len()..];
        } else {
            result.push((remaining, ""));
            break;
        }
    }

    result
}

/// Indent each line of a string.
///
/// # Arguments
///
/// * `input` - The input string
/// * `indent` - The indentation string to add
///
/// # Returns
///
/// The indented string
///
/// # Example
///
/// ```ignore
/// use clmd::shared::indent;
///
/// let text = indent("line1\nline2", "  ");
/// assert_eq!(text, "  line1\n  line2");
/// ```ignore
pub fn indent(input: &str, indent: &str) -> String {
    let mut result =
        String::with_capacity(input.len() + indent.len() * input.lines().count());

    for (i, line) in input.lines().enumerate() {
        if i > 0 {
            result.push('\n');
        }
        result.push_str(indent);
        result.push_str(line);
    }

    result
}

/// Escape special characters in a string for use in regular expressions.
///
/// # Arguments
///
/// * `input` - The input string
///
/// # Returns
///
/// The escaped string
pub fn escape_regex(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '\\' | '.' | '+' | '*' | '?' | '(' | ')' | '|' | '[' | ']' | '{' | '}'
            | '^' | '$' | '#' | '&' | '-' => {
                result.push('\\');
                result.push(c);
            }
            _ => result.push(c),
        }
    }
    result
}

/// Check if a string is a valid URL.
///
/// # Arguments
///
/// * `s` - The string to check
///
/// # Returns
///
/// `true` if the string looks like a URL
pub fn is_url(s: &str) -> bool {
    s.starts_with("http://")
        || s.starts_with("https://")
        || s.starts_with("ftp://")
        || s.starts_with("mailto:")
        || s.starts_with("file://")
}

/// Extract the file extension from a path.
///
/// # Arguments
///
/// * `path` - The file path
///
/// # Returns
///
/// The file extension if present
pub fn get_extension(path: &str) -> Option<&str> {
    path.rfind('.').map(|i| &path[i + 1..])
}

/// Convert a string to kebab-case.
///
/// # Arguments
///
/// * `input` - The input string
///
/// # Returns
///
/// The kebab-case string
///
/// # Example
///
/// ```ignore
/// use clmd::shared::to_kebab_case;
///
/// assert_eq!(to_kebab_case("Hello World"), "hello-world");
/// assert_eq!(to_kebab_case("HTTPResponse"), "http-response");
/// ```ignore
pub fn to_kebab_case(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut prev_was_separator = false;
    let mut prev_was_lower = false;

    for (i, c) in input.chars().enumerate() {
        if c.is_uppercase() {
            // Add separator if:
            // 1. Not the first character
            // 2. Previous was not a separator
            // 3. Previous was lowercase (e.g., "fooBar") OR
            //    Next is lowercase (e.g., "HTTPResponse" -> "http-response")
            if i > 0 && !prev_was_separator {
                let next_is_lower = input
                    .chars()
                    .nth(i + 1)
                    .map(|n| n.is_lowercase())
                    .unwrap_or(false);
                if prev_was_lower || next_is_lower {
                    result.push('-');
                }
            }
            result.push(c.to_ascii_lowercase());
            prev_was_separator = false;
            prev_was_lower = false;
        } else if c.is_whitespace() || c == '_' {
            if !prev_was_separator {
                result.push('-');
            }
            prev_was_separator = true;
            prev_was_lower = false;
        } else {
            result.push(c);
            prev_was_separator = false;
            prev_was_lower = true;
        }
    }

    result
}

/// Convert a string to snake_case.
///
/// # Arguments
///
/// * `input` - The input string
///
/// # Returns
///
/// The snake_case string
///
/// # Example
///
/// ```ignore
/// use clmd::shared::to_snake_case;
///
/// assert_eq!(to_snake_case("Hello World"), "hello_world");
/// assert_eq!(to_snake_case("HTTPResponse"), "http_response");
/// ```ignore
pub fn to_snake_case(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut prev_was_separator = false;
    let mut prev_was_lower = false;

    for (i, c) in input.chars().enumerate() {
        if c.is_uppercase() {
            // Add separator if:
            // 1. Not the first character
            // 2. Previous was not a separator
            // 3. Previous was lowercase (e.g., "fooBar") OR
            //    Next is lowercase (e.g., "HTTPResponse" -> "http_response")
            if i > 0 && !prev_was_separator {
                let next_is_lower = input
                    .chars()
                    .nth(i + 1)
                    .map(|n| n.is_lowercase())
                    .unwrap_or(false);
                if prev_was_lower || next_is_lower {
                    result.push('_');
                }
            }
            result.push(c.to_ascii_lowercase());
            prev_was_separator = false;
            prev_was_lower = false;
        } else if c.is_whitespace() || c == '-' {
            if !prev_was_separator {
                result.push('_');
            }
            prev_was_separator = true;
            prev_was_lower = false;
        } else {
            result.push(c);
            prev_was_separator = false;
            prev_was_lower = true;
        }
    }

    result
}

/// Truncate a string to a maximum length, adding an ellipsis if truncated.
///
/// # Arguments
///
/// * `input` - The input string
/// * `max_len` - The maximum length
///
/// # Returns
///
/// The truncated string
///
/// # Example
///
/// ```ignore
/// use clmd::shared::truncate_with_ellipsis;
///
/// assert_eq!(truncate_with_ellipsis("Hello World", 8), "Hello...");
/// assert_eq!(truncate_with_ellipsis("Hi", 8), "Hi");
/// ```ignore
pub fn truncate_with_ellipsis(input: &str, max_len: usize) -> String {
    if input.len() <= max_len {
        return input.to_string();
    }

    let ellipsis = "...";
    let truncate_len = max_len.saturating_sub(ellipsis.len());

    if truncate_len == 0 {
        return ellipsis.to_string();
    }

    let mut result = input.chars().take(truncate_len).collect::<String>();
    result.push_str(ellipsis);
    result
}

/// Format a byte size as a human-readable string.
///
/// # Arguments
///
/// * `size` - The size in bytes
///
/// # Returns
///
/// A human-readable string like "1.5 MB"
///
/// # Example
///
/// ```ignore
/// use clmd::shared::format_size;
///
/// assert_eq!(format_size(1024), "1.0 KB");
/// assert_eq!(format_size(1536), "1.5 KB");
/// assert_eq!(format_size(1024 * 1024), "1.0 MB");
/// ```ignore
pub fn format_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];

    if size == 0 {
        return "0 B".to_string();
    }

    let exp = (size as f64).log(1024.0).min(UNITS.len() as f64 - 1.0) as usize;
    let value = size as f64 / 1024f64.powi(exp as i32);

    if exp == 0 {
        format!("{} {}", size, UNITS[0])
    } else {
        format!("{:.1} {}", value, UNITS[exp])
    }
}

/// Join strings with a separator, skipping empty strings.
///
/// # Arguments
///
/// * `items` - The items to join
/// * `separator` - The separator string
///
/// # Returns
///
/// The joined string
///
/// # Example
///
/// ```ignore
/// use clmd::shared::join_non_empty;
///
/// let items = vec!["a", "", "b", "", "c"];
/// assert_eq!(join_non_empty(&items, ", "), "a, b, c");
/// ```ignore
pub fn join_non_empty(items: &[&str], separator: &str) -> String {
    let non_empty: Vec<&str> = items.iter().copied().filter(|s| !s.is_empty()).collect();
    non_empty.join(separator)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stringify() {
        assert_eq!(stringify("Hello **world**!"), "Hello world!");
        assert_eq!(stringify("`code`"), "code");
        assert_eq!(stringify("[link](url)"), "link");
        assert_eq!(stringify("*emphasis*"), "emphasis");
    }

    #[test]
    fn test_normalize_whitespace() {
        assert_eq!(normalize_whitespace("  Hello   world  "), "Hello world");
        assert_eq!(normalize_whitespace("a\t\tb"), "a b");
        assert_eq!(normalize_whitespace("  "), "");
    }

    #[test]
    fn test_trim_blank_lines() {
        assert_eq!(trim_blank_lines("\n\nHello\n\n"), "Hello");
        assert_eq!(trim_blank_lines("Hello\nWorld"), "Hello\nWorld");
        assert_eq!(trim_blank_lines("\n\n"), "");
    }

    #[test]
    fn test_indent() {
        assert_eq!(indent("line1\nline2", "  "), "  line1\n  line2");
        assert_eq!(indent("line1", "\t"), "\tline1");
    }

    #[test]
    fn test_is_url() {
        assert!(is_url("http://example.com"));
        assert!(is_url("https://example.com"));
        assert!(is_url("mailto:test@example.com"));
        assert!(!is_url("example.com"));
    }

    #[test]
    fn test_get_extension() {
        assert_eq!(get_extension("file.txt"), Some("txt"));
        assert_eq!(get_extension("path/to/file.md"), Some("md"));
        assert_eq!(get_extension("no_extension"), None);
    }

    #[test]
    fn test_to_kebab_case() {
        assert_eq!(to_kebab_case("Hello World"), "hello-world");
        assert_eq!(to_kebab_case("HTTPResponse"), "http-response");
        assert_eq!(to_kebab_case("snake_case"), "snake-case");
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("Hello World"), "hello_world");
        assert_eq!(to_snake_case("HTTPResponse"), "http_response");
        assert_eq!(to_snake_case("kebab-case"), "kebab_case");
    }

    #[test]
    fn test_truncate_with_ellipsis() {
        assert_eq!(truncate_with_ellipsis("Hello World", 8), "Hello...");
        assert_eq!(truncate_with_ellipsis("Hi", 8), "Hi");
        assert_eq!(truncate_with_ellipsis("Hello", 3), "...");
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1536), "1.5 KB");
        assert_eq!(format_size(1024 * 1024), "1.0 MB");
    }

    #[test]
    fn test_join_non_empty() {
        let items = vec!["a", "", "b", "", "c"];
        assert_eq!(join_non_empty(&items, ", "), "a, b, c");

        let empty: Vec<&str> = vec![];
        assert_eq!(join_non_empty(&empty, ", "), "");
    }
}
