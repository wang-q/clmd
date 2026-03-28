//! String processing utilities for CommonMark
//!
//! This module provides various string manipulation functions used during
//! parsing and rendering of CommonMark documents.

use std::borrow::Cow;

/// Case sensitivity for label normalization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Case {
    /// Case-insensitive normalization (fold to lowercase)
    Fold,
    /// Preserve original case
    Preserve,
}

/// Normalize a link label according to CommonMark spec
///
/// This collapses internal whitespace and optionally folds case.
/// Following the CommonMark spec, this:
/// - Collapses consecutive internal whitespace to a single space
/// - Strips leading/trailing whitespace
/// - Optionally converts to lowercase
pub fn normalize_label(s: &str, case: Case) -> String {
    let mut result = String::with_capacity(s.len());
    let mut last_was_whitespace = true; // Treat leading as whitespace to strip

    for c in s.chars() {
        if c.is_whitespace() {
            if !last_was_whitespace {
                result.push(' ');
                last_was_whitespace = true;
            }
        } else {
            let c = match case {
                Case::Fold => c.to_lowercase().next().unwrap_or(c),
                Case::Preserve => c,
            };
            result.push(c);
            last_was_whitespace = false;
        }
    }

    // Remove trailing whitespace (if any)
    if result.ends_with(' ') {
        result.pop();
    }

    result
}

/// Clean a URL for use in a link
///
/// This handles:
/// - Percent-encoding special characters
/// - Resolving backslash escapes
/// - Stripping control characters
pub fn clean_url(url: &str) -> Cow<'_, str> {
    // Fast path: no special handling needed
    if url.bytes().all(|b| {
        b.is_ascii_alphanumeric()
            || matches!(
                b,
                b'/' | b':'
                    | b'.'
                    | b'-'
                    | b'_'
                    | b'~'
                    | b'?'
                    | b'#'
                    | b'['
                    | b']'
                    | b'@'
                    | b'!'
                    | b'$'
                    | b'&'
                    | b'\''
                    | b'('
                    | b')'
                    | b'*'
                    | b'+'
                    | b','
                    | b';'
                    | b'='
                    | b'%'
            )
    }) {
        return Cow::Borrowed(url);
    }

    let mut result = String::with_capacity(url.len());
    let bytes = url.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        match bytes[i] {
            b'\\'
                if i + 1 < bytes.len()
                    && matches!(
                        bytes[i + 1],
                        b'(' | b')' | b' ' | b'\t' | b'\n' | b'\r'
                    ) =>
            {
                // Backslash escape for these specific characters
                result.push(bytes[i + 1] as char);
                i += 2;
            }
            b' ' | b'\t' | b'\n' | b'\r' => {
                // Whitespace should be percent-encoded in URLs
                result.push_str("%20");
                i += 1;
            }
            b if b.is_ascii_control() => {
                // Strip control characters
                i += 1;
            }
            b => {
                result.push(b as char);
                i += 1;
            }
        }
    }

    Cow::Owned(result)
}

/// Clean a link title
///
/// Handles backslash escapes and strips control characters.
pub fn clean_title(title: &str) -> Cow<'_, str> {
    // Fast path: no special handling needed
    if !title.bytes().any(|b| b == b'\\' || b.is_ascii_control()) {
        return Cow::Borrowed(title);
    }

    let mut result = String::with_capacity(title.len());
    let bytes = title.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'\\' && i + 1 < bytes.len() {
            let next = bytes[i + 1];
            // Handle backslash escapes: convert \n to n, \t to t, etc.
            match next {
                b'n' => result.push('n'),
                b't' => result.push('t'),
                b'r' => result.push('r'),
                b'\\' | b'`' | b'*' | b'_' | b'{' | b'}' | b'[' | b']' | b'(' | b')'
                | b'#' | b'+' | b'-' | b'.' | b'!' | b'|' | b'<' | b'>' | b' '
                | b'\t' | b'\n' | b'\r' => {
                    result.push(next as char);
                }
                _ => {
                    // Not a valid escape sequence, keep the backslash
                    result.push('\\');
                    result.push(next as char);
                }
            }
            i += 2;
            continue;
        }

        if !bytes[i].is_ascii_control() {
            result.push(bytes[i] as char);
        }
        i += 1;
    }

    Cow::Owned(result)
}

/// Remove trailing blank lines from a string
///
/// This removes all trailing whitespace-only lines.
pub fn remove_trailing_blank_lines(s: &mut String) {
    while s.ends_with("\n\n") || s.ends_with("\r\n\r\n") {
        if s.ends_with("\r\n\r\n") {
            s.truncate(s.len() - 2);
        } else {
            s.pop();
        }
    }

    // Also remove trailing whitespace on the last line
    while s.ends_with('\n') || s.ends_with('\r') || s.ends_with(' ') || s.ends_with('\t')
    {
        s.pop();
    }
}

/// Unescape backslash escapes in a string (in-place)
///
/// This processes CommonMark backslash escapes.
pub fn unescape(s: &mut String) {
    let bytes = s.as_bytes();

    // Fast path: no backslashes
    if !bytes.contains(&b'\\') {
        return;
    }

    let mut result = String::with_capacity(s.len());
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'\\' && i + 1 < bytes.len() {
            let next = bytes[i + 1];
            // ASCII punctuation can be escaped
            if is_ascii_punct(next) {
                result.push(next as char);
                i += 2;
                continue;
            }
        }
        result.push(bytes[i] as char);
        i += 1;
    }

    *s = result;
}

/// Check if a byte is ASCII punctuation
fn is_ascii_punct(b: u8) -> bool {
    matches!(
        b,
        b'!' | b'"'
            | b'#'
            | b'$'
            | b'%'
            | b'&'
            | b'\''
            | b'('
            | b')'
            | b'*'
            | b'+'
            | b','
            | b'-'
            | b'.'
            | b'/'
            | b':'
            | b';'
            | b'<'
            | b'='
            | b'>'
            | b'?'
            | b'@'
            | b'['
            | b'\\'
            | b']'
            | b'^'
            | b'_'
            | b'`'
            | b'{'
            | b'|'
            | b'}'
            | b'~'
    )
}

/// Check if a byte is a space or tab
#[inline(always)]
pub fn is_space_or_tab(b: u8) -> bool {
    matches!(b, b' ' | b'\t')
}

/// Check if a byte is a line ending character
#[inline(always)]
pub fn is_line_end_char(b: u8) -> bool {
    matches!(b, b'\n' | b'\r')
}

/// Check if a string is blank (empty or only whitespace)
pub fn is_blank(s: &str) -> bool {
    s.chars().all(|c| c.is_whitespace())
}

/// Count leading spaces/tabs in a string
///
/// Returns the number of columns of indentation.
/// Tabs are counted as up to 4 spaces (modulo 4 alignment).
pub fn count_indent(s: &str) -> usize {
    let mut cols = 0;
    for c in s.chars() {
        match c {
            ' ' => cols += 1,
            '\t' => cols = (cols + 4) & !3, // Round up to next multiple of 4
            _ => break,
        }
    }
    cols
}

/// Normalize line endings to LF
///
/// Converts CRLF and CR to LF.
pub fn normalize_newlines(s: &str) -> Cow<'_, str> {
    if !s.contains('\r') {
        return Cow::Borrowed(s);
    }

    let mut result = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'\r' {
            result.push('\n');
            if i + 1 < bytes.len() && bytes[i + 1] == b'\n' {
                i += 2;
            } else {
                i += 1;
            }
        } else {
            result.push(bytes[i] as char);
            i += 1;
        }
    }

    Cow::Owned(result)
}

/// Trim leading ASCII whitespace
pub fn trim_start_ascii_whitespace(s: &str) -> &str {
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    &s[i..]
}

/// Trim trailing ASCII whitespace
pub fn trim_end_ascii_whitespace(s: &str) -> &str {
    let bytes = s.as_bytes();
    let mut i = bytes.len();
    while i > 0 && bytes[i - 1].is_ascii_whitespace() {
        i -= 1;
    }
    &s[..i]
}

/// Check if a string looks like a URL (has scheme://)
pub fn looks_like_url(s: &str) -> bool {
    let bytes = s.as_bytes();

    // Must start with a letter
    if bytes.is_empty() || !bytes[0].is_ascii_alphabetic() {
        return false;
    }

    // Find the colon
    let mut i = 1;
    while i < bytes.len()
        && (bytes[i].is_ascii_alphanumeric() || matches!(bytes[i], b'+' | b'-' | b'.'))
    {
        i += 1;
    }

    // Must have ://
    i + 2 < bytes.len()
        && bytes[i] == b':'
        && bytes[i + 1] == b'/'
        && bytes[i + 2] == b'/'
}

/// Percent-encode a string for use in URLs
pub fn percent_encode(s: &str) -> String {
    let mut result = String::with_capacity(s.len() * 3);

    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(b as char)
            }
            _ => {
                result.push('%');
                result.push_str(&format!("{:02X}", b));
            }
        }
    }

    result
}

/// Decode HTML entities in a string
///
/// Handles numeric entities (decimal and hex) and named entities.
pub fn decode_entities(s: &str) -> Cow<'_, str> {
    if !s.contains('&') {
        return Cow::Borrowed(s);
    }

    let mut result = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'&' {
            if let Some((ch, len)) = match_entity(&bytes[i..]) {
                result.push(ch);
                i += len;
                continue;
            }
        }
        result.push(bytes[i] as char);
        i += 1;
    }

    Cow::Owned(result)
}

/// Match an HTML entity at the start of a byte slice
///
/// Returns (decoded_char, length) if matched, or None.
fn match_entity(bytes: &[u8]) -> Option<(char, usize)> {
    if bytes.len() < 2 || bytes[0] != b'&' {
        return None;
    }

    // Numeric entity: &#123; or &#x7B;
    if bytes[1] == b'#' {
        if bytes.len() >= 3 && bytes[2] == b'x' || bytes[2] == b'X' {
            // Hex entity
            let mut i = 3;
            let mut value = 0u32;
            while i < bytes.len() && bytes[i].is_ascii_hexdigit() {
                value = value * 16 + hex_value(bytes[i]) as u32;
                i += 1;
            }
            if i > 3 && i < bytes.len() && bytes[i] == b';' {
                if let Some(c) = std::char::from_u32(value) {
                    return Some((c, i + 1));
                }
            }
        } else {
            // Decimal entity
            let mut i = 2;
            let mut value = 0u32;
            while i < bytes.len() && bytes[i].is_ascii_digit() {
                value = value * 10 + (bytes[i] - b'0') as u32;
                i += 1;
            }
            if i > 2 && i < bytes.len() && bytes[i] == b';' {
                if let Some(c) = std::char::from_u32(value) {
                    return Some((c, i + 1));
                }
            }
        }
    }

    // Named entity
    let mut i = 1;
    while i < bytes.len() && bytes[i].is_ascii_alphanumeric() {
        i += 1;
    }

    if i > 1 && i < bytes.len() && bytes[i] == b';' {
        let name = std::str::from_utf8(&bytes[1..i]).ok()?;
        if let Some(c) = lookup_named_entity(name) {
            return Some((c, i + 1));
        }
    }

    None
}

fn hex_value(b: u8) -> u8 {
    match b {
        b'0'..=b'9' => b - b'0',
        b'a'..=b'f' => b - b'a' + 10,
        b'A'..=b'F' => b - b'A' + 10,
        _ => 0,
    }
}

/// Lookup a named HTML entity
fn lookup_named_entity(name: &str) -> Option<char> {
    match name {
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" => Some('"'),
        "apos" => Some('\''),
        "nbsp" => Some('\u{00A0}'),
        "copy" => Some('\u{00A9}'),
        "reg" => Some('\u{00AE}'),
        "trade" => Some('\u{2122}'),
        "mdash" => Some('\u{2014}'),
        "ndash" => Some('\u{2013}'),
        "hellip" => Some('\u{2026}'),
        "laquo" => Some('\u{00AB}'),
        "raquo" => Some('\u{00BB}'),
        "ldquo" => Some('\u{201C}'),
        "rdquo" => Some('\u{201D}'),
        "lsquo" => Some('\u{2018}'),
        "rsquo" => Some('\u{2019}'),
        _ => None,
    }
}

/// Escape special HTML characters
pub fn escape_html(s: &str) -> String {
    let mut result = String::with_capacity(s.len());

    for c in s.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            _ => result.push(c),
        }
    }

    result
}

/// Check if a string contains only ASCII characters
pub fn is_ascii(s: &str) -> bool {
    s.bytes().all(|b| b.is_ascii())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_label() {
        assert_eq!(
            normalize_label("  Hello  World  ", Case::Fold),
            "hello world"
        );
        assert_eq!(normalize_label("Hello\t\tWorld", Case::Fold), "hello world");
        assert_eq!(normalize_label("HELLO", Case::Fold), "hello");
        assert_eq!(normalize_label("HELLO", Case::Preserve), "HELLO");
    }

    #[test]
    fn test_clean_url() {
        assert_eq!(clean_url("http://example.com"), "http://example.com");
        assert_eq!(clean_url("hello world"), "hello%20world");
        assert_eq!(clean_url("foo\\(bar)"), "foo(bar)");
    }

    #[test]
    fn test_clean_title() {
        assert_eq!(clean_title("hello"), "hello");
        // Input "hello\nworld" (backslash + n) should become "hellonworld"
        let input = "hello\\nworld";
        let result = clean_title(input);
        println!("Input bytes: {:?}", input.as_bytes());
        println!("Result bytes: {:?}", result.as_bytes());
        assert_eq!(result, "hellonworld");
        assert_eq!(clean_title("hello\\*world"), "hello*world");
    }

    #[test]
    fn test_unescape() {
        let mut s = "hello\\*world".to_string();
        unescape(&mut s);
        assert_eq!(s, "hello*world");
    }

    #[test]
    fn test_is_blank() {
        assert!(is_blank(""));
        assert!(is_blank("   "));
        assert!(is_blank("\t\n"));
        assert!(!is_blank("hello"));
    }

    #[test]
    fn test_count_indent() {
        assert_eq!(count_indent("hello"), 0);
        assert_eq!(count_indent("  hello"), 2);
        assert_eq!(count_indent("\thello"), 4);
        assert_eq!(count_indent("  \thello"), 4);
    }

    #[test]
    fn test_normalize_newlines() {
        assert_eq!(normalize_newlines("hello\r\nworld"), "hello\nworld");
        assert_eq!(normalize_newlines("hello\rworld"), "hello\nworld");
        assert_eq!(normalize_newlines("hello\nworld"), "hello\nworld");
    }

    #[test]
    fn test_decode_entities() {
        assert_eq!(decode_entities("&amp;"), "&");
        assert_eq!(decode_entities("&lt;"), "<");
        assert_eq!(decode_entities("&gt;"), ">");
        assert_eq!(decode_entities("&#65;"), "A");
        assert_eq!(decode_entities("&#x41;"), "A");
    }

    #[test]
    fn test_escape_html() {
        assert_eq!(escape_html("<script>"), "&lt;script&gt;");
        assert_eq!(escape_html("&"), "&amp;");
        assert_eq!(escape_html("\""), "&quot;");
    }

    #[test]
    fn test_looks_like_url() {
        assert!(looks_like_url("http://example.com"));
        assert!(looks_like_url("https://example.com"));
        assert!(looks_like_url("ftp://example.com"));
        assert!(!looks_like_url("example.com"));
        assert!(!looks_like_url("/path/to/file"));
    }
}
