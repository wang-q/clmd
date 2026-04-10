//! Character utilities for clmd.
//!
//! This module provides character classification functions,
//! inspired by Pandoc's Text.Pandoc.Char module.
//!
//! # Example
//!
//! ```ignore
//! use clmd::text::char::is_cjk;
//!
//! assert!(is_cjk('中'));
//! assert!(is_cjk('日'));
//! assert!(is_cjk('한'));
//! assert!(!is_cjk('A'));
//! ```

use crate::parse::util::{BoxedParser, ClmdError, ClmdResult, Position};

// =============================================================================
// Character Type Utilities (from parse::util::scanners::ctype)
// =============================================================================

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

/// Check if a character is a CJK (Chinese, Japanese, Korean) character.
///
/// This function detects characters in the following Unicode blocks:
/// - CJK Unified Ideographs: U+4E00 - U+9FFF
/// - CJK Unified Ideographs Extension A: U+3400 - U+4DBF
/// - CJK Unified Ideographs Extension B: U+20000 - U+2A6DF
/// - CJK Unified Ideographs Extension C: U+2A700 - U+2B73F
/// - CJK Unified Ideographs Extension D: U+2B740 - U+2B81F
/// - CJK Radicals Supplement: U+2E80 - U+2EFF
/// - Kangxi Radicals: U+2F00 - U+2FDF
/// - Ideographic Description Characters: U+2FF0 - U+2FFF
/// - CJK Symbols and Punctuation: U+3000 - U+303F
/// - Hiragana: U+3040 - U+309F
/// - Katakana: U+30A0 - U+30FF
/// - Bopomofo: U+3100 - U+312F
/// - Kanbun: U+3190 - U+319F
/// - CJK Strokes: U+31C0 - U+31EF
/// - Katakana Phonetic Extensions: U+31F0 - U+31FF
/// - Enclosed CJK Letters & Months: U+3200 - U+32FF
/// - CJK Compatibility: U+3300 - U+33FF
/// - CJK Compatibility Ideographs: U+F900 - U+FAFF
/// - CJK Compatibility Ideographs Supplement: U+2F800 - U+2FA1F
/// - Hangul Syllables: U+AC00 - U+D7AF
/// - Hangul Jamo: U+1100 - U+11FF
/// - Hangul Jamo Extended-A: U+A960 - U+A97F
/// - Hangul Jamo Extended-B: U+D7B0 - U+D7FF
/// - Halfwidth and Fullwidth Forms: U+FF00 - U+FFEF
/// - Bopomofo Extended: U+31A0 - U+31BF
/// - Kana Supplement: U+1B000 - U+1B0FF
/// - Kana Extended-A: U+1B100 - U+1B12F
/// - Small Kana Extension: U+1B130 - U+1B16F
/// - Nushu: U+1B170 - U+1B2FF
/// - Tangut: U+17000 - U+187FF
/// - Tangut Components: U+18800 - U+18AFF
/// - Khitan Small Script: U+18B00 - U+18CFF
///
/// # Example
///
/// ```ignore
/// use clmd::text::char::is_cjk;
///
/// assert!(is_cjk('中'));  // Chinese
/// assert!(is_cjk('日'));  // Japanese
/// assert!(is_cjk('한'));  // Korean
/// assert!(is_cjk('あ'));  // Hiragana
/// assert!(is_cjk('ア'));  // Katakana
/// assert!(!is_cjk('A'));  // ASCII
/// assert!(!is_cjk('α'));  // Greek
/// ```ignore
pub fn is_cjk(c: char) -> bool {
    // Fast path for ASCII and Hangul Jamo (which are below 0x2E80)
    if c < '\u{1100}' {
        return false;
    }

    matches!(c,
        // Hangul Jamo
        '\u{1100}'..='\u{11FF}' |
        // CJK Radicals Supplement
        '\u{2E80}'..='\u{2EFF}' |
        // Kangxi Radicals
        '\u{2F00}'..='\u{2FDF}' |
        // Ideographic Description Characters
        '\u{2FF0}'..='\u{2FFF}' |
        // CJK Symbols and Punctuation
        '\u{3000}'..='\u{303F}' |
        // Hiragana
        '\u{3040}'..='\u{309F}' |
        // Katakana
        '\u{30A0}'..='\u{30FF}' |
        // Bopomofo
        '\u{3100}'..='\u{312F}' |
        // Kanbun
        '\u{3190}'..='\u{319F}' |
        // CJK Strokes
        '\u{31C0}'..='\u{31EF}' |
        // Katakana Phonetic Extensions
        '\u{31F0}'..='\u{31FF}' |
        // Enclosed CJK Letters & Months
        '\u{3200}'..='\u{32FF}' |
        // CJK Compatibility
        '\u{3300}'..='\u{33FF}' |
        // CJK Unified Ideographs Extension A
        '\u{3400}'..='\u{4DBF}' |
        // CJK Unified Ideographs
        '\u{4E00}'..='\u{9FFF}' |
        // CJK Compatibility Ideographs
        '\u{F900}'..='\u{FAFF}' |
        // Halfwidth and Fullwidth Forms
        '\u{FF00}'..='\u{FFEF}' |
        // Kana Supplement
        '\u{1B000}'..='\u{1B0FF}' |
        // Kana Extended-A
        '\u{1B100}'..='\u{1B12F}' |
        // Small Kana Extension
        '\u{1B130}'..='\u{1B16F}' |
        // Nushu
        '\u{1B170}'..='\u{1B2FF}' |
        // Tangut
        '\u{17000}'..='\u{187FF}' |
        // Tangut Components
        '\u{18800}'..='\u{18AFF}' |
        // Khitan Small Script
        '\u{18B00}'..='\u{18CFF}' |
        // CJK Unified Ideographs Extension B
        '\u{20000}'..='\u{2A6DF}' |
        // CJK Unified Ideographs Extension C
        '\u{2A700}'..='\u{2B73F}' |
        // CJK Unified Ideographs Extension D
        '\u{2B740}'..='\u{2B81F}' |
        // CJK Unified Ideographs Extension E
        '\u{2B820}'..='\u{2CEAF}' |
        // CJK Unified Ideographs Extension F
        '\u{2CEB0}'..='\u{2EBEF}' |
        // CJK Compatibility Ideographs Supplement
        '\u{2F800}'..='\u{2FA1F}' |
        // CJK Unified Ideographs Extension G
        '\u{30000}'..='\u{3134F}' |
        // Hangul Syllables
        '\u{AC00}'..='\u{D7AF}' |
        // Hangul Jamo Extended-A
        '\u{A960}'..='\u{A97F}' |
        // Hangul Jamo Extended-B
        '\u{D7B0}'..='\u{D7FF}'
    )
}

/// Check if a character is a CJK punctuation mark.
///
/// This includes common CJK punctuation characters that should be
/// treated specially in text processing.
///
/// # Example
///
/// ```ignore
/// use clmd::text::char::is_cjk_punctuation;
///
/// assert!(is_cjk_punctuation('。'));
/// assert!(is_cjk_punctuation('、'));
/// assert!(!is_cjk_punctuation('.'));
/// ```ignore
pub fn is_cjk_punctuation(c: char) -> bool {
    matches!(c,
        // CJK Symbols and Punctuation
        '\u{3000}'..='\u{303F}' |
        // Fullwidth ASCII variants
        '\u{FF01}'..='\u{FF0F}' |
        '\u{FF1A}'..='\u{FF20}' |
        '\u{FF3B}'..='\u{FF40}' |
        '\u{FF5B}'..='\u{FF65}'
    )
}

/// Check if a character is a fullwidth character.
///
/// Fullwidth characters are typically displayed with double width
/// in monospace fonts.
///
/// # Example
///
/// ```ignore
/// use clmd::text::char::is_fullwidth;
///
/// assert!(is_fullwidth('Ａ'));  // Fullwidth A
/// assert!(is_fullwidth('中'));
/// assert!(!is_fullwidth('A'));
/// ```ignore
pub fn is_fullwidth(c: char) -> bool {
    matches!(c,
        '\u{1100}'..='\u{115F}' |  // Hangul Jamo
        '\u{2E80}'..='\u{2EFF}' |  // CJK Radicals Supplement
        '\u{2F00}'..='\u{2FDF}' |  // Kangxi Radicals
        '\u{2FF0}'..='\u{303F}' |  // Ideographic Description Characters + CJK Symbols and Punctuation
        '\u{3040}'..='\u{309F}' |  // Hiragana
        '\u{30A0}'..='\u{30FF}' |  // Katakana
        '\u{3100}'..='\u{312F}' |  // Bopomofo
        '\u{3130}'..='\u{318F}' |  // Hangul Compatibility Jamo
        '\u{3190}'..='\u{31BF}' |  // Kanbun + Bopomofo Extended
        '\u{31C0}'..='\u{31EF}' |  // CJK Strokes
        '\u{31F0}'..='\u{31FF}' |  // Katakana Phonetic Extensions
        '\u{3200}'..='\u{32FF}' |  // Enclosed CJK Letters & Months
        '\u{3300}'..='\u{33FF}' |  // CJK Compatibility
        '\u{3400}'..='\u{4DBF}' |  // CJK Unified Ideographs Extension A
        '\u{4E00}'..='\u{9FFF}' |  // CJK Unified Ideographs
        '\u{A960}'..='\u{A97F}' |  // Hangul Jamo Extended-A
        '\u{AC00}'..='\u{D7A3}' |  // Hangul Syllables
        '\u{D7B0}'..='\u{D7FF}' |  // Hangul Jamo Extended-B
        '\u{F900}'..='\u{FAFF}' |  // CJK Compatibility Ideographs
        '\u{FF01}'..='\u{FF60}' |  // Fullwidth ASCII variants
        '\u{FFE0}'..='\u{FFE6}'    // Fullwidth symbol variants
    )
}

/// Check if a string contains any CJK characters.
///
/// # Example
///
/// ```ignore
/// use clmd::text::char::has_cjk;
///
/// assert!(has_cjk("Hello 世界"));
/// assert!(!has_cjk("Hello World"));
/// ```ignore
pub fn has_cjk(s: &str) -> bool {
    s.chars().any(is_cjk)
}

/// Count the number of CJK characters in a string.
///
/// # Example
///
/// ```ignore
/// use clmd::text::char::count_cjk;
///
/// assert_eq!(count_cjk("Hello 世界"), 2);
/// assert_eq!(count_cjk("日本語"), 3);
/// ```ignore
pub fn count_cjk(s: &str) -> usize {
    s.chars().filter(|&c| is_cjk(c)).count()
}

/// Parse a digit character.
pub fn digit(input: &str, pos: Position) -> ClmdResult<(char, Position)> {
    if let Some(ch) = input[pos.offset..].chars().next() {
        if ch.is_ascii_digit() {
            let mut new_pos = pos;
            new_pos.advance(ch);
            return Ok((ch, new_pos));
        }
    }
    Err(ClmdError::parse_error(pos, "Expected digit"))
}

/// Parse an alphabetic character.
pub fn alpha(input: &str, pos: Position) -> ClmdResult<(char, Position)> {
    if let Some(ch) = input[pos.offset..].chars().next() {
        if ch.is_alphabetic() {
            let mut new_pos = pos;
            new_pos.advance(ch);
            return Ok((ch, new_pos));
        }
    }
    Err(ClmdError::parse_error(pos, "Expected letter"))
}

/// Parse an alphanumeric character.
pub fn alphanumeric(input: &str, pos: Position) -> ClmdResult<(char, Position)> {
    if let Some(ch) = input[pos.offset..].chars().next() {
        if ch.is_alphanumeric() {
            let mut new_pos = pos;
            new_pos.advance(ch);
            return Ok((ch, new_pos));
        }
    }
    Err(ClmdError::parse_error(
        pos,
        "Expected alphanumeric character",
    ))
}

/// Parse a whitespace character.
pub fn whitespace(input: &str, pos: Position) -> ClmdResult<(char, Position)> {
    if let Some(ch) = input[pos.offset..].chars().next() {
        if ch.is_whitespace() {
            let mut new_pos = pos;
            new_pos.advance(ch);
            return Ok((ch, new_pos));
        }
    }
    Err(ClmdError::parse_error(pos, "Expected whitespace"))
}

/// Parse a newline character (\n or \r\n).
pub fn newline(input: &str, pos: Position) -> ClmdResult<(char, Position)> {
    let remaining = &input[pos.offset..];
    if remaining.starts_with("\r\n") {
        let mut new_pos = pos;
        new_pos.advance('\r');
        new_pos.advance('\n');
        Ok(('\n', new_pos))
    } else if let Some('\n') = remaining.chars().next() {
        let mut new_pos = pos;
        new_pos.advance('\n');
        Ok(('\n', new_pos))
    } else {
        Err(ClmdError::parse_error(pos, "Expected newline"))
    }
}

/// Parse zero or more occurrences of a parser.
pub fn many<T>(parser: BoxedParser<T>) -> BoxedParser<Vec<T>>
where
    T: 'static,
{
    Box::new(move |input: &str, pos| {
        let mut results = Vec::new();
        let mut current_pos = pos;

        loop {
            match parser(input, current_pos) {
                Ok((value, new_pos)) => {
                    results.push(value);
                    current_pos = new_pos;
                }
                Err(_) => break,
            }
        }

        Ok((results, current_pos))
    })
}

/// Parse one or more occurrences of a parser.
pub fn many1<T>(parser: BoxedParser<T>) -> BoxedParser<Vec<T>>
where
    T: 'static,
{
    Box::new(move |input: &str, pos| {
        let mut results = Vec::new();
        let mut current_pos = pos;

        // Must have at least one
        match parser(input, current_pos) {
            Ok((value, new_pos)) => {
                results.push(value);
                current_pos = new_pos;
            }
            Err(e) => return Err(e),
        }

        // Then parse more
        loop {
            match parser(input, current_pos) {
                Ok((value, new_pos)) => {
                    results.push(value);
                    current_pos = new_pos;
                }
                Err(_) => break,
            }
        }

        Ok((results, current_pos))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // =============================================================================
    // Character Type Tests (from parse::util::scanners::ctype)
    // =============================================================================

    #[test]
    fn test_isspace() {
        assert!(isspace(b' '));
        assert!(isspace(b'\t'));
        assert!(isspace(b'\n'));
        assert!(isspace(b'\r'));
        assert!(!isspace(b'a'));
        assert!(!isspace(b'1'));
    }

    #[test]
    fn test_isdigit() {
        assert!(isdigit(b'0'));
        assert!(isdigit(b'9'));
        assert!(!isdigit(b'a'));
        assert!(!isdigit(b' '));
    }

    #[test]
    fn test_isalpha() {
        assert!(isalpha(b'a'));
        assert!(isalpha(b'Z'));
        assert!(!isalpha(b'1'));
        assert!(!isalpha(b' '));
    }

    #[test]
    fn test_isalnum() {
        assert!(isalnum(b'a'));
        assert!(isalnum(b'1'));
        assert!(!isalnum(b' '));
        assert!(!isalnum(b'!'));
    }

    #[test]
    fn test_is_punctuation_fast() {
        assert!(is_punctuation_fast(b'!'));
        assert!(is_punctuation_fast(b'.'));
        assert!(is_punctuation_fast(b'@'));
        assert!(!is_punctuation_fast(b'a'));
        assert!(!is_punctuation_fast(b'1'));
        assert!(!is_punctuation_fast(b' '));
    }

    #[test]
    fn test_is_punctuation() {
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

    // =============================================================================
    // Scanner Character Tests
    // =============================================================================

    #[test]
    fn test_is_space_or_tab() {
        assert!(is_space_or_tab(b' '));
        assert!(is_space_or_tab(b'\t'));
        assert!(!is_space_or_tab(b'\n'));
        assert!(!is_space_or_tab(b'a'));
    }

    #[test]
    fn test_is_line_end_char() {
        assert!(is_line_end_char(b'\n'));
        assert!(is_line_end_char(b'\r'));
        assert!(!is_line_end_char(b' '));
        assert!(!is_line_end_char(b'a'));
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
    }

    #[test]
    fn test_is_special_byte() {
        assert!(is_special_byte(b'*', false));
        assert!(is_special_byte(b'_', false));
        assert!(is_special_byte(b'[', false));
        assert!(!is_special_byte(b'a', false));
        assert!(!is_special_byte(b' ', false));
    }

    // =============================================================================
    // CJK Character Tests
    // =============================================================================

    #[test]
    fn test_is_cjk_chinese() {
        assert!(is_cjk('中'));
        assert!(is_cjk('文'));
        assert!(is_cjk('字'));
    }

    #[test]
    fn test_is_cjk_japanese() {
        assert!(is_cjk('日'));
        assert!(is_cjk('本'));
        assert!(is_cjk('語'));
        assert!(is_cjk('あ')); // Hiragana
        assert!(is_cjk('ア')); // Katakana
    }

    #[test]
    fn test_is_cjk_korean() {
        assert!(is_cjk('한'));
        assert!(is_cjk('국'));
        assert!(is_cjk('어'));
    }

    #[test]
    fn test_is_not_cjk() {
        assert!(!is_cjk('A'));
        assert!(!is_cjk('a'));
        assert!(!is_cjk('0'));
        assert!(!is_cjk(' '));
        assert!(!is_cjk('α')); // Greek
        assert!(!is_cjk('é')); // Latin with accent
    }

    #[test]
    fn test_is_cjk_punctuation() {
        assert!(is_cjk_punctuation('。'));
        assert!(is_cjk_punctuation('、'));
        assert!(is_cjk_punctuation('「'));
        assert!(is_cjk_punctuation('」'));
        assert!(!is_cjk_punctuation('.'));
        assert!(!is_cjk_punctuation(','));
    }

    #[test]
    fn test_is_fullwidth() {
        assert!(is_fullwidth('Ａ'));
        assert!(is_fullwidth('１'));
        assert!(is_fullwidth('中'));
        assert!(!is_fullwidth('A'));
        assert!(!is_fullwidth('1'));
    }

    #[test]
    fn test_has_cjk() {
        assert!(has_cjk("Hello 世界"));
        assert!(has_cjk("日本語"));
        assert!(!has_cjk("Hello World"));
        assert!(!has_cjk("12345"));
    }

    #[test]
    fn test_count_cjk() {
        assert_eq!(count_cjk("Hello 世界"), 2);
        assert_eq!(count_cjk("日本語"), 3);
        assert_eq!(count_cjk("Hello World"), 0);
    }

    #[test]
    fn test_cjk_extension_ranges() {
        // Test some characters from extension blocks
        assert!(is_cjk('\u{20000}')); // Extension B
        assert!(is_cjk('\u{2A700}')); // Extension C
        assert!(is_cjk('\u{2B740}')); // Extension D
    }

    #[test]
    fn test_hangul() {
        assert!(is_cjk('가')); // Hangul syllable
        assert!(is_cjk('힣')); // Last Hangul syllable
    }

    #[test]
    fn test_digit() {
        let result = digit("123", Position::start()).unwrap();
        assert_eq!(result.0, '1');
        assert!(digit("abc", Position::start()).is_err());
    }

    #[test]
    fn test_alpha() {
        let result = alpha("abc", Position::start()).unwrap();
        assert_eq!(result.0, 'a');
        assert!(alpha("123", Position::start()).is_err());
    }

    #[test]
    fn test_whitespace() {
        let result = whitespace("  hello", Position::start()).unwrap();
        assert_eq!(result.0, ' ');
        let result = whitespace("\thello", Position::start()).unwrap();
        assert_eq!(result.0, '\t');
        assert!(whitespace("hello", Position::start()).is_err());
    }

    #[test]
    fn test_newline() {
        assert_eq!(newline("\n", Position::start()).unwrap().0, '\n');
        assert_eq!(newline("\r\n", Position::start()).unwrap().0, '\n');
        assert!(newline("hello", Position::start()).is_err());
    }

    #[test]
    fn test_alphanumeric() {
        let result = alphanumeric("abc", Position::start()).unwrap();
        assert_eq!(result.0, 'a');
        let result = alphanumeric("123", Position::start()).unwrap();
        assert_eq!(result.0, '1');
        assert!(alphanumeric(" ", Position::start()).is_err());
    }

    #[test]
    fn test_many() {
        let parser = many(Box::new(digit));
        let result = parser("123abc", Position::start()).unwrap();
        assert_eq!(result.0, vec!['1', '2', '3']);

        // Zero matches is ok
        let result = parser("abc", Position::start()).unwrap();
        assert!(result.0.is_empty());
    }

    #[test]
    fn test_many1() {
        let parser = many1(Box::new(digit));
        let result = parser("123abc", Position::start()).unwrap();
        assert_eq!(result.0, vec!['1', '2', '3']);

        // Must have at least one
        assert!(parser("abc", Position::start()).is_err());
    }
}
