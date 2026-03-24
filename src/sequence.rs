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
        assert!(start <= end, "start must be <= end");
        assert!(end <= base.len(), "end must be <= base.len()");
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
        assert!(start <= end, "start must be <= end");
        assert!(end <= self.len(), "end must be <= self.len()");
        Self {
            base: self.base,
            start: self.start + start,
            end: self.start + end,
        }
    }

    /// Create a subsequence from the base (original) string
    pub fn base_sub_sequence(&self, start: usize, end: usize) -> Self {
        assert!(start <= end, "start must be <= end");
        assert!(end <= self.base.len(), "end must be <= base.len()");
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
        assert!(index <= self.len(), "index out of bounds");
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
        let mut count = 0;

        for c in self.base[self.end..].chars() {
            if count >= max_count || !f(c) {
                break;
            }
            end += c.len_utf8();
            count += 1;
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

/// A collection of BasedSequences
#[derive(Clone, Debug)]
pub struct SequenceList<'a> {
    sequences: Vec<BasedSequence<'a>>,
}

impl<'a> SequenceList<'a> {
    /// Create a new empty sequence list
    pub fn new() -> Self {
        Self {
            sequences: Vec::new(),
        }
    }

    /// Add a sequence to the list
    pub fn push(&mut self, seq: BasedSequence<'a>) {
        self.sequences.push(seq);
    }

    /// Get the number of sequences
    pub fn len(&self) -> usize {
        self.sequences.len()
    }

    /// Check if the list is empty
    pub fn is_empty(&self) -> bool {
        self.sequences.is_empty()
    }

    /// Get a sequence at index
    pub fn get(&self, index: usize) -> Option<&BasedSequence<'a>> {
        self.sequences.get(index)
    }

    /// Iterate over sequences
    pub fn iter(&self) -> impl Iterator<Item = &BasedSequence<'a>> {
        self.sequences.iter()
    }

    /// Join all sequences into a single string
    pub fn join(&self, sep: &str) -> String {
        self.sequences
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(sep)
    }
}

impl<'a> Default for SequenceList<'a> {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility functions for sequence operations
pub mod utils {
    use super::BasedSequence;

    /// Normalize end-of-line characters to \n
    pub fn normalize_eol<'a>(seq: BasedSequence<'a>) -> BasedSequence<'a> {
        // For now, return as-is. In full implementation, this would handle \r\n and \r
        seq
    }

    /// Count leading whitespace characters
    pub fn count_leading_whitespace(seq: &BasedSequence) -> usize {
        seq.count_leading(|c| c.is_whitespace())
    }

    /// Count trailing whitespace characters
    pub fn count_trailing_whitespace(seq: &BasedSequence) -> usize {
        seq.count_trailing(|c| c.is_whitespace())
    }

    /// Check if a character is a line ending
    pub fn is_line_end(c: char) -> bool {
        c == '\n' || c == '\r'
    }

    /// Get the line containing a position
    pub fn get_line_at<'a>(seq: BasedSequence<'a>, offset: usize) -> BasedSequence<'a> {
        let base = seq.base();
        let line_start = base[..offset].rfind('\n').map(|i| i + 1).unwrap_or(0);
        let line_end = base[offset..]
            .find('\n')
            .map(|i| offset + i)
            .unwrap_or(base.len());
        BasedSequence::with_offsets(base, line_start, line_end)
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
    fn test_sequence_list() {
        let text = "Hello, World!";
        let seq = BasedSequence::new(text);
        let mut list = SequenceList::new();
        list.push(seq.sub_sequence(0, 5));
        list.push(seq.sub_sequence(7, 12));
        assert_eq!(list.len(), 2);
        assert_eq!(list.join(" "), "Hello World");
    }
}
