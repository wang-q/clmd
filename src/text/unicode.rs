//! Unicode display width calculation
//!
//! This module provides functionality to determine the number of columns required
//! to display a string in a monospace font, following Unicode 16.0.0 standard.
//!
//! It correctly handles:
//! - ASCII characters (width 1)
//! - CJK characters (width 2)
//! - Emoji characters (width 1 or 2)
//! - Grapheme clusters (combined characters)
//! - Variation selectors
//!
//! # Examples
//!
//! ```ignore
//! use clmd::text::unicode_width::width;
//!
//! assert_eq!(width("hello"), 5);
//! assert_eq!(width("🦀"), 2);
//! assert_eq!(width("👨‍👩‍👧‍👧"), 2);
//! assert_eq!(width("中文"), 4);
//! ```

use unicode_segmentation::UnicodeSegmentation;

// Re-export character functions needed for CJK spacing
use crate::text::char::{
    is_ascii_digit, is_ascii_letter, is_ascii_punctuation_no_space, is_cjk_punctuation,
    is_closing_bracket,
};

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
/// use clmd::text::unicode_width::is_cjk;
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

/// Inclusive hexadecimal range of Unicode code points
pub type CodePointRange = std::ops::RangeInclusive<u32>;

/// Simple ASCII characters - used a lot, so we check them first.
const ASCII_TABLE: &[CodePointRange] = &[0x00020..=0x0007E];

/// Width 2 characters according to Unicode 16.0.0 East Asian Width specification.
const DOUBLEWIDE_TABLE: &[CodePointRange] = &[
    0x1100..=0x115f,
    0x231a..=0x231b,
    0x2329..=0x232a,
    0x23e9..=0x23ec,
    0x23f0..=0x23f0,
    0x23f3..=0x23f3,
    0x25fd..=0x25fe,
    0x2614..=0x2615,
    0x2630..=0x2637,
    0x2648..=0x2653,
    0x267f..=0x267f,
    0x268a..=0x268f,
    0x2693..=0x2693,
    0x26a1..=0x26a1,
    0x26aa..=0x26ab,
    0x26bd..=0x26be,
    0x26c4..=0x26c5,
    0x26ce..=0x26ce,
    0x26d4..=0x26d4,
    0x26ea..=0x26ea,
    0x26f2..=0x26f3,
    0x26f5..=0x26f5,
    0x26fa..=0x26fa,
    0x26fd..=0x26fd,
    0x2705..=0x2705,
    0x270a..=0x270b,
    0x2728..=0x2728,
    0x274c..=0x274c,
    0x274e..=0x274e,
    0x2753..=0x2755,
    0x2757..=0x2757,
    0x2795..=0x2797,
    0x27b0..=0x27b0,
    0x27bf..=0x27bf,
    0x2b1b..=0x2b1c,
    0x2b50..=0x2b50,
    0x2b55..=0x2b55,
    0x2e80..=0x2e99,
    0x2e9b..=0x2ef3,
    0x2f00..=0x2fd5,
    0x2ff0..=0x303e,
    0x3041..=0x3096,
    0x3099..=0x30ff,
    0x3105..=0x312f,
    0x3131..=0x318e,
    0x3190..=0x31e5,
    0x31ef..=0x321e,
    0x3220..=0x3247,
    0x3250..=0xa48c,
    0xa490..=0xa4c6,
    0xa960..=0xa97c,
    0xac00..=0xd7a3,
    0xf900..=0xfaff,
    0xfe10..=0xfe19,
    0xfe30..=0xfe52,
    0xfe54..=0xfe66,
    0xfe68..=0xfe6b,
    0xff01..=0xff60,
    0xffe0..=0xffe6,
    0x16fe0..=0x16fe4,
    0x16ff0..=0x16ff1,
    0x17000..=0x187f7,
    0x18800..=0x18cd5,
    0x18cff..=0x18d08,
    0x1aff0..=0x1aff3,
    0x1aff5..=0x1affb,
    0x1affd..=0x1affe,
    0x1b000..=0x1b122,
    0x1b132..=0x1b132,
    0x1b150..=0x1b152,
    0x1b155..=0x1b155,
    0x1b164..=0x1b167,
    0x1b170..=0x1b2fb,
    0x1d300..=0x1d356,
    0x1d360..=0x1d376,
    0x1f004..=0x1f004,
    0x1f0cf..=0x1f0cf,
    0x1f18e..=0x1f18e,
    0x1f191..=0x1f19a,
    0x1f1e6..=0x1f202,
    0x1f210..=0x1f23b,
    0x1f240..=0x1f248,
    0x1f250..=0x1f251,
    0x1f260..=0x1f265,
    0x1f300..=0x1f320,
    0x1f32d..=0x1f335,
    0x1f337..=0x1f37c,
    0x1f37e..=0x1f393,
    0x1f3a0..=0x1f3ca,
    0x1f3cf..=0x1f3d3,
    0x1f3e0..=0x1f3f0,
    0x1f3f4..=0x1f3f4,
    0x1f3f8..=0x1f43e,
    0x1f440..=0x1f440,
    0x1f442..=0x1f4fc,
    0x1f4ff..=0x1f53d,
    0x1f54b..=0x1f54e,
    0x1f550..=0x1f567,
    0x1f57a..=0x1f57a,
    0x1f595..=0x1f596,
    0x1f5a4..=0x1f5a4,
    0x1f5fb..=0x1f64f,
    0x1f680..=0x1f6c5,
    0x1f6cc..=0x1f6cc,
    0x1f6d0..=0x1f6d2,
    0x1f6d5..=0x1f6d7,
    0x1f6dc..=0x1f6df,
    0x1f6eb..=0x1f6ec,
    0x1f6f4..=0x1f6fc,
    0x1f7e0..=0x1f7eb,
    0x1f7f0..=0x1f7f0,
    0x1f90c..=0x1f93a,
    0x1f93c..=0x1f945,
    0x1f947..=0x1f9ff,
    0x1fa70..=0x1fa7c,
    0x1fa80..=0x1fa89,
    0x1fa8f..=0x1fac6,
    0x1face..=0x1fadc,
    0x1fadf..=0x1fae9,
    0x1faf0..=0x1faf8,
    0x20000..=0x2fffd,
    0x30000..=0x3fffd,
];

/// Check if char `c` is in array of code point ranges using binary search.
fn in_table(arr: &[CodePointRange], c: char) -> bool {
    let c = c as u32;
    arr.binary_search_by(|range| {
        if range.contains(&c) {
            std::cmp::Ordering::Equal
        } else {
            range.start().cmp(&c)
        }
    })
    .is_ok()
}

/// Check if the char `c` has double width.
///
/// Returns `true` if the character is a double-width character according to
/// Unicode East Asian Width specification or is an emoji presentation character.
///
/// # Examples
///
/// ```ignore
/// use clmd::text::unicode_width::is_double_width;
///
/// assert_eq!(is_double_width('✅'), true);
/// assert_eq!(is_double_width('a'), false);
/// assert_eq!(is_double_width('中'), true);
/// ```ignore
pub fn is_double_width(c: char) -> bool {
    // Since ASCII characters are so much more common in English text, check these first
    if in_table(ASCII_TABLE, c) {
        return false;
    }

    if in_table(DOUBLEWIDE_TABLE, c) {
        return true;
    }

    false
}

/// Get the number of columns required to display the grapheme cluster in a monospace font.
///
/// Returns either `1` or `2`.
fn get_grapheme_width(grapheme_cluster: &str) -> u64 {
    for scalar_value in grapheme_cluster.chars() {
        // emoji style variation selector
        if scalar_value == '\u{FE0F}' {
            return 2;
        }

        if is_double_width(scalar_value) {
            return 2;
        }
    }

    1
}

/// Return the number of columns required to display the `text` string in a monospace font.
///
/// This function calculates the display width by iterating over extended grapheme clusters
/// and summing their individual widths. Each grapheme cluster has a width of either 1 or 2.
///
/// Overflow is not realistically possible in this function with `u64` since each operation
/// takes ~20 nanoseconds to complete (~500 years of continuous operation to overflow).
///
/// # Examples
///
/// ```ignore
/// use clmd::text::unicode_width::width;
///
/// assert_eq!(width("hello"), 5);
/// assert_eq!(width("🦀"), 2);
/// assert_eq!(width("👨‍👩‍👧‍👧"), 2);
/// assert_eq!(width("слава україні"), 13);
/// assert_eq!(width("슬라바 우크라이나"), 17);
/// ```ignore
pub fn width(text: &str) -> u64 {
    text.graphemes(true).fold(0, |acc, grapheme_cluster| {
        acc + get_grapheme_width(grapheme_cluster)
    })
}

// =============================================================================
// CJK Spacing Utilities
// =============================================================================

/// Check if spacing is needed between two characters
pub fn needs_spacing(prev: char, next: char) -> bool {
    let prev_is_cjk = is_cjk(prev);
    let next_is_cjk = is_cjk(next);
    let prev_is_ascii_alnum = is_ascii_letter(prev) || is_ascii_digit(prev);
    let next_is_ascii_alnum = is_ascii_letter(next) || is_ascii_digit(next);
    let prev_is_cjk_punct = is_cjk_punctuation(prev);
    let next_is_cjk_punct = is_cjk_punctuation(next);
    let prev_is_ascii_punct_no_space = is_ascii_punctuation_no_space(prev);
    let next_is_ascii_punct_no_space = is_ascii_punctuation_no_space(next);
    let prev_is_closing_bracket = is_closing_bracket(prev);

    // CJK punctuation should NOT have space added after it
    // and should NOT have space added before it
    if prev_is_cjk_punct || next_is_cjk_punct {
        return false;
    }

    // ASCII punctuation like `:`, `,`, `.` should NOT have space added before it
    // This is important for Markdown formatting like `code`: text
    if next_is_ascii_punct_no_space {
        return false;
    }

    // ASCII punctuation like `:`, `,`, `.` should NOT have space added after it
    // when followed by CJK text (but may need space when followed by ASCII)
    if prev_is_ascii_punct_no_space && next_is_cjk {
        return false;
    }

    // Closing brackets (like `)` from links) should have space added after them
    // when followed by ASCII alphanumeric (like English words)
    if prev_is_closing_bracket && next_is_ascii_alnum {
        return true;
    }

    // CJK <-> ASCII alphanumeric needs spacing
    (prev_is_cjk && next_is_ascii_alnum) || (prev_is_ascii_alnum && next_is_cjk)
}

/// Add spaces between CJK and ASCII characters in text
pub fn add_cjk_spacing(text: &str) -> String {
    if text.is_empty() {
        return text.to_string();
    }

    let mut result = String::with_capacity(text.len() * 2);
    let chars: Vec<char> = text.chars().collect();

    for i in 0..chars.len() {
        result.push(chars[i]);

        // Check if we need to add space between current and next character
        if i + 1 < chars.len() {
            let current = chars[i];
            let next = chars[i + 1];

            // Don't add space if either character is already whitespace
            if !current.is_whitespace()
                && !next.is_whitespace()
                && needs_spacing(current, next)
            {
                result.push(' ');
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascii_width() {
        assert_eq!(width("hello"), 5);
        assert_eq!(width("Hello World"), 11);
        assert_eq!(width("12345"), 5);
        assert_eq!(width(""), 0);
    }

    #[test]
    fn test_emoji_width() {
        assert_eq!(width("🦀"), 2);
        assert_eq!(width("✅"), 2);
        assert_eq!(width("🔥"), 2);
    }

    #[test]
    fn test_grapheme_clusters() {
        // Family emoji (multiple code points combined)
        assert_eq!(width("👨‍👩‍👧‍👧"), 2);
        // Woman astronaut with skin tone
        assert_eq!(width("👩🏻‍🚀"), 2);
    }

    #[test]
    fn test_variation_selectors() {
        // Heavy Black Heart without variation selector (width 1)
        assert_eq!(width("\u{2764}"), 1);
        // Heavy Black Heart with emoji style variation selector (width 2)
        assert_eq!(width("\u{2764}\u{FE0F}"), 2);
        assert_eq!(width("❤️"), 2);
    }

    #[test]
    fn test_mixed_content() {
        assert_eq!(width("🔥🗡🍩👩🏻‍🚀⏰💃🏼🔦👍🏻"), 15);
        assert_eq!(width("test test"), 9);
    }

    #[test]
    fn test_is_double_width() {
        assert!(is_double_width('✅'));
        assert!(!is_double_width('a'));
        assert!(is_double_width('中'));
    }

    #[test]
    fn test_unassigned_code_points() {
        // Unassigned code points are assumed to have width 1
        assert_eq!(width("\u{00378}"), 1);
    }

    #[test]
    fn test_private_use_code_points() {
        // Private use code points are assumed to have width 1
        assert_eq!(width("\u{0E000}"), 1);
    }

    // The results of Indic script text may not be useful
    #[test]
    fn test_indic_scripts() {
        assert_eq!(width("ണ്‍"), 1);
        assert_eq!(width("ന്‍"), 1);
        assert_eq!(width("ര്‍"), 1);
    }

    // Edge cases from documentation analysis

    #[test]
    fn test_zalgo_text() {
        // Corrupted text with many combining marks
        // Each base character is width 1, combining marks don't add width
        let zalgo = "Ẓ̌á̲l͔̝̞̄̑͌g̖̘̘̔̔͢͞͝o̪̔T̢̙̫̈̍͞e̬͈͕͌̏͑x̺̍ṭ̓̓ͅ";
        assert_eq!(width(zalgo), 9);
    }

    #[test]
    fn test_emoji_vs_text_style() {
        // Emoji with text style variation selector (VS15) should be width 1
        assert_eq!(width("\u{2764}\u{FE0E}"), 1);
        // Emoji with emoji style variation selector (VS16) should be width 2
        assert_eq!(width("\u{2764}\u{FE0F}"), 2);
    }

    #[test]
    fn test_single_width_emojis() {
        // Some emojis are single width
        assert_eq!(width("🗡"), 1); // Dagger emoji
    }

    #[test]
    fn test_scientist_emoji() {
        // Woman scientist emoji (combination of 👩 + 🔬)
        assert_eq!(width("👩‍🔬"), 2);
    }

    #[test]
    fn test_flag_emojis() {
        // Country flag emojis (regional indicator symbols)
        assert_eq!(width("🇺🇸"), 2);
        assert_eq!(width("🇨🇳"), 2);
        assert_eq!(width("🇯🇵"), 2);
    }

    #[test]
    fn test_skin_tone_modifiers() {
        // Emojis with skin tone modifiers
        assert_eq!(width("👍"), 2);
        assert_eq!(width("👍🏻"), 2);
        assert_eq!(width("👍🏿"), 2);
    }

    #[test]
    fn test_gender_modifiers() {
        // Emojis with gender modifiers
        assert_eq!(width("🧑‍⚕️"), 2); // Health worker
        assert_eq!(width("👨‍⚕️"), 2); // Man health worker
        assert_eq!(width("👩‍⚕️"), 2); // Woman health worker
    }

    #[test]
    fn test_fullwidth_ascii() {
        // Fullwidth ASCII characters (width 2)
        assert_eq!(width("Ａ"), 2);
        assert_eq!(width("Ｂ"), 2);
        assert_eq!(width("１"), 2);
        assert_eq!(width("＠"), 2);
    }

    #[test]
    fn test_halfwidth_katakana() {
        // Halfwidth Katakana (width 1)
        assert_eq!(width("ｱ"), 1);
        assert_eq!(width("ｶ"), 1);
    }

    #[test]
    fn test_hangul_syllables() {
        // Precomposed Hangul syllables (width 2)
        assert_eq!(width("가"), 2);
        assert_eq!(width("힣"), 2);
    }

    #[test]
    fn test_hangul_jamo() {
        // Hangul Jamo (width 2)
        assert_eq!(width("ㄱ"), 2);
        assert_eq!(width("ㅏ"), 2);
    }

    #[test]
    fn test_cjk_punctuation() {
        // CJK punctuation (width 2)
        assert_eq!(width("。"), 2);
        assert_eq!(width("、"), 2);
        assert_eq!(width("「"), 2);
        assert_eq!(width("」"), 2);
    }

    #[test]
    fn test_control_characters() {
        // Control characters are treated as regular characters (width 1 each)
        // This is the behavior of unicode-segmentation treating them as graphemes
        assert_eq!(width("\n"), 1);
        assert_eq!(width("\t"), 1);
        assert_eq!(width("\r"), 1);
    }

    #[test]
    fn test_zero_width_joiner() {
        // Zero width joiner alone - treated as width 1 by current implementation
        // In practice, ZWJ is always used with other characters
        assert_eq!(width("\u{200D}"), 1);
    }

    #[test]
    fn test_zero_width_non_joiner() {
        // Zero width non-joiner - treated as width 1 by current implementation
        assert_eq!(width("\u{200C}"), 1);
    }

    #[test]
    fn test_combining_characters() {
        // Combining characters (should not add width when following base)
        assert_eq!(width("e\u{0301}"), 1); // e + combining acute
        assert_eq!(width("a\u{0300}"), 1); // a + combining grave
    }

    #[test]
    fn test_mixed_cjk_and_ascii() {
        // Mixed CJK and ASCII
        assert_eq!(width("Hello世界"), 9); // 5 + 4
        assert_eq!(width("Test测试123"), 11); // 4 + 4 + 3
    }

    #[test]
    fn test_mathematical_symbols() {
        // Mathematical alphanumeric symbols (width 1 or 2 depending)
        assert_eq!(width("𝐀"), 1); // Mathematical bold A
        assert_eq!(width("𝐴"), 1); // Mathematical italic A
    }

    #[test]
    fn test_box_drawing() {
        // Box drawing characters (width 1)
        assert_eq!(width("┌"), 1);
        assert_eq!(width("─"), 1);
        assert_eq!(width("┐"), 1);
    }

    #[test]
    fn test_block_elements() {
        // Block elements (width 1)
        assert_eq!(width("█"), 1);
        assert_eq!(width("▓"), 1);
        assert_eq!(width("░"), 1);
    }

    #[test]
    fn test_arabic_text() {
        // Arabic text (width 1)
        assert_eq!(width("مرحبا"), 5);
    }

    #[test]
    fn test_hebrew_text() {
        // Hebrew text (width 1)
        assert_eq!(width("שלום"), 4);
    }

    #[test]
    fn test_thai_text() {
        // Thai text - some characters may be combined into single graphemes
        // สวัสดี has combining characters that form fewer graphemes
        assert_eq!(width("สวัสดี"), 4);
    }

    #[test]
    fn test_russian_text() {
        // Cyrillic text (width 1)
        assert_eq!(width("Привет"), 6);
    }

    #[test]
    fn test_greek_text() {
        // Greek text (width 1)
        assert_eq!(width("Γειά"), 4);
    }

    #[test]
    fn test_devanagari() {
        // Devanagari - some characters combine into single graphemes
        // नमस्ते has combining marks that reduce the grapheme count
        assert_eq!(width("नमस्ते"), 3);
    }

    #[test]
    fn test_newlines_and_whitespace() {
        // Newlines and various whitespace - control chars are treated as width 1
        assert_eq!(width("hello\nworld"), 11); // 5 + 1 + 5
        assert_eq!(width("hello\tworld"), 11); // 5 + 1 + 5
        assert_eq!(width("hello world"), 11); // 5 + 1 + 5
    }

    #[test]
    fn test_empty_and_whitespace_only() {
        assert_eq!(width(""), 0);
        assert_eq!(width("   "), 3);
        assert_eq!(width("\t\n\r"), 3); // Each control char is width 1
    }

    #[test]
    fn test_very_long_string() {
        // Test with a very long string to ensure no overflow
        let long_string = "a".repeat(10000);
        assert_eq!(width(&long_string), 10000);
    }

    #[test]
    fn test_repeated_emojis() {
        // Repeated emojis
        assert_eq!(width("🦀🦀🦀"), 6);
        assert_eq!(width("🔥🔥🔥🔥🔥"), 10);
    }

    #[test]
    fn test_mixed_emoji_and_text() {
        // Mixed emoji and text
        assert_eq!(width("Hello 🦀 World"), 14); // 5 + 1 + 2 + 1 + 5
                                                 // Rust🦀is🔥awesome: R-u-s-t-🦀-i-s-🔥-a-w-e-s-o-m-e
                                                 // 1+1+1+1 + 2 + 1+1 + 2 + 1+1+1+1+1+1+1 = 17
        assert_eq!(width("Rust🦀is🔥awesome"), 17);
    }

    #[test]
    fn test_unicode_version_coverage() {
        // Test characters from various Unicode versions
        // Unicode 1.1
        assert_eq!(width("©"), 1);
        // Unicode 6.0
        assert_eq!(width("🀄"), 2);
        // Unicode 9.0
        assert_eq!(width("🤣"), 2);
        // Unicode 15.0
        assert_eq!(width("🫨"), 2);
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

    // =============================================================================
    // CJK Spacing Tests
    // =============================================================================

    #[test]
    fn test_needs_spacing() {
        assert!(needs_spacing('中', 'a'));
        assert!(needs_spacing('a', '中'));
        assert!(needs_spacing('中', '1'));
        assert!(needs_spacing('1', '中'));
        assert!(!needs_spacing('中', '文'));
        assert!(!needs_spacing('a', 'b'));
        assert!(!needs_spacing('1', '2'));
    }

    #[test]
    fn test_add_cjk_spacing() {
        assert_eq!(add_cjk_spacing("中文test"), "中文 test");
        assert_eq!(add_cjk_spacing("test中文"), "test 中文");
        assert_eq!(add_cjk_spacing("数字123"), "数字 123");
        assert_eq!(add_cjk_spacing("123数字"), "123 数字");
        assert_eq!(add_cjk_spacing("中文 test"), "中文 test"); // Already has space

        // Test that trailing space is preserved
        assert_eq!(add_cjk_spacing("This is "), "This is ");
        assert_eq!(add_cjk_spacing("hello "), "hello ");
    }

    #[test]
    fn test_cjk_punctuation_spacing() {
        // CJK punctuation should NOT have space added after it
        assert_eq!(add_cjk_spacing("示例，包含"), "示例，包含");
        assert_eq!(add_cjk_spacing("测试。通过"), "测试。通过");
        assert!(!needs_spacing('，', '包'));
        assert!(!needs_spacing('。', '测'));
    }

    #[test]
    fn test_cjk_punctuation_with_ascii() {
        // CJK punctuation should NOT have space added before or after ASCII characters
        // This is important for Markdown formatting like **特性**：
        assert!(
            !needs_spacing('：', '*'),
            "CJK colon should not have space before asterisk"
        );
        assert!(
            !needs_spacing('*', '：'),
            "Asterisk should not have space before CJK colon"
        );
        assert!(
            !needs_spacing('，', '*'),
            "CJK comma should not have space before asterisk"
        );
        assert!(
            !needs_spacing('*', '，'),
            "Asterisk should not have space before CJK comma"
        );
        assert!(
            !needs_spacing('。', 'a'),
            "CJK period should not have space before ASCII letter"
        );
        assert!(
            !needs_spacing('a', '。'),
            "ASCII letter should not have space before CJK period"
        );

        // Verify the actual spacing behavior
        assert_eq!(add_cjk_spacing("特性："), "特性：");
        assert_eq!(add_cjk_spacing("：test"), "：test");

        // Test Markdown markers with CJK punctuation
        assert_eq!(add_cjk_spacing("**特性**:"), "**特性**:");
        assert_eq!(add_cjk_spacing("**特性**："), "**特性**：");
    }
}
