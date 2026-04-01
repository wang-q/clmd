//! Shortcodes extension for emoji support
//!
//! This module implements shortcode parsing for emoji (e.g., `:thumbsup:` -> 👍).
//! Based on GitHub's gemoji: https://github.com/github/gemoji

use crate::ext::shortcodes_data::lookup_shortcode;

/// Parse a potential shortcode at the given position.
/// Returns `Some((emoji, consumed_len))` if a valid shortcode is found.
/// Returns `None` if no valid shortcode is found.
pub fn parse_shortcode(text: &str, start: usize) -> Option<(&'static str, usize)> {
    // Check if we're at the start of a potential shortcode
    if text.as_bytes().get(start) != Some(&b':') {
        return None;
    }

    // Find the closing colon
    let bytes = text.as_bytes();
    let mut end = start + 1;

    while end < bytes.len() {
        let byte = bytes[end];

        // Valid shortcode characters: alphanumeric, underscore, plus, minus
        if byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'+' || byte == b'-' {
            end += 1;
        } else if byte == b':' {
            // Found closing colon
            break;
        } else {
            // Invalid character
            return None;
        }
    }

    // Check if we found a closing colon
    if end >= bytes.len() || bytes[end] != b':' {
        return None;
    }

    // Extract the shortcode name (without colons)
    let code = &text[start + 1..end];

    // Shortcode must be at least 2 characters
    if code.len() < 2 {
        return None;
    }

    // Look up the shortcode
    lookup_shortcode(code).map(|emoji| (emoji, end - start + 1))
}

/// Check if a character can start a shortcode name
pub fn is_shortcode_start(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_' || ch == '+' || ch == '-'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_shortcode_valid() {
        // Valid shortcodes
        assert_eq!(parse_shortcode(":thumbsup:", 0), Some(("👍", 10)));
        assert_eq!(parse_shortcode(":smile:", 0), Some(("😄", 7)));
        assert_eq!(parse_shortcode(":+1:", 0), Some(("👍", 4)));
        assert_eq!(parse_shortcode(":heart:", 0), Some(("❤️", 7)));
    }

    #[test]
    fn test_parse_shortcode_with_offset() {
        let text = "Hello :thumbsup: world";
        assert_eq!(parse_shortcode(text, 6), Some(("👍", 10)));
    }

    #[test]
    fn test_parse_shortcode_invalid() {
        // Invalid shortcodes
        assert_eq!(parse_shortcode("not a shortcode", 0), None);
        assert_eq!(parse_shortcode(":", 0), None);
        assert_eq!(parse_shortcode(":a:", 0), None); // Too short
        assert_eq!(parse_shortcode(":invalid:", 0), None); // Unknown code
        assert_eq!(parse_shortcode(":no closing", 0), None);
    }

    #[test]
    fn test_parse_shortcode_special_chars() {
        // Shortcodes with special characters
        assert_eq!(parse_shortcode(":+1:", 0), Some(("👍", 4)));
        assert_eq!(parse_shortcode(":-1:", 0), Some(("👎", 4)));
        assert_eq!(parse_shortcode(":100:", 0), Some(("💯", 5)));
    }

    #[test]
    fn test_is_shortcode_start() {
        assert!(is_shortcode_start('a'));
        assert!(is_shortcode_start('Z'));
        assert!(is_shortcode_start('1'));
        assert!(is_shortcode_start('_'));
        assert!(is_shortcode_start('+'));
        assert!(is_shortcode_start('-'));
        assert!(!is_shortcode_start(':'));
        assert!(!is_shortcode_start(' '));
    }
}
