//! Format control processor
//!
//! This module provides functionality to control formatting via HTML comments,
//! inspired by flexmark-java's FormatControlProcessor.
//!
//! Format control comments allow users to disable and re-enable formatting
//! for specific regions of a document:
//! - `<!-- @formatter:off -->` - Disable formatting
//! - `<!-- @formatter:on -->` - Re-enable formatting

use crate::options::format::FormatOptions;
use regex::Regex;

/// Processor for format control comments
///
/// This processor handles HTML comments that control whether formatting
/// should be applied to specific regions of the document.
#[derive(Debug, Clone)]
pub struct FormatControlProcessor {
    /// The tag that turns formatting on
    formatter_on_tag: String,
    /// The tag that turns formatting off
    formatter_off_tag: String,
    /// Whether format control tags are enabled
    formatter_tags_enabled: bool,
    /// Whether to accept regular expressions for formatter tags
    formatter_tags_accept_regex: bool,
    /// Compiled regex for formatter on tag (if regex mode is enabled)
    formatter_on_regex: Option<Regex>,
    /// Compiled regex for formatter off tag (if regex mode is enabled)
    formatter_off_regex: Option<Regex>,
    /// Current state: whether formatting is off
    formatting_off: bool,
    /// Whether formatting was just turned off (for one-time detection)
    just_turned_off: bool,
    /// Whether formatting was just turned on (for one-time detection)
    just_turned_on: bool,
}

impl FormatControlProcessor {
    /// Create a new format control processor from options
    pub fn new(options: &FormatOptions) -> Self {
        let formatter_on_regex =
            if options.formatter_tags_enabled && options.formatter_tags_accept_regex {
                Regex::new(&options.formatter_on_tag).ok()
            } else {
                None
            };

        let formatter_off_regex =
            if options.formatter_tags_enabled && options.formatter_tags_accept_regex {
                Regex::new(&options.formatter_off_tag).ok()
            } else {
                None
            };

        Self {
            formatter_on_tag: options.formatter_on_tag.clone(),
            formatter_off_tag: options.formatter_off_tag.clone(),
            formatter_tags_enabled: options.formatter_tags_enabled,
            formatter_tags_accept_regex: options.formatter_tags_accept_regex,
            formatter_on_regex,
            formatter_off_regex,
            formatting_off: false,
            just_turned_off: false,
            just_turned_on: false,
        }
    }

    /// Check if formatting is currently off
    pub fn is_formatting_off(&self) -> bool {
        self.formatting_off
    }

    /// Process an HTML comment and update formatting state
    ///
    /// Returns true if the comment was a format control comment.
    pub fn process_comment(&mut self, comment_text: &str) -> bool {
        // Reset the just-turned flags
        self.just_turned_off = false;
        self.just_turned_on = false;

        if !self.formatter_tags_enabled {
            return false;
        }

        // Extract the content between <!-- and -->
        let trimmed = comment_text.trim();
        if !trimmed.starts_with("<!--") || !trimmed.ends_with("-->") {
            return false;
        }

        let content = trimmed[4..trimmed.len() - 3].trim();

        // Check using regex if enabled and compiled successfully
        if self.formatter_tags_accept_regex {
            if let Some(ref on_regex) = self.formatter_on_regex {
                if on_regex.is_match(content) {
                    if self.formatting_off {
                        self.formatting_off = false;
                        self.just_turned_on = true;
                    }
                    return true;
                }
            }

            if let Some(ref off_regex) = self.formatter_off_regex {
                if off_regex.is_match(content) {
                    if !self.formatting_off {
                        self.formatting_off = true;
                        self.just_turned_off = true;
                    }
                    return true;
                }
            }
        }

        // Check for exact match with formatter off tag
        if content == self.formatter_off_tag {
            if !self.formatting_off {
                self.formatting_off = true;
                self.just_turned_off = true;
            }
            return true;
        }

        // Check for exact match with formatter on tag
        if content == self.formatter_on_tag {
            if self.formatting_off {
                self.formatting_off = false;
                self.just_turned_on = true;
            }
            return true;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_options() -> FormatOptions {
        FormatOptions::default()
    }

    #[test]
    fn test_process_formatter_off_comment() {
        let options = create_test_options();
        let mut processor = FormatControlProcessor::new(&options);

        let result = processor.process_comment("<!-- formatter:off -->");
        assert!(result);
        assert!(processor.is_formatting_off());
    }

    #[test]
    fn test_process_formatter_on_comment() {
        let options = create_test_options();
        let mut processor = FormatControlProcessor::new(&options);

        // First turn off
        processor.process_comment("<!-- formatter:off -->");
        assert!(processor.is_formatting_off());

        // Then turn on
        let result = processor.process_comment("<!-- formatter:on -->");
        assert!(result);
        assert!(!processor.is_formatting_off());
    }

    #[test]
    fn test_process_non_control_comment() {
        let options = create_test_options();
        let mut processor = FormatControlProcessor::new(&options);

        let result = processor.process_comment("<!-- This is a regular comment -->");
        assert!(!result);
        assert!(!processor.is_formatting_off());
    }

    #[test]
    fn test_process_invalid_comment() {
        let options = create_test_options();
        let mut processor = FormatControlProcessor::new(&options);

        let result = processor.process_comment("This is not a comment");
        assert!(!result);
    }

    #[test]
    fn test_disabled_tags() {
        let mut options = create_test_options();
        options.formatter_tags_enabled = false;

        let mut processor = FormatControlProcessor::new(&options);

        let result = processor.process_comment("<!-- formatter:off -->");
        assert!(!result);
        assert!(!processor.is_formatting_off());
    }

    #[test]
    fn test_double_turn_off() {
        let options = create_test_options();
        let mut processor = FormatControlProcessor::new(&options);

        // First turn off
        processor.process_comment("<!-- formatter:off -->");
        assert!(processor.is_formatting_off());

        // Second turn off - should not change state
        processor.process_comment("<!-- formatter:off -->");
        assert!(processor.is_formatting_off());
    }

    #[test]
    fn test_double_turn_on() {
        let options = create_test_options();
        let mut processor = FormatControlProcessor::new(&options);

        // Turn off first
        processor.process_comment("<!-- formatter:off -->");

        // First turn on
        processor.process_comment("<!-- formatter:on -->");
        assert!(!processor.is_formatting_off());

        // Second turn on - should not change state
        processor.process_comment("<!-- formatter:on -->");
        assert!(!processor.is_formatting_off());
    }

    #[test]
    fn test_comment_with_whitespace() {
        let options = create_test_options();
        let mut processor = FormatControlProcessor::new(&options);

        // Comment with extra whitespace
        let result = processor.process_comment("  <!--   formatter:off   -->  ");
        assert!(result);
        assert!(processor.is_formatting_off());
    }

    #[test]
    fn test_invalid_comment_format() {
        let options = create_test_options();
        let mut processor = FormatControlProcessor::new(&options);

        // Missing closing --
        let result = processor.process_comment("<!-- formatter:off >");
        assert!(!result);

        // Missing opening <!--
        let result = processor.process_comment("formatter:off -->");
        assert!(!result);

        // No comment markers at all
        let result = processor.process_comment("formatter:off");
        assert!(!result);
    }

    #[test]
    fn test_consecutive_comments() {
        let options = create_test_options();
        let mut processor = FormatControlProcessor::new(&options);

        // First comment - turn off
        let result1 = processor.process_comment("<!-- formatter:off -->");
        assert!(result1);
        assert!(processor.is_formatting_off());

        // Second comment - not a control comment
        let result2 = processor.process_comment("<!-- regular comment -->");
        assert!(!result2);
        assert!(processor.is_formatting_off()); // State unchanged

        // Third comment - turn on
        let result3 = processor.process_comment("<!-- formatter:on -->");
        assert!(result3);
        assert!(!processor.is_formatting_off());
    }
}
