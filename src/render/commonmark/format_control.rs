//! Format control processor
//!
//! This module provides functionality to control formatting via HTML comments,
//! inspired by flexmark-java's FormatControlProcessor.
//!
//! Format control comments allow users to disable and re-enable formatting
//! for specific regions of a document:
//! - `<!-- @formatter:off -->` - Disable formatting
//! - `<!-- @formatter:on -->` - Re-enable formatting

use super::options::FormatterOptions;
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
    pub fn new(options: &FormatterOptions) -> Self {
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

    /// Check if formatting is currently on
    pub fn is_formatting_on(&self) -> bool {
        !self.formatting_off
    }

    /// Check if formatting was just turned off
    ///
    /// This returns true only once after formatting is turned off,
    /// then resets to false.
    pub fn is_just_turned_off(&self) -> bool {
        self.just_turned_off
    }

    /// Check if formatting was just turned on
    ///
    /// This returns true only once after formatting is turned on,
    /// then resets to false.
    pub fn is_just_turned_on(&self) -> bool {
        self.just_turned_on
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

    /// Get the formatter on tag
    pub fn get_formatter_on_tag(&self) -> &str {
        &self.formatter_on_tag
    }

    /// Get the formatter off tag
    pub fn get_formatter_off_tag(&self) -> &str {
        &self.formatter_off_tag
    }

    /// Check if formatter tags are enabled
    pub fn are_formatter_tags_enabled(&self) -> bool {
        self.formatter_tags_enabled
    }

    /// Check if formatter tags accept regex
    pub fn are_formatter_tags_regex_enabled(&self) -> bool {
        self.formatter_tags_accept_regex
    }

    /// Reset the formatting state to on
    pub fn reset(&mut self) {
        self.formatting_off = false;
        self.just_turned_off = false;
        self.just_turned_on = false;
    }

    /// Clear the just-turned flags
    pub fn clear_just_turned_flags(&mut self) {
        self.just_turned_off = false;
        self.just_turned_on = false;
    }

    /// Check if a node is in a formatting region (not in a format-off area)
    ///
    /// This method checks the document structure to determine if the given
    /// content should be formatted based on preceding format control comments.
    pub fn is_formatting_region(&self, _content: &str) -> bool {
        // For now, this is a simplified implementation
        // A full implementation would need to track the position in the document
        !self.formatting_off
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_options() -> FormatterOptions {
        FormatterOptions::default()
    }

    #[test]
    fn test_default_tags() {
        let options = create_test_options();
        let processor = FormatControlProcessor::new(&options);

        assert!(processor.are_formatter_tags_enabled());
        assert_eq!(processor.get_formatter_on_tag(), "formatter:on");
        assert_eq!(processor.get_formatter_off_tag(), "formatter:off");
    }

    #[test]
    fn test_process_formatter_off_comment() {
        let options = create_test_options();
        let mut processor = FormatControlProcessor::new(&options);

        assert!(processor.is_formatting_on());

        let result = processor.process_comment("<!-- formatter:off -->");
        assert!(result);
        assert!(processor.is_formatting_off());
        assert!(processor.is_just_turned_off());
        assert!(!processor.is_just_turned_on());
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
        assert!(processor.is_formatting_on());
        assert!(processor.is_just_turned_on());
        assert!(!processor.is_just_turned_off());
    }

    #[test]
    fn test_process_non_control_comment() {
        let options = create_test_options();
        let mut processor = FormatControlProcessor::new(&options);

        let result = processor.process_comment("<!-- This is a regular comment -->");
        assert!(!result);
        assert!(processor.is_formatting_on());
    }

    #[test]
    fn test_process_invalid_comment() {
        let options = create_test_options();
        let mut processor = FormatControlProcessor::new(&options);

        let result = processor.process_comment("This is not a comment");
        assert!(!result);
    }

    #[test]
    fn test_reset() {
        let options = create_test_options();
        let mut processor = FormatControlProcessor::new(&options);

        processor.process_comment("<!-- formatter:off -->");
        assert!(processor.is_formatting_off());

        processor.reset();
        assert!(processor.is_formatting_on());
        assert!(!processor.is_just_turned_off());
        assert!(!processor.is_just_turned_on());
    }

    #[test]
    fn test_disabled_tags() {
        let mut options = create_test_options();
        options.formatter_tags_enabled = false;

        let mut processor = FormatControlProcessor::new(&options);

        let result = processor.process_comment("<!-- formatter:off -->");
        assert!(!result);
        assert!(processor.is_formatting_on());
    }
}
