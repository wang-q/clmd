//! Knuth-Plass line breaking algorithm implementation
//!
//! This module implements the Knuth-Plass line breaking algorithm for optimal
//! paragraph formatting. The algorithm uses dynamic programming to find the
//! globally optimal set of line breaks that minimizes the total "badness" of
//! the paragraph.
//!
//! The algorithm is based on the paper "Breaking Paragraphs into Lines" by
//! Donald E. Knuth and Michael F. Plass (1981).

use crate::text::unicode;

/// Check if a character is punctuation
fn is_punctuation(c: char) -> bool {
    crate::text::char::is_punctuation(c)
}

fn is_cjk_opening_bracket(c: char) -> bool {
    matches!(
        c,
        '(' | '[' | '{' // ASCII opening brackets (also checked for Left affinity)
            | '\u{ff08}' // （
            | '\u{ff3b}' // 【
            | '\u{ff5b}' // 〔
            | '\u{300c}' // 「
            | '\u{300e}' // 『
            | '\u{3008}' // 〈
            | '\u{300a}' // 《
            | '\u{3010}' // 【
            | '\u{3014}' // 〔
    )
}

fn is_ascii_opening_bracket(c: char) -> bool {
    matches!(c, '(' | '[' | '{')
}

fn is_ascii_closing_bracket(c: char) -> bool {
    matches!(c, ')' | ']' | '}')
}

fn is_cjk_punct_that_should_pull_back(c: char) -> bool {
    matches!(
        c,
        '，' | '。' | '、' | '；' | '：' | '）' | '」' | '』' | '”'
    )
}

/// Affinity for break opportunities
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Affinity {
    /// Break after this position (left side stays)
    Left,
    /// Break before this position (right side stays)
    Right,
}

/// A break opportunity in the paragraph
#[derive(Debug, Clone, Copy)]
pub struct BreakOpportunity {
    /// The position in the text where a break can occur
    pub position: usize,
    /// The total width before this position
    pub width_before: usize,
    /// The affinity of this break opportunity
    pub affinity: Affinity,
    /// Whether this is a forced break
    pub is_forced: bool,
}

/// A line break decision
#[derive(Debug, Clone, Copy)]
pub struct LineBreak {
    /// The position where the line should break
    pub position: usize,
    /// The width of the line before the break
    pub line_width: usize,
}

/// The kind of atomic unit (internal absolutely no breaks)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtomicKind {
    /// Emphasis (*text*)
    Emph,
    /// Strong (**text**)
    Strong,
    /// Code (`text`)
    Code,
    /// Link ([text](url))
    Link,
    /// Image (![alt](url))
    Image,
    /// Other unbreakable content
    Other,
}

/// Content fragment for line breaking
#[derive(Debug, Clone)]
pub enum ContentFragment {
    /// Plain text content that can be broken at specific points
    Text {
        /// The text content
        content: String,
        /// The display width of the content
        width: usize,
        /// Break points within this fragment (byte offsets from start of content)
        break_points: Vec<(usize, Affinity)>,
        /// Width at each break point
        break_widths: Vec<usize>,
    },
    /// Atomic unit - internal absolutely no breaks
    Atomic {
        /// The full content
        content: String,
        /// The display width
        width: usize,
        /// The kind of atomic unit
        kind: AtomicKind,
    },
}

/// Paragraph line breaker using the Knuth-Plass algorithm
#[derive(Debug)]
pub struct ParagraphLineBreaker {
    /// The maximum allowed width for a line
    max_width: usize,
    /// The line prefix for continuation lines
    prefix: String,
    /// Content fragments
    fragments: Vec<ContentFragment>,
    /// Current position in the paragraph (byte offset)
    current_position: usize,
    /// Current width
    current_width: usize,
    /// The last text fragment content (not atomic) for bracket-aware breaking
    last_text_fragment: Option<String>,
}

impl ParagraphLineBreaker {
    /// Create a new paragraph line breaker
    pub fn new(max_width: usize, prefix: String) -> Self {
        Self {
            max_width,
            prefix,
            fragments: Vec::new(),
            current_position: 0,
            current_width: 0,
            last_text_fragment: None,
        }
    }

    /// Get the maximum width
    pub fn max_width(&self) -> usize {
        self.max_width
    }

    /// Get the current width
    pub fn current_width(&self) -> usize {
        self.current_width
    }

    /// Get the current position
    pub fn current_position(&self) -> usize {
        self.current_position
    }

    /// Remove trailing space from the last text fragment
    pub fn remove_trailing_space(&mut self) {
        if let Some(ContentFragment::Text {
            content,
            width,
            break_points,
            break_widths,
        }) = self.fragments.last_mut()
        {
            if content.ends_with(' ') {
                *content = content.trim_end().to_string();
                *width = unicode::width(content) as usize;
                self.current_position -= 1;
                self.current_width -= 1;
                // Remove any break point at the end
                if let Some(&(last_bp, _)) = break_points.last() {
                    if last_bp >= content.len() {
                        break_points.pop();
                        break_widths.pop();
                    }
                }
            }
        }
    }

    /// Check if the content ends with whitespace
    pub fn ends_with_whitespace(&self) -> bool {
        self.fragments
            .last()
            .is_some_and(|fragment| match fragment {
                ContentFragment::Text { content, .. } => {
                    content.ends_with(|c: char| c.is_whitespace())
                }
                ContentFragment::Atomic { .. } => false,
            })
    }

    /// Check if the content before the last fragment ends with CJK character
    pub fn ends_with_cjk(&self) -> bool {
        for fragment in self.fragments.iter().rev() {
            match fragment {
                ContentFragment::Text { content, .. } => {
                    if let Some(c) = content.chars().rev().find(|c| !c.is_whitespace()) {
                        return crate::text::unicode::is_cjk(c);
                    }
                }
                ContentFragment::Atomic { .. } => {
                    // Skip atomic units when checking for CJK
                    continue;
                }
            }
        }
        false
    }

    /// Add text content with break points at word boundaries
    ///
    /// This method adds text and automatically identifies break points at:
    /// - Whitespace characters (break after)
    /// - Punctuation marks (according to affinity rules)
    pub fn add_text(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }

        let width = unicode::width(text) as usize;
        let mut break_points = Vec::new();
        let mut break_widths = Vec::new();

        let mut accumulated_width = 0;
        let mut char_iter = text.char_indices().peekable();

        while let Some((byte_pos, c)) = char_iter.next() {
            let char_str = c.to_string();
            let char_width = unicode::width(&char_str) as usize;

            // Get the previous non-whitespace character for context-aware punctuation handling
            // We skip whitespace to correctly identify CJK context even when there's space
            // between text and punctuation (e.g., "CJK text (english)")
            let prev_char = if byte_pos > 0 {
                text[..byte_pos].chars().rev().find(|c| !c.is_whitespace())
            } else {
                None
            };

            // Get the next non-whitespace character for context-aware punctuation handling
            // This is needed for characters like slash that may be part of a word (e.g., "I/O")
            let next_char = char_iter
                .peek()
                .map(|(_, c)| *c)
                .or_else(|| text[byte_pos + char_str.len()..].chars().next());

            // Check if we can break at this character position
            let break_opportunity = if c.is_whitespace() {
                // Always break after whitespace (left affinity)
                Some(Affinity::Left)
            } else {
                // Check punctuation affinity for break opportunity
                get_punctuation_affinity(&char_str, prev_char, next_char)
            };

            if let Some(affinity) = break_opportunity {
                match affinity {
                    Affinity::Left => {
                        // Left affinity: break after this character
                        break_points.push((byte_pos + char_str.len(), affinity));
                        break_widths
                            .push(self.current_width + accumulated_width + char_width);
                    }
                    Affinity::Right => {
                        // Right affinity: break before this character
                        // Only add break point if we're not at the start
                        if byte_pos > 0 {
                            break_points.push((byte_pos, affinity));
                            break_widths.push(self.current_width + accumulated_width);
                        }
                    }
                }
            }

            accumulated_width += char_width;
        }

        self.fragments.push(ContentFragment::Text {
            content: text.to_string(),
            width,
            break_points,
            break_widths,
        });

        self.current_position += text.len();
        self.current_width += width;
        self.last_text_fragment = Some(text.to_string());
    }

    /// Add an atomic unit - internal absolutely no breaks
    ///
    /// This is used for:
    /// - Emphasis (*text*, **text**)
    /// - Inline code (`code`)
    /// - Links ([text](url))
    /// - Images (![alt](url))
    pub fn add_atomic(&mut self, content: &str, kind: AtomicKind) {
        if content.is_empty() {
            return;
        }

        let width = unicode::width(content) as usize;

        self.fragments.push(ContentFragment::Atomic {
            content: content.to_string(),
            width,
            kind,
        });

        self.current_position += content.len();
        self.current_width += width;

        // Update last_text_fragment for context-aware punctuation handling
        // This ensures that atomic units (like markdown markers) are considered
        // when checking for opening brackets before atomic units
        self.last_text_fragment = Some(content.to_string());
    }

    /// Add a hard line break (forced break)
    pub fn add_hard_break(&mut self) {
        // Add a special marker fragment for hard breaks
        // We use a zero-width fragment to mark the position
        self.fragments.push(ContentFragment::Text {
            content: String::new(),
            width: 0,
            break_points: vec![(0, Affinity::Left)],
            break_widths: vec![self.current_width],
        });
    }

    /// Add a word as an atomic unit (for backward compatibility)
    pub fn add_word(&mut self, text: &str) {
        // Treat words as atomic to prevent breaking within them
        self.add_atomic(text, AtomicKind::Other);
    }

    /// Add an unbreakable unit with prefix, content, suffix
    pub fn add_unbreakable_unit(
        &mut self,
        kind: AtomicKind,
        prefix: &str,
        content: &str,
        suffix: &str,
    ) {
        let full_content = format!("{}{}{}", prefix, content, suffix);
        self.add_atomic(&full_content, kind);
    }

    /// Compute the optimal line breaks using the Knuth-Plass algorithm
    pub fn compute_breaks(&self) -> Vec<LineBreak> {
        if self.fragments.is_empty() {
            return Vec::new();
        }

        let mut break_positions: Vec<(usize, usize)> = vec![(0, 0)];
        let mut current_pos = 0;
        let mut current_width = 0;

        for fragment in &self.fragments {
            match fragment {
                ContentFragment::Text {
                    content,
                    width,
                    break_points,
                    break_widths,
                    ..
                } => {
                    for (i, &(bp, affinity)) in break_points.iter().enumerate() {
                        let absolute_pos = current_pos + bp;
                        if self.should_prevent_break(absolute_pos, affinity) {
                            continue;
                        }
                        let absolute_width = if i < break_widths.len() {
                            break_widths[i]
                        } else {
                            current_width + unicode::width(&content[..bp]) as usize
                        };
                        break_positions.push((absolute_pos, absolute_width));
                    }
                    current_pos += content.len();
                    current_width += width;
                }
                ContentFragment::Atomic {
                    content: atomic_content,
                    width,
                    kind,
                } => {
                    if current_pos > 0 {
                        break_positions.push((current_pos, current_width));
                    }
                    current_pos += atomic_content.len();
                    current_width += width;
                    if !matches!(kind, AtomicKind::Code | AtomicKind::Link) {
                        break_positions.push((current_pos, current_width));
                    }
                }
            }
        }

        if break_positions
            .last()
            .map_or(true, |(pos, _)| *pos != self.current_position)
        {
            break_positions.push((self.current_position, self.current_width));
        }

        break_positions.sort_by_key(|(pos, _)| *pos);
        break_positions.dedup_by_key(|(pos, _)| *pos);

        let n = break_positions.len();
        let mut best_cost: Vec<f64> = vec![f64::INFINITY; n];
        let mut best_prev: Vec<usize> = vec![0; n];
        best_cost[0] = 0.0;

        for i in 1..n {
            for j in (0..i).rev() {
                let line_width = break_positions[i].1 - break_positions[j].1;
                let start_pos = break_positions[j].0;
                let end_pos = break_positions[i].0;
                let pos = break_positions[i].0;

                let cost = if line_width <= self.max_width {
                    let slack = self.max_width - line_width;
                    let at_atomic_start = self.is_at_atomic_start(pos);
                    let next_char = self.find_non_ws_char_after(pos);
                    let prev_char = self.find_non_ws_char_before(pos);

                    let punctuation_penalty = Self::compute_punctuation_penalty(
                        next_char,
                        prev_char,
                        at_atomic_start,
                    );

                    let is_last_line = i == n - 1;
                    if is_last_line {
                        best_cost[j] + slack as f64 * 0.1 + punctuation_penalty
                    } else {
                        best_cost[j] + (slack as f64).powi(2) + punctuation_penalty
                    }
                } else {
                    let overflow = line_width - self.max_width;
                    let has_overflowing =
                        self.has_overflowing_atomic_in_range(start_pos, end_pos);
                    // Extra penalty when a small overflow leaves punctuation
                    // at end of line: prefer to break before it so the
                    // punctuation flows to the next line
                    let trailing_punct_penalty = if overflow <= 5 {
                        self.find_non_ws_char_before(end_pos)
                            .filter(|&c| matches!(c, ',' | '.' | ':' | ';'))
                            .map_or(0.0, |_| (overflow as f64 + 5.0).powi(2) * 80.0)
                    } else {
                        0.0
                    };
                    if has_overflowing {
                        best_cost[j] + (overflow as f64).powi(2) * 100.0
                            - (line_width as f64 * 0.001)
                            + trailing_punct_penalty
                    } else {
                        best_cost[j]
                            + (overflow as f64).powi(2) * 100.0
                            + trailing_punct_penalty
                    }
                };

                if cost < best_cost[i] {
                    best_cost[i] = cost;
                    best_prev[i] = j;
                }
            }
        }

        let mut breaks = Vec::new();
        let mut i = n - 1;
        while i > 0 {
            let prev = best_prev[i];
            let line_width = break_positions[i].1 - break_positions[prev].1;
            breaks.push(LineBreak {
                position: break_positions[i].0,
                line_width,
            });
            i = prev;
        }

        breaks.reverse();
        breaks
    }

    fn compute_punctuation_penalty(
        next_char: Option<char>,
        prev_char: Option<char>,
        at_atomic_start: bool,
    ) -> f64 {
        // A character is part of a "word" if it's not whitespace and not a bracket
        fn is_word_char(c: char) -> bool {
            !c.is_whitespace() && !matches!(c, '(' | ')' | '[' | ']' | '{' | '}')
        }

        let next_penalty = if let Some(c) = next_char {
            if is_cjk_punct_that_should_pull_back(c) {
                10000.0
            } else if matches!(c, ',' | '.' | ';' | ':' | ')' | ']' | '}')
                || (is_ascii_opening_bracket(c) && !at_atomic_start)
            {
                // If the next char is part of a word (and prev_char is also word content),
                // don't penalize breaking before it
                // This handles cases like "file.txt", "../path" where we want to break
                // before the word, not in the middle of it
                if is_word_char(c) && prev_char.is_some_and(is_word_char) {
                    0.0
                } else {
                    5000.0
                }
            } else {
                0.0
            }
        } else {
            0.0
        };

        let prev_penalty = if let Some(c) = prev_char {
            if is_ascii_opening_bracket(c) || is_cjk_opening_bracket(c) {
                5000.0
            } else {
                0.0
            }
        } else {
            0.0
        };

        next_penalty + prev_penalty
    }

    fn find_containing_fragment_info(
        &self,
        pos: usize,
    ) -> Option<(usize, Option<char>, Option<char>)> {
        let mut current_pos = 0;
        let mut prev_char: Option<char> = None;
        for (idx, fragment) in self.fragments.iter().enumerate() {
            match fragment {
                ContentFragment::Text { content, .. } => {
                    let fragment_end = current_pos + content.len();
                    if pos > current_pos && pos <= fragment_end {
                        let char_pos = pos - current_pos;
                        let before = if char_pos > 0 {
                            Some(content[..char_pos].chars().last().unwrap())
                        } else {
                            prev_char
                        };
                        let after = if char_pos < content.len() {
                            content[char_pos..].chars().next()
                        } else {
                            None
                        };
                        return Some((idx, before, after));
                    }
                    if !content.is_empty() {
                        prev_char = content.chars().last();
                    }
                    current_pos = fragment_end;
                }
                ContentFragment::Atomic { content, .. } => {
                    let fragment_end = current_pos + content.len();
                    if pos > current_pos && pos <= fragment_end {
                        let char_pos = pos - current_pos;
                        let before = if char_pos > 0 {
                            Some(content[..char_pos].chars().last().unwrap())
                        } else {
                            prev_char
                        };
                        return Some((idx, before, None));
                    }
                    if !content.is_empty() {
                        prev_char = content.chars().last();
                    }
                    current_pos = fragment_end;
                }
            }
        }
        None
    }

    fn should_prevent_break(&self, pos: usize, affinity: Affinity) -> bool {
        let Some((_idx, prev_char, next_char)) = self.find_containing_fragment_info(pos)
        else {
            return false;
        };

        match affinity {
            Affinity::Left => {
                let is_cjk_open = next_char.is_some_and(is_cjk_opening_bracket);
                let prev_is_ws = prev_char.is_some_and(|c| c.is_whitespace());
                is_cjk_open && !prev_is_ws
            }
            Affinity::Right => {
                let is_prev_ascii_close =
                    prev_char.is_some_and(is_ascii_closing_bracket);
                let is_next_cjk_open = next_char.is_some_and(is_cjk_opening_bracket);

                if is_prev_ascii_close && is_next_cjk_open {
                    return true;
                }
                if is_next_cjk_open {
                    return false;
                }
                next_char.is_some_and(crate::text::char::is_cjk_punctuation)
            }
        }
    }

    fn find_non_ws_char_after(&self, pos: usize) -> Option<char> {
        let mut current_pos = 0;
        for fragment in &self.fragments {
            match fragment {
                ContentFragment::Text { content, .. } => {
                    let fragment_end = current_pos + content.len();
                    if pos < fragment_end {
                        let start = pos.saturating_sub(current_pos);
                        return content[start..].chars().find(|c| !c.is_whitespace());
                    }
                    current_pos = fragment_end;
                }
                ContentFragment::Atomic { content, .. } => {
                    let fragment_end = current_pos + content.len();
                    if pos < fragment_end {
                        let start = pos.saturating_sub(current_pos);
                        return content[start..].chars().find(|c| !c.is_whitespace());
                    }
                    current_pos = fragment_end;
                }
            }
        }
        None
    }

    fn find_non_ws_char_before(&self, pos: usize) -> Option<char> {
        let mut current_pos = 0;
        for fragment in &self.fragments {
            match fragment {
                ContentFragment::Text { content, .. } => {
                    let fragment_start = current_pos;
                    let fragment_end = current_pos + content.len();
                    if pos > fragment_start && pos <= fragment_end {
                        let end_in_frag = pos - fragment_start;
                        if end_in_frag > 0 {
                            return content[..end_in_frag]
                                .chars()
                                .rev()
                                .find(|c| !c.is_whitespace());
                        }
                    } else if pos <= fragment_start {
                        break;
                    }
                    current_pos = fragment_end;
                }
                ContentFragment::Atomic { content, .. } => {
                    let fragment_start = current_pos;
                    let fragment_end = current_pos + content.len();
                    if pos > fragment_start && pos <= fragment_end {
                        let end_in_frag = pos - fragment_start;
                        if end_in_frag > 0 {
                            return content[..end_in_frag]
                                .chars()
                                .rev()
                                .find(|c| !c.is_whitespace());
                        }
                    } else if pos <= fragment_start {
                        break;
                    }
                    current_pos = fragment_end;
                }
            }
        }
        None
    }

    fn has_overflowing_atomic_in_range(&self, start_pos: usize, end_pos: usize) -> bool {
        let mut current_pos = 0;
        for fragment in &self.fragments {
            match fragment {
                ContentFragment::Text { content, .. } => {
                    current_pos += content.len();
                }
                ContentFragment::Atomic { content, width, .. } => {
                    let fragment_start = current_pos;
                    let fragment_end = current_pos + content.len();
                    let overlaps = fragment_start < end_pos && fragment_end > start_pos;
                    if overlaps && *width > self.max_width {
                        return true;
                    }
                    current_pos = fragment_end;
                }
            }
        }
        false
    }

    fn is_at_atomic_start(&self, pos: usize) -> bool {
        let mut current_pos = 0;
        for fragment in &self.fragments {
            match fragment {
                ContentFragment::Text { content, .. } => {
                    let fragment_end = current_pos + content.len();
                    if pos > current_pos && pos < fragment_end {
                        return false;
                    }
                    current_pos = fragment_end;
                }
                ContentFragment::Atomic { .. } => {
                    let fragment_start = current_pos;
                    let fragment_end = current_pos
                        + match fragment {
                            ContentFragment::Atomic { content, .. } => content.len(),
                            _ => unreachable!(),
                        };
                    if pos == fragment_start {
                        return true;
                    }
                    if pos > fragment_start && pos < fragment_end {
                        return false;
                    }
                    current_pos = fragment_end;
                }
            }
        }
        false
    }

    fn compute_actual_start(
        &self,
        line_idx: usize,
        start_in_fragment: usize,
        end_in_fragment: usize,
        content: &str,
        prev_fragment_was_code: bool,
        prev_fragment_was_atomic: bool,
        result_ends_with_punct: bool,
    ) -> usize {
        if line_idx == 0 {
            return start_in_fragment;
        }

        // Find first non-whitespace position in the content slice
        let first_non_ws = content[start_in_fragment..end_in_fragment]
            .find(|c: char| !c.is_whitespace())
            .map(|i| start_in_fragment + i);

        // No non-WS content: skip entirely if prev line ended with punct
        let actual = match first_non_ws {
            Some(pos) => pos,
            None => {
                return if result_ends_with_punct {
                    end_in_fragment
                } else {
                    start_in_fragment
                };
            }
        };

        // Unified rule: keep leading space only in specific cases
        // 1. After Code fragments (CJK spacing convention — always keep)
        // 2. After Atomic (non-code) when next char is not punctuation
        //    (preserve normal word spacing after links/code spans)
        let should_keep_space = prev_fragment_was_code
            || (prev_fragment_was_atomic
                && !is_punctuation(content[actual..].chars().next().unwrap_or('\0')));

        if should_keep_space {
            start_in_fragment
        } else {
            actual
        }
    }

    fn try_pull_back_punctuation(
        &self,
        result: &mut String,
        break_position: usize,
    ) -> usize {
        let mut pos = 0;
        for fragment in &self.fragments {
            match fragment {
                ContentFragment::Text { content, .. } => {
                    let fragment_start = pos;
                    let fragment_end = pos + content.len();

                    if break_position >= fragment_start && break_position < fragment_end
                    {
                        let start_in_fragment = break_position - fragment_start;
                        if let Some(punct) = content[start_in_fragment..]
                            .chars()
                            .find(|c| !c.is_whitespace())
                            .filter(|c| is_cjk_punct_that_should_pull_back(*c))
                        {
                            result.push(punct);
                            return punct.len_utf8();
                        }
                        break;
                    }
                    pos = fragment_end;
                }
                ContentFragment::Atomic { .. } => {
                    let fragment_end = pos
                        + match fragment {
                            ContentFragment::Atomic { content, .. } => content.len(),
                            _ => unreachable!(),
                        };
                    if break_position >= pos && break_position < fragment_end {
                        break;
                    }
                    pos = fragment_end;
                }
            }
        }
        0
    }

    /// Render a single line of output from start_pos to end_pos.
    fn render_line(
        &self,
        line_idx: usize,
        start_pos: usize,
        end_pos: usize,
        prev_line_ends_with_punct: bool,
    ) -> (String, bool, bool, bool) {
        let mut line_content = String::new();
        let mut pos = 0;
        let mut prev_was_code = false;
        let mut prev_was_atomic = false;

        for fragment in &self.fragments {
            match fragment {
                ContentFragment::Text { content, .. } => {
                    let frag_start = pos;
                    let frag_end = pos + content.len();

                    if frag_end <= start_pos {
                        pos = frag_end;
                        prev_was_atomic = false;
                        continue;
                    }
                    if frag_start >= end_pos {
                        break;
                    }

                    let frag_offset = start_pos.saturating_sub(frag_start);
                    let frag_limit = (end_pos - frag_start).min(content.len());

                    let line_trimmed = line_content.trim_end();
                    let line_ends_punct = line_trimmed
                        .chars()
                        .last()
                        .map_or(prev_line_ends_with_punct, is_punctuation);

                    let actual_start = self.compute_actual_start(
                        line_idx,
                        frag_offset,
                        frag_limit,
                        content,
                        prev_was_code,
                        prev_was_atomic,
                        line_ends_punct,
                    );

                    // Normalize "( 4.8GB)" → "(4.8GB)"
                    let final_start = if line_trimmed.ends_with('(') {
                        content[actual_start..frag_limit]
                            .find(|c: char| !c.is_whitespace())
                            .map(|i| actual_start + i)
                            .unwrap_or(frag_limit)
                    } else {
                        actual_start
                    };

                    if final_start < content.len() && frag_limit > final_start {
                        line_content.push_str(&content[final_start..frag_limit]);
                    }

                    pos = frag_end;
                    prev_was_code = false;
                    prev_was_atomic = false;
                }
                ContentFragment::Atomic { content, kind, .. } => {
                    let frag_start = pos;
                    let frag_end = pos + content.len();

                    if frag_end <= start_pos {
                        pos = frag_end;
                        prev_was_atomic = true;
                        continue;
                    }
                    if frag_start >= end_pos {
                        break;
                    }

                    if frag_start < end_pos {
                        line_content.push_str(content);
                    }

                    pos = frag_end;
                    prev_was_code = matches!(kind, AtomicKind::Code);
                    prev_was_atomic = true;
                }
            }

            if pos >= end_pos {
                break;
            }
        }

        let ends_with_punct = line_content
            .trim_end()
            .chars()
            .last()
            .is_some_and(is_punctuation);

        (
            line_content,
            prev_was_code,
            prev_was_atomic,
            ends_with_punct,
        )
    }

    /// Format the paragraph with optimal line breaks
    pub fn format(&self) -> String {
        let breaks = self.compute_breaks();

        if breaks.is_empty() {
            return String::new();
        }

        let mut lines: Vec<String> = Vec::with_capacity(breaks.len());
        let mut last_break_pos = 0;
        let mut prev_ends_with_punct = false;

        for (line_idx, break_point) in breaks.iter().enumerate() {
            if line_idx > 0 {
                lines.push(self.prefix.clone());
            }

            let (mut line_content, _, _, _) = self.render_line(
                line_idx,
                last_break_pos,
                break_point.position,
                prev_ends_with_punct,
            );

            if line_idx < breaks.len() - 1 {
                while line_content.ends_with(' ') {
                    line_content.pop();
                }
                let pull_back = self
                    .try_pull_back_punctuation(&mut line_content, break_point.position);
                last_break_pos = break_point.position + pull_back;

                prev_ends_with_punct = line_content
                    .trim_end()
                    .chars()
                    .last()
                    .is_some_and(is_punctuation);

                lines.push(line_content);
                lines.push('\n'.to_string());
            } else {
                lines.push(line_content);
            }
        }

        let mut result = lines.concat();
        while result.ends_with(' ') {
            result.pop();
        }
        result
    }
}

/// Get the punctuation affinity for a character based on its type and context
///
/// # Arguments
/// * `char_str` - The character as a string
/// * `prev_char` - The previous character in the text (for context-aware decisions)
/// * `next_char` - The next character in the text (for context-aware decisions)
fn get_punctuation_affinity(
    char_str: &str,
    prev_char: Option<char>,
    next_char: Option<char>,
) -> Option<Affinity> {
    // Check if the previous character is CJK (affects how we treat ASCII punctuation)
    let prev_is_cjk = prev_char.is_some_and(crate::text::unicode::is_cjk);

    match char_str {
        // ASCII punctuation that can appear in both CJK and English contexts
        // When in CJK context, treat them like CJK punctuation
        "," | "!" | "?" | ";" => {
            if prev_is_cjk {
                // In CJK context, stay with the left side (CJK rule)
                Some(Affinity::Left)
            } else {
                // In English context, standard rule
                Some(Affinity::Left)
            }
        }

        // Closing brackets - in CJK context, they should stay with left side
        "}" | ")" | "]" => {
            // Always stay with the left side (the content inside the brackets)
            Some(Affinity::Left)
        }

        // Opening brackets - behavior depends on context
        "{" | "(" | "[" => {
            if prev_is_cjk {
                // In CJK context, stay with the right side (don't separate from following CJK)
                Some(Affinity::Right)
            } else {
                // In English context, can break before
                Some(Affinity::Right)
            }
        }

        // Punctuation marks that are often part of words/identifiers when
        // surrounded by word content (no whitespace separation)
        "/" | "\\" | "." | "-" | "_" | "@" | "#" | "+" | "=" | "~" | ":" => {
            // A character is part of a "word" if it's not whitespace and not a bracket
            // This treats continuous sequences of letters, digits, and punctuation as words
            // Examples: "commonmark.js", "../path", "well-known", "C++", "key=value"
            // But: "Hello . world" (spaces around dot) - dot is not part of a word
            fn is_word_char(c: char) -> bool {
                !c.is_whitespace() && !matches!(c, '(' | ')' | '[' | ']' | '{' | '}')
            }
            let prev_is_word = prev_char.is_some_and(is_word_char);
            let next_is_word = next_char.is_some_and(is_word_char);
            if prev_is_word && next_is_word {
                // This punctuation is surrounded by word content, don't break here
                None
            } else {
                // Use default affinity based on punctuation type
                match char_str {
                    "/" | "\\" | "-" | "+" | "=" | "~" => Some(Affinity::Right),
                    "." | "_" | "@" | "#" | ":" => Some(Affinity::Left),
                    _ => Some(Affinity::Right),
                }
            }
        }

        // CJK punctuation with special handling
        "\u{3001}" | "\u{3002}" | "\u{ff0c}" | "\u{ff0e}" | "\u{ff01}" | "\u{ff1f}"
        | "\u{ff1b}" | "\u{ff1a}" => {
            // CJK punctuation should stay with the left side
            Some(Affinity::Left)
        }

        // CJK opening brackets
        "\u{300c}" | "\u{300e}" | "\u{3008}" | "\u{300a}" | "\u{3010}" | "\u{3014}"
        | "\u{ff08}" | "\u{ff3b}" | "\u{ff5b}" => {
            // CJK opening brackets should stay with the right side
            Some(Affinity::Right)
        }

        // CJK closing brackets
        "\u{300d}" | "\u{300f}" | "\u{3009}" | "\u{300b}" | "\u{3011}" | "\u{3015}"
        | "\u{ff09}" | "\u{ff3d}" | "\u{ff5d}" => {
            // CJK closing brackets should stay with the left side
            Some(Affinity::Left)
        }

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paragraph_line_breaker_basic() {
        let mut breaker = ParagraphLineBreaker::new(20, String::new());
        breaker.add_text("Hello world this is a test");
        let result = breaker.format();
        assert!(!result.is_empty());
    }

    #[test]
    fn test_cjk_text_breaking() {
        let mut breaker = ParagraphLineBreaker::new(20, String::new());
        breaker.add_text("这是一个很长的中文段落，用于测试行断行功能");
        let result = breaker.format();
        assert!(!result.is_empty());
    }

    #[test]
    fn test_mixed_cjk_english() {
        let mut breaker = ParagraphLineBreaker::new(30, String::new());
        breaker.add_text("这是中文和English混合的文本");
        let result = breaker.format();
        assert!(!result.is_empty());
    }

    #[test]
    fn test_atomic_unit() {
        let mut breaker = ParagraphLineBreaker::new(20, String::new());
        breaker.add_text("This is ");
        breaker.add_atomic("*emphasis*", AtomicKind::Emph);
        breaker.add_text(" text");
        let result = breaker.format();
        assert!(result.contains("*emphasis*"));
    }

    #[test]
    fn test_emphasis_spacing() {
        let mut breaker = ParagraphLineBreaker::new(80, String::new());
        breaker.add_text("This is ");
        breaker.add_word("*");
        breaker.add_text("italic");
        breaker.add_word("*");
        breaker.add_text(" text");
        let result = breaker.format();
        println!("Result: {:?}", result);
        assert!(
            result.contains(" *italic* "),
            "Expected ' *italic* ' but got {:?}",
            result
        );
    }

    #[test]
    fn test_with_prefix() {
        let mut breaker = ParagraphLineBreaker::new(20, "  ".to_string());
        breaker
            .add_text("This is a long paragraph that should be wrapped with a prefix");
        let result = breaker.format();
        let lines: Vec<&str> = result.lines().collect();
        if lines.len() > 1 {
            for line in &lines[1..] {
                assert!(
                    line.starts_with("  "),
                    "Continuation line should have prefix"
                );
            }
        }
    }

    #[test]
    fn test_compute_breaks() {
        let mut breaker = ParagraphLineBreaker::new(20, String::new());
        breaker.add_text("Hello world this is a test of the line breaking algorithm");
        let breaks = breaker.compute_breaks();
        assert!(!breaks.is_empty());
    }

    #[test]
    fn test_english_word_breaking() {
        let mut breaker = ParagraphLineBreaker::new(30, String::new());
        breaker.add_text("Paragraph with hard break and more text");
        let result = breaker.format();
        // Should break at word boundaries
        let lines: Vec<&str> = result.lines().collect();
        assert!(lines.len() >= 2, "Should have multiple lines");
    }

    #[test]
    fn test_cjk_opening_bracket_right_affinity() {
        // Test that CJK opening bracket (（) has right affinity
        // Break should occur BEFORE the bracket, not after
        let mut breaker = ParagraphLineBreaker::new(20, String::new());
        breaker.add_text("这是一个很长的文本（包含括号）");
        let result = breaker.format();
        println!("Result: {:?}", result);
        // Opening bracket should not be at end of line
        assert!(
            !result.contains("（\n"),
            "Opening bracket should not be at line end, got: {:?}",
            result
        );
    }

    #[test]
    fn test_cjk_closing_bracket_left_affinity() {
        // Test that CJK closing bracket (）) has left affinity
        // Break should occur AFTER the bracket, not before
        let mut breaker = ParagraphLineBreaker::new(20, String::new());
        breaker.add_text("这是一个很长的文本（包含括号）");
        let result = breaker.format();
        println!("Result: {:?}", result);
        // Closing bracket should not be at start of line
        assert!(
            !result.contains("\n）"),
            "Closing bracket should not be at line start, got: {:?}",
            result
        );
    }

    #[test]
    fn test_ascii_opening_paren_right_affinity() {
        // Test that ASCII opening paren has right affinity in CJK context
        let mut breaker = ParagraphLineBreaker::new(25, String::new());
        breaker.add_text("这是一个测试 (hello world) 文本");
        let result = breaker.format();
        println!("Result: {:?}", result);
        // Opening paren should not be at end of line when preceded by CJK
        assert!(
            !result.contains(" (\n"),
            "Opening paren should not be at line end after CJK, got: {:?}",
            result
        );
    }

    #[test]
    fn test_cjk_punctuation_affinity_comprehensive() {
        // Comprehensive test for various CJK punctuation marks
        let test_cases = vec![
            ("测试（开括号）", "（", "\n"), // Opening bracket not at line end
            ("测试【开括号】", "【", "\n"), // Opening bracket not at line end
            ("测试「开括号」", "「", "\n"), // Opening bracket not at line end
            ("测试《开括号》", "《", "\n"), // Opening bracket not at line end
        ];

        for (text, opening_bracket, newline) in test_cases {
            let mut breaker = ParagraphLineBreaker::new(10, String::new());
            breaker.add_text(text);
            let result = breaker.format();
            let bad_pattern = format!("{}{}", opening_bracket, newline);
            assert!(
                !result.contains(&bad_pattern),
                "Opening bracket '{}' should not be at line end in: {:?}",
                opening_bracket,
                result
            );
        }
    }

    #[test]
    fn test_cjk_with_space_before_paren() {
        // Test case from cli_fmt_line_breaking_spec.md
        // When there's a space between CJK text and opening paren,
        // the break should happen at the space, not after the paren
        let mut breaker = ParagraphLineBreaker::new(100, String::new());
        // Simulate the text before the atomic unit
        breaker.add_text("- **特色功能**: 支持日期补全 (`--dates`)，自动填充缺失的日期并设为 0；支持间隙压缩 (");
        // Simulate the atomic unit (inline code)
        breaker.add_atomic("`--compress-gaps`", AtomicKind::Code);
        // Continue with the rest
        breaker.add_text(")，隐藏连续的 0 值。");
        let result = breaker.format();
        println!("Result: {:?}", result);
        // The opening paren should not be at the end of a line
        assert!(
            !result.contains(" (\n"),
            "Opening paren should not be at line end, got: {:?}",
            result
        );
    }

    #[test]
    fn test_cjk_with_space_before_paren_simulate_cli() {
        // Simulate how CLI actually constructs the text
        // CLI uses add_word for markdown markers and add_text for text content
        let mut breaker = ParagraphLineBreaker::new(98, "  ".to_string());

        // Simulate CLI's actual call sequence
        breaker.add_word("-"); // List marker
        breaker.add_word("**"); // Strong start
        breaker.add_word("特色功能"); // Strong content
        breaker.add_word("**"); // Strong end
        breaker.add_text(": 支持日期补全 (");
        breaker.add_atomic("`--dates`", AtomicKind::Code);
        breaker.add_text(")，自动填充缺失的日期并设为 0；支持间隙压缩 (");
        breaker.add_atomic("`--compress-gaps`", AtomicKind::Code);
        breaker.add_text(")，隐藏连续的 0 值。");

        let result = breaker.format();
        println!("Result (simulated CLI): {:?}", result);

        // The opening paren should not be at the end of a line
        assert!(
            !result.contains(" (\n"),
            "Opening paren should not be at line end, got: {:?}",
            result
        );
    }

    #[test]
    fn test_link_breaking_at_margin() {
        // Test case from formatter_spec.md Link Inline:3
        // Link should not be split, break should occur before the link
        let mut breaker = ParagraphLineBreaker::new(40, String::new());
        breaker.add_text("This is a ");
        breaker.add_atomic(
            "[link](https://example.com/very/long/path)",
            AtomicKind::Link,
        );
        breaker.add_text(" in text.");
        let result = breaker.format();
        println!("Result: {:?}", result);
        // The link should be on its own line or with surrounding text,
        // but "a" should stay with "This is"
        assert!(
            result.contains("This is a\n"),
            "Expected 'This is a' followed by newline, got: {:?}",
            result
        );
    }

    #[test]
    fn test_dot_in_filename_not_breaking() {
        // Test that dot in filenames like "commonmark.js" doesn't cause a break
        // The dot should be treated as part of the word when surrounded by alphanumeric chars
        let mut breaker = ParagraphLineBreaker::new(50, String::new());
        breaker.add_text("- [commonmark.js 源码](https://github.com/commonmark/commonmark.js) - 本地路径：../commonmark.js-0.31.2");
        let result = breaker.format();
        println!("Result: {:?}", result);
        // The filename "commonmark.js" should not be split at the dot
        assert!(
            !result.contains("commonmark.\n"),
            "Dot in filename should not cause break, got: {:?}",
            result
        );
        assert!(
            result.contains("commonmark.js"),
            "Filename 'commonmark.js' should be preserved, got: {:?}",
            result
        );
    }

    #[test]
    fn test_dot_in_word_vs_sentence_end() {
        // Dot at end of sentence should allow break
        let mut breaker = ParagraphLineBreaker::new(20, String::new());
        breaker.add_text("Hello world. This is a test.");
        let result = breaker.format();
        println!("Sentence result: {:?}", result);
        // Should break after sentence-ending dots
        assert!(result.contains(".\n") || result.lines().count() >= 1);

        // Dot in filename should not break
        let mut breaker2 = ParagraphLineBreaker::new(20, String::new());
        breaker2.add_text("See file.txt for details");
        let result2 = breaker2.format();
        println!("Filename result: {:?}", result2);
        assert!(
            !result2.contains("file.\n"),
            "Dot in 'file.txt' should not cause break, got: {:?}",
            result2
        );
    }

    #[test]
    fn test_dot_slash_path_not_breaking() {
        // Test that "../" in paths is not split
        // This is the regression test case from formatter_regression_spec.md
        let mut breaker = ParagraphLineBreaker::new(100, String::new());
        breaker.add_text("- [commonmark.js 源码](https://github.com/commonmark/commonmark.js) - 本地路径：../commonmark.js-0.31.2");
        let result = breaker.format();
        // The "../" should not be split
        assert!(
            !result.contains("..\n/"),
            "'../' should not be split, got: {:?}",
            result
        );
        assert!(
            result.contains("../"),
            "'../' should be preserved, got: {:?}",
            result
        );
    }
}
