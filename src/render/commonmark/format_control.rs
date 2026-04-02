//! Format control processor for Markdown formatter
//!
//! This module provides support for formatter control comments, inspired by
//! flexmark-java's FormatControlProcessor.
//!
//! Format control comments allow users to disable and re-enable formatting
//! for specific sections of a document:
//!
//! ```markdown
//! <!-- formatter:off -->
//! This section will not be formatted
//! <!-- formatter:on -->
//! ```
//!
//! The tags can be customized via formatter options, and regular expressions
//! can be used for more flexible matching.

use crate::formatter::options::FormatterOptions;
use regex::Regex;

/// Default formatter on tag
pub const DEFAULT_FORMATTER_ON_TAG: &str = "formatter:on";

/// Default formatter off tag
pub const DEFAULT_FORMATTER_OFF_TAG: &str = "formatter:off";

/// HTML comment open marker
pub const HTML_COMMENT_OPEN: &str = "<!--";

/// HTML comment close marker
pub const HTML_COMMENT_CLOSE: &str = "-->";

/// Processor for format control comments
///
/// Tracks the formatting state based on HTML comments in the document.
#[derive(Debug, Clone)]
pub struct FormatControlProcessor {
    /// The tag that enables formatting
    formatter_on_tag: String,
    /// The tag that disables formatting
    formatter_off_tag: String,
    /// Whether format control tags are enabled
    tags_enabled: bool,
    /// Whether to accept regular expressions as tags
    accept_regex: bool,
    /// Current formatting state (true = off, false = on)
    formatter_off: bool,
    /// Whether formatting was just turned off
    just_turned_off: bool,
    /// Whether formatting was just turned on
    just_turned_on: bool,
    /// Compiled regex for off tag (if regex mode enabled)
    off_regex: Option<Regex>,
    /// Compiled regex for on tag (if regex mode enabled)
    on_regex: Option<Regex>,
}

impl FormatControlProcessor {
    /// Create a new format control processor with the given options
    pub fn new(options: &FormatterOptions) -> Self {
        let formatter_on_tag = options.formatter_on_tag.clone();
        let formatter_off_tag = options.formatter_off_tag.clone();
        let tags_enabled = options.formatter_tags_enabled;
        let accept_regex = options.formatter_tags_accept_regex;

        let mut processor = Self {
            formatter_on_tag,
            formatter_off_tag,
            tags_enabled,
            accept_regex,
            formatter_off: false,
            just_turned_off: false,
            just_turned_on: false,
            off_regex: None,
            on_regex: None,
        };

        // Compile regex patterns if regex mode is enabled
        if tags_enabled && accept_regex {
            processor.compile_regex_patterns();
        }

        processor
    }

    /// Compile regex patterns for tag matching
    fn compile_regex_patterns(&mut self) {
        // Try to compile the off tag pattern
        if let Ok(regex) = Regex::new(&self.formatter_off_tag) {
            self.off_regex = Some(regex);
        }

        // Try to compile the on tag pattern
        if let Ok(regex) = Regex::new(&self.formatter_on_tag) {
            self.on_regex = Some(regex);
        }
    }

    /// Check if formatting is currently disabled
    pub fn is_formatting_off(&self) -> bool {
        self.formatter_off
    }

    /// Check if formatting is currently enabled
    pub fn is_formatting_on(&self) -> bool {
        !self.formatter_off
    }

    /// Check if formatting was just turned off in the last processed comment
    pub fn just_turned_off(&self) -> bool {
        self.just_turned_off
    }

    /// Check if formatting was just turned on in the last processed comment
    pub fn just_turned_on(&self) -> bool {
        self.just_turned_on
    }

    /// Get the formatter on tag
    pub fn formatter_on_tag(&self) -> &str {
        &self.formatter_on_tag
    }

    /// Get the formatter off tag
    pub fn formatter_off_tag(&self) -> &str {
        &self.formatter_off_tag
    }

    /// Check if format control tags are enabled
    pub fn tags_enabled(&self) -> bool {
        self.tags_enabled
    }

    /// Check if regex patterns are accepted
    pub fn accept_regex(&self) -> bool {
        self.accept_regex
    }

    /// Process a potential format control comment
    ///
    /// This method should be called for each HTML comment encountered during formatting.
    /// It updates the internal state based on the comment content.
    ///
    /// # Arguments
    ///
    /// * `comment_text` - The full text of the HTML comment (including `<!--` and `-->`)
    ///
    /// # Returns
    ///
    /// `true` if the comment was a format control comment, `false` otherwise.
    pub fn process_comment(&mut self, comment_text: &str) -> bool {
        self.just_turned_off = false;
        self.just_turned_on = false;

        if !self.tags_enabled {
            return false;
        }

        // Extract the content between <!-- and -->
        let content_opt = extract_comment_content(comment_text);
        if content_opt.is_none() {
            return false;
        }
        let content = content_opt.unwrap();
        let content = content.trim();

        // Check if this is a format control tag
        let is_off = self.is_formatter_off_tag(content);
        let is_on = self.is_formatter_on_tag(content);

        if is_off {
            if !self.formatter_off {
                self.just_turned_off = true;
            }
            self.formatter_off = true;
            true
        } else if is_on {
            if self.formatter_off {
                self.just_turned_on = true;
            }
            self.formatter_off = false;
            true
        } else {
            false
        }
    }

    /// Check if the given content matches the formatter off tag
    fn is_formatter_off_tag(&self, content: &str) -> bool {
        if self.accept_regex && self.off_regex.is_some() {
            self.off_regex.as_ref().unwrap().is_match(content)
        } else {
            content == self.formatter_off_tag
        }
    }

    /// Check if the given content matches the formatter on tag
    fn is_formatter_on_tag(&self, content: &str) -> bool {
        if self.accept_regex && self.on_regex.is_some() {
            self.on_regex.as_ref().unwrap().is_match(content)
        } else {
            content == self.formatter_on_tag
        }
    }

    /// Reset the processor to its initial state
    pub fn reset(&mut self) {
        self.formatter_off = false;
        self.just_turned_off = false;
        self.just_turned_on = false;
    }
}

/// Extract the content from an HTML comment
///
/// Returns `None` if the text is not a valid HTML comment.
fn extract_comment_content(comment_text: &str) -> Option<String> {
    let trimmed = comment_text.trim();

    if !trimmed.starts_with(HTML_COMMENT_OPEN) || !trimmed.ends_with(HTML_COMMENT_CLOSE) {
        return None;
    }

    let start = HTML_COMMENT_OPEN.len();
    let end = trimmed.len() - HTML_COMMENT_CLOSE.len();

    if start > end {
        return None;
    }

    Some(trimmed[start..end].to_string())
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

        assert!(processor.tags_enabled());
        assert_eq!(processor.formatter_on_tag(), DEFAULT_FORMATTER_ON_TAG);
        assert_eq!(processor.formatter_off_tag(), DEFAULT_FORMATTER_OFF_TAG);
    }

    #[test]
    fn test_process_formatter_off_comment() {
        let options = create_test_options();
        let mut processor = FormatControlProcessor::new(&options);

        assert!(processor.is_formatting_on());

        let result = processor.process_comment("<!-- formatter:off -->");
        assert!(result);
        assert!(processor.is_formatting_off());
        assert!(processor.just_turned_off());
        assert!(!processor.just_turned_on());
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
        assert!(processor.just_turned_on());
        assert!(!processor.just_turned_off());
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
        assert!(!processor.just_turned_off());
        assert!(!processor.just_turned_on());
    }

    #[test]
    fn test_extract_comment_content() {
        assert_eq!(
            extract_comment_content("<!-- formatter:off -->"),
            Some(" formatter:off ".to_string())
        );
        assert_eq!(
            extract_comment_content("<!--formatter:on-->"),
            Some("formatter:on".to_string())
        );
        assert_eq!(
            extract_comment_content("Not a comment"),
            None
        );
        assert_eq!(
            extract_comment_content("<!-- incomplete"),
            None
        );
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
