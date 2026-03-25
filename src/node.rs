//! AST node definitions for CommonMark documents
//!
//! This module defines the node types and structures used to represent
//! a CommonMark document as an Abstract Syntax Tree (AST).
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
//! use clmd::node::{Node, NodeType, NodeData};
//!
//! let node = Node::new(NodeType::Paragraph, NodeData::default());
//! ```

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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableAlignment {
    None,
    Left,
    Center,
    Right,
}

impl Default for TableAlignment {
    fn default() -> Self {
        TableAlignment::None
    }
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

/// A node in the AST
pub struct Node {
    pub node_type: NodeType,
    pub data: NodeData,
    pub source_pos: SourcePos,
    pub parent: RefCell<Option<std::rc::Weak<RefCell<Node>>>>,
    pub first_child: RefCell<Option<std::rc::Rc<RefCell<Node>>>>,
    pub last_child: RefCell<Option<std::rc::Rc<RefCell<Node>>>>,
    pub next: RefCell<Option<std::rc::Rc<RefCell<Node>>>>,
    pub prev: RefCell<Option<std::rc::Weak<RefCell<Node>>>>,
}

use std::cell::RefCell;
use std::rc::Rc;

impl std::fmt::Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node")
            .field("node_type", &self.node_type)
            .field("data", &self.data)
            .field("source_pos", &self.source_pos)
            .finish()
    }
}

impl Node {
    pub fn new(node_type: NodeType) -> Self {
        let data = match node_type {
            NodeType::Document => NodeData::Document,
            NodeType::BlockQuote => NodeData::BlockQuote,
            NodeType::List => NodeData::List {
                list_type: ListType::None,
                delim: DelimType::None,
                start: 0,
                tight: false,
                bullet_char: '\0',
            },
            NodeType::Item => NodeData::Item,
            NodeType::CodeBlock => NodeData::CodeBlock {
                info: String::new(),
                literal: String::new(),
            },
            NodeType::HtmlBlock => NodeData::HtmlBlock {
                literal: String::new(),
            },
            NodeType::CustomBlock => NodeData::CustomBlock {
                on_enter: String::new(),
                on_exit: String::new(),
            },
            NodeType::Paragraph => NodeData::Paragraph,
            NodeType::Heading => NodeData::Heading {
                level: 0,
                content: String::new(),
            },
            NodeType::ThematicBreak => NodeData::ThematicBreak,
            NodeType::Text => NodeData::Text {
                literal: String::new(),
            },
            NodeType::SoftBreak => NodeData::SoftBreak,
            NodeType::LineBreak => NodeData::LineBreak,
            NodeType::Code => NodeData::Code {
                literal: String::new(),
            },
            NodeType::HtmlInline => NodeData::HtmlInline {
                literal: String::new(),
            },
            NodeType::CustomInline => NodeData::CustomInline {
                on_enter: String::new(),
                on_exit: String::new(),
            },
            NodeType::Emph => NodeData::Emph,
            NodeType::Strong => NodeData::Strong,
            NodeType::Link => NodeData::Link {
                url: String::new(),
                title: String::new(),
            },
            NodeType::Image => NodeData::Image {
                url: String::new(),
                title: String::new(),
            },
            NodeType::Table => NodeData::Table {
                num_columns: 0,
                alignments: Vec::new(),
            },
            NodeType::TableHead => NodeData::TableHead,
            NodeType::TableRow => NodeData::TableRow,
            NodeType::TableCell => NodeData::TableCell {
                column_index: 0,
                alignment: TableAlignment::None,
                is_header: false,
            },
            NodeType::Strikethrough => NodeData::Strikethrough,
            NodeType::TaskItem => NodeData::TaskItem { checked: false },
            NodeType::FootnoteRef => NodeData::FootnoteRef {
                label: String::new(),
                ordinal: 0,
            },
            NodeType::FootnoteDef => NodeData::FootnoteDef {
                label: String::new(),
                ordinal: 0,
                ref_count: 0,
            },
            NodeType::None => NodeData::None,
        };

        Node {
            node_type,
            data,
            source_pos: SourcePos::default(),
            parent: RefCell::new(None),
            first_child: RefCell::new(None),
            last_child: RefCell::new(None),
            next: RefCell::new(None),
            prev: RefCell::new(None),
        }
    }

    pub fn new_with_data(node_type: NodeType, data: NodeData) -> Self {
        Node {
            node_type,
            data,
            source_pos: SourcePos::default(),
            parent: RefCell::new(None),
            first_child: RefCell::new(None),
            last_child: RefCell::new(None),
            next: RefCell::new(None),
            prev: RefCell::new(None),
        }
    }

    pub fn is_block(&self) -> bool {
        self.node_type.is_block()
    }

    pub fn is_inline(&self) -> bool {
        self.node_type.is_inline()
    }

    pub fn is_leaf(&self) -> bool {
        self.node_type.is_leaf()
    }
}

/// Append a child to a parent node
#[inline(always)]
pub fn append_child(parent: &Rc<RefCell<Node>>, child: Rc<RefCell<Node>>) {
    // Set child's parent first
    {
        let child_mut = child.borrow_mut();
        child_mut.parent.borrow_mut().replace(Rc::downgrade(parent));
    }

    // Get the last child of parent (if any)
    let last_child_opt = parent.borrow().last_child.borrow().clone();

    if let Some(last_child) = last_child_opt {
        // Link child to previous last child
        {
            let child_mut = child.borrow_mut();
            child_mut
                .prev
                .borrow_mut()
                .replace(Rc::downgrade(&last_child));
        }
        {
            let last_mut = last_child.borrow_mut();
            last_mut.next.borrow_mut().replace(child.clone());
        }
    } else {
        // No children yet, set as first child
        parent
            .borrow_mut()
            .first_child
            .borrow_mut()
            .replace(child.clone());
    }

    // Always update last_child
    parent.borrow_mut().last_child.borrow_mut().replace(child);
}

/// Prepend a child to a parent node
pub fn prepend_child(parent: &Rc<RefCell<Node>>, child: Rc<RefCell<Node>>) {
    // Set child's parent first
    *child.borrow_mut().parent.borrow_mut() = Some(Rc::downgrade(parent));

    // Get the first child of parent (if any)
    let first_child_opt = parent.borrow().first_child.borrow().clone();

    if let Some(first_child) = first_child_opt {
        // Link child to current first child
        *child.borrow_mut().next.borrow_mut() = Some(first_child.clone());
        *first_child.borrow_mut().prev.borrow_mut() = Some(Rc::downgrade(&child));
    } else {
        // No children yet, set as last child too
        *parent.borrow_mut().last_child.borrow_mut() = Some(child.clone());
    }

    // Always update first_child
    *parent.borrow_mut().first_child.borrow_mut() = Some(child);
}

/// Insert a sibling after a node
pub fn insert_after(node: &Rc<RefCell<Node>>, sibling: Rc<RefCell<Node>>) {
    // Set sibling's parent from node
    let parent_weak = node.borrow().parent.borrow().clone();
    *sibling.borrow_mut().parent.borrow_mut() = parent_weak.clone();

    // Get node's next sibling (if any)
    let next_opt = node.borrow().next.borrow().clone();

    if let Some(next) = next_opt {
        // Link sibling between node and next
        *sibling.borrow_mut().next.borrow_mut() = Some(next.clone());
        *next.borrow_mut().prev.borrow_mut() = Some(Rc::downgrade(&sibling));
    } else if let Some(parent_weak) = parent_weak {
        // Node was the last child, update parent's last_child
        if let Some(parent) = parent_weak.upgrade() {
            *parent.borrow_mut().last_child.borrow_mut() = Some(sibling.clone());
        }
    }

    // Link sibling to node
    *sibling.borrow_mut().prev.borrow_mut() = Some(Rc::downgrade(node));
    *node.borrow_mut().next.borrow_mut() = Some(sibling);
}

/// Insert a sibling before a node
pub fn insert_before(node: &Rc<RefCell<Node>>, sibling: Rc<RefCell<Node>>) {
    // Set sibling's parent from node
    let parent_weak = node.borrow().parent.borrow().clone();
    *sibling.borrow_mut().parent.borrow_mut() = parent_weak.clone();

    // Get node's previous sibling (if any)
    let prev_weak_opt = node.borrow().prev.borrow().clone();

    if let Some(prev_weak) = prev_weak_opt {
        // Link sibling between prev and node
        if let Some(prev) = prev_weak.upgrade() {
            *sibling.borrow_mut().prev.borrow_mut() = Some(Rc::downgrade(&prev));
            *prev.borrow_mut().next.borrow_mut() = Some(sibling.clone());
        }
    } else if let Some(parent_weak) = parent_weak {
        // Node was the first child, update parent's first_child
        if let Some(parent) = parent_weak.upgrade() {
            *parent.borrow_mut().first_child.borrow_mut() = Some(sibling.clone());
        }
    }

    // Link sibling to node
    *sibling.borrow_mut().next.borrow_mut() = Some(node.clone());
    *node.borrow_mut().prev.borrow_mut() = Some(Rc::downgrade(&sibling));
}

/// Unlink a node from its parent and siblings
pub fn unlink(node: &Rc<RefCell<Node>>) {
    // Get references we need before making any changes
    let prev_weak_opt = node.borrow().prev.borrow().clone();
    let next_opt = node.borrow().next.borrow().clone();
    let parent_weak_opt = node.borrow().parent.borrow().clone();

    // Update previous node's next pointer
    if let Some(ref prev_weak) = prev_weak_opt {
        if let Some(prev) = prev_weak.upgrade() {
            *prev.borrow_mut().next.borrow_mut() = next_opt.clone();
        }
    } else if let Some(parent_weak) = &parent_weak_opt {
        // Node is first child, update parent's first_child
        if let Some(parent) = parent_weak.upgrade() {
            *parent.borrow_mut().first_child.borrow_mut() = next_opt.clone();
        }
    }

    // Update next node's prev pointer
    if let Some(next) = &next_opt {
        *next.borrow_mut().prev.borrow_mut() = prev_weak_opt.clone();
    } else if let Some(parent_weak) = &parent_weak_opt {
        // Node is last child, update parent's last_child
        if let Some(parent) = parent_weak.upgrade() {
            *parent.borrow_mut().last_child.borrow_mut() =
                prev_weak_opt.as_ref().and_then(|w| w.upgrade());
        }
    }

    // Clear this node's connections
    *node.borrow_mut().parent.borrow_mut() = None;
    *node.borrow_mut().next.borrow_mut() = None;
    *node.borrow_mut().prev.borrow_mut() = None;
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
    fn test_node_creation() {
        let node = Node::new(NodeType::Paragraph);
        assert_eq!(node.node_type, NodeType::Paragraph);
        assert!(node.is_block());
    }

    #[test]
    fn test_append_child() {
        let parent = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let child = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));

        append_child(&parent, child.clone());

        assert!(parent.borrow().first_child.borrow().is_some());
        assert!(parent.borrow().last_child.borrow().is_some());
        assert!(child.borrow().parent.borrow().is_some());
    }

    #[test]
    fn test_unlink() {
        let parent = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let child = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));

        append_child(&parent, child.clone());
        unlink(&child);

        assert!(parent.borrow().first_child.borrow().is_none());
        assert!(child.borrow().parent.borrow().is_none());
    }

    #[test]
    fn test_prepend_child() {
        let parent = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let child1 = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        let child2 = Rc::new(RefCell::new(Node::new(NodeType::Heading)));

        append_child(&parent, child1.clone());
        prepend_child(&parent, child2.clone());

        // child2 should be first, child1 should be last
        assert!(Rc::ptr_eq(
            parent.borrow().first_child.borrow().as_ref().unwrap(),
            &child2
        ));
        assert!(Rc::ptr_eq(
            parent.borrow().last_child.borrow().as_ref().unwrap(),
            &child1
        ));
        assert!(child2.borrow().parent.borrow().is_some());
    }

    #[test]
    fn test_prepend_child_to_empty() {
        let parent = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let child = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));

        prepend_child(&parent, child.clone());

        assert!(parent.borrow().first_child.borrow().is_some());
        assert!(parent.borrow().last_child.borrow().is_some());
        assert!(Rc::ptr_eq(
            parent.borrow().first_child.borrow().as_ref().unwrap(),
            &child
        ));
        assert!(Rc::ptr_eq(
            parent.borrow().last_child.borrow().as_ref().unwrap(),
            &child
        ));
    }

    #[test]
    fn test_insert_after() {
        let parent = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let child1 = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        let child2 = Rc::new(RefCell::new(Node::new(NodeType::Heading)));
        let child3 = Rc::new(RefCell::new(Node::new(NodeType::BlockQuote)));

        append_child(&parent, child1.clone());
        append_child(&parent, child2.clone());
        insert_after(&child1, child3.clone());

        // Order should be: child1 -> child3 -> child2
        assert!(Rc::ptr_eq(
            child1.borrow().next.borrow().as_ref().unwrap(),
            &child3
        ));
        let child3_prev = child3
            .borrow()
            .prev
            .borrow()
            .as_ref()
            .unwrap()
            .upgrade()
            .unwrap();
        assert!(Rc::ptr_eq(&child3_prev, &child1));
        assert!(Rc::ptr_eq(
            child3.borrow().next.borrow().as_ref().unwrap(),
            &child2
        ));
        let child2_prev = child2
            .borrow()
            .prev
            .borrow()
            .as_ref()
            .unwrap()
            .upgrade()
            .unwrap();
        assert!(Rc::ptr_eq(&child2_prev, &child3));
    }

    #[test]
    fn test_insert_after_last() {
        let parent = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let child1 = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        let child2 = Rc::new(RefCell::new(Node::new(NodeType::Heading)));

        append_child(&parent, child1.clone());
        insert_after(&child1, child2.clone());

        // child2 should be last child
        assert!(Rc::ptr_eq(
            parent.borrow().last_child.borrow().as_ref().unwrap(),
            &child2
        ));
        assert!(child1.borrow().next.borrow().is_some());
        assert!(child2.borrow().prev.borrow().is_some());
    }

    #[test]
    fn test_insert_before() {
        let parent = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let child1 = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        let child2 = Rc::new(RefCell::new(Node::new(NodeType::Heading)));
        let child3 = Rc::new(RefCell::new(Node::new(NodeType::BlockQuote)));

        append_child(&parent, child1.clone());
        append_child(&parent, child2.clone());
        insert_before(&child2, child3.clone());

        // Order should be: child1 -> child3 -> child2
        assert!(Rc::ptr_eq(
            child1.borrow().next.borrow().as_ref().unwrap(),
            &child3
        ));
        let child3_prev = child3
            .borrow()
            .prev
            .borrow()
            .as_ref()
            .unwrap()
            .upgrade()
            .unwrap();
        assert!(Rc::ptr_eq(&child3_prev, &child1));
        assert!(Rc::ptr_eq(
            child3.borrow().next.borrow().as_ref().unwrap(),
            &child2
        ));
    }

    #[test]
    fn test_insert_before_first() {
        let parent = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let child1 = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        let child2 = Rc::new(RefCell::new(Node::new(NodeType::Heading)));

        append_child(&parent, child1.clone());
        insert_before(&child1, child2.clone());

        // child2 should be first child
        assert!(Rc::ptr_eq(
            parent.borrow().first_child.borrow().as_ref().unwrap(),
            &child2
        ));
        assert!(child2.borrow().next.borrow().is_some());
        assert!(child1.borrow().prev.borrow().is_some());
    }

    #[test]
    fn test_unlink_middle() {
        let parent = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let child1 = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        let child2 = Rc::new(RefCell::new(Node::new(NodeType::Heading)));
        let child3 = Rc::new(RefCell::new(Node::new(NodeType::BlockQuote)));

        append_child(&parent, child1.clone());
        append_child(&parent, child2.clone());
        append_child(&parent, child3.clone());

        unlink(&child2);

        // child1 and child3 should be linked
        assert!(Rc::ptr_eq(
            child1.borrow().next.borrow().as_ref().unwrap(),
            &child3
        ));
        let child3_prev = child3
            .borrow()
            .prev
            .borrow()
            .as_ref()
            .unwrap()
            .upgrade()
            .unwrap();
        assert!(Rc::ptr_eq(&child3_prev, &child1));
        assert!(child2.borrow().parent.borrow().is_none());
        assert!(child2.borrow().next.borrow().is_none());
        assert!(child2.borrow().prev.borrow().is_none());
    }

    #[test]
    fn test_node_new_with_data() {
        let data = NodeData::Heading {
            level: 2,
            content: "Test".to_string(),
        };
        let node = Node::new_with_data(NodeType::Heading, data.clone());

        assert_eq!(node.node_type, NodeType::Heading);
        assert_eq!(node.data, data);
    }

    #[test]
    fn test_node_is_methods() {
        let para = Node::new(NodeType::Paragraph);
        assert!(para.is_block());
        assert!(!para.is_inline());
        assert!(!para.is_leaf());

        let text = Node::new(NodeType::Text);
        assert!(!text.is_block());
        assert!(text.is_inline());
        assert!(text.is_leaf());

        let code_block = Node::new(NodeType::CodeBlock);
        assert!(code_block.is_block());
        assert!(!code_block.is_inline());
        assert!(code_block.is_leaf());
    }

    #[test]
    fn test_all_node_types() {
        // Test that all node types can be created
        let types = vec![
            NodeType::Document,
            NodeType::BlockQuote,
            NodeType::List,
            NodeType::Item,
            NodeType::CodeBlock,
            NodeType::HtmlBlock,
            NodeType::CustomBlock,
            NodeType::Paragraph,
            NodeType::Heading,
            NodeType::ThematicBreak,
            NodeType::Text,
            NodeType::SoftBreak,
            NodeType::LineBreak,
            NodeType::Code,
            NodeType::HtmlInline,
            NodeType::CustomInline,
            NodeType::Emph,
            NodeType::Strong,
            NodeType::Link,
            NodeType::Image,
            NodeType::None,
        ];

        for node_type in types {
            let node = Node::new(node_type);
            assert_eq!(node.node_type, node_type);
        }
    }

    #[test]
    fn test_node_data_variants() {
        // Test List data
        let list_node = Node::new(NodeType::List);
        if let NodeData::List {
            list_type,
            delim,
            start,
            tight,
            bullet_char,
        } = &list_node.data
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
        let heading_node = Node::new(NodeType::Heading);
        if let NodeData::Heading { level, content } = &heading_node.data {
            assert_eq!(*level, 0);
            assert_eq!(content, "");
        } else {
            panic!("Expected Heading data");
        }

        // Test CodeBlock data
        let code_node = Node::new(NodeType::CodeBlock);
        if let NodeData::CodeBlock { info, literal } = &code_node.data {
            assert_eq!(info, "");
            assert_eq!(literal, "");
        } else {
            panic!("Expected CodeBlock data");
        }
    }

    #[test]
    fn test_source_pos() {
        let mut node = Node::new(NodeType::Paragraph);
        node.source_pos = SourcePos {
            start_line: 1,
            start_column: 2,
            end_line: 3,
            end_column: 4,
        };

        assert_eq!(node.source_pos.start_line, 1);
        assert_eq!(node.source_pos.start_column, 2);
        assert_eq!(node.source_pos.end_line, 3);
        assert_eq!(node.source_pos.end_column, 4);
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
}
