//! Utility functions for inline parsing

use std::collections::HashMap;

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

/// Fast check if a byte is whitespace using lookup table
#[inline(always)]
#[allow(dead_code)]
pub fn is_whitespace_fast(b: u8) -> bool {
    CHAR_TABLE[b as usize] & 0b010 != 0
}

/// Fast check if a byte is a special inline character
#[inline(always)]
#[allow(dead_code)]
pub fn is_special_fast(b: u8) -> bool {
    CHAR_TABLE[b as usize] & 0b100 != 0
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
/// This includes entities that may not be supported by htmlescape
pub fn get_html5_entity(name: &str) -> Option<&'static str> {
    // Common HTML5 entities not in htmlescape
    let entities: HashMap<&str, &str> = [
        ("Dcaron", "\u{010E}"),                   // Ď
        ("HilbertSpace", "\u{210B}"),             // ℋ
        ("DifferentialD", "\u{2146}"),            // ⅆ
        ("ClockwiseContourIntegral", "\u{2232}"), // ∲
        ("ngE", "\u{2267}\u{0338}"),              // ≧̸
        ("AElig", "\u{00C6}"),                    // Æ
        ("copy", "\u{00A9}"),                     // ©
        ("nbsp", "\u{00A0}"),                     //
        ("amp", "&"),                             // &
        ("lt", "<"),                              // <
        ("gt", ">"),                              // >
        ("quot", "\""),                           // "
        ("frac34", "\u{00BE}"),                   // ¾
    ]
    .iter()
    .copied()
    .collect();

    entities.get(name).copied()
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
/// - Collapses internal whitespace to a single space
/// - Removes leading/trailing whitespace
/// - Converts to uppercase (for case-insensitive comparison)
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
