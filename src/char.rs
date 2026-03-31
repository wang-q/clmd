//! Character utilities for clmd.
//!
//! This module provides character classification functions,
//! inspired by Pandoc's Text.Pandoc.Char module.
//!
//! # Example
//!
//! ```
//! use clmd::char::is_cjk;
//!
//! assert!(is_cjk('中'));
//! assert!(is_cjk('日'));
//! assert!(is_cjk('한'));
//! assert!(!is_cjk('A'));
//! ```

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
/// ```
/// use clmd::char::is_cjk;
///
/// assert!(is_cjk('中'));  // Chinese
/// assert!(is_cjk('日'));  // Japanese
/// assert!(is_cjk('한'));  // Korean
/// assert!(is_cjk('あ'));  // Hiragana
/// assert!(is_cjk('ア'));  // Katakana
/// assert!(!is_cjk('A'));  // ASCII
/// assert!(!is_cjk('α'));  // Greek
/// ```
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
/// ```
/// use clmd::char::is_cjk_punctuation;
///
/// assert!(is_cjk_punctuation('。'));
/// assert!(is_cjk_punctuation('、'));
/// assert!(!is_cjk_punctuation('.'));
/// ```
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
/// ```
/// use clmd::char::is_fullwidth;
///
/// assert!(is_fullwidth('Ａ'));  // Fullwidth A
/// assert!(is_fullwidth('中'));
/// assert!(!is_fullwidth('A'));
/// ```
pub fn is_fullwidth(c: char) -> bool {
    matches!(c,
        '\u{1100}'..='\u{115F}' |  // Hangul Jamo
        '\u{2E80}'..='\u{A4CF}' |  // CJK and related
        '\u{AC00}'..='\u{D7A3}' |  // Hangul Syllables
        '\u{F900}'..='\u{FAFF}' |  // CJK Compatibility Ideographs
        '\u{FF01}'..='\u{FF60}' |  // Fullwidth ASCII variants
        '\u{FFE0}'..='\u{FFE6}'    // Fullwidth symbol variants
    )
}

/// Check if a string contains any CJK characters.
///
/// # Example
///
/// ```
/// use clmd::char::has_cjk;
///
/// assert!(has_cjk("Hello 世界"));
/// assert!(!has_cjk("Hello World"));
/// ```
pub fn has_cjk(s: &str) -> bool {
    s.chars().any(is_cjk)
}

/// Count the number of CJK characters in a string.
///
/// # Example
///
/// ```
/// use clmd::char::count_cjk;
///
/// assert_eq!(count_cjk("Hello 世界"), 2);
/// assert_eq!(count_cjk("日本語"), 3);
/// ```
pub fn count_cjk(s: &str) -> usize {
    s.chars().filter(|&c| is_cjk(c)).count()
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(is_cjk('あ'));  // Hiragana
        assert!(is_cjk('ア'));  // Katakana
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
        assert!(!is_cjk('α'));  // Greek
        assert!(!is_cjk('é'));  // Latin with accent
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
        assert!(is_cjk('\u{20000}'));  // Extension B
        assert!(is_cjk('\u{2A700}'));  // Extension C
        assert!(is_cjk('\u{2B740}'));  // Extension D
    }

    #[test]
    fn test_hangul() {
        assert!(is_cjk('가'));  // Hangul syllable
        assert!(is_cjk('힣'));  // Last Hangul syllable
    }
}
