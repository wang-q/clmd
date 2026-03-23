/// Lexer for CommonMark documents
/// 
/// The lexer processes input text line by line, tracking position information
/// and providing helper methods for parsing.

/// Code indent threshold (4 spaces or 1 tab)
pub const CODE_INDENT: usize = 4;

/// Tab stop size
pub const TAB_STOP: usize = 4;

/// Position in the input
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}

/// Line information
#[derive(Debug, Clone)]
pub struct LineInfo {
    pub content: String,
    pub line_number: usize,
}

/// Lexer state for parsing a single line
#[derive(Debug)]
pub struct LineLexer<'a> {
    pub line: &'a str,
    pub line_number: usize,
    pub offset: usize,
    pub column: usize,
    pub next_nonspace: usize,
    pub next_nonspace_column: usize,
    pub indent: usize,
    pub indented: bool,
    pub blank: bool,
    pub partially_consumed_tab: bool,
}

impl<'a> LineLexer<'a> {
    /// Create a new line lexer
    pub fn new(line: &'a str, line_number: usize) -> Self {
        let mut lexer = LineLexer {
            line,
            line_number,
            offset: 0,
            column: 0,
            next_nonspace: 0,
            next_nonspace_column: 0,
            indent: 0,
            indented: false,
            blank: false,
            partially_consumed_tab: false,
        };
        lexer.find_next_nonspace();
        lexer
    }

    /// Find the next non-space character position
    pub fn find_next_nonspace(&mut self) {
        let mut i = self.offset;
        let mut cols = self.column;

        for c in self.line[i..].chars() {
            if c == ' ' {
                i += 1;
                cols += 1;
            } else if c == '\t' {
                i += 1;
                cols += TAB_STOP - (cols % TAB_STOP);
            } else {
                break;
            }
        }

        self.blank = i >= self.line.len() || self.line[i..].starts_with('\n') || self.line[i..].starts_with('\r');
        self.next_nonspace = i;
        self.next_nonspace_column = cols;
        self.indent = self.next_nonspace_column - self.column;
        self.indented = self.indent >= CODE_INDENT;
    }

    /// Advance to the next non-space position
    pub fn advance_next_nonspace(&mut self) {
        self.offset = self.next_nonspace;
        self.column = self.next_nonspace_column;
        self.partially_consumed_tab = false;
    }

    /// Advance offset by count characters
    /// If columns is true, count columns (tabs count as multiple spaces)
    pub fn advance_offset(&mut self, count: usize, columns: bool) {
        let chars: Vec<char> = self.line[self.offset..].chars().collect();
        let mut remaining = count;
        let mut i = 0;

        while remaining > 0 && i < chars.len() {
            let c = chars[i];
            if c == '\t' {
                let chars_to_tab = TAB_STOP - (self.column % TAB_STOP);
                if columns {
                    self.partially_consumed_tab = chars_to_tab > remaining;
                    let chars_to_advance = if chars_to_tab > remaining { remaining } else { chars_to_tab };
                    self.column += chars_to_advance;
                    if !self.partially_consumed_tab {
                        self.offset += 1;
                    }
                    remaining -= chars_to_advance;
                } else {
                    self.partially_consumed_tab = false;
                    self.column += chars_to_tab;
                    self.offset += 1;
                    remaining -= 1;
                }
            } else {
                self.partially_consumed_tab = false;
                self.offset += c.len_utf8();
                self.column += 1;
                remaining -= 1;
            }
            i += 1;
        }
    }

    /// Peek at a character at the given offset
    pub fn peek(&self, pos: usize) -> Option<char> {
        self.line[pos..].chars().next()
    }

    /// Peek at the current offset
    pub fn peek_current(&self) -> Option<char> {
        self.peek(self.offset)
    }

    /// Peek at the next non-space position
    pub fn peek_next_nonspace(&self) -> Option<char> {
        self.peek(self.next_nonspace)
    }

    /// Get the remaining content from current offset
    pub fn remaining(&self) -> &str {
        &self.line[self.offset..]
    }

    /// Get the content from next non-space position
    pub fn from_next_nonspace(&self) -> &str {
        &self.line[self.next_nonspace..]
    }

    /// Check if the line matches a blank line
    pub fn is_blank(&self) -> bool {
        self.line.trim().is_empty()
    }

    /// Get the indent level
    pub fn get_indent(&self) -> usize {
        self.indent
    }
}

/// Split input text into lines
pub fn split_lines(text: &str) -> Vec<&str> {
    text.lines().collect()
}

/// Check if a character is a space or tab
pub fn is_space_or_tab(c: char) -> bool {
    c == ' ' || c == '\t'
}

/// Check if a string is blank (only whitespace)
pub fn is_blank(s: &str) -> bool {
    s.trim().is_empty()
}

/// Count leading spaces on a line
pub fn count_leading_spaces(line: &str) -> usize {
    line.chars().take_while(|&c| c == ' ').count()
}

/// Count leading whitespace (spaces and tabs) on a line,
/// accounting for tab stops
pub fn count_leading_whitespace(line: &str) -> (usize, usize) {
    let mut offset = 0;
    let mut column = 0;

    for c in line.chars() {
        if c == ' ' {
            offset += 1;
            column += 1;
        } else if c == '\t' {
            offset += 1;
            column += TAB_STOP - (column % TAB_STOP);
        } else {
            break;
        }
    }

    (offset, column)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_lexer_basic() {
        let line = "  Hello world";
        let lexer = LineLexer::new(line, 1);

        assert_eq!(lexer.offset, 0);
        assert_eq!(lexer.column, 0);
        assert_eq!(lexer.next_nonspace, 2);
        assert_eq!(lexer.next_nonspace_column, 2);
        assert_eq!(lexer.indent, 2);
        assert!(!lexer.indented);
        assert!(!lexer.blank);
    }

    #[test]
    fn test_line_lexer_with_tabs() {
        let line = "\tHello";
        let mut lexer = LineLexer::new(line, 1);

        assert_eq!(lexer.next_nonspace, 1);
        assert_eq!(lexer.next_nonspace_column, 4); // Tab = 4 spaces
        assert_eq!(lexer.indent, 4);
        assert!(lexer.indented);

        lexer.advance_next_nonspace();
        assert_eq!(lexer.peek_current(), Some('H'));
    }

    #[test]
    fn test_line_lexer_blank() {
        let line = "   ";
        let lexer = LineLexer::new(line, 1);

        assert!(lexer.blank);
        assert!(lexer.is_blank());
    }

    #[test]
    fn test_advance_offset() {
        let line = "Hello world";
        let mut lexer = LineLexer::new(line, 1);

        lexer.advance_offset(5, false);
        assert_eq!(lexer.offset, 5);
        assert_eq!(lexer.column, 5);
        assert_eq!(lexer.peek_current(), Some(' '));
    }

    #[test]
    fn test_advance_offset_with_tabs() {
        let line = "\t\tHello";
        let mut lexer = LineLexer::new(line, 1);

        lexer.advance_offset(1, false);
        assert_eq!(lexer.column, 4);

        lexer.advance_offset(1, false);
        assert_eq!(lexer.column, 8);
    }

    #[test]
    fn test_split_lines() {
        let text = "line1\nline2\nline3";
        let lines = split_lines(text);
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "line1");
        assert_eq!(lines[1], "line2");
        assert_eq!(lines[2], "line3");
    }

    #[test]
    fn test_is_space_or_tab() {
        assert!(is_space_or_tab(' '));
        assert!(is_space_or_tab('\t'));
        assert!(!is_space_or_tab('a'));
    }

    #[test]
    fn test_count_leading_whitespace() {
        let line = "  \t  hello";
        let (offset, column) = count_leading_whitespace(line);
        assert_eq!(offset, 5); // 2 spaces + 1 tab + 2 spaces = 5 characters
        assert_eq!(column, 6); // 2 spaces + 4 for tab = 6 columns (tab stops at 4)
    }
}
