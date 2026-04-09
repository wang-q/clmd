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

    /// Create a new word that doesn't need leading space (for punctuation)
    pub fn new_punctuation(text: impl Into<String>) -> Self {
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

    /// Mark this word as part of a link
    pub fn as_link_part(mut self) -> Self {
        self.is_link_part = true;
        self
    }
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

/// The kind of unbreakable unit (legacy, for backward compatibility)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitKind {
    /// Emphasis (*text*)
    Emph,
    /// Strong (**text**)
    Strong,
    /// Code (`text`)
    Code,
    /// Inline code (`text`)
    InlineCode,
    /// Link ([text](url))
    Link,
}

/// Handle for an unbreakable unit (legacy, for backward compatibility)
#[derive(Debug, Clone, Copy)]
pub struct UnitHandle {
    index: usize,
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
        break_points: Vec<usize>,
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
        if let Some(last_fragment) = self.fragments.last_mut() {
            if let ContentFragment::Text {
                content,
                width,
                break_points,
                break_widths,
            } = last_fragment
            {
                if content.ends_with(' ') {
                    *content = content.trim_end().to_string();
                    *width = unicode_width::width(content) as usize;
                    self.current_position -= 1;
                    self.current_width -= 1;
                    // Remove any break point at the end
                    if let Some(&last_bp) = break_points.last() {
                        if last_bp >= content.len() {
                            break_points.pop();
                            break_widths.pop();
                        }
                    }
                }
            }
        }
    }

    /// Check if the content ends with whitespace
    pub fn ends_with_whitespace(&self) -> bool {
        self.fragments
            .last()
            .map_or(false, |fragment| match fragment {
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
                        return crate::text::char::is_cjk(c);
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

        let width = unicode_width::width(text) as usize;
        let mut break_points = Vec::new();
        let mut break_widths = Vec::new();

        let mut accumulated_width = 0;
        let mut char_iter = text.char_indices().peekable();

        while let Some((byte_pos, c)) = char_iter.next() {
            let char_str = c.to_string();
            let char_width = unicode_width::width(&char_str) as usize;

            // Check if we can break after this character
            let can_break = if c.is_whitespace() {
                // Always break after whitespace
                true
            } else if let Some(affinity) = get_punctuation_affinity(&char_str) {
                // Break according to punctuation affinity
                matches!(affinity, Affinity::Left)
            } else {
                false
            };

            if can_break {
                break_points.push(byte_pos + char_str.len());
                break_widths.push(self.current_width + accumulated_width + char_width);
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

        let width = unicode_width::width(content) as usize;

        self.fragments.push(ContentFragment::Atomic {
            content: content.to_string(),
            width,
            kind,
        });

        self.current_position += content.len();
        self.current_width += width;
    }

    /// Add a hard line break (forced break)
    pub fn add_hard_break(&mut self) {
        // Add a special marker fragment for hard breaks
        // We use a zero-width fragment to mark the position
        self.fragments.push(ContentFragment::Text {
            content: String::new(),
            width: 0,
            break_points: vec![0],
            break_widths: vec![self.current_width],
        });
    }

    /// Add a word as an atomic unit (for backward compatibility)
    pub fn add_word(&mut self, text: &str) {
        // Treat words as atomic to prevent breaking within them
        self.add_atomic(text, AtomicKind::Other);
    }

    /// Start an unbreakable unit (legacy, for backward compatibility)
    pub fn start_unit(&mut self, _kind: UnitKind, _marker_width: usize) -> UnitHandle {
        // In the new implementation, we don't track units separately
        // Just return a dummy handle
        UnitHandle { index: 0 }
    }

    /// End an unbreakable unit (legacy, for backward compatibility)
    pub fn end_unit(
        &mut self,
        _handle: UnitHandle,
        _content_width: usize,
        _marker_width: usize,
    ) {
        // In the new implementation, this is a no-op
    }

    /// Add an unbreakable unit with prefix, content, suffix
    pub fn add_unbreakable_unit(
        &mut self,
        kind: UnitKind,
        prefix: &str,
        content: &str,
        suffix: &str,
    ) {
        // Combine them into a single atomic unit
        let full_content = format!("{}{}{}", prefix, content, suffix);
        // Map UnitKind to AtomicKind
        let atomic_kind = match kind {
            UnitKind::Emph => AtomicKind::Emph,
            UnitKind::Strong => AtomicKind::Strong,
            UnitKind::Code | UnitKind::InlineCode => AtomicKind::Code,
            UnitKind::Link => AtomicKind::Link,
        };
        self.add_atomic(&full_content, atomic_kind);
    }

    /// Compute the optimal line breaks using the Knuth-Plass algorithm
    pub fn compute_breaks(&self) -> Vec<LineBreak> {
        if self.fragments.is_empty() {
            return Vec::new();
        }

        // Build the list of break positions
        // Each entry is (position, width_before)
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
                    // Add break points within this text fragment
                    for (i, &bp) in break_points.iter().enumerate() {
                        let absolute_pos = current_pos + bp;
                        let absolute_width = if i < break_widths.len() {
                            break_widths[i]
                        } else {
                            current_width + unicode_width::width(&content[..bp]) as usize
                        };
                        break_positions.push((absolute_pos, absolute_width));
                    }
                    current_pos += content.len();
                    current_width += width;
                }
                ContentFragment::Atomic {
                    content,
                    width,
                    kind,
                } => {
                    // For atomic units, we can break before and after, but not inside
                    // Break before
                    break_positions.push((current_pos, current_width));
                    // Break after
                    current_pos += content.len();
                    current_width += width;
                    // For Code, don't add break after to preserve CJK spacing
                    if !matches!(kind, AtomicKind::Code) {
                        break_positions.push((current_pos, current_width));
                    }
                }
            }
        }

        // Ensure the end position is included
        if break_positions
            .last()
            .map_or(true, |(pos, _)| *pos != self.current_position)
        {
            break_positions.push((self.current_position, self.current_width));
        }

        // Sort and deduplicate
        break_positions.sort_by_key(|(pos, _)| *pos);
        break_positions.dedup_by_key(|(pos, _)| *pos);

        // Dynamic programming to find optimal breaks
        let n = break_positions.len();
        let mut best_cost: Vec<f64> = vec![f64::INFINITY; n];
        let mut best_prev: Vec<usize> = vec![0; n];

        best_cost[0] = 0.0;

        for i in 1..n {
            for j in (0..i).rev() {
                let line_width = break_positions[i].1 - break_positions[j].1;

                // Check if this segment fits
                let cost = if line_width <= self.max_width {
                    // Standard Knuth-Plass: penalize slack quadratically
                    let slack = self.max_width - line_width;

                    // For the last line, don't penalize shortness
                    let is_last_line = i == n - 1;
                    if is_last_line {
                        best_cost[j] + slack as f64 * 0.1 // Minimal penalty
                    } else {
                        best_cost[j] + (slack as f64).powi(2)
                    }
                } else {
                    // Line overflows - this is bad, but we may have to accept it
                    // for unbreakable content
                    let overflow = line_width - self.max_width;
                    best_cost[j] + (overflow as f64).powi(2) * 100.0
                };

                if cost < best_cost[i] {
                    best_cost[i] = cost;
                    best_prev[i] = j;
                }
            }
        }

        // Backtrack to find the optimal breaks
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

    /// Format the paragraph with optimal line breaks
    pub fn format(&self) -> String {
        let breaks = self.compute_breaks();

        if breaks.is_empty() {
            return String::new();
        }

        let mut result = String::new();
        let mut last_break_pos = 0;

        for (line_idx, break_point) in breaks.iter().enumerate() {
            // Add the prefix for continuation lines
            if line_idx > 0 {
                result.push_str(&self.prefix);
            }

            // Collect content from last_break_pos up to this break point
            let mut pos = 0;
            let mut prev_fragment_was_code = false;
            for fragment in &self.fragments {
                match fragment {
                    ContentFragment::Text { content, .. } => {
                        let fragment_start = pos;
                        let fragment_end = pos + content.len();

                        if fragment_end <= last_break_pos {
                            // This fragment is before the current line
                            pos = fragment_end;
                            continue;
                        }

                        if fragment_start >= break_point.position {
                            // This fragment is after the current line
                            break;
                        }

                        // This fragment overlaps with the current line
                        let start_in_fragment =
                            last_break_pos.saturating_sub(fragment_start);
                        let end_in_fragment =
                            (break_point.position - fragment_start).min(content.len());

                        // For continuation lines, skip leading spaces
                        // But preserve leading space if previous fragment was Code (for CJK spacing)
                        let mut actual_start = if line_idx > 0
                            && start_in_fragment == 0
                            && !prev_fragment_was_code
                        {
                            // Find first non-space character
                            content[start_in_fragment..end_in_fragment]
                                .find(|c: char| !c.is_whitespace())
                                .map(|i| start_in_fragment + i)
                                .unwrap_or(start_in_fragment)
                        } else {
                            start_in_fragment
                        };
                        
                        // If result ends with '(' (possibly with trailing spaces), 
                        // skip leading spaces in this fragment
                        // This handles cases like "( 4.8GB)" -> "(4.8GB)"
                        let result_trimmed = result.trim_end();
                        if result_trimmed.ends_with('(') {
                            actual_start = content[actual_start..end_in_fragment]
                                .find(|c: char| !c.is_whitespace())
                                .map(|i| actual_start + i)
                                .unwrap_or(end_in_fragment); // If all spaces, skip to end
                        }

                        if actual_start < content.len() && end_in_fragment > actual_start
                        {
                            result.push_str(&content[actual_start..end_in_fragment]);
                        }

                        pos = fragment_end;
                        prev_fragment_was_code = false;
                    }
                    ContentFragment::Atomic { content, kind, .. } => {
                        let fragment_start = pos;
                        let fragment_end = pos + content.len();

                        if fragment_end <= last_break_pos {
                            // This fragment is before the current line
                            pos = fragment_end;
                            continue;
                        }

                        if fragment_start >= break_point.position {
                            // This fragment is after the current line
                            break;
                        }

                        // This fragment overlaps with the current line
                        // Atomic units are output as a whole on the first line that contains them
                        if fragment_start < break_point.position {
                            result.push_str(content);
                        }

                        pos = fragment_end;
                        prev_fragment_was_code = matches!(kind, AtomicKind::Code);
                    }
                }

                if pos >= break_point.position {
                    break;
                }
            }

            // Update last_break_pos for the next line
            last_break_pos = break_point.position;

            // Add newline (except for the last break)
            if line_idx < breaks.len() - 1 {
                // Remove trailing spaces before adding newline
                while result.ends_with(' ') {
                    result.pop();
                }
                result.push('\n');
            }
        }

        // Remove trailing spaces from the result
        while result.ends_with(' ') {
            result.pop();
        }

        result
    }
}

/// Get the punctuation affinity for a character
fn get_punctuation_affinity(char_str: &str) -> Option<Affinity> {
    match char_str {
        // Left-affinity: break AFTER these characters (they stay with left side)
        "," | "." | "!" | "?" | ";" | ":" | "}" | ")" | "]" | "\\" => {
            Some(Affinity::Left)
        }

        // Right-affinity: break BEFORE these characters (they stay with right side)
        "{" | "(" | "[" | "/" => Some(Affinity::Right),

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
}
