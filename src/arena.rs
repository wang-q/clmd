//! Arena-based node allocation for AST
//!
//! This module provides an efficient arena allocator for AST (Abstract Syntax Tree) nodes,
//! replacing the previous `Rc<RefCell<Node>>` approach with a more performant bump allocator pattern.
//!
//! # Overview
//!
//! The arena allocator provides:
//! - O(1) node allocation
//! - Cache-friendly memory layout
//! - Simple lifetime management
//! - Tree operations via NodeId references
//!
//! # Example
//!
//! ```
//! use clmd::{NodeArena, TreeOps, NodeValue, Node};
//!
//! let mut arena = NodeArena::new();
//! let root = arena.alloc(Node::with_value(NodeValue::Document));
//! let paragraph = arena.alloc(Node::with_value(NodeValue::Paragraph));
//! TreeOps::append_child(&mut arena, root, paragraph);
//! ```

use crate::node::{NodeData, NodeType, SourcePos};
use crate::node_value::NodeValue;

/// Node ID type - index into the arena
pub type NodeId = u32;

/// Invalid node ID (used for Option<NodeId> patterns)
pub const INVALID_NODE_ID: NodeId = u32::MAX;

/// A node in the AST with arena-based references
///
/// This struct maintains both the new `NodeValue` API and backward-compatible
/// `node_type`/`data` fields. When using `Node::with_value()`, the `node_type`
/// and `data` fields are automatically computed from the value.
pub struct Node {
    /// The node type (backward compatibility - computed from value)
    pub node_type: NodeType,
    /// The node data (backward compatibility - computed from value)
    pub data: NodeData,
    /// The node value (new API - primary storage)
    pub value: NodeValue,
    /// Source position information
    pub source_pos: SourcePos,
    /// Parent node ID
    pub parent: Option<NodeId>,
    /// First child node ID
    pub first_child: Option<NodeId>,
    /// Last child node ID
    pub last_child: Option<NodeId>,
    /// Next sibling node ID
    pub next: Option<NodeId>,
    /// Previous sibling node ID
    pub prev: Option<NodeId>,
}

impl Node {
    /// Create a new node with NodeValue (primary API)
    pub fn with_value(value: NodeValue) -> Self {
        let node_type = (&value).into();
        let data = (&value).into();
        Self {
            node_type,
            data,
            value,
            source_pos: SourcePos::default(),
            parent: None,
            first_child: None,
            last_child: None,
            next: None,
            prev: None,
        }
    }

    /// Get a reference to the NodeValue
    pub fn value(&self) -> &NodeValue {
        &self.value
    }

    /// Get a mutable reference to the NodeValue
    pub fn value_mut(&mut self) -> &mut NodeValue {
        &mut self.value
    }

    /// Set the NodeValue and update legacy fields
    pub fn set_value(&mut self, value: NodeValue) {
        self.node_type = (&value).into();
        self.data = (&value).into();
        self.value = value;
    }

    /// Create a new node from NodeType (legacy API, for backward compatibility)
    ///
    /// # Deprecated
    /// Use `Node::with_value()` instead.
    #[deprecated(since = "0.2.0", note = "Use Node::with_value() instead")]
    pub fn new(node_type: NodeType) -> Self {
        Self::with_value(node_type.into())
    }

    /// Create a new node with data (legacy API, for backward compatibility)
    ///
    /// # Deprecated
    /// Use `Node::with_value()` instead.
    #[deprecated(since = "0.2.0", note = "Use Node::with_value() instead")]
    pub fn with_data(node_type: NodeType, data: NodeData) -> Self {
        // Create value from node_type and data
        let value = Self::node_data_to_value(node_type, &data);
        let mut node = Self::with_value(value);
        node.node_type = node_type;
        node.data = data;
        node
    }

    /// Helper function to convert NodeType and NodeData to NodeValue
    fn node_data_to_value(node_type: NodeType, data: &NodeData) -> NodeValue {
        use crate::node_value::{
            NodeCode, NodeCodeBlock, NodeFootnoteDefinition, NodeFootnoteReference,
            NodeHeading, NodeHtmlBlock, NodeLink, NodeList, NodeTable, NodeTaskItem,
        };

        match (node_type, data) {
            (NodeType::Document, _) => NodeValue::Document,
            (NodeType::BlockQuote, _) => NodeValue::BlockQuote,
            (
                NodeType::List,
                NodeData::List {
                    list_type,
                    delim,
                    start,
                    tight,
                    bullet_char,
                },
            ) => NodeValue::List(NodeList {
                list_type: (*list_type).into(),
                marker_offset: 0,
                padding: 0,
                start: *start as usize,
                delimiter: (*delim).into(),
                bullet_char: *bullet_char as u8,
                tight: *tight,
                is_task_list: false,
            }),
            (NodeType::Item, _) => NodeValue::Item(NodeList::default()),
            (NodeType::CodeBlock, NodeData::CodeBlock { info, literal }) => {
                NodeValue::CodeBlock(NodeCodeBlock {
                    fenced: !info.is_empty(),
                    fence_char: b'`',
                    fence_length: 3,
                    fence_offset: 0,
                    info: info.clone(),
                    literal: literal.clone(),
                    closed: true,
                })
            }
            (NodeType::HtmlBlock, NodeData::HtmlBlock { literal }) => {
                NodeValue::HtmlBlock(NodeHtmlBlock {
                    block_type: 0,
                    literal: literal.clone(),
                })
            }
            (NodeType::Paragraph, _) => NodeValue::Paragraph,
            (NodeType::Heading, NodeData::Heading { level, content: _ }) => {
                NodeValue::Heading(NodeHeading {
                    level: *level as u8,
                    setext: false,
                    closed: true,
                })
            }
            (NodeType::ThematicBreak, _) => NodeValue::ThematicBreak,
            (
                NodeType::Table,
                NodeData::Table {
                    num_columns,
                    alignments,
                },
            ) => NodeValue::Table(NodeTable {
                alignments: alignments.iter().map(|a| (*a).into()).collect(),
                num_columns: *num_columns,
                num_rows: 0,
                num_nonempty_cells: 0,
            }),
            (NodeType::TableHead, _) => NodeValue::TableRow(true),
            (NodeType::TableRow, _) => NodeValue::TableRow(false),
            (NodeType::TableCell, _) => NodeValue::TableCell,
            (NodeType::Text, NodeData::Text { literal }) => {
                NodeValue::Text(literal.clone())
            }
            (NodeType::SoftBreak, _) => NodeValue::SoftBreak,
            (NodeType::LineBreak, _) => NodeValue::HardBreak,
            (NodeType::Code, NodeData::Code { literal }) => NodeValue::Code(NodeCode {
                num_backticks: 1,
                literal: literal.clone(),
            }),
            (NodeType::HtmlInline, NodeData::HtmlInline { literal }) => {
                NodeValue::HtmlInline(literal.clone())
            }
            (NodeType::Emph, _) => NodeValue::Emph,
            (NodeType::Strong, _) => NodeValue::Strong,
            (NodeType::Link, NodeData::Link { url, title }) => {
                NodeValue::Link(NodeLink {
                    url: url.clone(),
                    title: title.clone(),
                })
            }
            (NodeType::Image, NodeData::Image { url, title }) => {
                NodeValue::Image(NodeLink {
                    url: url.clone(),
                    title: title.clone(),
                })
            }
            (NodeType::Strikethrough, _) => NodeValue::Strikethrough,
            (NodeType::TaskItem, NodeData::TaskItem { checked }) => {
                NodeValue::TaskItem(NodeTaskItem {
                    symbol: if *checked { Some('x') } else { None },
                })
            }
            (NodeType::FootnoteRef, NodeData::FootnoteRef { label, ordinal }) => {
                NodeValue::FootnoteReference(NodeFootnoteReference {
                    name: label.clone(),
                    ref_num: *ordinal as u32,
                    ix: *ordinal as u32,
                })
            }
            (
                NodeType::FootnoteDef,
                NodeData::FootnoteDef {
                    label,
                    ordinal: _,
                    ref_count,
                },
            ) => NodeValue::FootnoteDefinition(NodeFootnoteDefinition {
                name: label.clone(),
                total_references: *ref_count as u32,
            }),
            _ => NodeValue::Raw(String::new()),
        }
    }
}

impl std::fmt::Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node")
            .field("value", &self.value)
            .field("source_pos", &self.source_pos)
            .finish()
    }
}

/// Arena for allocating and managing nodes
#[derive(Debug)]
pub struct NodeArena {
    nodes: Vec<Node>,
}

impl NodeArena {
    /// Create a new empty arena
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    /// Create a new arena with capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            nodes: Vec::with_capacity(capacity),
        }
    }

    /// Allocate a new node and return its ID
    pub fn alloc(&mut self, node: Node) -> NodeId {
        let id = self.nodes.len() as NodeId;
        self.nodes.push(node);
        id
    }

    /// Get a reference to a node by ID
    ///
    /// # Panics
    ///
    /// Panics if the ID is invalid (out of bounds).
    pub fn get(&self, id: NodeId) -> &Node {
        assert!(
            (id as usize) < self.nodes.len(),
            "Invalid NodeId: {} (arena has {} nodes)",
            id,
            self.nodes.len()
        );
        &self.nodes[id as usize]
    }

    /// Get a mutable reference to a node by ID
    ///
    /// # Panics
    ///
    /// Panics if the ID is invalid (out of bounds).
    pub fn get_mut(&mut self, id: NodeId) -> &mut Node {
        assert!(
            (id as usize) < self.nodes.len(),
            "Invalid NodeId: {} (arena has {} nodes)",
            id,
            self.nodes.len()
        );
        &mut self.nodes[id as usize]
    }

    /// Get a reference to a node by ID, returning None if the ID is invalid
    ///
    /// This is the safe alternative to `get()` which panics on invalid IDs.
    pub fn try_get(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(id as usize)
    }

    /// Get a mutable reference to a node by ID, returning None if the ID is invalid
    ///
    /// This is the safe alternative to `get_mut()` which panics on invalid IDs.
    pub fn try_get_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(id as usize)
    }

    /// Get the root node (document) - always ID 0
    pub fn root(&self) -> NodeId {
        0
    }

    /// Check if a node ID is valid
    pub fn is_valid(&self, id: NodeId) -> bool {
        (id as usize) < self.nodes.len()
    }

    /// Get the number of nodes in the arena
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Check if the arena is empty
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Synchronize NodeValue for all nodes from their node_type and data
    ///
    /// This method is kept for backward compatibility but is now a no-op
    /// since NodeValue is now the primary storage.
    pub fn sync_node_values(&mut self) {
        // NodeValue is now the primary storage, so this is a no-op
        // The node_type and data fields are computed from value on demand
    }
}

impl Default for NodeArena {
    fn default() -> Self {
        Self::new()
    }
}

/// Tree operations for arena-based nodes
pub struct TreeOps;

impl TreeOps {
    /// Append a child to a parent node
    pub fn append_child(arena: &mut NodeArena, parent_id: NodeId, child_id: NodeId) {
        let parent = arena.get_mut(parent_id);

        if let Some(last_child_id) = parent.last_child {
            // Link child to previous last child
            let last_child = arena.get_mut(last_child_id);
            last_child.next = Some(child_id);

            let child = arena.get_mut(child_id);
            child.prev = Some(last_child_id);
        } else {
            // No children yet, set as first child
            let parent = arena.get_mut(parent_id);
            parent.first_child = Some(child_id);
        }

        // Always update last_child and set parent
        let parent = arena.get_mut(parent_id);
        parent.last_child = Some(child_id);

        let child = arena.get_mut(child_id);
        child.parent = Some(parent_id);
    }

    /// Unlink a node from its parent and siblings
    pub fn unlink(arena: &mut NodeArena, node_id: NodeId) {
        let node = arena.get(node_id);
        let prev_id = node.prev;
        let next_id = node.next;
        let parent_id = node.parent;

        // Update previous node's next pointer
        if let Some(prev) = prev_id {
            let prev_node = arena.get_mut(prev);
            prev_node.next = next_id;
        } else if let Some(parent) = parent_id {
            // Node is first child, update parent's first_child
            let parent_node = arena.get_mut(parent);
            parent_node.first_child = next_id;
        }

        // Update next node's prev pointer
        if let Some(next) = next_id {
            let next_node = arena.get_mut(next);
            next_node.prev = prev_id;
        } else if let Some(parent) = parent_id {
            // Node is last child, update parent's last_child
            let parent_node = arena.get_mut(parent);
            parent_node.last_child = prev_id;
        }

        // Clear this node's connections
        let node = arena.get_mut(node_id);
        node.parent = None;
        node.next = None;
        node.prev = None;
    }

    /// Get the first child of a node
    pub fn first_child(arena: &NodeArena, node_id: NodeId) -> Option<NodeId> {
        arena.get(node_id).first_child
    }

    /// Get the last child of a node
    pub fn last_child(arena: &NodeArena, node_id: NodeId) -> Option<NodeId> {
        arena.get(node_id).last_child
    }

    /// Get the next sibling of a node
    pub fn next_sibling(arena: &NodeArena, node_id: NodeId) -> Option<NodeId> {
        arena.get(node_id).next
    }

    /// Get the parent of a node
    pub fn parent(arena: &NodeArena, node_id: NodeId) -> Option<NodeId> {
        arena.get(node_id).parent
    }

    /// Insert a node after a reference node (as a sibling)
    pub fn insert_after(
        arena: &mut NodeArena,
        reference_id: NodeId,
        new_node_id: NodeId,
    ) {
        let reference = arena.get(reference_id);
        let next_id = reference.next;
        let parent_id = reference.parent;

        // Update new node's connections
        let new_node = arena.get_mut(new_node_id);
        new_node.prev = Some(reference_id);
        new_node.next = next_id;
        new_node.parent = parent_id;

        // Update reference node's next pointer
        let reference = arena.get_mut(reference_id);
        reference.next = Some(new_node_id);

        // Update next node's prev pointer if exists
        if let Some(next) = next_id {
            let next_node = arena.get_mut(next);
            next_node.prev = Some(new_node_id);
        } else if let Some(parent) = parent_id {
            // Reference was the last child, update parent's last_child
            let parent_node = arena.get_mut(parent);
            parent_node.last_child = Some(new_node_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arena_alloc() {
        let mut arena = NodeArena::new();
        let node = Node::with_value(NodeValue::Document);
        let id = arena.alloc(node);

        assert_eq!(id, 0);
        assert_eq!(arena.len(), 1);
        assert!(arena.is_valid(id));
    }

    #[test]
    fn test_tree_operations() {
        let mut arena = NodeArena::new();

        // Create parent
        let parent = arena.alloc(Node::with_value(NodeValue::Document));

        // Create children
        let child1 = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let child2 = arena.alloc(Node::with_value(NodeValue::Paragraph));

        // Append children
        TreeOps::append_child(&mut arena, parent, child1);
        TreeOps::append_child(&mut arena, parent, child2);

        // Verify tree structure
        assert_eq!(arena.get(parent).first_child, Some(child1));
        assert_eq!(arena.get(parent).last_child, Some(child2));
        assert_eq!(arena.get(child1).next, Some(child2));
        assert_eq!(arena.get(child2).prev, Some(child1));
        assert_eq!(arena.get(child1).parent, Some(parent));
        assert_eq!(arena.get(child2).parent, Some(parent));
    }

    #[test]
    fn test_unlink() {
        let mut arena = NodeArena::new();

        let parent = arena.alloc(Node::with_value(NodeValue::Document));
        let child1 = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let child2 = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let child3 = arena.alloc(Node::with_value(NodeValue::Paragraph));

        TreeOps::append_child(&mut arena, parent, child1);
        TreeOps::append_child(&mut arena, parent, child2);
        TreeOps::append_child(&mut arena, parent, child3);

        // Unlink middle child
        TreeOps::unlink(&mut arena, child2);

        // Verify structure
        assert_eq!(arena.get(child1).next, Some(child3));
        assert_eq!(arena.get(child3).prev, Some(child1));
        assert_eq!(arena.get(child2).parent, None);
        assert_eq!(arena.get(child2).prev, None);
        assert_eq!(arena.get(child2).next, None);
    }

    #[test]
    fn test_try_get() {
        let mut arena = NodeArena::new();
        let node = Node::with_value(NodeValue::Document);
        let id = arena.alloc(node);

        // Valid ID should return Some
        assert!(arena.try_get(id).is_some());
        assert!(arena.try_get_mut(id).is_some());

        // Invalid ID should return None
        assert!(arena.try_get(999).is_none());
        assert!(arena.try_get_mut(999).is_none());
    }

    #[test]
    fn test_try_get_mut_modification() {
        let mut arena = NodeArena::new();
        let node = Node::with_value(NodeValue::Paragraph);
        let id = arena.alloc(node);

        // Modify through try_get_mut
        if let Some(node_mut) = arena.try_get_mut(id) {
            node_mut.set_value(NodeValue::BlockQuote);
        }

        // Verify modification
        assert!(matches!(arena.get(id).value, NodeValue::BlockQuote));
    }

    #[test]
    fn test_node_value_api() {
        let mut arena = NodeArena::new();

        // Create nodes using new API
        let doc = arena.alloc(Node::with_value(NodeValue::Document));
        let heading = arena.alloc(Node::with_value(NodeValue::Heading(
            crate::node_value::NodeHeading {
                level: 1,
                setext: false,
                closed: false,
            },
        )));

        // Verify values
        assert!(matches!(arena.get(doc).value, NodeValue::Document));
        if let NodeValue::Heading(h) = &arena.get(heading).value {
            assert_eq!(h.level, 1);
        } else {
            panic!("Expected Heading");
        }
    }

    #[test]
    fn test_backward_compatibility_node_type() {
        let mut arena = NodeArena::new();

        // Create using new API
        let doc = arena.alloc(Node::with_value(NodeValue::Document));
        let paragraph = arena.alloc(Node::with_value(NodeValue::Paragraph));

        // Verify node_type field works
        assert_eq!(arena.get(doc).node_type, NodeType::Document);
        assert_eq!(arena.get(paragraph).node_type, NodeType::Paragraph);
    }

    #[test]
    fn test_backward_compatibility_data() {
        let mut arena = NodeArena::new();

        // Create a heading with metadata
        let heading = arena.alloc(Node::with_value(NodeValue::Heading(
            crate::node_value::NodeHeading {
                level: 2,
                setext: false,
                closed: true,
            },
        )));

        // Verify data field works
        if let NodeData::Heading { level, .. } = &arena.get(heading).data {
            assert_eq!(*level, 2);
        } else {
            panic!("Expected Heading data");
        }
    }
}
