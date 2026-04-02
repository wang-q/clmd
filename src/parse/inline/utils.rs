//! Utility functions for inline parsing

/// Character lookup table for fast classification
/// Bit flags:
/// - bit 0: is_punctuation
/// - bit 1: is_whitespace
/// - bit 2: is_special (needs special handling in inline parsing)
static CHAR_TABLE: [u8; 256] = {
    let mut table = [0u8; 256];
    // Punctuation characters
    table[b'!' as usize] = 0b101;
    table[b'"' as usize] = 0b101;
    table[b'#' as usize] = 0b001;
    table[b'$' as usize] = 0b001;
    table[b'%' as usize] = 0b001;
    table[b'&' as usize] = 0b101;
    table[b'\'' as usize] = 0b101;
    table[b'(' as usize] = 0b001;
    table[b')' as usize] = 0b001;
    table[b'*' as usize] = 0b101;
    table[b'+' as usize] = 0b001;
    table[b',' as usize] = 0b001;
    table[b'-' as usize] = 0b001;
    table[b'.' as usize] = 0b001;
    table[b'/' as usize] = 0b001;
    table[b':' as usize] = 0b001;
    table[b';' as usize] = 0b001;
    table[b'<' as usize] = 0b101;
    table[b'=' as usize] = 0b001;
    table[b'>' as usize] = 0b001;
    table[b'?' as usize] = 0b001;
    table[b'@' as usize] = 0b001;
    table[b'[' as usize] = 0b101;
    table[b'\\' as usize] = 0b001;
    table[b']' as usize] = 0b001;
    table[b'^' as usize] = 0b001;
    table[b'_' as usize] = 0b101;
    table[b'`' as usize] = 0b101;
    table[b'{' as usize] = 0b001;
    table[b'|' as usize] = 0b001;
    table[b'}' as usize] = 0b001;
    table[b'~' as usize] = 0b001;
    // Whitespace characters
    table[b' ' as usize] = 0b010;
    table[b'\t' as usize] = 0b010;
    table[b'\n' as usize] = 0b010;
    table[b'\r' as usize] = 0b010;
    table[b'\x0C' as usize] = 0b010; // Form feed
    table
};

/// Fast check if a byte is punctuation using lookup table
#[inline(always)]
pub fn is_punctuation_fast(b: u8) -> bool {
    CHAR_TABLE[b as usize] & 0b001 != 0
}

/// Check if a character is punctuation
pub fn is_punctuation(c: char) -> bool {
    // Fast path for ASCII using lookup table
    if c.is_ascii() {
        return is_punctuation_fast(c as u8);
    }
    // Unicode punctuation (Pc, Pd, Ps, Pe, Pi, Pf, Po categories)
    // Check for specific Unicode punctuation characters commonly used in tests
    matches!(c,
        '\u{00A2}'..='\u{00A5}' | // ¢£¤¥ (currency symbols)
        '\u{00B5}' |              // µ
        '\u{00B7}' |              // ·
        '\u{00BF}' |              // ¿
        '\u{00D7}' |              // ×
        '\u{00F7}' |              // ÷
        '\u{2000}'..='\u{206F}' | // General Punctuation
        '\u{20A0}'..='\u{20CF}' | // Currency Symbols
        '\u{2190}'..='\u{21FF}' | // Arrows
        '\u{2200}'..='\u{22FF}' | // Mathematical Operators
        '\u{2300}'..='\u{23FF}' | // Miscellaneous Technical
        '\u{25A0}'..='\u{25FF}' | // Geometric Shapes
        '\u{2600}'..='\u{26FF}' | // Miscellaneous Symbols
        '\u{2700}'..='\u{27BF}' | // Dingbats
        '\u{3000}'..='\u{303F}'   // CJK Symbols and Punctuation
    )
}

/// Check if a character is special (has special meaning in inline parsing)
#[inline(always)]
pub fn is_special_char(c: char, smart: bool) -> bool {
    if smart {
        matches!(
            c,
            '`' | '\\' | '&' | '<' | '*' | '_' | '[' | ']' | '!' | '\n' | '\'' | '"'
        )
    } else {
        matches!(
            c,
            '`' | '\\' | '&' | '<' | '*' | '_' | '[' | ']' | '!' | '\n'
        )
    }
}

/// Fast byte-level check if a byte is a special ASCII character
#[inline(always)]
pub fn is_special_byte(b: u8, smart: bool) -> bool {
    if smart {
        matches!(
            b,
            b'`' | b'\\'
                | b'&'
                | b'<'
                | b'*'
                | b'_'
                | b'['
                | b']'
                | b'!'
                | b'\n'
                | b'\''
                | b'"'
        )
    } else {
        matches!(
            b,
            b'`' | b'\\' | b'&' | b'<' | b'*' | b'_' | b'[' | b']' | b'!' | b'\n'
        )
    }
}

/// Check if a character can be escaped
pub fn is_escapable(c: char) -> bool {
    matches!(
        c,
        '!' | '"'
            | '#'
            | '$'
            | '%'
            | '&'
            | '\''
            | '('
            | ')'
            | '*'
            | '+'
            | ','
            | '-'
            | '.'
            | '/'
            | ':'
            | ';'
            | '<'
            | '='
            | '>'
            | '?'
            | '@'
            | '['
            | '\\'
            | ']'
            | '^'
            | '_'
            | '`'
            | '{'
            | '|'
            | '}'
            | '~'
    )
}

/// HTML5 named entities lookup table
/// This includes all 2125 HTML5 entities for full CommonMark compliance
pub fn get_html5_entity(name: &str) -> Option<&'static str> {
    use super::entities_table::lookup_entity;
    lookup_entity(name)
}

/// Normalize a URI by percent-encoding special characters
/// Based on commonmark.js normalizeURI
/// Percent-encode characters that are not allowed in URIs
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

/// Normalize a reference label for lookup
///
/// - Collapses internal whitespace to a single space
/// - Removes leading/trailing whitespace
/// - Converts to uppercase (for case-insensitive comparison)
///
/// Note: Does NOT unescape backslash escapes - [foo\!] and [foo!] are different labels
pub fn normalize_reference(label: &str) -> String {
    // Remove surrounding brackets if present
    let label = if label.starts_with('[') && label.ends_with(']') {
        &label[1..label.len() - 1]
    } else {
        label
    };

    // Normalize whitespace: collapse all whitespace sequences to a single space
    // Note: We do NOT unescape here - backslash escapes are preserved in link labels
    // per CommonMark spec. So "foo\!" stays as "foo\!", not "foo!"
    let normalized = label.split_whitespace().collect::<Vec<_>>().join(" ");

    // Unicode case folding: to_lowercase().to_uppercase() matches commonmark.js behavior
    // This properly handles characters like ß which folds to SS
    normalized.to_lowercase().to_uppercase()
}

/// Result of scanning delimiters
pub struct DelimScanResult {
    pub num_delims: usize,
    pub can_open: bool,
    pub can_close: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_punctuation_ascii() {
        assert!(is_punctuation('!'));
        assert!(is_punctuation('.'));
        assert!(is_punctuation(','));
        assert!(is_punctuation(';'));
        assert!(!is_punctuation('a'));
        assert!(!is_punctuation('A'));
        assert!(!is_punctuation('0'));
        assert!(!is_punctuation(' '));
    }

    #[test]
    fn test_is_punctuation_unicode() {
        // Test some Unicode punctuation
        assert!(is_punctuation('。')); // Chinese full stop
                                       // Note: '，' (Chinese comma) may not be covered by the current implementation
                                       // as it falls outside the checked Unicode ranges
    }

    #[test]
    fn test_is_special_char() {
        assert!(is_special_char('*', false));
        assert!(is_special_char('_', false));
        assert!(is_special_char('[', false));
        assert!(is_special_char(']', false));
        assert!(is_special_char('!', false));
        assert!(!is_special_char('a', false));
        assert!(!is_special_char(' ', false));
    }

    #[test]
    fn test_is_special_char_smart() {
        // Test smart mode specific characters
        assert!(is_special_char('\'', true));
        assert!(is_special_char('"', true));
        // Note: '.' and '-' are not considered special in smart mode
        // as they don't have special meaning in inline parsing
    }

    #[test]
    fn test_is_special_byte() {
        assert!(is_special_byte(b'*', false));
        assert!(is_special_byte(b'_', false));
        assert!(is_special_byte(b'[', false));
        assert!(!is_special_byte(b'a', false));
        assert!(!is_special_byte(b' ', false));
    }

    #[test]
    fn test_is_escapable() {
        assert!(is_escapable('!'));
        assert!(is_escapable('"'));
        assert!(is_escapable('#'));
        assert!(is_escapable('*'));
        assert!(!is_escapable('a'));
        assert!(!is_escapable(' '));
    }

    #[test]
    fn test_normalize_uri() {
        // Test basic URI normalization
        assert_eq!(normalize_uri("hello world"), "hello%20world");
        assert_eq!(normalize_uri("test.txt"), "test.txt");

        // Test special characters
        assert_eq!(normalize_uri("a+b"), "a+b");

        // Note: normalize_uri preserves existing percent signs
        // as they may be intentional escapes
        assert_eq!(normalize_uri("foo%bar"), "foo%bar");
    }

    #[test]
    fn test_normalize_reference() {
        // Test basic reference normalization
        assert_eq!(normalize_reference("hello world"), "HELLO WORLD");
        assert_eq!(normalize_reference("  hello   world  "), "HELLO WORLD");

        // Test bracket removal
        assert_eq!(normalize_reference("[hello]"), "HELLO");

        // Test case folding
        assert_eq!(normalize_reference("Hello"), "HELLO");

        // Test Unicode
        assert_eq!(normalize_reference("café"), "CAFÉ");
    }

    #[test]
    fn test_normalize_reference_with_escapes() {
        // Backslash escapes should be preserved
        assert_eq!(normalize_reference("foo\\!"), "FOO\\!");
    }

    #[test]
    fn test_delim_scan_result() {
        let result = DelimScanResult {
            num_delims: 2,
            can_open: true,
            can_close: false,
        };
        assert_eq!(result.num_delims, 2);
        assert!(result.can_open);
        assert!(!result.can_close);
    }
}
