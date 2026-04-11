//! Node formatter context trait definitions
//!
//! This module defines the context trait for node formatters,
//! inspired by flexmark-java's NodeFormatterContext interface.

use crate::core::arena::{NodeArena, NodeId};
use crate::core::nodes::NodeValue;
use crate::options::format::FormatOptions;
use crate::render::commonmark::node::NodeType;

/// Context for node formatting operations
///
/// This trait provides the interface that node formatters use to
/// interact with the formatting system, including rendering nodes,
/// accessing configuration, and managing output.
pub trait NodeFormatterContext {
    /// Render a specific node
    ///
    /// This should be used to render child nodes. Be careful not to
    /// pass the node that is currently being rendered, as that would
    /// result in infinite recursion.
    fn render(&mut self, node_id: NodeId);

    /// Render the children of a node
    ///
    /// Renders all child nodes of the given parent node.
    fn render_children(&mut self, node_id: NodeId);

    /// Get the formatter options
    fn get_formatter_options(&self) -> &FormatOptions;

    /// Get the node arena
    fn get_arena(&self) -> &NodeArena;

    /// Get the current node being rendered
    fn get_current_node(&self) -> Option<NodeId>;

    /// Get the parent of the current node
    fn get_current_node_parent(&self) -> Option<NodeId> {
        self.get_current_node()
            .and_then(|id| self.get_arena().get(id).parent)
    }

    /// Check if a node has a child of a specific type
    fn has_child_of_type(&self, node_id: NodeId, node_type: NodeType) -> bool {
        let arena = self.get_arena();
        let node = arena.get(node_id);
        if let Some(first_child) = node.first_child {
            let mut current = Some(first_child);
            while let Some(child_id) = current {
                let child = arena.get(child_id);
                if std::mem::discriminant(&child.value) == node_type {
                    return true;
                }
                // Recursively check grandchildren
                if self.has_child_of_type(child_id, node_type) {
                    return true;
                }
                current = child.next;
            }
        }
        false
    }

    // Table data collection methods

    /// Start collecting table data
    ///
    /// Called when entering a table node to begin collecting row and cell data.
    fn start_table_collection(
        &mut self,
        alignments: Vec<crate::core::nodes::TableAlignment>,
    );

    /// Add a table row
    ///
    /// Called when entering a table row.
    fn add_table_row(&mut self);

    /// Add a table cell
    ///
    /// Called when rendering a table cell with its content.
    fn add_table_cell(&mut self, content: String);

    /// Get collected table data and clear it
    ///
    /// Called when exiting a table node to get all collected data for formatting.
    fn take_table_data(
        &mut self,
    ) -> Option<(Vec<Vec<String>>, Vec<crate::core::nodes::TableAlignment>)>;

    /// Check if we're currently collecting table data
    fn is_collecting_table(&self) -> bool;

    /// Set whether to skip rendering children (for table cells)
    fn set_skip_children(&mut self, skip: bool);

    /// Render children to a string and return the content
    ///
    /// This is used to capture the rendered output of child nodes
    /// without writing to the main output.
    fn render_children_to_string(&mut self, node_id: NodeId) -> String;

    /// Check if we're currently in a tight list context
    fn is_in_tight_list(&self) -> bool;

    /// Set the tight list context
    fn set_tight_list(&mut self, tight: bool);

    /// Get the current list nesting level
    fn get_list_nesting_level(&self) -> usize;

    /// Increment the list nesting level
    fn increment_list_nesting(&mut self);

    /// Decrement the list nesting level
    fn decrement_list_nesting(&mut self);

    /// Check if we're in a block quote context
    fn is_in_block_quote(&self) -> bool;

    /// Set the block quote context
    fn set_in_block_quote(&mut self, in_block_quote: bool);

    /// Get the current block quote nesting level
    fn get_block_quote_nesting_level(&self) -> usize;

    /// Increment the block quote nesting level
    fn increment_block_quote_nesting(&mut self);

    /// Decrement the block quote nesting level
    fn decrement_block_quote_nesting(&mut self);

    /// Check if the current node's parent is a list item
    fn is_parent_list_item(&self) -> bool {
        self.get_current_node_parent().is_some_and(|parent_id| {
            matches!(self.get_arena().get(parent_id).value, NodeValue::Item(_))
        })
    }

    /// Check if the current node has a next sibling
    fn has_next_sibling(&self) -> bool {
        self.get_current_node()
            .is_some_and(|node_id| self.get_arena().get(node_id).next.is_some())
    }

    // ParagraphLineBreaker methods

    /// Start paragraph line breaking with the new AST-based breaker
    fn start_paragraph_line_breaking(&mut self, max_width: usize, prefix: String);

    /// Finish paragraph line breaking and return formatted text
    fn finish_paragraph_line_breaking(&mut self) -> Option<String>;

    /// Add text to the paragraph line breaker
    fn add_paragraph_text(&mut self, text: &str);

    /// Add a word to the paragraph line breaker
    fn add_paragraph_word(&mut self, text: &str);

    /// Add an unbreakable unit with markers (prefix, content, suffix)
    fn add_paragraph_unbreakable_unit(
        &mut self,
        kind: crate::render::commonmark::line_breaking::UnitKind,
        prefix: &str,
        content: &str,
        suffix: &str,
    );

    /// Add a hard line break to the paragraph line breaker
    fn add_paragraph_hard_break(&mut self);

    /// Check if paragraph line breaking is active
    fn is_paragraph_line_breaking(&self) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::arena::{Node, NodeArena};
    use crate::core::nodes::NodeValue;

    /// Mock implementation of NodeFormatterContext for testing
    struct MockContext {
        arena: NodeArena,
        options: FormatOptions,
        current_node: Option<NodeId>,
        tight_list: bool,
        list_nesting: usize,
        in_block_quote: bool,
        block_quote_nesting: usize,
        table_data: Option<(Vec<Vec<String>>, Vec<crate::core::nodes::TableAlignment>)>,
    }

    impl MockContext {
        fn new() -> Self {
            Self {
                arena: NodeArena::new(),
                options: FormatOptions::new(),
                current_node: None,
                tight_list: false,
                list_nesting: 0,
                in_block_quote: false,
                block_quote_nesting: 0,
                table_data: None,
            }
        }
    }

    impl NodeFormatterContext for MockContext {
        fn render(&mut self, _node_id: NodeId) {
            unimplemented!()
        }

        fn render_children(&mut self, _node_id: NodeId) {
            unimplemented!()
        }

        fn get_formatter_options(&self) -> &FormatOptions {
            &self.options
        }

        fn get_arena(&self) -> &NodeArena {
            &self.arena
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

        fn start_table_collection(
            &mut self,
            alignments: Vec<crate::core::nodes::TableAlignment>,
        ) {
            self.table_data = Some((vec![], alignments));
        }

        fn add_table_row(&mut self) {
            if let Some((rows, _)) = &mut self.table_data {
                rows.push(vec![]);
            }
        }

        fn add_table_cell(&mut self, content: String) {
            if let Some((rows, _)) = &mut self.table_data {
                if let Some(last_row) = rows.last_mut() {
                    last_row.push(content);
                } else {
                    rows.push(vec![content]);
                }
            }
        }

        fn take_table_data(
            &mut self,
        ) -> Option<(Vec<Vec<String>>, Vec<crate::core::nodes::TableAlignment>)>
        {
            self.table_data.take()
        }

        fn is_collecting_table(&self) -> bool {
            self.table_data.is_some()
        }

        fn set_skip_children(&mut self, _skip: bool) {}

        fn render_children_to_string(&mut self, _node_id: NodeId) -> String {
            String::new()
        }

        fn start_paragraph_line_breaking(&mut self, _max_width: usize, _prefix: String) {
        }

        fn finish_paragraph_line_breaking(&mut self) -> Option<String> {
            None
        }

        fn add_paragraph_text(&mut self, _text: &str) {}

        fn add_paragraph_word(&mut self, _text: &str) {}

        fn add_paragraph_unbreakable_unit(
            &mut self,
            _kind: crate::render::commonmark::line_breaking::UnitKind,
            _prefix: &str,
            _content: &str,
            _suffix: &str,
        ) {
        }

        fn add_paragraph_hard_break(&mut self) {}

        fn is_paragraph_line_breaking(&self) -> bool {
            false
        }
    }

    #[test]
    fn test_mock_context_tight_list() {
        let mut ctx = MockContext::new();
        assert!(!ctx.is_in_tight_list());

        ctx.set_tight_list(true);
        assert!(ctx.is_in_tight_list());

        ctx.set_tight_list(false);
        assert!(!ctx.is_in_tight_list());
    }

    #[test]
    fn test_mock_context_list_nesting() {
        let mut ctx = MockContext::new();
        assert_eq!(ctx.get_list_nesting_level(), 0);

        ctx.increment_list_nesting();
        assert_eq!(ctx.get_list_nesting_level(), 1);

        ctx.increment_list_nesting();
        assert_eq!(ctx.get_list_nesting_level(), 2);

        ctx.decrement_list_nesting();
        assert_eq!(ctx.get_list_nesting_level(), 1);

        ctx.decrement_list_nesting();
        assert_eq!(ctx.get_list_nesting_level(), 0);

        // Should not go below 0
        ctx.decrement_list_nesting();
        assert_eq!(ctx.get_list_nesting_level(), 0);
    }

    #[test]
    fn test_mock_context_block_quote() {
        let mut ctx = MockContext::new();
        assert!(!ctx.is_in_block_quote());
        assert_eq!(ctx.get_block_quote_nesting_level(), 0);

        ctx.set_in_block_quote(true);
        assert!(ctx.is_in_block_quote());

        ctx.increment_block_quote_nesting();
        assert_eq!(ctx.get_block_quote_nesting_level(), 1);

        ctx.increment_block_quote_nesting();
        assert_eq!(ctx.get_block_quote_nesting_level(), 2);

        ctx.decrement_block_quote_nesting();
        assert_eq!(ctx.get_block_quote_nesting_level(), 1);

        ctx.decrement_block_quote_nesting();
        assert_eq!(ctx.get_block_quote_nesting_level(), 0);

        // Should not go below 0
        ctx.decrement_block_quote_nesting();
        assert_eq!(ctx.get_block_quote_nesting_level(), 0);
    }

    #[test]
    fn test_mock_context_table_collection() {
        let mut ctx = MockContext::new();
        assert!(!ctx.is_collecting_table());
        assert!(ctx.take_table_data().is_none());

        // Start collecting table data
        ctx.start_table_collection(vec![
            crate::core::nodes::TableAlignment::Left,
            crate::core::nodes::TableAlignment::Center,
        ]);
        assert!(ctx.is_collecting_table());

        // Add rows and cells
        ctx.add_table_row();
        ctx.add_table_cell("Cell 1".to_string());
        ctx.add_table_cell("Cell 2".to_string());

        ctx.add_table_row();
        ctx.add_table_cell("Cell 3".to_string());
        ctx.add_table_cell("Cell 4".to_string());

        // Take table data
        let (rows, alignments) = ctx.take_table_data().unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0], vec!["Cell 1", "Cell 2"]);
        assert_eq!(rows[1], vec!["Cell 3", "Cell 4"]);
        assert_eq!(alignments.len(), 2);

        // After taking, should be None
        assert!(!ctx.is_collecting_table());
        assert!(ctx.take_table_data().is_none());
    }

    #[test]
    fn test_mock_context_get_current_node_parent() {
        let mut ctx = MockContext::new();

        // Create a simple tree: Document -> Paragraph -> Text
        let doc = ctx.arena.alloc(Node::with_value(NodeValue::Document));
        let para = ctx.arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = ctx
            .arena
            .alloc(Node::with_value(NodeValue::make_text("Hello")));

        // Set up parent relationships manually
        ctx.arena.get_mut(para).parent = Some(doc);
        ctx.arena.get_mut(text).parent = Some(para);

        // No current node set
        assert!(ctx.get_current_node_parent().is_none());

        // Set current node to text
        ctx.current_node = Some(text);
        assert_eq!(ctx.get_current_node_parent(), Some(para));

        // Set current node to paragraph
        ctx.current_node = Some(para);
        assert_eq!(ctx.get_current_node_parent(), Some(doc));

        // Set current node to document (no parent)
        ctx.current_node = Some(doc);
        assert!(ctx.get_current_node_parent().is_none());
    }
}
