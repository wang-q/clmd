//! Container continuation checking for block parsing
//!
//! This module handles checking whether open containers can continue
//! on the current line.

use crate::arena::NodeId;
use crate::blocks::BlockParser;
use crate::nodes::NodeValue;
use crate::{is_space_or_tab, CODE_INDENT};

impl<'a> BlockParser<'a> {
    /// Check which open blocks can continue on this line
    pub(crate) fn check_open_blocks(
        &mut self,
        all_matched: &mut bool,
    ) -> Option<NodeId> {
        *all_matched = true;
        let mut container = self.doc;

        loop {
            let last_child_opt = self.arena.get(container).last_child;
            if let Some(last_child) = last_child_opt {
                if !self.is_open(last_child) {
                    break;
                }
                container = last_child;
            } else {
                break;
            }

            self.find_next_nonspace();

            let result = self.check_container_continuation(container);
            match result {
                0 => {} // Matched, continue
                1 => {
                    // Failed to match
                    *all_matched = false;
                    // Get parent before modifying container
                    let parent_opt = self.arena.get(container).parent;
                    container = parent_opt.unwrap_or(self.doc);
                    break;
                }
                2 => return None, // End of line for fenced code
                _ => panic!("Invalid continuation result"),
            }
        }

        self.all_closed = container == self.old_tip;

        Some(container)
    }

    /// Check if a container can continue on this line
    pub(crate) fn check_container_continuation(&mut self, container: NodeId) -> i32 {
        let node_value = &self.arena.get(container).value;

        match node_value {
            NodeValue::BlockQuote => self.continue_block_quote(),
            NodeValue::List(..) => self.continue_list(container),
            NodeValue::Item(..) => self.continue_item(container),
            NodeValue::CodeBlock(..) => self.continue_code_block(container),
            NodeValue::HtmlBlock(..) => self.continue_html_block(container),
            NodeValue::Heading(..) => {
                // Headings can only contain one line
                1
            }
            NodeValue::ThematicBreak => {
                // Thematic breaks can only contain one line
                1
            }
            NodeValue::Paragraph => {
                if self.blank {
                    1
                } else {
                    0
                }
            }
            NodeValue::Table(..) => {
                // Tables can continue if the current line looks like a table row
                if self.blank {
                    1 // Blank line ends table
                } else {
                    let line = &self.current_line[self.next_nonspace..];
                    if crate::ext::tables::is_table_row(line) {
                        0 // Continue table
                    } else {
                        1 // Not a table row, end table
                    }
                }
            }
            NodeValue::TableRow(..) => {
                // Table rows don't continue - they are complete when created
                1
            }
            NodeValue::TableCell => {
                // Table cells don't continue - they are complete when created
                1
            }
            _ => 0,
        }
    }

    /// Continue block quote
    pub(crate) fn continue_block_quote(&mut self) -> i32 {
        if !self.indented && self.peek_next_nonspace() == Some('>') {
            // Advance past the >
            self.advance_next_nonspace();
            self.advance_offset(1, false);
            // Optional following space
            if self.peek_current().is_some_and(is_space_or_tab) {
                self.advance_offset(1, true);
            }
            0
        } else {
            1
        }
    }

    /// Continue list
    pub(crate) fn continue_list(&mut self, _container: NodeId) -> i32 {
        // Lists always continue - new list items are handled in open_new_blocks
        // This matches commonmark.js behavior where list.continue returns 0
        0
    }

    /// Continue list item
    pub(crate) fn continue_item(&mut self, container: NodeId) -> i32 {
        if self.blank {
            if self.arena.get(container).first_child.is_none() {
                // Blank line after empty list item
                1
            } else {
                // Blank line in list item - mark as loose but continue
                // The list item ends when we encounter a non-blank line that doesn't match
                self.advance_next_nonspace();
                0
            }
        } else {
            // Check indent
            let (marker_offset, padding) = self.get_list_data(container);
            if self.indent >= marker_offset + padding {
                // Advance past the list marker padding
                self.advance_offset(marker_offset + padding, true);
                0
            } else {
                1
            }
        }
    }

    /// Continue code block
    pub(crate) fn continue_code_block(&mut self, container: NodeId) -> i32 {
        let is_fenced = self.is_fenced_code_block(container);

        if is_fenced {
            let (fence_char, fence_length, fence_offset) =
                self.get_fence_info(container);
            // Fenced code block
            if self.indent <= 3 {
                let line = &self.current_line[self.next_nonspace..];
                if line.starts_with(fence_char) {
                    // Check for closing fence
                    let fence_chars: String =
                        line.chars().take_while(|&c| c == fence_char).collect();
                    if fence_chars.len() >= fence_length {
                        // Check that only whitespace follows the fence
                        let after_fence = &line[fence_chars.len()..];
                        let is_closing = after_fence.trim().is_empty();
                        if is_closing {
                            // Closing fence found
                            self.finalize_block(container);
                            return 2;
                        }
                    }
                }
            }
            // Continue with the code block - skip optional spaces of fence offset
            let mut i = fence_offset;
            while i > 0 && self.peek_current().is_some_and(is_space_or_tab) {
                self.advance_offset(1, true);
                i -= 1;
            }
            0
        } else {
            // Indented code block
            if self.indent >= CODE_INDENT {
                self.advance_offset(CODE_INDENT, true);
                0
            } else if self.blank {
                self.advance_next_nonspace();
                0
            } else {
                1
            }
        }
    }

    /// Continue HTML block
    pub(crate) fn continue_html_block(&self, container: NodeId) -> i32 {
        let html_block_type = self.get_html_block_type(container);

        // HTML blocks 6 and 7 can be interrupted by blank lines
        if self.blank && (html_block_type == 6 || html_block_type == 7) {
            1
        } else {
            0
        }
    }
}
