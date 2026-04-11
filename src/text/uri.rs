//! URI handling utilities for clmd.
//!
//! This module provides URI normalization utilities.

/// Normalize a URI by percent-encoding special characters.
///
/// Based on commonmark.js normalizeURI.
/// Percent-encode characters that are not allowed in URIs.
pub fn normalize_uri(uri: &str) -> String {
    let mut result = String::new();

    for c in uri.chars() {
        match c {
            // Unreserved characters (no encoding needed)
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => {
                result.push(c);
            }
            // Reserved characters that are commonly used in URIs
            ':' | '/' | '?' | '#' | '@' | '!' | '$' | '&' | '\'' | '(' | ')' | '*'
            | '+' | ',' | ';' | '=' => {
                result.push(c);
            }
            // Percent sign (already encoded)
            '%' => {
                result.push(c);
            }
            // Space should be encoded as %20 (not +)
            ' ' => {
                result.push_str("%20");
            }
            // Backslash should be encoded
            '\\' => {
                result.push_str("%5C");
            }
            // Square brackets should be encoded in URLs
            '[' => {
                result.push_str("%5B");
            }
            ']' => {
                result.push_str("%5D");
            }
            // Other characters: percent-encode
            _ => {
                let mut buf = [0; 4];
                let s = c.encode_utf8(&mut buf);
                for b in s.bytes() {
                    result.push_str(&format!("%{:02X}", b));
                }
            }
        }
    }

    result
}

/// Parse a data URI and extract the MIME type and data.
///
/// Data URIs have the format: `data:[<mediatype>][;base64],<data>`
pub fn parse_data_uri(uri: &str) -> Option<(&str, &str)> {
    const DATA_PREFIX: &str = "data:";

    if !uri.starts_with(DATA_PREFIX) {
        return None;
    }

    let rest = &uri[DATA_PREFIX.len()..];

    // Find the comma that separates metadata from data
    let comma_pos = rest.find(',')?;

    let metadata = &rest[..comma_pos];
    let data = &rest[comma_pos + 1..];

    // Parse metadata to extract MIME type
    let mime_type = if metadata.is_empty() {
        "text/plain"
    } else if let Some(semi_pos) = metadata.find(';') {
        &metadata[..semi_pos]
    } else {
        metadata
    };

    if data.is_empty() {
        return None;
    }

    Some((mime_type, data))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_uri() {
        assert_eq!(normalize_uri("hello world"), "hello%20world");
        assert_eq!(normalize_uri("test.txt"), "test.txt");
        assert_eq!(normalize_uri("a+b"), "a+b");
        assert_eq!(normalize_uri("foo%bar"), "foo%bar");
        assert_eq!(normalize_uri("path\\to\\file"), "path%5Cto%5Cfile");
        assert_eq!(normalize_uri("[test]"), "%5Btest%5D");
        assert_eq!(normalize_uri("café"), "caf%C3%A9");
    }

    #[test]
    fn test_parse_data_uri() {
        assert_eq!(
            parse_data_uri("data:text/plain;base64,SGVsbG8="),
            Some(("text/plain", "SGVsbG8="))
        );
        assert_eq!(
            parse_data_uri("data:image/png,abc123"),
            Some(("image/png", "abc123"))
        );
        assert_eq!(parse_data_uri("data:,hello"), Some(("text/plain", "hello")));
        assert_eq!(parse_data_uri("not a data uri"), None);
        assert_eq!(parse_data_uri("data:"), None);
    }
}
