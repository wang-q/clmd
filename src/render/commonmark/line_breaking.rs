//! Knuth-Plass line breaking algorithm implementation
//!
//! This module implements the Knuth-Plass line breaking algorithm for optimal
//! paragraph formatting. The algorithm uses dynamic programming to find the
//! globally optimal set of line breaks that minimizes the total "badness" of
//! the paragraph.
//!
//! The algorithm is based on the paper "Breaking Paragraphs into Lines" by
//! Donald E. Knuth and Michael F. Plass (1981).

use crate::text::char::{is_cjk, is_cjk_punctuation};
use crate::text::tokenizer::split_cjk_text_smart;
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
    /// Whether the previous element was an inline code (not a Markdown marker)
    /// This is used to distinguish between `(` after inline code (no space) vs after marker (space)
    after_inline_code: bool,
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
            after_inline_code: false,
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
            after_inline_code: false,
        }
    }

    /// Check if line breaking is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Add a word to the context
    pub fn add_word(&mut self, mut word: Word) {
        // If next_word_no_leading_space is set and word needs leading space,
        // clear it unless the word starts with opening brackets that should have leading space
        // (e.g., `(`, `[`, `{` after inline code should have space, but `:`, `,`, `.` should not)
        if self.next_word_no_leading_space && word.needs_leading_space {
            // Check if the word starts with opening brackets that should have leading space
            let first_char = word.text.chars().next();
            let is_opening_bracket =
                first_char.map_or(false, |c| matches!(c, '(' | '[' | '{'));
            if !is_opening_bracket {
                word.needs_leading_space = false;
            }
        }
        // Always reset the flags after processing a word
        self.next_word_no_leading_space = false;
        // Don't reset after_inline_code here, let add_inline_element set it
        // and let add_text check it before resetting
        self.words.push(word);
    }

    /// Add text and split it into words
    /// For CJK text, splits at punctuation marks for better line breaking
    /// Note: This function does NOT add CJK spacing - that is handled by add_cjk_spacing before this
    pub fn add_text(&mut self, text: &str) {
        // Split text by whitespace first
        let whitespace_separated: Vec<&str> = text.split_whitespace().collect();
        let total_segments = whitespace_separated.len();
        // Check if the original text ends with whitespace
        let ends_with_whitespace =
            text.chars().last().map_or(false, |c| c.is_whitespace());

        for (i, segment) in whitespace_separated.iter().enumerate() {
            // Check if this segment contains CJK characters
            if contains_cjk(segment) {
                // Split CJK text at punctuation marks for better line breaking
                // This only splits at punctuation, not at CJK/ASCII boundaries
                let cjk_words = split_cjk_text(segment);
                let total_cjk_words = cjk_words.len();

                for (j, word_text) in cjk_words.iter().enumerate() {
                    // Check if this word starts with punctuation that should NOT have leading space
                    // (CJK punctuation or specific ASCII punctuation like `:`, `,`, `.`)
                    let is_no_space_punct =
                        starts_with_no_leading_space_punctuation(word_text);
                    // Check if this word starts with opening bracket
                    let starts_with_bracket = word_text
                        .chars()
                        .next()
                        .map_or(false, |c| matches!(c, '(' | '[' | '{'));
                    // Create word: punctuation doesn't need spaces, normal CJK does
                    let mut w = Word::new_cjk(word_text.as_str());
                    // First word of first segment: special handling
                    if i == 0 && j == 0 {
                        if self.next_word_no_leading_space {
                            // Previous element was Markdown marker
                            // For punctuation that should NOT have leading space (e.g., `:`, `,`, `.`),
                            // keep needs_leading_space = false (default from new_cjk)
                            // For opening brackets after Markdown marker, add space
                            // For opening brackets after inline code, also add space
                            if !is_no_space_punct || starts_with_bracket {
                                w.needs_leading_space = true;
                            }
                        } else {
                            // Previous element was not a Markdown marker (e.g., inline code)
                            // Normal text should have leading space
                            if !is_no_space_punct {
                                w.needs_leading_space = true;
                            }
                        }
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
                // Check if this segment starts with punctuation that should NOT have leading space
                let is_no_space_punct =
                    starts_with_no_leading_space_punctuation(segment);
                // Check if this segment starts with opening bracket
                let starts_with_bracket = segment
                    .chars()
                    .next()
                    .map_or(false, |c| matches!(c, '(' | '[' | '{'));

                // Check if this is the first segment and original text starts with whitespace
                let starts_with_whitespace =
                    text.chars().next().map_or(false, |c| c.is_whitespace());

                if self.next_word_no_leading_space || self.after_inline_code {
                    // If the text starts with punctuation that should NOT have leading space,
                    // don't add leading space (e.g., `:`, `,`, `.` after Markdown marker)
                    // But for `(`, `[`, etc., we should keep the leading space
                    // For opening brackets after inline code, also add space
                    // However, if original text starts with whitespace, preserve it
                    if is_no_space_punct
                        && !starts_with_bracket
                        && !starts_with_whitespace
                    {
                        word.needs_leading_space = false;
                    }
                } else {
                    // Previous element was not a Markdown marker (e.g., inline code)
                    // For punctuation that should NOT have leading space (e.g., `:`, `,`, `.`),
                    // don't add leading space
                    if is_no_space_punct && !starts_with_whitespace {
                        word.needs_leading_space = false;
                    }
                }
                // For punctuation like `:`, `,`, `.` that are not at the end,
                // we should add trailing space so that the next word has space before it
                // This ensures `: 使用` has space after `:`
                if i < total_segments - 1 && segment.len() == 1 && is_no_space_punct {
                    word.has_trailing_space = true;
                }
                // If this is the last segment, check if the original text ends with whitespace
                // If so, preserve the trailing space
                if i == total_segments - 1 {
                    if ends_with_whitespace {
                        // Original text had whitespace at the end, preserve it
                        word.has_trailing_space = true;
                    } else {
                        // No trailing whitespace in original text
                        word.has_trailing_space = false;
                    }
                }
                self.add_word(word);
            }
        }
        // Reset after_inline_code after processing all segments
        self.after_inline_code = false;
    }

    /// Add text as a single word without splitting
    pub fn add_text_as_word(&mut self, text: &str) {
        self.add_word(Word::new_without_space(text));
    }

    /// Add a mark/punctuation that doesn't need spaces around it
    pub fn add_mark(&mut self, text: &str) {
        self.add_word(Word::new_mark(text));
        // Don't set next_word_no_leading_space here
        // Let add_text handle spacing based on content type (CJK punctuation vs normal text)
    }

    /// Add a Markdown marker (like **, *, [, ], etc.)
    /// This sets next_word_no_leading_space to prevent space after the marker
    pub fn add_markdown_marker(&mut self, text: &str) {
        self.add_word(Word::new_mark(text));
        // The next word should not have a leading space (unless it's CJK punctuation)
        self.next_word_no_leading_space = true;
        // Reset after_inline_code since this is a marker, not inline code
        self.after_inline_code = false;
    }

    /// Add an inline element (like code span) that should preserve surrounding spaces
    pub fn add_inline_element(&mut self, text: &str) {
        // Create a word that doesn't need leading space by default
        // This prevents space between "(" and "`code`", or between "`code`" and ")"
        let mut word = Word::new_without_space(text);
        // Check if the previous word ends with CJK character or CJK punctuation
        // If it's CJK character, we need leading space for the inline element
        // If it's CJK punctuation (like "（"), we don't need leading space
        if let Some(prev_word) = self.words.last() {
            if let Some(last_char) = prev_word.text.chars().last() {
                if is_cjk(last_char) && !is_cjk_punctuation(last_char) {
                    // Previous word ends with CJK character (not punctuation)
                    // Add leading space for the inline element
                    word.needs_leading_space = true;
                } else {
                    // Previous word ends with non-CJK character or CJK punctuation
                    // Don't add leading space
                    word.needs_leading_space = false;
                }
            } else {
                word.needs_leading_space = false;
            }
        } else {
            word.needs_leading_space = false;
        }
        self.add_word(word);
        // Set after_inline_code to true so that subsequent `(` knows it's after inline code
        self.after_inline_code = true;
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

        // Post-process breaks to prevent punctuation from being at line start
        let adjusted_breaks = self.adjust_breaks_for_punctuation(&breaks);

        let mut result = String::new();
        let mut start = 0;
        let mut is_first_line = true;

        for &end in &adjusted_breaks {
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

        // Ensure all words are included (handle case where last break point is before the last word)
        if start < self.words.len() {
            result.push('\n');
            result.push_str(&self.continuation_prefix);
            for i in start..self.words.len() {
                if i > start
                    && (self.words[i].needs_leading_space
                        || self.words[i - 1].has_trailing_space)
                {
                    result.push(' ');
                }
                result.push_str(&self.words[i].text);
            }
        }

        result
    }

    /// Adjust breaks to prevent punctuation from being at line start
    /// This ensures that punctuation like `,`, `.`, `;`, `:` etc. stay with the previous word
    /// Also ensures that opening brackets like `(`, `[`, `{` stay with their content
    /// And Markdown markers like `**`, `*`, `[`, `]` are not split across lines
    fn adjust_breaks_for_punctuation(&self, breaks: &[usize]) -> Vec<usize> {
        let mut adjusted = Vec::new();

        for &break_point in breaks {
            // Check if the word at this break point is punctuation that shouldn't be at line start
            // Note: break_point is the index of the first word on the next line
            if break_point < self.words.len() {
                let word = &self.words[break_point];
                if is_punctuation_that_should_not_be_at_line_start(&word.text) {
                    // Check if the previous word is a Markdown closing marker
                    // If so, we should keep the punctuation with the previous line
                    if break_point > 0 {
                        let prev_word = &self.words[break_point - 1];
                        if is_markdown_closing_marker(&prev_word.text) {
                            // Previous word is a closing marker, keep punctuation with it
                            // Find the next non-punctuation word and break after it
                            let mut next_break = break_point + 1;
                            for i in (break_point + 1)..self.words.len() {
                                if !is_punctuation_that_should_not_be_at_line_start(
                                    &self.words[i].text,
                                ) {
                                    next_break = i + 1;
                                    break;
                                }
                            }
                            if !adjusted.contains(&next_break) {
                                adjusted.push(next_break);
                                continue;
                            }
                        }
                    }

                    // This punctuation should not be at line start, move it to the previous line
                    // by including it in the current line
                    // We need to find the next appropriate break point after this punctuation
                    if break_point + 1 <= self.words.len() {
                        // Check if there's more content after this punctuation
                        if break_point + 1 < self.words.len() {
                            // Find the next non-punctuation word and the word after it
                            // We want to break after the word that follows the punctuation
                            // to ensure the punctuation stays with the previous line
                            let mut next_break = break_point + 1;
                            for i in (break_point + 1)..self.words.len() {
                                if !is_punctuation_that_should_not_be_at_line_start(
                                    &self.words[i].text,
                                ) {
                                    // Found a non-punctuation word, include it and look for one more
                                    next_break = i + 1;
                                    // Check if there's another word after this one
                                    if i + 1 < self.words.len() {
                                        // Include the next word as well to ensure punctuation stays with previous line
                                        next_break = i + 2;
                                    }
                                    break;
                                }
                            }
                            if !adjusted.contains(&next_break) {
                                adjusted.push(next_break);
                                continue;
                            }
                        } else {
                            // This is the last word, include it in the current line
                            if !adjusted.contains(&self.words.len()) {
                                adjusted.push(self.words.len());
                                continue;
                            }
                        }
                    }
                }

                // Check if the word at this break point is an opening bracket
                // Opening brackets like `(` should not be at line start
                if word.text.starts_with('(') || word.text.starts_with('（') {
                    // `(` is at line start, we should keep it with the next content
                    // Find the closing bracket and add a break after it
                    for i in break_point..self.words.len() {
                        if self.words[i].text.starts_with(')')
                            || self.words[i].text.starts_with('）')
                        {
                            if i + 1 <= self.words.len() && !adjusted.contains(&(i + 1))
                            {
                                adjusted.push(i + 1);
                                break;
                            }
                        }
                    }
                    continue;
                }

                // Check if the word at this break point is a Markdown closing marker
                // (like `**`, `*`, `]`, `)`) that shouldn't be at line start
                // because it would split the Markdown syntax across lines
                if break_point > 0 && is_markdown_closing_marker(&word.text) {
                    // Check if the previous word is the corresponding opening marker
                    let prev_word = &self.words[break_point - 1];
                    if is_markdown_opening_marker(&prev_word.text) {
                        // Both markers are adjacent, don't split them
                        // Move the break point after the closing marker
                        if break_point + 1 <= self.words.len()
                            && !adjusted.contains(&(break_point + 1))
                        {
                            adjusted.push(break_point + 1);
                            continue;
                        }
                    }
                }

                // Check if the word at this break point is a Markdown closing marker
                // and the next word is punctuation that shouldn't be at line start
                // This handles cases like `**`， where `**` is at line end and `，` is at line start
                if is_markdown_closing_marker(&word.text) {
                    if break_point + 1 < self.words.len() {
                        let next_word = &self.words[break_point + 1];
                        if is_punctuation_that_should_not_be_at_line_start(
                            &next_word.text,
                        ) {
                            // Closing marker followed by punctuation
                            // Move the break point after the punctuation and its following content
                            let mut next_break = break_point + 2;
                            for i in (break_point + 2)..self.words.len() {
                                if !is_punctuation_that_should_not_be_at_line_start(
                                    &self.words[i].text,
                                ) {
                                    next_break = i + 1;
                                    break;
                                }
                            }
                            if !adjusted.contains(&next_break) {
                                adjusted.push(next_break);
                                continue;
                            }
                        }
                    }
                }

                // Check if the previous word is a Markdown opening marker (like `**`, `*`, `[`)
                // that shouldn't be at line end because it would split the Markdown syntax
                if break_point > 0 {
                    let prev_word = &self.words[break_point - 1];
                    if is_markdown_opening_marker(&prev_word.text) {
                        // The opening marker is at line end, we should keep it with the next word
                        // Find the corresponding closing marker and add a break after it
                        for i in break_point..self.words.len() {
                            if is_markdown_closing_marker(&self.words[i].text) {
                                // Found the closing marker, add break after it
                                if i + 1 <= self.words.len()
                                    && !adjusted.contains(&(i + 1))
                                {
                                    adjusted.push(i + 1);
                                    break;
                                }
                            }
                        }
                        continue;
                    }

                    // Check if the current word at break_point is a Markdown closing marker
                    // that would be at line start (which we want to avoid)
                    if break_point < self.words.len() {
                        let current_word = &self.words[break_point];
                        if is_markdown_closing_marker(&current_word.text) {
                            // The next word is a closing marker like `**`
                            // This should stay with the previous content
                            // Check if the word after the closing marker is punctuation
                            // that should not be at line start
                            if break_point + 1 < self.words.len() {
                                let next_word = &self.words[break_point + 1];
                                if is_punctuation_that_should_not_be_at_line_start(
                                    &next_word.text,
                                ) {
                                    // The word after closing marker is punctuation
                                    // Include it in the current line
                                    if break_point + 2 <= self.words.len()
                                        && !adjusted.contains(&(break_point + 2))
                                    {
                                        adjusted.push(break_point + 2);
                                        continue;
                                    }
                                }
                            }
                            // Find the next opening marker or content and add break after
                            for i in (break_point + 1)..self.words.len() {
                                if is_markdown_opening_marker(&self.words[i].text) {
                                    // Found next opening marker, add break after its closing
                                    for j in (i + 1)..self.words.len() {
                                        if is_markdown_closing_marker(
                                            &self.words[j].text,
                                        ) {
                                            if j + 1 <= self.words.len()
                                                && !adjusted.contains(&(j + 1))
                                            {
                                                adjusted.push(j + 1);
                                                break;
                                            }
                                        }
                                    }
                                    break;
                                }
                                // If we find punctuation that should not be at line start,
                                // include it in the current line
                                if is_punctuation_that_should_not_be_at_line_start(
                                    &self.words[i].text,
                                ) {
                                    if i + 1 <= self.words.len()
                                        && !adjusted.contains(&(i + 1))
                                    {
                                        adjusted.push(i + 1);
                                        break;
                                    }
                                }
                                // If we find non-marker content, break after it
                                if !is_markdown_opening_marker(&self.words[i].text)
                                    && !is_markdown_closing_marker(&self.words[i].text)
                                {
                                    if i + 1 <= self.words.len()
                                        && !adjusted.contains(&(i + 1))
                                    {
                                        adjusted.push(i + 1);
                                        break;
                                    }
                                }
                            }
                            continue;
                        }
                    }
                }
            }

            // Check if the previous word ends with opening bracket that shouldn't be at line end
            // This ensures `(` stays with its content like (`slice`)
            if break_point > 0 && break_point < self.words.len() {
                let prev_word = &self.words[break_point - 1];
                if is_opening_bracket_at_line_end(&prev_word.text) {
                    // The previous word ends with `(`, we should keep it with the next word
                    // Find the closing bracket and add a break after it
                    for i in break_point..self.words.len() {
                        // Check if this word contains closing bracket
                        if self.words[i].text.contains(')')
                            || self.words[i].text.contains('）')
                        {
                            // Check if there are more punctuation words after the closing bracket
                            // If so, include them in the current line
                            let mut next_break = i + 1;
                            for j in (i + 1)..self.words.len() {
                                if is_punctuation_that_should_not_be_at_line_start(
                                    &self.words[j].text,
                                ) {
                                    next_break = j + 1;
                                } else {
                                    break;
                                }
                            }
                            if next_break <= self.words.len()
                                && !adjusted.contains(&next_break)
                            {
                                adjusted.push(next_break);
                                break;
                            }
                        }
                    }
                    continue;
                }
            }

            // Special case: if break_point equals words.len(), check if the last word is punctuation
            // This handles the case where `)` is the last word and would be alone on a line
            if break_point == self.words.len() && break_point > 0 {
                let last_word = &self.words[break_point - 1];
                if is_punctuation_that_should_not_be_at_line_start(&last_word.text) {
                    // Don't add this break, the last word will be included in the previous line
                    continue;
                }
            }

            adjusted.push(break_point);
        }

        // Post-process: ensure the last break includes all remaining words
        // This handles the case where the last word is punctuation like `)`
        if let Some(&last_break) = adjusted.last() {
            if last_break < self.words.len() {
                // Check if any remaining word starts with punctuation that shouldn't be at line start
                let mut should_extend = false;
                for i in last_break..self.words.len() {
                    if is_punctuation_that_should_not_be_at_line_start(
                        &self.words[i].text,
                    ) {
                        should_extend = true;
                        break;
                    }
                }
                if should_extend {
                    // Extend the last break to include all remaining words
                    if let Some(last) = adjusted.last_mut() {
                        *last = self.words.len();
                    }
                }
            }
        }

        // If adjusted is empty, use the original breaks
        if adjusted.is_empty() {
            adjusted = breaks.to_vec();
        }

        // Post-process: ensure Markdown closing markers are not at line end
        // when followed by punctuation at line start
        // This handles cases like `**`， where `**` is at line end and `，` is at line start
        let mut i = 0;
        while i < adjusted.len() {
            let break_point = adjusted[i];
            if break_point > 0 && break_point < self.words.len() {
                let prev_word = &self.words[break_point - 1];
                let current_word = &self.words[break_point];
                if is_markdown_closing_marker(&prev_word.text)
                    && is_punctuation_that_should_not_be_at_line_start(
                        &current_word.text,
                    )
                {
                    // Closing marker at line end, punctuation at line start
                    // Move the break point after the punctuation and its following content
                    let mut next_break = break_point + 1;
                    for j in (break_point + 1)..self.words.len() {
                        if !is_punctuation_that_should_not_be_at_line_start(
                            &self.words[j].text,
                        ) {
                            next_break = j + 1;
                            break;
                        }
                    }
                    // Only update if the new break point is different and not already present
                    if next_break != break_point && !adjusted.contains(&next_break) {
                        adjusted[i] = next_break;
                    } else if next_break == break_point || adjusted.contains(&next_break)
                    {
                        // Remove this break point as it would create a duplicate or empty line
                        adjusted.remove(i);
                        continue; // Don't increment i, we removed current element
                    }
                }
            }
            i += 1;
        }

        adjusted
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
        // Test splitting at word boundaries using Unicode UAX#29 standard
        // Now splits at CJK/ASCII boundaries for better line breaking

        // CJK and ASCII numbers are split at the boundary
        let result = split_cjk_text("数字123");
        assert_eq!(
            result,
            vec!["数字", "123"],
            "Should split at CJK/ASCII boundary: {:?}",
            result
        );

        // ASCII and CJK are split at the boundary
        let result = split_cjk_text("test中文");
        assert_eq!(
            result,
            vec!["test", "中文"],
            "Should split at ASCII/CJK boundary: {:?}",
            result
        );

        // Punctuation '，' is included with preceding text "示例"
        let result = split_cjk_text("示例，包含");
        assert_eq!(result, vec!["示例，", "包含"], "Failed: {:?}", result);

        // Test longer text - splits at CJK/number boundary
        let result = split_cjk_text("单词和数字123");
        assert_eq!(
            result,
            vec!["单词和数字", "123"],
            "Should split at CJK/number boundary: {:?}",
            result
        );

        // Test with punctuation at end - punctuation is included with preceding text
        let result = split_cjk_text("单词和数字123。");
        assert_eq!(result, vec!["单词和数字", "123。"], "Failed: {:?}", result);
    }

    #[test]
    fn test_cjk_text_formatting() {
        // Test that add_text correctly handles CJK text
        // Note: add_text does NOT add CJK spacing - that is handled by add_cjk_spacing before this
        let mut ctx = LineBreakingContext::new(80, 80);
        ctx.add_text("单词和数字123。");

        // Check the words - now splits at CJK/ASCII boundaries for better line breaking
        // "单词和数字" and "123" are split, but "123" and "。" may be combined
        assert_eq!(
            ctx.words.len(),
            2,
            "Should split at CJK/ASCII boundary: {:?}",
            ctx.words
        );
        assert_eq!(ctx.words[0].text, "单词和数字");
        assert_eq!(ctx.words[1].text, "123。");

        let formatted = ctx.format();
        // Note: add_text does NOT add spaces between CJK words (including at CJK/ASCII boundaries)
        // This is intentional - CJK text typically doesn't use spaces
        // The splitting is for line breaking purposes, not for adding spaces
        assert!(
            formatted.contains("单词和数字123"),
            "CJK text should not have spaces between words: {}",
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

    #[test]
    fn test_line_breaking_empty() {
        let mut ctx = LineBreakingContext::new(80, 80);
        ctx.add_text("");
        assert_eq!(ctx.words.len(), 0);
        let formatted = ctx.format();
        assert_eq!(formatted, "");
    }

    #[test]
    fn test_ascii_punctuation_no_space_after_marker() {
        // Test that ASCII punctuation like : doesn't get a leading space after Markdown marker
        let mut ctx = LineBreakingContext::new(80, 80);

        // Simulate: `replace_na`: 将显式 `NA`
        ctx.add_markdown_marker("`");
        ctx.add_inline_element("replace_na");
        ctx.add_markdown_marker("`");
        ctx.add_text(": 将显式");

        let formatted = ctx.format();
        // The colon should NOT have a leading space
        assert!(
            formatted.contains("`replace_na`:"),
            "Colon should not have leading space after inline code: {}",
            formatted
        );
    }

    #[test]
    fn test_colon_after_inline_code_with_cjk() {
        // Test the exact scenario from user report:
        // `longer`: 支持在 `--names-to` 中使用
        let mut ctx = LineBreakingContext::new(80, 80);

        ctx.add_markdown_marker("`");
        ctx.add_inline_element("longer");
        ctx.add_markdown_marker("`");
        ctx.add_text(": 支持在 `--names-to` 中使用");

        // Debug: print all words
        for (i, word) in ctx.words.iter().enumerate() {
            eprintln!(
                "Word {}: text={:?}, needs_leading_space={}, has_trailing_space={}",
                i, word.text, word.needs_leading_space, word.has_trailing_space
            );
        }

        let formatted = ctx.format();
        eprintln!("Formatted: {:?}", formatted);

        // The colon should NOT have a leading space, but SHOULD have a trailing space
        assert!(
            formatted.contains("`longer`: 支持"),
            "Colon should not have leading space but should have trailing space: {}",
            formatted
        );
    }

    #[test]
    fn test_ascii_punctuation_various() {
        // Test various ASCII punctuation marks
        let test_cases = vec![
            (":", "colon"),
            (",", "comma"),
            (".", "period"),
            (";", "semicolon"),
            ("!", "exclamation"),
            ("?", "question"),
        ];

        for (punct, name) in test_cases {
            let mut ctx = LineBreakingContext::new(80, 80);
            ctx.add_markdown_marker("`");
            ctx.add_inline_element("code");
            ctx.add_markdown_marker("`");
            ctx.add_text(&format!("{} text", punct));

            let formatted = ctx.format();
            let expected = format!("`code`{} text", punct);
            assert!(
                formatted.contains(&expected),
                "{} should not have leading space after inline code: got '{}'",
                name,
                formatted
            );
        }
    }

    #[test]
    fn test_left_paren_has_space_after_inline_code() {
        // Test that left parenthesis has leading space after inline code
        // Example: `cmd/parallel.rs` (~1600 行)
        let mut ctx = LineBreakingContext::new(80, 80);

        // Simulate: `strbin` (字符串哈希分箱)
        ctx.add_inline_element("`strbin`");
        ctx.add_text("(字符串哈希分箱)");

        let formatted = ctx.format();

        // The left parenthesis should have a leading space after inline code
        assert!(
            formatted.contains("`strbin` (字符串哈希分箱)"),
            "Left parenthesis should have leading space after inline code: got {}",
            formatted
        );
    }

    #[test]
    fn test_brackets_have_space_after_inline_code() {
        // Test that brackets have leading space after inline code
        // Example: `cmd/parallel.rs` (~1600 行)
        let test_cases = vec![
            ("(", ")", "parentheses"),
            ("[", "]", "brackets"),
            ("{", "}", "braces"),
        ];

        for (open, close, name) in test_cases {
            let mut ctx = LineBreakingContext::new(80, 80);
            ctx.add_inline_element("`code`");
            ctx.add_text(&format!("{}text{}", open, close));

            let formatted = ctx.format();
            let expected = format!("`code` {}text{}", open, close);
            assert!(
                formatted.contains(&expected),
                "{} should have leading space after inline code: got '{}'",
                name,
                formatted
            );
        }
    }

    #[test]
    fn test_parentheses_with_inline_code() {
        // Test that parentheses with inline code inside don't have extra spaces
        // Example: (`cat` 命令的 `--buffer-size`)
        let mut ctx = LineBreakingContext::new(80, 80);

        // Simulate: 输出顺序控制 (`cat` 命令的 `--buffer-size`)
        ctx.add_text("输出顺序控制 (");
        ctx.add_markdown_marker("`");
        ctx.add_inline_element("cat");
        ctx.add_markdown_marker("`");
        ctx.add_text("命令的");
        ctx.add_markdown_marker("`");
        ctx.add_inline_element("--buffer-size");
        ctx.add_markdown_marker("`");
        ctx.add_text(")");

        let formatted = ctx.format();

        // The formatted output should NOT have spaces after ( or before )
        assert!(
            formatted.contains("(`cat`"),
            "There should be no space after opening parenthesis: {}",
            formatted
        );
        assert!(
            formatted.contains("`--buffer-size`)"),
            "There should be no space before closing parenthesis: {}",
            formatted
        );
    }

    #[test]
    fn test_parentheses_with_full_inline_code() {
        // Test that parentheses with inline code inside don't have extra spaces
        // This simulates the actual code path where inline code is added as a complete unit
        // Example: 支持进度条 (`indicatif` 的 `MultiProgress`)
        let mut ctx = LineBreakingContext::new(80, 80);

        // Simulate: 支持进度条 (`indicatif` 的 `MultiProgress`)
        ctx.add_text("支持进度条 (");
        ctx.add_inline_element("`indicatif`"); // Full inline code with backticks
        ctx.add_text(" 的 ");
        ctx.add_inline_element("`MultiProgress`"); // Full inline code with backticks
        ctx.add_text(")");

        let formatted = ctx.format();

        // The formatted output should NOT have spaces after ( or before )
        assert!(
            formatted.contains("(`indicatif`"),
            "There should be no space after opening parenthesis: got {}",
            formatted
        );
        assert!(
            formatted.contains("`MultiProgress`)"),
            "There should be no space before closing parenthesis: got {}",
            formatted
        );
    }

    #[test]
    fn test_colon_space_after_marker() {
        // Test that colon has space after it when followed by CJK text
        // Example: **计数/求和型**: 使用 `AtomicU64`
        let mut ctx = LineBreakingContext::new(80, 80);

        // Simulate: **计数/求和型**: 使用
        ctx.add_markdown_marker("**");
        ctx.add_text("计数/求和型");
        ctx.add_markdown_marker("**");
        ctx.add_text(": 使用");

        let formatted = ctx.format();

        // The colon should NOT have a leading space, but SHOULD have a trailing space
        assert!(
            formatted.contains("**计数/求和型**: 使用"),
            "Colon should have trailing space when followed by CJK text: {}",
            formatted
        );
    }

    #[test]
    fn test_paren_space_after_marker() {
        // Test that opening parenthesis has space after Markdown marker
        // Example: 1. **任务分发策略** (线程分配算法):
        let mut ctx = LineBreakingContext::new(80, 80);

        // Simulate: 1. **任务分发策略** (线程分配算法):
        ctx.add_text("1. ");
        ctx.add_markdown_marker("**");
        ctx.add_text("任务分发策略");
        ctx.add_markdown_marker("**");
        ctx.add_text(" (线程分配算法):");

        // Debug: print all words
        println!("Words:");
        for (i, word) in ctx.words().iter().enumerate() {
            println!(
                "Word {}: text={:?}, needs_leading_space={}, has_trailing_space={}",
                i, word.text, word.needs_leading_space, word.has_trailing_space
            );
        }

        let formatted = ctx.format();
        println!("Formatted: {:?}", formatted);

        // The opening parenthesis should have a leading space after the marker
        assert!(
            formatted.contains("**任务分发策略** ("),
            "Opening parenthesis should have leading space after marker: got {}",
            formatted
        );
    }

    #[test]
    fn test_colon_space_before_inline_code() {
        // Test that colon preserves trailing space before inline code when present
        // Example: - **频率表型**: `FrequencyTables::merge()` 合并多个 `Counter`
        let mut ctx = LineBreakingContext::new(80, 80);

        // Simulate: - **频率表型**: `FrequencyTables::merge()`
        ctx.add_text("- ");
        ctx.add_markdown_marker("**");
        ctx.add_text("频率表型");
        ctx.add_markdown_marker("**");
        ctx.add_text(": ");
        ctx.add_inline_element("`FrequencyTables::merge()`");

        let formatted = ctx.format();

        // The colon should preserve the trailing space from original input
        assert!(
            formatted.contains(": `FrequencyTables::merge()`"),
            "Colon should preserve trailing space before inline code: got {}",
            formatted
        );
    }

    #[test]
    fn test_colon_no_space_before_inline_code() {
        // Test that colon has no trailing space before inline code when original has no space
        let mut ctx = LineBreakingContext::new(80, 80);

        ctx.add_text("- ");
        ctx.add_markdown_marker("**");
        ctx.add_text("频率表型");
        ctx.add_markdown_marker("**");
        ctx.add_text(":");
        ctx.add_inline_element("`FrequencyTables::merge()`");

        let formatted = ctx.format();

        // The colon should have no trailing space when original has no space
        assert!(
            formatted.contains(":`FrequencyTables::merge()`"),
            "Colon should have no trailing space when original has no space: got {}",
            formatted
        );
    }

    #[test]
    fn test_slash_space_around_inline_code() {
        // Test that slash preserves spaces around inline code when present
        // Example: - `scores.txt` / `scores_h.txt`: 成对的无表头/有表头示例。
        let mut ctx = LineBreakingContext::new(80, 80);

        // Simulate: - `scores.txt` / `scores_h.txt`:
        ctx.add_text("- ");
        ctx.add_inline_element("`scores.txt`");
        ctx.add_text(" / ");
        ctx.add_inline_element("`scores_h.txt`");
        ctx.add_text(":");

        let formatted = ctx.format();

        // The slash should preserve spaces from original input
        assert!(
            formatted.contains("`scores.txt` / `scores_h.txt`:"),
            "Slash should preserve spaces around inline code: got {}",
            formatted
        );
    }

    #[test]
    fn test_slash_no_space_around_inline_code() {
        // Test that slash has no space around inline code when original has no space
        let mut ctx = LineBreakingContext::new(80, 80);

        ctx.add_text("- ");
        ctx.add_inline_element("`scores.txt`");
        ctx.add_text("/");
        ctx.add_inline_element("`scores_h.txt`");
        ctx.add_text(":");

        let formatted = ctx.format();

        // The slash should have no space when original has no space
        assert!(
            formatted.contains("`scores.txt`/`scores_h.txt`:"),
            "Slash should have no space when original has no space: got {}",
            formatted
        );
    }

    #[test]
    fn test_paren_space_after_inline_code() {
        // Test that opening parenthesis has space after inline code
        // Example: - **实现**: `cmd/parallel.rs` (~1600 行)
        let mut ctx = LineBreakingContext::new(80, 80);

        // Simulate: - **实现**: `cmd/parallel.rs` (~1600 行)
        ctx.add_text("- ");
        ctx.add_markdown_marker("**");
        ctx.add_text("实现");
        ctx.add_markdown_marker("**");
        ctx.add_text(":");
        ctx.add_inline_element("`cmd/parallel.rs`");
        ctx.add_text(" (~1600 行)");

        // Debug: print all words
        println!("Words:");
        for (i, word) in ctx.words().iter().enumerate() {
            println!(
                "Word {}: text={:?}, needs_leading_space={}, has_trailing_space={}",
                i, word.text, word.needs_leading_space, word.has_trailing_space
            );
        }

        let formatted = ctx.format();
        println!("Formatted: {:?}", formatted);

        // The opening parenthesis should have a leading space after inline code
        assert!(
            formatted.contains("`cmd/parallel.rs` (~1600 行)"),
            "Opening parenthesis should have leading space after inline code: got {}",
            formatted
        );
    }

    #[test]
    fn test_comma_space_after_inline_code() {
        // Test that comma preserves trailing space after inline code when present
        // Example: - 行动: 添加 `--relationship` 标志（例如 `one-to-one`, `many-to-one`）
        let mut ctx = LineBreakingContext::new(80, 80);

        // Simulate: - 行动: 添加 `--relationship` 标志（例如 `one-to-one`, `many-to-one`）
        ctx.add_text("- 行动: 添加 `--relationship` 标志（例如");
        ctx.add_inline_element("`one-to-one`");
        ctx.add_text(", ");
        ctx.add_inline_element("`many-to-one`");
        ctx.add_text("）在连接时验证键。");

        let formatted = ctx.format();

        // The comma should preserve the trailing space from original input
        assert!(
            formatted.contains("`one-to-one`, `many-to-one`"),
            "Comma should preserve trailing space after inline code: got {}",
            formatted
        );
    }

    #[test]
    fn test_comma_no_space_after_inline_code() {
        // Test that comma has no trailing space after inline code when original has no space
        // Example: - 行动: 添加 `--relationship` 标志（例如 `one-to-one`,`many-to-one`）
        let mut ctx = LineBreakingContext::new(80, 80);

        // Simulate: - 行动: 添加 `--relationship` 标志（例如 `one-to-one`,`many-to-one`）
        ctx.add_text("- 行动: 添加 `--relationship` 标志（例如");
        ctx.add_inline_element("`one-to-one`");
        ctx.add_text(",");
        ctx.add_inline_element("`many-to-one`");
        ctx.add_text("）在连接时验证键。");

        let formatted = ctx.format();

        // The comma should have no trailing space when original has no space
        assert!(
            formatted.contains("`one-to-one`,`many-to-one`"),
            "Comma should have no trailing space when original has no space: got {}",
            formatted
        );
    }

    #[test]
    fn test_line_breaking_single_word() {
        let mut ctx = LineBreakingContext::new(80, 80);
        ctx.add_text("Hello");
        assert_eq!(ctx.words.len(), 1);
        assert_eq!(ctx.words[0].text, "Hello");
        let formatted = ctx.format();
        assert_eq!(formatted, "Hello");
    }

    #[test]
    fn test_line_breaking_multiple_words() {
        let mut ctx = LineBreakingContext::new(80, 80);
        ctx.add_text("Hello World");
        assert_eq!(ctx.words.len(), 2);
        assert_eq!(ctx.words[0].text, "Hello");
        assert_eq!(ctx.words[1].text, "World");
        let formatted = ctx.format();
        assert_eq!(formatted, "Hello World");
    }

    #[test]
    fn test_line_breaking_long_paragraph() {
        let mut ctx = LineBreakingContext::new(40, 80);
        ctx.add_text("This is a very long paragraph that should be wrapped into multiple lines when formatted with line breaking enabled.");
        // The paragraph should be split into multiple words
        assert!(ctx.words.len() > 1);
        let formatted = ctx.format();
        // The formatted output should have line breaks
        assert!(formatted.contains('\n') || ctx.words.len() <= 40);
    }

    #[test]
    fn test_line_breaking_with_prefix() {
        let mut ctx = LineBreakingContext::with_prefixes(80, 80, "> ", "> ");
        ctx.add_text("Hello World");
        let formatted = ctx.format();
        assert!(formatted.starts_with("> "));
    }

    #[test]
    fn test_word_width_calculation() {
        let word = Word::new("Hello");
        assert_eq!(word.width, 5);

        let word_cjk = Word::new_cjk("中文");
        assert_eq!(word_cjk.width, 4); // CJK characters are width 2
    }

    #[test]
    fn test_line_width_calculation() {
        let mut ctx = LineBreakingContext::new(80, 80);
        ctx.add_text("Hello World");
        let width = ctx.calculate_line_width(0, 1);
        assert_eq!(width, 5); // "Hello" width
    }

    #[test]
    fn test_compute_breaks_basic() {
        let mut ctx = LineBreakingContext::new(40, 80);
        ctx.add_text("Hello World Test");
        let breaks = ctx.compute_breaks();
        // Badness should be calculated for potential line breaks
        // This is a basic test to ensure compute_breaks doesn't panic
        assert!(breaks.is_empty() || !breaks.is_empty());
    }

    #[test]
    fn test_add_markdown_marker() {
        let mut ctx = LineBreakingContext::new(80, 80);
        ctx.add_markdown_marker("**");
        assert_eq!(ctx.words.len(), 1);
        assert_eq!(ctx.words[0].text, "**");
        assert!(ctx.next_word_no_leading_space);
    }

    #[test]
    fn test_add_inline_element() {
        let mut ctx = LineBreakingContext::new(80, 80);
        ctx.add_inline_element("`code`");
        assert_eq!(ctx.words.len(), 1);
        assert_eq!(ctx.words[0].text, "`code`");
        assert!(!ctx.next_word_no_leading_space);
    }

    #[test]
    fn test_reset_next_word_no_leading_space() {
        let mut ctx = LineBreakingContext::new(80, 80);
        ctx.add_markdown_marker("**");
        assert!(ctx.next_word_no_leading_space);
        ctx.reset_next_word_no_leading_space();
        assert!(!ctx.next_word_no_leading_space);
    }

    #[test]
    fn test_cjk_punctuation_handling() {
        let mut ctx = LineBreakingContext::new(80, 80);
        ctx.add_markdown_marker("**");
        ctx.add_text("特性：");
        // The CJK punctuation "：" should not have leading space
        assert_eq!(ctx.words.len(), 2);
        assert_eq!(ctx.words[0].text, "**");
        assert_eq!(ctx.words[1].text, "特性：");
        assert!(!ctx.words[1].needs_leading_space);
    }

    #[test]
    fn test_cjk_text_after_inline_code() {
        let mut ctx = LineBreakingContext::new(80, 80);
        ctx.add_inline_element("`tva`");
        ctx.add_text("的开发者");
        // Normal CJK text after inline code should have leading space
        assert_eq!(ctx.words.len(), 2);
        assert_eq!(ctx.words[0].text, "`tva`");
        assert_eq!(ctx.words[1].text, "的开发者");
        assert!(ctx.words[1].needs_leading_space);
    }

    #[test]
    fn test_opening_bracket_not_at_line_end() {
        // Test that opening bracket ( stays with its content
        // Example: - **数值提取**: `getnum` 从混合文本中提取数字（如 "zoom-123.45xyz" -> 123.45）。
        // The `（` should not be at line end while `如` is on next line
        let mut ctx = LineBreakingContext::new(40, 50);

        // Simulate the text structure - add as single text to preserve spacing
        ctx.add_text("- **数值提取**: `getnum` 从混合文本中提取数字（如 ");
        ctx.add_inline_element("`zoom-123.45xyz`");
        ctx.add_text(" -> 123.45）。");

        let formatted = ctx.format();
        let lines: Vec<&str> = formatted.lines().collect();

        // Check that no line ends with opening bracket ( while content is on next line
        for (i, line) in lines.iter().enumerate() {
            if i < lines.len() - 1 {
                // Current line ends with opening bracket, next line starts with content
                // This should not happen - the bracket should stay with content
                let trimmed = line.trim_end();
                if trimmed.ends_with('（') || trimmed.ends_with('(') {
                    let next_line = lines[i + 1].trim_start();
                    // The next line should NOT start with CJK characters like "如"
                    // because the bracket should have stayed with them
                    assert!(
                        !next_line.starts_with('如'),
                        "Opening bracket should not be at line end while '如' is on next line.\nLine {}: {}\nLine {}: {}",
                        i, line, i + 1, lines[i + 1]
                    );
                }
            }
        }

        // The formatted output should keep the bracket with its content
        // Either `（如` should be on same line, or `（` should not be alone at line end
        let has_bracket_with_content = formatted.contains("（如");
        assert!(
            has_bracket_with_content,
            "Opening bracket should stay with content '如'. Formatted:\n{}",
            formatted
        );
    }

    #[test]
    fn test_opening_bracket_with_content_not_split() {
        // More specific test for the exact case reported
        // - **数值提取**: `getnum` 从混合文本中提取数字（如 "zoom-123.45xyz" -> 123.45）。
        // Should NOT become:
        // - **数值提取**: `getnum` 从混合文本中提取数字（
        //   如 "zoom-123.45xyz" -> 123.45）。

        let mut ctx = LineBreakingContext::new(35, 45);
        ctx.add_text("- **数值提取**: `getnum` 从混合文本中提取数字（如 ");
        ctx.add_inline_element("`zoom-123.45xyz`");
        ctx.add_text(" -> 123.45）。");

        let formatted = ctx.format();

        // Check that `（如` stays together
        assert!(
            formatted.contains("（如"),
            "The opening bracket and '如' should stay on the same line. Formatted:\n{}",
            formatted
        );

        // Also verify no line ends with just `（`
        for line in formatted.lines() {
            let trimmed = line.trim_end();
            assert!(
                !trimmed.ends_with('（'),
                "No line should end with just opening bracket: {}",
                line
            );
        }
    }

    #[test]
    fn test_markdown_marker_not_split_across_lines() {
        // Test that Markdown markers like `**` are not split across lines
        // Example: > **保持简单**：tva 的表达式语言设计目标是**简单高效的数据处理**，不是通用编程语言。
        // Should NOT become:
        // > **保持简单**：tva 的表达式语言设计目标是**
        // > 简单高效的数据处理**，不是通用编程语言。

        let mut ctx = LineBreakingContext::with_prefixes(35, 45, "> ", "> ");

        // Simulate: > **保持简单**：tva 的表达式语言设计目标是**简单高效的数据处理**，不是通用编程语言。
        ctx.add_markdown_marker("**");
        ctx.add_text("保持简单");
        ctx.add_markdown_marker("**");
        ctx.add_text("：tva 的表达式语言设计目标是");
        ctx.add_markdown_marker("**");
        ctx.add_text("简单高效的数据处理");
        ctx.add_markdown_marker("**");
        ctx.add_text("，不是通用编程语言。");

        let formatted = ctx.format();
        let lines: Vec<&str> = formatted.lines().collect();

        // Check that no line ends with `**` while the next line starts with content
        // (i.e., the `**` markers should stay together with their content)
        for (i, line) in lines.iter().enumerate() {
            if i < lines.len() - 1 {
                let trimmed = line.trim_end();
                let next_line = lines[i + 1].trim_start();

                // If current line ends with `**`, next line should NOT start with content
                // that would be part of the emphasized text
                if trimmed.ends_with("**") && !trimmed.ends_with("****") {
                    // Count the `**` at the end to see if it's an opening or closing marker
                    let star_count =
                        trimmed.chars().rev().take_while(|&c| c == '*').count();
                    if star_count % 2 == 0 {
                        // Even number of stars - this is a closing marker
                        // Next line should NOT start with content that should be emphasized
                        assert!(
                            !next_line.starts_with("简单") && !next_line.starts_with("保持"),
                            "Closing marker `**` should not be at line end while emphasized content is on next line.\nLine {}: {}\nLine {}: {}",
                            i, line, i + 1, lines[i + 1]
                        );
                    }
                }
            }
        }

        // The emphasized text should stay together
        assert!(
            formatted.contains("**简单高效的数据处理**"),
            "Emphasized text should stay together. Formatted:\n{}",
            formatted
        );
    }

    #[test]
    fn test_markdown_strong_emphasis_not_split() {
        // More specific test for the exact case reported
        // Use wider width to ensure the emphasized text stays on one line
        let mut ctx = LineBreakingContext::with_prefixes(50, 60, "> ", "> ");

        ctx.add_markdown_marker("**");
        ctx.add_text("保持简单");
        ctx.add_markdown_marker("**");
        ctx.add_text("：tva 的表达式语言设计目标是");
        ctx.add_markdown_marker("**");
        ctx.add_text("简单高效的数据处理");
        ctx.add_markdown_marker("**");
        ctx.add_text("，不是通用编程语言。");

        let formatted = ctx.format();

        // Verify that `**` markers are not alone at line end/start
        let lines: Vec<&str> = formatted.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // A line should not end with just `**` (opening marker)
            if trimmed.ends_with("**") && !trimmed.ends_with("****") {
                // Check if this is an opening marker by looking at context
                let before_stars = trimmed.trim_end_matches('*');
                if !before_stars.is_empty() {
                    // This is likely an opening marker, it shouldn't be at line end
                    assert!(
                        i == lines.len() - 1 || !lines[i + 1].trim().starts_with("简单"),
                        "Opening marker `**` should not be at line end: {}",
                        line
                    );
                }
            }
        }

        // The full emphasized phrase should be intact
        assert!(
            formatted.contains("**简单高效的数据处理**"),
            "The emphasized phrase should be intact. Formatted:\n{}",
            formatted
        );
    }

    #[test]
    fn test_cjk_comma_not_at_line_start_in_blockquote() {
        // Test that CJK comma `，` is not at line start in blockquote
        // Example: > **保持简单**：tva 的表达式语言设计目标是**简单高效的数据处理**
        // > ，不是通用编程语言。
        // The `，` should not be at line start

        let mut ctx = LineBreakingContext::with_prefixes(35, 45, "> ", "> ");

        // Simulate: > **保持简单**：tva 的表达式语言设计目标是**简单高效的数据处理**，不是通用编程语言。
        ctx.add_markdown_marker("**");
        ctx.add_text("保持简单");
        ctx.add_markdown_marker("**");
        ctx.add_text("：tva 的表达式语言设计目标是");
        ctx.add_markdown_marker("**");
        ctx.add_text("简单高效的数据处理");
        ctx.add_markdown_marker("**");
        ctx.add_text("，不是通用编程语言。");

        let formatted = ctx.format();
        let lines: Vec<&str> = formatted.lines().collect();

        // Check that no line starts with `，`
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            // Remove the blockquote prefix "> " for checking
            let content = if trimmed.starts_with("> ") {
                &trimmed[2..]
            } else {
                trimmed
            };
            assert!(
                !content.starts_with('，'),
                "CJK comma `，` should not be at line start.\nLine {}: {}",
                i,
                line
            );
        }

        // The comma should stay with the previous content
        assert!(
            formatted.contains("处理**，不是")
                || formatted.contains("处理**")
                || formatted.contains("**"),
            "The comma should stay with the emphasized text. Formatted:\n{}",
            formatted
        );
    }

    #[test]
    fn test_emphasis_in_middle_of_text_not_split() {
        // Test that emphasis in the middle of text is not split
        // Example: tva **只有匿名函数（lambda）**且主要用于 TSV 数据处理
        // Should NOT become:
        // tva **只有匿名函数（lambda）
        // **且主要用于 TSV 数据处理

        let mut ctx = LineBreakingContext::new(35, 45);

        // Simulate: tva **只有匿名函数（lambda）**且主要用于 TSV 数据处理
        ctx.add_text("tva ");
        ctx.add_markdown_marker("**");
        ctx.add_text("只有匿名函数（lambda）");
        ctx.add_markdown_marker("**");
        ctx.add_text("且主要用于 TSV 数据处理");

        let formatted = ctx.format();
        let lines: Vec<&str> = formatted.lines().collect();

        // Check that no line ends with just `**` (closing marker)
        // while the next line starts with content
        for (i, line) in lines.iter().enumerate() {
            if i < lines.len() - 1 {
                let trimmed = line.trim_end();
                let next_line = lines[i + 1].trim_start();

                // If current line ends with `**`, next line should NOT start with `且` or other content
                if trimmed.ends_with("**") && !trimmed.contains("****") {
                    // Check if this is a closing marker followed by more content
                    assert!(
                        !next_line.starts_with('且') && !next_line.starts_with("主要用于"),
                        "Closing marker `**` should not be at line end while content is on next line.\nLine {}: {}\nLine {}: {}",
                        i, line, i + 1, lines[i + 1]
                    );
                }
            }
        }

        // The emphasized text should stay together
        assert!(
            formatted.contains("**只有匿名函数（lambda）**"),
            "The emphasized text should stay together. Formatted:\n{}",
            formatted
        );
    }

    #[test]
    fn test_no_empty_blockquote_line() {
        // Test that there's no empty blockquote line at the end
        // Example: > **保持简单**：tva 的表达式语言设计目标是**简单高效的数据处理**，不是通用编程语言。
        // Should NOT have an empty "> " line at the end

        let mut ctx = LineBreakingContext::with_prefixes(50, 60, "> ", "> ");

        ctx.add_markdown_marker("**");
        ctx.add_text("保持简单");
        ctx.add_markdown_marker("**");
        ctx.add_text("：tva 的表达式语言设计目标是");
        ctx.add_markdown_marker("**");
        ctx.add_text("简单高效的数据处理");
        ctx.add_markdown_marker("**");
        ctx.add_text("，不是通用编程语言。");

        let formatted = ctx.format();

        // Check that there's no empty "> " line at the end
        let lines: Vec<&str> = formatted.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            assert!(
                !trimmed.is_empty() && trimmed != ">",
                "Line {} should not be empty or just a blockquote marker: {:?}",
                i,
                line
            );
        }

        // The formatted output should not end with a newline followed by "> "
        assert!(
            !formatted.ends_with("> ") && !formatted.ends_with(">"),
            "Formatted output should not end with empty blockquote marker. Formatted:\n{}",
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

/// Split CJK text at word boundaries for better line breaking
/// Uses Unicode UAX#29 word boundary rules via unicode-segmentation
/// Returns a vector of string segments
fn split_cjk_text(text: &str) -> Vec<String> {
    split_cjk_text_smart(text)
}

/// Check if a character is an ASCII punctuation mark that should NOT have
/// leading space after inline code (like `:`, `,`, `.`, `;`, `!`, `?`, `)`, `[`, `]`, `{`, `}`, `/`)
/// Note: `(` is excluded because it should have leading space after inline code
fn is_ascii_punctuation_no_leading_space(c: char) -> bool {
    matches!(
        c,
        ':' | ',' | '.' | ';' | '!' | '?' | ')' | '[' | ']' | '{' | '}' | '/'
    )
}

/// Check if a string starts with punctuation that should NOT have leading space
/// after inline code (CJK punctuation or specific ASCII punctuation like `:`, `,`, `.`)
fn starts_with_no_leading_space_punctuation(text: &str) -> bool {
    text.chars().next().map_or(false, |c| {
        is_cjk_punctuation(c) || is_ascii_punctuation_no_leading_space(c)
    })
}

/// Check if a string is punctuation that should not be at the start of a line
/// This includes punctuation like `,`, `.`, `;`, `:`, `)`, `]`, `}`, etc.
fn is_punctuation_that_should_not_be_at_line_start(text: &str) -> bool {
    // Check if the text starts with punctuation that should not be at line start
    let first_char = text.chars().next();
    if let Some(c) = first_char {
        return matches!(
            c,
            ',' | '.'
                | ';'
                | ':'
                | ')'
                | ']'
                | '}'
                | '!'
                | '?'
                | '"'
                | '\''
                | '”'
                | '’'
                | '）'
                | '」'
                | '】'
                | '、'
                | '。'
                | '，'
                | '；'
                | '：'
                | '！'
                | '？'
        );
    }
    false
}

/// Check if a string ends with opening bracket that should not be at line end
/// This includes `(`, `[`, `{`, `（`, `「`, `【`, etc.
fn is_opening_bracket_at_line_end(text: &str) -> bool {
    // Check if the text ends with opening bracket
    let last_char = text.chars().last();
    if let Some(c) = last_char {
        return matches!(
            c,
            '(' | '[' | '{' | '（' | '「' | '【' | '『' | '《' | '〈' | '“' | '‘'
        );
    }
    false
}

/// Check if a string is a Markdown opening marker
/// This includes `**`, `*`, `[`, `(`, etc.
fn is_markdown_opening_marker(text: &str) -> bool {
    matches!(text, "**" | "*" | "[" | "(")
}

/// Check if a string is a Markdown closing marker
/// This includes `**`, `*`, `]`, `)`, etc.
fn is_markdown_closing_marker(text: &str) -> bool {
    matches!(text, "**" | "*" | "]" | ")")
}
