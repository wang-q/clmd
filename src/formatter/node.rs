//! Node formatter trait definitions
//!
//! This module defines the core traits for node formatters,
//! inspired by flexmark-java's NodeFormatter interface.

use std::rc::Rc;

use crate::core::nodes::NodeValue;
use crate::formatter::context::NodeFormatterContext;
use crate::formatter::writer::MarkdownWriter;

/// A handler for formatting a specific node type
///
/// This handler supports both opening and closing callbacks for nodes
/// that need special handling at the end (like links and images).
#[derive(Clone)]
pub struct NodeFormattingHandler {
    /// The node type this handler can format
    pub node_type: NodeValueType,
    /// The opening formatter (called when entering the node)
    pub open_formatter: NodeFormatterFn,
    /// The closing formatter (called when exiting the node, optional)
    pub close_formatter: Option<NodeFormatterFn>,
}

impl std::fmt::Debug for NodeFormattingHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NodeFormattingHandler")
            .field("node_type", &self.node_type)
            .field("has_close_formatter", &self.close_formatter.is_some())
            .finish_non_exhaustive()
    }
}

impl NodeFormattingHandler {
    /// Create a new node formatting handler with only an opening formatter
    pub fn new<F>(node_type: NodeValueType, formatter: F) -> Self
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
    pub fn with_close<F, G>(node_type: NodeValueType, open: F, close: G) -> Self
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

    /// Create a new handler for a specific node type
    pub fn for_type<F>(formatter: F) -> Self
    where
        F: Fn(&NodeValue, &mut dyn NodeFormatterContext, &mut MarkdownWriter)
            + Send
            + Sync
            + 'static,
    {
        Self {
            node_type: NodeValueType::from_formatter::<F>(),
            open_formatter: Rc::new(formatter),
            close_formatter: None,
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

/// Node value type for identifying node types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeValueType {
    /// Document node
    Document,
    /// Front matter node
    FrontMatter,
    /// Block quote node
    BlockQuote,
    /// List node
    List,
    /// List item node
    Item,
    /// Description list node
    DescriptionList,
    /// Description item node
    DescriptionItem,
    /// Description term node
    DescriptionTerm,
    /// Description details node
    DescriptionDetails,
    /// Code block node
    CodeBlock,
    /// HTML block node
    HtmlBlock,
    /// Paragraph node
    Paragraph,
    /// Heading node
    Heading,
    /// Thematic break node
    ThematicBreak,
    /// Footnote definition node
    FootnoteDefinition,
    /// Footnote reference node
    FootnoteReference,
    /// Table node
    Table,
    /// Table row node
    TableRow,
    /// Table cell node
    TableCell,
    /// Text node
    Text,
    /// Task item node
    TaskItem,
    /// Soft break node
    SoftBreak,
    /// Hard break node
    HardBreak,
    /// Inline code node
    Code,
    /// Inline HTML node
    HtmlInline,
    /// Emphasis node
    Emph,
    /// Strong emphasis node
    Strong,
    /// Strikethrough node
    Strikethrough,
    /// Highlight node
    Highlight,
    /// Insert node
    Insert,
    /// Superscript node
    Superscript,
    /// Subscript node
    Subscript,
    /// Link node
    Link,
    /// Image node
    Image,
    /// Math node
    Math,
    /// Wiki link node
    WikiLink,
    /// Underline node
    Underline,
    /// Spoilered text node
    SpoileredText,
    /// Escaped character node
    Escaped,
    /// Multiline block quote node
    MultilineBlockQuote,
    /// Alert node
    Alert,
    /// Subtext node
    Subtext,
    /// Raw content node
    Raw,
    /// Escaped tag node
    EscapedTag,
    /// Shortcode emoji node
    ShortCode,
    /// Other node type
    Other(&'static str),
}

impl NodeValueType {
    /// Get the type from a node value
    pub fn from_node_value(value: &NodeValue) -> Self {
        match value {
            NodeValue::Document => NodeValueType::Document,
            NodeValue::FrontMatter(_) => NodeValueType::FrontMatter,
            NodeValue::BlockQuote => NodeValueType::BlockQuote,
            NodeValue::List(_) => NodeValueType::List,
            NodeValue::Item(_) => NodeValueType::Item,
            NodeValue::DescriptionList => NodeValueType::DescriptionList,
            NodeValue::DescriptionItem(_) => NodeValueType::DescriptionItem,
            NodeValue::DescriptionTerm => NodeValueType::DescriptionTerm,
            NodeValue::DescriptionDetails => NodeValueType::DescriptionDetails,
            NodeValue::CodeBlock(_) => NodeValueType::CodeBlock,
            NodeValue::HtmlBlock(_) => NodeValueType::HtmlBlock,
            NodeValue::Paragraph => NodeValueType::Paragraph,
            NodeValue::Heading(_) => NodeValueType::Heading,
            NodeValue::ThematicBreak => NodeValueType::ThematicBreak,
            NodeValue::FootnoteDefinition(_) => NodeValueType::FootnoteDefinition,
            NodeValue::FootnoteReference(_) => NodeValueType::FootnoteReference,
            NodeValue::Table(_) => NodeValueType::Table,
            NodeValue::TableRow(_) => NodeValueType::TableRow,
            NodeValue::TableCell => NodeValueType::TableCell,
            NodeValue::Text(_) => NodeValueType::Text,
            NodeValue::TaskItem(_) => NodeValueType::TaskItem,
            NodeValue::SoftBreak => NodeValueType::SoftBreak,
            NodeValue::HardBreak => NodeValueType::HardBreak,
            NodeValue::Code(_) => NodeValueType::Code,
            NodeValue::HtmlInline(_) => NodeValueType::HtmlInline,
            NodeValue::Emph => NodeValueType::Emph,
            NodeValue::Strong => NodeValueType::Strong,
            NodeValue::Strikethrough => NodeValueType::Strikethrough,
            NodeValue::Highlight => NodeValueType::Highlight,
            NodeValue::Insert => NodeValueType::Insert,
            NodeValue::Superscript => NodeValueType::Superscript,
            NodeValue::Subscript => NodeValueType::Subscript,
            NodeValue::Link(_) => NodeValueType::Link,
            NodeValue::Image(_) => NodeValueType::Image,
            NodeValue::Math(_) => NodeValueType::Math,
            NodeValue::WikiLink(_) => NodeValueType::WikiLink,
            NodeValue::Underline => NodeValueType::Underline,
            NodeValue::SpoileredText => NodeValueType::SpoileredText,
            NodeValue::Escaped => NodeValueType::Escaped,
            NodeValue::MultilineBlockQuote(_) => NodeValueType::MultilineBlockQuote,
            NodeValue::Alert(_) => NodeValueType::Alert,
            NodeValue::Subtext => NodeValueType::Subtext,
            NodeValue::Raw(_) => NodeValueType::Raw,
            NodeValue::EscapedTag(_) => NodeValueType::EscapedTag,
            NodeValue::ShortCode(_) => NodeValueType::ShortCode,
        }
    }

    /// Get the type from a formatter function type
    pub fn from_formatter<F>() -> Self
    where
        F: Fn(&NodeValue, &mut dyn NodeFormatterContext, &mut MarkdownWriter),
    {
        NodeValueType::Other(std::any::type_name::<F>())
    }

    /// Get the display name for this type
    pub fn name(&self) -> &'static str {
        match self {
            NodeValueType::Document => "Document",
            NodeValueType::FrontMatter => "FrontMatter",
            NodeValueType::BlockQuote => "BlockQuote",
            NodeValueType::List => "List",
            NodeValueType::Item => "Item",
            NodeValueType::DescriptionList => "DescriptionList",
            NodeValueType::DescriptionItem => "DescriptionItem",
            NodeValueType::DescriptionTerm => "DescriptionTerm",
            NodeValueType::DescriptionDetails => "DescriptionDetails",
            NodeValueType::CodeBlock => "CodeBlock",
            NodeValueType::HtmlBlock => "HtmlBlock",
            NodeValueType::Paragraph => "Paragraph",
            NodeValueType::Heading => "Heading",
            NodeValueType::ThematicBreak => "ThematicBreak",
            NodeValueType::FootnoteDefinition => "FootnoteDefinition",
            NodeValueType::FootnoteReference => "FootnoteReference",
            NodeValueType::Table => "Table",
            NodeValueType::TableRow => "TableRow",
            NodeValueType::TableCell => "TableCell",
            NodeValueType::Text => "Text",
            NodeValueType::TaskItem => "TaskItem",
            NodeValueType::SoftBreak => "SoftBreak",
            NodeValueType::HardBreak => "HardBreak",
            NodeValueType::Code => "Code",
            NodeValueType::HtmlInline => "HtmlInline",
            NodeValueType::Emph => "Emph",
            NodeValueType::Strong => "Strong",
            NodeValueType::Strikethrough => "Strikethrough",
            NodeValueType::Highlight => "Highlight",
            NodeValueType::Insert => "Insert",
            NodeValueType::Superscript => "Superscript",
            NodeValueType::Subscript => "Subscript",
            NodeValueType::Link => "Link",
            NodeValueType::Image => "Image",
            NodeValueType::Math => "Math",
            NodeValueType::WikiLink => "WikiLink",
            NodeValueType::Underline => "Underline",
            NodeValueType::SpoileredText => "SpoileredText",
            NodeValueType::Escaped => "Escaped",
            NodeValueType::MultilineBlockQuote => "MultilineBlockQuote",
            NodeValueType::Alert => "Alert",
            NodeValueType::Subtext => "Subtext",
            NodeValueType::Raw => "Raw",
            NodeValueType::EscapedTag => "EscapedTag",
            NodeValueType::ShortCode => "ShortCode",
            NodeValueType::Other(name) => name,
        }
    }

    /// Check if this is a block-level node type
    pub fn is_block(&self) -> bool {
        matches!(
            self,
            NodeValueType::Document
                | NodeValueType::BlockQuote
                | NodeValueType::List
                | NodeValueType::Item
                | NodeValueType::DescriptionList
                | NodeValueType::DescriptionItem
                | NodeValueType::DescriptionTerm
                | NodeValueType::DescriptionDetails
                | NodeValueType::CodeBlock
                | NodeValueType::HtmlBlock
                | NodeValueType::Paragraph
                | NodeValueType::Heading
                | NodeValueType::ThematicBreak
                | NodeValueType::FootnoteDefinition
                | NodeValueType::Table
                | NodeValueType::TableRow
                | NodeValueType::MultilineBlockQuote
                | NodeValueType::Alert
                | NodeValueType::Subtext
        )
    }

    /// Check if this is an inline-level node type
    pub fn is_inline(&self) -> bool {
        !self.is_block()
    }

    /// Check if this is a container node type (can have children)
    pub fn is_container(&self) -> bool {
        matches!(
            self,
            NodeValueType::Document
                | NodeValueType::BlockQuote
                | NodeValueType::List
                | NodeValueType::Item
                | NodeValueType::DescriptionList
                | NodeValueType::DescriptionItem
                | NodeValueType::DescriptionTerm
                | NodeValueType::DescriptionDetails
                | NodeValueType::Paragraph
                | NodeValueType::Heading
                | NodeValueType::FootnoteDefinition
                | NodeValueType::Table
                | NodeValueType::TableRow
                | NodeValueType::TableCell
                | NodeValueType::Emph
                | NodeValueType::Strong
                | NodeValueType::Strikethrough
                | NodeValueType::Highlight
                | NodeValueType::Insert
                | NodeValueType::Superscript
                | NodeValueType::Subscript
                | NodeValueType::Link
                | NodeValueType::Underline
                | NodeValueType::SpoileredText
                | NodeValueType::MultilineBlockQuote
                | NodeValueType::Alert
        )
    }

    /// Check if this is a leaf node type (cannot have children)
    pub fn is_leaf(&self) -> bool {
        !self.is_container()
    }
}

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
    fn get_node_classes(&self) -> Vec<NodeValueType> {
        Vec::new()
    }

    /// Get the block quote-like prefix character
    ///
    /// Returns a character that should be treated like a block quote prefix
    /// for indentation purposes.
    fn get_block_quote_like_prefix_char(&self) -> Option<char> {
        None
    }
}

/// Trait for node formatters that can be created from options
pub trait NodeFormatterFactory: Send + Sync {
    /// Create a new node formatter
    fn create(&self) -> Box<dyn NodeFormatter>;

    /// Get the formatter classes that must be executed before this one
    fn get_after_dependents(&self) -> Vec<&'static str> {
        Vec::new()
    }

    /// Get the formatter classes that must be executed after this one
    fn get_before_dependents(&self) -> Vec<&'static str> {
        Vec::new()
    }

    /// Check if this formatter affects global scope
    fn affects_global_scope(&self) -> bool {
        false
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
    pub fn get_all_node_classes(&self) -> Vec<NodeValueType> {
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

    fn get_node_classes(&self) -> Vec<NodeValueType> {
        self.get_all_node_classes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formatter::options::FormatterOptions;
    use crate::formatter::writer::MarkdownWriter;

    #[test]
    fn test_node_value_type_from_node_value() {
        let text = NodeValue::make_text("hello");
        let ty = NodeValueType::from_node_value(&text);
        assert!(matches!(ty, NodeValueType::Text));

        let heading = NodeValue::Heading(crate::core::nodes::NodeHeading {
            level: 1,
            setext: false,
            closed: false,
        });
        let ty = NodeValueType::from_node_value(&heading);
        assert!(matches!(ty, NodeValueType::Heading));

        let para = NodeValue::Paragraph;
        let ty = NodeValueType::from_node_value(&para);
        assert!(matches!(ty, NodeValueType::Paragraph));

        let list = NodeValue::List(crate::core::nodes::NodeList {
            list_type: crate::core::nodes::ListType::Bullet,
            marker_offset: 0,
            padding: 0,
            start: 1,
            delimiter: crate::core::nodes::ListDelimType::Period,
            bullet_char: b'-',
            tight: true,
            is_task_list: false,
        });
        let ty = NodeValueType::from_node_value(&list);
        assert!(matches!(ty, NodeValueType::List));
    }

    #[test]
    fn test_node_value_type_is_block() {
        assert!(NodeValueType::Paragraph.is_block());
        assert!(NodeValueType::Heading.is_block());
        assert!(NodeValueType::BlockQuote.is_block());
        assert!(NodeValueType::List.is_block());
        assert!(NodeValueType::CodeBlock.is_block());
        assert!(!NodeValueType::Text.is_block());
        assert!(!NodeValueType::Emph.is_block());
        assert!(!NodeValueType::Strong.is_block());
        assert!(!NodeValueType::Code.is_block());
        assert!(!NodeValueType::Link.is_block());
    }

    #[test]
    fn test_node_value_type_is_inline() {
        assert!(NodeValueType::Text.is_inline());
        assert!(NodeValueType::Emph.is_inline());
        assert!(NodeValueType::Strong.is_inline());
        assert!(NodeValueType::Code.is_inline());
        assert!(NodeValueType::Link.is_inline());
        assert!(NodeValueType::Image.is_inline());
        assert!(!NodeValueType::Paragraph.is_inline());
        assert!(!NodeValueType::Heading.is_inline());
        assert!(!NodeValueType::BlockQuote.is_inline());
    }

    #[test]
    fn test_node_value_type_is_container() {
        assert!(NodeValueType::Document.is_container());
        assert!(NodeValueType::Paragraph.is_container());
        assert!(NodeValueType::Emph.is_container());
        assert!(NodeValueType::Strong.is_container());
        assert!(NodeValueType::List.is_container());
        assert!(NodeValueType::BlockQuote.is_container());
        assert!(!NodeValueType::Text.is_container());
        assert!(!NodeValueType::Code.is_container());
        assert!(!NodeValueType::SoftBreak.is_container());
        assert!(!NodeValueType::HardBreak.is_container());
    }

    #[test]
    fn test_node_value_type_is_leaf() {
        assert!(NodeValueType::Text.is_leaf());
        assert!(NodeValueType::Code.is_leaf());
        assert!(NodeValueType::SoftBreak.is_leaf());
        assert!(NodeValueType::HardBreak.is_leaf());
        assert!(!NodeValueType::Paragraph.is_leaf());
        assert!(!NodeValueType::Document.is_leaf());
        assert!(!NodeValueType::Emph.is_leaf());
    }

    #[test]
    fn test_node_value_type_name() {
        assert_eq!(NodeValueType::Document.name(), "Document");
        assert_eq!(NodeValueType::Paragraph.name(), "Paragraph");
        assert_eq!(NodeValueType::Text.name(), "Text");
        assert_eq!(NodeValueType::Heading.name(), "Heading");
        assert_eq!(NodeValueType::List.name(), "List");
        assert_eq!(NodeValueType::Link.name(), "Link");
        assert_eq!(NodeValueType::Image.name(), "Image");
        assert_eq!(NodeValueType::Code.name(), "Code");
        assert_eq!(NodeValueType::Emph.name(), "Emph");
        assert_eq!(NodeValueType::Strong.name(), "Strong");
    }

    #[test]
    fn test_node_value_type_other_name() {
        let other = NodeValueType::Other("CustomType");
        assert_eq!(other.name(), "CustomType");
    }

    #[test]
    fn test_node_value_type_debug() {
        let ty = NodeValueType::Paragraph;
        let debug_str = format!("{:?}", ty);
        assert!(debug_str.contains("Paragraph"));
    }

    #[test]
    fn test_node_value_type_clone() {
        let ty = NodeValueType::Heading;
        let cloned = ty;
        assert_eq!(ty, cloned);
    }

    #[test]
    fn test_node_value_type_copy() {
        let ty = NodeValueType::Text;
        let copied = ty;
        assert_eq!(ty, copied);
    }

    #[test]
    fn test_node_value_type_eq() {
        assert_eq!(NodeValueType::Text, NodeValueType::Text);
        assert_ne!(NodeValueType::Text, NodeValueType::Paragraph);
    }

    #[test]
    fn test_node_value_type_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(NodeValueType::Text);
        set.insert(NodeValueType::Paragraph);
        set.insert(NodeValueType::Text); // Duplicate
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_node_formatting_handler_new() {
        let handler = NodeFormattingHandler::new(NodeValueType::Text, |_, _, writer| {
            writer.append("test");
        });

        assert!(matches!(handler.node_type, NodeValueType::Text));
        assert!(handler.close_formatter.is_none());
    }

    #[test]
    fn test_node_formatting_handler_with_close() {
        let handler = NodeFormattingHandler::with_close(
            NodeValueType::Link,
            |_, _, writer| {
                writer.append("[");
            },
            |_, _, writer| {
                writer.append("]");
            },
        );

        assert!(matches!(handler.node_type, NodeValueType::Link));
        assert!(handler.close_formatter.is_some());
    }

    #[test]
    fn test_node_formatting_handler_for_type() {
        fn text_formatter(
            _: &NodeValue,
            _: &mut dyn NodeFormatterContext,
            writer: &mut MarkdownWriter,
        ) {
            writer.append("formatted");
        }

        let handler = NodeFormattingHandler::for_type(text_formatter);
        assert!(matches!(handler.node_type, NodeValueType::Other(_)));
    }

    #[test]
    fn test_node_formatting_handler_format_open() {
        let handler = NodeFormattingHandler::new(NodeValueType::Text, |_, _, writer| {
            writer.append("hello");
        });

        let options = FormatterOptions::new();
        let mut writer = MarkdownWriter::new(options.format_flags);
        let text = NodeValue::make_text("test");

        // This should not panic
        handler.format_open(&text, &mut MockContext::new(), &mut writer);
    }

    #[test]
    fn test_node_formatting_handler_format_close_with_close() {
        let handler = NodeFormattingHandler::with_close(
            NodeValueType::Emph,
            |_, _, _| {},
            |_, _, writer| {
                writer.append("*");
            },
        );

        let options = FormatterOptions::new();
        let mut writer = MarkdownWriter::new(options.format_flags);
        let text = NodeValue::make_text("test");

        handler.format_close(&text, &mut MockContext::new(), &mut writer);
    }

    #[test]
    fn test_node_formatting_handler_format_close_without_close() {
        let handler = NodeFormattingHandler::new(NodeValueType::Text, |_, _, _| {});

        let options = FormatterOptions::new();
        let mut writer = MarkdownWriter::new(options.format_flags);
        let text = NodeValue::make_text("test");

        // Should not panic even without close formatter
        handler.format_close(&text, &mut MockContext::new(), &mut writer);
    }

    #[test]
    fn test_node_formatting_handler_debug() {
        let handler = NodeFormattingHandler::new(NodeValueType::Text, |_, _, _| {});
        let debug_str = format!("{:?}", handler);
        assert!(debug_str.contains("NodeFormattingHandler"));
        assert!(debug_str.contains("Text"));
    }

    #[test]
    fn test_node_formatting_handler_clone() {
        let handler = NodeFormattingHandler::new(NodeValueType::Text, |_, _, _| {});
        let cloned = handler.clone();
        assert!(matches!(cloned.node_type, NodeValueType::Text));
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
                    NodeFormattingHandler::new(NodeValueType::Text, |_, _, _| {}),
                    NodeFormattingHandler::new(NodeValueType::Paragraph, |_, _, _| {}),
                ]
            }
        }

        struct TestFormatter2;
        impl NodeFormatter for TestFormatter2 {
            fn get_node_formatting_handlers(&self) -> Vec<NodeFormattingHandler> {
                vec![NodeFormattingHandler::new(
                    NodeValueType::Heading,
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

            fn get_node_classes(&self) -> Vec<NodeValueType> {
                vec![NodeValueType::Text, NodeValueType::Paragraph]
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
                    NodeValueType::Text,
                    |_, _, _| {},
                )]
            }

            fn get_node_classes(&self) -> Vec<NodeValueType> {
                vec![NodeValueType::Text]
            }

            fn get_block_quote_like_prefix_char(&self) -> Option<char> {
                Some('>')
            }
        }

        let formatter = SimpleFormatter;
        assert_eq!(formatter.get_node_formatting_handlers().len(), 1);
        assert_eq!(formatter.get_node_classes().len(), 1);
        assert_eq!(formatter.get_block_quote_like_prefix_char(), Some('>'));
    }

    #[test]
    fn test_node_formatter_factory_trait() {
        struct TestFactory;
        impl NodeFormatterFactory for TestFactory {
            fn create(&self) -> Box<dyn NodeFormatter> {
                struct DummyFormatter;
                impl NodeFormatter for DummyFormatter {
                    fn get_node_formatting_handlers(
                        &self,
                    ) -> Vec<NodeFormattingHandler> {
                        vec![]
                    }
                }
                Box::new(DummyFormatter)
            }

            fn get_after_dependents(&self) -> Vec<&'static str> {
                vec!["OtherFormatter"]
            }

            fn get_before_dependents(&self) -> Vec<&'static str> {
                vec!["AnotherFormatter"]
            }

            fn affects_global_scope(&self) -> bool {
                true
            }
        }

        let factory = TestFactory;
        let formatter = factory.create();
        assert_eq!(formatter.get_node_formatting_handlers().len(), 0);
        assert_eq!(factory.get_after_dependents(), vec!["OtherFormatter"]);
        assert_eq!(factory.get_before_dependents(), vec!["AnotherFormatter"]);
        assert!(factory.affects_global_scope());
    }

    #[test]
    fn test_node_formatter_factory_default_methods() {
        struct SimpleFactory;
        impl NodeFormatterFactory for SimpleFactory {
            fn create(&self) -> Box<dyn NodeFormatter> {
                struct DummyFormatter;
                impl NodeFormatter for DummyFormatter {
                    fn get_node_formatting_handlers(
                        &self,
                    ) -> Vec<NodeFormattingHandler> {
                        vec![]
                    }
                }
                Box::new(DummyFormatter)
            }
        }

        let factory = SimpleFactory;
        assert!(factory.get_after_dependents().is_empty());
        assert!(factory.get_before_dependents().is_empty());
        assert!(!factory.affects_global_scope());
    }

    // Mock context for testing
    struct MockContext;

    impl MockContext {
        fn new() -> Self {
            Self
        }
    }

    impl NodeFormatterContext for MockContext {
        fn get_markdown_writer(&mut self) -> &mut MarkdownWriter {
            unimplemented!()
        }
        fn render(&mut self, _node_id: crate::core::arena::NodeId) {
            unimplemented!()
        }
        fn render_children(&mut self, _node_id: crate::core::arena::NodeId) {
            unimplemented!()
        }
        fn get_formatting_phase(&self) -> crate::formatter::phase::FormattingPhase {
            crate::formatter::phase::FormattingPhase::Document
        }
        fn delegate_render(&mut self) {
            unimplemented!()
        }
        fn get_formatter_options(&self) -> &FormatterOptions {
            unimplemented!()
        }
        fn get_render_purpose(&self) -> crate::formatter::purpose::RenderPurpose {
            crate::formatter::purpose::RenderPurpose::Format
        }
        fn get_arena(&self) -> &crate::core::arena::NodeArena {
            unimplemented!()
        }
        fn get_current_node(&self) -> Option<crate::core::arena::NodeId> {
            None
        }
        fn get_nodes_of_type(
            &self,
            _node_type: NodeValueType,
        ) -> Vec<crate::core::arena::NodeId> {
            vec![]
        }
        fn get_nodes_of_types(
            &self,
            _node_types: &[NodeValueType],
        ) -> Vec<crate::core::arena::NodeId> {
            vec![]
        }
        fn get_block_quote_like_prefix_predicate(&self) -> Box<dyn Fn(char) -> bool> {
            Box::new(|_| false)
        }
        fn get_block_quote_like_prefix_chars(&self) -> &str {
            ""
        }
        fn transform_non_translating(&self, text: &str) -> String {
            text.to_string()
        }
        fn transform_translating(&self, text: &str) -> String {
            text.to_string()
        }
        fn create_sub_context(&self) -> Box<dyn NodeFormatterContext> {
            unimplemented!()
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
        ) -> Option<(Vec<Vec<String>>, Vec<crate::core::nodes::TableAlignment>)> {
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
    }
}
