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
//! ```
//! use clmd::unicode_width::width;
//!
//! assert_eq!(width("hello"), 5);
//! assert_eq!(width("🦀"), 2);
//! assert_eq!(width("👨‍👩‍👧‍👧"), 2);
//! assert_eq!(width("中文"), 4);
//! ```

use unicode_segmentation::UnicodeSegmentation;

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
/// ```
/// use clmd::unicode_width::is_double_width;
///
/// assert_eq!(is_double_width('✅'), true);
/// assert_eq!(is_double_width('a'), false);
/// assert_eq!(is_double_width('中'), true);
/// ```
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
/// ```
/// use clmd::unicode_width::width;
///
/// assert_eq!(width("hello"), 5);
/// assert_eq!(width("🦀"), 2);
/// assert_eq!(width("👨‍👩‍👧‍👧"), 2);
/// assert_eq!(width("слава україні"), 13);
/// assert_eq!(width("슬라바 우크라이나"), 17);
/// ```
pub fn width(text: &str) -> u64 {
    text.graphemes(true).fold(0, |acc, grapheme_cluster| {
        acc + get_grapheme_width(grapheme_cluster)
    })
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
    fn test_cjk_width() {
        assert_eq!(width("中"), 2);
        assert_eq!(width("中文"), 4);
        assert_eq!(width("日本語"), 6);
        assert_eq!(width("한국어"), 6);
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
    fn test_ukrainian() {
        assert_eq!(width("слава україні"), 13);
    }

    #[test]
    fn test_korean() {
        assert_eq!(width("슬라바 우크라이나"), 17);
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
}
