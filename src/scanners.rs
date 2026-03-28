//! Scanners for CommonMark syntax elements
//!
//! This module provides functions to scan and recognize various CommonMark
//! syntax constructs at the beginning of lines or within text.

use std::ops::Range;

/// Character type utilities
pub mod ctype {
    /// Check if byte is an ASCII whitespace character
    #[inline(always)]
    pub fn isspace(b: u8) -> bool {
        matches!(b, b' ' | b'\t' | b'\n' | b'\r' | b'\x0c')
    }

    /// Check if byte is an ASCII digit
    #[inline(always)]
    pub fn isdigit(b: u8) -> bool {
        b.is_ascii_digit()
    }

    /// Check if byte is an ASCII alphabetic character
    #[inline(always)]
    pub fn isalpha(b: u8) -> bool {
        b.is_ascii_alphabetic()
    }

    /// Check if byte is an ASCII alphanumeric character
    #[inline(always)]
    pub fn isalnum(b: u8) -> bool {
        b.is_ascii_alphanumeric()
    }

    /// Check if byte is a punctuation character
    #[inline(always)]
    pub fn ispunct(b: u8) -> bool {
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

/// The character used in a setext heading underline
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetextChar {
    /// Equals sign `=` (for level 1 headings)
    Equals,
    /// Hyphen `-` (for level 2 headings)
    Hyphen,
}

/// Scan for an ATX heading start
///
/// Returns the number of consecutive `#` characters (1-6) if found,
/// or None if this is not an ATX heading start.
pub fn atx_heading_start(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut i = 0;

    // Skip up to 3 spaces of indentation
    while i < bytes.len() && i < 4 && bytes[i] == b' ' {
        i += 1;
    }

    // Count hash characters
    let start = i;
    while i < bytes.len() && bytes[i] == b'#' {
        i += 1;
    }

    let hash_count = i - start;

    // Must have 1-6 hashes
    if hash_count == 0 || hash_count > 6 {
        return None;
    }

    // After hashes, must have space, tab, or end of line
    if i < bytes.len() && !is_space_or_tab(bytes[i]) {
        return None;
    }

    Some(hash_count)
}

/// Scan for a setext heading underline
///
/// Returns the SetextChar type if a valid underline is found,
/// or None if this is not a setext heading line.
pub fn setext_heading_line(line: &str) -> Option<SetextChar> {
    let bytes = line.as_bytes();
    let mut i = 0;

    // Skip up to 3 spaces of indentation
    while i < bytes.len() && i < 4 && bytes[i] == b' ' {
        i += 1;
    }

    if i >= bytes.len() {
        return None;
    }

    // Determine the character
    let c = bytes[i];
    let setext_char = match c {
        b'=' => SetextChar::Equals,
        b'-' => SetextChar::Hyphen,
        _ => return None,
    };

    // Count consecutive characters
    let start = i;
    while i < bytes.len() && bytes[i] == c {
        i += 1;
    }

    // Must have at least one character
    if i == start {
        return None;
    }

    // Must have at least 3 characters for setext heading underline per CommonMark spec
    if i - start < 3 {
        return None;
    }

    // After the underline, only spaces/tabs are allowed, then end of line
    while i < bytes.len() && is_space_or_tab(bytes[i]) {
        i += 1;
    }

    // Must be at end of line
    if i < bytes.len() && !is_line_end_char(bytes[i]) {
        return None;
    }

    Some(setext_char)
}

/// Scan for an opening code fence
///
/// Returns the length of the fence (>= 3) if found, or None.
pub fn open_code_fence(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut i = 0;

    // Skip up to 3 spaces of indentation
    while i < bytes.len() && i < 4 && bytes[i] == b' ' {
        i += 1;
    }

    if i >= bytes.len() {
        return None;
    }

    // Must start with backtick or tilde
    let c = bytes[i];
    if c != b'`' && c != b'~' {
        return None;
    }

    // Count consecutive fence characters
    let start = i;
    while i < bytes.len() && bytes[i] == c {
        i += 1;
    }

    let fence_len = i - start;

    // Must be at least 3 characters
    if fence_len < 3 {
        return None;
    }

    // If using backticks, must not contain backticks in the info string
    if c == b'`' {
        while i < bytes.len() {
            if bytes[i] == b'`' {
                return None;
            }
            if is_line_end_char(bytes[i]) {
                break;
            }
            i += 1;
        }
    }

    Some(fence_len)
}

/// Scan for a closing code fence
///
/// Returns the length of the fence if a valid closing fence is found,
/// or None if this is not a closing fence.
pub fn close_code_fence(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut i = 0;

    // Skip up to 3 spaces of indentation
    while i < bytes.len() && i < 4 && bytes[i] == b' ' {
        i += 1;
    }

    if i >= bytes.len() {
        return None;
    }

    // Must start with backtick or tilde
    let c = bytes[i];
    if c != b'`' && c != b'~' {
        return None;
    }

    // Count consecutive fence characters
    let start = i;
    while i < bytes.len() && bytes[i] == c {
        i += 1;
    }

    let fence_len = i - start;

    // Must be at least 3 characters
    if fence_len < 3 {
        return None;
    }

    // After the fence, only spaces/tabs are allowed, then end of line
    while i < bytes.len() && is_space_or_tab(bytes[i]) {
        i += 1;
    }

    // Must be at end of line
    if i < bytes.len() && !is_line_end_char(bytes[i]) {
        return None;
    }

    Some(fence_len)
}

/// Scan for a thematic break
///
/// Returns the position after the thematic break if found, or None.
pub fn thematic_break(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut i = 0;

    // Skip up to 3 spaces of indentation
    while i < bytes.len() && i < 4 && bytes[i] == b' ' {
        i += 1;
    }

    if i >= bytes.len() {
        return None;
    }

    // Determine the character
    let c = bytes[i];
    if c != b'*' && c != b'-' && c != b'_' {
        return None;
    }

    let mut count = 0;
    while i < bytes.len() {
        if bytes[i] == c {
            count += 1;
            i += 1;
        } else if is_space_or_tab(bytes[i]) {
            i += 1;
        } else {
            break;
        }
    }

    // Must have at least 3 characters
    if count < 3 {
        return None;
    }

    // Must be at end of line
    while i < bytes.len() && is_space_or_tab(bytes[i]) {
        i += 1;
    }

    if i < bytes.len() && !is_line_end_char(bytes[i]) {
        return None;
    }

    Some(i)
}

/// Scan for HTML block start (types 1-6)
///
/// Returns the HTML block type (1-6) if found, or None.
pub fn html_block_start(line: &str) -> Option<u8> {
    let bytes = line.as_bytes();
    let mut i = 0;

    // Skip up to 3 spaces of indentation
    while i < bytes.len() && i < 4 && bytes[i] == b' ' {
        i += 1;
    }

    if i >= bytes.len() || bytes[i] != b'<' {
        return None;
    }

    let rest = &line[i..];

    // Type 1: <script, <pre, <style (case-insensitive)
    if rest.len() >= 7 {
        let tag = &rest[1..7].to_ascii_lowercase();
        if tag.starts_with("script")
            || tag.starts_with("pre")
            || tag.starts_with("style")
        {
            // Check for tag end or whitespace
            if tag.len() >= 7 {
                let after_tag = tag.as_bytes()[6];
                if after_tag == b'>' || after_tag == b' ' || after_tag == b'\t' {
                    return Some(1);
                }
            }
        }
    }

    // Type 2: <!--
    if rest.starts_with("<!--") {
        return Some(2);
    }

    // Type 3: <?
    if rest.starts_with("<?") {
        return Some(3);
    }

    // Type 4: <!
    if rest.starts_with("<!") && rest.len() >= 3 {
        let c = rest.as_bytes()[2];
        if c.is_ascii_uppercase() {
            return Some(4);
        }
    }

    // Type 5: <![CDATA[
    if rest.starts_with("<![CDATA[") {
        return Some(5);
    }

    // Type 6: HTML tags (checked separately)
    None
}

/// Scan for HTML block start type 7
///
/// Returns 7 if a valid HTML tag is found, or None.
pub fn html_block_start_7(line: &str) -> Option<u8> {
    let bytes = line.as_bytes();
    let mut i = 0;

    // Skip up to 3 spaces of indentation
    while i < bytes.len() && i < 4 && bytes[i] == b' ' {
        i += 1;
    }

    if i >= bytes.len() || bytes[i] != b'<' {
        return None;
    }

    // Try to match an HTML tag
    if let Some(len) = match_html_tag(&line[i..]) {
        if len > 0 {
            return Some(7);
        }
    }

    None
}

/// Match an HTML tag
///
/// Returns the length of the tag if found, or None.
fn match_html_tag(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    if bytes.is_empty() || bytes[0] != b'<' {
        return None;
    }

    let mut i = 1;

    // Check for closing tag
    if i < bytes.len() && bytes[i] == b'/' {
        i += 1;
    }

    // Must have a tag name
    if i >= bytes.len() || !bytes[i].is_ascii_alphabetic() {
        return None;
    }

    // Read tag name
    while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'-') {
        i += 1;
    }

    // Read attributes
    while i < bytes.len() {
        // Skip whitespace
        while i < bytes.len() && is_space_or_tab(bytes[i]) {
            i += 1;
        }

        if i >= bytes.len() {
            break;
        }

        // Check for tag end
        if bytes[i] == b'>' {
            return Some(i + 1);
        }

        if bytes[i] == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'>' {
            return Some(i + 2);
        }

        // Try to read an attribute
        if !bytes[i].is_ascii_alphabetic() && bytes[i] != b'_' && bytes[i] != b':' {
            break;
        }

        // Attribute name
        while i < bytes.len()
            && (bytes[i].is_ascii_alphanumeric()
                || matches!(bytes[i], b'_' | b':' | b'-'))
        {
            i += 1;
        }

        // Optional attribute value
        if i < bytes.len() && bytes[i] == b'=' {
            i += 1;
            if i < bytes.len() && (bytes[i] == b'"' || bytes[i] == b'\'') {
                let quote = bytes[i];
                i += 1;
                while i < bytes.len() && bytes[i] != quote {
                    i += 1;
                }
                if i < bytes.len() {
                    i += 1;
                }
            } else {
                // Unquoted attribute value
                while i < bytes.len() && !isspace(bytes[i]) && bytes[i] != b'>' {
                    i += 1;
                }
            }
        }
    }

    None
}

/// Check for HTML block end conditions
pub fn html_block_end_1(line: &str) -> bool {
    line.to_ascii_lowercase().contains("</script>")
        || line.to_ascii_lowercase().contains("</pre>")
        || line.to_ascii_lowercase().contains("</style>")
}

/// Check if line ends HTML block type 2 (comment end)
pub fn html_block_end_2(line: &str) -> bool {
    line.contains("-->")
}

/// Check if line ends HTML block type 3 (processing instruction end)
pub fn html_block_end_3(line: &str) -> bool {
    line.contains("?>")
}

/// Check if line ends HTML block type 4 (declaration end)
pub fn html_block_end_4(line: &str) -> bool {
    line.contains(">")
}

/// Check if line ends HTML block type 5 (CDATA end)
pub fn html_block_end_5(line: &str) -> bool {
    line.contains("]]>")
}

/// Scan for a task list item
///
/// Returns (end_position, matched_text, symbol_range) if found, or None.
pub fn tasklist(text: &str) -> Option<(usize, &str, Range<usize>)> {
    let bytes = text.as_bytes();
    let mut i = 0;

    // Skip leading whitespace
    while i < bytes.len() && is_space_or_tab(bytes[i]) {
        i += 1;
    }

    // Must start with [
    if i >= bytes.len() || bytes[i] != b'[' {
        return None;
    }
    i += 1;

    let symbol_start = i;

    // Check for space or x/X
    if i >= bytes.len() {
        return None;
    }

    let symbol = bytes[i] as char;
    if symbol != ' ' && symbol != 'x' && symbol != 'X' {
        return None;
    }
    i += 1;

    let symbol_end = i;

    // Must end with ]
    if i >= bytes.len() || bytes[i] != b']' {
        return None;
    }
    i += 1;

    // After ], must have space, tab, or end of line
    if i < bytes.len() && !is_space_or_tab(bytes[i]) && !is_line_end_char(bytes[i]) {
        return None;
    }

    Some((i, &text[symbol_start..symbol_end], symbol_start..symbol_end))
}

/// Scan for a footnote definition
///
/// Returns the position after the label if found, or None.
pub fn footnote_definition(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut i = 0;

    // Skip up to 3 spaces of indentation
    while i < bytes.len() && i < 4 && bytes[i] == b' ' {
        i += 1;
    }

    // Must start with [^
    if i + 1 >= bytes.len() || bytes[i] != b'[' || bytes[i + 1] != b'^' {
        return None;
    }
    i += 2;

    let label_start = i;

    // Read label (cannot contain whitespace or ]
    while i < bytes.len() && !matches!(bytes[i], b']' | b' ' | b'\t' | b'\n' | b'\r') {
        i += 1;
    }

    if i == label_start {
        return None;
    }

    // Must end with ]:
    if i + 1 >= bytes.len() || bytes[i] != b']' || bytes[i + 1] != b':' {
        return None;
    }

    Some(i + 2)
}

/// Scan for a link title (in reference definition)
///
/// Returns the length of the title if found, or None.
pub fn link_title(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    if bytes.is_empty() {
        return None;
    }

    let quote = bytes[0];
    if quote != b'"' && quote != b'\'' && quote != b'(' {
        return None;
    }

    let close = if quote == b'(' { b')' } else { quote };

    let mut i = 1;
    while i < bytes.len() {
        if bytes[i] == b'\\' && i + 1 < bytes.len() {
            i += 2;
        } else if bytes[i] == close {
            return Some(i + 1);
        } else if bytes[i] == b'\n' || bytes[i] == b'\r' {
            return None;
        } else {
            i += 1;
        }
    }

    None
}

/// Scan for an autolink (URL or email in angle brackets)
///
/// Returns (matched_text, length) if found, or None.
pub fn autolink(s: &str) -> Option<(&str, usize)> {
    let bytes = s.as_bytes();
    if bytes.is_empty() || bytes[0] != b'<' {
        return None;
    }

    // Try email first
    if let Some(result) = match_email_autolink(s) {
        return Some(result);
    }

    // Try URL
    if let Some(result) = match_url_autolink(s) {
        return Some(result);
    }

    None
}

/// Match an email autolink
fn match_email_autolink(s: &str) -> Option<(&str, usize)> {
    let bytes = s.as_bytes();
    if bytes.len() < 3 || bytes[0] != b'<' {
        return None;
    }

    let mut i = 1;

    // Must have at least one character before @
    let local_start = i;
    while i < bytes.len() && is_email_local_char(bytes[i]) {
        i += 1;
    }

    if i == local_start || i >= bytes.len() || bytes[i] != b'@' {
        return None;
    }

    i += 1; // skip @

    // Domain part
    let domain_start = i;
    while i < bytes.len() && is_email_domain_char(bytes[i]) {
        i += 1;
    }

    if i == domain_start {
        return None;
    }

    // Must end with >
    if i >= bytes.len() || bytes[i] != b'>' {
        return None;
    }

    let email = &s[1..i];
    Some((email, i + 1))
}

/// Match a URL autolink
fn match_url_autolink(s: &str) -> Option<(&str, usize)> {
    let bytes = s.as_bytes();
    if bytes.len() < 3 || bytes[0] != b'<' {
        return None;
    }

    let mut i = 1;

    // Scheme must start with letter
    if !bytes[i].is_ascii_alphabetic() {
        return None;
    }

    // Scheme characters
    let scheme_start = i;
    while i < bytes.len()
        && (bytes[i].is_ascii_alphanumeric() || matches!(bytes[i], b'+' | b'-' | b'.'))
    {
        i += 1;
    }

    // Must have ://
    if i + 2 >= bytes.len() || &s[i..i + 3] != "://" {
        return None;
    }

    i += 3;

    // URL body
    let _url_start = scheme_start;
    while i < bytes.len()
        && !matches!(bytes[i], b'<' | b'>' | b' ' | b'\t' | b'\n' | b'\r')
    {
        i += 1;
    }

    // Must end with >
    if i >= bytes.len() || bytes[i] != b'>' {
        return None;
    }

    let url = &s[1..i];
    Some((url, i + 1))
}

fn is_email_local_char(b: u8) -> bool {
    b.is_ascii_alphanumeric()
        || matches!(
            b,
            b'!' | b'#'
                | b'$'
                | b'%'
                | b'&'
                | b'\''
                | b'*'
                | b'+'
                | b'-'
                | b'/'
                | b'='
                | b'?'
                | b'^'
                | b'_'
                | b'`'
                | b'{'
                | b'|'
                | b'}'
                | b'~'
                | b'.'
        )
}

fn is_email_domain_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.')
}

fn isspace(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\n' | b'\r')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atx_heading_start() {
        assert_eq!(atx_heading_start("# Heading"), Some(1));
        assert_eq!(atx_heading_start("## Heading"), Some(2));
        assert_eq!(atx_heading_start("###### Heading"), Some(6));
        assert_eq!(atx_heading_start("####### Too many"), None);
        assert_eq!(atx_heading_start("No heading"), None);
        assert_eq!(atx_heading_start("  ## Indented"), Some(2));
    }

    #[test]
    fn test_setext_heading_line() {
        assert_eq!(setext_heading_line("==="), Some(SetextChar::Equals));
        assert_eq!(setext_heading_line("---"), Some(SetextChar::Hyphen));
        assert_eq!(setext_heading_line("  ==="), Some(SetextChar::Equals));
        assert_eq!(setext_heading_line("= = ="), None);
        assert_eq!(setext_heading_line("--"), None);
    }

    #[test]
    fn test_code_fence() {
        assert_eq!(open_code_fence("```"), Some(3));
        assert_eq!(open_code_fence("~~~~"), Some(4));
        assert_eq!(open_code_fence("``"), None);
        assert_eq!(open_code_fence("  ```"), Some(3));
        assert_eq!(open_code_fence("```rust"), Some(3));

        assert_eq!(close_code_fence("```"), Some(3));
        assert_eq!(close_code_fence("  ```"), Some(3));
    }

    #[test]
    fn test_thematic_break() {
        assert!(thematic_break("***").is_some());
        assert!(thematic_break("---").is_some());
        assert!(thematic_break("___").is_some());
        assert!(thematic_break(" * * * ").is_some());
        assert!(thematic_break("--").is_none());
        assert!(thematic_break("---text").is_none());
    }

    #[test]
    fn test_tasklist() {
        assert!(tasklist("[ ]").is_some());
        assert!(tasklist("[x]").is_some());
        assert!(tasklist("[X]").is_some());
        assert!(tasklist("[ ] Task").is_some());
        assert!(tasklist("[]").is_none());
        assert!(tasklist("[y]").is_none());
    }

    #[test]
    fn test_footnote_definition() {
        assert_eq!(footnote_definition("[^1]:"), Some(5));
        assert_eq!(footnote_definition("[^label]:"), Some(9));
        assert_eq!(footnote_definition("  [^1]:"), Some(7));
        assert_eq!(footnote_definition("[1]:"), None);
        assert_eq!(footnote_definition("[^1]"), None);
    }
}
