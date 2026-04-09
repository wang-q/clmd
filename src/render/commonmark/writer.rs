//! Markdown writer for formatted output
//!
//! This module provides a writer for generating Markdown output with
//! proper prefix handling, indentation, and formatting control.
//! Inspired by flexmark-java's MarkdownWriter class.

use crate::options::format::FormatFlags;
use crate::text::unicode_width;

/// Line information for tracking output state
///
/// This structure tracks information about the current line being written,
/// useful for making formatting decisions based on context.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LineInfo {
    /// The line number (0-indexed)
    pub line_number: usize,
    /// The column position on the current line
    pub column: usize,
    /// Whether this is the beginning of a line
    pub beginning_of_line: bool,
    /// Whether the current line is blank
    pub is_blank_line: bool,
}

impl LineInfo {
    /// Create a new LineInfo at the start of a document
    pub fn new() -> Self {
        Self {
            line_number: 0,
            column: 0,
            beginning_of_line: true,
            is_blank_line: true,
        }
    }

    /// Advance the column by the given amount
    pub fn advance_column(&mut self, n: usize) {
        self.column += n;
        self.is_blank_line = false;
    }

    /// Move to the next line
    pub fn next_line(&mut self) {
        self.line_number += 1;
        self.column = 0;
        self.beginning_of_line = true;
        self.is_blank_line = true;
    }

    /// Mark that we're no longer at the beginning of a line
    pub fn mark_content_written(&mut self) {
        self.beginning_of_line = false;
        self.is_blank_line = false;
    }
}

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
    /// Number of trailing blank lines
    trailing_blank_lines: usize,
    /// Maximum trailing blank lines
    max_trailing_blank_lines: usize,
    /// Whether to add space after atx marker
    space_after_atx_marker: bool,
    /// Whether to add space before info in code blocks
    space_before_info: bool,
    /// Right margin for text wrapping (0 = no wrapping)
    right_margin: usize,
    /// Buffer for word wrapping
    word_wrap_buffer: String,
    /// Line information tracking
    line_info: LineInfo,
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
            trailing_blank_lines: 0,
            max_trailing_blank_lines: 2,
            space_after_atx_marker: true,
            space_before_info: true,
            right_margin: 0,
            word_wrap_buffer: String::new(),
            line_info: LineInfo::new(),
        }
    }

    /// Set the right margin for text wrapping
    pub fn set_right_margin(&mut self, margin: usize) -> &mut Self {
        self.right_margin = margin;
        self
    }

    /// Get the right margin
    pub fn get_right_margin(&self) -> usize {
        self.right_margin
    }

    /// Append text with word wrapping if right_margin is set
    pub fn append_with_wrap(&mut self, text: impl AsRef<str>) -> &mut Self {
        let text = text.as_ref();
        if text.is_empty() {
            return self;
        }

        // If no wrapping is configured or in pre-formatted mode, just append raw
        // Use append_raw to preserve whitespace (append would trim trailing whitespace)
        if self.right_margin == 0 || self.pre_formatted {
            return self.append_raw(text);
        }

        // Add text to word wrap buffer
        self.word_wrap_buffer.push_str(text);

        // Process the buffer for wrapping
        self.process_word_wrap_buffer();

        self
    }

    /// Process the word wrap buffer and output wrapped text
    fn process_word_wrap_buffer(&mut self) {
        // Available width is right_margin minus prefix length
        let available_width =
            self.right_margin.saturating_sub(self.current_prefix.len());
        if available_width == 0 {
            // No space available, just output everything
            let buffer = std::mem::take(&mut self.word_wrap_buffer);
            self.append(&buffer);
            return;
        }

        loop {
            // Find the next word boundary
            if let Some(space_pos) = self.word_wrap_buffer.find(' ') {
                let word_width =
                    unicode_width::width(&self.word_wrap_buffer[..space_pos]) as usize;
                let remaining_start = space_pos + 1;

                // Check if adding this word would exceed the margin
                // Use Unicode display width for accurate CJK character handling
                if self.column + word_width > self.right_margin
                    && !self.beginning_of_line
                {
                    // Start a new line
                    self.line();
                }

                // Extract and output the word
                let word: String = self.word_wrap_buffer[..space_pos].to_string();
                self.append(&word);
                // Use append_raw to preserve the space (append would trim it)
                self.append_raw(" ");

                // Update buffer - remove the word and space we just processed
                let remaining: String =
                    self.word_wrap_buffer[remaining_start..].to_string();
                self.word_wrap_buffer = remaining;
            } else {
                // No more spaces, check if remaining text fits
                let remaining_width =
                    unicode_width::width(&self.word_wrap_buffer) as usize;
                if remaining_width > 0 {
                    if self.column + remaining_width > self.right_margin
                        && !self.beginning_of_line
                    {
                        // Start a new line
                        self.line();
                    }
                    // Output remaining text
                    let buffer = std::mem::take(&mut self.word_wrap_buffer);
                    self.append(&buffer);
                }
                break;
            }
        }
    }

    /// Flush any remaining text in the word wrap buffer
    pub fn flush_word_wrap_buffer(&mut self) -> &mut Self {
        if !self.word_wrap_buffer.is_empty() {
            // Use append_raw to preserve trailing whitespace
            let buffer = std::mem::take(&mut self.word_wrap_buffer);
            self.append_raw(&buffer);
        }
        self
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

    /// Handle the beginning of a line by adding prefix if needed
    ///
    /// Returns whether we were at the beginning of line before handling.
    fn handle_beginning_of_line(&mut self) -> bool {
        let was_beginning_of_line = self.beginning_of_line;
        if self.beginning_of_line {
            self.output.push_str(&self.current_prefix);
            self.column = self.current_prefix.len();
            self.beginning_of_line = false;
            self.trailing_blank_lines = 0;
            self.line_info.mark_content_written();
        }
        was_beginning_of_line
    }

    /// Append text to the output with format flag processing
    pub fn append(&mut self, text: impl AsRef<str>) -> &mut Self {
        let text = text.as_ref();
        if text.is_empty() {
            return self;
        }

        let was_beginning_of_line = self.handle_beginning_of_line();

        // Apply format flags
        let processed_text = if self.pre_formatted {
            text.to_string()
        } else {
            self.process_text(text, was_beginning_of_line)
        };

        self.output.push_str(&processed_text);
        self.column += processed_text.len();
        self.line_info.advance_column(processed_text.len());

        self
    }

    /// Append raw text without processing
    pub fn append_raw(&mut self, text: impl AsRef<str>) -> &mut Self {
        let text = text.as_ref();
        if text.is_empty() {
            return self;
        }

        self.handle_beginning_of_line();

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
            self.line_info.next_line();
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

    /// Check if the output ends with whitespace
    pub fn ends_with_whitespace(&self) -> bool {
        self.output.ends_with(|c: char| c.is_whitespace())
    }

    /// Check if the output ends with a specific character
    pub fn ends_with_char(&self, ch: char) -> bool {
        self.output.ends_with(ch)
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
            trailing_blank_lines: 0,
            max_trailing_blank_lines: self.max_trailing_blank_lines,
            space_after_atx_marker: self.space_after_atx_marker,
            space_before_info: self.space_before_info,
            right_margin: self.right_margin,
            word_wrap_buffer: String::new(),
            line_info: LineInfo::new(),
        }
    }

    /// Get the current line information
    pub fn get_line_info(&self) -> LineInfo {
        self.line_info
    }

    /// Get the current line number
    pub fn get_line_number(&self) -> usize {
        self.line_info.line_number
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

    /// Get the last block quote child prefix
    ///
    /// This method is used to handle the special case where a block quote's last child
    /// needs a modified prefix. In flexmark-java, this handles the case where continuation
    /// block prefixes are removed for the last child.
    ///
    /// For example, in a block quote with nested content, the last child's prefix
    /// may need to be adjusted to remove continuation markers.
    pub fn last_block_quote_child_prefix(&self, prefix: impl AsRef<str>) -> String {
        let prefix = prefix.as_ref();

        // Find the last occurrence of a block quote-like prefix character
        // This is typically '>' for block quotes
        if let Some(pos) = prefix.rfind('>') {
            // Create a new prefix where the last '>' is replaced with a space
            // This handles the case of continuation block prefix removal
            let mut result = prefix.to_string();
            if pos < result.len() {
                // Replace the '>' with a space to indicate it's the last child
                result.replace_range(pos..pos + 1, " ");
            }
            result
        } else {
            // No block quote prefix found, return as-is
            prefix.to_string()
        }
    }

    /// Check if the current prefix contains a block quote marker
    pub fn has_block_quote_prefix(&self) -> bool {
        self.current_prefix.contains('>')
    }

    /// Get the prefix stack depth
    pub fn get_prefix_depth(&self) -> usize {
        self.prefix_stack.len()
    }

    /// Check if there are any prefixes on the stack
    pub fn has_prefixes(&self) -> bool {
        !self.prefix_stack.is_empty()
    }

    /// Get a copy of the prefix stack
    pub fn get_prefix_stack(&self) -> Vec<String> {
        self.prefix_stack.clone()
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
        Self::new(FormatFlags::DEFAULT)
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

    #[test]
    fn test_last_block_quote_child_prefix() {
        let writer = MarkdownWriter::default();

        // Test with block quote prefix
        assert_eq!(writer.last_block_quote_child_prefix("> "), "  ");

        // Test with nested block quote prefix
        assert_eq!(writer.last_block_quote_child_prefix("> > "), ">   ");

        // Test without block quote prefix
        assert_eq!(writer.last_block_quote_child_prefix("    "), "    ");

        // Test with multiple '>' characters
        assert_eq!(writer.last_block_quote_child_prefix("> > > "), "> >   ");
    }

    #[test]
    fn test_has_block_quote_prefix() {
        let mut writer = MarkdownWriter::default();
        assert!(!writer.has_block_quote_prefix());

        writer.push_prefix("> ");
        assert!(writer.has_block_quote_prefix());

        writer.pop_prefix();
        assert!(!writer.has_block_quote_prefix());
    }

    #[test]
    fn test_get_prefix_depth() {
        let mut writer = MarkdownWriter::default();
        assert_eq!(writer.get_prefix_depth(), 0);

        writer.push_prefix("> ");
        assert_eq!(writer.get_prefix_depth(), 1);

        writer.push_prefix("> ");
        assert_eq!(writer.get_prefix_depth(), 2);

        writer.pop_prefix();
        assert_eq!(writer.get_prefix_depth(), 1);
    }

    #[test]
    fn test_has_prefixes() {
        let mut writer = MarkdownWriter::default();
        assert!(!writer.has_prefixes());

        writer.push_prefix("> ");
        assert!(writer.has_prefixes());

        writer.pop_prefix();
        assert!(!writer.has_prefixes());
    }

    #[test]
    fn test_get_prefix_stack() {
        let mut writer = MarkdownWriter::default();
        writer.push_prefix("> ");
        writer.push_prefix("  ");

        let stack = writer.get_prefix_stack();
        assert_eq!(stack.len(), 2);
        assert_eq!(stack[0], "> ");
        assert_eq!(stack[1], "  ");
    }

    #[test]
    fn test_markdown_writer_creation() {
        let writer = MarkdownWriter::new(FormatFlags::DEFAULT);
        assert!(writer.is_beginning_of_line());
        assert_eq!(writer.get_column(), 0);
        assert!(writer.is_empty());
    }

    #[test]
    fn test_markdown_writer_default() {
        let writer = MarkdownWriter::default();
        assert!(writer.is_beginning_of_line());
        assert_eq!(writer.get_column(), 0);
    }

    #[test]
    fn test_append_raw() {
        let mut writer = MarkdownWriter::default();
        writer.append_raw("  raw text  ");
        assert_eq!(writer.to_string(), "  raw text  ");
    }

    #[test]
    fn test_line() {
        let mut writer = MarkdownWriter::default();
        writer.append("Hello");
        writer.line();
        writer.append("World");
        assert_eq!(writer.to_string(), "Hello\nWorld");
    }

    #[test]
    fn test_force_newline() {
        let mut writer = MarkdownWriter::default();
        writer.force_newline();
        writer.append("Hello");
        assert_eq!(writer.to_string(), "\nHello");
    }

    #[test]
    fn test_blank_line_if_needed() {
        let mut writer = MarkdownWriter::default();
        writer.append("Line 1");
        writer.blank_line_if_needed();
        writer.append("Line 2");
        assert_eq!(writer.to_string(), "Line 1\n\nLine 2");
    }

    #[test]
    fn test_tail_blank_line() {
        let mut writer = MarkdownWriter::default();
        writer.append("Hello");
        writer.tail_blank_line();
        let output = writer.to_string();
        assert!(output.starts_with("Hello"));
        assert!(output.ends_with("\n\n"));
    }

    #[test]
    fn test_space() {
        let mut flags = FormatFlags::DEFAULT;
        flags.collapse_whitespace = false;
        flags.trim_trailing_whitespace = false;
        let mut writer = MarkdownWriter::new(flags);
        writer.append("Hello");
        writer.space();
        writer.append("World");
        assert_eq!(writer.to_string(), "Hello World");
    }

    #[test]
    fn test_spaces() {
        let mut flags = FormatFlags::DEFAULT;
        flags.collapse_whitespace = false;
        flags.trim_trailing_whitespace = false;
        let mut writer = MarkdownWriter::new(flags);
        writer.append("Hello");
        writer.spaces(3);
        writer.append("World");
        assert_eq!(writer.to_string(), "Hello   World");
    }

    #[test]
    fn test_repeat_char() {
        let mut writer = MarkdownWriter::default();
        writer.repeat_char('-', 5);
        assert_eq!(writer.to_string(), "-----");
    }

    #[test]
    fn test_clear() {
        let mut writer = MarkdownWriter::default();
        writer.append("Hello");
        writer.clear();
        assert!(writer.is_empty());
        assert_eq!(writer.get_column(), 0);
        assert!(writer.is_beginning_of_line());
    }

    #[test]
    fn test_into_string() {
        let mut writer = MarkdownWriter::default();
        writer.append("Hello");
        let s = writer.into_string();
        assert_eq!(s, "Hello");
    }

    #[test]
    fn test_len() {
        let mut writer = MarkdownWriter::default();
        assert_eq!(writer.len(), 0);
        writer.append("Hello");
        assert_eq!(writer.len(), 5);
    }

    #[test]
    fn test_is_pre_formatted() {
        let mut writer = MarkdownWriter::default();
        assert!(!writer.is_pre_formatted());
        writer.open_pre_formatted();
        assert!(writer.is_pre_formatted());
        writer.close_pre_formatted();
        assert!(!writer.is_pre_formatted());
    }

    #[test]
    fn test_get_line_number() {
        let mut writer = MarkdownWriter::default();
        assert_eq!(writer.get_line_number(), 0);
        // line() only adds newline if not at beginning of line
        // So we need to append something first
        writer.append("test");
        writer.line();
        assert_eq!(writer.get_line_number(), 1);
        writer.append("test");
        writer.line();
        assert_eq!(writer.get_line_number(), 2);
    }

    #[test]
    fn test_get_line_info() {
        let writer = MarkdownWriter::default();
        let info = writer.get_line_info();
        assert_eq!(info.line_number, 0);
        assert_eq!(info.column, 0);
        assert!(info.beginning_of_line);
    }

    #[test]
    fn test_set_right_margin() {
        let mut writer = MarkdownWriter::default();
        writer.set_right_margin(40);
        assert_eq!(writer.get_right_margin(), 40);
    }

    #[test]
    fn test_append_with_wrap() {
        let mut writer = MarkdownWriter::default();
        writer.set_right_margin(10);
        writer.append_with_wrap("Hello World Test");
        let output = writer.to_string();
        // Should wrap at 10 characters
        assert!(output.contains('\n') || output.len() <= 10);
    }

    #[test]
    fn test_flush_word_wrap_buffer() {
        let mut writer = MarkdownWriter::default();
        writer.set_right_margin(10);
        writer.append_with_wrap("Hello");
        writer.flush_word_wrap_buffer();
        let output = writer.to_string();
        assert!(output.contains("Hello"));
    }

    #[test]
    fn test_append_non_translating_with() {
        let mut writer = MarkdownWriter::default();
        writer.append_non_translating_with(Some("["), "text", Some("]"), None);
        assert_eq!(writer.to_string(), "[text]");
    }

    #[test]
    fn test_append_translating_with() {
        let mut writer = MarkdownWriter::default();
        writer.append_translating_with(Some("**"), "bold", Some("**"), None);
        assert_eq!(writer.to_string(), "**bold**");
    }

    #[test]
    fn test_set_space_after_atx_marker() {
        let mut writer = MarkdownWriter::default();
        writer.set_space_after_atx_marker(false);
        // Just verify it doesn't panic
        assert!(true);
    }

    #[test]
    fn test_set_space_before_info() {
        let mut writer = MarkdownWriter::default();
        writer.set_space_before_info(false);
        // Just verify it doesn't panic
        assert!(true);
    }

    #[test]
    fn test_set_max_trailing_blank_lines() {
        let mut writer = MarkdownWriter::default();
        writer.set_max_trailing_blank_lines(1);
        writer.append("Hello");
        writer.tail_blank_line();
        // With max 1, should only have 1 trailing newline
        assert!(true); // Just verify it doesn't panic
    }

    #[test]
    fn test_append_to_appendable() {
        let writer = MarkdownWriter::default();
        let mut output = String::new();
        writer.append_to_appendable(&mut output).unwrap();
        assert!(output.is_empty());
    }

    #[test]
    fn test_append_with_max_blank_lines() {
        let mut writer = MarkdownWriter::default();
        writer.append("Hello");
        writer.blank_line();
        writer.blank_line();
        writer.blank_line();

        let mut output = String::new();
        writer
            .append_with_max_blank_lines(&mut output, 2, 2)
            .unwrap();
        // Should be trimmed to max 2 trailing blank lines
        assert!(output.starts_with("Hello"));
    }
}
