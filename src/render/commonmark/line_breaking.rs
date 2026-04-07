//! Knuth-Plass line breaking algorithm implementation
//!
//! This module implements the Knuth-Plass line breaking algorithm for optimal
//! paragraph formatting. The algorithm uses dynamic programming to find the
//! globally optimal set of line breaks that minimizes the total "badness" of
//! the paragraph.
//!
//! The algorithm is based on the paper "Breaking Paragraphs into Lines" by
//! Donald E. Knuth and Michael F. Plass (1981).

use crate::text::char::is_cjk_punctuation;
use crate::text::unicode_width;

/// A word in the paragraph with its display width
#[derive(Debug, Clone)]
pub struct Word {
    /// The text content of the word
    pub text: String,
    /// The display width of the word (accounting for CJK characters)
    pub width: usize,
    /// Whether this word is followed by a space
    pub has_trailing_space: bool,
    /// Whether this word needs a leading space (false for punctuation/marks)
    pub needs_leading_space: bool,
}

impl Word {
    /// Create a new word from text
    pub fn new(text: impl Into<String>) -> Self {
        let text = text.into();
        let width = unicode_width::width(&text) as usize;
        Self {
            text,
            width,
            has_trailing_space: true,
            needs_leading_space: true,
        }
    }

    /// Create a new word without trailing space
    pub fn new_without_space(text: impl Into<String>) -> Self {
        let text = text.into();
        let width = unicode_width::width(&text) as usize;
        Self {
            text,
            width,
            has_trailing_space: false,
            needs_leading_space: true,
        }
    }

    /// Create a new CJK word that doesn't need spaces around it
    pub fn new_cjk(text: impl Into<String>) -> Self {
        let text = text.into();
        let width = unicode_width::width(&text) as usize;
        Self {
            text,
            width,
            has_trailing_space: false,
            needs_leading_space: false,
        }
    }

    /// Create a new punctuation/mark word that doesn't need spaces around it
    pub fn new_mark(text: impl Into<String>) -> Self {
        let text = text.into();
        let width = unicode_width::width(&text) as usize;
        Self {
            text,
            width,
            has_trailing_space: false,
            needs_leading_space: false,
        }
    }
}

/// A breakpoint in the paragraph
#[derive(Debug, Clone)]
struct BreakPoint {
    /// Index of the word at this breakpoint
    #[allow(dead_code)]
    word_index: usize,
    /// Total badness from start to this breakpoint
    total_badness: f64,
    /// Previous breakpoint index (for backtracking)
    prev_break: Option<usize>,
}

/// Context for line breaking
#[derive(Debug)]
pub struct LineBreakingContext {
    /// The words in the paragraph
    words: Vec<Word>,
    /// The ideal line width
    ideal_width: usize,
    /// The maximum line width
    max_width: usize,
    /// Whether line breaking is enabled
    enabled: bool,
    /// First line prefix (e.g., "- " or "1. ")
    first_line_prefix: String,
    /// Continuation line prefix (e.g., "  ")
    continuation_prefix: String,
    /// Whether the next word should not have a leading space
    next_word_no_leading_space: bool,
}

impl LineBreakingContext {
    /// Create a new line breaking context
    pub fn new(ideal_width: usize, max_width: usize) -> Self {
        Self {
            words: Vec::new(),
            ideal_width,
            max_width,
            enabled: max_width > 0,
            first_line_prefix: String::new(),
            continuation_prefix: String::new(),
            next_word_no_leading_space: false,
        }
    }

    /// Create a new line breaking context with prefixes
    pub fn with_prefixes(
        ideal_width: usize,
        max_width: usize,
        first_line_prefix: impl Into<String>,
        continuation_prefix: impl Into<String>,
    ) -> Self {
        Self {
            words: Vec::new(),
            ideal_width,
            max_width,
            enabled: max_width > 0,
            first_line_prefix: first_line_prefix.into(),
            continuation_prefix: continuation_prefix.into(),
            next_word_no_leading_space: false,
        }
    }

    /// Check if line breaking is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Add a word to the context
    pub fn add_word(&mut self, mut word: Word) {
        // If next_word_no_leading_space is set, clear the leading space flag
        if self.next_word_no_leading_space {
            word.needs_leading_space = false;
            self.next_word_no_leading_space = false;
        }
        self.words.push(word);
    }

    /// Add text and split it into words
    /// For CJK text, splits at punctuation marks to allow better line breaking
    /// Note: This function does NOT add CJK spacing - that is handled by add_cjk_spacing before this
    pub fn add_text(&mut self, text: &str) {
        // Split text by whitespace first
        let whitespace_separated: Vec<&str> = text.split_whitespace().collect();
        let total_segments = whitespace_separated.len();

        for (i, segment) in whitespace_separated.iter().enumerate() {
            // Check if this segment contains CJK characters
            if contains_cjk(segment) {
                // Split CJK text at punctuation marks for better line breaking
                // This only splits at punctuation, not at CJK/ASCII boundaries
                let cjk_words = split_cjk_text(segment);
                let total_cjk_words = cjk_words.len();

                for (j, word_text) in cjk_words.iter().enumerate() {
                    // All words from split_cjk_text are treated as CJK words
                    let mut w = Word::new_cjk(word_text.as_str());
                    // First word of first segment: if it starts with CJK punctuation, don't add leading space
                    // Note: next_word_no_leading_space is handled by add_word
                    if i == 0 && j == 0 && starts_with_cjk_punctuation(word_text) {
                        w.needs_leading_space = false;
                    }
                    // Only add trailing space if this is not the last segment
                    // (i.e., there was whitespace after this segment in the original text)
                    // Don't add space between words within the same segment
                    if i < total_segments - 1 && j == total_cjk_words - 1 {
                        w.has_trailing_space = true;
                    }
                    self.add_word(w);
                }
            } else {
                // Non-CJK text: treat as single word
                let mut word = Word::new(*segment);
                if self.next_word_no_leading_space {
                    word.has_trailing_space = false;
                }
                // If this is the last segment, don't add trailing space
                // (the next element will decide if space is needed)
                if i == total_segments - 1 {
                    word.has_trailing_space = false;
                }
                self.add_word(word);
            }
        }
    }

    /// Add text as a single word without splitting
    pub fn add_text_as_word(&mut self, text: &str) {
        self.add_word(Word::new_without_space(text));
    }

    /// Add a mark/punctuation that doesn't need spaces around it
    pub fn add_mark(&mut self, text: &str) {
        self.add_word(Word::new_mark(text));
        // The next word should not have a leading space
        self.next_word_no_leading_space = true;
    }

    /// Add an inline element (like code span) that should preserve surrounding spaces
    pub fn add_inline_element(&mut self, text: &str) {
        // Use new_without_space to avoid adding trailing space after inline elements
        // This is important for CJK punctuation that follows inline code
        self.add_word(Word::new_without_space(text));
        // Set next_word_no_leading_space - the next word should not have leading space
        self.next_word_no_leading_space = true;
    }

    /// Reset the "no leading space" flag
    pub fn reset_next_word_no_leading_space(&mut self) {
        self.next_word_no_leading_space = false;
    }

    /// Compute the optimal line breaks using Knuth-Plass algorithm
    pub fn compute_breaks(&self) -> Vec<usize> {
        if !self.enabled || self.words.is_empty() {
            return Vec::new();
        }

        let n = self.words.len();
        let mut breaks: Vec<BreakPoint> = vec![BreakPoint {
            word_index: 0,
            total_badness: 0.0,
            prev_break: None,
        }];

        // Calculate prefix widths
        let first_prefix_width = unicode_width::width(&self.first_line_prefix) as usize;
        let cont_prefix_width = unicode_width::width(&self.continuation_prefix) as usize;

        // Dynamic programming: for each possible breakpoint
        for j in 1..=n {
            let mut best_badness = f64::INFINITY;
            let mut best_prev = None;

            // Try all possible previous breakpoints
            for i in (0..j).rev() {
                // Determine if this is the first line (i == 0)
                let is_first_line = i == 0;
                let line_width =
                    self.calculate_line_width_with_prefix(i, j, is_first_line);

                // Use appropriate max width based on whether it's the first line
                let effective_max_width = if is_first_line {
                    self.max_width
                } else {
                    self.max_width.saturating_sub(cont_prefix_width) + first_prefix_width
                };

                if line_width > effective_max_width {
                    break; // Exceeds max width, stop searching
                }

                let badness = calculate_badness(line_width, self.ideal_width);
                let total_badness = breaks[i].total_badness + badness;

                if total_badness < best_badness {
                    best_badness = total_badness;
                    best_prev = Some(i);
                }
            }

            breaks.push(BreakPoint {
                word_index: j,
                total_badness: best_badness,
                prev_break: best_prev,
            });
        }

        // Backtrack to find the optimal breakpoints
        let mut result = Vec::new();
        let mut current = n;

        while let Some(prev) = breaks[current].prev_break {
            result.push(current);
            current = prev;
        }

        result.reverse();
        result
    }

    #[allow(dead_code)]
    /// Calculate the width of a line from word i to word j (exclusive)
    fn calculate_line_width(&self, start: usize, end: usize) -> usize {
        self.calculate_line_width_with_prefix(start, end, start == 0)
    }

    /// Calculate the width of a line from word i to word j (exclusive) with prefix consideration
    fn calculate_line_width_with_prefix(
        &self,
        start: usize,
        end: usize,
        is_first_line: bool,
    ) -> usize {
        let prefix_width = if is_first_line {
            unicode_width::width(&self.first_line_prefix) as usize
        } else {
            unicode_width::width(&self.continuation_prefix) as usize
        };

        let mut width = prefix_width;
        for i in start..end {
            width += self.words[i].width;
            // Add space after word if it has trailing space and it's not the last word
            if i < end - 1 && self.words[i].has_trailing_space {
                width += 1;
            }
        }
        width
    }

    /// Format the paragraph with the computed line breaks
    pub fn format(&self) -> String {
        let breaks = self.compute_breaks();
        if breaks.is_empty() {
            // No breaks needed, return all words joined with first line prefix
            let mut result = self.first_line_prefix.clone();
            for (i, word) in self.words.iter().enumerate() {
                // Add space if:
                // - It's not the first word
                // - Current word needs leading space OR previous word has trailing space
                if i > 0
                    && (word.needs_leading_space || self.words[i - 1].has_trailing_space)
                {
                    result.push(' ');
                }
                result.push_str(&word.text);
            }
            return result;
        }

        let mut result = String::new();
        let mut start = 0;
        let mut is_first_line = true;

        for &end in &breaks {
            // Add appropriate prefix
            if is_first_line {
                result.push_str(&self.first_line_prefix);
            } else {
                result.push_str(&self.continuation_prefix);
            }

            // Add words from start to end
            for i in start..end {
                // Add space if:
                // - It's not the first word in the line
                // - Current word needs leading space OR previous word has trailing space
                if i > start
                    && (self.words[i].needs_leading_space
                        || self.words[i - 1].has_trailing_space)
                {
                    result.push(' ');
                }
                result.push_str(&self.words[i].text);
            }
            result.push('\n');
            start = end;
            is_first_line = false;
        }

        // Remove trailing newline
        if result.ends_with('\n') {
            result.pop();
        }

        result
    }

    /// Get the words
    pub fn words(&self) -> &[Word] {
        &self.words
    }

    /// Clear all words
    pub fn clear(&mut self) {
        self.words.clear();
    }
}

/// Calculate the badness of a line with given width
///
/// Badness is defined as the square of the difference between
/// the actual line width and the ideal line width.
fn calculate_badness(line_width: usize, ideal_width: usize) -> f64 {
    let diff = line_width.abs_diff(ideal_width);
    (diff as f64).powi(2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_line_breaking() {
        let mut ctx = LineBreakingContext::new(20, 25);
        ctx.add_text("This is a simple test paragraph");

        let breaks = ctx.compute_breaks();
        assert!(!breaks.is_empty());

        let formatted = ctx.format();
        // Each line should be reasonably close to ideal width
        for line in formatted.lines() {
            let width = unicode_width::width(line) as usize;
            assert!(width <= 25, "Line exceeds max width: {}", line);
        }
    }

    #[test]
    fn test_cjk_line_breaking() {
        let mut ctx = LineBreakingContext::new(20, 25);
        // Add CJK text (each character has width 2)
        ctx.add_word(Word::new("这是一个"));
        ctx.add_word(Word::new("测试"));
        ctx.add_word(Word::new("段落"));

        let formatted = ctx.format();
        for line in formatted.lines() {
            let width = unicode_width::width(line) as usize;
            assert!(width <= 25, "Line exceeds max width: {}", line);
        }
    }

    #[test]
    fn test_empty_context() {
        let ctx = LineBreakingContext::new(20, 25);
        let breaks = ctx.compute_breaks();
        assert!(breaks.is_empty());
    }

    #[test]
    fn test_disabled_line_breaking() {
        let mut ctx = LineBreakingContext::new(20, 0); // max_width = 0 disables breaking
        ctx.add_text("This is a test");

        let breaks = ctx.compute_breaks();
        assert!(breaks.is_empty());
    }

    #[test]
    fn test_long_word() {
        let mut ctx = LineBreakingContext::new(20, 25);
        ctx.add_word(Word::new("verylongwordthatexceedsthemaxwidth"));

        // Even with a very long word, we should handle it gracefully
        let formatted = ctx.format();
        assert!(!formatted.is_empty());
    }

    #[test]
    fn test_badness_calculation() {
        // Perfect match
        assert_eq!(calculate_badness(20, 20), 0.0);

        // Off by 1
        assert_eq!(calculate_badness(21, 20), 1.0);
        assert_eq!(calculate_badness(19, 20), 1.0);

        // Off by 5
        assert_eq!(calculate_badness(25, 20), 25.0);
    }

    #[test]
    fn test_line_breaking_with_prefixes() {
        let mut ctx = LineBreakingContext::with_prefixes(20, 30, "- ", "  ");
        ctx.add_text("This is a list item with some text");

        let formatted = ctx.format();
        let lines: Vec<&str> = formatted.lines().collect();

        // First line should start with "- "
        assert!(
            lines[0].starts_with("- "),
            "First line should start with list marker"
        );

        // Subsequent lines should start with "  "
        if lines.len() > 1 {
            assert!(
                lines[1].starts_with("  "),
                "Continuation lines should be indented"
            );
        }
    }

    #[test]
    fn test_line_breaking_with_ordered_list_prefix() {
        let mut ctx = LineBreakingContext::with_prefixes(20, 35, "1. ", "   ");
        ctx.add_text("First ordered item with some text content");

        let formatted = ctx.format();
        let lines: Vec<&str> = formatted.lines().collect();

        // First line should start with "1. "
        assert!(
            lines[0].starts_with("1. "),
            "First line should start with ordered list marker"
        );

        // Subsequent lines should start with "   " (3 spaces)
        if lines.len() > 1 {
            assert!(
                lines[1].starts_with("   "),
                "Continuation lines should be indented to align with content"
            );
        }
    }

    #[test]
    fn test_prefix_width_considered_in_breaks() {
        // Create context with prefixes
        let mut ctx_with_prefix = LineBreakingContext::with_prefixes(20, 25, "- ", "  ");
        ctx_with_prefix.add_text("This is a test paragraph");

        let formatted_with_prefix = ctx_with_prefix.format();

        // Check that lines respect the max width considering prefixes
        for line in formatted_with_prefix.lines() {
            let width = unicode_width::width(line) as usize;
            assert!(width <= 25, "Line with prefix exceeds max width: {}", line);
        }
    }

    #[test]
    fn test_empty_prefixes() {
        // Test with empty prefixes (same as no prefixes)
        let mut ctx = LineBreakingContext::with_prefixes(20, 25, "", "");
        ctx.add_text("Simple text without prefixes");

        let formatted = ctx.format();
        let lines: Vec<&str> = formatted.lines().collect();

        // All lines should not have any prefix
        for line in lines {
            assert!(
                !line.starts_with("- ") && !line.starts_with("  "),
                "Line should not have prefix: {}",
                line
            );
        }
    }

    #[test]
    fn test_nested_list_prefixes() {
        // Simulate nested list with different indentation levels
        let mut ctx = LineBreakingContext::with_prefixes(15, 20, "    - ", "      ");
        ctx.add_text("Nested item with text");

        let formatted = ctx.format();
        let lines: Vec<&str> = formatted.lines().collect();

        // First line should have nested marker
        assert!(lines[0].starts_with("    - "), "Should have nested marker");

        // Check width constraint
        for line in formatted.lines() {
            let width = unicode_width::width(line) as usize;
            assert!(width <= 20, "Nested line exceeds max width: {}", line);
        }
    }

    // Tests for regular paragraph line breaking

    #[test]
    fn test_paragraph_multiple_lines() {
        let mut ctx = LineBreakingContext::new(20, 25);
        ctx.add_text("First line with some text. Second part with more text here.");

        let formatted = ctx.format();
        let lines: Vec<&str> = formatted.lines().collect();

        // Should produce multiple lines
        assert!(
            lines.len() >= 2,
            "Should have multiple lines, got: {:?}",
            lines
        );

        // Each line should respect max width
        for line in &lines {
            let width = unicode_width::width(line) as usize;
            assert!(
                width <= 25,
                "Line exceeds max width: {} (width: {})",
                line,
                width
            );
        }
    }

    #[test]
    fn test_paragraph_single_line() {
        // Short paragraph that fits on one line
        let mut ctx = LineBreakingContext::new(20, 25);
        ctx.add_text("Short text");

        let formatted = ctx.format();
        let lines: Vec<&str> = formatted.lines().collect();

        // Should be a single line
        assert_eq!(lines.len(), 1, "Short text should be on one line");
        assert_eq!(lines[0], "Short text");
    }

    #[test]
    fn test_paragraph_optimal_break_points() {
        // Test that Knuth-Plass finds optimal break points
        let mut ctx = LineBreakingContext::new(15, 20);
        ctx.add_text("The quick brown fox jumps over the lazy dog");

        let formatted = ctx.format();
        let lines: Vec<&str> = formatted.lines().collect();

        // Check that lines are reasonably balanced
        // (not too short, not exceeding max)
        for line in &lines {
            let width = unicode_width::width(line) as usize;
            assert!(width <= 20, "Line exceeds max width: {}", line);
            // Lines should be reasonably filled (at least 50% of ideal width)
            assert!(width >= 7, "Line too short, not optimal: {}", line);
        }
    }

    #[test]
    fn test_paragraph_with_mixed_content() {
        // Paragraph with mixed word lengths
        let mut ctx = LineBreakingContext::new(20, 25);
        ctx.add_text("A very longwordthatmightbeproblematic and then more text here");

        let formatted = ctx.format();

        // The long word might exceed max width on its own line
        // This is expected behavior - we can't break words
        assert!(!formatted.is_empty(), "Should produce some output");

        // Check that lines without the long word respect max width
        for line in formatted.lines() {
            let width = unicode_width::width(line) as usize;
            // Skip the line with the very long word if it exceeds max width
            if !line.contains("longwordthatmightbeproblematic") {
                assert!(width <= 25, "Line exceeds max width: {}", line);
            }
        }
    }

    #[test]
    fn test_paragraph_empty() {
        let ctx = LineBreakingContext::new(20, 25);
        // No words added

        let formatted = ctx.format();
        assert!(
            formatted.is_empty(),
            "Empty paragraph should produce empty string"
        );
    }

    #[test]
    fn test_paragraph_whitespace_only() {
        let mut ctx = LineBreakingContext::new(20, 25);
        ctx.add_text("   "); // Only whitespace

        let formatted = ctx.format();
        // Whitespace-only text should produce empty result
        // (split_whitespace returns no words)
        assert!(formatted.is_empty());
    }

    #[test]
    fn test_paragraph_large_width() {
        // Paragraph with large max width (minimal breaking)
        let mut ctx = LineBreakingContext::new(100, 120);
        ctx.add_text("This is a paragraph that should fit on a single line because the max width is very large");

        let formatted = ctx.format();
        let lines: Vec<&str> = formatted.lines().collect();

        // Should be on one line
        assert_eq!(lines.len(), 1, "Should fit on one line with large width");
    }

    #[test]
    fn test_paragraph_small_width() {
        // Paragraph with small max width (aggressive breaking)
        let mut ctx = LineBreakingContext::new(10, 12);
        ctx.add_text("This is a test");

        let formatted = ctx.format();
        let lines: Vec<&str> = formatted.lines().collect();

        // Should produce multiple short lines
        assert!(
            lines.len() >= 2,
            "Should have multiple lines with small width"
        );

        for line in &lines {
            let width = unicode_width::width(line) as usize;
            assert!(width <= 12, "Line exceeds max width: {}", line);
        }
    }

    #[test]
    fn test_paragraph_with_numbers_and_punctuation() {
        let mut ctx = LineBreakingContext::new(20, 25);
        ctx.add_text("Version 1.2.3 is released on 2024-01-15! Check it out.");

        let formatted = ctx.format();

        for line in formatted.lines() {
            let width = unicode_width::width(line) as usize;
            assert!(width <= 25, "Line exceeds max width: {}", line);
        }
    }

    // Tests for block quote line breaking

    #[test]
    fn test_block_quote_line_breaking() {
        // First line prefix is empty (BlockQuote handler already outputs "> ")
        // Continuation lines have "> " prefix
        let mut ctx = LineBreakingContext::with_prefixes(20, 30, "", "> ");
        ctx.add_text("This is a blockquote with some text that should wrap");

        let formatted = ctx.format();
        let lines: Vec<&str> = formatted.lines().collect();

        // First line should NOT have prefix (it's already output by BlockQuote handler)
        // Continuation lines should have "> " prefix
        if lines.len() > 1 {
            for line in &lines[1..] {
                assert!(
                    line.starts_with("> "),
                    "Block quote continuation line should start with '> ': {}",
                    line
                );
            }
        }
    }

    #[test]
    fn test_nested_block_quote_line_breaking() {
        // Nested block quote: continuation lines have "> > " prefix
        let mut ctx = LineBreakingContext::with_prefixes(15, 20, "", "> > ");
        ctx.add_text("Nested blockquote with some text");

        let formatted = ctx.format();
        let lines: Vec<&str> = formatted.lines().collect();

        // Continuation lines should start with "> > "
        if lines.len() > 1 {
            for line in &lines[1..] {
                assert!(
                    line.starts_with("> > "),
                    "Nested block quote continuation line should start with '> > ': {}",
                    line
                );
            }
        }
    }

    #[test]
    fn test_block_quote_single_line() {
        // Short block quote that fits on one line
        // First line prefix is empty, no continuation lines
        let mut ctx = LineBreakingContext::with_prefixes(20, 30, "", "> ");
        ctx.add_text("Short quote");

        let formatted = ctx.format();
        let lines: Vec<&str> = formatted.lines().collect();

        // Should be a single line without prefix
        assert_eq!(lines.len(), 1, "Short quote should be on one line");
        assert_eq!(lines[0], "Short quote");
    }

    #[test]
    fn test_block_quote_width_constraint() {
        // Test that block quote lines respect max width
        // First line: no prefix, continuation: "> "
        let mut ctx = LineBreakingContext::with_prefixes(15, 20, "", "> ");
        ctx.add_text("This is a longer text that should wrap properly");

        let formatted = ctx.format();

        // Check that all lines respect max width
        for line in formatted.lines() {
            let width = unicode_width::width(line) as usize;
            assert!(
                width <= 20,
                "Block quote line exceeds max width: {} (width: {})",
                line,
                width
            );
        }
    }

    #[test]
    fn test_triple_nested_block_quote() {
        // Test triple nesting
        let mut ctx = LineBreakingContext::with_prefixes(10, 15, "", "> > > ");
        ctx.add_text("Deeply nested quote");

        let formatted = ctx.format();
        let lines: Vec<&str> = formatted.lines().collect();

        // Continuation lines should start with "> > > "
        if lines.len() > 1 {
            for line in &lines[1..] {
                assert!(
                    line.starts_with("> > > "),
                    "Triple nested continuation line should start with '> > > ': {}",
                    line
                );
            }
        }
    }

    #[test]
    fn test_is_cjk_punctuation() {
        assert!(is_cjk_punctuation('。'), "'。' should be CJK punctuation");
        assert!(is_cjk_punctuation('，'), "'，' should be CJK punctuation");
        assert!(
            !is_cjk_punctuation('a'),
            "'a' should NOT be CJK punctuation"
        );
        assert!(
            !is_cjk_punctuation('1'),
            "'1' should NOT be CJK punctuation"
        );
    }

    #[test]
    fn test_split_cjk_text() {
        // Test splitting at punctuation marks (not at CJK/ASCII boundaries)
        // Punctuation is included with the preceding text
        let result = split_cjk_text("数字123");
        assert_eq!(result, vec!["数字123"]);

        let result = split_cjk_text("test中文");
        assert_eq!(result, vec!["test中文"]);

        // Punctuation '，' is included with preceding text "示例"
        let result = split_cjk_text("示例，包含");
        assert_eq!(result, vec!["示例，", "包含"], "Failed: {:?}", result);

        // Test longer text
        let result = split_cjk_text("单词和数字123");
        assert_eq!(result, vec!["单词和数字123"]);

        // Test with punctuation at end - punctuation is included with preceding text
        let result = split_cjk_text("单词和数字123。");
        assert_eq!(result, vec!["单词和数字123。"], "Failed: {:?}", result);
    }

    #[test]
    fn test_cjk_text_formatting() {
        // Test that add_text correctly handles CJK text
        // Note: add_text does NOT add CJK spacing - that is handled by add_cjk_spacing before this
        let mut ctx = LineBreakingContext::new(80, 80);
        ctx.add_text("单词和数字123。");

        // Check the words (only split at punctuation, punctuation stays with preceding text)
        assert_eq!(ctx.words.len(), 1);
        assert_eq!(ctx.words[0].text, "单词和数字123。");

        let formatted = ctx.format();
        // Without CJK spacing, there should be no space between CJK and number
        assert!(
            formatted.contains("单词和数字123"),
            "Should NOT have space between CJK and number without CJK spacing: {}",
            formatted
        );
    }

    #[test]
    fn test_cjk_text_formatting_with_spacing() {
        // Test that add_text correctly handles CJK text when CJK spacing is already applied
        let mut ctx = LineBreakingContext::new(80, 80);
        // Simulate CJK spacing applied before add_text
        ctx.add_text("单词和数字 123。");

        // Check the words (split by whitespace, then by punctuation)
        // "单词和数字" -> ["单词和数字"]
        // "123。" -> ["123。"] (punctuation stays with preceding text)
        assert_eq!(ctx.words.len(), 2);
        assert_eq!(ctx.words[0].text, "单词和数字");
        assert_eq!(ctx.words[1].text, "123。");

        let formatted = ctx.format();
        // The space between "单词和数字" and "123" should be preserved
        assert!(
            formatted.contains("单词和数字 123"),
            "Should have space between CJK and number with CJK spacing: {}",
            formatted
        );
    }
}

/// Check if a string contains CJK characters
fn contains_cjk(text: &str) -> bool {
    text.chars().any(|c| {
        // CJK Unified Ideographs
        (0x4E00..=0x9FFF).contains(&(c as u32))
            // CJK Unified Ideographs Extension A
            || (0x3400..=0x4DBF).contains(&(c as u32))
            // CJK Unified Ideographs Extension B-F
            || (0x20000..=0x2EBEF).contains(&(c as u32))
            // CJK Compatibility Ideographs
            || (0xF900..=0xFAFF).contains(&(c as u32))
            // CJK Symbols and Punctuation
            || (0x3000..=0x303F).contains(&(c as u32))
            // Hiragana
            || (0x3040..=0x309F).contains(&(c as u32))
            // Katakana
            || (0x30A0..=0x30FF).contains(&(c as u32))
            // Hangul Syllables
            || (0xAC00..=0xD7AF).contains(&(c as u32))
            // Hangul Jamo
            || (0x1100..=0x11FF).contains(&(c as u32))
            // Fullwidth ASCII variants
            || (0xFF01..=0xFF5E).contains(&(c as u32))
            // Halfwidth Katakana
            || (0xFF65..=0xFF9F).contains(&(c as u32))
    })
}

/// Split CJK text at punctuation marks for better line breaking
/// Note: This function does NOT split at CJK/ASCII boundaries - that's handled by CJK spacing
/// Returns a vector of string segments
fn split_cjk_text(text: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current_segment = String::new();

    for c in text.chars() {
        let is_punct = is_cjk_punctuation(c);

        // If this is punctuation, add it to current segment and end the segment
        // This allows line breaking after punctuation
        if is_punct {
            current_segment.push(c);
            result.push(current_segment.clone());
            current_segment.clear();
        } else {
            current_segment.push(c);
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

/// Check if a string starts with CJK punctuation
fn starts_with_cjk_punctuation(text: &str) -> bool {
    text.chars().next().map_or(false, is_cjk_punctuation)
}
