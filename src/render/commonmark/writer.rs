//! Markdown writer for formatted output
//!
//! This module provides a writer for generating Markdown output with
//! proper prefix handling, indentation, and formatting control.
//! Inspired by flexmark-java's MarkdownWriter class.

use crate::formatter::options::FormatFlags;

/// Markdown output writer
///
/// This writer handles the generation of Markdown text with proper
/// prefix management, indentation, and formatting options.
#[derive(Debug, Clone)]
pub struct MarkdownWriter {
    /// The output buffer
    output: String,
    /// Stack of prefixes for nested blocks
    prefix_stack: Vec<String>,
    /// Current column position
    column: usize,
    /// Whether we're at the beginning of a line
    beginning_of_line: bool,
    /// Whether we're in pre-formatted mode
    pre_formatted: bool,
    /// Format flags
    format_flags: FormatFlags,
    /// Current line prefix
    current_prefix: String,
    /// Pending prefix for next line
    pending_prefix: Option<String>,
    /// Number of trailing blank lines
    trailing_blank_lines: usize,
    /// Maximum trailing blank lines
    max_trailing_blank_lines: usize,
    /// Whether to add space after atx marker
    space_after_atx_marker: bool,
    /// Whether to add space before info in code blocks
    space_before_info: bool,
}

impl MarkdownWriter {
    /// Create a new Markdown writer
    pub fn new(format_flags: FormatFlags) -> Self {
        Self {
            output: String::new(),
            prefix_stack: Vec::new(),
            column: 0,
            beginning_of_line: true,
            pre_formatted: false,
            format_flags,
            current_prefix: String::new(),
            pending_prefix: None,
            trailing_blank_lines: 0,
            max_trailing_blank_lines: 2,
            space_after_atx_marker: true,
            space_before_info: true,
        }
    }

    /// Create a new Markdown writer with default flags
    #[allow(clippy::should_implement_trait)]
    pub fn default() -> Self {
        Self::new(FormatFlags::DEFAULT)
    }

    /// Set whether to add space after atx marker
    pub fn set_space_after_atx_marker(&mut self, value: bool) -> &mut Self {
        self.space_after_atx_marker = value;
        self
    }

    /// Set whether to add space before info in code blocks
    pub fn set_space_before_info(&mut self, value: bool) -> &mut Self {
        self.space_before_info = value;
        self
    }

    /// Push a prefix onto the stack
    pub fn push_prefix(&mut self, prefix: impl AsRef<str>) -> &mut Self {
        let prefix = prefix.as_ref();
        self.prefix_stack.push(prefix.to_string());
        self.current_prefix.push_str(prefix);
        self
    }

    /// Pop a prefix from the stack
    pub fn pop_prefix(&mut self) -> Option<String> {
        if let Some(popped) = self.prefix_stack.pop() {
            let len = popped.len();
            let new_len = self.current_prefix.len().saturating_sub(len);
            self.current_prefix.truncate(new_len);
            Some(popped)
        } else {
            None
        }
    }

    /// Get the current prefix
    pub fn get_prefix(&self) -> &str {
        &self.current_prefix
    }

    /// Append text to the output
    pub fn append(&mut self, text: impl AsRef<str>) -> &mut Self {
        let text = text.as_ref();
        if text.is_empty() {
            return self;
        }

        // Handle beginning of line
        let was_beginning_of_line = self.beginning_of_line;
        if self.beginning_of_line {
            self.output.push_str(&self.current_prefix);
            self.column = self.current_prefix.len();
            self.beginning_of_line = false;
            self.trailing_blank_lines = 0;
        }

        // Apply format flags
        let processed_text = if self.pre_formatted {
            text.to_string()
        } else {
            self.process_text(text, was_beginning_of_line)
        };

        self.output.push_str(&processed_text);
        self.column += processed_text.len();

        self
    }

    /// Append raw text without processing
    pub fn append_raw(&mut self, text: impl AsRef<str>) -> &mut Self {
        let text = text.as_ref();
        if text.is_empty() {
            return self;
        }

        if self.beginning_of_line {
            self.output.push_str(&self.current_prefix);
            self.column = self.current_prefix.len();
            self.beginning_of_line = false;
            self.trailing_blank_lines = 0;
        }

        self.output.push_str(text);
        self.column += text.len();

        self
    }

    /// Append text for non-translating content (e.g., URLs, code)
    /// This is used for content that should not be translated
    pub fn append_non_translating(&mut self, text: impl AsRef<str>) -> &mut Self {
        // In a full implementation, this would handle translation placeholders
        // For now, just append the text directly
        self.append(text)
    }

    /// Append text for non-translating content with prefix and suffix
    pub fn append_non_translating_with(
        &mut self,
        prefix: Option<&str>,
        text: impl AsRef<str>,
        suffix: Option<&str>,
        suffix2: Option<&str>,
    ) -> &mut Self {
        if let Some(p) = prefix {
            self.append(p);
        }
        self.append_non_translating(text);
        if let Some(s) = suffix {
            self.append(s);
        }
        if let Some(s2) = suffix2 {
            self.append(s2);
        }
        self
    }

    /// Append text for translating content
    /// This is used for content that should be translated
    pub fn append_translating(&mut self, text: impl AsRef<str>) -> &mut Self {
        // In a full implementation, this would handle translation placeholders
        // For now, just append the text directly
        self.append(text)
    }

    /// Append text for translating content with prefix and suffix
    pub fn append_translating_with(
        &mut self,
        prefix: Option<&str>,
        text: impl AsRef<str>,
        suffix: Option<&str>,
        suffix2: Option<&str>,
    ) -> &mut Self {
        if let Some(p) = prefix {
            self.append(p);
        }
        self.append_translating(text);
        if let Some(s) = suffix {
            self.append(s);
        }
        if let Some(s2) = suffix2 {
            self.append(s2);
        }
        self
    }

    /// Process text according to format flags
    fn process_text(&self, text: &str, at_beginning_of_line: bool) -> String {
        let mut result = text.to_string();

        if self.format_flags.trim_leading_whitespace && at_beginning_of_line {
            result = result.trim_start().to_string();
        }

        if self.format_flags.trim_trailing_whitespace {
            result = result.trim_end().to_string();
        }

        if self.format_flags.convert_tabs {
            result = result.replace('\t', "    ");
        }

        if self.format_flags.collapse_whitespace {
            result = result.split_whitespace().collect::<Vec<_>>().join(" ");
        }

        result
    }

    /// Start a new line
    pub fn line(&mut self) -> &mut Self {
        if !self.beginning_of_line {
            self.output.push('\n');
            self.beginning_of_line = true;
            self.column = 0;
        }
        self
    }

    /// Force a newline even if already at the beginning of a line
    /// This is used for preserving empty lines in code blocks
    pub fn force_newline(&mut self) -> &mut Self {
        self.output.push('\n');
        self.beginning_of_line = true;
        self.column = 0;
        self
    }

    /// Add a blank line
    pub fn blank_line(&mut self) -> &mut Self {
        self.line();
        if self.trailing_blank_lines < self.max_trailing_blank_lines {
            self.output.push('\n');
            self.trailing_blank_lines += 1;
        }
        self.beginning_of_line = true;
        self
    }

    /// Add a blank line only if not already at a blank line
    pub fn blank_line_if_needed(&mut self) -> &mut Self {
        if self.trailing_blank_lines == 0 {
            self.blank_line();
        }
        self
    }

    /// Add trailing blank lines up to the maximum
    pub fn tail_blank_line(&mut self) -> &mut Self {
        while self.trailing_blank_lines < self.max_trailing_blank_lines {
            self.output.push('\n');
            self.trailing_blank_lines += 1;
        }
        self.beginning_of_line = true;
        self
    }

    /// Set the maximum number of trailing blank lines
    pub fn set_max_trailing_blank_lines(&mut self, max: usize) -> &mut Self {
        self.max_trailing_blank_lines = max;
        self
    }

    /// Enter pre-formatted mode
    pub fn open_pre_formatted(&mut self) -> &mut Self {
        self.pre_formatted = true;
        self
    }

    /// Exit pre-formatted mode
    pub fn close_pre_formatted(&mut self) -> &mut Self {
        self.pre_formatted = false;
        self
    }

    /// Check if in pre-formatted mode
    pub fn is_pre_formatted(&self) -> bool {
        self.pre_formatted
    }

    /// Get the current column position
    pub fn get_column(&self) -> usize {
        self.column
    }

    /// Get whether we're at the beginning of a line
    pub fn is_beginning_of_line(&self) -> bool {
        self.beginning_of_line
    }

    /// Get the current output length
    pub fn len(&self) -> usize {
        self.output.len()
    }

    /// Check if the output is empty
    pub fn is_empty(&self) -> bool {
        self.output.is_empty()
    }

    /// Get the output as a string
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        self.output.clone()
    }

    /// Get the output and consume the writer
    pub fn into_string(self) -> String {
        self.output
    }

    /// Clear the output
    pub fn clear(&mut self) {
        self.output.clear();
        self.column = 0;
        self.beginning_of_line = true;
        self.trailing_blank_lines = 0;
    }

    /// Append another writer's content
    pub fn append_writer(&mut self, other: &MarkdownWriter) -> &mut Self {
        self.append_raw(&other.output)
    }

    /// Repeat a character n times
    pub fn repeat_char(&mut self, ch: char, n: usize) -> &mut Self {
        self.append(ch.to_string().repeat(n))
    }

    /// Append a space
    pub fn space(&mut self) -> &mut Self {
        self.append(" ")
    }

    /// Append multiple spaces
    pub fn spaces(&mut self, n: usize) -> &mut Self {
        self.append(" ".repeat(n))
    }

    /// Get an empty appendable writer with the same options
    pub fn get_empty_appendable(&self) -> Self {
        Self {
            output: String::new(),
            prefix_stack: self.prefix_stack.clone(),
            column: 0,
            beginning_of_line: true,
            pre_formatted: self.pre_formatted,
            format_flags: self.format_flags,
            current_prefix: self.current_prefix.clone(),
            pending_prefix: None,
            trailing_blank_lines: 0,
            max_trailing_blank_lines: self.max_trailing_blank_lines,
            space_after_atx_marker: self.space_after_atx_marker,
            space_before_info: self.space_before_info,
        }
    }

    /// Append the content of another writer to this one
    pub fn append_to(&mut self, other: &Self) -> &mut Self {
        self.append_raw(&other.output)
    }

    /// Append to an appendable (like String or another writer)
    pub fn append_to_appendable(
        &self,
        out: &mut dyn std::fmt::Write,
    ) -> std::fmt::Result {
        out.write_str(&self.output)
    }

    /// Append with maximum blank lines constraint
    pub fn append_with_max_blank_lines(
        &self,
        out: &mut dyn std::fmt::Write,
        _max_blank_lines: usize,
        max_trailing_blank_lines: usize,
    ) -> std::fmt::Result {
        // Trim trailing blank lines to max_trailing_blank_lines
        let trimmed = self.trim_trailing_blank_lines(max_trailing_blank_lines);
        out.write_str(&trimmed)
    }

    /// Trim trailing blank lines to a maximum
    fn trim_trailing_blank_lines(&self, max: usize) -> String {
        let mut result = self.output.clone();
        let mut blank_count = 0;

        // Count trailing newlines
        while result.ends_with('\n') {
            blank_count += 1;
            result.pop();
        }

        // Add back up to max blank lines
        for _ in 0..blank_count.min(max) {
            result.push('\n');
        }

        result
    }

    /// Get the format flags
    pub fn get_options(&self) -> FormatFlags {
        self.format_flags
    }

    /// Flush the writer to an appendable with constraints
    pub fn flush_to(
        &mut self,
        out: &mut dyn std::fmt::Write,
        max_blank_lines: usize,
        max_trailing_blank_lines: usize,
    ) -> std::fmt::Result {
        self.line();
        self.append_with_max_blank_lines(out, max_blank_lines, max_trailing_blank_lines)
    }
}

impl Default for MarkdownWriter {
    fn default() -> Self {
        Self::default()
    }
}

impl std::fmt::Write for MarkdownWriter {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.append(s);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_append() {
        let mut writer = MarkdownWriter::default();
        writer.append("Hello");
        assert_eq!(writer.to_string(), "Hello");
    }

    #[test]
    fn test_prefix_handling() {
        let mut writer = MarkdownWriter::default();
        writer.push_prefix("> ");
        writer.append("Hello");
        assert_eq!(writer.to_string(), "> Hello");

        writer.line();
        writer.append("World");
        assert_eq!(writer.to_string(), "> Hello\n> World");

        writer.pop_prefix();
        writer.line();
        writer.append("Done");
        assert_eq!(writer.to_string(), "> Hello\n> World\nDone");
    }

    #[test]
    fn test_nested_prefixes() {
        let mut writer = MarkdownWriter::default();
        writer.push_prefix("> ");
        writer.push_prefix("> ");
        writer.append("Nested");
        assert_eq!(writer.to_string(), "> > Nested");

        writer.pop_prefix();
        writer.line();
        writer.append("Less nested");
        assert_eq!(writer.to_string(), "> > Nested\n> Less nested");
    }

    #[test]
    fn test_blank_lines() {
        let mut writer = MarkdownWriter::default();
        writer.append("Line 1");
        writer.blank_line();
        writer.append("Line 2");
        assert_eq!(writer.to_string(), "Line 1\n\nLine 2");
    }

    #[test]
    fn test_pre_formatted() {
        let mut writer = MarkdownWriter::default();
        writer.open_pre_formatted();
        writer.append("  Indented  ");
        writer.close_pre_formatted();
        assert_eq!(writer.to_string(), "  Indented  ");
    }

    #[test]
    fn test_format_flags_trim() {
        let mut flags = FormatFlags::DEFAULT;
        flags.trim_leading_whitespace = true;
        flags.trim_trailing_whitespace = true;

        let mut writer = MarkdownWriter::new(flags);
        writer.line();
        writer.append("  Hello  ");
        assert_eq!(writer.to_string(), "Hello");
    }

    #[test]
    fn test_column_tracking() {
        let mut writer = MarkdownWriter::default();
        writer.append("Hello");
        assert_eq!(writer.get_column(), 5);

        writer.line();
        assert_eq!(writer.get_column(), 0);
        assert!(writer.is_beginning_of_line());
    }

    #[test]
    fn test_append_writer() {
        let mut writer1 = MarkdownWriter::default();
        writer1.append("Hello");

        let mut writer2 = MarkdownWriter::default();
        writer2.append("World");

        writer1.append_writer(&writer2);
        assert_eq!(writer1.to_string(), "HelloWorld");
    }

    #[test]
    fn test_write_trait() {
        use std::fmt::Write;

        let mut writer = MarkdownWriter::default();
        write!(writer, "Hello, World!").unwrap();
        assert_eq!(writer.to_string(), "Hello, World!");
    }

    #[test]
    fn test_append_non_translating() {
        let mut writer = MarkdownWriter::default();
        writer.append_non_translating("code");
        assert_eq!(writer.to_string(), "code");
    }

    #[test]
    fn test_append_translating() {
        let mut writer = MarkdownWriter::default();
        writer.append_translating("text");
        assert_eq!(writer.to_string(), "text");
    }

    #[test]
    fn test_get_empty_appendable() {
        let mut writer = MarkdownWriter::default();
        writer.push_prefix("> ");
        writer.append("Hello");

        let empty = writer.get_empty_appendable();
        assert_eq!(empty.get_prefix(), "> ");
        assert!(empty.is_beginning_of_line());
        assert_eq!(empty.to_string(), "");
    }

    #[test]
    fn test_flush_to() {
        let mut writer = MarkdownWriter::default();
        writer.append("Hello");

        let mut output = String::new();
        writer.flush_to(&mut output, 2, 2).unwrap();
        assert_eq!(output, "Hello\n");
    }
}
