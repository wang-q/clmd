//! Knuth-Plass line breaking algorithm implementation
//!
//! This module implements the Knuth-Plass line breaking algorithm for optimal
//! paragraph formatting. The algorithm uses dynamic programming to find the
//! globally optimal set of line breaks that minimizes the total "badness" of
//! the paragraph.
//!
//! The algorithm is based on the paper "Breaking Paragraphs into Lines" by
//! Donald E. Knuth and Michael F. Plass (1981).

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
        }
    }
}

/// A breakpoint in the paragraph
#[derive(Debug, Clone)]
struct BreakPoint {
    /// Index of the word at this breakpoint
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
}

impl LineBreakingContext {
    /// Create a new line breaking context
    pub fn new(ideal_width: usize, max_width: usize) -> Self {
        Self {
            words: Vec::new(),
            ideal_width,
            max_width,
            enabled: max_width > 0,
        }
    }

    /// Check if line breaking is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Add a word to the context
    pub fn add_word(&mut self, word: Word) {
        self.words.push(word);
    }

    /// Add text and split it into words
    pub fn add_text(&mut self, text: &str) {
        // Split text by whitespace
        for (i, word_text) in text.split_whitespace().enumerate() {
            if i > 0 || text.starts_with(' ') {
                // Add space before this word if it's not the first word
                // or if the original text started with space
            }
            self.add_word(Word::new(word_text));
        }
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

        // Dynamic programming: for each possible breakpoint
        for j in 1..=n {
            let mut best_badness = f64::INFINITY;
            let mut best_prev = None;

            // Try all possible previous breakpoints
            for i in (0..j).rev() {
                let line_width = self.calculate_line_width(i, j);

                if line_width > self.max_width {
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

    /// Calculate the width of a line from word i to word j (exclusive)
    fn calculate_line_width(&self, start: usize, end: usize) -> usize {
        let mut width = 0;
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
            // No breaks needed, return all words joined
            return self.words.iter().map(|w| &*w.text).collect::<Vec<_>>().join(" ");
        }

        let mut result = String::new();
        let mut start = 0;

        for &end in &breaks {
            // Add words from start to end
            for i in start..end {
                if i > start {
                    result.push(' ');
                }
                result.push_str(&self.words[i].text);
            }
            result.push('\n');
            start = end;
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
    let diff = if line_width > ideal_width {
        line_width - ideal_width
    } else {
        ideal_width - line_width
    };
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
}
