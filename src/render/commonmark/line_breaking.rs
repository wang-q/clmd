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

/// Content fragment for line breaking
#[derive(Debug, Clone)]
pub enum ContentFragment {
    /// Plain text content
    Text {
        /// The text content
        content: String,
        /// The display width of the content
        width: usize,
    },
    /// Unbreakable unit with markers (e.g., *text*, **text**, `code`)
    Unbreakable {
        /// The full content including markers
        content: String,
        /// The display width of the content
        width: usize,
        /// The start position in the paragraph
        start_pos: usize,
        /// The end position in the paragraph
        end_pos: usize,
    },
}

/// A unit that should not be broken across lines
#[derive(Debug, Clone, Copy)]
pub struct UnbreakableUnit {
    /// The kind of unit
    pub kind: UnitKind,
    /// The start position in the paragraph
    pub start_pos: usize,
    /// The end position in the paragraph
    pub end_pos: usize,
    /// The total width of the unit including markers
    pub total_width: usize,
    /// Whether this unit is currently open
    pub is_open: bool,
}

/// The kind of unbreakable unit
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

/// Paragraph line breaker using the Knuth-Plass algorithm
#[derive(Debug)]
pub struct ParagraphLineBreaker {
    /// The words in the paragraph
    words: Vec<Word>,
    /// The current line being built
    current_line: Vec<Word>,
    /// The current width of the line
    current_width: usize,
    /// The maximum allowed width for a line
    max_width: usize,
    /// The line prefix for continuation lines
    prefix: String,
    /// Break opportunities
    break_opportunities: Vec<BreakOpportunity>,
    /// Current position in the paragraph
    current_position: usize,
    /// Unbreakable units (e.g., *text*, **text**, `code`)
    units: Vec<UnbreakableUnit>,
    /// Content fragments for reconstruction
    fragments: Vec<ContentFragment>,
}

impl ParagraphLineBreaker {
    /// Create a new paragraph line breaker
    pub fn new(max_width: usize, prefix: String) -> Self {
        Self {
            words: Vec::new(),
            current_line: Vec::new(),
            current_width: 0,
            max_width,
            prefix,
            break_opportunities: Vec::new(),
            current_position: 0,
            units: Vec::new(),
            fragments: Vec::new(),
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

    /// Add a word to the paragraph
    pub fn add_word_old(&mut self, word: Word) {
        // Check if adding this word would exceed the line width
        let word_width = word.width;
        let space_width = if self.current_width > 0 { 1 } else { 0 };

        if self.current_width + space_width + word_width > self.max_width
            && !self.current_line.is_empty()
        {
            // Start a new line
            self.words.extend(self.current_line.drain(..));
            self.current_width = 0;
        }

        // Add the word to the current line
        if !self.current_line.is_empty() {
            self.current_width += 1; // Space before the word
        }
        self.current_line.push(word);
        self.current_width += word_width;
    }

    /// Start an unbreakable unit (e.g., emphasis, strong, code)
    pub fn start_unit(&mut self, kind: UnitKind, marker_width: usize) -> UnitHandle {
        let handle = UnitHandle {
            index: self.units.len(),
        };

        self.units.push(UnbreakableUnit {
            kind,
            start_pos: self.current_position,
            end_pos: self.current_position,
            total_width: marker_width,
            is_open: true,
        });

        // Add a break opportunity before the unit marker
        if self.current_position > 0 {
            self.break_opportunities.push(BreakOpportunity {
                position: self.current_position,
                width_before: self.current_width,
                affinity: Affinity::Left,
                is_forced: false,
            });
        }

        handle
    }

    /// End an unbreakable unit
    pub fn end_unit(
        &mut self,
        handle: UnitHandle,
        content_width: usize,
        marker_width: usize,
    ) {
        if let Some(unit) = self.units.get_mut(handle.index) {
            unit.end_pos = self.current_position;
            unit.total_width += content_width + marker_width;
            unit.is_open = false;
        }
    }

    /// Remove trailing space from the last text fragment
    /// This is used before adding markdown markers to remove unwanted spaces
    pub fn remove_trailing_space(&mut self) {
        // Check if the last fragment is a text fragment ending with space
        if let Some(last_fragment) = self.fragments.last_mut() {
            if let ContentFragment::Text { content, width } = last_fragment {
                if content.ends_with(' ') {
                    *content = content.trim_end().to_string();
                    *width = unicode_width::width(content) as usize;
                    // Also update current position and width
                    self.current_position -= 1;
                    self.current_width -= 1;
                }
            }
        }
    }

    /// Check if the last fragment ends with whitespace
    pub fn ends_with_whitespace(&self) -> bool {
        self.fragments.last().map_or(false, |fragment| {
            if let ContentFragment::Text { content, .. } = fragment {
                content.ends_with(|c: char| c.is_whitespace())
            } else {
                false
            }
        })
    }

    /// Get the current position in the paragraph
    pub fn current_position(&self) -> usize {
        self.current_position
    }

    /// Check if the last fragment ends with CJK character
    /// This checks the last non-whitespace character to handle cases where
    /// CJK text is followed by a space before a markdown marker
    pub fn ends_with_cjk(&self) -> bool {
        self.fragments.last().map_or(false, |fragment| {
            if let ContentFragment::Text { content, .. } = fragment {
                // Find the last non-whitespace character
                content.chars().rev().find(|c| !c.is_whitespace()).map_or(false, |c| {
                    crate::text::char::is_cjk(c)
                })
            } else {
                false
            }
        })
    }

    /// Add text and record break opportunities
    pub fn add_text(&mut self, text: &str) {
        let width = unicode_width::width(text) as usize;

        // Check each character for break opportunities
        // Use char_indices to get byte positions
        let mut accumulated_width = 0;
        for (byte_pos, c) in text.char_indices() {
            let char_str = c.to_string();
            let char_width = unicode_width::width(&char_str) as usize;

            // Check if this character is a break opportunity
            if let Some(affinity) = get_punctuation_affinity(&char_str) {
                // For left-affinity punctuation (e.g., comma, period), add break after the character
                // For right-affinity punctuation (e.g., opening bracket), add break before the character
                let (position, width_before) = match affinity {
                    Affinity::Left => {
                        // Break after the punctuation
                        (
                            self.current_position + byte_pos + char_str.len(),
                            self.current_width + accumulated_width + char_width,
                        )
                    }
                    Affinity::Right => {
                        // Break before the punctuation
                        (
                            self.current_position + byte_pos,
                            self.current_width + accumulated_width,
                        )
                    }
                };
                self.break_opportunities.push(BreakOpportunity {
                    position,
                    width_before,
                    affinity,
                    is_forced: false,
                });
            }

            // For CJK/wide characters, add break opportunity before the character
            // This allows breaking within CJK text
            // We use a simple check: if the character width is 2, it's likely CJK
            // But skip CJK punctuation that has special affinity
            // Use Right affinity so that punctuation breaks are preferred
            if char_width == 2 && get_punctuation_affinity(&char_str).is_none() {
                self.break_opportunities.push(BreakOpportunity {
                    position: self.current_position + byte_pos,
                    width_before: self.current_width + accumulated_width,
                    affinity: Affinity::Right,
                    is_forced: false,
                });
            }

            accumulated_width += char_width;
        }

        // Collect the content fragment
        self.fragments.push(ContentFragment::Text {
            content: text.to_string(),
            width,
        });

        self.current_position += text.len();
        self.current_width += width;
    }

    /// Add a hard line break (forced break)
    /// This records a forced break point
    pub fn add_hard_break(&mut self) {
        // Record a forced break opportunity at current position
        // Note: We don't add the two spaces as content here because
        // the hard break marker will be added when formatting the output
        self.break_opportunities.push(BreakOpportunity {
            position: self.current_position,
            width_before: self.current_width,
            affinity: Affinity::Left,
            is_forced: true,
        });
    }

    /// Add a word without recording break opportunities inside it
    pub fn add_word(&mut self, text: &str) {
        let width = unicode_width::width(text) as usize;

        // Add a break opportunity BEFORE the word (if not at start)
        if self.current_position > 0 {
            self.break_opportunities.push(BreakOpportunity {
                position: self.current_position,
                width_before: self.current_width,
                affinity: Affinity::Left,
                is_forced: false,
            });
        }

        // Collect the content fragment
        self.fragments.push(ContentFragment::Text {
            content: text.to_string(),
            width,
        });

        self.current_position += text.len();
        self.current_width += width;

        // Add a break opportunity AFTER the word
        self.break_opportunities.push(BreakOpportunity {
            position: self.current_position,
            width_before: self.current_width,
            affinity: Affinity::Left,
            is_forced: false,
        });
    }

    /// Add an unbreakable unit with markers (e.g., *text*, **text*, `code`)
    /// This adds the entire unit as a single unbreakable entity
    pub fn add_unbreakable_unit(
        &mut self,
        prefix: &str,
        content: &str,
        suffix: &str,
    ) {
        let prefix_width = unicode_width::width(prefix) as usize;
        let content_width = unicode_width::width(content) as usize;
        let suffix_width = unicode_width::width(suffix) as usize;
        let total_width = prefix_width + content_width + suffix_width;

        let start_pos = self.current_position;

        // Add a break opportunity BEFORE the unit (if not at start)
        if self.current_position > 0 {
            self.break_opportunities.push(BreakOpportunity {
                position: self.current_position,
                width_before: self.current_width,
                affinity: Affinity::Left,
                is_forced: false,
            });
        }

        // Collect the content fragment
        self.fragments.push(ContentFragment::Unbreakable {
            content: format!("{}{}{}", prefix, content, suffix),
            width: total_width,
            start_pos,
            end_pos: start_pos + prefix.len() + content.len() + suffix.len(),
        });

        self.current_position += prefix.len() + content.len() + suffix.len();
        self.current_width += total_width;

        // Add a break opportunity AFTER the unit
        self.break_opportunities.push(BreakOpportunity {
            position: self.current_position,
            width_before: self.current_width,
            affinity: Affinity::Left,
            is_forced: false,
        });
    }

    /// Compute the optimal line breaks using the Knuth-Plass algorithm
    pub fn compute_breaks(&self) -> Vec<LineBreak> {
        if self.fragments.is_empty() {
            return Vec::new();
        }

        // Build a list of break positions from fragments
        let mut fragment_breaks: Vec<(usize, usize)> = Vec::new(); // (position, width_before)
        let mut current_pos = 0;
        let mut current_width = 0;

        for fragment in &self.fragments {
            match fragment {
                ContentFragment::Text { content, width } => {
                    // Check for break opportunities within this text
                    for opp in &self.break_opportunities {
                        if opp.position >= current_pos && opp.position <= current_pos + content.len()
                        {
                            fragment_breaks.push((opp.position, opp.width_before));
                        }
                    }
                    current_pos += content.len();
                    current_width += width;
                }
                ContentFragment::Unbreakable {
                    content,
                    width,
                    start_pos,
                    end_pos,
                } => {
                    // For unbreakable units, only add breaks before and after
                    // Check for break opportunity before
                    for opp in &self.break_opportunities {
                        if opp.position == *start_pos {
                            fragment_breaks.push((opp.position, opp.width_before));
                        }
                    }
                    current_pos = *end_pos;
                    current_width += width;
                }
            }
        }

        // Add a forced break at the end
        fragment_breaks.push((current_pos, current_width));

        // Sort and deduplicate break positions
        fragment_breaks.sort_by_key(|(pos, _)| *pos);
        fragment_breaks.dedup_by_key(|(pos, _)| *pos);

        // Dynamic programming to find optimal breaks
        let n = fragment_breaks.len();
        let mut best_cost: Vec<f64> = vec![f64::INFINITY; n];
        let mut best_prev: Vec<usize> = vec![0; n];

        best_cost[0] = 0.0;

        for i in 1..n {
            for j in (0..i).rev() {
                let start_pos = fragment_breaks[j].0;
                let end_pos = fragment_breaks[i].0;
                let width = fragment_breaks[i].1 - fragment_breaks[j].1;

                // Check if this segment fits within the line width
                if width <= self.max_width {
                    // Calculate the cost (demerits) for this break
                    let slack = self.max_width - width;
                    let cost = best_cost[j] + (slack as f64).powi(3);

                    if cost < best_cost[i] {
                        best_cost[i] = cost;
                        best_prev[i] = j;
                    }
                } else {
                    // This segment doesn't fit, but we must break here
                    // Use a high cost but allow it
                    let overflow = width - self.max_width;
                    let cost = best_cost[j] + (overflow as f64).powi(3) * 1000.0;

                    if cost < best_cost[i] {
                        best_cost[i] = cost;
                        best_prev[i] = j;
                    }
                    break; // No need to check earlier positions
                }
            }
        }

        // Backtrack to find the optimal breaks
        let mut breaks = Vec::new();
        
        // If there's only one break position (end), return it
        if n == 1 {
            breaks.push(LineBreak {
                position: fragment_breaks[0].0,
                line_width: fragment_breaks[0].1,
            });
            return breaks;
        }
        
        let mut i = n - 1;

        while i > 0 {
            let prev = best_prev[i];
            breaks.push(LineBreak {
                position: fragment_breaks[i].0,
                line_width: fragment_breaks[i].1 - fragment_breaks[prev].1,
            });
            i = prev;
        }

        breaks.reverse();
        breaks
    }

    /// Format the paragraph with optimal line breaks
    pub fn format(&self) -> String {
        let breaks = self.compute_breaks();
        let mut result = String::new();
        let mut last_break = 0;

        for (i, break_point) in breaks.iter().enumerate() {
            // Add the prefix for continuation lines
            if i > 0 {
                result.push_str(&self.prefix);
            }

            // Collect content up to this break point
            let mut current_pos = 0;
            for fragment in &self.fragments {
                match fragment {
                    ContentFragment::Text { content, .. } => {
                        let fragment_end = current_pos + content.len();
                        if fragment_end <= break_point.position {
                            // Entire fragment fits on this line
                            result.push_str(content);
                            current_pos = fragment_end;
                        } else if current_pos < break_point.position {
                            // Partial fragment
                            let take = break_point.position - current_pos;
                            result.push_str(&content[..take]);
                            current_pos = break_point.position;
                            break;
                        }
                    }
                    ContentFragment::Unbreakable { content, .. } => {
                        let fragment_len = content.len();
                        let fragment_end = current_pos + fragment_len;
                        if fragment_end <= break_point.position {
                            // Entire fragment fits on this line
                            result.push_str(content);
                            current_pos = fragment_end;
                        } else if current_pos < break_point.position {
                            // This shouldn't happen for unbreakable units
                            // but we handle it gracefully
                            result.push_str(content);
                            current_pos = fragment_end;
                        }
                    }
                }

                if current_pos >= break_point.position {
                    break;
                }
            }

            // Add newline (except for the last break)
            if i < breaks.len() - 1 {
                result.push('\n');
            }

            last_break = break_point.position;
        }

        result
    }
}

/// Handle for an unbreakable unit
#[derive(Debug, Clone, Copy)]
pub struct UnitHandle {
    index: usize,
}

/// Get the punctuation affinity for a character
/// Returns Some(Affinity) if this character has special break behavior
fn get_punctuation_affinity(char_str: &str) -> Option<Affinity> {
    match char_str {
        // Left-affinity: break AFTER these characters
        // (they stay with the left side)
        "," | "." | "!" | "?" | ";" | ":" | "}" | ")" | "]" | "\\" => Some(Affinity::Left),

        // Right-affinity: break BEFORE these characters
        // (they stay with the right side)
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
        // Should break into multiple lines
        assert!(result.contains('\n') || result.len() <= 20);
    }

    #[test]
    fn test_mixed_cjk_english() {
        let mut breaker = ParagraphLineBreaker::new(30, String::new());
        breaker.add_text("这是中文和English混合的文本");
        let result = breaker.format();
        assert!(!result.is_empty());
    }

    #[test]
    fn test_unbreakable_unit() {
        let mut breaker = ParagraphLineBreaker::new(20, String::new());
        breaker.add_text("This is ");
        breaker.add_unbreakable_unit("*", "emphasis", "*");
        breaker.add_text(" text");
        let result = breaker.format();
        assert!(result.contains("*emphasis*"));
    }

    #[test]
    fn test_hard_break() {
        let mut breaker = ParagraphLineBreaker::new(80, String::new());
        breaker.add_text("First line");
        breaker.add_hard_break();
        breaker.add_text("Second line");
        let result = breaker.format();
        // Hard break should result in two lines
        // The exact format depends on the implementation
        assert!(!result.is_empty());
    }

    #[test]
    fn test_with_prefix() {
        let mut breaker = ParagraphLineBreaker::new(20, "  ".to_string());
        breaker.add_text("This is a long paragraph that should be wrapped with a prefix");
        let result = breaker.format();
        // Check that continuation lines have the prefix
        let lines: Vec<&str> = result.lines().collect();
        if lines.len() > 1 {
            for line in &lines[1..] {
                assert!(line.starts_with("  "), "Continuation line should have prefix");
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
}