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
//! use clmd::{NodeArena, TreeOps, NodeType, Node};
//!
//! let mut arena = NodeArena::new();
//! let root = arena.alloc(Node::new(NodeType::Document));
//! let paragraph = arena.alloc(Node::new(NodeType::Paragraph));
//! TreeOps::append_child(&mut arena, root, paragraph);
//! ```

use crate::node::{NodeData, NodeType, SourcePos};

/// Node ID type - index into the arena
pub type NodeId = u32;

/// Invalid node ID (used for Option<NodeId> patterns)
pub const INVALID_NODE_ID: NodeId = u32::MAX;

/// A node in the AST with arena-based references
pub struct Node {
    pub node_type: NodeType,
    pub data: NodeData,
    pub source_pos: SourcePos,
    pub parent: Option<NodeId>,
    pub first_child: Option<NodeId>,
    pub last_child: Option<NodeId>,
    pub next: Option<NodeId>,
    pub prev: Option<NodeId>,
}

impl Node {
    /// Create a new node
    pub fn new(node_type: NodeType) -> Self {
        let data = match node_type {
            NodeType::Document => NodeData::Document,
            NodeType::BlockQuote => NodeData::BlockQuote,
            NodeType::List => NodeData::List {
                list_type: crate::node::ListType::None,
                delim: crate::node::DelimType::None,
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
                alignment: crate::node::TableAlignment::None,
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

        Self {
            node_type,
            data,
            source_pos: SourcePos::default(),
            parent: None,
            first_child: None,
            last_child: None,
            next: None,
            prev: None,
        }
    }

    /// Create a new node with data
    pub fn with_data(node_type: NodeType, data: NodeData) -> Self {
        Self {
            node_type,
            data,
            source_pos: SourcePos::default(),
            parent: None,
            first_child: None,
            last_child: None,
            next: None,
            prev: None,
        }
    }
}

impl std::fmt::Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node")
            .field("node_type", &self.node_type)
            .field("data", &self.data)
            .field("source_pos", &self.source_pos)
            .finish()
    }
}

/// Arena for allocating and managing nodes
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
    pub fn get(&self, id: NodeId) -> &Node {
        &self.nodes[id as usize]
    }

    /// Get a mutable reference to a node by ID
    pub fn get_mut(&mut self, id: NodeId) -> &mut Node {
        &mut self.nodes[id as usize]
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
        let node = Node::new(NodeType::Document);
        let id = arena.alloc(node);

        assert_eq!(id, 0);
        assert_eq!(arena.len(), 1);
        assert!(arena.is_valid(id));
    }

    #[test]
    fn test_tree_operations() {
        let mut arena = NodeArena::new();

        // Create parent
        let parent = arena.alloc(Node::new(NodeType::Document));

        // Create children
        let child1 = arena.alloc(Node::new(NodeType::Paragraph));
        let child2 = arena.alloc(Node::new(NodeType::Paragraph));

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

        let parent = arena.alloc(Node::new(NodeType::Document));
        let child1 = arena.alloc(Node::new(NodeType::Paragraph));
        let child2 = arena.alloc(Node::new(NodeType::Paragraph));
        let child3 = arena.alloc(Node::new(NodeType::Paragraph));

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
}
