//! Node formatter trait definitions
//!
//! This module defines the core traits for node formatters,
//! inspired by flexmark-java's NodeFormatter interface.

use std::mem::Discriminant;
use std::rc::Rc;

use crate::core::nodes::NodeValue;
use crate::render::commonmark::context::NodeFormatterContext;
use crate::render::commonmark::writer::MarkdownWriter;

/// Type alias for node type discriminant
pub type NodeType = Discriminant<NodeValue>;

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

/// Trait for node formatters
///
/// Implementors of this trait can provide custom formatting for specific node types.
pub trait NodeFormatter: Send + Sync {
    /// Get the node formatting handlers provided by this formatter
    ///
    /// Returns a list of handlers that map node types to formatting functions.
    fn get_node_formatting_handlers(&self) -> Vec<NodeFormattingHandler>;

    /// Get the node types this formatter is interested in
    ///
    /// These node types will be collected during the document traversal
    /// for quick access without re-traversing the AST.
    fn get_node_classes(&self) -> Vec<NodeType> {
        Vec::new()
    }
}

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

    /// Get all node classes from all formatters
    pub fn get_all_node_classes(&self) -> Vec<NodeType> {
        self.formatters
            .iter()
            .flat_map(|f| f.get_node_classes())
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

    fn get_node_classes(&self) -> Vec<NodeType> {
        self.get_all_node_classes()
    }
}

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
        handler.format_open(&text, &mut MockContext::new(), &mut writer);
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

        handler.format_close(&text, &mut MockContext::new(), &mut writer);
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
        handler.format_close(&text, &mut MockContext::new(), &mut writer);
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
        assert_eq!(composed.get_all_node_classes().len(), 0);
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
    fn test_composed_formatter_get_node_classes() {
        struct TestFormatter;
        impl NodeFormatter for TestFormatter {
            fn get_node_formatting_handlers(&self) -> Vec<NodeFormattingHandler> {
                vec![]
            }

            fn get_node_classes(&self) -> Vec<NodeType> {
                vec![
                    std::mem::discriminant(&NodeValue::Paragraph),
                    std::mem::discriminant(&NodeValue::Heading(
                        crate::core::nodes::NodeHeading::default(),
                    )),
                ]
            }
        }

        let mut composed = ComposedNodeFormatter::new();
        composed.add_formatter(Box::new(TestFormatter));

        let classes = composed.get_all_node_classes();
        assert_eq!(classes.len(), 2);
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

            fn get_node_classes(&self) -> Vec<NodeType> {
                vec![std::mem::discriminant(&NodeValue::Paragraph)]
            }
        }

        let formatter = SimpleFormatter;
        assert_eq!(formatter.get_node_formatting_handlers().len(), 1);
        assert_eq!(formatter.get_node_classes().len(), 1);
    }

    // Mock context for testing - provides safe default implementations
    struct MockContext {
        writer: MarkdownWriter,
        options: FormatOptions,
        arena: crate::core::arena::NodeArena,
    }

    impl MockContext {
        fn new() -> Self {
            Self {
                writer: MarkdownWriter::new(
                    crate::options::format::FormatFlags::DEFAULT,
                ),
                options: FormatOptions::new(),
                arena: crate::core::arena::NodeArena::new(),
            }
        }
    }

    impl NodeFormatterContext for MockContext {
        fn get_markdown_writer(&mut self) -> &mut MarkdownWriter {
            &mut self.writer
        }
        fn render(&mut self, _node_id: crate::core::arena::NodeId) {
            // No-op for mock
        }
        fn render_children(&mut self, _node_id: crate::core::arena::NodeId) {
            // No-op for mock
        }
        fn get_formatter_options(&self) -> &FormatOptions {
            &self.options
        }

        fn get_arena(&self) -> &crate::core::arena::NodeArena {
            &self.arena
        }
        fn get_current_node(&self) -> Option<crate::core::arena::NodeId> {
            None
        }
        fn is_in_tight_list(&self) -> bool {
            false
        }
        fn set_tight_list(&mut self, _tight: bool) {}
        fn get_list_nesting_level(&self) -> usize {
            0
        }
        fn increment_list_nesting(&mut self) {}
        fn decrement_list_nesting(&mut self) {}
        fn is_in_block_quote(&self) -> bool {
            false
        }
        fn set_in_block_quote(&mut self, _in_block_quote: bool) {}
        fn get_block_quote_nesting_level(&self) -> usize {
            0
        }
        fn increment_block_quote_nesting(&mut self) {}
        fn decrement_block_quote_nesting(&mut self) {}
        fn start_table_collection(
            &mut self,
            _alignments: Vec<crate::core::nodes::TableAlignment>,
        ) {
        }
        fn add_table_row(&mut self) {}
        fn add_table_cell(&mut self, _content: String) {}
        fn take_table_data(
            &mut self,
        ) -> Option<(Vec<Vec<String>>, Vec<crate::core::nodes::TableAlignment>)>
        {
            None
        }
        fn is_collecting_table(&self) -> bool {
            false
        }
        fn set_skip_children(&mut self, _skip: bool) {}
        fn render_children_to_string(
            &mut self,
            _node_id: crate::core::arena::NodeId,
        ) -> String {
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
}
