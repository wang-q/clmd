//! AST node definitions for CommonMark documents (Deprecated)
//!
//! **Deprecated**: This module is deprecated in favor of [`node_value`](crate::node_value).
//! Please use the new `NodeValue` enum which provides better type safety and ergonomics.
//!
//! This module defines the node types and structures used to represent
//! a CommonMark document as an Abstract Syntax Tree (AST).
//!
//! # Migration
//!
//! Instead of using `NodeType` and `NodeData` separately:
//! ```rust,ignore
//! // Old API (deprecated)
//! use clmd::node::{NodeType, NodeData};
//! let node = Node::new(NodeType::Paragraph);
//! ```
//!
//! Use the unified `NodeValue`:
//! ```rust
//! use clmd::node_value::NodeValue;
//! let value = NodeValue::Paragraph;
//! ```
//!
//! # Node Structure
//!
//! Each node has:
//! - A [`NodeType`] identifying the kind of element
//! - [`NodeData`] containing type-specific information
//! - Parent/child/sibling pointers for tree navigation
//!
//! # Example
//!
//! ```rust,ignore
//! use clmd::{NodeArena, NodeType, NodeData, Node};
//!
//! let mut arena = NodeArena::new();
//! let node = arena.alloc(Node::new(NodeType::Paragraph));
//! ```
#![deprecated(
    since = "0.2.0",
    note = "Use the `node_value` module instead. The `NodeValue` enum provides better type safety and ergonomics."
)]

/// Node types in the CommonMark AST
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    // Block types
    Document,
    BlockQuote,
    List,
    Item,
    CodeBlock,
    HtmlBlock,
    CustomBlock,
    Paragraph,
    Heading,
    ThematicBreak,

    // Table types (GitHub Flavored Markdown)
    Table,
    TableHead,
    TableRow,
    TableCell,

    // Footnote types
    FootnoteRef,
    FootnoteDef,

    // Inline types
    Text,
    SoftBreak,
    LineBreak,
    Code,
    HtmlInline,
    CustomInline,
    Emph,
    Strong,
    Link,
    Image,
    Strikethrough,

    // Task list item
    TaskItem,

    /// Placeholder type used during node initialization.
    ///
    /// This variant should not be used for actual AST nodes.
    /// Nodes must be properly initialized with a concrete type before use.
    None,
}

impl NodeType {
    /// Check if this is a block type
    pub fn is_block(&self) -> bool {
        matches!(
            self,
            NodeType::Document
                | NodeType::BlockQuote
                | NodeType::List
                | NodeType::Item
                | NodeType::CodeBlock
                | NodeType::HtmlBlock
                | NodeType::CustomBlock
                | NodeType::Paragraph
                | NodeType::Heading
                | NodeType::ThematicBreak
                | NodeType::Table
                | NodeType::TableHead
                | NodeType::TableRow
                | NodeType::TableCell
        )
    }

    /// Check if this is an inline type
    pub fn is_inline(&self) -> bool {
        matches!(
            self,
            NodeType::Text
                | NodeType::SoftBreak
                | NodeType::LineBreak
                | NodeType::Code
                | NodeType::HtmlInline
                | NodeType::CustomInline
                | NodeType::Emph
                | NodeType::Strong
                | NodeType::Link
                | NodeType::Image
                | NodeType::Strikethrough
        )
    }

    /// Check if this is a leaf type (cannot have children)
    pub fn is_leaf(&self) -> bool {
        matches!(
            self,
            NodeType::Text
                | NodeType::SoftBreak
                | NodeType::LineBreak
                | NodeType::Code
                | NodeType::HtmlInline
                | NodeType::CodeBlock
                | NodeType::HtmlBlock
                | NodeType::ThematicBreak
        )
    }
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

/// Source position information
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SourcePos {
    pub start_line: u32,
    pub start_column: u32,
    pub end_line: u32,
    pub end_column: u32,
}

/// Table cell alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TableAlignment {
    #[default]
    None,
    Left,
    Center,
    Right,
}

/// Node data variants
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeData {
    Document,
    BlockQuote,
    List {
        list_type: ListType,
        delim: DelimType,
        start: u32,
        tight: bool,
        /// For bullet lists, stores the bullet character (-, +, or *)
        /// This is needed to distinguish between different bullet list markers
        bullet_char: char,
    },
    Item,
    CodeBlock {
        info: String,
        literal: String,
    },
    HtmlBlock {
        literal: String,
    },
    CustomBlock {
        on_enter: String,
        on_exit: String,
    },
    Paragraph,
    Heading {
        level: u32,
        content: String,
    },
    ThematicBreak,
    // Table data
    Table {
        /// Number of columns in the table
        num_columns: usize,
        /// Alignment for each column
        alignments: Vec<TableAlignment>,
    },
    TableHead,
    TableRow,
    TableCell {
        /// Column index (0-based)
        column_index: usize,
        /// Cell alignment
        alignment: TableAlignment,
        /// Whether this is a header cell
        is_header: bool,
    },
    Text {
        literal: String,
    },
    SoftBreak,
    LineBreak,
    Code {
        literal: String,
    },
    HtmlInline {
        literal: String,
    },
    CustomInline {
        on_enter: String,
        on_exit: String,
    },
    Emph,
    Strong,
    Strikethrough,
    Link {
        url: String,
        title: String,
    },
    Image {
        url: String,
        title: String,
    },
    TaskItem {
        /// Whether the task is checked
        checked: bool,
    },
    FootnoteRef {
        /// The footnote label
        label: String,
        /// The ordinal number
        ordinal: usize,
    },
    FootnoteDef {
        /// The footnote label
        label: String,
        /// The ordinal number
        ordinal: usize,
        /// Number of references
        ref_count: usize,
    },
    None,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_type_classification() {
        assert!(NodeType::Document.is_block());
        assert!(NodeType::Paragraph.is_block());
        assert!(!NodeType::Text.is_block());

        assert!(NodeType::Text.is_inline());
        assert!(NodeType::Link.is_inline());
        assert!(!NodeType::Paragraph.is_inline());

        assert!(NodeType::Text.is_leaf());
        assert!(NodeType::CodeBlock.is_leaf());
        assert!(!NodeType::Paragraph.is_leaf());
    }

    #[test]
    fn test_list_type_variants() {
        assert_ne!(ListType::Bullet, ListType::Ordered);
        assert_ne!(ListType::Bullet, ListType::None);
        assert_ne!(ListType::Ordered, ListType::None);
    }

    #[test]
    fn test_delim_type_variants() {
        assert_ne!(DelimType::Period, DelimType::Paren);
        assert_ne!(DelimType::Period, DelimType::None);
        assert_ne!(DelimType::Paren, DelimType::None);
    }

    #[test]
    fn test_source_pos() {
        let pos = SourcePos {
            start_line: 1,
            start_column: 2,
            end_line: 3,
            end_column: 4,
        };

        assert_eq!(pos.start_line, 1);
        assert_eq!(pos.start_column, 2);
        assert_eq!(pos.end_line, 3);
        assert_eq!(pos.end_column, 4);
    }

    #[test]
    fn test_node_data_variants() {
        // Test List data
        let list_data = NodeData::List {
            list_type: ListType::None,
            delim: DelimType::None,
            start: 0,
            tight: false,
            bullet_char: '\0',
        };
        if let NodeData::List {
            list_type,
            delim,
            start,
            tight,
            bullet_char,
        } = &list_data
        {
            assert_eq!(*list_type, ListType::None);
            assert_eq!(*delim, DelimType::None);
            assert_eq!(*start, 0);
            assert!(!*tight);
            assert_eq!(*bullet_char, '\0');
        } else {
            panic!("Expected List data");
        }

        // Test Heading data
        let heading_data = NodeData::Heading {
            level: 2,
            content: "Test".to_string(),
        };
        if let NodeData::Heading { level, content } = &heading_data {
            assert_eq!(*level, 2);
            assert_eq!(content, "Test");
        } else {
            panic!("Expected Heading data");
        }

        // Test CodeBlock data
        let code_data = NodeData::CodeBlock {
            info: "rust".to_string(),
            literal: "fn main() {}".to_string(),
        };
        if let NodeData::CodeBlock { info, literal } = &code_data {
            assert_eq!(info, "rust");
            assert_eq!(literal, "fn main() {}");
        } else {
            panic!("Expected CodeBlock data");
        }
    }
}
