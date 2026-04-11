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
//! use clmd::core::{NodeArena, TreeOps, Node};
//! use clmd::core::NodeValue;
//!
//! let mut arena = NodeArena::new();
//! let root = arena.alloc(Node::with_value(NodeValue::Document));
//! let paragraph = arena.alloc(Node::with_value(NodeValue::Paragraph));
//! TreeOps::append_child(&mut arena, root, paragraph);
//! ```

use crate::core::error::{ClmdError, LimitKind};
use crate::core::nodes::{NodeValue, SourcePos};

/// Node ID type - index into the arena
pub type NodeId = u32;

/// Invalid node ID (used for `Option<NodeId>` patterns)
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

    /// Try to allocate a new node and return its ID
    ///
    /// # Panics
    ///
    /// Panics if the maximum node limit is reached (when configured)
    /// or if the node ID would overflow (exceeds `u32::MAX`).
    ///
    /// For a non-panicking version, use [`try_alloc`](Self::try_alloc).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use clmd::core::{NodeArena, Node, NodeValue};
    ///
    /// let mut arena = NodeArena::new();
    /// let node = Node::with_value(NodeValue::Document);
    /// let id = arena.alloc(node);
    /// ```
    pub fn alloc(&mut self, node: Node) -> NodeId {
        self.try_alloc(node).unwrap_or_else(|e| {
            panic!("Arena allocation failed: {}", e);
        })
    }

    /// Try to allocate a new node and return its ID
    ///
    /// # Errors
    ///
    /// Returns an error if the maximum node limit is reached (when configured)
    /// or if the node ID would overflow.
    pub fn try_alloc(&mut self, node: Node) -> Result<NodeId, ClmdError> {
        // Check memory limit first (if configured)
        if self.max_nodes > 0 && self.nodes.len() >= self.max_nodes {
            return Err(ClmdError::limit_exceeded(
                LimitKind::NestingDepth,
                self.max_nodes,
                self.nodes.len(),
            ));
        }

        // Check for integer overflow (NodeId is u32)
        // Use saturating_add to safely check for overflow
        let current_len = self.nodes.len();
        if current_len >= u32::MAX as usize {
            return Err(ClmdError::limit_exceeded(
                LimitKind::NestingDepth,
                u32::MAX as usize,
                current_len,
            ));
        }

        let id = current_len as NodeId;
        self.nodes.push(node);
        self.total_allocations += 1;
        Ok(id)
    }

    /// Get a reference to a node by ID.
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

    /// Get a mutable reference to a node by ID.
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

    /// Get a reference to a node by ID, returning None if the ID is invalid.
    ///
    /// This is the safe alternative to `get()` which may panic or return
    /// a default node for invalid IDs.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use clmd::core::{NodeArena, Node, NodeValue};
    ///
    /// let mut arena = NodeArena::new();
    /// let id = arena.alloc(Node::with_value(NodeValue::Document));
    ///
    /// if let Some(node) = arena.try_get(id) {
    ///     println!("Node value: {:?}", node.value);
    /// }
    /// ```
    pub fn try_get(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(id as usize)
    }

    /// Get a mutable reference to a node by ID, returning None if the ID is invalid.
    ///
    /// This is the safe alternative to `get_mut()` which panics on invalid IDs.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use clmd::core::{NodeArena, Node, NodeValue};
    ///
    /// let mut arena = NodeArena::new();
    /// let id = arena.alloc(Node::with_value(NodeValue::Document));
    ///
    /// if let Some(node) = arena.try_get_mut(id) {
    ///     node.value = NodeValue::Paragraph;
    /// }
    /// ```
    pub fn try_get_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(id as usize)
    }

    /// Get the number of nodes in the arena
    pub fn len(&self) -> usize {
        self.nodes.len()
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

    /// Returns an iterator over all descendants of the given node.
    ///
    /// The iterator yields `NodeId`s in depth-first order.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use clmd::core::{NodeArena, TreeOps, Node};
    /// use clmd::core::NodeValue;
    ///
    /// let mut arena = NodeArena::new();
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

    /// Returns an iterator over the children of the given node.
    ///
    /// The iterator yields `NodeId`s in order from first child to last child.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use clmd::core::{NodeArena, TreeOps, Node};
    /// use clmd::core::NodeValue;
    ///
    /// let mut arena = NodeArena::new();
    /// let root = arena.alloc(Node::with_value(NodeValue::Document));
    /// let para1 = arena.alloc(Node::with_value(NodeValue::Paragraph));
    /// let para2 = arena.alloc(Node::with_value(NodeValue::Paragraph));
    /// TreeOps::append_child(&mut arena, root, para1);
    /// TreeOps::append_child(&mut arena, root, para2);
    ///
    /// let children: Vec<_> = arena.children(root).collect();
    /// assert_eq!(children.len(), 2);
    /// ```
    pub fn children(&self, parent: NodeId) -> ChildrenIterator<'_> {
        ChildrenIterator {
            arena: self,
            current: self.get(parent).first_child,
        }
    }

    /// Returns an iterator over the ancestors of the given node.
    ///
    /// The iterator yields `NodeId`s from the immediate parent up to the root.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use clmd::core::{NodeArena, TreeOps, Node};
    /// use clmd::core::NodeValue;
    ///
    /// let mut arena = NodeArena::new();
    /// let root = arena.alloc(Node::with_value(NodeValue::Document));
    /// let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
    /// TreeOps::append_child(&mut arena, root, para);
    ///
    /// let ancestors: Vec<_> = arena.ancestors(para).collect();
    /// assert_eq!(ancestors, vec![root]);
    /// ```
    pub fn ancestors(&self, node: NodeId) -> AncestorIterator<'_> {
        AncestorIterator {
            arena: self,
            current: self.get(node).parent,
        }
    }

    /// Returns the parent of the given node, if any.
    pub fn parent(&self, node: NodeId) -> Option<NodeId> {
        self.get(node).parent
    }

    /// Returns the first child of the given node, if any.
    #[inline]
    pub fn first_child(&self, node: NodeId) -> Option<NodeId> {
        self.get(node).first_child
    }

    /// Returns the last child of the given node, if any.
    #[inline]
    pub fn last_child(&self, node: NodeId) -> Option<NodeId> {
        self.get(node).last_child
    }

    /// Returns the next sibling of the given node, if any.
    #[inline]
    pub fn next_sibling(&self, node: NodeId) -> Option<NodeId> {
        self.get(node).next
    }

    /// Returns the previous sibling of the given node, if any.
    #[inline]
    pub fn prev_sibling(&self, node: NodeId) -> Option<NodeId> {
        self.get(node).prev
    }

    /// Returns an iterator over all siblings of the given node (excluding the node itself).
    ///
    /// The iterator yields `NodeId`s in order from first sibling to last sibling.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use clmd::core::{NodeArena, TreeOps, Node};
    /// use clmd::core::NodeValue;
    ///
    /// let mut arena = NodeArena::new();
    /// let root = arena.alloc(Node::with_value(NodeValue::Document));
    /// let child1 = arena.alloc(Node::with_value(NodeValue::Paragraph));
    /// let child2 = arena.alloc(Node::with_value(NodeValue::Paragraph));
    /// let child3 = arena.alloc(Node::with_value(NodeValue::Paragraph));
    /// TreeOps::append_child(&mut arena, root, child1);
    /// TreeOps::append_child(&mut arena, root, child2);
    /// TreeOps::append_child(&mut arena, root, child3);
    ///
    /// // Get siblings of child2 (should be child1 and child3)
    /// let siblings: Vec<_> = arena.siblings(child2).collect();
    /// assert_eq!(siblings, vec![child1, child3]);
    /// ```
    pub fn siblings(&self, node: NodeId) -> SiblingsIterator<'_> {
        let parent = self.get(node).parent;
        SiblingsIterator {
            arena: self,
            current: self.first_child_of_parent(parent),
            exclude: Some(node),
        }
    }

    /// Returns an iterator over all following siblings of the given node.
    ///
    /// The iterator yields `NodeId`s in order from the next sibling to the last sibling.
    pub fn following_siblings(&self, node: NodeId) -> FollowingSiblingsIterator<'_> {
        FollowingSiblingsIterator {
            arena: self,
            current: self.get(node).next,
        }
    }

    /// Returns an iterator over all preceding siblings of the given node.
    ///
    /// The iterator yields `NodeId`s in reverse order (from previous sibling to first sibling).
    pub fn preceding_siblings(&self, node: NodeId) -> PrecedingSiblingsIterator<'_> {
        PrecedingSiblingsIterator {
            arena: self,
            current: self.get(node).prev,
        }
    }

    /// Helper to get first child of a parent (handling Option<NodeId>)
    fn first_child_of_parent(&self, parent: Option<NodeId>) -> Option<NodeId> {
        parent.and_then(|p| self.get(p).first_child)
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

/// Iterator for traversing children of a node
#[derive(Debug)]
pub struct ChildrenIterator<'a> {
    arena: &'a NodeArena,
    current: Option<NodeId>,
}

impl<'a> Iterator for ChildrenIterator<'a> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        self.current.map(|node_id| {
            self.current = self.arena.get(node_id).next;
            node_id
        })
    }
}

/// Iterator for traversing ancestors of a node
#[derive(Debug)]
pub struct AncestorIterator<'a> {
    arena: &'a NodeArena,
    current: Option<NodeId>,
}

impl<'a> Iterator for AncestorIterator<'a> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        self.current.map(|node_id| {
            self.current = self.arena.get(node_id).parent;
            node_id
        })
    }
}

/// Iterator for traversing all siblings of a node
#[derive(Debug)]
pub struct SiblingsIterator<'a> {
    arena: &'a NodeArena,
    current: Option<NodeId>,
    exclude: Option<NodeId>,
}

impl<'a> Iterator for SiblingsIterator<'a> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let current = self.current?;
            self.current = self.arena.get(current).next;

            // Skip the excluded node (the original node we started from)
            if self.exclude != Some(current) {
                return Some(current);
            }

            // If we've exhausted all siblings, return None
            self.current?;
        }
    }
}

/// Iterator for traversing following siblings of a node
#[derive(Debug)]
pub struct FollowingSiblingsIterator<'a> {
    arena: &'a NodeArena,
    current: Option<NodeId>,
}

impl<'a> Iterator for FollowingSiblingsIterator<'a> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        self.current.map(|node_id| {
            self.current = self.arena.get(node_id).next;
            node_id
        })
    }
}

/// Iterator for traversing preceding siblings of a node
#[derive(Debug)]
pub struct PrecedingSiblingsIterator<'a> {
    arena: &'a NodeArena,
    current: Option<NodeId>,
}

impl<'a> Iterator for PrecedingSiblingsIterator<'a> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        self.current.map(|node_id| {
            self.current = self.arena.get(node_id).prev;
            node_id
        })
    }
}

/// Extension trait for NodeArena to provide traversal iterators.
pub trait TraverseExt {
    /// Get an iterator over children of the given node.
    fn children_iter(&self, node_id: NodeId) -> ChildIter<'_>;
    /// Get an iterator over ancestors of the given node.
    fn ancestors_iter(&self, node_id: NodeId) -> AncestorIter<'_>;
}

/// Iterator over direct children of a node.
#[derive(Debug)]
pub struct ChildIter<'a> {
    arena: &'a NodeArena,
    current: Option<NodeId>,
}

impl<'a> Iterator for ChildIter<'a> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current?;
        if let Some(node) = self.arena.try_get(current) {
            self.current = node.next;
            Some(current)
        } else {
            None
        }
    }
}

/// Iterator over ancestors of a node (rootward).
#[derive(Debug)]
pub struct AncestorIter<'a> {
    arena: &'a NodeArena,
    current: Option<NodeId>,
}

impl<'a> Iterator for AncestorIter<'a> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current?;
        if let Some(node) = self.arena.try_get(current) {
            self.current = node.parent;
            Some(current)
        } else {
            None
        }
    }
}

impl TraverseExt for NodeArena {
    fn children_iter(&self, node_id: NodeId) -> ChildIter<'_> {
        let first_child = self.try_get(node_id).and_then(|n| n.first_child);
        ChildIter {
            arena: self,
            current: first_child,
        }
    }

    fn ancestors_iter(&self, node_id: NodeId) -> AncestorIter<'_> {
        let parent = self.try_get(node_id).and_then(|n| n.parent);
        AncestorIter {
            arena: self,
            current: parent,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nodes::NodeValue;

    #[test]
    fn test_descendants_iterator() {
        let mut arena = NodeArena::new();

        // Create tree structure:
        // root
        //   ├── child1
        //   │     └── grandchild
        //   └── child2
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let child1 = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let child2 = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let grandchild = arena.alloc(Node::with_value(NodeValue::make_text("test")));

        TreeOps::append_child(&mut arena, root, child1);
        TreeOps::append_child(&mut arena, root, child2);
        TreeOps::append_child(&mut arena, child1, grandchild);

        // Test descendants iterator
        let descendants: Vec<NodeId> = arena.descendants(root).collect();
        assert_eq!(descendants.len(), 4);
        assert_eq!(descendants[0], root);
        assert_eq!(descendants[1], child1);
        assert_eq!(descendants[2], grandchild);
        assert_eq!(descendants[3], child2);
    }

    #[test]
    fn test_children_iterator() {
        let mut arena = NodeArena::new();

        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let child1 = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let child2 = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let child3 = arena.alloc(Node::with_value(NodeValue::Paragraph));

        TreeOps::append_child(&mut arena, root, child1);
        TreeOps::append_child(&mut arena, root, child2);
        TreeOps::append_child(&mut arena, root, child3);

        // Test children iterator
        let children: Vec<NodeId> = arena.children(root).collect();
        assert_eq!(children, vec![child1, child2, child3]);
    }

    #[test]
    fn test_ancestors_iterator() {
        let mut arena = NodeArena::new();

        // Create tree: root -> parent -> child
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let parent = arena.alloc(Node::with_value(NodeValue::BlockQuote));
        let child = arena.alloc(Node::with_value(NodeValue::Paragraph));

        TreeOps::append_child(&mut arena, root, parent);
        TreeOps::append_child(&mut arena, parent, child);

        // Test ancestors iterator
        let ancestors: Vec<NodeId> = arena.ancestors(child).collect();
        assert_eq!(ancestors, vec![parent, root]);
    }

    #[test]
    fn test_node_relationship_methods() {
        let mut arena = NodeArena::new();

        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let child1 = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let child2 = arena.alloc(Node::with_value(NodeValue::Paragraph));

        TreeOps::append_child(&mut arena, root, child1);
        TreeOps::append_child(&mut arena, root, child2);

        // Test relationship methods
        assert_eq!(arena.parent(child1), Some(root));
        assert_eq!(arena.parent(child2), Some(root));
        assert_eq!(arena.parent(root), None);

        assert_eq!(arena.first_child(root), Some(child1));
        assert_eq!(arena.last_child(root), Some(child2));

        assert_eq!(arena.next_sibling(child1), Some(child2));
        assert_eq!(arena.prev_sibling(child2), Some(child1));
        assert_eq!(arena.next_sibling(child2), None);
        assert_eq!(arena.prev_sibling(child1), None);
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
            crate::core::nodes::NodeHeading {
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
    fn test_siblings_iterator() {
        let mut arena = NodeArena::new();

        // Create tree: root -> [child1, child2, child3]
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let child1 = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let child2 = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let child3 = arena.alloc(Node::with_value(NodeValue::Paragraph));

        TreeOps::append_child(&mut arena, root, child1);
        TreeOps::append_child(&mut arena, root, child2);
        TreeOps::append_child(&mut arena, root, child3);

        // Test siblings of child2 (should be child1 and child3)
        let siblings: Vec<NodeId> = arena.siblings(child2).collect();
        assert_eq!(siblings, vec![child1, child3]);

        // Test siblings of child1 (should be child2 and child3)
        let siblings: Vec<NodeId> = arena.siblings(child1).collect();
        assert_eq!(siblings, vec![child2, child3]);

        // Test siblings of root (no parent, so no siblings)
        let siblings: Vec<NodeId> = arena.siblings(root).collect();
        assert!(siblings.is_empty());
    }

    #[test]
    fn test_following_siblings_iterator() {
        let mut arena = NodeArena::new();

        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let child1 = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let child2 = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let child3 = arena.alloc(Node::with_value(NodeValue::Paragraph));

        TreeOps::append_child(&mut arena, root, child1);
        TreeOps::append_child(&mut arena, root, child2);
        TreeOps::append_child(&mut arena, root, child3);

        // Test following siblings of child1
        let following: Vec<NodeId> = arena.following_siblings(child1).collect();
        assert_eq!(following, vec![child2, child3]);

        // Test following siblings of child2
        let following: Vec<NodeId> = arena.following_siblings(child2).collect();
        assert_eq!(following, vec![child3]);

        // Test following siblings of child3 (none)
        let following: Vec<NodeId> = arena.following_siblings(child3).collect();
        assert!(following.is_empty());
    }

    #[test]
    fn test_preceding_siblings_iterator() {
        let mut arena = NodeArena::new();

        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let child1 = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let child2 = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let child3 = arena.alloc(Node::with_value(NodeValue::Paragraph));

        TreeOps::append_child(&mut arena, root, child1);
        TreeOps::append_child(&mut arena, root, child2);
        TreeOps::append_child(&mut arena, root, child3);

        // Test preceding siblings of child3
        let preceding: Vec<NodeId> = arena.preceding_siblings(child3).collect();
        assert_eq!(preceding, vec![child2, child1]);

        // Test preceding siblings of child2
        let preceding: Vec<NodeId> = arena.preceding_siblings(child2).collect();
        assert_eq!(preceding, vec![child1]);

        // Test preceding siblings of child1 (none)
        let preceding: Vec<NodeId> = arena.preceding_siblings(child1).collect();
        assert!(preceding.is_empty());
    }
}
