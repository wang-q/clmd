//! CJK spacing utilities
//!
//! This module provides functionality to add spaces between CJK (Chinese, Japanese, Korean)
//! characters and ASCII letters/numbers for better typography.

use crate::text::char::{
    is_ascii_digit, is_ascii_letter, is_ascii_punctuation_no_space, is_cjk_punctuation,
    is_closing_bracket,
};
use crate::text::unicode::is_cjk;

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
