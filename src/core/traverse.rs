//! Unified AST traversal module.
//!
//! This module provides a unified API for traversing and querying the AST.
//! It consolidates functionality from the previous iterator.rs, tree.rs, and walk.rs modules.

use crate::core::arena::{NodeArena, NodeId, INVALID_NODE_ID};
use crate::core::nodes::NodeValue;
use std::collections::HashSet;

/// Maximum recursion depth for tree traversal to prevent stack overflow.
const MAX_TRAVERSE_DEPTH: usize = 1000;

/// Traversal order for generic traversal (internal use only).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TraverseOrder {
    PreOrder,
    PostOrder,
}

/// Event types for tree traversal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    /// Entering a node during traversal.
    Enter,
    /// Exiting a node during traversal.
    Exit,
}

/// A traversal event produced during tree walking.
#[derive(Debug, Clone, Copy)]
pub struct TraverseEvent {
    /// The node ID this event refers to.
    pub node_id: NodeId,
    /// The type of event (enter or exit).
    pub event_type: EventType,
}

impl TraverseEvent {
    /// Create a new enter event for the given node.
    pub fn enter(node_id: NodeId) -> Self {
        Self {
            node_id,
            event_type: EventType::Enter,
        }
    }

    /// Create a new exit event for the given node.
    pub fn exit(node_id: NodeId) -> Self {
        Self {
            node_id,
            event_type: EventType::Exit,
        }
    }
}

/// Trait for traversing the AST in various orders.
pub trait Traverse {
    /// Traverse the tree in pre-order (root, then children).
    fn traverse_pre_order<F>(&self, root: NodeId, f: F)
    where
        F: FnMut(&NodeValue);

    /// Traverse the tree in post-order (children, then root).
    fn traverse_post_order<F>(&self, root: NodeId, f: F)
    where
        F: FnMut(&NodeValue);

    /// Traverse the tree with enter/exit events.
    fn traverse_with_events<F>(&self, root: NodeId, f: F)
    where
        F: FnMut(&NodeValue, EventType);

    /// Traverse with mutable access in pre-order.
    fn traverse_pre_order_mut<F>(&mut self, root: NodeId, f: F)
    where
        F: FnMut(&mut NodeValue);

    /// Traverse with mutable access in post-order.
    fn traverse_post_order_mut<F>(&mut self, root: NodeId, f: F)
    where
        F: FnMut(&mut NodeValue);
}

impl Traverse for NodeArena {
    fn traverse_pre_order<F>(&self, root: NodeId, mut f: F)
    where
        F: FnMut(&NodeValue),
    {
        let mut visited = HashSet::new();
        self.traverse_pre_order_recursive(root, 0, &mut visited, &mut f);
    }

    fn traverse_post_order<F>(&self, root: NodeId, mut f: F)
    where
        F: FnMut(&NodeValue),
    {
        let mut visited = HashSet::new();
        self.traverse_post_order_recursive(root, 0, &mut visited, &mut f);
    }

    fn traverse_with_events<F>(&self, root: NodeId, mut f: F)
    where
        F: FnMut(&NodeValue, EventType),
    {
        let mut visited = HashSet::new();
        self.traverse_with_events_recursive(root, 0, &mut visited, &mut f);
    }

    fn traverse_pre_order_mut<F>(&mut self, root: NodeId, mut f: F)
    where
        F: FnMut(&mut NodeValue),
    {
        let mut visited = HashSet::new();
        self.traverse_pre_order_mut_recursive(root, 0, &mut visited, &mut f);
    }

    fn traverse_post_order_mut<F>(&mut self, root: NodeId, mut f: F)
    where
        F: FnMut(&mut NodeValue),
    {
        let mut visited = HashSet::new();
        self.traverse_post_order_mut_recursive(root, 0, &mut visited, &mut f);
    }
}

impl NodeArena {
    fn traverse_recursive_generic<F>(
        &self,
        node_id: NodeId,
        depth: usize,
        visited: &mut HashSet<NodeId>,
        order: TraverseOrder,
        f: &mut F,
    ) where
        F: FnMut(&NodeValue, Option<EventType>),
    {
        if depth > MAX_TRAVERSE_DEPTH {
            eprintln!(
                "Warning: Tree traversal depth exceeded maximum of {}. Stopping traversal.",
                MAX_TRAVERSE_DEPTH
            );
            return;
        }

        if node_id == INVALID_NODE_ID {
            return;
        }

        if visited.contains(&node_id) {
            eprintln!(
                "Warning: Circular reference detected at node {}. Stopping traversal.",
                node_id
            );
            return;
        }

        if let Some(node) = self.try_get(node_id) {
            visited.insert(node_id);

            if order == TraverseOrder::PreOrder {
                f(&node.value, None);
            }

            let mut child_id = node.first_child;
            while let Some(child) = child_id {
                self.traverse_recursive_generic(child, depth + 1, visited, order, f);
                child_id = self.try_get(child).and_then(|n| n.next);
            }

            if order == TraverseOrder::PostOrder {
                f(&node.value, None);
            }

            visited.remove(&node_id);
        }
    }

    fn traverse_pre_order_recursive<F>(
        &self,
        node_id: NodeId,
        depth: usize,
        visited: &mut HashSet<NodeId>,
        f: &mut F,
    ) where
        F: FnMut(&NodeValue),
    {
        self.traverse_recursive_generic(
            node_id,
            depth,
            visited,
            TraverseOrder::PreOrder,
            &mut |value, _| f(value),
        );
    }

    fn traverse_post_order_recursive<F>(
        &self,
        node_id: NodeId,
        depth: usize,
        visited: &mut HashSet<NodeId>,
        f: &mut F,
    ) where
        F: FnMut(&NodeValue),
    {
        self.traverse_recursive_generic(
            node_id,
            depth,
            visited,
            TraverseOrder::PostOrder,
            &mut |value, _| f(value),
        );
    }

    fn traverse_with_events_recursive<F>(
        &self,
        node_id: NodeId,
        depth: usize,
        visited: &mut HashSet<NodeId>,
        f: &mut F,
    ) where
        F: FnMut(&NodeValue, EventType),
    {
        if depth > MAX_TRAVERSE_DEPTH {
            eprintln!(
                "Warning: Tree traversal depth exceeded maximum of {}. Stopping traversal.",
                MAX_TRAVERSE_DEPTH
            );
            return;
        }

        if node_id == INVALID_NODE_ID {
            return;
        }

        if visited.contains(&node_id) {
            eprintln!(
                "Warning: Circular reference detected at node {}. Stopping traversal.",
                node_id
            );
            return;
        }

        if let Some(node) = self.try_get(node_id) {
            visited.insert(node_id);
            f(&node.value, EventType::Enter);

            let mut child_id = node.first_child;
            while let Some(child) = child_id {
                self.traverse_with_events_recursive(child, depth + 1, visited, f);
                child_id = self.try_get(child).and_then(|n| n.next);
            }

            f(&node.value, EventType::Exit);
            visited.remove(&node_id);
        }
    }

    fn traverse_pre_order_mut_recursive<F>(
        &mut self,
        node_id: NodeId,
        depth: usize,
        visited: &mut HashSet<NodeId>,
        f: &mut F,
    ) where
        F: FnMut(&mut NodeValue),
    {
        if depth > MAX_TRAVERSE_DEPTH {
            eprintln!(
                "Warning: Tree traversal depth exceeded maximum of {}. Stopping traversal.",
                MAX_TRAVERSE_DEPTH
            );
            return;
        }

        if node_id == INVALID_NODE_ID {
            return;
        }

        if visited.contains(&node_id) {
            eprintln!(
                "Warning: Circular reference detected at node {}. Stopping traversal.",
                node_id
            );
            return;
        }

        let child_ids: Vec<NodeId> = if let Some(node) = self.try_get(node_id) {
            let mut ids = Vec::new();
            let mut child_id = node.first_child;
            while let Some(child) = child_id {
                ids.push(child);
                child_id = self.try_get(child).and_then(|n| n.next);
            }
            ids
        } else {
            return;
        };

        visited.insert(node_id);
        if let Some(node) = self.try_get_mut(node_id) {
            f(&mut node.value);
        }

        for child_id in child_ids {
            self.traverse_pre_order_mut_recursive(child_id, depth + 1, visited, f);
        }
        visited.remove(&node_id);
    }

    fn traverse_post_order_mut_recursive<F>(
        &mut self,
        node_id: NodeId,
        depth: usize,
        visited: &mut HashSet<NodeId>,
        f: &mut F,
    ) where
        F: FnMut(&mut NodeValue),
    {
        if depth > MAX_TRAVERSE_DEPTH {
            eprintln!(
                "Warning: Tree traversal depth exceeded maximum of {}. Stopping traversal.",
                MAX_TRAVERSE_DEPTH
            );
            return;
        }

        if node_id == INVALID_NODE_ID {
            return;
        }

        if visited.contains(&node_id) {
            eprintln!(
                "Warning: Circular reference detected at node {}. Stopping traversal.",
                node_id
            );
            return;
        }

        let child_ids: Vec<NodeId> = if let Some(node) = self.try_get(node_id) {
            let mut ids = Vec::new();
            let mut child_id = node.first_child;
            while let Some(child) = child_id {
                ids.push(child);
                child_id = self.try_get(child).and_then(|n| n.next);
            }
            ids
        } else {
            return;
        };

        visited.insert(node_id);

        for child_id in child_ids {
            self.traverse_post_order_mut_recursive(child_id, depth + 1, visited, f);
        }

        if let Some(node) = self.try_get_mut(node_id) {
            f(&mut node.value);
        }

        visited.remove(&node_id);
    }
}

/// Extension trait for NodeArena to provide additional traversal iterators.
pub trait TraverseExt {
    /// Get an iterator over all descendants of the given node (depth-first).
    fn descendants_iter(&self, root: NodeId) -> DescendantIter<'_>;
    /// Get an iterator over children of the given node.
    fn children_iter(&self, node_id: NodeId) -> ChildIter<'_>;
    /// Get an iterator over ancestors of the given node.
    fn ancestors_iter(&self, node_id: NodeId) -> AncestorIter<'_>;
    /// Get an iterator over siblings of the given node (excluding itself).
    fn siblings_iter(&self, node_id: NodeId) -> SiblingIter<'_>;
}

/// Iterator over descendants in depth-first order.
#[derive(Debug)]
pub struct DescendantIter<'a> {
    arena: &'a NodeArena,
    stack: Vec<NodeId>,
}

impl<'a> Iterator for DescendantIter<'a> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        let node_id = self.stack.pop()?;

        if let Some(node) = self.arena.try_get(node_id) {
            let mut children = Vec::new();
            let mut child_id = node.first_child;
            while let Some(child) = child_id {
                children.push(child);
                child_id = self.arena.try_get(child).and_then(|n| n.next);
            }
            for child in children.into_iter().rev() {
                self.stack.push(child);
            }
        }

        Some(node_id)
    }
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

/// Iterator over siblings of a node (excluding the node itself).
#[derive(Debug)]
pub struct SiblingIter<'a> {
    arena: &'a NodeArena,
    current: Option<NodeId>,
    exclude: Option<NodeId>,
}

impl<'a> Iterator for SiblingIter<'a> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(current) = self.current {
            self.current = if let Some(node) = self.arena.try_get(current) {
                node.next
            } else {
                None
            };

            if Some(current) != self.exclude {
                return Some(current);
            }
        }
        None
    }
}

impl TraverseExt for NodeArena {
    fn descendants_iter(&self, root: NodeId) -> DescendantIter<'_> {
        let mut stack = Vec::new();
        if root != INVALID_NODE_ID {
            stack.push(root);
        }
        DescendantIter { arena: self, stack }
    }

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

    fn siblings_iter(&self, node_id: NodeId) -> SiblingIter<'_> {
        let parent = self.try_get(node_id).and_then(|n| n.parent);
        let first_child =
            parent.and_then(|p| self.try_get(p).and_then(|n| n.first_child));
        SiblingIter {
            arena: self,
            current: first_child,
            exclude: Some(node_id),
        }
    }
}
