//! Unified AST traversal module.
//!
//! This module provides a unified API for traversing and querying the AST.
//! It consolidates functionality from the previous iterator.rs, tree.rs, and walk.rs modules.
//!
//! # Example
//!
//! ```ignore
//! use clmd::core::traverse::{Traverse, Query};
//! use clmd::core::arena::NodeArena;
//!
//! // Traverse the tree
//! arena.traverse_pre_order(root_id, |node| {
//!     println!("Visiting: {:?}", node.value());
//! });
//!
//! // Query for specific nodes
//! let links: Vec<NodeId> = arena.query(root_id)
//!     .filter(|id| arena.get(*id).value().is_link())
//!     .collect();
//! ```

use crate::core::arena::{NodeArena, NodeId, INVALID_NODE_ID};
use crate::core::error::ClmdError;
use crate::core::nodes::NodeValue;
use std::collections::HashSet;

/// Maximum recursion depth for tree traversal to prevent stack overflow.
///
/// This limit is set to 1000, which should be sufficient for most documents
/// while preventing stack overflow from malicious or deeply nested input.
/// For context, a depth of 1000 would mean ~1000 nested blockquotes/lists,
/// which is extremely rare in practice.
const MAX_TRAVERSE_DEPTH: usize = 1000;

/// Traversal order for generic traversal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraverseOrder {
    /// Pre-order: visit node before children
    PreOrder,
    /// Post-order: visit children before node
    PostOrder,
}

/// Context for tree traversal with cycle detection and depth tracking.
#[derive(Debug)]
pub struct TraverseContext {
    /// Set of currently visited node IDs (for cycle detection).
    visited: HashSet<NodeId>,
    /// Current traversal depth.
    depth: usize,
    /// Maximum allowed depth.
    max_depth: usize,
}

impl TraverseContext {
    /// Create a new traverse context with the specified maximum depth.
    pub fn new(max_depth: usize) -> Self {
        Self {
            visited: HashSet::new(),
            depth: 0,
            max_depth,
        }
    }

    /// Create a new traverse context with default maximum depth.
    pub fn with_default_depth() -> Self {
        Self::new(MAX_TRAVERSE_DEPTH)
    }

    /// Attempt to enter a node, checking for cycles and depth limits.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The node has already been visited (circular reference)
    /// - The maximum depth has been exceeded
    pub fn enter_node(&mut self, node_id: NodeId) -> Result<(), ClmdError> {
        if self.visited.contains(&node_id) {
            return Err(ClmdError::circular_reference(format!(
                "Node {} has already been visited (circular reference detected)",
                node_id
            )));
        }

        self.depth += 1;
        if self.depth > self.max_depth {
            return Err(ClmdError::limit_exceeded(
                crate::core::error::LimitKind::NestingDepth,
                self.max_depth,
                self.depth,
            ));
        }

        self.visited.insert(node_id);
        Ok(())
    }

    /// Exit a node, removing it from the visited set.
    pub fn exit_node(&mut self, node_id: NodeId) {
        self.visited.remove(&node_id);
        self.depth -= 1;
    }

    /// Get the current traversal depth.
    pub fn depth(&self) -> usize {
        self.depth
    }

    /// Check if the given node has been visited.
    pub fn is_visited(&self, node_id: NodeId) -> bool {
        self.visited.contains(&node_id)
    }
}

/// Event types for tree traversal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    /// Entering a node.
    Enter,
    /// Exiting a node.
    Exit,
}

/// A traversal event.
#[derive(Debug, Clone, Copy)]
pub struct TraverseEvent {
    /// The node ID.
    pub node_id: NodeId,
    /// The event type.
    pub event_type: EventType,
}

impl TraverseEvent {
    /// Create a new enter event.
    pub fn enter(node_id: NodeId) -> Self {
        Self {
            node_id,
            event_type: EventType::Enter,
        }
    }

    /// Create a new exit event.
    pub fn exit(node_id: NodeId) -> Self {
        Self {
            node_id,
            event_type: EventType::Exit,
        }
    }
}

/// Trait for traversing the AST.
pub trait Traverse {
    /// Traverse the tree in pre-order (root, then children).
    fn traverse_pre_order<F>(&self, root: NodeId, f: F)
    where
        F: FnMut(&NodeValue);

    /// Traverse the tree in post-order (children, then root).
    fn traverse_post_order<F>(&self, root: NodeId, f: F)
    where
        F: FnMut(&NodeValue);

    /// Traverse the tree with events (enter/exit).
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
    /// Generic recursive traversal implementation.
    ///
    /// This method consolidates the common traversal logic to reduce code duplication.
    /// It handles depth limits, cycle detection, and delegates to the callback at appropriate times.
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

        // Check for circular reference
        if visited.contains(&node_id) {
            eprintln!(
                "Warning: Circular reference detected at node {}. Stopping traversal.",
                node_id
            );
            return;
        }

        if let Some(node) = self.try_get(node_id) {
            visited.insert(node_id);

            // Pre-order: visit node before children
            if order == TraverseOrder::PreOrder {
                f(&node.value, None);
            }

            // Traverse children using the linked list structure
            let mut child_id = node.first_child;
            while let Some(child) = child_id {
                self.traverse_recursive_generic(child, depth + 1, visited, order, f);
                child_id = self.try_get(child).and_then(|n| n.next);
            }

            // Post-order: visit node after children
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

        // Check for circular reference
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

            // Traverse children using the linked list structure
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

        // Check for circular reference
        if visited.contains(&node_id) {
            eprintln!(
                "Warning: Circular reference detected at node {}. Stopping traversal.",
                node_id
            );
            return;
        }

        // Collect child IDs first to avoid borrow issues
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

        // Apply function to current node
        visited.insert(node_id);
        if let Some(node) = self.try_get_mut(node_id) {
            f(&mut node.value);
        }

        // Now recurse into children
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

        // Check for circular reference
        if visited.contains(&node_id) {
            eprintln!(
                "Warning: Circular reference detected at node {}. Stopping traversal.",
                node_id
            );
            return;
        }

        // Collect child IDs first to avoid borrow issues
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

        // First recurse into children
        for child_id in child_ids {
            self.traverse_post_order_mut_recursive(child_id, depth + 1, visited, f);
        }

        // Then apply function to current node
        if let Some(node) = self.try_get_mut(node_id) {
            f(&mut node.value);
        }

        visited.remove(&node_id);
    }
}

/// Trait for querying the AST.
pub trait Query {
    /// Check if any node matches the predicate.
    fn any<F>(&self, root: NodeId, predicate: F) -> bool
    where
        F: Fn(&NodeValue) -> bool;

    /// Check if all nodes match the predicate.
    fn all<F>(&self, root: NodeId, predicate: F) -> bool
    where
        F: Fn(&NodeValue) -> bool;

    /// Count nodes matching the predicate.
    fn count<F>(&self, root: NodeId, predicate: F) -> usize
    where
        F: Fn(&NodeValue) -> bool;

    /// Find the first node matching the predicate.
    fn find_first<F>(&self, root: NodeId, predicate: F) -> Option<NodeId>
    where
        F: Fn(&NodeValue) -> bool;

    /// Find all nodes matching the predicate.
    fn find_all<F>(&self, root: NodeId, predicate: F) -> Vec<NodeId>
    where
        F: Fn(&NodeValue) -> bool;
}

impl Query for NodeArena {
    fn any<F>(&self, root: NodeId, predicate: F) -> bool
    where
        F: Fn(&NodeValue) -> bool,
    {
        let mut found = false;
        self.traverse_pre_order(root, |value| {
            if predicate(value) {
                found = true;
            }
        });
        found
    }

    fn all<F>(&self, root: NodeId, predicate: F) -> bool
    where
        F: Fn(&NodeValue) -> bool,
    {
        let mut all_match = true;
        self.traverse_pre_order(root, |value| {
            if !predicate(value) {
                all_match = false;
            }
        });
        all_match
    }

    fn count<F>(&self, root: NodeId, predicate: F) -> usize
    where
        F: Fn(&NodeValue) -> bool,
    {
        let mut count = 0;
        self.traverse_pre_order(root, |value| {
            if predicate(value) {
                count += 1;
            }
        });
        count
    }

    fn find_first<F>(&self, root: NodeId, predicate: F) -> Option<NodeId>
    where
        F: Fn(&NodeValue) -> bool,
    {
        self.find_first_recursive(root, 0, &predicate)
    }

    fn find_all<F>(&self, root: NodeId, predicate: F) -> Vec<NodeId>
    where
        F: Fn(&NodeValue) -> bool,
    {
        let mut result = Vec::new();
        self.find_all_recursive(root, 0, &predicate, &mut result);
        result
    }
}

impl NodeArena {
    fn find_first_recursive<F>(
        &self,
        node_id: NodeId,
        depth: usize,
        predicate: &F,
    ) -> Option<NodeId>
    where
        F: Fn(&NodeValue) -> bool,
    {
        if depth > MAX_TRAVERSE_DEPTH {
            eprintln!(
                "Warning: Tree traversal depth exceeded maximum of {}. Stopping traversal.",
                MAX_TRAVERSE_DEPTH
            );
            return None;
        }

        if node_id == INVALID_NODE_ID {
            return None;
        }

        if let Some(node) = self.try_get(node_id) {
            if predicate(&node.value) {
                return Some(node_id);
            }

            // Search children using the linked list structure
            let mut child_id = node.first_child;
            while let Some(child) = child_id {
                if let Some(found) =
                    self.find_first_recursive(child, depth + 1, predicate)
                {
                    return Some(found);
                }
                child_id = self.try_get(child).and_then(|n| n.next);
            }
        }

        None
    }

    fn find_all_recursive<F>(
        &self,
        node_id: NodeId,
        depth: usize,
        predicate: &F,
        result: &mut Vec<NodeId>,
    ) where
        F: Fn(&NodeValue) -> bool,
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

        if let Some(node) = self.try_get(node_id) {
            if predicate(&node.value) {
                result.push(node_id);
            }

            // Search children using the linked list structure
            let mut child_id = node.first_child;
            while let Some(child) = child_id {
                self.find_all_recursive(child, depth + 1, predicate, result);
                child_id = self.try_get(child).and_then(|n| n.next);
            }
        }
    }
}

/// Iterator for traversing the tree with events.
#[derive(Debug)]
pub struct EventIterator<'a> {
    arena: &'a NodeArena,
    stack: Vec<(NodeId, EventType)>,
}

impl<'a> EventIterator<'a> {
    /// Create a new event iterator starting from the given root.
    pub fn new(arena: &'a NodeArena, root: NodeId) -> Self {
        let mut stack = Vec::new();
        if root != INVALID_NODE_ID {
            stack.push((root, EventType::Enter));
        }
        Self { arena, stack }
    }
}

impl<'a> Iterator for EventIterator<'a> {
    type Item = TraverseEvent;

    fn next(&mut self) -> Option<Self::Item> {
        let (node_id, event_type) = self.stack.pop()?;

        if event_type == EventType::Enter {
            // Push exit event first (will be processed after children)
            self.stack.push((node_id, EventType::Exit));

            // Push children in reverse order so they're processed left-to-right
            if let Some(node) = self.arena.try_get(node_id) {
                // Collect all children first
                let mut children = Vec::new();
                let mut child_id = node.first_child;
                while let Some(child) = child_id {
                    children.push(child);
                    child_id = self.arena.try_get(child).and_then(|n| n.next);
                }
                // Push in reverse order
                for child in children.into_iter().rev() {
                    self.stack.push((child, EventType::Enter));
                }
            }
        }

        Some(TraverseEvent {
            node_id,
            event_type,
        })
    }
}

/// Extension trait for NodeArena to provide additional traversal methods.
pub trait TraverseExt {
    /// Get an iterator over all descendants.
    fn descendants_iter(&self, root: NodeId) -> DescendantIter<'_>;

    /// Get an iterator over all children.
    fn children_iter(&self, node_id: NodeId) -> ChildIter<'_>;

    /// Get an iterator over all ancestors.
    fn ancestors_iter(&self, node_id: NodeId) -> AncestorIter<'_>;

    /// Get an iterator over siblings.
    fn siblings_iter(&self, node_id: NodeId) -> SiblingIter<'_>;
}

/// Iterator over descendants.
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
            // Push children in reverse order for left-to-right traversal
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

/// Iterator over children.
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

/// Iterator over ancestors.
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

/// Iterator over siblings.
#[derive(Debug)]
pub struct SiblingIter<'a> {
    arena: &'a NodeArena,
    #[allow(dead_code)]
    parent_first_child: Option<NodeId>,
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
        let parent_first_child =
            parent.and_then(|p| self.try_get(p).and_then(|n| n.first_child));
        SiblingIter {
            arena: self,
            parent_first_child,
            current: parent_first_child,
            exclude: Some(node_id),
        }
    }
}

// ============================================================================
// Iterator module integration (from iterator.rs)
// ============================================================================

/// Event type for tree iteration (compatible with iterator.rs)
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IteratorEventType {
    /// No event (initial state)
    None,
    /// Iteration complete
    Done,
    /// Entering a node
    Enter,
    /// Exiting a node
    Exit,
}

/// Iterator for traversing the Arena-based AST (from iterator.rs)
#[derive(Debug)]
pub struct ArenaNodeIterator<'a> {
    arena: &'a NodeArena,
    root: NodeId,
    current: Option<NodeId>,
    event_type: IteratorEventType,
}

impl<'a> ArenaNodeIterator<'a> {
    /// Create a new iterator for the given arena and root node
    pub fn new(arena: &'a NodeArena, root: NodeId) -> Self {
        ArenaNodeIterator {
            arena,
            root,
            current: None,
            event_type: IteratorEventType::None,
        }
    }

    /// Advance the iterator and return the next event type
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> IteratorEventType {
        self.advance()
    }

    /// Get the current node ID
    pub fn get_node(&self) -> Option<NodeId> {
        self.current
    }

    /// Get the current event type
    pub fn get_event_type(&self) -> IteratorEventType {
        self.event_type
    }

    /// Reset the iterator to a specific node and event type
    pub fn reset(&mut self, current: NodeId, event_type: IteratorEventType) {
        self.current = Some(current);
        self.event_type = event_type;
    }

    /// Internal method to advance the iterator
    fn advance(&mut self) -> IteratorEventType {
        if self.event_type == IteratorEventType::None {
            self.current = Some(self.root);
            self.event_type = IteratorEventType::Enter;
            return IteratorEventType::Enter;
        }

        if let Some(current) = self.current {
            match self.event_type {
                IteratorEventType::Enter => {
                    let first_child = self.arena.get(current).first_child;
                    if let Some(first_child) = first_child {
                        self.current = Some(first_child);
                        self.event_type = IteratorEventType::Enter;
                        IteratorEventType::Enter
                    } else {
                        self.event_type = IteratorEventType::Exit;
                        IteratorEventType::Exit
                    }
                }
                IteratorEventType::Exit => {
                    if current == self.root {
                        self.event_type = IteratorEventType::Done;
                        return IteratorEventType::Done;
                    }

                    let next = self.arena.get(current).next;
                    if let Some(next) = next {
                        self.current = Some(next);
                        self.event_type = IteratorEventType::Enter;
                        IteratorEventType::Enter
                    } else {
                        let parent = self.arena.get(current).parent;
                        if let Some(parent) = parent {
                            self.current = Some(parent);
                            self.event_type = IteratorEventType::Exit;
                            return IteratorEventType::Exit;
                        }
                        self.event_type = IteratorEventType::Done;
                        IteratorEventType::Done
                    }
                }
                _ => IteratorEventType::Done,
            }
        } else {
            IteratorEventType::Done
        }
    }
}

/// Item type for the standard Iterator implementation
pub type ArenaIteratorItem = (NodeId, IteratorEventType);

impl<'a> Iterator for ArenaNodeIterator<'a> {
    type Item = ArenaIteratorItem;

    fn next(&mut self) -> Option<Self::Item> {
        let event = self.advance();
        if event == IteratorEventType::Done {
            None
        } else {
            self.current.map(|node| (node, event))
        }
    }
}

/// A walker that can be used to iterate through the node tree
#[derive(Debug)]
pub struct ArenaNodeWalker<'a> {
    iterator: ArenaNodeIterator<'a>,
}

/// Event returned by the walker
#[derive(Debug, Clone, Copy)]
pub struct ArenaWalkerEvent {
    /// The node ID
    pub node: NodeId,
    /// Whether we are entering (true) or exiting (false) the node
    pub entering: bool,
}

impl<'a> ArenaNodeWalker<'a> {
    /// Create a new walker for the given arena and root node
    pub fn new(arena: &'a NodeArena, root: NodeId) -> Self {
        ArenaNodeWalker {
            iterator: ArenaNodeIterator::new(arena, root),
        }
    }

    /// Get the next event from the walker
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<ArenaWalkerEvent> {
        let event_type = self.iterator.next();
        if event_type == IteratorEventType::Done {
            None
        } else {
            self.iterator.get_node().map(|node| ArenaWalkerEvent {
                node,
                entering: event_type == IteratorEventType::Enter,
            })
        }
    }

    /// Resume iteration at a specific node
    pub fn resume_at(&mut self, node: NodeId, entering: bool) {
        let event_type = if entering {
            IteratorEventType::Enter
        } else {
            IteratorEventType::Exit
        };
        self.iterator.reset(node, event_type);
    }
}

// ============================================================================
// Tree module integration (from tree.rs)
// ============================================================================

/// Walk direction for tree traversal
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WalkDirection {
    /// Bottom-up: process children before parent
    BottomUp,
    /// Top-down: process parent before children
    TopDown,
}

/// Trait for walking/transforming the AST
pub trait Walkable {
    /// Walk the tree bottom-up, applying a function to each node
    fn walk_bottom_up<F>(&mut self, root: NodeId, f: &mut F)
    where
        F: FnMut(NodeId, &mut NodeValue);

    /// Walk the tree top-down, applying a function to each node
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
                let children: Vec<NodeId> = self.children(root).collect();
                for child in children {
                    self.walk_bottom_up(child, f);
                }
                let value = &mut self.get_mut(root).value;
                f(root, value);
            }
            WalkDirection::TopDown => {
                let value = &mut self.get_mut(root).value;
                f(root, value);

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
        matches!(
            (self, value),
            (NodeType::Document, NodeValue::Document)
                | (NodeType::BlockQuote, NodeValue::BlockQuote)
                | (NodeType::List, NodeValue::List(_))
                | (NodeType::Item, NodeValue::Item(_))
                | (NodeType::CodeBlock, NodeValue::CodeBlock(_))
                | (NodeType::HtmlBlock, NodeValue::HtmlBlock(_))
                | (NodeType::Paragraph, NodeValue::Paragraph)
                | (NodeType::Heading, NodeValue::Heading(_))
                | (NodeType::ThematicBreak, NodeValue::ThematicBreak)
                | (
                    NodeType::FootnoteDefinition,
                    NodeValue::FootnoteDefinition(_)
                )
                | (NodeType::Table, NodeValue::Table(_))
                | (NodeType::TableRow, NodeValue::TableRow(_))
                | (NodeType::TableCell, NodeValue::TableCell)
                | (NodeType::Text, NodeValue::Text(_))
                | (NodeType::TaskItem, NodeValue::TaskItem(_))
                | (NodeType::SoftBreak, NodeValue::SoftBreak)
                | (NodeType::HardBreak, NodeValue::HardBreak)
                | (NodeType::Code, NodeValue::Code(_))
                | (NodeType::HtmlInline, NodeValue::HtmlInline(_))
                | (NodeType::Emph, NodeValue::Emph)
                | (NodeType::Strong, NodeValue::Strong)
                | (NodeType::Strikethrough, NodeValue::Strikethrough)
                | (NodeType::Superscript, NodeValue::Superscript)
                | (NodeType::Subscript, NodeValue::Subscript)
                | (NodeType::Link, NodeValue::Link(_))
                | (NodeType::Image, NodeValue::Image(_))
                | (NodeType::FootnoteReference, NodeValue::FootnoteReference(_))
                | (NodeType::Math, NodeValue::Math(_))
                | (NodeType::Raw, NodeValue::Raw(_))
                | (NodeType::DescriptionList, NodeValue::DescriptionList)
                | (NodeType::DescriptionItem, NodeValue::DescriptionItem(_))
                | (NodeType::DescriptionTerm, NodeValue::DescriptionTerm)
                | (NodeType::DescriptionDetails, NodeValue::DescriptionDetails)
                | (NodeType::Alert, NodeValue::Alert(_))
                | (NodeType::WikiLink, NodeValue::WikiLink(_))
        )
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
pub trait Queryable {
    /// Query the tree and collect results
    fn query<T, F>(&self, root: NodeId, f: &mut F) -> Vec<T>
    where
        F: FnMut(NodeId, &NodeValue) -> Option<T>;

    /// Query with early termination
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

    // Unified naming aliases (preferred over the above methods)

    /// Check if any node matches the predicate (unified naming)
    fn query_any<F>(&self, root: NodeId, f: &mut F) -> bool
    where
        F: FnMut(NodeId, &NodeValue) -> bool,
    {
        self.any(root, f)
    }

    /// Check if all nodes match the predicate (unified naming)
    fn query_all<F>(&self, root: NodeId, f: &mut F) -> bool
    where
        F: FnMut(NodeId, &NodeValue) -> bool,
    {
        self.all(root, f)
    }

    /// Count nodes matching the predicate (unified naming)
    fn query_count<F>(&self, root: NodeId, f: &mut F) -> usize
    where
        F: FnMut(NodeId, &NodeValue) -> bool,
    {
        self.count(root, f)
    }
}

impl Queryable for NodeArena {
    fn query<T, F>(&self, root: NodeId, f: &mut F) -> Vec<T>
    where
        F: FnMut(NodeId, &NodeValue) -> Option<T>,
    {
        let mut results = Vec::new();
        self.queryable_recursive(root, f, &mut results);
        results
    }

    fn query_first<T, F>(&self, root: NodeId, f: &mut F) -> Option<T>
    where
        F: FnMut(NodeId, &NodeValue) -> Option<T>,
    {
        let value = &self.get(root).value;
        if let Some(result) = f(root, value) {
            return Some(result);
        }

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
        !Queryable::any(self, root, &mut |id, value| !f(id, value))
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
        Queryable::any(self, root, &mut |_, value| {
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
        Queryable::any(self, root, &mut |_, value| {
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
    fn queryable_recursive<T, F>(&self, node: NodeId, f: &mut F, results: &mut Vec<T>)
    where
        F: FnMut(NodeId, &NodeValue) -> Option<T>,
    {
        let value = &self.get(node).value;
        if let Some(result) = f(node, value) {
            results.push(result);
        }

        let children: Vec<NodeId> = self.children(node).collect();
        for child in children {
            self.queryable_recursive(child, f, results);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::arena::Node;

    fn create_test_arena() -> (NodeArena, NodeId) {
        use crate::core::arena::TreeOps;

        let mut arena = NodeArena::new();

        // Create a simple tree:
        // root
        //   ├── child1
        //   │     └── grandchild1
        //   └── child2

        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let child1 = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let child2 = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let grandchild1 = arena.alloc(Node::with_value(NodeValue::Text("test".into())));

        TreeOps::append_child(&mut arena, root, child1);
        TreeOps::append_child(&mut arena, root, child2);
        TreeOps::append_child(&mut arena, child1, grandchild1);

        (arena, root)
    }

    #[test]
    fn test_traverse_pre_order() {
        let (arena, root) = create_test_arena();
        let mut values = Vec::new();

        arena.traverse_pre_order(root, |value| {
            values.push(format!("{:?}", value));
        });

        assert_eq!(values.len(), 4);
        assert!(values[0].contains("Document"));
        assert!(values[1].contains("Paragraph"));
        assert!(values[2].contains("Text"));
        assert!(values[3].contains("Paragraph"));
    }

    #[test]
    fn test_traverse_post_order() {
        let (arena, root) = create_test_arena();
        let mut values = Vec::new();

        arena.traverse_post_order(root, |value| {
            values.push(format!("{:?}", value));
        });

        assert_eq!(values.len(), 4);
        // Post-order: grandchild1, child1, child2, root
        assert!(values[0].contains("Text"));
        assert!(values[3].contains("Document"));
    }

    #[test]
    fn test_traverse_with_events() {
        let (arena, root) = create_test_arena();
        let mut events = Vec::new();

        arena.traverse_with_events(root, |value, event_type| {
            events.push((format!("{:?}", value), event_type));
        });

        // Should have 8 events (enter + exit for each of 4 nodes)
        assert_eq!(events.len(), 8);
    }

    #[test]
    fn test_query_operations() {
        let (arena, root) = create_test_arena();

        // Test any - check if any node matches predicate
        assert!(Query::any(&arena, root, |v| matches!(
            v,
            NodeValue::Text(_)
        )));
        assert!(!Query::any(&arena, root, |v| matches!(
            v,
            NodeValue::Code(_)
        )));

        // Test all - check if all nodes match predicate
        assert!(Query::all(&arena, root, |v| {
            matches!(
                v,
                NodeValue::Document | NodeValue::Paragraph | NodeValue::Text(_)
            )
        }));

        // Test count - count nodes matching predicate
        assert_eq!(
            Query::count(&arena, root, |v| matches!(v, NodeValue::Paragraph)),
            2
        );

        // Test find_first - find first matching node
        assert!(
            Query::find_first(&arena, root, |v| matches!(v, NodeValue::Text(_)))
                .is_some()
        );
        assert!(
            Query::find_first(&arena, root, |v| matches!(v, NodeValue::Code(_)))
                .is_none()
        );

        // Test find_all - find all matching nodes
        let paragraphs =
            Query::find_all(&arena, root, |v| matches!(v, NodeValue::Paragraph));
        assert_eq!(paragraphs.len(), 2);
    }

    #[test]
    fn test_event_iterator() {
        let (arena, root) = create_test_arena();
        let events: Vec<_> = EventIterator::new(&arena, root).collect();

        // Should have 8 events (enter + exit for each of 4 nodes)
        assert_eq!(events.len(), 8);

        // First event should be enter root
        assert_eq!(events[0].event_type, EventType::Enter);
        assert_eq!(events[0].node_id, root);

        // Last event should be exit root
        assert_eq!(events[7].event_type, EventType::Exit);
        assert_eq!(events[7].node_id, root);
    }

    #[test]
    fn test_iterator_operations() {
        let (arena, root) = create_test_arena();

        // Test descendants_iter - iterate over all descendants
        let descendants: Vec<_> = arena.descendants_iter(root).collect();
        assert_eq!(descendants.len(), 4); // root + 3 children

        // Test children_iter - iterate over direct children
        let children: Vec<_> = arena.children_iter(root).collect();
        assert_eq!(children.len(), 2); // child1, child2

        // Test ancestors_iter - iterate over ancestors
        let grandchild1 =
            Query::find_first(&arena, root, |v| matches!(v, NodeValue::Text(_)));
        assert!(grandchild1.is_some());
        let ancestors: Vec<_> = arena.ancestors_iter(grandchild1.unwrap()).collect();
        assert_eq!(ancestors.len(), 2); // child1, root

        // Test siblings_iter - iterate over siblings
        let child1 = children[0];
        let siblings: Vec<_> = arena.siblings_iter(child1).collect();
        assert_eq!(siblings.len(), 1); // child2 (excluding child1 itself)
    }
}
