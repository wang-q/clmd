//! Unified AST traversal module.
//!
//! This module provides iterator-based traversal and querying for the AST.

use crate::core::arena::{NodeArena, NodeId};

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
