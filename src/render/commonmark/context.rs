//! Main formatter context implementation
//!
//! This module contains the `MainFormatterContext` struct, which is the core context
//! used during CommonMark rendering. It implements the `NodeFormatterContext` trait
//! and manages rendering state including:
//!
//! - List nesting and tight/loose mode
//! - Block quote nesting
//! - Table data collection
//! - Paragraph line breaking
//! - Text collection for inline elements

use crate::core::arena::{NodeArena, NodeId};
use crate::core::nodes::NodeValue;
use crate::options::format::FormatOptions;
use crate::render::commonmark::core::{
    ComposedNodeFormatter, NodeFormatterContext, NodeFormattingHandler,
};
use crate::render::commonmark::line_breaking;
use crate::render::commonmark::writer;
use std::collections::HashMap;
use std::mem::Discriminant;

/// Main formatter context implementation
///
/// This struct holds all the state needed during the rendering process,
/// including handler mappings, list/block quote nesting levels, table data,
/// and line breaking state.
pub(crate) struct MainFormatterContext<'a> {
    /// Reference to the node arena
    arena: &'a NodeArena,
    /// Formatter options
    options: &'a FormatOptions,
    /// Node formatters
    formatters: &'a ComposedNodeFormatter,
    /// Handler map: node type discriminant -> list of handlers
    handler_map: HashMap<Discriminant<NodeValue>, Vec<NodeFormattingHandler>>,
    /// Current node being rendered
    current_node: Option<NodeId>,
    /// Handler delegation stack (discriminant, handler_index)
    handler_stack: Vec<(Discriminant<NodeValue>, usize)>,
    /// Tight list context
    tight_list: bool,
    /// List nesting level
    list_nesting: usize,
    /// Block quote context
    in_block_quote: bool,
    /// Block quote nesting level
    block_quote_nesting: usize,
    /// Table data collection
    table_rows: Vec<Vec<String>>,
    /// Table alignments
    table_alignments: Vec<crate::core::nodes::TableAlignment>,
    /// Whether we're collecting table data
    collecting_table: bool,
    /// Whether to skip rendering children (for table cells)
    skip_children: bool,
    /// Whether delegation was requested by the current handler
    delegation_requested: bool,
    /// Paragraph line breaker for AST-based line breaking
    paragraph_line_breaker: Option<line_breaking::ParagraphLineBreaker>,
    /// Text collection buffer for render_children_to_string
    text_collection_buffer: Option<String>,
}

impl<'a> std::fmt::Debug for MainFormatterContext<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MainFormatterContext")
            .field("current_node", &self.current_node)
            .field("tight_list", &self.tight_list)
            .field("list_nesting", &self.list_nesting)
            .field("in_block_quote", &self.in_block_quote)
            .field("block_quote_nesting", &self.block_quote_nesting)
            .finish_non_exhaustive()
    }
}

impl<'a> MainFormatterContext<'a> {
    /// Create a new main formatter context
    pub fn new(
        arena: &'a NodeArena,
        options: &'a FormatOptions,
        formatters: &'a ComposedNodeFormatter,
    ) -> Self {
        let mut context = Self {
            arena,
            options,
            formatters,
            handler_map: HashMap::new(),
            current_node: None,
            handler_stack: Vec::new(),
            tight_list: false,
            list_nesting: 0,
            in_block_quote: false,
            block_quote_nesting: 0,
            table_rows: Vec::new(),
            table_alignments: Vec::new(),
            collecting_table: false,
            skip_children: false,
            delegation_requested: false,
            paragraph_line_breaker: None,
            text_collection_buffer: None,
        };
        context.build_handler_map();
        context
    }

    /// Build the handler map from all formatters
    fn build_handler_map(&mut self) {
        for handler in self.formatters.get_all_handlers() {
            self.handler_map
                .entry(handler.node_type)
                .or_default()
                .push(handler);
        }
    }

    /// Render a node using the appropriate handler
    pub fn render(&mut self, node_id: NodeId, markdown: &mut writer::MarkdownWriter) {
        self.render_with_handler_index(node_id, markdown, 0);
    }

    /// Render a node starting from a specific handler index
    fn render_with_handler_index(
        &mut self,
        node_id: NodeId,
        markdown: &mut writer::MarkdownWriter,
        handler_index: usize,
    ) {
        let node = self.arena.get(node_id);
        let node_discriminant = std::mem::discriminant(&node.value);

        // Check if we have handlers for this node type
        let handlers = self.handler_map.get(&node_discriminant);

        if let Some(handler_list) = handlers {
            if handler_index < handler_list.len() {
                let handler = handler_list[handler_index].clone();

                self.current_node = Some(node_id);
                self.handler_stack.push((node_discriminant, handler_index));
                let node_value = &self.arena.get(node_id).value;

                // Call the opening formatter
                handler.format_open(node_value, self, markdown);

                // Render children unless skip_children is set
                // (used for table cells to avoid double rendering)
                if !self.skip_children {
                    self.render_children(node_id, markdown);
                }
                self.skip_children = false;

                // Re-set current_node before calling format_close
                // because render_children may have set it to None
                self.current_node = Some(node_id);

                // Call the closing formatter if present
                handler.format_close(node_value, self, markdown);

                self.handler_stack.pop();
                self.current_node = None;

                // Check if delegation was requested
                if self.delegation_requested {
                    self.delegation_requested = false;
                    // Call the next handler for this node type
                    self.render_with_handler_index(node_id, markdown, handler_index + 1);
                }
            } else {
                // No more handlers, render children
                self.render_children(node_id, markdown);
            }
        } else {
            // No handler registered, render children
            self.render_children(node_id, markdown);
        }
    }

    /// Render children of a node
    fn render_children(
        &mut self,
        node_id: NodeId,
        markdown: &mut writer::MarkdownWriter,
    ) {
        let node = self.arena.get(node_id);
        let mut child_id = node.first_child;
        while let Some(child) = child_id {
            self.render(child, markdown);
            child_id = self.arena.get(child).next;
        }
    }
}

impl<'a> NodeFormatterContext for MainFormatterContext<'a> {
    fn render(&mut self, node_id: NodeId) {
        // This is a no-op in the main context because rendering
        // is done through render() with a writer
        let _ = node_id;
    }

    fn render_children(&mut self, node_id: NodeId) {
        // Same as above - this is a no-op in the main context
        let _ = node_id;
    }

    fn get_formatter_options(&self) -> &FormatOptions {
        self.options
    }

    fn get_arena(&self) -> &NodeArena {
        self.arena
    }

    fn get_current_node(&self) -> Option<NodeId> {
        self.current_node
    }

    fn is_in_tight_list(&self) -> bool {
        self.tight_list
    }

    fn set_tight_list(&mut self, tight: bool) {
        self.tight_list = tight;
    }

    fn get_list_nesting_level(&self) -> usize {
        self.list_nesting
    }

    fn increment_list_nesting(&mut self) {
        self.list_nesting += 1;
    }

    fn decrement_list_nesting(&mut self) {
        if self.list_nesting > 0 {
            self.list_nesting -= 1;
        }
    }

    fn is_in_block_quote(&self) -> bool {
        self.in_block_quote
    }

    fn set_in_block_quote(&mut self, in_block_quote: bool) {
        self.in_block_quote = in_block_quote;
    }

    fn get_block_quote_nesting_level(&self) -> usize {
        self.block_quote_nesting
    }

    fn increment_block_quote_nesting(&mut self) {
        self.block_quote_nesting += 1;
    }

    fn decrement_block_quote_nesting(&mut self) {
        if self.block_quote_nesting > 0 {
            self.block_quote_nesting -= 1;
        }
    }

    // Table data collection methods

    fn start_table_collection(
        &mut self,
        alignments: Vec<crate::core::nodes::TableAlignment>,
    ) {
        self.table_rows = Vec::new();
        self.table_alignments = alignments;
        self.collecting_table = true;
    }

    fn add_table_row(&mut self) {
        if self.collecting_table {
            self.table_rows.push(Vec::new());
        }
    }

    fn add_table_cell(&mut self, content: String) {
        if self.collecting_table {
            if let Some(last_row) = self.table_rows.last_mut() {
                last_row.push(content);
            }
        }
    }

    fn take_table_data(
        &mut self,
    ) -> Option<(Vec<Vec<String>>, Vec<crate::core::nodes::TableAlignment>)> {
        if self.collecting_table {
            self.collecting_table = false;
            let rows = std::mem::take(&mut self.table_rows);
            let alignments = std::mem::take(&mut self.table_alignments);
            if !rows.is_empty() {
                Some((rows, alignments))
            } else {
                None
            }
        } else {
            None
        }
    }

    fn is_collecting_table(&self) -> bool {
        self.collecting_table
    }

    fn set_skip_children(&mut self, skip: bool) {
        self.skip_children = skip;
    }

    fn render_children_to_string(&mut self, node_id: NodeId) -> String {
        // Temporarily reset skip_children to allow rendering
        let old_skip_children = self.skip_children;
        self.skip_children = false;

        // Temporarily disable paragraph line breaking to avoid double-adding text
        let old_line_breaker = self.paragraph_line_breaker.take();

        // Set up text collection buffer
        self.text_collection_buffer = Some(String::new());

        // Create a temporary writer to capture output
        let mut temp_writer = writer::MarkdownWriter::new(self.options.format_flags);

        // Render children to the temporary writer
        self.render_children(node_id, &mut temp_writer);

        // Collect the text and clear the buffer
        let result = self.text_collection_buffer.take().unwrap_or_default();

        // Restore paragraph line breaker
        self.paragraph_line_breaker = old_line_breaker;

        // Restore skip_children
        self.skip_children = old_skip_children;

        // Return the captured content
        result
    }

    // Line breaking methods (legacy - deprecated, use paragraph line breaking methods instead)

    // ParagraphLineBreaker methods

    fn start_paragraph_line_breaking(&mut self, max_width: usize, prefix: String) {
        self.paragraph_line_breaker =
            Some(line_breaking::ParagraphLineBreaker::new(max_width, prefix));
    }

    fn finish_paragraph_line_breaking(&mut self) -> Option<String> {
        self.paragraph_line_breaker
            .take()
            .map(|breaker| breaker.format())
    }

    fn add_paragraph_text(&mut self, text: &str) {
        if let Some(ref mut breaker) = self.paragraph_line_breaker {
            breaker.add_text(text);
        }
        // Also add to text collection buffer if active
        if let Some(ref mut buffer) = self.text_collection_buffer {
            buffer.push_str(text);
        }
    }

    fn add_paragraph_word(&mut self, text: &str) {
        if let Some(ref mut breaker) = self.paragraph_line_breaker {
            // For markdown markers like * and **, add as regular word
            // The emphasis/strong content will be handled as a unit by start_unit/end_unit
            breaker.add_word(text);
        }
        // Also add to text collection buffer if active
        if let Some(ref mut buffer) = self.text_collection_buffer {
            buffer.push_str(text);
        }
    }

    fn add_paragraph_unbreakable_unit(
        &mut self,
        kind: line_breaking::AtomicKind,
        prefix: &str,
        content: &str,
        suffix: &str,
    ) {
        if let Some(ref mut breaker) = self.paragraph_line_breaker {
            breaker.add_unbreakable_unit(kind, prefix, content, suffix);
        }
        // Also add to text collection buffer if active
        if let Some(ref mut buffer) = self.text_collection_buffer {
            buffer.push_str(prefix);
            buffer.push_str(content);
            buffer.push_str(suffix);
        }
    }

    fn add_paragraph_hard_break(&mut self) {
        if let Some(ref mut breaker) = self.paragraph_line_breaker {
            breaker.add_hard_break();
        }
    }

    fn is_paragraph_line_breaking(&self) -> bool {
        // Return true if either paragraph line breaker is active or we're collecting text
        self.paragraph_line_breaker.is_some() || self.text_collection_buffer.is_some()
    }
}
