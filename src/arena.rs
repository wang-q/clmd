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
//! ```ignore
//! use clmd::{NodeArena, TreeOps, NodeValue, Node};
//!
//! let mut arena = NodeArena::new();
//! let root = arena.alloc(Node::with_value(NodeValue::Document));
//! let paragraph = arena.alloc(Node::with_value(NodeValue::Paragraph));
//! TreeOps::append_child(&mut arena, root, paragraph);
//! ```

use crate::nodes::{NodeValue, SourcePos};

/// Node ID type - index into the arena
pub type NodeId = u32;

/// Invalid node ID (used for Option<NodeId> patterns)
pub const INVALID_NODE_ID: NodeId = u32::MAX;

/// A node in the AST with arena-based references
pub struct Node {
    /// The node value
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
    /// Create a new node with NodeValue
    pub fn with_value(value: NodeValue) -> Self {
        Self {
            value,
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
            .field("value", &self.value)
            .field("source_pos", &self.source_pos)
            .finish()
    }
}

/// Arena for allocating and managing nodes
#[derive(Debug)]
pub struct NodeArena {
    nodes: Vec<Node>,
    /// Maximum number of nodes allowed (0 = unlimited)
    max_nodes: usize,
    /// Total allocations counter
    total_allocations: usize,
}

impl NodeArena {
    /// Create a new empty arena
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            max_nodes: 0,
            total_allocations: 0,
        }
    }

    /// Create a new arena with capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            nodes: Vec::with_capacity(capacity),
            max_nodes: 0,
            total_allocations: 0,
        }
    }

    /// Create a new arena with memory limits
    ///
    /// # Arguments
    ///
    /// * `capacity` - Initial capacity for the arena
    /// * `max_nodes` - Maximum number of nodes allowed (0 = unlimited)
    ///
    /// # Example
    ///
    /// ```ignore
    /// use clmd::NodeArena;
    ///
    /// let arena = NodeArena::with_limits(100, 10000);
    /// ```
    pub fn with_limits(capacity: usize, max_nodes: usize) -> Self {
        Self {
            nodes: Vec::with_capacity(capacity),
            max_nodes,
            total_allocations: 0,
        }
    }

    /// Allocate a new node and return its ID
    ///
    /// # Panics
    ///
    /// Panics if the maximum node limit is reached (when configured).
    pub fn alloc(&mut self, node: Node) -> NodeId {
        // Check memory limit
        if self.max_nodes > 0 && self.nodes.len() >= self.max_nodes {
            panic!(
                "Arena node limit exceeded: {} nodes (max: {})",
                self.nodes.len(),
                self.max_nodes
            );
        }

        let id = self.nodes.len() as NodeId;
        self.nodes.push(node);
        self.total_allocations += 1;
        id
    }

    /// Get memory usage statistics
    ///
    /// Returns a tuple of (current_nodes, total_allocations, memory_estimate_bytes)
    ///
    /// # Example
    ///
    /// ```ignore
    /// use clmd::NodeArena;
    ///
    /// let arena = NodeArena::new();
    /// let (nodes, allocs, memory) = arena.memory_stats();
    /// ```
    pub fn memory_stats(&self) -> (usize, usize, usize) {
        let node_size = std::mem::size_of::<Node>();
        let memory_estimate = self.nodes.len() * node_size;
        (self.nodes.len(), self.total_allocations, memory_estimate)
    }

    /// Get the maximum node limit (0 = unlimited)
    pub fn max_nodes(&self) -> usize {
        self.max_nodes
    }

    /// Set the maximum node limit (0 = unlimited)
    pub fn set_max_nodes(&mut self, max_nodes: usize) {
        self.max_nodes = max_nodes;
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

    /// Returns an iterator over all nodes in the arena.
    ///
    /// The iterator yields `(NodeId, &Node)` tuples.
    pub fn iter(&self) -> impl Iterator<Item = (NodeId, &Node)> {
        self.nodes
            .iter()
            .enumerate()
            .map(|(i, node)| (i as NodeId, node))
    }

    /// Returns a mutable iterator over all nodes in the arena.
    ///
    /// The iterator yields `(NodeId, &mut Node)` tuples.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (NodeId, &mut Node)> {
        self.nodes
            .iter_mut()
            .enumerate()
            .map(|(i, node)| (i as NodeId, node))
    }

    /// Shrinks the capacity of the arena to match the current number of nodes.
    pub fn shrink_to_fit(&mut self) {
        self.nodes.shrink_to_fit();
    }

    /// Returns an iterator over all descendants of the given node.
    ///
    /// The iterator yields `NodeId`s in depth-first order.
    ///
    /// # Example
    ///
    /// ```
    /// use clmd::{Arena, Node, NodeValue, TreeOps};
    ///
    /// let mut arena = Arena::new();
    /// let root = arena.alloc(Node::with_value(NodeValue::Document));
    /// let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
    /// TreeOps::append_child(&mut arena, root, para);
    ///
    /// let descendants: Vec<_> = arena.descendants(root).collect();
    /// assert_eq!(descendants.len(), 2); // root and para
    /// ```
    pub fn descendants(&self, root: NodeId) -> DescendantIterator<'_> {
        DescendantIterator::new(self, root)
    }
}

impl Default for NodeArena {
    fn default() -> Self {
        Self::new()
    }
}

/// Tree operations for arena-based nodes
#[derive(Debug, Clone, Copy)]
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
            child.next = None; // Clear next pointer
        } else {
            // No children yet, set as first child
            let parent = arena.get_mut(parent_id);
            parent.first_child = Some(child_id);

            let child = arena.get_mut(child_id);
            child.next = None; // Clear next pointer
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

/// Iterator for traversing all descendants of a node
#[derive(Debug)]
pub struct DescendantIterator<'a> {
    arena: &'a NodeArena,
    stack: Vec<NodeId>,
}

impl<'a> DescendantIterator<'a> {
    /// Create a new descendant iterator
    fn new(arena: &'a NodeArena, root: NodeId) -> Self {
        DescendantIterator {
            arena,
            stack: vec![root],
        }
    }
}

impl<'a> Iterator for DescendantIterator<'a> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        self.stack.pop().map(|node_id| {
            // Add children to stack in reverse order so first child is processed first
            let node = self.arena.get(node_id);
            let mut child = node.last_child;
            while let Some(child_id) = child {
                self.stack.push(child_id);
                child = self.arena.get(child_id).prev;
            }
            node_id
        })
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
            node_mut.value = NodeValue::BlockQuote;
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
            crate::nodes::NodeHeading {
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
    fn test_memory_stats() {
        let mut arena = NodeArena::new();

        // Initially empty
        let (nodes, allocs, memory) = arena.memory_stats();
        assert_eq!(nodes, 0);
        assert_eq!(allocs, 0);
        assert_eq!(memory, 0);

        // Allocate some nodes
        arena.alloc(Node::with_value(NodeValue::Document));
        arena.alloc(Node::with_value(NodeValue::Paragraph));
        arena.alloc(Node::with_value(NodeValue::Paragraph));

        let (nodes, allocs, memory) = arena.memory_stats();
        assert_eq!(nodes, 3);
        assert_eq!(allocs, 3);
        assert!(memory > 0);
    }

    #[test]
    fn test_arena_with_limits() {
        let mut arena = NodeArena::with_limits(10, 5);

        assert_eq!(arena.max_nodes(), 5);

        // Should be able to allocate up to 5 nodes
        for _ in 0..5 {
            arena.alloc(Node::with_value(NodeValue::Paragraph));
        }

        assert_eq!(arena.len(), 5);

        // 6th allocation should panic
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut arena = NodeArena::with_limits(10, 5);
            for _ in 0..6 {
                arena.alloc(Node::with_value(NodeValue::Paragraph));
            }
        }));
        assert!(result.is_err());
    }

    #[test]
    fn test_set_max_nodes() {
        let mut arena = NodeArena::new();
        assert_eq!(arena.max_nodes(), 0); // Unlimited

        arena.set_max_nodes(100);
        assert_eq!(arena.max_nodes(), 100);
    }

    #[test]
    fn test_total_allocations_counter() {
        let mut arena = NodeArena::new();

        // Allocate and verify counter
        arena.alloc(Node::with_value(NodeValue::Document));
        let (_, allocs, _) = arena.memory_stats();
        assert_eq!(allocs, 1);

        arena.alloc(Node::with_value(NodeValue::Paragraph));
        let (_, allocs, _) = arena.memory_stats();
        assert_eq!(allocs, 2);

        // Counter should keep increasing even if nodes are unlinked
        arena.alloc(Node::with_value(NodeValue::Paragraph));
        let (_, allocs, _) = arena.memory_stats();
        assert_eq!(allocs, 3);
    }
}
