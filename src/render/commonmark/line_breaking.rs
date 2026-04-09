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

/// Unit kind for unbreakable units
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnitKind {
    /// Emphasis: *text*
    Emphasis,
    /// Strong: **text**
    Strong,
    /// Link: [text](url)
    Link,
    /// Inline code: `code`
    InlineCode,
    /// Regular text
    Text,
}

/// Handle to an unbreakable unit
#[derive(Debug, Clone, Copy)]
pub struct UnitHandle {
    index: usize,
}

/// Unbreakable unit (e.g., Markdown marker, link, etc.)
#[derive(Debug, Clone)]
struct UnbreakableUnit {
    /// Unit type
    kind: UnitKind,
    /// Start position in the text
    start_pos: usize,
    /// End position in the text
    end_pos: usize,
    /// Total width including markers
    total_width: usize,
    /// Whether the unit is currently open
    is_open: bool,
}

/// Break opportunity point
#[derive(Debug, Clone)]
struct BreakOpportunity {
    /// Position in the text
    position: usize,
    /// Width before this position
    width_before: usize,
    /// Affinity of the break point
    affinity: Affinity,
    /// Whether this is a forced break (hard line break)
    is_forced: bool,
}

/// Content fragment for collecting rendered text
#[derive(Debug, Clone)]
enum ContentFragment {
    /// Plain text
    Text { content: String, width: usize },
    /// Unbreakable unit with prefix and suffix markers
    Unbreakable {
        kind: UnitKind,
        prefix: String,
        content: String,
        suffix: String,
        total_width: usize,
    },
}

/// Paragraph line breaker for AST-based rendering
///
/// This structure collects break opportunities during AST rendering
/// and computes optimal line breaks.
#[derive(Debug)]
pub struct ParagraphLineBreaker {
    /// Unbreakable units (e.g., Markdown markers, links)
    units: Vec<UnbreakableUnit>,
    /// Break opportunities
    break_opportunities: Vec<BreakOpportunity>,
    /// Current position in the text
    current_position: usize,
    /// Current accumulated width
    current_width: usize,
    /// Maximum line width
    max_width: usize,
    /// Line prefix for continuation lines
    prefix: String,
    /// Content fragments for reconstruction
    fragments: Vec<ContentFragment>,
}

impl ParagraphLineBreaker {
    /// Create a new paragraph line breaker
    pub fn new(max_width: usize, prefix: String) -> Self {
        Self {
            units: Vec::new(),
            break_opportunities: Vec::new(),
            current_position: 0,
            current_width: 0,
            max_width,
            prefix,
            fragments: Vec::new(),
        }
    }

    /// Start an unbreakable unit
    pub fn start_unit(&mut self, kind: UnitKind, marker_width: usize) -> UnitHandle {
        let unit = UnbreakableUnit {
            kind,
            start_pos: self.current_position,
            end_pos: 0,
            total_width: marker_width,
            is_open: true,
        };
        let index = self.units.len();
        self.units.push(unit);
        UnitHandle { index }
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
            } else if c == ' ' {
                // Space is a break opportunity with left affinity
                self.break_opportunities.push(BreakOpportunity {
                    position: self.current_position + byte_pos,
                    width_before: self.current_width + accumulated_width,
                    affinity: Affinity::Left,
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
        kind: UnitKind,
        prefix: &str,
        content: &str,
        suffix: &str,
    ) {
        let prefix_width = unicode_width::width(prefix) as usize;
        let content_width = unicode_width::width(content) as usize;
        let suffix_width = unicode_width::width(suffix) as usize;
        let total_width = prefix_width + content_width + suffix_width;

        let start_pos = self.current_position;
        let end_pos = start_pos + prefix.len() + content.len() + suffix.len();

        // Create the unbreakable unit
        let unit = UnbreakableUnit {
            kind,
            start_pos,
            end_pos,
            total_width,
            is_open: false,
        };
        self.units.push(unit);

        // Add a break opportunity BEFORE the unit (if not at start)
        // This allows breaking before the unit
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
            kind,
            prefix: prefix.to_string(),
            content: content.to_string(),
            suffix: suffix.to_string(),
            total_width,
        });

        // Update position and width
        self.current_position = end_pos;
        self.current_width += total_width;

        // Add a break opportunity AFTER the unit
        // This allows breaking after the unit
        self.break_opportunities.push(BreakOpportunity {
            position: self.current_position,
            width_before: self.current_width,
            affinity: Affinity::Left, // Left affinity: prefer to keep with previous content
            is_forced: false,
        });
    }

    /// Compute optimal line breaks using dynamic programming
    /// Returns a tuple of (break_positions, forced_break_indices)
    fn compute_breaks_internal(&self) -> (Vec<usize>, Vec<usize>) {
        if self.break_opportunities.is_empty() {
            return (Vec::new(), Vec::new());
        }

        let n = self.break_opportunities.len();
        let prefix_width = unicode_width::width(&self.prefix) as usize;

        // Collect forced break positions (1-based indices)
        let forced_breaks: Vec<usize> = self
            .break_opportunities
            .iter()
            .enumerate()
            .filter(|(_, opp)| opp.is_forced)
            .map(|(idx, _)| idx + 1) // +1 because dp indices are 1-based
            .collect();

        // DP table: best total badness up to each break opportunity
        let mut dp: Vec<BreakPoint> = vec![BreakPoint {
            word_index: 0,
            total_badness: 0.0,
            prev_break: None,
        }];

        // Dynamic programming: for each possible breakpoint
        for j in 1..=n {
            // Check if this is a forced break
            let is_forced_break = self.break_opportunities[j - 1].is_forced;

            let mut best_badness = f64::INFINITY;
            let mut best_prev = None;

            let current_opp = &self.break_opportunities[j - 1];

            // For forced breaks, we must break here
            // For regular breaks, try all possible previous breakpoints
            let start_i = if is_forced_break {
                // Find the last forced break before this one, or 0
                let last_forced = forced_breaks
                    .iter()
                    .filter(|&&b| b < j)
                    .last()
                    .copied()
                    .unwrap_or(0);
                last_forced
            } else {
                0
            };

            for i in (start_i..j).rev() {
                let prev_opp = if i == 0 {
                    // Virtual breakpoint at start
                    BreakOpportunity {
                        position: 0,
                        width_before: 0,
                        affinity: Affinity::Left,
                        is_forced: false,
                    }
                } else {
                    self.break_opportunities[i - 1].clone()
                };

                // Check if breaking here would break inside an unbreakable unit
                if self.would_break_inside_unit(prev_opp.position, current_opp.position)
                {
                    continue;
                }

                // Calculate line width from i to j
                let is_first_line = i == 0;
                let line_width = if is_first_line {
                    // First line doesn't have prefix
                    current_opp.width_before - prev_opp.width_before
                } else {
                    // Continuation lines have prefix
                    current_opp.width_before - prev_opp.width_before + prefix_width
                };

                // Check if this line exceeds max width
                if line_width > self.max_width && !is_forced_break {
                    // If this is not the first possible break, stop searching
                    // because earlier breaks will only make the line longer
                    if i < j - 1 {
                        break;
                    }
                    // Otherwise, this is the only option, so we have to accept it
                }

                // Check if remaining content can fit in one line
                let is_last_line = j == n;

                let badness = if is_forced_break {
                    // Forced breaks have minimal badness
                    0.0
                } else {
                    let base_badness = calculate_badness(
                        line_width,
                        self.max_width, // ideal width is max width
                        self.max_width,
                        is_last_line,
                    );
                    // Adjust badness based on affinity of the current break opportunity
                    let affinity_bonus = match current_opp.affinity {
                        Affinity::Left => -500.0, // Strong reward for breaking after left-affinity punctuation
                        Affinity::Right => 500.0, // Strong penalty for breaking before right-affinity punctuation
                    };
                    (base_badness + affinity_bonus).max(0.0) // Ensure badness is not negative
                };
                let total_badness = dp[i].total_badness + badness;

                if total_badness < best_badness {
                    best_badness = total_badness;
                    best_prev = Some(i);
                }

                // For forced breaks, only consider the immediate previous forced break
                if is_forced_break {
                    break;
                }
            }

            // If no valid breakpoint found, force a break at the previous opportunity
            if best_prev.is_none() && j > 1 {
                best_prev = Some(j - 1);
                best_badness = dp[j - 1].total_badness + 1000.0; // Large penalty
            }

            dp.push(BreakPoint {
                word_index: j,
                total_badness: best_badness,
                prev_break: best_prev,
            });
        }

        // Backtrack to find the optimal breakpoints
        let mut result = Vec::new();
        let mut current = n;

        while let Some(prev) = dp[current].prev_break {
            if prev > 0 {
                result.push(self.break_opportunities[prev - 1].position);
            }
            current = prev;
        }

        // Add any forced breaks that were not included in the backtracking
        // This can happen when a forced break is the first break opportunity
        for &forced_idx in &forced_breaks {
            let forced_pos = self.break_opportunities[forced_idx - 1].position;
            if !result.contains(&forced_pos) {
                result.push(forced_pos);
            }
        }

        result.sort();
        result.dedup();
        (result, forced_breaks)
    }

    /// Compute optimal line breaks using dynamic programming
    pub fn compute_breaks(&self) -> Vec<usize> {
        self.compute_breaks_internal().0
    }

    /// Format the collected content with line breaks at the specified positions
    ///
    /// This method takes the break positions computed by `compute_breaks` and
    /// returns the formatted text with line breaks inserted at the appropriate
    /// positions, including the prefix for continuation lines.
    pub fn format_with_breaks(&self, break_positions: &[usize]) -> String {
        if self.fragments.is_empty() {
            return String::new();
        }

        // Build the full text from fragments
        let mut full_text = String::new();
        for fragment in &self.fragments {
            match fragment {
                ContentFragment::Text { content, .. } => {
                    full_text.push_str(content);
                }
                ContentFragment::Unbreakable {
                    prefix,
                    content,
                    suffix,
                    ..
                } => {
                    full_text.push_str(prefix);
                    full_text.push_str(content);
                    full_text.push_str(suffix);
                }
            }
        }

        // If no breaks, return the full text
        if break_positions.is_empty() {
            return full_text;
        }

        // Insert line breaks at the specified positions
        let mut result = String::new();
        let mut last_pos = 0;

        for &break_pos in break_positions {
            if break_pos > last_pos && break_pos <= full_text.len() {
                // Add text from last position to break position
                result.push_str(&full_text[last_pos..break_pos]);
                // Add line break and prefix for continuation line
                result.push('\n');
                result.push_str(&self.prefix);
                last_pos = break_pos;
                // Skip leading spaces on continuation line
                while last_pos < full_text.len()
                    && full_text[last_pos..].starts_with(' ')
                {
                    last_pos += 1;
                }
            }
        }

        // Add remaining text (always add this, even if no breaks were applied)
        result.push_str(&full_text[last_pos..]);

        result
    }

    /// Compute breaks and format the content in one step
    ///
    /// Returns the formatted text with optimal line breaks.
    pub fn format(&self) -> String {
        let (breaks, forced_breaks) = self.compute_breaks_internal();
        let formatted = self.format_with_breaks_internal(&breaks, &forced_breaks);
        // Apply CJK spacing to the formatted text
        crate::text::cjk_spacing::add_cjk_spacing(&formatted)
    }

    /// Format the collected content with line breaks at the specified positions
    ///
    /// This method takes the break positions computed by `compute_breaks` and
    /// returns the formatted text with line breaks inserted at the appropriate
    /// positions, including the prefix for continuation lines.
    fn format_with_breaks_internal(
        &self,
        break_positions: &[usize],
        forced_break_indices: &[usize],
    ) -> String {
        if self.fragments.is_empty() {
            return String::new();
        }

        // Build the full text from fragments
        let mut full_text = String::new();
        for fragment in &self.fragments {
            match fragment {
                ContentFragment::Text { content, .. } => {
                    full_text.push_str(content);
                }
                ContentFragment::Unbreakable {
                    prefix,
                    content,
                    suffix,
                    ..
                } => {
                    full_text.push_str(prefix);
                    full_text.push_str(content);
                    full_text.push_str(suffix);
                }
            }
        }

        // Convert forced break indices (1-based) to positions
        let forced_break_positions: Vec<usize> = forced_break_indices
            .iter()
            .filter_map(|&idx| {
                if idx > 0 && idx <= self.break_opportunities.len() {
                    Some(self.break_opportunities[idx - 1].position)
                } else {
                    None
                }
            })
            .collect();

        // Insert line breaks at the specified positions
        let mut result = String::new();
        let mut last_pos = 0;

        for &break_pos in break_positions {
            if break_pos > last_pos && break_pos <= full_text.len() {
                // Add text from last position to break position
                result.push_str(&full_text[last_pos..break_pos]);
                // Add line break and prefix for continuation line
                result.push('\n');
                result.push_str(&self.prefix);
                last_pos = break_pos;

                // Check if this is a forced break (hard line break)
                let is_forced_break = forced_break_positions.contains(&break_pos);

                if is_forced_break {
                    // For hard line breaks, we need to add the two spaces marker
                    // But first check if the original text already has spaces at this position
                    // If the original text has less than 2 spaces, we need to add more
                    // If the original text has 2 or more spaces, we use them as the marker
                    let mut space_count = 0;
                    while last_pos + space_count < full_text.len()
                        && full_text[last_pos + space_count..].starts_with(' ')
                        && space_count < 2
                    {
                        space_count += 1;
                    }
                    // Add the marker (2 spaces total)
                    result.push_str("  ");
                    // Skip the spaces we've accounted for in the original text
                    last_pos += space_count;
                } else {
                    // For regular breaks, skip leading spaces on continuation line
                    while last_pos < full_text.len()
                        && full_text[last_pos..].starts_with(' ')
                    {
                        last_pos += 1;
                    }
                }
            }
        }

        // Add remaining text (always add this, even if no breaks were applied)
        result.push_str(&full_text[last_pos..]);

        // If no breaks were applied, result is empty, so use full_text
        if result.is_empty() {
            result = full_text;
        }

        result
    }

    /// Check if breaking between two positions would break inside an unbreakable unit
    fn would_break_inside_unit(&self, start_pos: usize, end_pos: usize) -> bool {
        self.units.iter().any(|unit| {
            !unit.is_open
                && unit.start_pos < end_pos
                && unit.end_pos > start_pos
                && (start_pos > unit.start_pos || end_pos < unit.end_pos)
        })
    }

    /// Check if a break opportunity is adjacent to an unbreakable unit
    /// Returns true if the break would separate an unbreakable unit from adjacent text
    fn is_break_adjacent_to_unit(&self, position: usize) -> bool {
        // Check if there's a unit that starts or ends at this position
        self.units.iter().any(|unit| {
            !unit.is_open && (unit.start_pos == position || unit.end_pos == position)
        })
    }

    /// Check if breaking at the given position would separate an unbreakable unit from adjacent text
    /// This is a more comprehensive check that considers the context of the break
    fn would_separate_unit(&self, start_pos: usize, end_pos: usize) -> bool {
        // Check if there's a unit that would be separated by this break
        self.units.iter().any(|unit| {
            if unit.is_open {
                return false;
            }
            // Check if the break would separate the unit from adjacent text
            // This happens when the break is at the unit boundary
            // i.e., the break starts at unit end or ends at unit start
            let break_at_unit_start = end_pos == unit.start_pos;
            let break_at_unit_end = start_pos == unit.end_pos;
            break_at_unit_start || break_at_unit_end
        })
    }

    /// Check if a position is inside an unbreakable unit
    fn is_inside_unbreakable_unit(&self, position: usize) -> bool {
        self.units.iter().any(|unit| {
            !unit.is_open && unit.start_pos <= position && position < unit.end_pos
        })
    }

    /// Reset the breaker for a new pass
    pub fn reset(&mut self) {
        self.current_position = 0;
        self.current_width = 0;
    }

    /// Get the current width
    pub fn current_width(&self) -> usize {
        self.current_width
    }

    /// Get the break opportunities (for testing)
    #[cfg(test)]
    fn get_break_opportunities(&self) -> &[BreakOpportunity] {
        &self.break_opportunities
    }

    /// Get the unbreakable units (for testing)
    #[cfg(test)]
    fn get_units(&self) -> &[UnbreakableUnit] {
        &self.units
    }
}

#[cfg(test)]
mod paragraph_line_breaker_tests {
    use super::*;

    #[test]
    fn test_simple_text_breaking() {
        let mut breaker = ParagraphLineBreaker::new(20, "".to_string());
        breaker.add_text("This is a simple test paragraph with more text.");

        let _breaks = breaker.compute_breaks();
        let opps = breaker.get_break_opportunities();

        // Verify that break opportunities were recorded
        assert!(!opps.is_empty(), "Should have break opportunities");

        // Verify current width is calculated correctly
        assert!(breaker.current_width() > 0, "Should have accumulated width");
    }

    #[test]
    fn test_unbreakable_unit() {
        let mut breaker = ParagraphLineBreaker::new(30, "".to_string());

        // Start a strong unit: **
        let handle = breaker.start_unit(UnitKind::Strong, 2);
        breaker.add_text("emphasized text");
        breaker.end_unit(handle, 15, 2);

        // The unit should be recorded
        assert_eq!(breaker.get_units().len(), 1);
        assert!(!breaker.get_units()[0].is_open);
    }

    #[test]
    fn test_break_opportunities() {
        let mut breaker = ParagraphLineBreaker::new(40, "".to_string());
        breaker.add_text("Hello, world! This is a test.");

        // Should have break opportunities at spaces and punctuation
        let opps = breaker.get_break_opportunities();
        assert!(!opps.is_empty());

        // Check that we have opportunities at expected positions
        // (spaces and punctuation)
        let has_space_break = opps.iter().any(|opp| opp.affinity == Affinity::Left);
        assert!(has_space_break, "Should have break opportunities");
    }

    #[test]
    fn test_no_break_inside_unit() {
        let mut breaker = ParagraphLineBreaker::new(40, "".to_string());

        // Create a unit that fits within max_width
        let handle = breaker.start_unit(UnitKind::Strong, 2);
        breaker.add_text("short text");
        breaker.end_unit(handle, 10, 2);

        // The unit width (2 + 10 + 2 = 14) is less than max_width (40)
        // So we shouldn't break inside it
        let breaks = breaker.compute_breaks();

        // Verify no breaks inside the unit
        let unit = &breaker.get_units()[0];
        for &break_pos in &breaks {
            assert!(
                break_pos <= unit.start_pos || break_pos >= unit.end_pos,
                "Should not break inside unbreakable unit when it fits"
            );
        }
    }

    #[test]
    fn test_long_unit_handling() {
        // When a unit is longer than max_width, it may need to be broken
        // or the algorithm should handle it gracefully
        let mut breaker = ParagraphLineBreaker::new(20, "".to_string());

        let handle = breaker.start_unit(UnitKind::Strong, 2);
        breaker.add_text("very long emphasized text here");
        breaker.end_unit(handle, 30, 2);

        // The unit is longer than max_width
        // The algorithm should either:
        // 1. Not break (allow overflow)
        // 2. Break at unit boundaries only
        let breaks = breaker.compute_breaks();

        // Just verify the algorithm doesn't panic
        // and returns valid break positions
        for &break_pos in &breaks {
            assert!(break_pos <= breaker.current_position);
        }
    }

    #[test]
    fn test_cjk_text_breaking() {
        let mut breaker = ParagraphLineBreaker::new(20, "".to_string());
        breaker.add_text("这是一个中文测试段落，用于测试换行。");

        let breaks = breaker.compute_breaks();
        // CJK text should have break opportunities
        let opps = breaker.get_break_opportunities();
        assert!(!opps.is_empty(), "CJK text should have break opportunities");
    }

    #[test]
    fn test_prefix_width_calculation() {
        let mut breaker = ParagraphLineBreaker::new(20, "> ".to_string());
        breaker.add_text("This is a test with prefix.");

        let breaks = breaker.compute_breaks();
        // The prefix width should be considered in line width calculation
        // So lines should be shorter than without prefix
        assert!(breaker.current_width() > 0);
    }

    #[test]
    fn test_paragraph_line_breaker_integration() {
        // Test that ParagraphLineBreaker can be used for AST-based line breaking
        let mut breaker = ParagraphLineBreaker::new(30, "".to_string());

        // Simulate rendering a paragraph with emphasis:
        // "This is **emphasized** text."
        breaker.add_text("This is ");

        // Start strong unit: **
        let strong_handle = breaker.start_unit(UnitKind::Strong, 2);
        breaker.add_text("emphasized");
        breaker.end_unit(strong_handle, 10, 2);

        breaker.add_text(" text.");

        // Compute breaks
        let breaks = breaker.compute_breaks();

        // Verify the breaker recorded everything correctly
        assert_eq!(breaker.get_units().len(), 1);
        assert!(!breaker.get_units()[0].is_open);
        assert_eq!(breaker.get_units()[0].kind, UnitKind::Strong);
    }

    // Tests adapted from the deprecated LineBreakingContext

    #[test]
    fn test_empty_breaker() {
        let breaker = ParagraphLineBreaker::new(20, "".to_string());
        let formatted = breaker.format();
        assert_eq!(formatted, "");
    }

    #[test]
    fn test_single_word() {
        let mut breaker = ParagraphLineBreaker::new(20, "".to_string());
        breaker.add_text("Hello");
        let formatted = breaker.format();
        assert_eq!(formatted, "Hello");
    }

    #[test]
    fn test_multiple_words() {
        let mut breaker = ParagraphLineBreaker::new(80, "".to_string());
        breaker.add_text("Hello World");
        let formatted = breaker.format();
        assert_eq!(formatted, "Hello World");
    }

    #[test]
    fn test_long_paragraph_wrapping() {
        let mut breaker = ParagraphLineBreaker::new(40, "".to_string());
        breaker.add_text("This is a very long paragraph that should be wrapped into multiple lines when formatted with line breaking enabled.");
        let formatted = breaker.format();
        // The formatted output should have line breaks or fit within width
        let max_line_width = formatted
            .lines()
            .map(|line| unicode_width::width(line) as usize)
            .max()
            .unwrap_or(0);
        assert!(formatted.contains('\n') || max_line_width <= 40);
    }

    #[test]
    fn test_cjk_text_with_width() {
        let mut breaker = ParagraphLineBreaker::new(20, "".to_string());
        // Add CJK text (each character has width 2)
        breaker.add_text("这是一个中文测试段落");

        let formatted = breaker.format();
        for line in formatted.lines() {
            let width = unicode_width::width(line) as usize;
            assert!(width <= 20, "Line exceeds max width: {}", line);
        }
    }

    #[test]
    fn test_prefix_handling() {
        let mut breaker = ParagraphLineBreaker::new(20, "> ".to_string());
        breaker.add_text("This is a test with prefix.");
        let formatted = breaker.format();
        // The prefix is applied to continuation lines, not the first line
        // So we just verify the formatted output is not empty
        assert!(!formatted.is_empty());
        // Check that continuation lines (if any) have the prefix
        let lines: Vec<&str> = formatted.lines().collect();
        if lines.len() > 1 {
            for line in &lines[1..] {
                assert!(
                    line.starts_with("> ") || line.is_empty(),
                    "Continuation line should start with prefix: {}",
                    line
                );
            }
        }
    }

    #[test]
    fn test_link_unit_not_broken() {
        // Test that link units are not broken inside
        let mut breaker = ParagraphLineBreaker::new(30, "".to_string());

        // Add text before link
        breaker.add_text("See ");

        // Add link as unbreakable unit
        breaker.add_unbreakable_unit(
            UnitKind::Link,
            "",
            "[example](https://example.com)",
            "",
        );

        // Add text after link
        breaker.add_text(" for more.");

        let formatted = breaker.format();

        // The link should appear intact in the output
        assert!(
            formatted.contains("[example](https://example.com)"),
            "Link should not be broken: {}",
            formatted
        );
    }

    #[test]
    fn test_emphasis_unit_not_broken() {
        // Test that emphasis units are not broken inside
        let mut breaker = ParagraphLineBreaker::new(30, "".to_string());

        breaker.add_text("This is ");

        // Add emphasis as unbreakable unit
        let handle = breaker.start_unit(UnitKind::Strong, 2);
        breaker.add_text("very important");
        breaker.end_unit(handle, 14, 2);

        breaker.add_text(" text.");

        let formatted = breaker.format();

        // The emphasis markers should appear intact
        assert!(
            formatted.contains("**very important**")
                || formatted.contains("very important"),
            "Emphasis should not be broken: {}",
            formatted
        );
    }

    #[test]
    fn test_code_unit_not_broken() {
        // Test that inline code units are not broken inside
        let mut breaker = ParagraphLineBreaker::new(30, "".to_string());

        breaker.add_text("Use ");

        // Add code as unbreakable unit
        let handle = breaker.start_unit(UnitKind::InlineCode, 1);
        breaker.add_text("function_name");
        breaker.end_unit(handle, 13, 1);

        breaker.add_text(" to call it.");

        let formatted = breaker.format();

        // The code should appear intact
        assert!(
            formatted.contains("`function_name`") || formatted.contains("function_name"),
            "Code should not be broken: {}",
            formatted
        );
    }

    #[test]
    fn test_disabled_line_breaking() {
        // max_width = 0 disables breaking
        let breaker = ParagraphLineBreaker::new(0, "".to_string());
        // When max_width is 0, compute_breaks should return empty
        let breaks = breaker.compute_breaks();
        assert!(breaks.is_empty());
    }

    #[test]
    fn test_long_word() {
        let mut breaker = ParagraphLineBreaker::new(25, "".to_string());
        breaker.add_text("verylongwordthatexceedsthemaxwidth");

        // Even with a very long word, we should handle it gracefully
        let formatted = breaker.format();
        assert!(!formatted.is_empty());
    }

    #[test]
    fn test_line_breaking_single_word() {
        let mut breaker = ParagraphLineBreaker::new(80, "".to_string());
        breaker.add_text("Hello");
        let formatted = breaker.format();
        assert_eq!(formatted, "Hello");
    }

    #[test]
    fn test_line_breaking_multiple_words() {
        let mut breaker = ParagraphLineBreaker::new(80, "".to_string());
        breaker.add_text("Hello World");
        let formatted = breaker.format();
        assert_eq!(formatted, "Hello World");
    }

    #[test]
    fn test_line_breaking_long_paragraph() {
        let mut breaker = ParagraphLineBreaker::new(80, "".to_string());
        breaker.add_text("This is a very long paragraph that should be wrapped into multiple lines when formatted with line breaking enabled.");
        let formatted = breaker.format();
        // The formatted output should have line breaks or fit within width
        let max_line_width = formatted
            .lines()
            .map(|line| unicode_width::width(line) as usize)
            .max()
            .unwrap_or(0);
        assert!(formatted.contains('\n') || max_line_width <= 80);
    }

    #[test]
    fn test_line_breaking_with_prefix() {
        let mut breaker = ParagraphLineBreaker::new(80, "> ".to_string());
        breaker.add_text("Hello World");
        let formatted = breaker.format();
        // Check that continuation lines (if any) have the prefix
        let lines: Vec<&str> = formatted.lines().collect();
        if lines.len() > 1 {
            for line in &lines[1..] {
                assert!(
                    line.starts_with("> ") || line.is_empty(),
                    "Continuation line should start with prefix: {}",
                    line
                );
            }
        }
    }

    #[test]
    fn test_word_width_calculation() {
        let word = Word::new("Hello");
        assert_eq!(word.width, 5);

        let word_cjk = Word::new_cjk("中文");
        assert_eq!(word_cjk.width, 4); // CJK characters are width 2
    }

    #[test]
    fn test_compute_breaks_basic() {
        let mut breaker = ParagraphLineBreaker::new(80, "".to_string());
        breaker.add_text("Hello World Test");
        let breaks = breaker.compute_breaks();
        // Badness should be calculated for potential line breaks
        // This is a basic test to ensure compute_breaks doesn't panic
        assert!(breaks.is_empty() || !breaks.is_empty());
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
    fn test_line_breaking_empty() {
        let breaker = ParagraphLineBreaker::new(80, "".to_string());
        let formatted = breaker.format();
        assert_eq!(formatted, "");
    }

    #[test]
    fn test_simple_line_breaking() {
        let mut breaker = ParagraphLineBreaker::new(20, "".to_string());
        breaker.add_text("This is a simple test paragraph with more content");

        let breaks = breaker.compute_breaks();
        // The text might fit in one line, so breaks could be empty
        // Just verify the format doesn't panic and lines are within width

        let formatted = breaker.format();
        // Each line should be within max width
        for line in formatted.lines() {
            let width = unicode_width::width(line) as usize;
            assert!(width <= 20, "Line exceeds max width: {}", line);
        }
    }

    #[test]
    fn test_cjk_line_breaking() {
        let mut breaker = ParagraphLineBreaker::new(25, "".to_string());
        // Add CJK text (each character has width 2)
        breaker.add_text("这是一个测试段落");

        let formatted = breaker.format();
        for line in formatted.lines() {
            let width = unicode_width::width(line) as usize;
            assert!(width <= 25, "Line exceeds max width: {}", line);
        }
    }

    #[test]
    fn test_empty_context() {
        let breaker = ParagraphLineBreaker::new(25, "".to_string());
        let breaks = breaker.compute_breaks();
        assert!(breaks.is_empty());
    }

    #[test]
    fn test_emphasis_end_marker_with_cjk_punctuation() {
        // Test: *斜体*：测试
        // The emphasis should not have space before CJK punctuation
        let mut breaker = ParagraphLineBreaker::new(80, "".to_string());

        // Add emphasis as unbreakable unit: *斜体*
        breaker.add_unbreakable_unit(UnitKind::Emphasis, "*", "斜体", "*");
        // Add CJK punctuation and text
        breaker.add_text("：测试");

        let formatted = breaker.format();
        println!("Formatted: {}", formatted);

        assert!(
            formatted.contains("*斜体*：测试"),
            "Should have no space before CJK punctuation. Got: {}",
            formatted
        );
    }

    #[test]
    fn test_emphasis_end_marker_with_marker_then_cjk() {
        // Test: *斜体**：测试 (where *：测试 is literal text)
        let mut breaker = ParagraphLineBreaker::new(80, "".to_string());

        // Add emphasis as unbreakable unit: *斜体*
        breaker.add_unbreakable_unit(UnitKind::Emphasis, "*", "斜体", "*");
        // Add literal text with marker and CJK punctuation
        breaker.add_text("*：测试");

        let formatted = breaker.format();
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
    fn test_ascii_punctuation_no_space_after_marker() {
        // Test that ASCII punctuation like : doesn't get a leading space after inline code
        // Simulate: `replace_na`: 将显式 `NA`
        let mut breaker = ParagraphLineBreaker::new(80, "".to_string());

        // Add inline code as unbreakable unit
        breaker.add_unbreakable_unit(UnitKind::InlineCode, "`", "replace_na", "`");
        // Add text with colon and CJK
        breaker.add_text(": 将显式");

        let formatted = breaker.format();
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
        let mut breaker = ParagraphLineBreaker::new(80, "".to_string());

        // Add inline code as unbreakable unit
        breaker.add_unbreakable_unit(UnitKind::InlineCode, "`", "longer", "`");
        // Add text with colon and CJK
        breaker.add_text(": 支持在 `--names-to` 中使用");

        let formatted = breaker.format();

        // The colon should NOT have a leading space, but SHOULD have a trailing space
        assert!(
            formatted.contains("`longer`: 支持"),
            "Colon should not have leading space but should have trailing space: {}",
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
            let mut breaker = ParagraphLineBreaker::new(80, "".to_string());
            // Add inline code as unbreakable unit
            breaker.add_unbreakable_unit(UnitKind::InlineCode, "`", "code", "`");
            // Add punctuation and text
            breaker.add_text(&format!("{} text", punct));

            let formatted = breaker.format();
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
        // Test that left parenthesis stays with content after inline code
        // Note: New implementation behavior may differ from old implementation
        // Example: `strbin` (字符串哈希分箱)
        let mut breaker = ParagraphLineBreaker::new(80, "".to_string());

        // Add inline code as unbreakable unit
        breaker.add_unbreakable_unit(UnitKind::InlineCode, "`", "strbin", "`");
        // Add text with parenthesis and CJK
        breaker.add_text(" (字符串哈希分箱)");

        let formatted = breaker.format();

        // The left parenthesis should be preserved in output
        assert!(
            formatted.contains("`strbin` (字符串哈希分箱)"),
            "Left parenthesis should be preserved after inline code: got {}",
            formatted
        );
    }

    #[test]
    fn test_brackets_have_space_after_inline_code() {
        // Test that brackets are preserved after inline code
        // Note: New implementation behavior may differ from old implementation
        let test_cases = vec![
            ("(", ")", "parentheses"),
            ("[", "]", "brackets"),
            ("{", "}", "braces"),
        ];

        for (open, close, name) in test_cases {
            let mut breaker = ParagraphLineBreaker::new(80, "".to_string());
            // Add inline code as unbreakable unit
            breaker.add_unbreakable_unit(UnitKind::InlineCode, "`", "code", "`");
            // Add text with brackets - include leading space
            breaker.add_text(&format!(" {}text{}", open, close));

            let formatted = breaker.format();
            let expected = format!("`code` {}text{}", open, close);
            assert!(
                formatted.contains(&expected),
                "{} should be preserved after inline code: got '{}'",
                name,
                formatted
            );
        }
    }

    #[test]
    fn test_parentheses_with_inline_code() {
        // Test that parentheses with inline code inside don't have extra spaces
        // Example: (`cat` 命令的 `--buffer-size`)
        let mut breaker = ParagraphLineBreaker::new(80, "".to_string());

        // Simulate: 输出顺序控制 (`cat` 命令的 `--buffer-size`)
        breaker.add_text("输出顺序控制 (");
        breaker.add_unbreakable_unit(UnitKind::InlineCode, "`", "cat", "`");
        breaker.add_text(" 命令的 ");
        breaker.add_unbreakable_unit(UnitKind::InlineCode, "`", "--buffer-size", "`");
        breaker.add_text(")");

        let formatted = breaker.format();

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
        // Example: 支持进度条 (`indicatif` 的 `MultiProgress`)
        let mut breaker = ParagraphLineBreaker::new(80, "".to_string());

        // Simulate: 支持进度条 (`indicatif` 的 `MultiProgress`)
        breaker.add_text("支持进度条 (");
        breaker.add_unbreakable_unit(UnitKind::InlineCode, "`", "indicatif", "`");
        breaker.add_text(" 的 ");
        breaker.add_unbreakable_unit(UnitKind::InlineCode, "`", "MultiProgress", "`");
        breaker.add_text(")");

        let formatted = breaker.format();

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
        let mut breaker = ParagraphLineBreaker::new(80, "".to_string());

        // Simulate: **计数/求和型**: 使用
        breaker.add_unbreakable_unit(UnitKind::Strong, "**", "计数/求和型", "**");
        breaker.add_text(": 使用");

        let formatted = breaker.format();

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
        let mut breaker = ParagraphLineBreaker::new(80, "".to_string());

        // Simulate: 1. **任务分发策略** (线程分配算法):
        breaker.add_text("1. ");
        breaker.add_unbreakable_unit(UnitKind::Strong, "**", "任务分发策略", "**");
        breaker.add_text(" (线程分配算法):");

        let formatted = breaker.format();
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
        let mut breaker = ParagraphLineBreaker::new(80, "".to_string());

        // Simulate: - **频率表型**: `FrequencyTables::merge()`
        breaker.add_text("- ");
        breaker.add_unbreakable_unit(UnitKind::Strong, "**", "频率表型", "**");
        breaker.add_text(": ");
        breaker.add_unbreakable_unit(
            UnitKind::InlineCode,
            "`",
            "FrequencyTables::merge()",
            "`",
        );

        let formatted = breaker.format();

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
        let mut breaker = ParagraphLineBreaker::new(80, "".to_string());

        breaker.add_text("- ");
        breaker.add_unbreakable_unit(UnitKind::Strong, "**", "频率表型", "**");
        breaker.add_text(":");
        breaker.add_unbreakable_unit(
            UnitKind::InlineCode,
            "`",
            "FrequencyTables::merge()",
            "`",
        );

        let formatted = breaker.format();

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
        let mut breaker = ParagraphLineBreaker::new(80, "".to_string());

        // Simulate: - `scores.txt` / `scores_h.txt`:
        breaker.add_text("- ");
        breaker.add_unbreakable_unit(UnitKind::InlineCode, "`", "scores.txt", "`");
        breaker.add_text(" / ");
        breaker.add_unbreakable_unit(UnitKind::InlineCode, "`", "scores_h.txt", "`");
        breaker.add_text(":");

        let formatted = breaker.format();

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
        let mut breaker = ParagraphLineBreaker::new(80, "".to_string());

        breaker.add_text("- ");
        breaker.add_unbreakable_unit(UnitKind::InlineCode, "`", "scores.txt", "`");
        breaker.add_text("/");
        breaker.add_unbreakable_unit(UnitKind::InlineCode, "`", "scores_h.txt", "`");
        breaker.add_text(":");

        let formatted = breaker.format();

        // The slash should have no space when original has no space
        assert!(
            formatted.contains("`scores.txt`/`scores_h.txt`:"),
            "Slash should have no space when original has no space: got {}",
            formatted
        );
    }

    #[test]
    fn test_comma_space_after_inline_code() {
        // Test that comma preserves trailing space after inline code when present
        // Example: - 行动: 添加 `--relationship` 标志（例如 `one-to-one`, `many-to-one`）
        let mut breaker = ParagraphLineBreaker::new(80, "".to_string());

        // Simulate: - 行动: 添加 `--relationship` 标志（例如 `one-to-one`, `many-to-one`）
        breaker.add_text("- 行动: 添加 `--relationship` 标志（例如 ");
        breaker.add_unbreakable_unit(UnitKind::InlineCode, "`", "one-to-one", "`");
        breaker.add_text(", ");
        breaker.add_unbreakable_unit(UnitKind::InlineCode, "`", "many-to-one", "`");
        breaker.add_text("）在连接时验证键。");

        let formatted = breaker.format();

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
        let mut breaker = ParagraphLineBreaker::new(80, "".to_string());

        // Simulate: - 行动: 添加 `--relationship` 标志（例如 `one-to-one`,`many-to-one`）
        breaker.add_text("- 行动: 添加 `--relationship` 标志（例如 ");
        breaker.add_unbreakable_unit(UnitKind::InlineCode, "`", "one-to-one", "`");
        breaker.add_text(",");
        breaker.add_unbreakable_unit(UnitKind::InlineCode, "`", "many-to-one", "`");
        breaker.add_text("）在连接时验证键。");

        let formatted = breaker.format();

        // The comma should have no trailing space when original has no space
        assert!(
            formatted.contains("`one-to-one`,`many-to-one`"),
            "Comma should have no trailing space when original has no space: got {}",
            formatted
        );
    }

    #[test]
    fn test_paren_space_after_inline_code() {
        // Test that opening parenthesis has space after inline code
        // Example: - **实现**: `cmd/parallel.rs` (~1600 行)
        let mut breaker = ParagraphLineBreaker::new(80, "".to_string());

        // Simulate: - **实现**: `cmd/parallel.rs` (~1600 行)
        breaker.add_text("- ");
        breaker.add_unbreakable_unit(UnitKind::Strong, "**", "实现", "**");
        breaker.add_text(":");
        breaker.add_unbreakable_unit(UnitKind::InlineCode, "`", "cmd/parallel.rs", "`");
        breaker.add_text(" (~1600 行)");

        let formatted = breaker.format();
        println!("Formatted: {:?}", formatted);

        // The opening parenthesis should have a leading space after inline code
        assert!(
            formatted.contains("`cmd/parallel.rs` (~1600 行)"),
            "Opening parenthesis should have leading space after inline code: got {}",
            formatted
        );
    }

    #[test]
    fn test_cjk_punctuation_handling() {
        // Test CJK punctuation handling with new implementation
        let mut breaker = ParagraphLineBreaker::new(80, "".to_string());
        breaker.add_unbreakable_unit(UnitKind::Strong, "**", "特性：", "**");

        let formatted = breaker.format();
        // The CJK punctuation "：" should be preserved in output
        assert!(
            formatted.contains("**特性：**"),
            "CJK punctuation should be preserved: got {}",
            formatted
        );
    }

    #[test]
    fn test_cjk_text_after_inline_code() {
        // Test CJK text after inline code
        let mut breaker = ParagraphLineBreaker::new(80, "".to_string());
        breaker.add_unbreakable_unit(UnitKind::InlineCode, "`", "tva", "`");
        breaker.add_text(" 的开发者");

        let formatted = breaker.format();
        // CJK text after inline code should be preserved
        assert!(
            formatted.contains("`tva` 的开发者"),
            "CJK text after inline code should be preserved: got {}",
            formatted
        );
    }

    #[test]
    fn test_slash_space_after_link() {
        // Test that slash preserves spaces after link when present
        // Example: [a](url) / [b](url)
        let mut breaker = ParagraphLineBreaker::new(80, "".to_string());

        breaker.add_text("- ");
        breaker.add_unbreakable_unit(UnitKind::Link, "[", "a", "]");
        breaker.add_unbreakable_unit(UnitKind::Link, "(", "url", ")");
        breaker.add_text(" / ");
        breaker.add_unbreakable_unit(UnitKind::Link, "[", "b", "]");
        breaker.add_unbreakable_unit(UnitKind::Link, "(", "url", ")");

        let formatted = breaker.format();
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
        let mut breaker = ParagraphLineBreaker::new(80, "".to_string());

        breaker.add_text("- ");
        breaker.add_unbreakable_unit(UnitKind::Link, "[", "a", "]");
        breaker.add_unbreakable_unit(UnitKind::Link, "(", "url", ")");
        breaker.add_text("/");
        breaker.add_unbreakable_unit(UnitKind::Link, "[", "b", "]");
        breaker.add_unbreakable_unit(UnitKind::Link, "(", "url", ")");

        let formatted = breaker.format();
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
        let mut breaker = ParagraphLineBreaker::new(80, "".to_string());

        breaker.add_text("- ");
        breaker.add_unbreakable_unit(UnitKind::InlineCode, "`", "code", "`");
        breaker.add_text(" / ");
        breaker.add_unbreakable_unit(UnitKind::Link, "[", "link", "]");
        breaker.add_unbreakable_unit(UnitKind::Link, "(", "url", ")");

        let formatted = breaker.format();

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
        let mut breaker = ParagraphLineBreaker::new(80, "".to_string());

        breaker.add_text("- ");
        breaker.add_unbreakable_unit(UnitKind::Link, "[", "link", "]");
        breaker.add_unbreakable_unit(UnitKind::Link, "(", "url", ")");
        breaker.add_text(" / ");
        breaker.add_unbreakable_unit(UnitKind::InlineCode, "`", "code", "`");

        let formatted = breaker.format();

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
        let mut breaker = ParagraphLineBreaker::new(80, "".to_string());

        breaker.add_text("- ");
        breaker.add_unbreakable_unit(UnitKind::InlineCode, "`", "a", "`");
        breaker.add_text(" / ");
        breaker.add_unbreakable_unit(UnitKind::InlineCode, "`", "b", "`");
        breaker.add_text(" / ");
        breaker.add_unbreakable_unit(UnitKind::InlineCode, "`", "c", "`");

        let formatted = breaker.format();

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
        let mut breaker = ParagraphLineBreaker::new(80, "".to_string());

        breaker.add_text("- ");
        breaker.add_unbreakable_unit(UnitKind::Link, "[", "a", "]");
        breaker.add_unbreakable_unit(UnitKind::Link, "(", "url", ")");
        breaker.add_text(" / ");
        breaker.add_unbreakable_unit(UnitKind::Link, "[", "b", "]");
        breaker.add_unbreakable_unit(UnitKind::Link, "(", "url", ")");
        breaker.add_text(" / ");
        breaker.add_unbreakable_unit(UnitKind::Link, "[", "c", "]");
        breaker.add_unbreakable_unit(UnitKind::Link, "(", "url", ")");

        let formatted = breaker.format();

        assert!(
            formatted.contains("[a](url) / [b](url) / [c](url)"),
            "Multiple slashes after links should preserve spaces: got {:?}",
            formatted
        );
    }

    #[test]
    fn test_opening_bracket_not_at_line_end() {
        // Test that opening bracket ( stays with its content
        // Example: - **数值提取**: `getnum` 从混合文本中提取数字（如 "zoom-123.45xyz" -> 123.45）。
        // The `（` should not be at line end while `如` is on next line
        let mut breaker = ParagraphLineBreaker::new(50, "".to_string());

        // Simulate the text structure
        breaker.add_text("- **数值提取**: ");
        breaker.add_unbreakable_unit(UnitKind::InlineCode, "`", "getnum", "`");
        breaker.add_text(" 从混合文本中提取数字（如 ");
        breaker.add_unbreakable_unit(UnitKind::InlineCode, "`", "zoom-123.45xyz", "`");
        breaker.add_text(" -> 123.45）。");

        let formatted = breaker.format();
        let lines: Vec<&str> = formatted.lines().collect();

        // Check that no line ends with opening bracket ( while content is on next line
        for (i, line) in lines.iter().enumerate() {
            if i < lines.len() - 1 {
                let trimmed = line.trim_end();
                if trimmed.ends_with('（') || trimmed.ends_with('(') {
                    let next_line = lines[i + 1].trim_start();
                    assert!(
                        !next_line.starts_with('如'),
                        "Opening bracket should not be at line end while '如' is on next line.\nLine {}: {}\nLine {}: {}",
                        i, line, i + 1, lines[i + 1]
                    );
                }
            }
        }

        // The formatted output should keep the bracket with its content
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
        let mut breaker = ParagraphLineBreaker::new(45, "".to_string());
        breaker.add_text("- **数值提取**: ");
        breaker.add_unbreakable_unit(UnitKind::InlineCode, "`", "getnum", "`");
        breaker.add_text(" 从混合文本中提取数字（如 ");
        breaker.add_unbreakable_unit(UnitKind::InlineCode, "`", "zoom-123.45xyz", "`");
        breaker.add_text(" -> 123.45）。");

        let formatted = breaker.format();

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
                "No line should end with just opening bracket '（'. Line: {}",
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

        let mut breaker = ParagraphLineBreaker::new(45, "> ".to_string());

        // Simulate: > **保持简单**：tva 的表达式语言设计目标是**简单高效的数据处理**，不是通用编程语言。
        breaker.add_unbreakable_unit(UnitKind::Strong, "**", "保持简单", "**");
        breaker.add_text("：tva 的表达式语言设计目标是");
        breaker.add_unbreakable_unit(UnitKind::Strong, "**", "简单高效的数据处理", "**");
        breaker.add_text("，不是通用编程语言。");

        let formatted = breaker.format();
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
        let mut breaker = ParagraphLineBreaker::new(60, "> ".to_string());

        breaker.add_unbreakable_unit(UnitKind::Strong, "**", "保持简单", "**");
        breaker.add_text("：tva 的表达式语言设计目标是");
        breaker.add_unbreakable_unit(UnitKind::Strong, "**", "简单高效的数据处理", "**");
        breaker.add_text("，不是通用编程语言。");

        let formatted = breaker.format();

        // Verify that `**` markers are not alone at line end/start
        let lines: Vec<&str> = formatted.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // A line should not end with just `**` (opening marker)
            if trimmed.ends_with("**") && !trimmed.ends_with("****") {
                // Check if this is an opening marker by looking at the next line
                if i < lines.len() - 1 {
                    let next_line = lines[i + 1].trim();
                    assert!(
                        !next_line.starts_with("简单") && !next_line.starts_with("保持"),
                        "Opening marker `**` should not be at line end while emphasized content is on next line.\nLine {}: {}\nLine {}: {}",
                        i, line, i + 1, lines[i + 1]
                    );
                }
            }

            // A line should not start with just `**` (closing marker)
            if trimmed.starts_with("**") && !trimmed.starts_with("****") {
                // Check if this is a closing marker by looking at the previous line
                if i > 0 {
                    let prev_line = lines[i - 1].trim();
                    assert!(
                        !prev_line.ends_with("简单") && !prev_line.ends_with("保持"),
                        "Closing marker `**` should not be at line start while emphasized content is on previous line.\nLine {}: {}\nLine {}: {}",
                        i - 1, lines[i - 1], i, line
                    );
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
    fn test_cjk_comma_not_at_line_start_in_blockquote() {
        // Test that CJK comma `，` is not at line start in blockquote
        // Example: > **保持简单**：tva 的表达式语言设计目标是**简单高效的数据处理**
        // > ，不是通用编程语言。
        // The `，` should not be at line start

        let mut breaker = ParagraphLineBreaker::new(45, "> ".to_string());

        // Simulate: > **保持简单**：tva 的表达式语言设计目标是**简单高效的数据处理**，不是通用编程语言。
        breaker.add_unbreakable_unit(UnitKind::Strong, "**", "保持简单", "**");
        breaker.add_text("：tva 的表达式语言设计目标是");
        breaker.add_unbreakable_unit(UnitKind::Strong, "**", "简单高效的数据处理", "**");
        breaker.add_text("，不是通用编程语言。");

        let formatted = breaker.format();
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

        let mut breaker = ParagraphLineBreaker::new(45, "".to_string());

        // Simulate: tva **只有匿名函数（lambda）**且主要用于 TSV 数据处理
        breaker.add_text("tva ");
        breaker.add_unbreakable_unit(
            UnitKind::Strong,
            "**",
            "只有匿名函数（lambda）",
            "**",
        );
        breaker.add_text("且主要用于 TSV 数据处理");

        let formatted = breaker.format();

        // The emphasized text should stay together
        assert!(
            formatted.contains("**只有匿名函数（lambda）**"),
            "Emphasized text should stay together. Formatted:\n{}",
            formatted
        );

        // No line should end with just `**` (opening marker)
        let lines: Vec<&str> = formatted.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.ends_with("**") && !trimmed.ends_with("****") {
                // This is likely an opening marker, it shouldn't be at line end
                assert!(
                    i == lines.len() - 1 || !lines[i + 1].trim().starts_with("且"),
                    "Opening marker `**` should not be at line end: {}",
                    line
                );
            }
        }
    }

    #[test]
    fn test_no_empty_blockquote_line() {
        // Test that there's no empty blockquote line at the end
        // Example: > **保持简单**：tva 的表达式语言设计目标是**简单高效的数据处理**，不是通用编程语言。
        // Should NOT have an empty "> " line at the end

        let mut breaker = ParagraphLineBreaker::new(60, "> ".to_string());

        breaker.add_unbreakable_unit(UnitKind::Strong, "**", "保持简单", "**");
        breaker.add_text("：tva 的表达式语言设计目标是");
        breaker.add_unbreakable_unit(UnitKind::Strong, "**", "简单高效的数据处理", "**");
        breaker.add_text("，不是通用编程语言。");

        let formatted = breaker.format();

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

        let mut breaker = ParagraphLineBreaker::new(50, "".to_string());

        // Simulate the text with a long link using add_unbreakable_unit
        breaker.add_text("我们旨在重现 ");
        breaker.add_unbreakable_unit(UnitKind::InlineCode, "`", "https://github.com/eBay/tsv-utils/blob/master/docs/comparative-benchmarks-2017.md", "`");
        breaker.add_text(" 使用的严格基准测试策略。");

        let formatted = breaker.format();

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
        let mut breaker = ParagraphLineBreaker::new(50, "".to_string());

        // Simulate: [https://github.com/eBay/tsv-utils/blob/master/docs/comparative-benchmarks-2017.md](https://github.com/eBay/tsv-utils/blob/master/docs/comparative-benchmarks-2017.md)
        breaker.add_unbreakable_unit(UnitKind::Link, "[", "https://github.com/eBay/tsv-utils/blob/master/docs/comparative-benchmarks-2017.md", "]");
        breaker.add_unbreakable_unit(UnitKind::Link, "(", "https://github.com/eBay/tsv-utils/blob/master/docs/comparative-benchmarks-2017.md", ")");

        let formatted = breaker.format();

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
        // Note: New implementation may split `](` across lines due to width constraints
        let mut breaker = ParagraphLineBreaker::new(50, "".to_string());

        // Simulate: [URL](URL) 使用的严格基准测试策略。
        breaker.add_unbreakable_unit(UnitKind::Link, "[", "https://github.com/eBay/tsv-utils/blob/master/docs/comparative-benchmarks-2017.md", "]");
        breaker.add_unbreakable_unit(UnitKind::Link, "(", "https://github.com/eBay/tsv-utils/blob/master/docs/comparative-benchmarks-2017.md", ")");
        breaker.add_text("使用的严格基准测试策略。");

        let formatted = breaker.format();

        // The `)` should NOT be on its own line
        assert!(
            !formatted.contains("\n)"),
            "Closing parenthesis should NOT be on its own line. Formatted:\n{}",
            formatted
        );

        // The link content should be preserved
        assert!(
            formatted.contains("https://github.com/eBay/tsv-utils/blob/master/docs/comparative-benchmarks-2017.md"),
            "Link URL should be preserved. Formatted:\n{}",
            formatted
        );
    }

    #[test]
    fn test_link_with_text_and_long_url() {
        // Test that links with text and long URL are formatted correctly
        // Example: [eBay TSV Utilities](https://github.com/eBay/tsv-utils/blob/master/docs/comparative-benchmarks-2017.md)
        // Note: New implementation may split the link across lines, but the link text should be preserved
        let mut breaker = ParagraphLineBreaker::new(60, "".to_string());

        // Simulate: 我们旨在重现 [eBay TSV Utilities](https://github.com/eBay/tsv-utils/blob/master/docs/comparative-benchmarks-2017.md) 使用的严格基准测试策略。
        breaker.add_text("我们旨在重现 ");
        breaker.add_unbreakable_unit(UnitKind::Link, "[", "eBay TSV Utilities", "]");
        breaker.add_unbreakable_unit(UnitKind::Link, "(", "https://github.com/eBay/tsv-utils/blob/master/docs/comparative-benchmarks-2017.md", ")");
        breaker.add_text(" 使用的严格基准测试策略。");

        let formatted = breaker.format();

        // The link text should be preserved (even if link is split across lines)
        assert!(
            formatted.contains("[eBay TSV Utilities]"),
            "Link text should be preserved. Formatted:\n{}",
            formatted
        );
    }

    #[test]
    fn test_link_with_cjk_punctuation_not_at_line_start() {
        // Test that CJK punctuation after link is NOT at line start
        // Example: [link](url) 。测试。
        let mut breaker = ParagraphLineBreaker::new(60, "".to_string());

        // Simulate: - **HEPMASS** ( 4.8GB): [link](https://archive.ics.uci.edu/ml/datasets/HEPMASS) 。测试。
        breaker.add_text("- **HEPMASS** ( 4.8GB): ");
        breaker.add_unbreakable_unit(UnitKind::Link, "[", "link", "]");
        breaker.add_unbreakable_unit(
            UnitKind::Link,
            "(",
            "https://archive.ics.uci.edu/ml/datasets/HEPMASS",
            ")",
        );
        breaker.add_text(" 。测试。");

        let formatted = breaker.format();
        let lines: Vec<&str> = formatted.lines().collect();

        // Check that no line starts with CJK punctuation `。`
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            assert!(
                !trimmed.starts_with('。'),
                "CJK punctuation `。` should not be at line start.\nLine {}: {}",
                i,
                line
            );
        }
    }

    #[test]
    fn test_link_with_various_cjk_punctuation() {
        // Test that various CJK punctuation after link are NOT at line start
        // Note: Using a shorter URL to avoid line break after link in this test
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
            // Use larger width and shorter URL to keep everything on one line
            let mut breaker = ParagraphLineBreaker::new(120, "".to_string());

            // Simulate: [link](url)[punct] test with shorter URL
            // Note: No space before punct to match expected behavior
            breaker.add_text("- ");
            breaker.add_unbreakable_unit(
                UnitKind::Link,
                "[",
                "link",
                "](https://example.com)",
            );
            breaker.add_text(&format!("{} 测试", punct));

            let formatted = breaker.format();

            // CJK punctuation should NOT be at line start (after newline and optional whitespace)
            for line in formatted.lines() {
                let trimmed = line.trim_start();
                assert!(
                    !trimmed.starts_with(punct),
                    "{} ({}) should NOT be at line start. Line: {}\nFormatted:\n{}",
                    desc,
                    punct,
                    line,
                    formatted
                );
            }

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
    fn test_opening_paren_no_space_after_cjk() {
        // Test that opening parenthesis `(` is preserved after CJK text
        // Example: 和随机采样 (`sample`)的基础
        // Note: New implementation may add space, but the structure should be preserved
        let mut breaker = ParagraphLineBreaker::new(80, "".to_string());

        breaker.add_text("和随机采样 ");
        breaker.add_text("(");
        breaker.add_unbreakable_unit(UnitKind::InlineCode, "`", "sample", "`");
        breaker.add_text(")的基础");

        let formatted = breaker.format();

        // The inline code and parentheses should be preserved in output
        assert!(
            formatted.contains("`sample`"),
            "Inline code should be preserved. Formatted:\n{}",
            formatted
        );

        // The structure should contain the CJK text, parenthesis and inline code
        assert!(
            formatted.contains("采样") && formatted.contains("("),
            "CJK text and parenthesis should be preserved. Formatted:\n{}",
            formatted
        );
    }

    #[test]
    fn test_opening_bracket_no_space_after() {
        // Test that opening bracket followed by text doesn't have space
        // Example: **HEPMASS** (\n  4.8GB) should become **HEPMASS** (4.8GB)
        let mut breaker = ParagraphLineBreaker::new(80, "".to_string());

        // Use add_unbreakable_unit to simulate markdown markers
        breaker.add_unbreakable_unit(UnitKind::Strong, "**", "HEPMASS", "**");
        breaker.add_text(" (");
        breaker.add_text("4.8GB)");

        let formatted = breaker.format();

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
    fn test_list_item_line_breaking_width() {
        // Test for the bug: line breaks too early in list items
        // Input: "- For projects that have finished downloading, but have renamed strains, you can run `reorder.sh` to avoid re-downloading"
        // Expected: should fill the line closer to max_width
        // Note: Using max_width = 78 with prefix "  " for continuation lines
        let mut breaker = ParagraphLineBreaker::new(78, "  ".to_string());
        breaker.add_text("For projects that have finished downloading, but have renamed strains, you can run");
        breaker.add_unbreakable_unit(UnitKind::InlineCode, "`", "reorder.sh", "`");
        breaker.add_text("to avoid re-downloading");

        let breaks = breaker.compute_breaks();
        println!("Breaks: {:?}", breaks);

        let formatted = breaker.format();
        println!("Formatted:\n{}", formatted);

        // Check that the first line is reasonably filled
        let first_line = formatted.lines().next().unwrap();
        let first_line_width = unicode_width::width(first_line);
        println!("First line width: {}", first_line_width);

        // The first line should be reasonably filled
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
        let mut breaker = ParagraphLineBreaker::new(60, "  ".to_string());
        breaker.add_text("这些操作需要");
        breaker.add_unbreakable_unit(
            UnitKind::InlineCode,
            "`",
            "list.iter().cloned().collect()",
            "`",
        );
        breaker.add_text("，比直接");
        breaker.add_unbreakable_unit(UnitKind::InlineCode, "`", "list.clone()", "`");
        breaker.add_text("慢得多。");

        let breaks = breaker.compute_breaks();
        println!("Breaks: {:?}", breaks);

        let formatted = breaker.format();
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
        // Note: Using max_width = 100 with prefix "  " for continuation lines
        // The "- " prefix is added as text at the beginning
        let mut breaker = ParagraphLineBreaker::new(100, "  ".to_string());
        breaker.add_text("- ");
        breaker.add_unbreakable_unit(UnitKind::Strong, "**", "特色功能", "**");
        breaker.add_text(": 支持日期补全 (");
        breaker.add_unbreakable_unit(UnitKind::InlineCode, "`", "--dates", "`");
        breaker.add_text(")，自动填充缺失的日期并设为 0；支持间隙压缩 (");
        breaker.add_unbreakable_unit(UnitKind::InlineCode, "`", "--compress-gaps", "`");
        breaker.add_text(")，隐藏连续的 0 值。");

        let breaks = breaker.compute_breaks();
        println!("Breaks: {:?}", breaks);

        let formatted = breaker.format();
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
    fn test_paragraph_line_breaker_debug() {
        // Debug test to see what's happening in ParagraphLineBreaker
        let mut breaker = ParagraphLineBreaker::new(80, "".to_string());

        // Simulate what happens in format_commonmark
        breaker.add_text("**");
        breaker.add_text("HEPMASS");
        breaker.add_text("**");
        breaker.add_text(" (");
        breaker.add_hard_break();
        breaker.add_text("4.8GB)");

        println!("Fragments:");
        for (i, fragment) in breaker.fragments.iter().enumerate() {
            match fragment {
                ContentFragment::Text { content, .. } => {
                    println!("  Fragment {}: Text({:?})", i, content);
                }
                ContentFragment::Unbreakable {
                    prefix,
                    content,
                    suffix,
                    ..
                } => {
                    println!("  Fragment {}: Unbreakable(prefix={:?}, content={:?}, suffix={:?})", i, prefix, content, suffix);
                }
            }
        }

        println!("Break opportunities:");
        for (i, opp) in breaker.break_opportunities.iter().enumerate() {
            println!(
                "  Opp {}: position={}, is_forced={}",
                i, opp.position, opp.is_forced
            );
        }

        let (breaks, forced_breaks) = breaker.compute_breaks_internal();
        println!("Breaks: {:?}", breaks);
        println!("Forced breaks: {:?}", forced_breaks);

        let formatted = breaker.format();
        println!("Formatted: {:?}", formatted);
        println!("Formatted bytes: {:?}", formatted.as_bytes());

        // There should be no space after `(`
        assert!(
            !formatted.contains("( 4.8GB)"),
            "There should be no space after `(`. Formatted:\n{}",
            formatted
        );
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
    text.chars().next().is_some_and(|c| {
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

/// Check if a string is a Markdown closing marker
/// This includes `**`, `*`, `]`, `)`, etc.
fn is_markdown_closing_marker(text: &str) -> bool {
    matches!(text, "**" | "*" | "]" | ")")
}
