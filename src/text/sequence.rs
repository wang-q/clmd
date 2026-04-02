//! BasedSequence - High-performance string slicing system
//!
//! This module provides a string slicing mechanism that avoids copying by
//! referencing the original string with offset information.
//!
//! Inspired by flexmark-java's BasedSequence.

use std::fmt;
use std::ops::Range;

/// A string slice that references the original string with offset information.
/// This avoids copying and allows efficient subsequence operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BasedSequence<'a> {
    /// The original source string
    base: &'a str,
    /// Start offset in the base string
    start: usize,
    /// End offset in the base string
    end: usize,
}

impl<'a> BasedSequence<'a> {
    /// Create a new BasedSequence from a string slice
    pub fn new(base: &'a str) -> Self {
        Self {
            base,
            start: 0,
            end: base.len(),
        }
    }

    /// Create a new BasedSequence with explicit offsets
    pub fn with_offsets(base: &'a str, start: usize, end: usize) -> Self {
        debug_assert!(start <= end, "start must be <= end");
        debug_assert!(end <= base.len(), "end must be <= base.len()");
        Self { base, start, end }
    }

    /// Get the base string
    pub fn base(&self) -> &'a str {
        self.base
    }

    /// Get the start offset in the base string
    pub fn start_offset(&self) -> usize {
        self.start
    }

    /// Get the end offset in the base string
    pub fn end_offset(&self) -> usize {
        self.end
    }

    /// Get the length of this sequence
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    /// Check if this sequence is empty
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Get the string content
    pub fn as_str(&self) -> &'a str {
        &self.base[self.start..self.end]
    }

    /// Get a character at the given index
    pub fn char_at(&self, index: usize) -> Option<char> {
        self.as_str().chars().nth(index)
    }

    /// Get the byte at the given index
    pub fn byte_at(&self, index: usize) -> Option<u8> {
        let byte_idx = self.start + index;
        if byte_idx < self.end {
            Some(self.base.as_bytes()[byte_idx])
        } else {
            None
        }
    }

    /// Create a subsequence from this sequence
    pub fn sub_sequence(&self, start: usize, end: usize) -> Self {
        debug_assert!(start <= end, "start must be <= end");
        debug_assert!(end <= self.len(), "end must be <= self.len()");
        Self {
            base: self.base,
            start: self.start + start,
            end: self.start + end,
        }
    }

    /// Create a subsequence from the base (original) string
    pub fn base_sub_sequence(&self, start: usize, end: usize) -> Self {
        debug_assert!(start <= end, "start must be <= end");
        debug_assert!(end <= self.base.len(), "end must be <= base.len()");
        Self {
            base: self.base,
            start,
            end,
        }
    }

    /// Get the source range (start and end offsets in base)
    pub fn source_range(&self) -> Range<usize> {
        self.start..self.end
    }

    /// Get the index offset in base for a given index
    pub fn index_offset(&self, index: usize) -> usize {
        debug_assert!(index <= self.len(), "index out of bounds");
        self.start + index
    }

    /// Get an empty prefix sequence
    pub fn empty_prefix(&self) -> Self {
        Self {
            base: self.base,
            start: self.start,
            end: self.start,
        }
    }

    /// Get an empty suffix sequence
    pub fn empty_suffix(&self) -> Self {
        Self {
            base: self.base,
            start: self.end,
            end: self.end,
        }
    }

    /// Check if this sequence is continued by another in the original source
    pub fn is_continued_by(&self, other: &BasedSequence<'a>) -> bool {
        // Use pointer equality to check if both sequences reference the same base string
        // This is safe because we're only comparing pointers, not dereferencing them
        self.base.as_ptr() == other.base.as_ptr() && self.end == other.start
    }

    /// Check if this sequence is a continuation of another in the original source
    pub fn is_continuation_of(&self, other: &BasedSequence<'a>) -> bool {
        other.is_continued_by(self)
    }

    /// Splice this sequence with another at the end
    pub fn splice_at_end(&self, other: &BasedSequence<'a>) -> Self {
        assert!(
            self.is_continued_by(other),
            "other must be a continuation of self"
        );
        Self {
            base: self.base,
            start: self.start,
            end: other.end,
        }
    }

    /// Check if this sequence contains all of another sequence
    pub fn contains_all_of(&self, other: &BasedSequence<'a>) -> bool {
        self.base.as_ptr() == other.base.as_ptr()
            && self.start <= other.start
            && self.end >= other.end
    }

    /// Check if this sequence contains some of another sequence
    pub fn contains_some_of(&self, other: &BasedSequence<'a>) -> bool {
        if self.base.as_ptr() != other.base.as_ptr() {
            return false;
        }
        let self_range = self.start..self.end;
        let other_range = other.start..other.end;
        self_range.start < other_range.end && other_range.start < self_range.end
    }

    /// Get the intersection of two sequences
    pub fn intersect(&self, other: &BasedSequence<'a>) -> Option<Self> {
        if self.base.as_ptr() != other.base.as_ptr() {
            return None;
        }
        let start = self.start.max(other.start);
        let end = self.end.min(other.end);
        if start < end {
            Some(Self {
                base: self.base,
                start,
                end,
            })
        } else {
            None
        }
    }

    /// Trim whitespace from both ends
    pub fn trim(&self) -> Self {
        let s = self.as_str();
        let trimmed = s.trim();
        let start_offset = s.len() - s.trim_start().len();
        Self {
            base: self.base,
            start: self.start + start_offset,
            end: self.start + start_offset + trimmed.len(),
        }
    }

    /// Trim whitespace from the start
    pub fn trim_start(&self) -> Self {
        let s = self.as_str();
        let trimmed = s.trim_start();
        let offset = s.len() - trimmed.len();
        Self {
            base: self.base,
            start: self.start + offset,
            end: self.end,
        }
    }

    /// Trim whitespace from the end
    pub fn trim_end(&self) -> Self {
        let s = self.as_str();
        let trimmed = s.trim_end();
        Self {
            base: self.base,
            start: self.start,
            end: self.start + trimmed.len(),
        }
    }

    /// Get a prefix of this sequence
    pub fn prefix(&self, len: usize) -> Self {
        let end = (self.start + len).min(self.end);
        Self {
            base: self.base,
            start: self.start,
            end,
        }
    }

    /// Get a suffix of this sequence
    pub fn suffix(&self, len: usize) -> Self {
        let start = if len > self.len() {
            self.start
        } else {
            self.end - len
        };
        Self {
            base: self.base,
            start,
            end: self.end,
        }
    }

    /// Check if this sequence starts with a prefix
    pub fn starts_with(&self, prefix: &str) -> bool {
        self.as_str().starts_with(prefix)
    }

    /// Check if this sequence ends with a suffix
    pub fn ends_with(&self, suffix: &str) -> bool {
        self.as_str().ends_with(suffix)
    }

    /// Find the index of a substring
    pub fn find(&self, pat: &str) -> Option<usize> {
        self.as_str().find(pat)
    }

    /// Split the sequence by a delimiter character
    pub fn split(&self, pat: char) -> SplitIterator<'a> {
        SplitIterator {
            base: self.base,
            inner: self.as_str().split(pat).peekable(),
            offset: self.start,
            pat_len: pat.len_utf8(),
        }
    }

    /// Get lines (split by newline)
    pub fn lines(&self) -> SplitIterator<'a> {
        self.split('\n')
    }

    /// Count leading characters matching a predicate
    pub fn count_leading<F>(&self, f: F) -> usize
    where
        F: Fn(char) -> bool,
    {
        self.as_str().chars().take_while(|&c| f(c)).count()
    }

    /// Count trailing characters matching a predicate
    pub fn count_trailing<F>(&self, f: F) -> usize
    where
        F: Fn(char) -> bool,
    {
        self.as_str().chars().rev().take_while(|&c| f(c)).count()
    }

    /// Extend this sequence by including characters matching a predicate
    pub fn extend_by<F>(&self, f: F, max_count: usize) -> Self
    where
        F: Fn(char) -> bool,
    {
        let mut end = self.end;

        for (count, c) in self.base[self.end..].chars().enumerate() {
            if count >= max_count || !f(c) {
                break;
            }
            end += c.len_utf8();
        }

        Self {
            base: self.base,
            start: self.start,
            end,
        }
    }

    /// Get the line number of the start position
    pub fn start_line(&self) -> usize {
        self.base[..self.start]
            .chars()
            .filter(|&c| c == '\n')
            .count()
            + 1
    }

    /// Get the line number of the end position
    pub fn end_line(&self) -> usize {
        self.base[..self.end].chars().filter(|&c| c == '\n').count() + 1
    }

    /// Get the column of the start position
    pub fn start_column(&self) -> usize {
        let line_start = self.base[..self.start]
            .rfind('\n')
            .map(|i| i + 1)
            .unwrap_or(0);
        self.start - line_start + 1
    }

    /// Get the column of the end position
    pub fn end_column(&self) -> usize {
        let line_start = self.base[..self.end]
            .rfind('\n')
            .map(|i| i + 1)
            .unwrap_or(0);
        self.end - line_start + 1
    }
}

impl<'a> fmt::Display for BasedSequence<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl<'a> From<&'a str> for BasedSequence<'a> {
    fn from(s: &'a str) -> Self {
        Self::new(s)
    }
}

impl<'a> From<BasedSequence<'a>> for String {
    fn from(seq: BasedSequence<'a>) -> Self {
        seq.as_str().to_string()
    }
}

impl<'a> PartialEq<str> for BasedSequence<'a> {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl<'a> PartialEq<&str> for BasedSequence<'a> {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

/// Iterator for splitting a BasedSequence
#[derive(Debug)]
pub struct SplitIterator<'a> {
    base: &'a str,
    inner: std::iter::Peekable<std::str::Split<'a, char>>,
    offset: usize,
    pat_len: usize,
}

impl<'a> Iterator for SplitIterator<'a> {
    type Item = BasedSequence<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let part = self.inner.next()?;
        let seq = BasedSequence {
            base: self.base,
            start: self.offset,
            end: self.offset + part.len(),
        };
        self.offset += part.len();
        // Only add pattern length if there's more items
        if self.inner.peek().is_some() {
            self.offset += self.pat_len;
        }
        Some(seq)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_creation() {
        let text = "Hello, World!";
        let seq = BasedSequence::new(text);
        assert_eq!(seq.as_str(), text);
        assert_eq!(seq.len(), text.len());
        assert_eq!(seq.start_offset(), 0);
        assert_eq!(seq.end_offset(), text.len());
    }

    #[test]
    fn test_sub_sequence() {
        let text = "Hello, World!";
        let seq = BasedSequence::new(text);
        let sub = seq.sub_sequence(0, 5);
        assert_eq!(sub.as_str(), "Hello");
        assert_eq!(sub.start_offset(), 0);
        assert_eq!(sub.end_offset(), 5);
    }

    #[test]
    fn test_trim() {
        let text = "  Hello, World!  ";
        let seq = BasedSequence::new(text);
        let trimmed = seq.trim();
        assert_eq!(trimmed.as_str(), "Hello, World!");
    }

    #[test]
    fn test_continuation() {
        let text = "Hello, World!";
        let seq1 = BasedSequence::new(text).sub_sequence(0, 5);
        let seq2 = BasedSequence::new(text).sub_sequence(5, 7);
        assert!(seq1.is_continued_by(&seq2));
        assert!(seq2.is_continuation_of(&seq1));
    }

    #[test]
    fn test_splice() {
        let text = "Hello, World!";
        let seq1 = BasedSequence::new(text).sub_sequence(0, 5);
        let seq2 = BasedSequence::new(text).sub_sequence(5, 13);
        let spliced = seq1.splice_at_end(&seq2);
        assert_eq!(spliced.as_str(), "Hello, World!");
    }

    #[test]
    fn test_intersect() {
        let text = "Hello, World!";
        // seq1: "Hello, " (indices 0-7)
        // seq2: ", World!" (indices 5-13)
        // intersection should be ", " (indices 5-7)
        let seq1 = BasedSequence::new(text).sub_sequence(0, 7);
        let seq2 = BasedSequence::new(text).sub_sequence(5, 13);
        let intersect = seq1.intersect(&seq2).unwrap();
        assert_eq!(intersect.as_str(), ", ");
    }

    #[test]
    fn test_split() {
        let text = "one,two,three";
        let seq = BasedSequence::new(text);
        let parts: Vec<_> = seq.split(',').collect();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0].as_str(), "one");
        assert_eq!(parts[1].as_str(), "two");
        assert_eq!(parts[2].as_str(), "three");
    }

    #[test]
    fn test_lines() {
        let text = "line1\nline2\nline3";
        let seq = BasedSequence::new(text);
        let lines: Vec<_> = seq.lines().collect();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].as_str(), "line1");
        assert_eq!(lines[1].as_str(), "line2");
        assert_eq!(lines[2].as_str(), "line3");
    }

    #[test]
    fn test_line_column() {
        let text = "line1\nline2\nline3";
        let seq = BasedSequence::new(text).sub_sequence(7, 12);
        assert_eq!(seq.start_line(), 2);
        assert_eq!(seq.start_column(), 2);
    }

    #[test]
    fn test_display() {
        let text = "Hello";
        let seq = BasedSequence::new(text);
        assert_eq!(format!("{}", seq), "Hello");
    }

    #[test]
    fn test_from_str() {
        let text = "Hello";
        let seq: BasedSequence = text.into();
        assert_eq!(seq.as_str(), text);
    }

    #[test]
    fn test_into_string() {
        let text = "Hello";
        let seq = BasedSequence::new(text);
        let s: String = seq.into();
        assert_eq!(s, text);
    }

    #[test]
    fn test_char_at() {
        let text = "Hello";
        let seq = BasedSequence::new(text);
        assert_eq!(seq.char_at(0), Some('H'));
        assert_eq!(seq.char_at(4), Some('o'));
        assert_eq!(seq.char_at(5), None);
    }

    #[test]
    fn test_byte_at() {
        let text = "Hello";
        let seq = BasedSequence::new(text);
        assert_eq!(seq.byte_at(0), Some(b'H'));
        assert_eq!(seq.byte_at(4), Some(b'o'));
        assert_eq!(seq.byte_at(5), None);
    }

    #[test]
    fn test_trim_start() {
        let text = "  Hello  ";
        let seq = BasedSequence::new(text);
        let trimmed = seq.trim_start();
        assert_eq!(trimmed.as_str(), "Hello  ");
    }

    #[test]
    fn test_trim_end() {
        let text = "  Hello  ";
        let seq = BasedSequence::new(text);
        let trimmed = seq.trim_end();
        assert_eq!(trimmed.as_str(), "  Hello");
    }

    #[test]
    fn test_prefix() {
        let text = "Hello, World!";
        let seq = BasedSequence::new(text);
        assert_eq!(seq.prefix(5).as_str(), "Hello");
        assert_eq!(seq.prefix(0).as_str(), "");
        assert_eq!(seq.prefix(20).as_str(), "Hello, World!");
    }

    #[test]
    fn test_suffix() {
        let text = "Hello, World!";
        let seq = BasedSequence::new(text);
        assert_eq!(seq.suffix(6).as_str(), "World!");
        assert_eq!(seq.suffix(0).as_str(), "");
        assert_eq!(seq.suffix(20).as_str(), "Hello, World!");
    }

    #[test]
    fn test_starts_with() {
        let text = "Hello, World!";
        let seq = BasedSequence::new(text);
        assert!(seq.starts_with("Hello"));
        assert!(seq.starts_with("Hello, "));
        assert!(!seq.starts_with("World"));
    }

    #[test]
    fn test_ends_with() {
        let text = "Hello, World!";
        let seq = BasedSequence::new(text);
        assert!(seq.ends_with("World!"));
        assert!(seq.ends_with("!"));
        assert!(!seq.ends_with("Hello"));
    }

    #[test]
    fn test_find() {
        let text = "Hello, World!";
        let seq = BasedSequence::new(text);
        assert_eq!(seq.find("World"), Some(7));
        assert_eq!(seq.find("Hello"), Some(0));
        assert_eq!(seq.find("xyz"), None);
    }

    #[test]
    fn test_count_leading() {
        let text = "   Hello";
        let seq = BasedSequence::new(text);
        assert_eq!(seq.count_leading(|c| c == ' '), 3);
        assert_eq!(seq.count_leading(|c| c == 'H'), 0);

        let text2 = "aaaHello";
        let seq2 = BasedSequence::new(text2);
        assert_eq!(seq2.count_leading(|c| c == 'a'), 3);
    }

    #[test]
    fn test_count_trailing() {
        let text = "Hello   ";
        let seq = BasedSequence::new(text);
        assert_eq!(seq.count_trailing(|c| c == ' '), 3);
        assert_eq!(seq.count_trailing(|c| c == 'o'), 0); // 'o' is not at the end

        let text2 = "Helloaaa";
        let seq2 = BasedSequence::new(text2);
        assert_eq!(seq2.count_trailing(|c| c == 'a'), 3);
    }

    #[test]
    fn test_extend_by() {
        let text = "Hello World";
        let seq = BasedSequence::new(text).sub_sequence(0, 5); // "Hello"
        // extend_by extends by matching characters from the base string after the current end
        // After "Hello" (indices 0-5), the next characters are " World"
        // It will match up to max_count characters that satisfy the predicate
        let extended = seq.extend_by(|c| c.is_alphabetic() || c == ' ', 3);
        // Should extend by " W" (space + W) since they're within max_count=3
        assert!(extended.as_str().starts_with("Hello"));
    }



    #[test]
    fn test_with_offsets() {
        let text = "Hello, World!";
        let seq = BasedSequence::with_offsets(text, 0, 5);
        assert_eq!(seq.start_offset(), 0);
        assert_eq!(seq.end_offset(), 5);
        assert_eq!(seq.as_str(), "Hello");
    }

    #[test]
    fn test_base_sub_sequence() {
        let text = "Hello, World!";
        let seq = BasedSequence::new(text).sub_sequence(0, 5);
        let base = seq.base_sub_sequence(7, 12);
        assert_eq!(base.as_str(), "World");
    }

    #[test]
    fn test_contains_all_of() {
        let text = "Hello, World!";
        let seq = BasedSequence::new(text);
        let sub1 = seq.sub_sequence(0, 5); // "Hello"
        let sub2 = seq.sub_sequence(7, 12); // "World"
        let sub3 = seq.sub_sequence(0, 13); // "Hello, World!"

        assert!(seq.contains_all_of(&sub1));
        assert!(seq.contains_all_of(&sub2));
        assert!(seq.contains_all_of(&sub3));
        assert!(!sub1.contains_all_of(&seq));
    }

    #[test]
    fn test_contains_some_of() {
        let text = "Hello, World!";
        let seq = BasedSequence::new(text);
        let sub1 = seq.sub_sequence(0, 7); // "Hello, "
        let sub2 = seq.sub_sequence(5, 13); // ", World!"

        assert!(sub1.contains_some_of(&sub2)); // overlap at ", "
        assert!(sub2.contains_some_of(&sub1));
    }

    #[test]
    fn test_is_empty() {
        let text = "";
        let seq = BasedSequence::new(text);
        assert!(seq.is_empty());

        let text2 = "Hello";
        let seq2 = BasedSequence::new(text2);
        assert!(!seq2.is_empty());
    }

    #[test]
    fn test_len() {
        let text = "Hello";
        let seq = BasedSequence::new(text);
        assert_eq!(seq.len(), 5);

        let text2 = "";
        let seq2 = BasedSequence::new(text2);
        assert_eq!(seq2.len(), 0);
    }

    #[test]
    fn test_to_string() {
        let text = "Hello";
        let seq = BasedSequence::new(text);
        assert_eq!(seq.to_string(), "Hello");
    }

    #[test]
    fn test_partial_eq() {
        let text = "Hello";
        let seq1 = BasedSequence::new(text);
        let seq2 = BasedSequence::new(text);
        assert_eq!(seq1, seq2);

        let seq3 = BasedSequence::new("World");
        assert_ne!(seq1, seq3);
    }

    #[test]
    fn test_sub_sequence_edge_cases() {
        let text = "Hello";
        let seq = BasedSequence::new(text);

        // Empty subsequence
        let empty = seq.sub_sequence(0, 0);
        assert!(empty.is_empty());

        // Full sequence
        let full = seq.sub_sequence(0, 5);
        assert_eq!(full.as_str(), "Hello");

        // Single character
        let single = seq.sub_sequence(0, 1);
        assert_eq!(single.as_str(), "H");
    }

    #[test]
    fn test_intersect_edge_cases() {
        let text = "Hello, World!";
        let seq = BasedSequence::new(text);

        // No intersection
        let seq1 = seq.sub_sequence(0, 5);
        let seq2 = seq.sub_sequence(7, 12);
        assert!(seq1.intersect(&seq2).is_none());

        // Full intersection
        let seq3 = seq.sub_sequence(0, 5);
        let seq4 = seq.sub_sequence(0, 5);
        let intersect = seq3.intersect(&seq4).unwrap();
        assert_eq!(intersect.as_str(), "Hello");
    }

    #[test]
    fn test_split_edge_cases() {
        let text = "";
        let seq = BasedSequence::new(text);
        let parts: Vec<_> = seq.split(',').collect();
        assert_eq!(parts.len(), 1);
        assert!(parts[0].is_empty());

        let text2 = ",";
        let seq2 = BasedSequence::new(text2);
        let parts2: Vec<_> = seq2.split(',').collect();
        assert_eq!(parts2.len(), 2);
        assert!(parts2[0].is_empty());
        assert!(parts2[1].is_empty());
    }

    #[test]
    fn test_lines_edge_cases() {
        let text = "";
        let seq = BasedSequence::new(text);
        let lines: Vec<_> = seq.lines().collect();
        assert_eq!(lines.len(), 1);
        assert!(lines[0].is_empty());

        let text2 = "\n";
        let seq2 = BasedSequence::new(text2);
        let lines2: Vec<_> = seq2.lines().collect();
        assert_eq!(lines2.len(), 2);
        assert!(lines2[0].is_empty());
        assert!(lines2[1].is_empty());
    }
}
