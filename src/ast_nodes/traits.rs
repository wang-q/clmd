//! Node type traits
//!
//! Defines the core traits for AST node types.
//! This is the foundation of the trait-based node system.

use std::fmt::Debug;

/// Core trait for all AST nodes
///
/// This trait defines the basic properties that all node types must have.
/// It's designed to be similar to flexmark-java's NodeType concept.
pub trait NodeType: Debug {
    /// Get the node type name
    fn node_type_name(&self) -> &'static str;

    /// Check if this is a block-level node
    fn is_block(&self) -> bool;

    /// Check if this is an inline-level node
    fn is_inline(&self) -> bool;

    /// Check if this is a leaf node (cannot have children)
    fn is_leaf(&self) -> bool;
}

/// Trait for block-level nodes
///
/// Block nodes can contain other block nodes or inline nodes.
pub trait BlockNode: NodeType {}

/// Trait for inline-level nodes
///
/// Inline nodes can contain other inline nodes.
pub trait InlineNode: NodeType {}

/// Trait for nodes that can be visited
///
/// This allows nodes to participate in the visitor pattern.
pub trait Visitable {
    /// Accept a visitor
    fn accept<V: Visitor>(&self, visitor: &mut V);
}

/// Visitor trait for typed node visitation
///
/// This is an alternative to the generic Visitor in ast::visitor
/// that provides type-safe visitation for specific node types.
pub trait Visitor {
    /// Visit a document node
    fn visit_document(&mut self, node: &Document) {
        self.visit_default(node);
    }

    /// Visit a block quote node
    fn visit_block_quote(&mut self, node: &BlockQuote) {
        self.visit_default(node);
    }

    /// Visit a list node
    fn visit_list(&mut self, node: &List) {
        self.visit_default(node);
    }

    /// Visit a list item node
    fn visit_item(&mut self, node: &Item) {
        self.visit_default(node);
    }

    /// Visit a code block node
    fn visit_code_block(&mut self, node: &CodeBlock) {
        self.visit_default(node);
    }

    /// Visit an HTML block node
    fn visit_html_block(&mut self, node: &HtmlBlock) {
        self.visit_default(node);
    }

    /// Visit a paragraph node
    fn visit_paragraph(&mut self, node: &Paragraph) {
        self.visit_default(node);
    }

    /// Visit a heading node
    fn visit_heading(&mut self, node: &Heading) {
        self.visit_default(node);
    }

    /// Visit a thematic break node
    fn visit_thematic_break(&mut self, node: &ThematicBreak) {
        self.visit_default(node);
    }

    /// Visit a text node
    fn visit_text(&mut self, node: &Text) {
        self.visit_default(node);
    }

    /// Visit a soft break node
    fn visit_soft_break(&mut self, node: &SoftBreak) {
        self.visit_default(node);
    }

    /// Visit a line break node
    fn visit_line_break(&mut self, node: &LineBreak) {
        self.visit_default(node);
    }

    /// Visit an inline code node
    fn visit_code(&mut self, node: &Code) {
        self.visit_default(node);
    }

    /// Visit an inline HTML node
    fn visit_html_inline(&mut self, node: &HtmlInline) {
        self.visit_default(node);
    }

    /// Visit an emphasis node
    fn visit_emph(&mut self, node: &Emph) {
        self.visit_default(node);
    }

    /// Visit a strong node
    fn visit_strong(&mut self, node: &Strong) {
        self.visit_default(node);
    }

    /// Visit a link node
    fn visit_link(&mut self, node: &Link) {
        self.visit_default(node);
    }

    /// Visit an image node
    fn visit_image(&mut self, node: &Image) {
        self.visit_default(node);
    }

    /// Default visit handler
    fn visit_default<N: NodeType>(&mut self, _node: &N) {}
}

// Forward declarations of node types
// These will be fully defined in their respective modules

/// Document node
#[derive(Debug, Clone)]
pub struct Document;

/// Block quote node
#[derive(Debug, Clone)]
pub struct BlockQuote;

/// List node
#[derive(Debug, Clone)]
pub struct List {
    pub list_type: ListType,
    pub delim: DelimType,
    pub start: u32,
    pub tight: bool,
    pub bullet_char: char,
}

/// List item node
#[derive(Debug, Clone)]
pub struct Item;

/// Code block node
#[derive(Debug, Clone)]
pub struct CodeBlock {
    pub info: String,
    pub literal: String,
}

/// HTML block node
#[derive(Debug, Clone)]
pub struct HtmlBlock {
    pub literal: String,
}

/// Paragraph node
#[derive(Debug, Clone)]
pub struct Paragraph;

/// Heading node
#[derive(Debug, Clone)]
pub struct Heading {
    pub level: u32,
}

/// Thematic break node
#[derive(Debug, Clone)]
pub struct ThematicBreak;

/// Text node
#[derive(Debug, Clone)]
pub struct Text {
    pub literal: String,
}

/// Soft break node
#[derive(Debug, Clone)]
pub struct SoftBreak;

/// Line break node
#[derive(Debug, Clone)]
pub struct LineBreak;

/// Inline code node
#[derive(Debug, Clone)]
pub struct Code {
    pub literal: String,
}

/// Inline HTML node
#[derive(Debug, Clone)]
pub struct HtmlInline {
    pub literal: String,
}

/// Emphasis node
#[derive(Debug, Clone)]
pub struct Emph;

/// Strong node
#[derive(Debug, Clone)]
pub struct Strong;

/// Link node
#[derive(Debug, Clone)]
pub struct Link {
    pub url: String,
    pub title: String,
}

/// Image node
#[derive(Debug, Clone)]
pub struct Image {
    pub url: String,
    pub title: String,
}

/// List type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListType {
    Bullet,
    Ordered,
    None,
}

/// Delimiter type for ordered lists
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DelimType {
    Period,
    Paren,
    None,
}

// Implement NodeType for all node types

impl NodeType for Document {
    fn node_type_name(&self) -> &'static str {
        "Document"
    }

    fn is_block(&self) -> bool {
        true
    }

    fn is_inline(&self) -> bool {
        false
    }

    fn is_leaf(&self) -> bool {
        false
    }
}

impl BlockNode for Document {}

impl NodeType for BlockQuote {
    fn node_type_name(&self) -> &'static str {
        "BlockQuote"
    }

    fn is_block(&self) -> bool {
        true
    }

    fn is_inline(&self) -> bool {
        false
    }

    fn is_leaf(&self) -> bool {
        false
    }
}

impl BlockNode for BlockQuote {}

impl NodeType for List {
    fn node_type_name(&self) -> &'static str {
        "List"
    }

    fn is_block(&self) -> bool {
        true
    }

    fn is_inline(&self) -> bool {
        false
    }

    fn is_leaf(&self) -> bool {
        false
    }
}

impl BlockNode for List {}

impl NodeType for Item {
    fn node_type_name(&self) -> &'static str {
        "Item"
    }

    fn is_block(&self) -> bool {
        true
    }

    fn is_inline(&self) -> bool {
        false
    }

    fn is_leaf(&self) -> bool {
        false
    }
}

impl BlockNode for Item {}

impl NodeType for CodeBlock {
    fn node_type_name(&self) -> &'static str {
        "CodeBlock"
    }

    fn is_block(&self) -> bool {
        true
    }

    fn is_inline(&self) -> bool {
        false
    }

    fn is_leaf(&self) -> bool {
        true
    }
}

impl BlockNode for CodeBlock {}

impl NodeType for HtmlBlock {
    fn node_type_name(&self) -> &'static str {
        "HtmlBlock"
    }

    fn is_block(&self) -> bool {
        true
    }

    fn is_inline(&self) -> bool {
        false
    }

    fn is_leaf(&self) -> bool {
        true
    }
}

impl BlockNode for HtmlBlock {}

impl NodeType for Paragraph {
    fn node_type_name(&self) -> &'static str {
        "Paragraph"
    }

    fn is_block(&self) -> bool {
        true
    }

    fn is_inline(&self) -> bool {
        false
    }

    fn is_leaf(&self) -> bool {
        false
    }
}

impl BlockNode for Paragraph {}

impl NodeType for Heading {
    fn node_type_name(&self) -> &'static str {
        "Heading"
    }

    fn is_block(&self) -> bool {
        true
    }

    fn is_inline(&self) -> bool {
        false
    }

    fn is_leaf(&self) -> bool {
        false
    }
}

impl BlockNode for Heading {}

impl NodeType for ThematicBreak {
    fn node_type_name(&self) -> &'static str {
        "ThematicBreak"
    }

    fn is_block(&self) -> bool {
        true
    }

    fn is_inline(&self) -> bool {
        false
    }

    fn is_leaf(&self) -> bool {
        true
    }
}

impl BlockNode for ThematicBreak {}

impl NodeType for Text {
    fn node_type_name(&self) -> &'static str {
        "Text"
    }

    fn is_block(&self) -> bool {
        false
    }

    fn is_inline(&self) -> bool {
        true
    }

    fn is_leaf(&self) -> bool {
        true
    }
}

impl InlineNode for Text {}

impl NodeType for SoftBreak {
    fn node_type_name(&self) -> &'static str {
        "SoftBreak"
    }

    fn is_block(&self) -> bool {
        false
    }

    fn is_inline(&self) -> bool {
        true
    }

    fn is_leaf(&self) -> bool {
        true
    }
}

impl InlineNode for SoftBreak {}

impl NodeType for LineBreak {
    fn node_type_name(&self) -> &'static str {
        "LineBreak"
    }

    fn is_block(&self) -> bool {
        false
    }

    fn is_inline(&self) -> bool {
        true
    }

    fn is_leaf(&self) -> bool {
        true
    }
}

impl InlineNode for LineBreak {}

impl NodeType for Code {
    fn node_type_name(&self) -> &'static str {
        "Code"
    }

    fn is_block(&self) -> bool {
        false
    }

    fn is_inline(&self) -> bool {
        true
    }

    fn is_leaf(&self) -> bool {
        true
    }
}

impl InlineNode for Code {}

impl NodeType for HtmlInline {
    fn node_type_name(&self) -> &'static str {
        "HtmlInline"
    }

    fn is_block(&self) -> bool {
        false
    }

    fn is_inline(&self) -> bool {
        true
    }

    fn is_leaf(&self) -> bool {
        true
    }
}

impl InlineNode for HtmlInline {}

impl NodeType for Emph {
    fn node_type_name(&self) -> &'static str {
        "Emph"
    }

    fn is_block(&self) -> bool {
        false
    }

    fn is_inline(&self) -> bool {
        true
    }

    fn is_leaf(&self) -> bool {
        false
    }
}

impl InlineNode for Emph {}

impl NodeType for Strong {
    fn node_type_name(&self) -> &'static str {
        "Strong"
    }

    fn is_block(&self) -> bool {
        false
    }

    fn is_inline(&self) -> bool {
        true
    }

    fn is_leaf(&self) -> bool {
        false
    }
}

impl InlineNode for Strong {}

impl NodeType for Link {
    fn node_type_name(&self) -> &'static str {
        "Link"
    }

    fn is_block(&self) -> bool {
        false
    }

    fn is_inline(&self) -> bool {
        true
    }

    fn is_leaf(&self) -> bool {
        false
    }
}

impl InlineNode for Link {}

impl NodeType for Image {
    fn node_type_name(&self) -> &'static str {
        "Image"
    }

    fn is_block(&self) -> bool {
        false
    }

    fn is_inline(&self) -> bool {
        true
    }

    fn is_leaf(&self) -> bool {
        false
    }
}

impl InlineNode for Image {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_type_properties() {
        let doc = Document;
        assert!(doc.is_block());
        assert!(!doc.is_inline());
        assert!(!doc.is_leaf());
        assert_eq!(doc.node_type_name(), "Document");

        let text = Text {
            literal: "hello".to_string(),
        };
        assert!(!text.is_block());
        assert!(text.is_inline());
        assert!(text.is_leaf());
        assert_eq!(text.node_type_name(), "Text");

        let code_block = CodeBlock {
            info: "rust".to_string(),
            literal: "fn main() {}".to_string(),
        };
        assert!(code_block.is_block());
        assert!(!code_block.is_inline());
        assert!(code_block.is_leaf());
        assert_eq!(code_block.node_type_name(), "CodeBlock");
    }

    #[test]
    fn test_list_types() {
        let bullet_list = List {
            list_type: ListType::Bullet,
            delim: DelimType::None,
            start: 0,
            tight: true,
            bullet_char: '-',
        };
        assert_eq!(bullet_list.list_type, ListType::Bullet);
        assert!(bullet_list.is_block());

        let ordered_list = List {
            list_type: ListType::Ordered,
            delim: DelimType::Period,
            start: 1,
            tight: false,
            bullet_char: '\0',
        };
        assert_eq!(ordered_list.list_type, ListType::Ordered);
        assert_eq!(ordered_list.delim, DelimType::Period);
    }

    #[test]
    fn test_link_properties() {
        let link = Link {
            url: "https://example.com".to_string(),
            title: "Example".to_string(),
        };
        assert_eq!(link.url, "https://example.com");
        assert_eq!(link.title, "Example");
        assert!(link.is_inline());
        assert!(!link.is_leaf()); // Link can have children (text)
    }

    #[test]
    fn test_heading_levels() {
        let h1 = Heading { level: 1 };
        let h6 = Heading { level: 6 };
        assert_eq!(h1.level, 1);
        assert_eq!(h6.level, 6);
        assert!(h1.is_block());
        assert!(!h1.is_leaf());
    }
}
