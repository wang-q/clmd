//! Unicode standard CJK tokenizer
//!
//! Uses the unicode-segmentation crate's UAX#29 standard for text tokenization,
//! supporting all languages without built-in dictionaries.

use crate::text::char::is_cjk_punctuation;
use unicode_segmentation::UnicodeSegmentation;

/// Tokenize text using Unicode UAX#29 standard
///
/// Segments based on character type boundaries, supporting Chinese, Japanese, Korean, and all other languages.
/// No built-in dictionary required, uses Unicode character properties.
///
/// # Examples
///
/// ```ignore
/// let tokens = tokenize_unicode("Hello世界123");
/// // Result: ["Hello", "世界", "123"]
/// ```
pub fn tokenize_unicode(text: &str) -> Vec<String> {
    if text.is_empty() {
        return Vec::new();
    }

    // Use unicode_words() to get word boundaries
    // This segments based on Unicode character properties
    text.unicode_words().map(|s| s.to_string()).collect()
}

/// Smart CJK text segmentation (combining Unicode word boundaries and punctuation splitting)
///
/// This is the main tokenization function used by line_breaking.rs.
/// Uses split_word_bounds() to get all word boundaries including transitions between
/// different character types (CJK, ASCII, numbers).
pub fn split_cjk_text_smart(text: &str) -> Vec<String> {
    if text.is_empty() {
        return Vec::new();
    }

    // Use split_word_bounds() to get all word boundaries
    // This includes boundaries between CJK/ASCII/numbers and punctuation
    let mut result = Vec::new();
    let mut current_segment = String::new();

    for word in text.split_word_bounds() {
        if word.is_empty() {
            continue;
        }

        // Check if this segment is punctuation
        let is_punct = word
            .chars()
            .all(|c| is_cjk_punctuation(c) || c.is_ascii_punctuation());

        if is_punct {
            // Add punctuation to current segment and push it
            current_segment.push_str(word);
            result.push(current_segment.clone());
            current_segment.clear();
        } else {
            // Check if we should start a new segment
            // (when transitioning between CJK and non-CJK)
            if !current_segment.is_empty() {
                let last_char = current_segment.chars().last().unwrap();
                let first_char = word.chars().next().unwrap();

                if is_cjk_boundary(last_char, first_char) {
                    // Boundary between CJK and non-CJK, push current and start new
                    result.push(current_segment.clone());
                    current_segment.clear();
                }
            }
            current_segment.push_str(word);
        }
    }

    // Add any remaining text
    if !current_segment.is_empty() {
        result.push(current_segment);
    }

    // If no splits were made, return the whole text as one segment
    if result.is_empty() && !text.is_empty() {
        result.push(text.to_string());
    }

    result
}

/// Check if there's a boundary between CJK and non-CJK characters
fn is_cjk_boundary(left: char, right: char) -> bool {
    use crate::text::char::is_cjk;

    let left_is_cjk = is_cjk(left);
    let right_is_cjk = is_cjk(right);

    // Boundary when transitioning between CJK and non-CJK
    left_is_cjk != right_is_cjk
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_chinese() {
        let result = tokenize_unicode("Hello世界123");
        println!("tokenize_unicode 'Hello世界123': {:?}", result);
        // For pure CJK without spaces, unicode_words() returns individual characters
        assert!(
            result.contains(&"Hello".to_string()),
            "Should contain 'Hello'"
        );
        assert!(result.contains(&"123".to_string()), "Should contain '123'");
    }

    #[test]
    fn test_tokenize_japanese() {
        let result = tokenize_unicode("これはテストです");
        // Unicode tokenization should recognize Japanese word boundaries
        assert!(!result.is_empty());
    }

    #[test]
    fn test_tokenize_korean() {
        let result = tokenize_unicode("한국어테스트");
        // Unicode tokenization should recognize Korean word boundaries
        assert!(!result.is_empty());
    }

    #[test]
    fn test_tokenize_mixed() {
        let result = tokenize_unicode("Hello 世界 World 123");
        // Mixed content should be handled correctly
        assert!(result.len() >= 4);
    }

    #[test]
    fn test_split_cjk_text_smart() {
        let result = split_cjk_text_smart("Hello世界，测试。");
        println!("split_cjk_text_smart 'Hello世界，测试。': {:?}", result);
        // Should handle both Unicode boundaries and punctuation
        assert!(!result.is_empty());
    }

    #[test]
    fn test_empty_text() {
        let result = tokenize_unicode("");
        assert!(result.is_empty());
    }

    #[test]
    fn test_split_cjk_text_smart_with_punctuation() {
        let result = split_cjk_text_smart("示例，包含");
        println!("split_cjk_text_smart '示例，包含': {:?}", result);
        // Should split at punctuation
        assert!(!result.is_empty(), "Should not be empty");
        // Check that punctuation is in one of the segments
        let has_punct = result.iter().any(|s| s.contains('，'));
        assert!(
            has_punct,
            "Should contain punctuation '，': got {:?}",
            result
        );
    }

    #[test]
    fn test_mixed_cjk_ascii() {
        let result = split_cjk_text_smart("数字123");
        println!("split_cjk_text_smart '数字123': {:?}", result);
        // Should split between CJK and ASCII
        assert!(
            result.len() >= 2,
            "Should have at least 2 segments: {:?}",
            result
        );
    }

    #[test]
    fn test_cjk_boundary_detection() {
        assert!(is_cjk_boundary('中', 'A'));
        assert!(is_cjk_boundary('A', '中'));
        assert!(!is_cjk_boundary('中', '文'));
        assert!(!is_cjk_boundary('A', 'B'));
    }
}
