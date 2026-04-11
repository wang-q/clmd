//! Core types for CommonMark formatting
//!
//! This module defines the core traits and types for node formatting:
//! - NodeFormatterContext: Context interface for formatting operations
//! - NodeFormatter: Trait for node formatters
//! - NodeFormattingHandler: Handler for specific node types

use std::mem::Discriminant;
use std::rc::Rc;

use crate::core::arena::{NodeArena, NodeId};
use crate::core::nodes::NodeValue;
use crate::options::format::FormatOptions;
use crate::render::commonmark::writer::MarkdownWriter;

// ============================================================================
// Node Type
// ============================================================================

/// Type alias for node type discriminant
pub type NodeType = Discriminant<NodeValue>;

// ============================================================================
// Node Formatting Handler
// ============================================================================

/// A handler for formatting a specific node type
///
/// This handler supports both opening and closing callbacks for nodes
/// that need special handling at the end (like links and images).
#[derive(Clone)]
pub struct NodeFormattingHandler {
    /// The node type this handler can format (using discriminant for efficiency)
    pub node_type: NodeType,
    /// The opening formatter (called when entering the node)
    pub open_formatter: NodeFormatterFn,
    /// The closing formatter (called when exiting the node, optional)
    pub close_formatter: Option<NodeFormatterFn>,
}

impl std::fmt::Debug for NodeFormattingHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NodeFormattingHandler")
            .field("has_close_formatter", &self.close_formatter.is_some())
            .finish_non_exhaustive()
    }
}

impl NodeFormattingHandler {
    /// Create a new node formatting handler with only an opening formatter
    pub fn new<F>(node_type: NodeType, formatter: F) -> Self
    where
        F: Fn(&NodeValue, &mut dyn NodeFormatterContext, &mut MarkdownWriter)
            + Send
            + Sync
            + 'static,
    {
        Self {
            node_type,
            open_formatter: Rc::new(formatter),
            close_formatter: None,
        }
    }

    /// Create a new handler with both opening and closing formatters
    pub fn with_close<F, G>(node_type: NodeType, open: F, close: G) -> Self
    where
        F: Fn(&NodeValue, &mut dyn NodeFormatterContext, &mut MarkdownWriter)
            + Send
            + Sync
            + 'static,
        G: Fn(&NodeValue, &mut dyn NodeFormatterContext, &mut MarkdownWriter)
            + Send
            + Sync
            + 'static,
    {
        Self {
            node_type,
            open_formatter: Rc::new(open),
            close_formatter: Some(Rc::new(close)),
        }
    }

    /// Call the opening formatter
    pub fn format_open(
        &self,
        value: &NodeValue,
        ctx: &mut dyn NodeFormatterContext,
        writer: &mut MarkdownWriter,
    ) {
        (self.open_formatter)(value, ctx, writer);
    }

    /// Call the closing formatter if present
    pub fn format_close(
        &self,
        value: &NodeValue,
        ctx: &mut dyn NodeFormatterContext,
        writer: &mut MarkdownWriter,
    ) {
        if let Some(ref close) = self.close_formatter {
            (close)(value, ctx, writer);
        }
    }
}

/// Type alias for node formatter functions
pub type NodeFormatterFn = Rc<
    dyn Fn(&NodeValue, &mut dyn NodeFormatterContext, &mut MarkdownWriter) + Send + Sync,
>;

// ============================================================================
// Node Formatter Trait
// ============================================================================

/// Trait for node formatters
///
/// Implementors of this trait can provide custom formatting for specific node types.
pub trait NodeFormatter: Send + Sync {
    /// Get the node formatting handlers provided by this formatter
    ///
    /// Returns a list of handlers that map node types to formatting functions.
    fn get_node_formatting_handlers(&self) -> Vec<NodeFormattingHandler>;
}

// ============================================================================
// Composed Node Formatter
// ============================================================================

/// A composed node formatter that combines multiple formatters
pub struct ComposedNodeFormatter {
    formatters: Vec<Box<dyn NodeFormatter>>,
}

impl std::fmt::Debug for ComposedNodeFormatter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComposedNodeFormatter")
            .field("formatters", &self.formatters.len())
            .finish()
    }
}

impl ComposedNodeFormatter {
    /// Create a new composed formatter
    pub fn new() -> Self {
        Self {
            formatters: Vec::new(),
        }
    }

    /// Add a formatter to the composition
    pub fn add_formatter(&mut self, formatter: Box<dyn NodeFormatter>) {
        self.formatters.push(formatter);
    }

    /// Get all handlers from all formatters
    pub fn get_all_handlers(&self) -> Vec<NodeFormattingHandler> {
        self.formatters
            .iter()
            .flat_map(|f| f.get_node_formatting_handlers())
            .collect()
    }
}

impl Default for ComposedNodeFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeFormatter for ComposedNodeFormatter {
    fn get_node_formatting_handlers(&self) -> Vec<NodeFormattingHandler> {
        self.get_all_handlers()
    }
}

// ============================================================================
// Node Formatter Context Trait
// ============================================================================

/// Context for node formatting operations
pub trait NodeFormatterContext {
    /// Render a specific node
    fn render(&mut self, node_id: NodeId);

    /// Render the children of a node
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
    fn start_table_collection(
        &mut self,
        alignments: Vec<crate::core::nodes::TableAlignment>,
    );

    /// Add a table row
    fn add_table_row(&mut self);

    /// Add a table cell
    fn add_table_cell(&mut self, content: String);

    /// Get collected table data and clear it
    fn take_table_data(
        &mut self,
    ) -> Option<(Vec<Vec<String>>, Vec<crate::core::nodes::TableAlignment>)>;

    /// Check if we're currently collecting table data
    fn is_collecting_table(&self) -> bool;

    /// Set whether to skip rendering children (for table cells)
    fn set_skip_children(&mut self, skip: bool);

    /// Render children to a string and return the content
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

    // Paragraph line breaking methods

    /// Start paragraph line breaking
    fn start_paragraph_line_breaking(&mut self, max_width: usize, prefix: String);

    /// Finish paragraph line breaking and return formatted text
    fn finish_paragraph_line_breaking(&mut self) -> Option<String>;

    /// Add text to the paragraph line breaker
    fn add_paragraph_text(&mut self, text: &str);

    /// Add a word to the paragraph line breaker
    fn add_paragraph_word(&mut self, text: &str);

    /// Add an unbreakable unit with markers
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

// ============================================================================
// Test Utilities
// ============================================================================

#[cfg(test)]
/// Test utilities for the commonmark module
pub mod test_utils {
    use super::*;
    use crate::core::arena::{Node, NodeArena};
    use crate::core::nodes::NodeValue;

    /// Shared mock implementation of NodeFormatterContext for testing
    #[derive(Debug)]
    pub struct MockContext {
        /// Node arena for testing
        pub arena: NodeArena,
        /// Formatter options for testing
        pub options: FormatOptions,
        /// Current node being processed
        pub current_node: Option<NodeId>,
        /// Tight list context flag
        pub tight_list: bool,
        /// List nesting level
        pub list_nesting: usize,
        /// Block quote context flag
        pub in_block_quote: bool,
        /// Block quote nesting level
        pub block_quote_nesting: usize,
        /// Table data collection
        pub table_data:
            Option<(Vec<Vec<String>>, Vec<crate::core::nodes::TableAlignment>)>,
    }

    impl MockContext {
        /// Create a new mock context for testing
        pub fn new() -> Self {
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

// ============================================================================
// Node Formatting Handler Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::format::FormatOptions;
    use crate::render::commonmark::writer::MarkdownWriter;

    #[test]
    fn test_node_formatting_handler_new() {
        let handler = NodeFormattingHandler::new(
            std::mem::discriminant(&NodeValue::Paragraph),
            |_, _, writer| {
                writer.append("test");
            },
        );

        assert!(handler.close_formatter.is_none());
    }

    #[test]
    fn test_node_formatting_handler_with_close() {
        let handler = NodeFormattingHandler::with_close(
            std::mem::discriminant(&NodeValue::Link(Box::default())),
            |_, _, writer| {
                writer.append("[");
            },
            |_, _, writer| {
                writer.append("]");
            },
        );

        assert!(handler.close_formatter.is_some());
    }

    #[test]
    fn test_node_formatting_handler_format_open() {
        let handler = NodeFormattingHandler::new(
            std::mem::discriminant(&NodeValue::Paragraph),
            |_, _, writer| {
                writer.append("hello");
            },
        );

        let options = FormatOptions::new();
        let mut writer = MarkdownWriter::new(options.format_flags);
        let text = NodeValue::make_text("test");

        // This should not panic
        handler.format_open(&text, &mut test_utils::MockContext::new(), &mut writer);
    }

    #[test]
    fn test_node_formatting_handler_format_close_with_close() {
        let handler = NodeFormattingHandler::with_close(
            std::mem::discriminant(&NodeValue::Emph),
            |_, _, _| {},
            |_, _, writer| {
                writer.append("*");
            },
        );

        let options = FormatOptions::new();
        let mut writer = MarkdownWriter::new(options.format_flags);
        let text = NodeValue::make_text("test");

        handler.format_close(&text, &mut test_utils::MockContext::new(), &mut writer);
    }

    #[test]
    fn test_node_formatting_handler_format_close_without_close() {
        let handler = NodeFormattingHandler::new(
            std::mem::discriminant(&NodeValue::Paragraph),
            |_, _, _| {},
        );

        let options = FormatOptions::new();
        let mut writer = MarkdownWriter::new(options.format_flags);
        let text = NodeValue::make_text("test");

        // Should not panic even without close formatter
        handler.format_close(&text, &mut test_utils::MockContext::new(), &mut writer);
    }

    #[test]
    fn test_node_formatting_handler_debug() {
        let handler = NodeFormattingHandler::new(
            std::mem::discriminant(&NodeValue::Paragraph),
            |_, _, _| {},
        );
        let debug_str = format!("{:?}", handler);
        assert!(debug_str.contains("NodeFormattingHandler"));
    }

    #[test]
    fn test_node_formatting_handler_clone() {
        let handler = NodeFormattingHandler::new(
            std::mem::discriminant(&NodeValue::Paragraph),
            |_, _, _| {},
        );
        let cloned = handler.clone();
        assert_eq!(
            std::mem::discriminant(&NodeValue::Paragraph),
            cloned.node_type
        );
    }

    #[test]
    fn test_composed_formatter_new() {
        let composed = ComposedNodeFormatter::new();
        assert_eq!(composed.get_all_handlers().len(), 0);
    }

    #[test]
    fn test_composed_formatter_default() {
        let composed: ComposedNodeFormatter = Default::default();
        assert_eq!(composed.get_all_handlers().len(), 0);
    }

    #[test]
    fn test_composed_formatter_add_multiple() {
        struct TestFormatter1;
        impl NodeFormatter for TestFormatter1 {
            fn get_node_formatting_handlers(&self) -> Vec<NodeFormattingHandler> {
                vec![
                    NodeFormattingHandler::new(
                        std::mem::discriminant(&NodeValue::Paragraph),
                        |_, _, _| {},
                    ),
                    NodeFormattingHandler::new(
                        std::mem::discriminant(&NodeValue::Heading(
                            crate::core::nodes::NodeHeading::default(),
                        )),
                        |_, _, _| {},
                    ),
                ]
            }
        }

        struct TestFormatter2;
        impl NodeFormatter for TestFormatter2 {
            fn get_node_formatting_handlers(&self) -> Vec<NodeFormattingHandler> {
                vec![NodeFormattingHandler::new(
                    std::mem::discriminant(&NodeValue::BlockQuote),
                    |_, _, _| {},
                )]
            }
        }

        let mut composed = ComposedNodeFormatter::new();
        composed.add_formatter(Box::new(TestFormatter1));
        composed.add_formatter(Box::new(TestFormatter2));

        let handlers = composed.get_all_handlers();
        assert_eq!(handlers.len(), 3);
    }

    #[test]
    fn test_composed_formatter_debug() {
        let composed = ComposedNodeFormatter::new();
        let debug_str = format!("{:?}", composed);
        assert!(debug_str.contains("ComposedNodeFormatter"));
    }

    #[test]
    fn test_node_formatter_trait() {
        struct SimpleFormatter;
        impl NodeFormatter for SimpleFormatter {
            fn get_node_formatting_handlers(&self) -> Vec<NodeFormattingHandler> {
                vec![NodeFormattingHandler::new(
                    std::mem::discriminant(&NodeValue::Paragraph),
                    |_, _, _| {},
                )]
            }
        }

        let formatter = SimpleFormatter;
        assert_eq!(formatter.get_node_formatting_handlers().len(), 1);
    }
}
