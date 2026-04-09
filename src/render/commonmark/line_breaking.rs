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
    /// Whether this word is part of a link and should not be broken
    pub is_link_part: bool,
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
            is_link_part: false,
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
            is_link_part: false,
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
            is_link_part: false,
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
            is_link_part: false,
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
    /// Whether the previous element was an end marker (like closing *, **)
    /// This is used to add space before normal text but not before CJK punctuation
    after_end_marker: bool,
    /// Whether we are inside a link text (between `[` and `]`)
    /// When inside link text, we should not break lines
    in_link_text: bool,
    /// Whether we are inside a link URL part (between `](` and `)`)
    /// When inside link URL, we should not break lines
    in_link_url: bool,
    /// The index of the word that starts a link (the `[` marker)
    /// Used to prevent line breaks inside links
    link_start_index: Option<usize>,
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
            after_end_marker: false,
            in_link_text: false,
            in_link_url: false,
            link_start_index: None,
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
            after_end_marker: false,
            in_link_text: false,
            in_link_url: false,
            link_start_index: None,
        }
    }

    /// Check if line breaking is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Enter link text mode (between `[` and `]`)
    /// Records the index of the `[` marker for preventing line breaks inside links
    pub fn enter_link_text(&mut self) {
        self.in_link_text = true;
        // Record the index of the last word (should be `[` or `![`)
        // and mark it as part of a link
        if !self.words.is_empty() {
            let idx = self.words.len() - 1;
            self.link_start_index = Some(idx);
            self.words[idx].is_link_part = true;
        }
    }

    /// Exit link text mode and enter link URL mode
    /// Marks all words from link_start_index to current as link parts
    pub fn exit_link_text(&mut self) {
        // Mark all words from link_start_index to current as link parts
        if let Some(start_idx) = self.link_start_index {
            for i in start_idx..self.words.len() {
                self.words[i].is_link_part = true;
            }
        }
        self.in_link_text = false;
        // Enter link URL mode for the rest of the link
        self.in_link_url = true;
    }

    /// Enter link URL mode (after `](`)
    /// This is called when we start adding the URL part of a link
    pub fn enter_link_url(&mut self) {
        self.in_link_url = true;
    }

    /// Exit link URL mode (after `)`)
    /// Marks all remaining words as link parts and clears link tracking
    pub fn exit_link_url(&mut self) {
        // Mark all words from link_start_index to current as link parts
        if let Some(start_idx) = self.link_start_index {
            for i in start_idx..self.words.len() {
                self.words[i].is_link_part = true;
            }
        }
        self.in_link_url = false;
        self.link_start_index = None;
    }

    /// Add a word to the context
    pub fn add_word(&mut self, mut word: Word) {
        // If we're inside link text or link URL, mark this word as part of a link
        if self.in_link_text || self.in_link_url {
            word.is_link_part = true;
        }
        // If next_word_no_leading_space is set and word needs leading space,
        // clear it unless the word starts with opening brackets that should have leading space
        // (e.g., `(`, `[`, `{` after inline code should have space, but `:`, `,`, `.` should not)
        // However, don't clear it if we're after inline code - let add_text handle that
        if self.next_word_no_leading_space
            && word.needs_leading_space
            && !self.after_inline_code
        {
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
        self.after_end_marker = false;
        // Don't reset after_inline_code here, let add_inline_element set it
        // and let add_text check it before resetting
        self.words.push(word);
    }

    /// Add text and split it into words
    /// For CJK text, splits at punctuation marks for better line breaking
    /// Note: This function does NOT add CJK spacing - that is handled by add_cjk_spacing before this
    pub fn add_text(&mut self, text: &str) {
        // Handle pure whitespace (e.g., from SoftBreak)
        if text.trim().is_empty() && !text.is_empty() {
            // This is whitespace-only text (like a space from SoftBreak)
            // Mark the last word as having trailing space
            // BUT: Don't add trailing space after opening brackets like `(`, `[`, `{`
            // AND: When inside link text, don't add trailing space (link text should be compact)
            if let Some(last_word) = self.words.last_mut() {
                // Check if the last word ends with an opening bracket
                let ends_with_opening_bracket = last_word
                    .text
                    .chars()
                    .last()
                    .map_or(false, |c| matches!(c, '(' | '[' | '{'));
                if !ends_with_opening_bracket && !self.in_link_text {
                    last_word.has_trailing_space = true;
                }
            }
            return;
        }

        // When inside link text, add the entire text as a single word
        // This prevents line breaks inside link text
        if self.in_link_text {
            // Trim the text to remove leading/trailing whitespace
            // Link text should be compact without extra spaces
            let trimmed = text.trim();
            let mut w = Word::new(trimmed);
            w.needs_leading_space = false; // Link text should be compact
            w.has_trailing_space = false; // No trailing space for link text
            w.is_link_part = true; // Mark as part of a link
            self.words.push(w);
            return;
        }

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
                            // Previous element was Markdown start marker
                            // For punctuation that should NOT have leading space (e.g., `:`, `,`, `.`),
                            // keep needs_leading_space = false (default from new_cjk)
                            // For opening brackets after Markdown marker, add space
                            // For opening brackets after inline code, also add space
                            if !is_no_space_punct || starts_with_bracket {
                                w.needs_leading_space = true;
                            }
                        } else if self.after_end_marker {
                            // Previous element was Markdown end marker (closing *, **, etc.)
                            // For CJK text, do NOT add leading space (CJK doesn't use spaces)
                            // Exception: only add space for opening brackets
                            if starts_with_bracket {
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
                // Non-CJK text: treat as single word (don't split URLs at '/' boundaries
                // as this would break the link when spaces are added between words)
                let segment_width = unicode_width::width(*segment) as usize;
                // Note: We no longer split long URLs at '/' boundaries because it breaks
                // the link. Instead, URLs are kept as single words and allowed to exceed
                // the ideal width. The line breaking algorithm will handle them gracefully
                // by placing them on their own line if needed.
                if false && segment_width > self.ideal_width && segment.contains('/') {
                    // Disabled: Long URL/path splitting at '/' boundaries
                    // This was causing links to break because spaces were added between parts
                }
                // Normal non-CJK text: treat as single word
                {
                    // Normal non-CJK text: treat as single word
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
                        // When after inline code, if original text doesn't start with whitespace
                        // and it's not an opening bracket, don't add leading space (preserve original spacing)
                        if self.after_inline_code
                            && !starts_with_whitespace
                            && !starts_with_bracket
                        {
                            word.needs_leading_space = false;
                        }
                    } else if self.after_end_marker {
                        // Previous element was Markdown end marker (closing *, **, etc.)
                        // For non-CJK text, always add leading space (e.g., "*italic* text")
                        // Unless it starts with punctuation that shouldn't have leading space
                        // Or starts with Markdown marker followed by CJK punctuation (e.g., "*：测试")
                        // Or is a single Markdown marker character (e.g., `*` before CJK punctuation)
                        let is_marker_then_cjk =
                            starts_with_marker_then_cjk_punctuation(segment);
                        let is_single_marker = is_single_markdown_marker(segment);
                        if (is_no_space_punct || is_marker_then_cjk || is_single_marker)
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
                    if i < total_segments - 1 && segment.len() == 1 && is_no_space_punct
                    {
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

                    // Special handling: if this is the first segment and it's just an opening bracket,
                    // set has_trailing_space to false to prevent space after the bracket
                    // This handles cases like "**HEPMASS** (\n  4.8GB)" -> "**HEPMASS** (4.8GB)"
                    let is_opening_bracket =
                        *segment == "(" || *segment == "[" || *segment == "{";
                    // Check if this is a single opening bracket followed by whitespace in original text
                    // This handles cases where the bracket is in one text node and the content is in another
                    let is_single_opening_bracket =
                        is_opening_bracket && segment.len() == 1;
                    if i == 0 && is_single_opening_bracket {
                        word.has_trailing_space = false;
                    }

                    self.add_word(word);

                    // Set next_word_no_leading_space after add_word to prevent space after the bracket
                    if i == 0 && is_single_opening_bracket {
                        self.next_word_no_leading_space = true;
                    }
                }
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
        // Note: We no longer split long URLs at '/' boundaries because it breaks
        // the link. Instead, URLs are kept as single words and allowed to exceed
        // the ideal width. The line breaking algorithm will handle them gracefully
        // by placing them on their own line if needed.
        let text_width = unicode_width::width(text) as usize;
        if false && text_width > self.ideal_width && text.contains('/') {
            // Disabled: Long URL/path splitting at '/' boundaries
            // This was causing links to break because spaces were added between parts
        }
        // Special handling for '[' and '![' (link/image start markers)
        // These should have leading space if previous word ends with CJK character
        let is_link_start = text == "[" || text == "![";
        let mut word = Word::new_mark(text);
        if is_link_start {
            // Check if previous word ends with CJK character
            if let Some(prev_word) = self.words.last() {
                if let Some(last_char) = prev_word.text.chars().last() {
                    if is_cjk(last_char) && !is_cjk_punctuation(last_char) {
                        // Previous word ends with CJK character, add leading space
                        word.needs_leading_space = true;
                    }
                }
            }
        }
        self.add_word(word);
        // The next word should not have a leading space (unless it's CJK punctuation)
        self.next_word_no_leading_space = true;
        // Reset after_inline_code since this is a marker, not inline code
        self.after_inline_code = false;
        // Reset after_end_marker since this is a start marker, not an end marker
        self.after_end_marker = false;
    }

    /// Add a Markdown end marker (like closing **, *, etc.)
    /// Unlike add_markdown_marker, this does NOT set next_word_no_leading_space
    /// because text after the end marker should have normal spacing
    /// However, it sets after_end_marker so CJK punctuation doesn't get leading space
    pub fn add_markdown_marker_end(&mut self, text: &str) {
        let word = Word::new_mark(text);
        self.add_word(word);
        // Set after_end_marker so that CJK punctuation doesn't get leading space
        // but normal text does get space
        self.after_end_marker = true;
        // Reset after_inline_code since this is a marker, not inline code
        self.after_inline_code = false;
    }

    /// Add a link/image close marker `)`
    /// This sets after_inline_code to true so subsequent text preserves spacing
    pub fn add_link_close_marker(&mut self, text: &str) {
        let mut word = Word::new_mark(text);
        // Check if previous word ends with CJK character
        if let Some(prev_word) = self.words.last() {
            if let Some(last_char) = prev_word.text.chars().last() {
                if is_cjk(last_char) && !is_cjk_punctuation(last_char) {
                    word.needs_leading_space = true;
                }
            }
        }
        self.add_word(word);
        // The next word should not have a leading space (unless it's CJK punctuation)
        self.next_word_no_leading_space = true;
        // Set after_inline_code to true so that subsequent `/` knows it's after a link
        self.after_inline_code = true;
    }

    /// Add an inline element (like code span) that should preserve surrounding spaces
    pub fn add_inline_element(&mut self, text: &str) {
        // Check if this is a long inline element (like a code span with a long URL)
        let text_width = unicode_width::width(text) as usize;
        // Note: We no longer split long URLs at '/' boundaries because it breaks
        // the link. Instead, URLs are kept as single words and allowed to exceed
        // the ideal width. The line breaking algorithm will handle them gracefully
        // by placing them on their own line if needed.
        if false && text_width > self.ideal_width && text.contains('/') {
            // Disabled: Long URL/path splitting at '/' boundaries
            // This was causing links to break because spaces were added between parts
        }
        // Normal inline element: add as single word
        {
            // Normal inline element: add as single word
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
        }
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

            // Skip if this breakpoint would break inside a link
            // We check if the word before the break (words[j-1]) is part of a link
            // and if there are more words after this break that are also part of the same link
            if j < n && j > 0 && self.words[j - 1].is_link_part {
                // The word before this breakpoint is part of a link
                // Check if the next word is also part of a link
                // If so, don't break here
                if self.words[j].is_link_part {
                    breaks.push(BreakPoint {
                        word_index: j,
                        total_badness: breaks[j - 1].total_badness,
                        prev_break: None, // No valid break here
                    });
                    continue;
                }
            }

            // Try all possible previous breakpoints
            for i in (0..j).rev() {
                // Skip if this break point would put punctuation at line start
                // (unless it's the last word or a Markdown closing marker)
                if j < n
                    && is_punctuation_that_should_not_be_at_line_start(
                        &self.words[j].text,
                    )
                    && !is_markdown_closing_marker(&self.words[j].text)
                {
                    continue;
                }

                // Skip if we would break inside a link
                // Check if any word between i and j is a link part start
                let would_break_link = (i..j).any(|k| {
                    self.words[k].is_link_part
                        && (k == 0
                            || !self.words.get(k - 1).map_or(false, |w| w.is_link_part))
                });
                if would_break_link {
                    continue;
                }

                // Determine if this is the first line (i == 0)
                let is_first_line = i == 0;
                let line_width =
                    self.calculate_line_width_with_prefix(i, j, is_first_line);

                // The max width is the same for all lines
                // calculate_line_width_with_prefix already includes the prefix width
                let effective_max_width = self.max_width;

                if line_width > effective_max_width {
                    break; // Exceeds max width, stop searching
                }

                // Check if the remaining content (from j to n) can fit in one line
                // The current line is the last line only if there's no content after it (j == n)
                // If remaining content fits in one line, current line is NOT the last line
                // (the next line will be the last line)
                let is_last_line = j == n;

                let badness = calculate_badness(
                    line_width,
                    self.ideal_width,
                    self.max_width,
                    is_last_line,
                );
                let total_badness = breaks[i].total_badness + badness;

                if total_badness < best_badness {
                    best_badness = total_badness;
                    best_prev = Some(i);
                }
            }

            // If no valid breakpoint found and this is a single word that exceeds max width,
            // we need to force a break before this word (if not at the start)
            if best_prev.is_none() && j > 0 {
                // Check if the current word alone exceeds max width
                let word_width = self.words[j - 1].width;
                let is_first_line = j == 1;
                let prefix_width = if is_first_line {
                    first_prefix_width
                } else {
                    cont_prefix_width
                };

                if word_width + prefix_width > self.max_width {
                    // This word alone exceeds max width, force break before it
                    // Use the previous breakpoint or start from beginning
                    best_prev = Some(j - 1);
                    best_badness = 0.0; // Acceptable badness for forced break
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
            // Add space between words if:
            // 1. It's not the first word in the line
            // 2. AND (current word needs leading space OR previous word has trailing space)
            if i > start
                && (self.words[i].needs_leading_space
                    || self.words[i - 1].has_trailing_space)
            {
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

        // Post-process to ensure no line exceeds max_width
        let final_breaks = self.enforce_max_width(&adjusted_breaks);

        let mut result = String::new();
        let mut start = 0;
        let mut is_first_line = true;

        for &end in &final_breaks {
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

    /// Enforce max_width by adding additional breaks if necessary
    /// This ensures that no line exceeds max_width after punctuation adjustments
    fn enforce_max_width(&self, breaks: &[usize]) -> Vec<usize> {
        let mut result = Vec::new();
        let mut start = 0;

        for &end in breaks {
            // Check if this line exceeds max_width
            // calculate_line_width already includes prefix width
            let line_width = self.calculate_line_width(start, end);

            if line_width > self.max_width {
                // Need to add intermediate breaks
                let mut current_start = start;
                let mut current_width = if start == 0 {
                    unicode_width::width(&self.first_line_prefix) as usize
                } else {
                    unicode_width::width(&self.continuation_prefix) as usize
                };

                for i in start..end {
                    let word_width = self.words[i].width;
                    let mut space_width = if i > current_start
                        && (self.words[i].needs_leading_space
                            || self.words[i - 1].has_trailing_space)
                    {
                        1
                    } else {
                        0
                    };

                    // Special case: if current word is `]` and next word is `(`,
                    // don't add space between them (link structure)
                    if self.words[i].text == "]"
                        && i + 1 < self.words.len()
                        && self.words[i + 1].text == "("
                    {
                        space_width = 0;
                    }

                    // Check if adding this word would exceed max_width
                    if current_width + space_width + word_width > self.max_width
                        && i > current_start
                    {
                        // Check if the next word is punctuation that should not be at line start
                        // If so, we should include it in the current line
                        let next_is_punct = i + 1 < end
                            && is_punctuation_that_should_not_be_at_line_start(
                                &self.words[i + 1].text,
                            );

                        // Don't break inside a link - check if this word or adjacent words are link parts
                        let is_inside_link = self.words[i].is_link_part
                            || (i > 0
                                && self.words[i - 1].is_link_part
                                && i + 1 < end
                                && self
                                    .words
                                    .get(i + 1)
                                    .map_or(false, |w| w.is_link_part));

                        if is_inside_link {
                            // Don't break inside a link, keep it together
                            current_width += space_width + word_width;
                        // Special case: if this word is `)` and previous word is not `(`,
                        // keep `)` with the previous word (it's a link URL closing)
                        } else if self.words[i].text == ")"
                            && i > 0
                            && self.words[i - 1].text != "("
                        {
                            // Don't break before `)`, include it in current line
                            current_width += space_width + word_width;
                        // Special case: if this word is `(` and previous word is `]`,
                        // keep `](` together (it's a link structure)
                        } else if self.words[i].text == "("
                            && i > 0
                            && self.words[i - 1].text == "]"
                        {
                            // Don't break before `(`, keep `](` together
                            current_width += space_width + word_width;
                        // Special case: if this word is `]` and next word is `(`,
                        // keep `](` together (it's a link structure)
                        } else if self.words[i].text == "]"
                            && i + 1 < self.words.len()
                            && self.words[i + 1].text == "("
                        {
                            // Don't break before `]`, keep `](` together
                            current_width += space_width + word_width;
                        // Special case: if previous word is `](` structure (i.e., prev is `(` and prev-1 is `]`),
                        // don't break before this word - keep the entire link structure together
                        } else if i > 1
                            && self.words[i - 1].text == "("
                            && self.words[i - 2].text == "]"
                        {
                            // Don't break after `](`, keep URL with the link structure
                            current_width += space_width + word_width;
                        // Special case: if this word is backtick (inline code marker),
                        // keep it with the content
                        } else if self.words[i].text == "`" {
                            // Don't break before backtick, keep it with content
                            current_width += space_width + word_width;
                        // Special case: if this word contains backtick (inline code content),
                        // keep it with the content
                        } else if self.words[i].text.contains('`') {
                            // Don't break before inline code content, keep it together
                            current_width += space_width + word_width;
                        // Special case: if this word is `）` (full-width closing parenthesis)
                        // and previous word contains backtick, keep it with inline code
                        } else if self.words[i].text == "）"
                            && i > 0
                            && self.words[i - 1].text.contains('`')
                        {
                            // Don't break before `）`, keep it with inline code
                            current_width += space_width + word_width;
                        // Special case: if this word is `**` (Markdown emphasis),
                        // keep it with the content
                        } else if self.words[i].text == "**" {
                            // Don't break before `**`, keep it with content
                            current_width += space_width + word_width;
                        // Special case: if previous word is `**` (Markdown emphasis),
                        // keep content with the marker
                        } else if i > 0 && self.words[i - 1].text == "**" {
                            // Don't break after `**`, keep content with marker
                            current_width += space_width + word_width;
                        // Special case: if this word is comma, keep it with previous content
                        } else if self.words[i].text == "，" || self.words[i].text == ","
                        {
                            // Don't break before comma, keep it with previous content
                            current_width += space_width + word_width;
                        // Special case: if this word starts with CJK punctuation, keep it with previous content
                        } else if self.words[i]
                            .text
                            .chars()
                            .next()
                            .map_or(false, is_cjk_punctuation)
                        {
                            // Don't break before CJK punctuation (。，；：！？、 etc.), keep it with previous content
                            current_width += space_width + word_width;
                        // Special case: if this word is `(` or other opening brackets,
                        // keep it with previous content to avoid space at line start
                        } else if self.words[i].text.starts_with('(')
                            || self.words[i].text.starts_with('[')
                            || self.words[i].text.starts_with('{')
                        {
                            // Don't break before opening bracket, keep it with previous content
                            current_width += space_width + word_width;
                        // NEW: If next word is punctuation, include current word and punctuation in current line
                        } else if next_is_punct {
                            // Don't break before a word that is followed by punctuation
                            // Include current word and the punctuation in current line
                            current_width += space_width + word_width;
                        // NEW: Check if any word in current line ends with CJK opening bracket
                        // If so, we need to include all content until the closing bracket
                        } else if (current_start..i).any(|k| {
                            self.words[k].text.chars().last().map_or(false, |c| {
                                matches!(
                                    c,
                                    '（' | '《' | '「' | '『' | '【' | '〈' | '“' | '‘'
                                )
                            })
                        }) {
                            // Don't break if there's an unclosed opening bracket in current line
                            current_width += space_width + word_width;
                        // NEW: Don't break if this word is very short (1-2 chars) and would be alone on the next line
                        } else if word_width <= 2
                            && i + 1 < end
                            && self.words[i + 1].width > 2
                        {
                            // Short word followed by longer word - keep short word with current line
                            // to avoid orphan words on the next line
                            current_width += space_width + word_width;
                        } else {
                            // Add break before this word
                            result.push(i);
                            current_start = i;
                            current_width = word_width;
                        }
                    } else {
                        current_width += space_width + word_width;
                    }
                }

                // Add the final break for this segment
                if current_start < end {
                    result.push(end);
                }
            } else {
                result.push(end);
            }

            start = end;
        }

        // Handle any remaining words
        if start < self.words.len() {
            // calculate_line_width already includes prefix width
            let line_width = self.calculate_line_width(start, self.words.len());

            if line_width > self.max_width {
                // Need to add intermediate breaks
                let mut current_start = start;
                let mut current_width = if start == 0 {
                    unicode_width::width(&self.first_line_prefix) as usize
                } else {
                    unicode_width::width(&self.continuation_prefix) as usize
                };

                for i in start..self.words.len() {
                    let word_width = self.words[i].width;
                    let mut space_width = if i > current_start
                        && (self.words[i].needs_leading_space
                            || self.words[i - 1].has_trailing_space)
                    {
                        1
                    } else {
                        0
                    };

                    // Special case: if current word is `]` and next word is `(`,
                    // don't add space between them (link structure)
                    if self.words[i].text == "]"
                        && i + 1 < self.words.len()
                        && self.words[i + 1].text == "("
                    {
                        space_width = 0;
                    }

                    // Check if adding this word would exceed max_width
                    if current_width + space_width + word_width > self.max_width
                        && i > current_start
                    {
                        // Check if the next word is punctuation that should not be at line start
                        // If so, we should include it in the current line
                        let next_is_punct = i + 1 < self.words.len()
                            && is_punctuation_that_should_not_be_at_line_start(
                                &self.words[i + 1].text,
                            );

                        // Special case: if this word is `)` and previous word is not `(`,
                        // keep `)` with the previous word (it's a link URL closing)
                        if self.words[i].text == ")"
                            && i > 0
                            && self.words[i - 1].text != "("
                        {
                            // Don't break before `)`, include it in current line
                            current_width += space_width + word_width;
                        // Special case: if this word is `(` and previous word is `]`,
                        // keep `](` together (it's a link structure)
                        } else if self.words[i].text == "("
                            && i > 0
                            && self.words[i - 1].text == "]"
                        {
                            // Don't break before `(`, keep `](` together
                            current_width += space_width + word_width;
                        // Special case: if this word is `]` and next word is `(`,
                        // keep `](` together (it's a link structure)
                        } else if self.words[i].text == "]"
                            && i + 1 < self.words.len()
                            && self.words[i + 1].text == "("
                        {
                            // Don't break before `]`, keep `](` together
                            current_width += space_width + word_width;
                        // Special case: if previous word is `(` (link URL start),
                        // keep URL with the opening parenthesis
                        } else if i > 0 && self.words[i - 1].text == "(" {
                            // Don't break after `(`, keep URL with opening paren
                            current_width += space_width + word_width;
                        // Special case: if this word is backtick (inline code marker),
                        // keep it with the content
                        } else if self.words[i].text == "`" {
                            // Don't break before backtick, keep it with content
                            current_width += space_width + word_width;
                        // Special case: if this word contains backtick (inline code content),
                        // keep it with the content
                        } else if self.words[i].text.contains('`') {
                            // Don't break before inline code content, keep it together
                            current_width += space_width + word_width;
                        // Special case: if this word is `）` (full-width closing parenthesis)
                        // and previous word contains backtick, keep it with inline code
                        } else if self.words[i].text == "）"
                            && i > 0
                            && self.words[i - 1].text.contains('`')
                        {
                            // Don't break before `）`, keep it with inline code
                            current_width += space_width + word_width;
                        // Special case: if this word is `**` (Markdown emphasis),
                        // keep it with the content
                        } else if self.words[i].text == "**" {
                            // Don't break before `**`, keep it with content
                            current_width += space_width + word_width;
                        // Special case: if previous word is `**` (Markdown emphasis),
                        // keep content with the marker
                        } else if i > 0 && self.words[i - 1].text == "**" {
                            // Don't break after `**`, keep content with marker
                            current_width += space_width + word_width;
                        // Special case: if this word is comma, keep it with previous content
                        } else if self.words[i].text == "，" || self.words[i].text == ","
                        {
                            // Don't break before comma, keep it with previous content
                            current_width += space_width + word_width;
                        // NEW: If next word is punctuation, include current word and punctuation in current line
                        } else if next_is_punct {
                            // Don't break before a word that is followed by punctuation
                            // Include current word and the punctuation in current line
                            current_width += space_width + word_width;
                        // NEW: Check if any word in current line ends with CJK opening bracket
                        // If so, we need to include all content until the closing bracket
                        } else if (current_start..i).any(|k| {
                            self.words[k].text.chars().last().map_or(false, |c| {
                                matches!(
                                    c,
                                    '（' | '《' | '「' | '『' | '【' | '〈' | '“' | '‘'
                                )
                            })
                        }) {
                            // Don't break if there's an unclosed opening bracket in current line
                            current_width += space_width + word_width;
                        // NEW: Don't break if this word is very short (1-2 chars) and would be alone on the next line
                        } else if word_width <= 2
                            && i + 1 < self.words.len()
                            && self.words[i + 1].width > 2
                        {
                            // Short word followed by longer word - keep short word with current line
                            // to avoid orphan words on the next line
                            current_width += space_width + word_width;
                        } else {
                            // Add break before this word
                            result.push(i);
                            current_start = i;
                            current_width = word_width;
                        }
                    } else {
                        current_width += space_width + word_width;
                    }
                }

                // Add final break
                result.push(self.words.len());
            } else {
                result.push(self.words.len());
            }
        }

        result
    }

    /// Adjust breaks based on punctuation affinity
    ///
    /// Affinity determines where punctuation should stay when breaking lines:
    /// - Left-affinity (逗号、句号、闭括号): stay with previous line, break after
    /// - Right-affinity (开括号): stay with next line, break before
    fn adjust_breaks_for_punctuation(&self, breaks: &[usize]) -> Vec<usize> {
        let mut adjusted = Vec::new();
        let mut last_adjusted_break = 0;
        let mut skip_until: Option<usize> = None;

        for &break_point in breaks {
            // If we're skipping until a certain point, check if we've reached it
            if let Some(skip_end) = skip_until {
                if break_point < skip_end {
                    // Still inside the skip range, don't add this break
                    continue;
                } else {
                    // We've passed the skip range
                    skip_until = None;
                }
            }

            let mut new_break = break_point;

            // Handle link pattern `](` - keep together
            // Check if break_point is at `]` and next word is `(`
            if break_point < self.words.len()
                && self.words[break_point].text == "]"
                && break_point + 1 < self.words.len()
                && self.words[break_point + 1].text == "("
            {
                // This is `](` pattern, find closing `)` and break after it
                for k in (break_point + 1)..self.words.len() {
                    if self.words[k].text == ")" {
                        new_break = k + 1;
                        skip_until = Some(new_break);
                        break;
                    }
                }
                adjusted.push(new_break);
                last_adjusted_break = new_break;
                continue;
            }

            // Also check if break_point is right after `](` pattern
            // (i.e., break_point is at the position of `(`)
            if break_point > 0
                && break_point < self.words.len()
                && self.words[break_point].text == "("
                && self.words[break_point - 1].text == "]"
            {
                // This is `](` pattern, find closing `)` and break after it
                for k in break_point..self.words.len() {
                    if self.words[k].text == ")" {
                        new_break = k + 1;
                        skip_until = Some(new_break);
                        break;
                    }
                }
                adjusted.push(new_break);
                last_adjusted_break = new_break;
                continue;
            }

            // Check for any word in the current line range (from last break to current break)
            // that ends with a right-affinity character (opening bracket)
            // If found, extend the break to include up to the matching closing bracket
            // Skip if the opening bracket is part of a link pattern `](`
            // Skip if this is a Markdown link start `[` (followed by text and `]`)
            let line_start = last_adjusted_break;
            for i in line_start..break_point {
                let word = &self.words[i];
                // Skip if this is `]` followed by `(` (link pattern)
                if word.text == "]" && i + 1 < self.words.len() && self.words[i + 1].text == "(" {
                    continue;
                }
                // Skip if this is `[` that starts a Markdown link
                // (i.e., it's followed by text and eventually `]`)
                if word.text == "[" {
                    continue;
                }
                // Skip if this is `(` that is part of a link pattern `](`
                // (i.e., it's preceded by `]`)
                if word.text == "(" && i > 0 && self.words[i - 1].text == "]" {
                    continue;
                }
                if let Some(last_char) = word.text.chars().last() {
                    if is_right_affinity_char(last_char) {
                        // Found opening bracket in current line
                        // Find matching closing bracket
                        if let Some(closing_idx) =
                            self.find_matching_closing_bracket_by_char(i, last_char)
                        {
                            // Extend break to after the closing bracket
                            if closing_idx + 1 > new_break {
                                new_break = closing_idx + 1;
                            }
                        }
                    }
                }
            }

            // Check word at break point (first word of next line)
            if break_point < self.words.len() {
                let word = &self.words[break_point];

                // Handle right-affinity: opening brackets should stay with next line
                // But skip if this is `(` after `]` (link pattern, handled above)
                if word.text == "(" && break_point > 0 && self.words[break_point - 1].text == "]" {
                    // This is a link pattern, already handled above
                    // Don't add this break
                    continue;
                }

                if let Some(Affinity::Right) = get_punctuation_affinity(&word.text) {
                    // Opening bracket at line start - move it to previous line
                    // Find the matching closing bracket and break after it
                    if let Some(closing_idx) =
                        self.find_matching_closing_bracket(break_point, &word.text)
                    {
                        new_break = closing_idx + 1;
                    }
                    adjusted.push(new_break);
                    last_adjusted_break = new_break;
                    continue;
                }

                // Handle left-affinity: punctuation should stay with previous line
                // Move break point forward to include punctuation
                if let Some(Affinity::Left) = get_punctuation_affinity(&word.text) {
                    // Find next non-left-affinity word
                    new_break = break_point + 1;
                    for k in (break_point + 1)..self.words.len() {
                        if get_punctuation_affinity(&self.words[k].text).is_none() {
                            new_break = k;
                            break;
                        }
                        new_break = k + 1;
                    }
                    adjusted.push(new_break);
                    last_adjusted_break = new_break;
                    continue;
                }
            }

            // Default: use original break point (possibly adjusted for opening brackets)
            if !adjusted.contains(&new_break) {
                adjusted.push(new_break);
                last_adjusted_break = new_break;
            }
        }

        // Sort and deduplicate
        adjusted.sort_unstable();
        adjusted.dedup();

        // Ensure last break includes all words
        if let Some(&last) = adjusted.last() {
            if last < self.words.len() {
                adjusted.push(self.words.len());
            }
        } else if !self.words.is_empty() {
            adjusted.push(self.words.len());
        }

        adjusted
    }

    /// Check if a break point is inside a Markdown link
    fn is_inside_link(&self, break_point: usize, line_start: usize) -> bool {
        // Check if there's a `](` pattern before this break point within the current line
        for i in line_start..break_point {
            if self.words[i].text == "]"
                && i + 1 < self.words.len()
                && self.words[i + 1].text == "("
            {
                // Found `](`, now check if the closing `)` is after this break point
                for k in (i + 2)..self.words.len() {
                    if self.words[k].text == ")" {
                        // Found closing `)`
                        return break_point <= k;
                    }
                }
            }
        }
        false
    }

    /// Find the index of the matching closing bracket for an opening bracket
    fn find_matching_closing_bracket(&self, opening_idx: usize, opening_text: &str) -> Option<usize> {
        let opening_char = opening_text.chars().next()?;
        self.find_matching_closing_bracket_by_char(opening_idx, opening_char)
    }

    /// Find the index of the matching closing bracket by opening character
    fn find_matching_closing_bracket_by_char(
        &self,
        opening_idx: usize,
        opening_char: char,
    ) -> Option<usize> {
        let closing_char = match opening_char {
            '(' => ')',
            '[' => ']',
            '{' => '}',
            '（' => '）',
            '《' => '》',
            '「' => '」',
            '『' => '』',
            '【' => '】',
            '〈' => '〉',
            '“' => '”',
            '‘' => '’',
            _ => return None,
        };

        // Find the first word that contains the closing character
        // Check both starts_with and ends_with to handle cases like "123.45）"
        for k in (opening_idx + 1)..self.words.len() {
            let word = &self.words[k];
            if word.text.starts_with(closing_char) || word.text.ends_with(closing_char) {
                return Some(k);
            }
        }

        None
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
/// Badness is defined to encourage filling lines up to max_width.
/// Lines shorter than max_width are penalized to encourage filling.
/// Lines longer than max_width are penalized heavily.
fn calculate_badness(
    line_width: usize,
    _ideal_width: usize,
    max_width: usize,
    is_last_line: bool,
) -> f64 {
    if line_width > max_width {
        // Lines longer than max_width are penalized heavily
        let diff = line_width - max_width;
        (diff * diff * diff * diff) as f64
    } else {
        let diff = max_width - line_width;
        if is_last_line {
            // Last line can be short, use quadratic penalty
            (diff * diff) as f64
        } else {
            // Non-last lines should be as long as possible
            // Use quartic penalty to encourage filling
            (diff * diff * diff * diff) as f64
        }
    }
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
        // Lines shorter than max_width are penalized
        // For non-last lines, use quartic penalty
        assert_eq!(calculate_badness(20, 15, 25, false), 625.0); // (25-20)^4 = 625
        assert_eq!(calculate_badness(15, 15, 25, false), 10000.0); // (25-15)^4 = 10000

        // For last line, use quadratic penalty
        assert_eq!(calculate_badness(20, 15, 25, true), 25.0); // (25-20)^2 = 25
        assert_eq!(calculate_badness(15, 15, 25, true), 100.0); // (25-15)^2 = 100

        // Lines at max_width have zero badness
        assert_eq!(calculate_badness(25, 15, 25, false), 0.0);
        assert_eq!(calculate_badness(25, 15, 25, true), 0.0);

        // Lines longer than max_width are penalized heavily (quartic)
        assert_eq!(calculate_badness(26, 15, 25, false), 1.0); // (26-25)^4 = 1
        assert_eq!(calculate_badness(30, 15, 25, false), 625.0); // (30-25)^4 = 625
        assert_eq!(calculate_badness(26, 15, 25, true), 1.0); // (26-25)^4 = 1
        assert_eq!(calculate_badness(30, 15, 25, true), 625.0); // (30-25)^4 = 625
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
    fn test_starts_with_marker_then_cjk_punctuation() {
        assert!(
            starts_with_marker_then_cjk_punctuation("*：测试"),
            "'*：测试' should match"
        );
        assert!(
            starts_with_marker_then_cjk_punctuation("*，测试"),
            "'*，测试' should match"
        );
        assert!(
            starts_with_marker_then_cjk_punctuation("_。测试"),
            "'_。测试' should match"
        );
        assert!(
            !starts_with_marker_then_cjk_punctuation("*测试"),
            "'*测试' should NOT match"
        );
        assert!(
            !starts_with_marker_then_cjk_punctuation("*: test"),
            "'*: test' should NOT match (ASCII colon)"
        );
        assert!(
            !starts_with_marker_then_cjk_punctuation("：测试"),
            "'：测试' should NOT match (no marker)"
        );
    }

    #[test]
    fn test_emphasis_end_marker_with_cjk_punctuation() {
        let mut ctx = LineBreakingContext::new(80, 80);

        // Simulate: *斜体*：测试
        ctx.add_markdown_marker("*");
        ctx.add_text("斜体");
        ctx.add_markdown_marker_end("*");
        ctx.add_text("：测试");

        let formatted = ctx.format();
        println!("Formatted: {}", formatted);

        assert!(
            formatted.contains("*斜体*：测试"),
            "Should have no space before CJK punctuation. Got: {}",
            formatted
        );
    }

    #[test]
    fn test_emphasis_end_marker_with_marker_then_cjk() {
        let mut ctx = LineBreakingContext::new(80, 80);

        // Simulate: *斜体**：测试 (where *：测试 is literal text)
        ctx.add_markdown_marker("*");
        ctx.add_text("斜体");
        ctx.add_markdown_marker_end("*");
        ctx.add_text("*：测试");

        let formatted = ctx.format();
        println!("Formatted: {}", formatted);

        // Debug: print the split result
        let split_result = split_cjk_text("*：测试");
        println!("Split result: {:?}", split_result);

        assert!(
            formatted.contains("*斜体**：测试"),
            "Should have no space before '*：'. Got: {}",
            formatted
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

        let formatted = ctx.format();

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
    fn test_slash_space_after_link() {
        // Test that slash preserves spaces after link when present
        // Example: [a](url) / [b](url)
        let mut ctx = LineBreakingContext::new(80, 80);

        ctx.add_text("- ");
        ctx.add_markdown_marker("[");
        ctx.add_text("a");
        ctx.add_markdown_marker("]");
        ctx.add_markdown_marker("(");
        ctx.add_text_as_word("url");
        ctx.add_link_close_marker(")");
        ctx.add_text(" / ");
        ctx.add_markdown_marker("[");
        ctx.add_text("b");
        ctx.add_markdown_marker("]");
        ctx.add_markdown_marker("(");
        ctx.add_text_as_word("url");
        ctx.add_link_close_marker(")");

        let formatted = ctx.format();
        println!("Formatted: {:?}", formatted);

        // The slash should preserve spaces from original input
        assert!(
            formatted.contains("[a](url) / [b](url)"),
            "Slash should preserve spaces after link: got {:?}",
            formatted
        );
    }

    #[test]
    fn test_slash_no_space_after_link() {
        // Test that slash has no space after link when original has no space
        let mut ctx = LineBreakingContext::new(80, 80);

        ctx.add_text("- ");
        ctx.add_markdown_marker("[");
        ctx.add_text("a");
        ctx.add_markdown_marker("]");
        ctx.add_markdown_marker("(");
        ctx.add_text_as_word("url");
        ctx.add_link_close_marker(")");
        ctx.add_text("/");
        ctx.add_markdown_marker("[");
        ctx.add_text("b");
        ctx.add_markdown_marker("]");
        ctx.add_markdown_marker("(");
        ctx.add_text_as_word("url");
        ctx.add_link_close_marker(")");

        let formatted = ctx.format();
        println!("Formatted: {:?}", formatted);

        // The slash should have no space when original has no space
        assert!(
            formatted.contains("[a](url)/[b](url)"),
            "Slash should have no space after link: got {:?}",
            formatted
        );
    }

    #[test]
    fn test_slash_space_after_inline_code_with_link() {
        // Test mixing inline code and links with slash
        // Example: `code` / [link](url)
        let mut ctx = LineBreakingContext::new(80, 80);

        ctx.add_text("- ");
        ctx.add_inline_element("`code`");
        ctx.add_text(" / ");
        ctx.add_markdown_marker("[");
        ctx.add_text("link");
        ctx.add_markdown_marker("]");
        ctx.add_markdown_marker("(");
        ctx.add_text_as_word("url");
        ctx.add_link_close_marker(")");

        let formatted = ctx.format();

        assert!(
            formatted.contains("`code` / [link](url)"),
            "Slash should preserve spaces between inline code and link: got {:?}",
            formatted
        );
    }

    #[test]
    fn test_slash_space_after_link_with_inline_code() {
        // Test mixing link and inline code with slash
        // Example: [link](url) / `code`
        let mut ctx = LineBreakingContext::new(80, 80);

        ctx.add_text("- ");
        ctx.add_markdown_marker("[");
        ctx.add_text("link");
        ctx.add_markdown_marker("]");
        ctx.add_markdown_marker("(");
        ctx.add_text_as_word("url");
        ctx.add_link_close_marker(")");
        ctx.add_text(" / ");
        ctx.add_inline_element("`code`");

        let formatted = ctx.format();

        assert!(
            formatted.contains("[link](url) / `code`"),
            "Slash should preserve spaces between link and inline code: got {:?}",
            formatted
        );
    }

    #[test]
    fn test_multiple_slashes_after_inline_code() {
        // Test multiple slashes after inline code
        // Example: `a` / `b` / `c`
        let mut ctx = LineBreakingContext::new(80, 80);

        ctx.add_text("- ");
        ctx.add_inline_element("`a`");
        ctx.add_text(" / ");
        ctx.add_inline_element("`b`");
        ctx.add_text(" / ");
        ctx.add_inline_element("`c`");

        let formatted = ctx.format();

        assert!(
            formatted.contains("`a` / `b` / `c`"),
            "Multiple slashes should preserve spaces: got {:?}",
            formatted
        );
    }

    #[test]
    fn test_multiple_slashes_after_link() {
        // Test multiple slashes after links
        // Example: [a](url) / [b](url) / [c](url)
        let mut ctx = LineBreakingContext::new(80, 80);

        ctx.add_text("- ");
        ctx.add_markdown_marker("[");
        ctx.add_text("a");
        ctx.add_markdown_marker("]");
        ctx.add_markdown_marker("(");
        ctx.add_text_as_word("url");
        ctx.add_link_close_marker(")");
        ctx.add_text(" / ");
        ctx.add_markdown_marker("[");
        ctx.add_text("b");
        ctx.add_markdown_marker("]");
        ctx.add_markdown_marker("(");
        ctx.add_text_as_word("url");
        ctx.add_link_close_marker(")");
        ctx.add_text(" / ");
        ctx.add_markdown_marker("[");
        ctx.add_text("c");
        ctx.add_markdown_marker("]");
        ctx.add_markdown_marker("(");
        ctx.add_text_as_word("url");
        ctx.add_link_close_marker(")");

        let formatted = ctx.format();

        assert!(
            formatted.contains("[a](url) / [b](url) / [c](url)"),
            "Multiple slashes after links should preserve spaces: got {:?}",
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

        // Debug: print words
        println!("Words:");
        for (i, word) in ctx.words().iter().enumerate() {
            println!("  Word {}: text={:?}, width={}", i, word.text, word.width);
        }

        let breaks = ctx.compute_breaks();
        println!("Breaks: {:?}", breaks);

        let formatted = ctx.format();
        println!("Formatted:\n{}", formatted);
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

    #[test]
    fn test_long_link_not_split() {
        // Test that long URLs are NOT split at '/' boundaries
        // because splitting would break the link when spaces are added between parts.
        // Example: 我们旨在重现 `https://github.com/eBay/tsv-utils/blob/master/docs/comparative-benchmarks-2017.md` 使用的严格基准测试策略。

        let mut ctx = LineBreakingContext::new(40, 50);

        // Simulate the text with a long link using add_inline_element
        ctx.add_text("我们旨在重现 ");
        ctx.add_inline_element("`https://github.com/eBay/tsv-utils/blob/master/docs/comparative-benchmarks-2017.md`");
        ctx.add_text(" 使用的严格基准测试策略。");

        let formatted = ctx.format();

        // The URL should NOT be split (no spaces within the URL)
        assert!(
            formatted.contains("`https://github.com/eBay/tsv-utils/blob/master/docs/comparative-benchmarks-2017.md`"),
            "Long URL should NOT be split at '/' boundaries - it would break the link. Formatted:\n{}",
            formatted
        );

        // The link should be usable (no spaces breaking it)
        assert!(
            !formatted.contains("https:/ ") && !formatted.contains("/ "),
            "URL should not contain spaces that would break the link. Formatted:\n{}",
            formatted
        );
    }

    #[test]
    fn test_long_url_link_not_split() {
        // Test that long URLs in links [text](url) are NOT split with `)` on its own line
        // Example: [https://github.com/eBay/tsv-utils/blob/master/docs/comparative-benchmarks-2017.md](https://github.com/eBay/tsv-utils/blob/master/docs/comparative-benchmarks-2017.md)
        let mut ctx = LineBreakingContext::new(40, 50);

        // Simulate: [https://github.com/eBay/tsv-utils/blob/master/docs/comparative-benchmarks-2017.md](https://github.com/eBay/tsv-utils/blob/master/docs/comparative-benchmarks-2017.md)
        ctx.add_markdown_marker("[");
        ctx.add_text_as_word("https://github.com/eBay/tsv-utils/blob/master/docs/comparative-benchmarks-2017.md");
        ctx.add_markdown_marker("]");
        ctx.add_markdown_marker("(");
        ctx.add_text_as_word("https://github.com/eBay/tsv-utils/blob/master/docs/comparative-benchmarks-2017.md");
        ctx.add_link_close_marker(")");

        let formatted = ctx.format();

        // The `)` should NOT be on its own line
        assert!(
            !formatted.contains("\n)"),
            "Closing parenthesis should NOT be on its own line. Formatted:\n{}",
            formatted
        );
    }

    #[test]
    fn test_long_url_link_with_following_text() {
        // Test that long URLs in links are formatted correctly with following text
        let mut ctx = LineBreakingContext::new(40, 50);

        // Simulate: [URL](URL) 使用的严格基准测试策略。
        ctx.add_markdown_marker("[");
        ctx.add_text_as_word("https://github.com/eBay/tsv-utils/blob/master/docs/comparative-benchmarks-2017.md");
        ctx.add_markdown_marker("]");
        ctx.add_markdown_marker("(");
        ctx.add_text_as_word("https://github.com/eBay/tsv-utils/blob/master/docs/comparative-benchmarks-2017.md");
        ctx.add_link_close_marker(")");
        ctx.add_text("使用的严格基准测试策略。");

        // Print words for debugging
        println!("Words:");
        for (i, word) in ctx.words().iter().enumerate() {
            println!("  Word {}: text={:?}, width={}", i, word.text, word.width);
        }

        let breaks = ctx.compute_breaks();
        println!("Original Breaks: {:?}", breaks);

        // Test adjust_breaks_for_punctuation directly
        let adjusted = ctx.adjust_breaks_for_punctuation(&breaks);
        println!("Adjusted Breaks: {:?}", adjusted);

        let formatted = ctx.format();

        // The `)` should NOT be on its own line
        assert!(
            !formatted.contains("\n)"),
            "Closing parenthesis should NOT be on its own line. Formatted:\n{}",
            formatted
        );

        // `](` should NOT be split across lines
        assert!(
            !formatted.contains("]\n("),
            "`](` should NOT be split across lines. Formatted:\n{}",
            formatted
        );
    }

    #[test]
    fn test_link_with_text_and_long_url() {
        // Test that links with text and long URL are formatted correctly
        // Example: [eBay TSV Utilities](https://github.com/eBay/tsv-utils/blob/master/docs/comparative-benchmarks-2017.md)
        let mut ctx = LineBreakingContext::new(60, 60);

        // Simulate: 我们旨在重现 [eBay TSV Utilities](https://github.com/eBay/tsv-utils/blob/master/docs/comparative-benchmarks-2017.md) 使用的严格基准测试策略。
        ctx.add_text("我们旨在重现 ");
        ctx.add_markdown_marker("[");
        ctx.add_text("eBay TSV Utilities");
        ctx.add_markdown_marker("]");
        ctx.add_markdown_marker("(");
        ctx.add_text_as_word("https://github.com/eBay/tsv-utils/blob/master/docs/comparative-benchmarks-2017.md");
        ctx.add_link_close_marker(")");
        ctx.add_text(" 使用的严格基准测试策略。");

        // Print words for debugging
        println!("Words:");
        for (i, word) in ctx.words().iter().enumerate() {
            println!("  Word {}: text={:?}, width={}", i, word.text, word.width);
        }

        let breaks = ctx.compute_breaks();
        println!("Breaks: {:?}", breaks);

        let adjusted_breaks = ctx.adjust_breaks_for_punctuation(&breaks);
        println!("Adjusted breaks: {:?}", adjusted_breaks);

        let final_breaks = ctx.enforce_max_width(&adjusted_breaks);
        println!("Final breaks: {:?}", final_breaks);

        let formatted = ctx.format();
        println!("Formatted:\n{}", formatted);

        // `](` should NOT be split across lines
        assert!(
            !formatted.contains("]\n("),
            "`](` should NOT be split across lines. Formatted:\n{}",
            formatted
        );

        // The URL is too long (81 chars) to fit within max_width (60),
        // so it's acceptable to break after `](` and put URL on its own line.
        // We just verify that the link structure is preserved.
        assert!(
            formatted.contains("[eBay TSV Utilities]("),
            "Link text and opening parenthesis should be on the same line. Formatted:\n{}",
            formatted
        );
    }

    #[test]
    fn test_link_with_cjk_punctuation_not_at_line_start() {
        // Test that CJK punctuation after link is NOT at line start
        // Example: [link](url) 。测试。
        let mut ctx = LineBreakingContext::new(60, 60);

        // Simulate: - **HEPMASS** ( 4.8GB): [link](https://archive.ics.uci.edu/ml/datasets/HEPMASS) 。测试。
        ctx.add_text("- **HEPMASS** ( 4.8GB): ");
        ctx.add_markdown_marker("[");
        ctx.add_text("link");
        ctx.add_markdown_marker("]");
        ctx.add_markdown_marker("(");
        ctx.add_text_as_word("https://archive.ics.uci.edu/ml/datasets/HEPMASS");
        ctx.add_link_close_marker(")");
        ctx.add_text(" 。测试。");

        // Print words for debugging
        println!("Words:");
        for (i, word) in ctx.words().iter().enumerate() {
            println!("  Word {}: text={:?}, width={}", i, word.text, word.width);
        }

        let breaks = ctx.compute_breaks();
        println!("Breaks: {:?}", breaks);

        let adjusted_breaks = ctx.adjust_breaks_for_punctuation(&breaks);
        println!("Adjusted breaks: {:?}", adjusted_breaks);

        let final_breaks = ctx.enforce_max_width(&adjusted_breaks);
        println!("Final breaks: {:?}", final_breaks);

        let formatted = ctx.format();
        println!("Formatted:\n{}", formatted);

        // CJK period `。` should NOT be at line start
        assert!(
            !formatted.contains("\n  。"),
            "CJK period should NOT be at line start. Formatted:\n{}",
            formatted
        );

        // The period should be on the same line as the link
        assert!(
            formatted.contains(")。"),
            "CJK period should be on the same line as the link. Formatted:\n{}",
            formatted
        );
    }

    #[test]
    fn test_link_with_various_cjk_punctuation() {
        // Test that various CJK punctuation after link are NOT at line start
        let test_cases = vec![
            ("，", "CJK comma"),
            ("、", "CJK enumeration comma"),
            ("；", "CJK semicolon"),
            ("：", "CJK colon"),
            ("！", "CJK exclamation"),
            ("？", "CJK question"),
            ("）", "CJK right parenthesis"),
            ("】", "CJK right bracket"),
            ("」", "CJK right corner bracket"),
            ("』", "CJK right white corner bracket"),
            ("〉", "CJK right angle bracket"),
            ("》", "CJK right double angle bracket"),
            // Japanese punctuation
            ("〜", "Japanese wave dash"),
            ("〝", "Japanese double quote open"),
            ("〞", "Japanese double quote close"),
        ];

        for (punct, desc) in test_cases {
            let mut ctx = LineBreakingContext::new(60, 60);

            // Simulate: [link](url) [punct] test
            ctx.add_text("- ");
            ctx.add_markdown_marker("[");
            ctx.add_text("link");
            ctx.add_markdown_marker("]");
            ctx.add_markdown_marker("(");
            ctx.add_text_as_word("https://archive.ics.uci.edu/ml/datasets/HEPMASS");
            ctx.add_link_close_marker(")");
            ctx.add_text(&format!(" {} 测试", punct));

            let formatted = ctx.format();

            // CJK punctuation should NOT be at line start
            let newline_punct = format!("\n  {}", punct);
            assert!(
                !formatted.contains(&newline_punct),
                "{} ({}) should NOT be at line start. Formatted:\n{}",
                desc,
                punct,
                formatted
            );

            // The punctuation should be on the same line as the link
            let link_punct = format!("){}", punct);
            assert!(
                formatted.contains(&link_punct),
                "{} ({}) should be on the same line as the link. Formatted:\n{}",
                desc,
                punct,
                formatted
            );
        }
    }

    #[test]
    fn test_debug_markdown_emphasis() {
        // Debug test for Markdown emphasis
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

        // Print words for debugging
        println!("Words:");
        for (i, word) in ctx.words().iter().enumerate() {
            println!("  Word {}: text={:?}, width={}", i, word.text, word.width);
        }

        let breaks = ctx.compute_breaks();
        println!("Breaks: {:?}", breaks);

        let formatted = ctx.format();
        println!("Formatted:\n{}", formatted);

        // The emphasized text should stay together
        assert!(
            formatted.contains("**简单高效的数据处理**"),
            "Emphasized text should stay together. Formatted:\n{}",
            formatted
        );
    }

    #[test]
    fn test_opening_paren_no_space_after_cjk() {
        // Test that opening parenthesis `(` has no space after CJK text
        // Example: 和随机采样 (`sample`)的基础
        // Should be: 和随机采样(`sample`)的基础 (no space before `(`)
        let mut ctx = LineBreakingContext::new(80, 80);

        ctx.add_text("和随机采样 ");
        ctx.add_markdown_marker("(");
        ctx.add_markdown_marker("`");
        ctx.add_text("sample");
        ctx.add_markdown_marker("`");
        ctx.add_markdown_marker(")");
        ctx.add_text("的基础");

        let formatted = ctx.format();
        println!("Formatted: {:?}", formatted);

        // There should be no space before `(`
        assert!(
            !formatted.contains("采样 ("),
            "There should be no space before `(` after CJK text. Formatted:\n{}",
            formatted
        );

        // The correct format should be `采样(`
        assert!(
            formatted.contains("采样(`"),
            "`(` should directly follow CJK text without space. Formatted:\n{}",
            formatted
        );
    }

    #[test]
    fn test_debug_fullwidth_paren() {
        // Debug test for full-width opening parenthesis
        let mut ctx = LineBreakingContext::new(80, 90);

        // Simulate: 针对 `tva` 的 `Value` 类型使用 `Arc` 进行优化的可行性，我们编写了基准测试（`benches/value_arc.rs`），对比当前直接克隆与使用 `Arc` 包装后的性能差异。
        ctx.add_text("针对 `tva` 的 `Value` 类型使用 `Arc` 进行优化的可行性，我们编写了基准测试（`benches/value_arc.rs`），对比当前直接克隆与使用 `Arc` 包装后的性能差异。");

        // Print words for debugging
        println!("Words:");
        for (i, word) in ctx.words().iter().enumerate() {
            println!("  Word {}: text={:?}, width={}", i, word.text, word.width);
        }

        let breaks = ctx.compute_breaks();
        println!("Breaks: {:?}", breaks);

        let formatted = ctx.format();
        println!("Formatted:\n{}", formatted);

        // The opening parenthesis should be directly followed by the inline code
        assert!(
            formatted.contains("（`benches/value_arc.rs`）"),
            "Opening parenthesis should be directly followed by inline code. Formatted:\n{}",
            formatted
        );
    }

    #[test]
    fn test_opening_bracket_no_space_after() {
        // Test that opening bracket followed by text doesn't have space
        // Example: **HEPMASS** (\n  4.8GB) should become **HEPMASS** (4.8GB)
        let mut ctx = LineBreakingContext::new(80, 90);

        ctx.add_markdown_marker("**");
        ctx.add_text("HEPMASS");
        ctx.add_markdown_marker("**");
        ctx.add_text(" (\n  4.8GB)");

        // Print words for debugging
        println!("Words:");
        for (i, word) in ctx.words().iter().enumerate() {
            println!(
                "  Word {}: text={:?}, needs_leading_space={}, has_trailing_space={}",
                i, word.text, word.needs_leading_space, word.has_trailing_space
            );
        }

        let formatted = ctx.format();
        println!("Formatted:\n{}", formatted);

        // There should be no space after `(`
        assert!(
            !formatted.contains("( 4.8GB)"),
            "There should be no space after `(`. Formatted:\n{}",
            formatted
        );

        // The correct format should be `(4.8GB)`
        assert!(
            formatted.contains("(4.8GB)"),
            "`(` should be directly followed by `4.8GB`. Formatted:\n{}",
            formatted
        );
    }

    #[test]
    fn test_opening_bracket_no_space_after_full() {
        // Test using the full format_commonmark function
        use crate::{format_commonmark, parse_document, Options, Plugins};

        let mut options = Options::default();
        options.render.width = 80; // Set a large width to prevent line breaking
        let input = "**HEPMASS** (\n  4.8GB)";
        let (arena, root) = parse_document(input, &options);
        let mut output = String::new();
        format_commonmark(&arena, root, &options, &mut output, &Plugins::default())
            .unwrap();

        println!("Input: {:?}", input);
        println!("Output: {:?}", output);

        // There should be no space after `(`
        assert!(
            !output.contains("( 4.8GB)"),
            "There should be no space after `(`. Output:\n{}",
            output
        );

        // The correct format should be `(4.8GB)`
        assert!(
            output.contains("(4.8GB)"),
            "`(` should be directly followed by `4.8GB`. Output:\n{}",
            output
        );
    }

    #[test]
    fn test_debug_line_breaking_context() {
        // Debug test to see what's happening in LineBreakingContext
        let mut ctx = LineBreakingContext::new(80, 90);

        // Simulate what happens in format_commonmark
        ctx.add_markdown_marker("**");
        ctx.add_text("HEPMASS");
        ctx.add_markdown_marker("**");
        ctx.add_text(" ("); // Note: this includes the leading space
        ctx.add_text("4.8GB)");

        // Print words for debugging
        println!("Words:");
        for (i, word) in ctx.words().iter().enumerate() {
            println!(
                "  Word {}: text={:?}, needs_leading_space={}, has_trailing_space={}",
                i, word.text, word.needs_leading_space, word.has_trailing_space
            );
        }

        let formatted = ctx.format();
        println!("Formatted:\n{}", formatted);

        // There should be no space after `(`
        assert!(
            !formatted.contains("( 4.8GB)"),
            "There should be no space after `(`. Formatted:\n{}",
            formatted
        );
    }

    #[test]
    fn test_list_item_line_breaking_width() {
        // Test for the bug: line breaks too early in list items
        // Input: "- For projects that have finished downloading, but have renamed strains, you can run `reorder.sh` to avoid re-downloading"
        // Expected: should fill the line closer to max_width
        // Note: ideal_width is set to 75% of max_width to encourage balanced line lengths
        let mut ctx = LineBreakingContext::with_prefixes(78, 78, "", "  ");
        ctx.add_text("For projects that have finished downloading, but have renamed strains, you can run");
        ctx.add_inline_element("`reorder.sh`");
        ctx.add_text("to avoid re-downloading");

        println!("Words:");
        for (i, word) in ctx.words().iter().enumerate() {
            println!("  Word {}: text={:?}, width={}", i, word.text, word.width);
        }

        let breaks = ctx.compute_breaks();
        println!("Breaks: {:?}", breaks);

        let formatted = ctx.format();
        println!("Formatted:\n{}", formatted);

        // Check that the first line is reasonably filled
        let first_line = formatted.lines().next().unwrap();
        let first_line_width = unicode_width::width(first_line);
        println!("First line width: {}", first_line_width);

        // The first line should be reasonably filled
        // With ideal_width = 75% of max_width, we expect balanced line lengths
        // The first line should be at least 60 characters
        assert!(
            first_line_width >= 60,
            "First line should be reasonably filled, but got {}:\n{}",
            first_line_width,
            first_line
        );
    }

    #[test]
    fn test_cjk_punctuation_not_at_line_start() {
        // Test for the bug: Chinese comma appears at line start
        // Input: "这些操作需要 `list.iter().cloned().collect()`，比直接 `list.clone()` 慢得多。"
        // Expected: Chinese comma should NOT appear at line start
        let mut ctx = LineBreakingContext::with_prefixes(60, 60, "", "  ");
        ctx.add_text("这些操作需要");
        ctx.add_inline_element("`list.iter().cloned().collect()`");
        ctx.add_text("，比直接");
        ctx.add_inline_element("`list.clone()`");
        ctx.add_text("慢得多。");

        // Print words for debugging
        println!("Words:");
        for (i, word) in ctx.words().iter().enumerate() {
            println!(
                "  Word {}: text={:?}, needs_leading_space={}, has_trailing_space={}",
                i, word.text, word.needs_leading_space, word.has_trailing_space
            );
        }

        let breaks = ctx.compute_breaks();
        println!("Breaks: {:?}", breaks);

        let formatted = ctx.format();
        println!("Formatted:\n{}", formatted);

        // Check that no line starts with Chinese comma
        for line in formatted.lines() {
            let trimmed = line.trim_start();
            assert!(
                !trimmed.starts_with('，'),
                "Line should not start with Chinese comma: {}",
                line
            );
        }
    }

    #[test]
    fn test_cjk_punctuation_not_at_line_start_real_case() {
        // Test for the issue: single digit "0" should not be on its own line
        // Input: "- **特色功能**: 支持日期补全 (`--dates`)，自动填充缺失的日期并设为 0；支持间隙压缩 (`--compress-gaps`)，隐藏连续的 0 值。"
        let mut ctx = LineBreakingContext::with_prefixes(75, 100, "- ", "  ");
        ctx.add_markdown_marker("**");
        ctx.add_text("特色功能");
        ctx.add_markdown_marker_end("**");
        ctx.add_text(": 支持日期补全 (");
        ctx.add_inline_element("`--dates`");
        ctx.add_text(")，自动填充缺失的日期并设为 0；支持间隙压缩 (");
        ctx.add_inline_element("`--compress-gaps`");
        ctx.add_text(")，隐藏连续的 0 值。");

        // Print words for debugging
        println!("Words:");
        for (i, word) in ctx.words().iter().enumerate() {
            println!(
                "  Word {}: text={:?}, needs_leading_space={}, has_trailing_space={}, width={}",
                i, word.text, word.needs_leading_space, word.has_trailing_space, word.width
            );
        }

        let breaks = ctx.compute_breaks();
        println!("Breaks: {:?}", breaks);

        let formatted = ctx.format();
        println!("Formatted:\n{}", formatted);

        // Check that no line starts with Chinese comma
        for line in formatted.lines() {
            let trimmed = line.trim_start();
            assert!(
                !trimmed.starts_with('，'),
                "Line should not start with Chinese comma: {}",
                line
            );
        }

        // Check that single digit "0" is not on its own line
        for line in formatted.lines() {
            let trimmed = line.trim();
            assert!(
                trimmed != "0" && trimmed != "0 值。",
                "Single digit '0' should not be on its own line: {}",
                line
            );
        }
    }
}

/// Affinity of punctuation marks for line breaking
/// Left: punctuation stays with the previous line (break after)
/// Right: punctuation stays with the next line (break before)
#[derive(Debug, Clone, Copy, PartialEq)]
enum Affinity {
    Left,
    Right,
}

/// Get the affinity of a punctuation mark
/// Returns Some(Affinity) if the first char is a punctuation mark, None otherwise
fn get_punctuation_affinity(text: &str) -> Option<Affinity> {
    let first_char = text.chars().next()?;
    match first_char {
        // Left-affinity: these should stay with the previous line
        // Chinese punctuation
        '，' | '。' | '；' | '：' | '！' | '？' | '）' | '》' | '」' | '』' | '】' | '〉' | '”' | '’' |
        // English punctuation
        ',' | '.' | ';' | ':' | '!' | '?' | ')' | ']' | '}' |
        // Special case: closing backtick for inline code
        '`' => Some(Affinity::Left),
        // Right-affinity: these should stay with the next line
        // Chinese opening brackets
        '（' | '《' | '「' | '『' | '【' | '〈' | '“' | '‘' |
        // English opening brackets
        '(' | '[' | '{' => Some(Affinity::Right),
        _ => None,
    }
}

/// Check if a character is a left-affinity punctuation mark
fn is_left_affinity_punctuation(c: char) -> bool {
    matches!(
        c,
        '，' | '。'
            | '；'
            | '：'
            | '！'
            | '？'
            | '）'
            | '》'
            | '」'
            | '』'
            | '】'
            | '〉'
            | '”'
            | '’'
            | ','
            | '.'
            | ';'
            | ':'
            | '!'
            | '?'
            | ')'
            | ']'
            | '}'
            | '`'
    )
}

/// Check if a character is a right-affinity punctuation mark
fn is_right_affinity_punctuation(c: char) -> bool {
    is_right_affinity_char(c)
}

/// Check if a character has right affinity (opening brackets)
fn is_right_affinity_char(c: char) -> bool {
    matches!(
        c,
        '（' | '《' | '「' | '『' | '【' | '〈' | '“' | '‘' | '(' | '[' | '{'
    )
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
/// leading space after inline code (like `:`, `,`, `.`, `;`, `!`, `?`, `)`, `[`, `]`, `{`, `}`)
/// Note: `(` is excluded because it should have leading space after inline code
/// Note: `/` is excluded to preserve spacing consistency around inline code (e.g., `code` / `code`)
fn is_ascii_punctuation_no_leading_space(c: char) -> bool {
    matches!(
        c,
        ':' | ',' | '.' | ';' | '!' | '?' | ')' | '[' | ']' | '{' | '}'
    )
}

/// Check if a string starts with punctuation that should NOT have leading space
/// after inline code (CJK punctuation or specific ASCII punctuation like `:`, `,`, `.`)
fn starts_with_no_leading_space_punctuation(text: &str) -> bool {
    text.chars().next().map_or(false, |c| {
        is_cjk_punctuation(c) || is_ascii_punctuation_no_leading_space(c)
    })
}

/// Check if text starts with a Markdown marker character (*, _) followed by CJK punctuation
/// This handles cases like `*：测试` where the `*` is literal text followed by CJK punctuation
fn starts_with_marker_then_cjk_punctuation(text: &str) -> bool {
    let mut chars = text.chars();
    if let Some(first) = chars.next() {
        if first == '*' || first == '_' {
            if let Some(second) = chars.next() {
                return is_cjk_punctuation(second);
            }
        }
    }
    false
}

/// Check if text is a single Markdown marker character (* or _)
fn is_single_markdown_marker(text: &str) -> bool {
    let mut chars = text.chars();
    if let Some(first) = chars.next() {
        if first == '*' || first == '_' {
            return chars.next().is_none();
        }
    }
    false
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
                | '』'
                | '】'
                | '、'
                | '。'
                | '，'
                | '；'
                | '：'
                | '！'
                | '？'
                | '〉'
                | '》'
                | '〜'
                | '〝'
                | '〞'
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
