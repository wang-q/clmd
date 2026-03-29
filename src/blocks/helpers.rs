//! Helper functions for block parsing
//!
//! This module provides utility functions for position tracking,
//! character peeking, and block relationship checking.

use crate::arena::NodeId;
use crate::blocks::BlockParser;
use crate::lexer::TAB_STOP;
use crate::nodes::NodeValue;

impl<'a> BlockParser<'a> {
    /// Check if parent can contain child
    pub(crate) fn can_contain(&self, parent: NodeId, child_value: &NodeValue) -> bool {
        let parent_value = &self.arena.get(parent).value;

        match parent_value {
            NodeValue::Document | NodeValue::BlockQuote => {
                !matches!(child_value, NodeValue::Item(..))
            }
            NodeValue::List(..) => matches!(child_value, NodeValue::Item(..)),
            NodeValue::Item(..) => !matches!(child_value, NodeValue::Item(..)),
            _ => false,
        }
    }

    /// Check if block accepts lines
    pub(crate) fn accepts_lines(&self, block: NodeId) -> bool {
        let block_value = &self.arena.get(block).value;
        matches!(
            block_value,
            NodeValue::Paragraph | NodeValue::CodeBlock(..) | NodeValue::HtmlBlock(..)
        )
    }

    /// Find next non-space character
    pub(crate) fn find_next_nonspace(&mut self) {
        let mut chars_to_tab = TAB_STOP - (self.column % TAB_STOP);

        self.next_nonspace = self.offset;
        self.next_nonspace_column = self.column;

        while self.next_nonspace < self.current_line.len() {
            let c = self.current_line.as_bytes()[self.next_nonspace] as char;
            if c == ' ' {
                self.next_nonspace += 1;
                self.next_nonspace_column += 1;
                chars_to_tab -= 1;
                if chars_to_tab == 0 {
                    chars_to_tab = TAB_STOP;
                }
            } else if c == '\t' {
                self.next_nonspace += 1;
                self.next_nonspace_column += chars_to_tab;
                chars_to_tab = TAB_STOP;
            } else {
                break;
            }
        }

        self.indent = self.next_nonspace_column - self.column;
        self.blank = self.next_nonspace >= self.current_line.len()
            || self.current_line.as_bytes()[self.next_nonspace] == b'\n'
            || self.current_line.as_bytes()[self.next_nonspace] == b'\r';
    }

    /// Advance offset
    pub(crate) fn advance_offset(&mut self, count: usize, columns: bool) {
        let mut count = count;
        while count > 0 && self.offset < self.current_line.len() {
            let c = self.current_line.as_bytes()[self.offset] as char;
            if c == '\t' {
                let chars_to_tab = TAB_STOP - (self.column % TAB_STOP);
                if columns {
                    self.partially_consumed_tab = chars_to_tab > count;
                    let chars_to_advance = count.min(chars_to_tab);
                    self.column += chars_to_advance;
                    if !self.partially_consumed_tab {
                        self.offset += 1;
                    }
                    count -= chars_to_advance;
                } else {
                    self.partially_consumed_tab = false;
                    self.column += chars_to_tab;
                    self.offset += 1;
                    count -= 1;
                }
            } else {
                self.partially_consumed_tab = false;
                self.offset += 1;
                self.column += 1;
                count -= 1;
            }
        }
    }

    /// Advance to next non-space
    pub(crate) fn advance_next_nonspace(&mut self) {
        self.offset = self.next_nonspace;
        self.column = self.next_nonspace_column;
        self.partially_consumed_tab = false;
    }

    /// Peek at next non-space
    pub(crate) fn peek_next_nonspace(&self) -> Option<char> {
        if self.next_nonspace < self.current_line.len() {
            Some(self.current_line.as_bytes()[self.next_nonspace] as char)
        } else {
            None
        }
    }

    /// Peek at current position
    pub(crate) fn peek_current(&self) -> Option<char> {
        if self.offset < self.current_line.len() {
            Some(self.current_line.as_bytes()[self.offset] as char)
        } else {
            None
        }
    }

}
