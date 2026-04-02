//! Source file management for clmd.
//!
//! This module provides functionality for managing multiple source files
//! and tracking source positions, inspired by Pandoc's Text.Pandoc.Sources
//! module.
//!
//! # Example
//!
//! ```ignore
//! use clmd::parse::util::sources::{Source, SourcePos, Sources};
//!
//! let source = Source::from_string("Hello world");
//! let pos = SourcePos::new(1, 1, 0);
//! ```

use std::fmt;
use std::path::{Path, PathBuf};

/// A source of input text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    /// Input from a file.
    File {
        /// Path to the file.
        path: PathBuf,
        /// File content.
        content: String,
    },
    /// Input from a string (e.g., stdin or programmatic input).
    String {
        /// Optional name for identification.
        name: Option<String>,
        /// The content.
        content: String,
    },
    /// Input from a URL.
    Url {
        /// The URL.
        url: String,
        /// The content.
        content: String,
    },
}

impl Source {
    /// Create a source from a file path and content.
    pub fn from_file(path: impl AsRef<Path>, content: impl Into<String>) -> Self {
        Self::File {
            path: path.as_ref().to_path_buf(),
            content: content.into(),
        }
    }

    /// Create a source from a string.
    pub fn from_string(content: impl Into<String>) -> Self {
        Self::String {
            name: None,
            content: content.into(),
        }
    }

    /// Create a source from a named string.
    pub fn from_named_string(
        name: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        Self::String {
            name: Some(name.into()),
            content: content.into(),
        }
    }

    /// Create a source from a URL.
    pub fn from_url(url: impl Into<String>, content: impl Into<String>) -> Self {
        Self::Url {
            url: url.into(),
            content: content.into(),
        }
    }

    /// Get the content of this source.
    pub fn content(&self) -> &str {
        match self {
            Self::File { content, .. } => content,
            Self::String { content, .. } => content,
            Self::Url { content, .. } => content,
        }
    }

    /// Get the name of this source.
    pub fn name(&self) -> String {
        match self {
            Self::File { path, .. } => path.display().to_string(),
            Self::String { name, .. } => {
                name.clone().unwrap_or_else(|| "<string>".to_string())
            }
            Self::Url { url, .. } => url.clone(),
        }
    }

    /// Get the source type as a string.
    pub fn source_type(&self) -> &'static str {
        match self {
            Self::File { .. } => "file",
            Self::String { .. } => "string",
            Self::Url { .. } => "url",
        }
    }

    /// Check if this is a file source.
    pub fn is_file(&self) -> bool {
        matches!(self, Self::File { .. })
    }

    /// Check if this is a string source.
    pub fn is_string(&self) -> bool {
        matches!(self, Self::String { .. })
    }

    /// Check if this is a URL source.
    pub fn is_url(&self) -> bool {
        matches!(self, Self::Url { .. })
    }

    /// Get the file path if this is a file source.
    pub fn path(&self) -> Option<&Path> {
        match self {
            Self::File { path, .. } => Some(path),
            _ => None,
        }
    }

    /// Get the number of lines in the content.
    pub fn line_count(&self) -> usize {
        self.content().lines().count()
    }

    /// Get the number of characters in the content.
    pub fn char_count(&self) -> usize {
        self.content().chars().count()
    }

    /// Get a specific line from the content.
    pub fn get_line(&self, line_num: usize) -> Option<&str> {
        self.content().lines().nth(line_num.saturating_sub(1))
    }

    /// Get a range of lines from the content.
    pub fn get_lines(&self, start: usize, end: usize) -> Vec<&str> {
        self.content()
            .lines()
            .skip(start.saturating_sub(1))
            .take(end.saturating_sub(start) + 1)
            .collect()
    }
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// A position in a source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourcePos {
    /// Line number (1-indexed).
    pub line: usize,
    /// Column number (1-indexed).
    pub column: usize,
    /// Offset from the start of the file (0-indexed).
    pub offset: usize,
}

impl SourcePos {
    /// Create a new source position.
    pub fn new(line: usize, column: usize, offset: usize) -> Self {
        Self {
            line: line.max(1),
            column: column.max(1),
            offset,
        }
    }

    /// Create a position at the start of a document.
    pub fn start() -> Self {
        Self::new(1, 1, 0)
    }

    /// Advance by one character.
    pub fn advance(&mut self, c: char) {
        self.offset += c.len_utf8();
        if c == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
    }

    /// Advance by a string.
    pub fn advance_str(&mut self, s: &str) {
        for c in s.chars() {
            self.advance(c);
        }
    }

    /// Get the line and column as a tuple.
    pub fn line_column(&self) -> (usize, usize) {
        (self.line, self.column)
    }

    /// Format the position as a string.
    pub fn format(&self) -> String {
        format!("{}:{}", self.line, self.column)
    }
}

impl Default for SourcePos {
    fn default() -> Self {
        Self::start()
    }
}

impl fmt::Display for SourcePos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

/// A range in a source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceRange {
    /// Start position.
    pub start: SourcePos,
    /// End position.
    pub end: SourcePos,
}

impl SourceRange {
    /// Create a new source range.
    pub fn new(start: SourcePos, end: SourcePos) -> Self {
        Self { start, end }
    }

    /// Create a range from a single position.
    pub fn from_pos(pos: SourcePos) -> Self {
        Self {
            start: pos,
            end: pos,
        }
    }

    /// Get the length of the range in characters.
    pub fn len(&self) -> usize {
        self.end.offset.saturating_sub(self.start.offset)
    }

    /// Check if the range is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Check if a position is within this range.
    pub fn contains(&self, pos: SourcePos) -> bool {
        pos.offset >= self.start.offset && pos.offset <= self.end.offset
    }

    /// Merge two ranges.
    pub fn merge(&self, other: &Self) -> Self {
        Self {
            start: if self.start.offset < other.start.offset {
                self.start
            } else {
                other.start
            },
            end: if self.end.offset > other.end.offset {
                self.end
            } else {
                other.end
            },
        }
    }
}

impl Default for SourceRange {
    fn default() -> Self {
        Self::from_pos(SourcePos::start())
    }
}

impl fmt::Display for SourceRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.start.line == self.end.line {
            write!(
                f,
                "{}:{}-{}",
                self.start.line, self.start.column, self.end.column
            )
        } else {
            write!(
                f,
                "{}:{}-{}:{}",
                self.start.line, self.start.column, self.end.line, self.end.column
            )
        }
    }
}

/// A source location combining source and position.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLoc {
    /// The source.
    pub source: Source,
    /// The position in the source.
    pub pos: SourcePos,
}

impl SourceLoc {
    /// Create a new source location.
    pub fn new(source: Source, pos: SourcePos) -> Self {
        Self { source, pos }
    }

    /// Get the source name.
    pub fn name(&self) -> String {
        self.source.name()
    }

    /// Format the location as a string.
    pub fn format(&self) -> String {
        format!("{}:{}", self.name(), self.pos)
    }
}

impl fmt::Display for SourceLoc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format())
    }
}

/// A collection of source files.
#[derive(Debug, Clone, Default)]
pub struct Sources {
    /// The sources.
    sources: Vec<Source>,
    /// Current source index.
    current: Option<usize>,
}

impl Sources {
    /// Create an empty collection of sources.
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
            current: None,
        }
    }

    /// Create from a single source.
    pub fn from_source(source: Source) -> Self {
        let mut sources = Self::new();
        sources.add(source);
        sources
    }

    /// Add a source.
    pub fn add(&mut self, source: Source) {
        if self.current.is_none() {
            self.current = Some(0);
        }
        self.sources.push(source);
    }

    /// Add a file source.
    pub fn add_file(&mut self, path: impl AsRef<Path>, content: impl Into<String>) {
        self.add(Source::from_file(path, content));
    }

    /// Add a string source.
    pub fn add_string(&mut self, content: impl Into<String>) {
        self.add(Source::from_string(content));
    }

    /// Add a named string source.
    pub fn add_named_string(
        &mut self,
        name: impl Into<String>,
        content: impl Into<String>,
    ) {
        self.add(Source::from_named_string(name, content));
    }

    /// Get the number of sources.
    pub fn len(&self) -> usize {
        self.sources.len()
    }

    /// Check if there are no sources.
    pub fn is_empty(&self) -> bool {
        self.sources.is_empty()
    }

    /// Get a source by index.
    pub fn get(&self, index: usize) -> Option<&Source> {
        self.sources.get(index)
    }

    /// Get the current source.
    pub fn current(&self) -> Option<&Source> {
        self.current.and_then(|i| self.sources.get(i))
    }

    /// Set the current source.
    pub fn set_current(&mut self, index: usize) {
        if index < self.sources.len() {
            self.current = Some(index);
        }
    }

    /// Get the current source index.
    pub fn current_index(&self) -> Option<usize> {
        self.current
    }

    /// Move to the next source.
    pub fn advance(&mut self) -> Option<&Source> {
        if let Some(current) = self.current {
            let next = current + 1;
            if next < self.sources.len() {
                self.current = Some(next);
                return self.sources.get(next);
            }
        }
        None
    }

    /// Move to the previous source.
    pub fn prev(&mut self) -> Option<&Source> {
        if let Some(current) = self.current {
            if current > 0 {
                let prev = current - 1;
                self.current = Some(prev);
                return self.sources.get(prev);
            }
        }
        None
    }

    /// Iterate over sources.
    pub fn iter(&self) -> impl Iterator<Item = &Source> {
        self.sources.iter()
    }

    /// Get all content concatenated.
    pub fn concat_content(&self) -> String {
        self.sources
            .iter()
            .map(|s| s.content())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Get total character count.
    pub fn total_chars(&self) -> usize {
        self.sources.iter().map(|s| s.char_count()).sum()
    }

    /// Get total line count.
    pub fn total_lines(&self) -> usize {
        self.sources.iter().map(|s| s.line_count()).sum()
    }
}

impl IntoIterator for Sources {
    type Item = Source;
    type IntoIter = std::vec::IntoIter<Source>;

    fn into_iter(self) -> Self::IntoIter {
        self.sources.into_iter()
    }
}

impl<'a> IntoIterator for &'a Sources {
    type Item = &'a Source;
    type IntoIter = std::slice::Iter<'a, Source>;

    fn into_iter(self) -> Self::IntoIter {
        self.sources.iter()
    }
}

/// A source span with associated data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spanned<T> {
    /// The data.
    pub data: T,
    /// The source range.
    pub range: SourceRange,
    /// The source.
    pub source: Option<Source>,
}

impl<T> Spanned<T> {
    /// Create a new spanned value.
    pub fn new(data: T, range: SourceRange) -> Self {
        Self {
            data,
            range,
            source: None,
        }
    }

    /// Create a new spanned value with source.
    pub fn with_source(data: T, range: SourceRange, source: Source) -> Self {
        Self {
            data,
            range,
            source: Some(source),
        }
    }

    /// Map the data while preserving the span.
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Spanned<U> {
        Spanned {
            data: f(self.data),
            range: self.range,
            source: self.source,
        }
    }

    /// Get the start position.
    pub fn start(&self) -> SourcePos {
        self.range.start
    }

    /// Get the end position.
    pub fn end(&self) -> SourcePos {
        self.range.end
    }
}

impl<T: fmt::Display> fmt::Display for Spanned<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref source) = self.source {
            write!(f, "{}:{}: {}", source.name(), self.range, self.data)
        } else {
            write!(f, "{}: {}", self.range, self.data)
        }
    }
}

/// Create a source position at the start.
pub fn start_pos() -> SourcePos {
    SourcePos::start()
}

/// Create a source range from positions.
pub fn range(start: SourcePos, end: SourcePos) -> SourceRange {
    SourceRange::new(start, end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_from_file() {
        let source = Source::from_file("/path/to/file.md", "content");
        assert!(source.is_file());
        assert_eq!(source.name(), "/path/to/file.md");
        assert_eq!(source.content(), "content");
        assert_eq!(source.source_type(), "file");
    }

    #[test]
    fn test_source_from_string() {
        let source = Source::from_string("hello world");
        assert!(source.is_string());
        assert_eq!(source.name(), "<string>");
        assert_eq!(source.content(), "hello world");
    }

    #[test]
    fn test_source_from_named_string() {
        let source = Source::from_named_string("test", "content");
        assert!(source.is_string());
        assert_eq!(source.name(), "test");
    }

    #[test]
    fn test_source_from_url() {
        let source = Source::from_url("http://example.com/doc.md", "content");
        assert!(source.is_url());
        assert_eq!(source.name(), "http://example.com/doc.md");
    }

    #[test]
    fn test_source_line_count() {
        let source = Source::from_string("line1\nline2\nline3");
        assert_eq!(source.line_count(), 3);
    }

    #[test]
    fn test_source_get_line() {
        let source = Source::from_string("line1\nline2\nline3");
        assert_eq!(source.get_line(1), Some("line1"));
        assert_eq!(source.get_line(2), Some("line2"));
        assert_eq!(source.get_line(3), Some("line3"));
        assert_eq!(source.get_line(4), None);
    }

    #[test]
    fn test_source_pos() {
        let pos = SourcePos::new(5, 10, 100);
        assert_eq!(pos.line, 5);
        assert_eq!(pos.column, 10);
        assert_eq!(pos.offset, 100);
    }

    #[test]
    fn test_source_pos_advance() {
        let mut pos = SourcePos::start();
        pos.advance('a');
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 2);
        assert_eq!(pos.offset, 1);

        pos.advance('\n');
        assert_eq!(pos.line, 2);
        assert_eq!(pos.column, 1);
        assert_eq!(pos.offset, 2);
    }

    #[test]
    fn test_source_pos_advance_str() {
        let mut pos = SourcePos::start();
        pos.advance_str("hello\nworld");
        assert_eq!(pos.line, 2);
        assert_eq!(pos.column, 6);
    }

    #[test]
    fn test_source_range() {
        let start = SourcePos::new(1, 1, 0);
        let end = SourcePos::new(1, 10, 9);
        let range = SourceRange::new(start, end);

        assert_eq!(range.len(), 9);
        assert!(!range.is_empty());
        assert!(range.contains(SourcePos::new(1, 5, 4)));
        assert!(!range.contains(SourcePos::new(2, 1, 20)));
    }

    #[test]
    fn test_source_range_merge() {
        let r1 = SourceRange::new(SourcePos::new(1, 1, 0), SourcePos::new(1, 5, 4));
        let r2 = SourceRange::new(SourcePos::new(1, 3, 2), SourcePos::new(1, 10, 9));
        let merged = r1.merge(&r2);

        assert_eq!(merged.start.offset, 0);
        assert_eq!(merged.end.offset, 9);
    }

    #[test]
    fn test_source_loc() {
        let source = Source::from_string("content");
        let pos = SourcePos::new(1, 5, 4);
        let loc = SourceLoc::new(source, pos);

        assert_eq!(loc.name(), "<string>");
        assert_eq!(loc.format(), "<string>:1:5");
    }

    #[test]
    fn test_sources() {
        let mut sources = Sources::new();
        assert!(sources.is_empty());

        sources.add_string("first");
        sources.add_string("second");

        assert_eq!(sources.len(), 2);
        assert!(!sources.is_empty());

        let current = sources.current().unwrap();
        assert_eq!(current.content(), "first");

        sources.advance();
        let current = sources.current().unwrap();
        assert_eq!(current.content(), "second");
    }

    #[test]
    fn test_sources_concat() {
        let mut sources = Sources::new();
        sources.add_string("line1");
        sources.add_string("line2");

        let concat = sources.concat_content();
        assert!(concat.contains("line1"));
        assert!(concat.contains("line2"));
    }

    #[test]
    fn test_spanned() {
        let range = SourceRange::new(SourcePos::new(1, 1, 0), SourcePos::new(1, 5, 4));
        let spanned = Spanned::new("data", range);

        assert_eq!(spanned.data, "data");
        assert_eq!(spanned.start().line, 1);
        assert_eq!(spanned.end().column, 5);
    }

    #[test]
    fn test_spanned_map() {
        let range = SourceRange::new(SourcePos::new(1, 1, 0), SourcePos::new(1, 5, 4));
        let spanned = Spanned::new(10, range);
        let mapped = spanned.map(|x| x * 2);

        assert_eq!(mapped.data, 20);
    }

    #[test]
    fn test_sources_iterator() {
        let mut sources = Sources::new();
        sources.add_string("a");
        sources.add_string("b");
        sources.add_string("c");

        let contents: Vec<_> = sources.iter().map(|s| s.content()).collect();
        assert_eq!(contents, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_source_display() {
        let source = Source::from_named_string("test.md", "content");
        assert_eq!(format!("{}", source), "test.md");
    }

    #[test]
    fn test_source_pos_display() {
        let pos = SourcePos::new(5, 10, 100);
        assert_eq!(format!("{}", pos), "5:10");
    }

    #[test]
    fn test_source_range_display() {
        let range = SourceRange::new(SourcePos::new(1, 1, 0), SourcePos::new(1, 10, 9));
        assert_eq!(format!("{}", range), "1:1-10");

        let range2 = SourceRange::new(SourcePos::new(1, 1, 0), SourcePos::new(2, 5, 20));
        assert_eq!(format!("{}", range2), "1:1-2:5");
    }
}
