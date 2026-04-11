//! Character utilities for clmd.
//!
//! This module provides character classification functions,
//! inspired by Pandoc's Text.Pandoc.Char module.

/// Check if a character is punctuation
pub fn is_punctuation(c: char) -> bool {
    // Unicode punctuation (Pc, Pd, Ps, Pe, Pi, Pf, Po categories)
    // Includes CJK punctuation and fullwidth ASCII variants
    if c.is_ascii() {
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
    } else {
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
            '\u{3000}'..='\u{303F}' | // CJK Symbols and Punctuation
            // Fullwidth ASCII variants (CJK punctuation)
            '\u{FF01}'..='\u{FF0F}' | // ！＂＃＄％＆＇（）＊＋，－．／
            '\u{FF1A}'..='\u{FF20}' | // ：；＜＝＞？＠
            '\u{FF3B}'..='\u{FF40}' | // ［＼］＾＿｀
            '\u{FF5B}'..='\u{FF65}'   // ｛｜｝～｟｠｡｢｣､･
        )
    }
}

/// Check if a character is a CJK punctuation mark.
///
/// This includes common CJK punctuation characters that should be
/// treated specially in text processing.
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

/// Check if a character is an ASCII letter.
pub fn is_ascii_letter(c: char) -> bool {
    c.is_ascii_alphabetic()
}

/// Check if a character is an ASCII digit.
pub fn is_ascii_digit(c: char) -> bool {
    c.is_ascii_digit()
}

/// Check if a character is ASCII punctuation that should NOT have space added.
///
/// This includes characters like `:`, `,`, `.`, `;`, `!`, `?` which are
/// commonly used in Markdown formatting and should not have spaces added
/// before them.
pub fn is_ascii_punctuation_no_space(c: char) -> bool {
    matches!(c, ':' | ',' | '.' | ';' | '!' | '?')
}

/// Check if a character is a closing bracket.
///
/// These characters (`)`, `]`, `}`, `>`) should have space added after them
/// when followed by ASCII alphanumeric characters.
pub fn is_closing_bracket(c: char) -> bool {
    matches!(c, ')' | ']' | '}' | '>')
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_is_cjk_punctuation() {
        assert!(is_cjk_punctuation('。'));
        assert!(is_cjk_punctuation('、'));
        assert!(is_cjk_punctuation('「'));
        assert!(is_cjk_punctuation('」'));
        assert!(!is_cjk_punctuation('.'));
        assert!(!is_cjk_punctuation(','));
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

    #[test]
    fn test_is_ascii_letter() {
        assert!(is_ascii_letter('a'));
        assert!(is_ascii_letter('Z'));
        assert!(!is_ascii_letter('1'));
        assert!(!is_ascii_letter('中'));
    }

    #[test]
    fn test_is_ascii_digit() {
        assert!(is_ascii_digit('0'));
        assert!(is_ascii_digit('9'));
        assert!(!is_ascii_digit('a'));
        assert!(!is_ascii_digit('中'));
    }

    #[test]
    fn test_is_ascii_punctuation_no_space() {
        assert!(is_ascii_punctuation_no_space(':'));
        assert!(is_ascii_punctuation_no_space(','));
        assert!(is_ascii_punctuation_no_space('.'));
        assert!(!is_ascii_punctuation_no_space('a'));
    }

    #[test]
    fn test_is_closing_bracket() {
        assert!(is_closing_bracket(')'));
        assert!(is_closing_bracket(']'));
        assert!(is_closing_bracket('}'));
        assert!(!is_closing_bracket('('));
    }
}
