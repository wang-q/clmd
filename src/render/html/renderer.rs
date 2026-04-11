//! HTML renderer implementation
//!
//! This module provides the main HTML renderer for the Arena-based AST.

use crate::core::arena::{NodeArena, NodeId};
use crate::core::nodes::NodeValue;
use crate::options::Options;
use std::fmt::Write;

/// HTML renderer state
pub struct HtmlRenderer<'a> {
    pub(crate) arena: &'a NodeArena,
    pub(crate) output: String,
    /// Render options
    pub(crate) options: &'a Options<'a>,
    /// Stack for tracking whether we need to close a tag
    pub(crate) tag_stack: Vec<&'static str>,
    /// Track if we're in a tight list
    pub(crate) tight_list_stack: Vec<bool>,
    /// Track footnotes for rendering at the end
    pub(crate) footnotes: Vec<(String, NodeId)>,
    /// Track if we're in a code block
    pub(crate) in_code_block: bool,
    /// Track the last output character for cr() logic
    pub(crate) last_out: char,
    /// Counter to disable tag rendering (for image alt text)
    pub(crate) disable_tags: i32,
    /// Track if we're at the first child of a list item (for tight lists)
    pub(crate) item_child_count: Vec<usize>,
    /// Track table row index (0 = header, 1 = header end marker, 2+ = body)
    pub(crate) table_row_index: usize,
}

impl<'a> HtmlRenderer<'a> {
    pub(crate) fn new(arena: &'a NodeArena, options: &'a Options<'a>) -> Self {
        // Optimization: pre-allocate output buffer with estimated capacity
        // Typical HTML output is about 2x the input size
        let estimated_capacity = arena.len() * 64;
        HtmlRenderer {
            arena,
            output: String::with_capacity(estimated_capacity),
            options,
            tag_stack: Vec::new(),
            tight_list_stack: Vec::new(),
            footnotes: Vec::new(),
            in_code_block: false,
            last_out: '\n', // Initialize to newline like commonmark.js
            disable_tags: 0,
            item_child_count: Vec::new(),
            table_row_index: 0,
        }
    }

    /// Render data-sourcepos attribute if sourcepos is enabled
    pub(crate) fn render_sourcepos(&mut self, node_id: NodeId) {
        if self.options.render.sourcepos {
            let node = self.arena.get(node_id);
            let source_pos = &node.source_pos;
            // Only render if source_pos is not default (0,0-0,0)
            if source_pos.start.line != 0 {
                write!(
                    self.output,
                    " data-sourcepos=\"{}:{}-{}:{}\"",
                    source_pos.start.line,
                    source_pos.start.column,
                    source_pos.end.line,
                    source_pos.end.column
                )
                .expect("write to String cannot fail");
            }
        }
    }

    /// Output a newline if the last output wasn't already a newline
    pub(crate) fn cr(&mut self) {
        if self.last_out != '\n' {
            self.output.push('\n');
            self.last_out = '\n';
        }
    }

    /// Output a literal string and track last character
    pub(crate) fn lit(&mut self, s: &str) {
        if s.is_empty() {
            return;
        }

        self.output.push_str(s);
        self.last_out = s.chars().last().unwrap_or('\n');
    }

    /// Check if we're currently inside a tight list
    pub(crate) fn in_tight_list(&self) -> bool {
        self.tight_list_stack.last().copied().unwrap_or(false)
    }

    /// Check if we're inside a list item and track block-level children
    /// Returns true if we should add a newline before this block element
    pub(crate) fn track_item_child(&mut self) -> bool {
        let in_tight_list = self.in_tight_list();
        if let Some(count) = self.item_child_count.last_mut() {
            *count += 1;
            // In tight lists, add newline before block elements after the first one
            if in_tight_list && *count > 1 {
                return true;
            }
        }
        false
    }

    pub(crate) fn render(&mut self, root: NodeId) -> String {
        self.render_node(root, true);

        // Render footnotes if any
        if !self.footnotes.is_empty() {
            self.render_footnotes();
        }

        // Remove trailing newline to match CommonMark spec test format
        while self.output.ends_with('\n') {
            self.output.pop();
        }

        self.output.clone()
    }

    pub(crate) fn render_node(&mut self, node_id: NodeId, entering: bool) {
        if entering {
            self.enter_node(node_id);
            let node = self.arena.get(node_id);

            // For image nodes, don't render children as they are used for alt text
            let is_image = matches!(node.value, NodeValue::Image(..));

            if !is_image {
                let mut child_opt = node.first_child;
                while let Some(child_id) = child_opt {
                    self.render_node(child_id, true);
                    child_opt = self.arena.get(child_id).next;
                }
            }

            self.exit_node(node_id);
        }
    }

    /// Collect alt text from image node's children
    /// Alt text is the plain text content of the image's children, without HTML tags
    pub(crate) fn collect_alt_text(&self, node_id: NodeId) -> String {
        let mut alt_text = String::new();
        self.collect_alt_text_recursive(node_id, &mut alt_text);
        alt_text
    }

    fn collect_alt_text_recursive(&self, node_id: NodeId, alt_text: &mut String) {
        let node = self.arena.get(node_id);
        match &node.value {
            NodeValue::Text(literal) => {
                alt_text.push_str(literal);
            }
            NodeValue::SoftBreak => {
                alt_text.push(' ');
            }
            NodeValue::HardBreak => {
                alt_text.push(' ');
            }
            _ => {
                // For other node types, recursively collect from children
                let mut child_opt = node.first_child;
                while let Some(child_id) = child_opt {
                    self.collect_alt_text_recursive(child_id, alt_text);
                    child_opt = self.arena.get(child_id).next;
                }
            }
        }
    }
}
