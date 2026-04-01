//! Tree traversal and query traits for AST
//!
//! This module provides traits for traversing and querying the AST:
//! - `Walkable`: For bottom-up and top-down tree transformations
//! - `Queryable`: For searching and extracting information from the tree
//!
//! Inspired by Pandoc's Walk.hs and Generic.hs modules.

use crate::core::arena::{NodeArena, NodeId};
use crate::core::nodes::NodeValue;

/// Walk direction for tree traversal
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WalkDirection {
    /// Bottom-up: process children before parent
    BottomUp,
    /// Top-down: process parent before children
    TopDown,
}

/// Trait for walking/transforming the AST
///
/// This trait provides methods for traversing the AST and applying
/// transformations to nodes. It supports both bottom-up and top-down
/// traversal patterns.
///
/// Inspired by Pandoc's Walkable type class.
pub trait Walkable {
    /// Walk the tree bottom-up, applying a function to each node
    ///
    /// The function is called after processing all children.
    fn walk_bottom_up<F>(&mut self, root: NodeId, f: &mut F)
    where
        F: FnMut(NodeId, &mut NodeValue);

    /// Walk the tree top-down, applying a function to each node
    ///
    /// The function is called before processing children.
    fn walk_top_down<F>(&mut self, root: NodeId, f: &mut F)
    where
        F: FnMut(NodeId, &mut NodeValue);

    /// Walk with direction control
    fn walk_with_direction<F>(
        &mut self,
        root: NodeId,
        direction: WalkDirection,
        f: &mut F,
    ) where
        F: FnMut(NodeId, &mut NodeValue);
}

impl Walkable for NodeArena {
    fn walk_bottom_up<F>(&mut self, root: NodeId, f: &mut F)
    where
        F: FnMut(NodeId, &mut NodeValue),
    {
        self.walk_with_direction(root, WalkDirection::BottomUp, f);
    }

    fn walk_top_down<F>(&mut self, root: NodeId, f: &mut F)
    where
        F: FnMut(NodeId, &mut NodeValue),
    {
        self.walk_with_direction(root, WalkDirection::TopDown, f);
    }

    fn walk_with_direction<F>(
        &mut self,
        root: NodeId,
        direction: WalkDirection,
        f: &mut F,
    ) where
        F: FnMut(NodeId, &mut NodeValue),
    {
        match direction {
            WalkDirection::BottomUp => {
                // Process children first, then parent
                let children: Vec<NodeId> = self.children(root).collect();
                for child in children {
                    self.walk_bottom_up(child, f);
                }
                let value = &mut self.get_mut(root).value;
                f(root, value);
            }
            WalkDirection::TopDown => {
                // Process parent first, then children
                let value = &mut self.get_mut(root).value;
                f(root, value);

                // Collect children to avoid borrow issues
                let children: Vec<NodeId> = self.children(root).collect();
                for child in children {
                    self.walk_top_down(child, f);
                }
            }
        }
    }
}

/// Represents different node types for querying
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    /// Document root
    Document,
    /// Block quote
    BlockQuote,
    /// List
    List,
    /// List item
    Item,
    /// Code block
    CodeBlock,
    /// HTML block
    HtmlBlock,
    /// Paragraph
    Paragraph,
    /// Heading
    Heading,
    /// Thematic break
    ThematicBreak,
    /// Footnote definition
    FootnoteDefinition,
    /// Table
    Table,
    /// Table row
    TableRow,
    /// Table cell
    TableCell,
    /// Text
    Text,
    /// Task item
    TaskItem,
    /// Soft break
    SoftBreak,
    /// Hard break
    HardBreak,
    /// Inline code
    Code,
    /// HTML inline
    HtmlInline,
    /// Emphasis
    Emph,
    /// Strong
    Strong,
    /// Strikethrough
    Strikethrough,
    /// Superscript
    Superscript,
    /// Subscript
    Subscript,
    /// Link
    Link,
    /// Image
    Image,
    /// Footnote reference
    FootnoteReference,
    /// Math
    Math,
    /// Raw content
    Raw,
    /// Description list
    DescriptionList,
    /// Description item
    DescriptionItem,
    /// Description term
    DescriptionTerm,
    /// Description details
    DescriptionDetails,
    /// Alerts
    Alert,
    /// WikiLink
    WikiLink,
}

impl NodeType {
    /// Check if a NodeValue matches this node type
    pub fn matches(&self, value: &NodeValue) -> bool {
        match (self, value) {
            (NodeType::Document, NodeValue::Document) => true,
            (NodeType::BlockQuote, NodeValue::BlockQuote) => true,
            (NodeType::List, NodeValue::List(_)) => true,
            (NodeType::Item, NodeValue::Item(_)) => true,
            (NodeType::CodeBlock, NodeValue::CodeBlock(_)) => true,
            (NodeType::HtmlBlock, NodeValue::HtmlBlock(_)) => true,
            (NodeType::Paragraph, NodeValue::Paragraph) => true,
            (NodeType::Heading, NodeValue::Heading(_)) => true,
            (NodeType::ThematicBreak, NodeValue::ThematicBreak) => true,
            (NodeType::FootnoteDefinition, NodeValue::FootnoteDefinition(_)) => true,
            (NodeType::Table, NodeValue::Table(_)) => true,
            (NodeType::TableRow, NodeValue::TableRow(_)) => true,
            (NodeType::TableCell, NodeValue::TableCell) => true,
            (NodeType::Text, NodeValue::Text(_)) => true,
            (NodeType::TaskItem, NodeValue::TaskItem(_)) => true,
            (NodeType::SoftBreak, NodeValue::SoftBreak) => true,
            (NodeType::HardBreak, NodeValue::HardBreak) => true,
            (NodeType::Code, NodeValue::Code(_)) => true,
            (NodeType::HtmlInline, NodeValue::HtmlInline(_)) => true,
            (NodeType::Emph, NodeValue::Emph) => true,
            (NodeType::Strong, NodeValue::Strong) => true,
            (NodeType::Strikethrough, NodeValue::Strikethrough) => true,
            (NodeType::Superscript, NodeValue::Superscript) => true,
            (NodeType::Subscript, NodeValue::Subscript) => true,
            (NodeType::Link, NodeValue::Link(_)) => true,
            (NodeType::Image, NodeValue::Image(_)) => true,
            (NodeType::FootnoteReference, NodeValue::FootnoteReference(_)) => true,
            (NodeType::Math, NodeValue::Math(_)) => true,
            (NodeType::Raw, NodeValue::Raw(_)) => true,
            (NodeType::DescriptionList, NodeValue::DescriptionList) => true,
            (NodeType::DescriptionItem, NodeValue::DescriptionItem(_)) => true,
            (NodeType::DescriptionTerm, NodeValue::DescriptionTerm) => true,
            (NodeType::DescriptionDetails, NodeValue::DescriptionDetails) => true,
            (NodeType::Alert, NodeValue::Alert(_)) => true,
            (NodeType::WikiLink, NodeValue::WikiLink(_)) => true,
            _ => false,
        }
    }

    /// Check if this node type is a block element
    pub fn is_block(&self) -> bool {
        matches!(
            self,
            NodeType::Document
                | NodeType::BlockQuote
                | NodeType::List
                | NodeType::Item
                | NodeType::CodeBlock
                | NodeType::HtmlBlock
                | NodeType::Paragraph
                | NodeType::Heading
                | NodeType::ThematicBreak
                | NodeType::FootnoteDefinition
                | NodeType::Table
                | NodeType::TableRow
                | NodeType::TableCell
                | NodeType::DescriptionList
                | NodeType::DescriptionItem
                | NodeType::DescriptionTerm
                | NodeType::DescriptionDetails
                | NodeType::Alert
        )
    }

    /// Check if this node type is an inline element
    pub fn is_inline(&self) -> bool {
        matches!(
            self,
            NodeType::Text
                | NodeType::TaskItem
                | NodeType::SoftBreak
                | NodeType::HardBreak
                | NodeType::Code
                | NodeType::HtmlInline
                | NodeType::Emph
                | NodeType::Strong
                | NodeType::Strikethrough
                | NodeType::Superscript
                | NodeType::Subscript
                | NodeType::Link
                | NodeType::Image
                | NodeType::FootnoteReference
                | NodeType::Math
                | NodeType::Raw
                | NodeType::WikiLink
        )
    }
}

/// Trait for querying the AST
///
/// This trait provides methods for searching and extracting information
/// from the AST without modifying it.
///
/// Inspired by Pandoc's query and listify functions.
pub trait Queryable {
    /// Query the tree and collect results
    ///
    /// The query function is applied to each node. If it returns Some(value),
    /// the value is collected into the result vector.
    fn query<T, F>(&self, root: NodeId, f: &mut F) -> Vec<T>
    where
        F: FnMut(NodeId, &NodeValue) -> Option<T>;

    /// Query with early termination
    ///
    /// Stops traversing as soon as the predicate returns true.
    fn query_first<T, F>(&self, root: NodeId, f: &mut F) -> Option<T>
    where
        F: FnMut(NodeId, &NodeValue) -> Option<T>;

    /// Check if any node matches the predicate
    fn any<F>(&self, root: NodeId, f: &mut F) -> bool
    where
        F: FnMut(NodeId, &NodeValue) -> bool;

    /// Check if all nodes match the predicate
    fn all<F>(&self, root: NodeId, f: &mut F) -> bool
    where
        F: FnMut(NodeId, &NodeValue) -> bool;

    /// Count nodes matching the predicate
    fn count<F>(&self, root: NodeId, f: &mut F) -> usize
    where
        F: FnMut(NodeId, &NodeValue) -> bool;

    /// Find all nodes of a specific type
    fn find_by_type(&self, root: NodeId, node_type: NodeType) -> Vec<NodeId>;

    /// Get all text content as a single string
    fn extract_text(&self, root: NodeId) -> String;

    /// Find all links in the document
    fn find_links(&self, root: NodeId) -> Vec<NodeId>;

    /// Find all images in the document
    fn find_images(&self, root: NodeId) -> Vec<NodeId>;

    /// Find all headings in the document
    fn find_headings(&self, root: NodeId) -> Vec<NodeId>;

    /// Find all code blocks in the document
    fn find_code_blocks(&self, root: NodeId) -> Vec<NodeId>;

    /// Get the document structure as a list of headings with levels
    fn get_heading_structure(&self, root: NodeId) -> Vec<(usize, String)>;

    /// Check if the document contains any block elements
    fn has_blocks(&self, root: NodeId) -> bool;

    /// Check if the document contains any inline elements
    fn has_inlines(&self, root: NodeId) -> bool;
}

impl Queryable for NodeArena {
    fn query<T, F>(&self, root: NodeId, f: &mut F) -> Vec<T>
    where
        F: FnMut(NodeId, &NodeValue) -> Option<T>,
    {
        let mut results = Vec::new();
        self.query_recursive(root, f, &mut results);
        results
    }

    fn query_first<T, F>(&self, root: NodeId, f: &mut F) -> Option<T>
    where
        F: FnMut(NodeId, &NodeValue) -> Option<T>,
    {
        // Check current node
        let value = &self.get(root).value;
        if let Some(result) = f(root, value) {
            return Some(result);
        }

        // Recursively check children
        let children: Vec<NodeId> = self.children(root).collect();
        for child in children {
            if let Some(result) = self.query_first(child, f) {
                return Some(result);
            }
        }

        None
    }

    fn any<F>(&self, root: NodeId, f: &mut F) -> bool
    where
        F: FnMut(NodeId, &NodeValue) -> bool,
    {
        self.query_first(root, &mut |id, value| {
            if f(id, value) {
                Some(())
            } else {
                None
            }
        })
        .is_some()
    }

    fn all<F>(&self, root: NodeId, f: &mut F) -> bool
    where
        F: FnMut(NodeId, &NodeValue) -> bool,
    {
        !self.any(root, &mut |id, value| !f(id, value))
    }

    fn count<F>(&self, root: NodeId, f: &mut F) -> usize
    where
        F: FnMut(NodeId, &NodeValue) -> bool,
    {
        self.query(root, &mut |id, value| {
            if f(id, value) {
                Some(())
            } else {
                None
            }
        })
        .len()
    }

    fn find_by_type(&self, root: NodeId, node_type: NodeType) -> Vec<NodeId> {
        self.query(root, &mut |id, value| {
            if node_type.matches(value) {
                Some(id)
            } else {
                None
            }
        })
    }

    fn extract_text(&self, root: NodeId) -> String {
        let texts: Vec<String> = self.query(root, &mut |_, value| {
            if let NodeValue::Text(text) = value {
                Some(text.to_string())
            } else {
                None
            }
        });
        texts.join("")
    }

    fn find_links(&self, root: NodeId) -> Vec<NodeId> {
        self.find_by_type(root, NodeType::Link)
    }

    fn find_images(&self, root: NodeId) -> Vec<NodeId> {
        self.find_by_type(root, NodeType::Image)
    }

    fn find_headings(&self, root: NodeId) -> Vec<NodeId> {
        self.find_by_type(root, NodeType::Heading)
    }

    fn find_code_blocks(&self, root: NodeId) -> Vec<NodeId> {
        self.find_by_type(root, NodeType::CodeBlock)
    }

    fn get_heading_structure(&self, root: NodeId) -> Vec<(usize, String)> {
        self.query(root, &mut |id, value| {
            if let NodeValue::Heading(heading) = value {
                let text = self.extract_text(id);
                Some((heading.level as usize, text))
            } else {
                None
            }
        })
    }

    fn has_blocks(&self, root: NodeId) -> bool {
        self.any(root, &mut |_, value| {
            matches!(
                value,
                NodeValue::BlockQuote
                    | NodeValue::List(_)
                    | NodeValue::CodeBlock(_)
                    | NodeValue::HtmlBlock(_)
                    | NodeValue::Paragraph
                    | NodeValue::Heading(_)
                    | NodeValue::Table(_)
            )
        })
    }

    fn has_inlines(&self, root: NodeId) -> bool {
        self.any(root, &mut |_, value| {
            matches!(
                value,
                NodeValue::Text(_)
                    | NodeValue::Code(_)
                    | NodeValue::Emph
                    | NodeValue::Strong
                    | NodeValue::Link(_)
                    | NodeValue::Image(_)
            )
        })
    }
}

impl NodeArena {
    /// Helper method for recursive querying
    fn query_recursive<T, F>(&self, node: NodeId, f: &mut F, results: &mut Vec<T>)
    where
        F: FnMut(NodeId, &NodeValue) -> Option<T>,
    {
        let value = &self.get(node).value;
        if let Some(result) = f(node, value) {
            results.push(result);
        }

        let children: Vec<NodeId> = self.children(node).collect();
        for child in children {
            self.query_recursive(child, f, results);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::arena::{Node, TreeOps};
    use crate::core::nodes::{NodeHeading, NodeLink};

    #[test]
    fn test_walkable_bottom_up() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text1 = arena.alloc(Node::with_value(NodeValue::make_text("Hello ")));
        let text2 = arena.alloc(Node::with_value(NodeValue::make_text("world")));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, text1);
        TreeOps::append_child(&mut arena, para, text2);

        // Collect visit order
        let mut visit_order = Vec::new();
        arena.walk_bottom_up(root, &mut |_id, value| {
            let name = match value {
                NodeValue::Document => "Document",
                NodeValue::Paragraph => "Paragraph",
                NodeValue::Text(_) => "Text",
                _ => "Other",
            };
            visit_order.push(name.to_string());
        });

        // Bottom-up: children before parent
        assert_eq!(visit_order, vec!["Text", "Text", "Paragraph", "Document"]);
    }

    #[test]
    fn test_walkable_top_down() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text1 = arena.alloc(Node::with_value(NodeValue::make_text("Hello ")));
        let text2 = arena.alloc(Node::with_value(NodeValue::make_text("world")));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, text1);
        TreeOps::append_child(&mut arena, para, text2);

        // Collect visit order
        let mut visit_order = Vec::new();
        arena.walk_top_down(root, &mut |_id, value| {
            let name = match value {
                NodeValue::Document => "Document",
                NodeValue::Paragraph => "Paragraph",
                NodeValue::Text(_) => "Text",
                _ => "Other",
            };
            visit_order.push(name.to_string());
        });

        // Top-down: parent before children
        assert_eq!(visit_order, vec!["Document", "Paragraph", "Text", "Text"]);
    }

    #[test]
    fn test_walkable_transform() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("hello world")));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, text);

        // Transform text to uppercase
        arena.walk_bottom_up(root, &mut |_id, value| {
            if let NodeValue::Text(ref mut text) = value {
                *text = text.to_uppercase().into_boxed_str();
            }
        });

        // Verify transformation
        if let NodeValue::Text(text) = &arena.get(text).value {
            assert_eq!(text.as_ref(), "HELLO WORLD");
        }
    }

    #[test]
    fn test_queryable_query() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para1 = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let para2 = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("Hello")));

        TreeOps::append_child(&mut arena, root, para1);
        TreeOps::append_child(&mut arena, root, para2);
        TreeOps::append_child(&mut arena, para1, text);

        // Query for all paragraphs
        let paragraphs: Vec<NodeId> = arena.query(root, &mut |id, value| {
            if matches!(value, NodeValue::Paragraph) {
                Some(id)
            } else {
                None
            }
        });

        assert_eq!(paragraphs.len(), 2);
        assert!(paragraphs.contains(&para1));
        assert!(paragraphs.contains(&para2));
    }

    #[test]
    fn test_queryable_query_first() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para1 = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let para2 = arena.alloc(Node::with_value(NodeValue::Paragraph));

        TreeOps::append_child(&mut arena, root, para1);
        TreeOps::append_child(&mut arena, root, para2);

        // Find first paragraph
        let first_para = arena.query_first(root, &mut |id, value| {
            if matches!(value, NodeValue::Paragraph) {
                Some(id)
            } else {
                None
            }
        });

        assert_eq!(first_para, Some(para1));
    }

    #[test]
    fn test_queryable_any() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("Hello")));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, text);

        // Check if any node is a paragraph
        assert!(arena.any(root, &mut |_, value| {
            matches!(value, NodeValue::Paragraph)
        }));

        // Check if any node is a heading (should be false)
        assert!(!arena.any(root, &mut |_, value| {
            matches!(value, NodeValue::Heading(_))
        }));
    }

    #[test]
    fn test_queryable_all() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("Hello")));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, text);

        // All nodes should be valid (non-empty check)
        assert!(arena.all(root, &mut |_, _| true));

        // Not all nodes are paragraphs
        assert!(!arena.all(root, &mut |_, value| {
            matches!(value, NodeValue::Paragraph)
        }));
    }

    #[test]
    fn test_queryable_count() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para1 = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let para2 = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("Hello")));

        TreeOps::append_child(&mut arena, root, para1);
        TreeOps::append_child(&mut arena, root, para2);
        TreeOps::append_child(&mut arena, para1, text);

        // Count paragraphs
        let para_count =
            arena.count(root, &mut |_, value| matches!(value, NodeValue::Paragraph));
        assert_eq!(para_count, 2);

        // Count text nodes
        let text_count =
            arena.count(root, &mut |_, value| matches!(value, NodeValue::Text(_)));
        assert_eq!(text_count, 1);
    }

    #[test]
    fn test_queryable_find_by_type() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para1 = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let para2 = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("Hello")));

        TreeOps::append_child(&mut arena, root, para1);
        TreeOps::append_child(&mut arena, root, para2);
        TreeOps::append_child(&mut arena, para1, text);

        // Find all paragraphs
        let paragraphs = arena.find_by_type(root, NodeType::Paragraph);
        assert_eq!(paragraphs.len(), 2);
        assert!(paragraphs.contains(&para1));
        assert!(paragraphs.contains(&para2));

        // Find all text nodes
        let texts = arena.find_by_type(root, NodeType::Text);
        assert_eq!(texts.len(), 1);
        assert!(texts.contains(&text));
    }

    #[test]
    fn test_queryable_extract_text() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text1 = arena.alloc(Node::with_value(NodeValue::make_text("Hello ")));
        let text2 = arena.alloc(Node::with_value(NodeValue::make_text("world")));
        let text3 = arena.alloc(Node::with_value(NodeValue::make_text("!")));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, text1);
        TreeOps::append_child(&mut arena, para, text2);
        TreeOps::append_child(&mut arena, para, text3);

        // Extract all text
        let extracted = arena.extract_text(root);
        assert_eq!(extracted, "Hello world!");
    }

    #[test]
    fn test_node_type_matches() {
        assert!(NodeType::Document.matches(&NodeValue::Document));
        assert!(NodeType::Paragraph.matches(&NodeValue::Paragraph));
        assert!(NodeType::Text.matches(&NodeValue::make_text("test")));
        assert!(NodeType::Heading.matches(&NodeValue::Heading(NodeHeading {
            level: 1,
            setext: false,
            closed: false,
        })));

        assert!(!NodeType::Paragraph.matches(&NodeValue::Document));
        assert!(!NodeType::Text.matches(&NodeValue::Paragraph));
    }
}
