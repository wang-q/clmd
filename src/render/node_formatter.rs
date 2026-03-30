//! Node formatter trait definitions
//!
//! This module defines the core traits for node formatters,
//! inspired by flexmark-java's NodeFormatter interface.

use crate::nodes::NodeValue;
use crate::render::formatter_context::NodeFormatterContext;
use crate::render::markdown_writer::MarkdownWriter;

/// A handler for formatting a specific node type
pub struct NodeFormattingHandler {
    /// The node type this handler can format
    pub node_type: NodeValueType,
    /// The formatting function
    pub formatter: NodeFormatterFn,
}

impl std::fmt::Debug for NodeFormattingHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NodeFormattingHandler")
            .field("node_type", &self.node_type)
            .finish_non_exhaustive()
    }
}

impl NodeFormattingHandler {
    /// Create a new node formatting handler
    pub fn new<F>(node_type: NodeValueType, formatter: F) -> Self
    where
        F: Fn(&NodeValue, &mut dyn NodeFormatterContext, &mut MarkdownWriter) + Send + Sync + 'static,
    {
        Self {
            node_type,
            formatter: Box::new(formatter),
        }
    }

    /// Create a new handler for a specific node type
    pub fn for_type<F>(formatter: F) -> Self
    where
        F: Fn(&NodeValue, &mut dyn NodeFormatterContext, &mut MarkdownWriter) + Send + Sync + 'static,
    {
        Self {
            node_type: NodeValueType::from_formatter::<F>(),
            formatter: Box::new(formatter),
        }
    }
}

/// Type alias for node formatter functions
pub type NodeFormatterFn = Box<
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

    #[test]
    fn test_node_value_type_from_node_value() {
        let text = NodeValue::make_text("hello");
        let ty = NodeValueType::from_node_value(&text);
        assert!(matches!(ty, NodeValueType::Text));

        let heading = NodeValue::Heading(crate::nodes::NodeHeading {
            level: 1,
            setext: false,
            closed: false,
        });
        let ty = NodeValueType::from_node_value(&heading);
        assert!(matches!(ty, NodeValueType::Heading));
    }

    #[test]
    fn test_node_value_type_is_block() {
        assert!(NodeValueType::Paragraph.is_block());
        assert!(NodeValueType::Heading.is_block());
        assert!(!NodeValueType::Text.is_block());
        assert!(!NodeValueType::Emph.is_block());
    }

    #[test]
    fn test_node_value_type_is_container() {
        assert!(NodeValueType::Document.is_container());
        assert!(NodeValueType::Paragraph.is_container());
        assert!(NodeValueType::Emph.is_container());
        assert!(!NodeValueType::Text.is_container());
        assert!(!NodeValueType::Code.is_container());
    }

    #[test]
    fn test_node_formatting_handler() {
        let handler = NodeFormattingHandler::new(NodeValueType::Text, |_, _, writer| {
            writer.append("test");
        });

        assert!(matches!(handler.node_type, NodeValueType::Text));
    }

    #[test]
    fn test_composed_formatter() {
        struct TestFormatter;
        impl NodeFormatter for TestFormatter {
            fn get_node_formatting_handlers(&self) -> Vec<NodeFormattingHandler> {
                vec![NodeFormattingHandler::new(NodeValueType::Text, |_, _, _| {})]
            }
        }

        let mut composed = ComposedNodeFormatter::new();
        composed.add_formatter(Box::new(TestFormatter));

        let handlers = composed.get_all_handlers();
        assert_eq!(handlers.len(), 1);
    }
}
