//! Walkable trait for AST traversal.
//!
//! This module provides the `Walkable` trait, inspired by Pandoc's Walkable typeclass.
//! It allows generic traversal and transformation of the AST.
//!
//! # Example
//!
//! ```ignore
//! use clmd::core::walk::{Walkable, query, walk};
//! use clmd::{parse_document, Options};
//! use clmd::nodes::NodeValue;
//!
//! let options = Options::default();
//! let (arena, root) = parse_document("# Hello **World**", &options);
//!
//! // Query all headings
//! let headings: Vec<&NodeValue> = query(&arena, root, |node| {
//!     matches!(node, NodeValue::Heading(_)).then_some(node)
//! });
//!
//! // Transform all text to uppercase
//! let mut new_arena = arena.clone();
//! walk(&mut new_arena, root, |node| {
//!     if let NodeValue::Text(text) = node {
//!         *node = NodeValue::Text(text.to_uppercase().into());
//!     }
//! });
//! ```

use crate::arena::{NodeArena, NodeId};
use crate::nodes::NodeValue;

/// A trait for types that can be walked (traversed).
///
/// This is inspired by Pandoc's Walkable typeclass and allows for
/// generic AST traversal and transformation.
pub trait Walkable {
    /// Walk over the structure, applying a transformation function.
    ///
    /// The function is applied to each node in a bottom-up manner.
    fn walk<F>(&mut self, f: &mut F)
    where
        F: FnMut(&mut NodeValue);

    /// Walk over the structure, applying a fallible transformation function.
    ///
    /// Similar to `walk`, but the transformation can fail.
    fn walk_m<F, E>(&mut self, f: &mut F) -> Result<(), E>
    where
        F: FnMut(&mut NodeValue) -> Result<(), E>;

    /// Query the structure, collecting results.
    ///
    /// The function is applied to each node, and non-None results are collected.
    fn query<F, R>(&self, f: &mut F) -> Vec<R>
    where
        F: FnMut(&NodeValue) -> Option<R>;
}

impl Walkable for NodeValue {
    fn walk<F>(&mut self, f: &mut F)
    where
        F: FnMut(&mut NodeValue),
    {
        f(self);
    }

    fn walk_m<F, E>(&mut self, f: &mut F) -> Result<(), E>
    where
        F: FnMut(&mut NodeValue) -> Result<(), E>,
    {
        f(self)
    }

    fn query<F, R>(&self, f: &mut F) -> Vec<R>
    where
        F: FnMut(&NodeValue) -> Option<R>,
    {
        f(self).into_iter().collect()
    }
}

/// Walk over all nodes in the arena starting from a given node.
///
/// # Arguments
///
/// * `arena` - The arena containing the AST nodes
/// * `root` - The root node to start from
/// * `f` - The transformation function to apply to each node
///
/// # Example
///
/// ```ignore
/// use clmd::core::walk::walk;
/// use clmd::{parse_document, Options};
/// use clmd::nodes::NodeValue;
///
/// let options = Options::default();
/// let (mut arena, root) = parse_document("# Hello", &options);
///
/// walk(&mut arena, root, |node| {
///     if let NodeValue::Text(text) = node {
///         *node = NodeValue::Text(text.to_uppercase().into());
///     }
/// });
/// ```
pub fn walk<F>(arena: &mut NodeArena, root: NodeId, mut f: F)
where
    F: FnMut(&mut NodeValue),
{
    walk_recursive(arena, root, &mut f);
}

fn walk_recursive<F>(arena: &mut NodeArena, node_id: NodeId, f: &mut F)
where
    F: FnMut(&mut NodeValue),
{
    // First, recurse into children
    let child_ids: Vec<NodeId> = {
        let node = arena.get(node_id);
        let mut children = Vec::new();
        let mut child = node.first_child;
        while let Some(child_id) = child {
            children.push(child_id);
            child = arena.get(child_id).next;
        }
        children
    };

    for child_id in child_ids {
        walk_recursive(arena, child_id, f);
    }

    // Then apply transformation to this node
    let node = arena.get_mut(node_id);
    f(&mut node.value);
}

/// Walk over all nodes with a fallible function.
///
/// Similar to `walk`, but the transformation can fail.
///
/// # Arguments
///
/// * `arena` - The arena containing the AST nodes
/// * `root` - The root node to start from
/// * `f` - The transformation function to apply to each node
///
/// # Returns
///
/// Ok(()) if all transformations succeeded, or the first error encountered.
pub fn walk_m<F, E>(arena: &mut NodeArena, root: NodeId, mut f: F) -> Result<(), E>
where
    F: FnMut(&mut NodeValue) -> Result<(), E>,
{
    walk_m_recursive(arena, root, &mut f)
}

fn walk_m_recursive<F, E>(
    arena: &mut NodeArena,
    node_id: NodeId,
    f: &mut F,
) -> Result<(), E>
where
    F: FnMut(&mut NodeValue) -> Result<(), E>,
{
    // First, recurse into children
    let child_ids: Vec<NodeId> = {
        let node = arena.get(node_id);
        let mut children = Vec::new();
        let mut child = node.first_child;
        while let Some(child_id) = child {
            children.push(child_id);
            child = arena.get(child_id).next;
        }
        children
    };

    for child_id in child_ids {
        walk_m_recursive(arena, child_id, f)?;
    }

    // Then apply transformation to this node
    let node = arena.get_mut(node_id);
    f(&mut node.value)
}

/// Query the AST, collecting results.
///
/// # Arguments
///
/// * `arena` - The arena containing the AST nodes
/// * `root` - The root node to start from
/// * `f` - The query function to apply to each node
///
/// # Returns
///
/// A vector of all non-None results.
///
/// # Example
///
/// ```ignore
/// use clmd::core::walk::query;
/// use clmd::{parse_document, Options};
/// use clmd::nodes::NodeValue;
///
/// let options = Options::default();
/// let (arena, root) = parse_document("# Hello\n\n## World", &options);
///
/// // Collect all heading levels
/// let levels: Vec<u8> = query(&arena, root, |node| {
///     if let NodeValue::Heading(h) = node {
///         Some(h.level)
///     } else {
///         None
///     }
/// });
///
/// assert_eq!(levels, vec![1, 2]);
/// ```
pub fn query<F, R>(arena: &NodeArena, root: NodeId, mut f: F) -> Vec<R>
where
    F: FnMut(&NodeValue) -> Option<R>,
{
    let mut results = Vec::new();
    query_recursive(arena, root, &mut f, &mut results);
    results
}

fn query_recursive<F, R>(
    arena: &NodeArena,
    node_id: NodeId,
    f: &mut F,
    results: &mut Vec<R>,
) where
    F: FnMut(&NodeValue) -> Option<R>,
{
    let node = arena.get(node_id);

    // Apply query function to this node
    if let Some(result) = f(&node.value) {
        results.push(result);
    }

    // Recurse into children
    let mut child = node.first_child;
    while let Some(child_id) = child {
        query_recursive(arena, child_id, f, results);
        child = arena.get(child_id).next;
    }
}

/// Query with early termination.
///
/// Similar to `query`, but stops when the predicate returns true for any node.
///
/// # Arguments
///
/// * `arena` - The arena containing the AST nodes
/// * `root` - The root node to start from
/// * `f` - The predicate function
///
/// # Returns
///
/// true if the predicate matched any node, false otherwise.
pub fn query_any<F>(arena: &NodeArena, root: NodeId, mut f: F) -> bool
where
    F: FnMut(&NodeValue) -> bool,
{
    query_any_recursive(arena, root, &mut f)
}

fn query_any_recursive<F>(arena: &NodeArena, node_id: NodeId, f: &mut F) -> bool
where
    F: FnMut(&NodeValue) -> bool,
{
    let node = arena.get(node_id);

    // Check this node
    if f(&node.value) {
        return true;
    }

    // Recurse into children
    let mut child = node.first_child;
    while let Some(child_id) = child {
        if query_any_recursive(arena, child_id, f) {
            return true;
        }
        child = arena.get(child_id).next;
    }

    false
}

/// Walk with context information.
///
/// Similar to `walk`, but provides context about the node's position in the tree.
///
/// # Arguments
///
/// * `arena` - The arena containing the AST nodes
/// * `root` - The root node to start from
/// * `f` - The transformation function that receives context
///
/// # Example
///
/// ```ignore
/// use clmd::core::walk::walk_with_context;
/// use clmd::{parse_document, Options};
/// use clmd::nodes::NodeValue;
///
/// let options = Options::default();
/// let (mut arena, root) = parse_document("# Hello", &options);
///
/// walk_with_context(&mut arena, root, |node, depth, is_leaf| {
///     if is_leaf {
///         println!("Leaf node at depth {}", depth);
///     }
/// });
/// ```
pub fn walk_with_context<F>(arena: &mut NodeArena, root: NodeId, mut f: F)
where
    F: FnMut(&mut NodeValue, usize, bool),
{
    walk_context_recursive(arena, root, 0, &mut f);
}

fn walk_context_recursive<F>(
    arena: &mut NodeArena,
    node_id: NodeId,
    depth: usize,
    f: &mut F,
) where
    F: FnMut(&mut NodeValue, usize, bool),
{
    // Check if this is a leaf node
    let is_leaf = arena.get(node_id).first_child.is_none();

    // Recurse into children first (bottom-up)
    let child_ids: Vec<NodeId> = {
        let node = arena.get(node_id);
        let mut children = Vec::new();
        let mut child = node.first_child;
        while let Some(child_id) = child {
            children.push(child_id);
            child = arena.get(child_id).next;
        }
        children
    };

    for child_id in child_ids {
        walk_context_recursive(arena, child_id, depth + 1, f);
    }

    // Apply transformation to this node
    let node = arena.get_mut(node_id);
    f(&mut node.value, depth, is_leaf);
}

/// A walker that can be used to traverse the AST in different orders.
#[derive(Debug)]
pub struct Walker<'a> {
    arena: &'a NodeArena,
    root: NodeId,
}

impl<'a> Walker<'a> {
    /// Create a new walker.
    pub fn new(arena: &'a NodeArena, root: NodeId) -> Self {
        Self { arena, root }
    }

    /// Walk in pre-order (parent before children).
    pub fn pre_order<F>(&self, mut f: F)
    where
        F: FnMut(&NodeValue),
    {
        self.pre_order_recursive(self.root, &mut f);
    }

    fn pre_order_recursive<F>(&self, node_id: NodeId, f: &mut F)
    where
        F: FnMut(&NodeValue),
    {
        let node = self.arena.get(node_id);
        f(&node.value);

        let mut child = node.first_child;
        while let Some(child_id) = child {
            self.pre_order_recursive(child_id, f);
            child = self.arena.get(child_id).next;
        }
    }

    /// Walk in post-order (children before parent).
    pub fn post_order<F>(&self, mut f: F)
    where
        F: FnMut(&NodeValue),
    {
        self.post_order_recursive(self.root, &mut f);
    }

    fn post_order_recursive<F>(&self, node_id: NodeId, f: &mut F)
    where
        F: FnMut(&NodeValue),
    {
        let node = self.arena.get(node_id);

        let mut child = node.first_child;
        while let Some(child_id) = child {
            self.post_order_recursive(child_id, f);
            child = self.arena.get(child_id).next;
        }

        f(&node.value);
    }

    /// Walk level by level (breadth-first).
    pub fn level_order<F>(&self, mut f: F)
    where
        F: FnMut(&NodeValue, usize), // node value and depth
    {
        use std::collections::VecDeque;

        let mut queue = VecDeque::new();
        queue.push_back((self.root, 0));

        while let Some((node_id, depth)) = queue.pop_front() {
            let node = self.arena.get(node_id);
            f(&node.value, depth);

            let mut child = node.first_child;
            while let Some(child_id) = child {
                queue.push_back((child_id, depth + 1));
                child = self.arena.get(child_id).next;
            }
        }
    }
}

/// Collect all nodes of a specific type.
///
/// # Arguments
///
/// * `arena` - The arena containing the AST nodes
/// * `root` - The root node to start from
/// * `predicate` - A function that returns true for nodes to collect
///
/// # Returns
///
/// A vector of node IDs that match the predicate.
pub fn collect_nodes<F>(arena: &NodeArena, root: NodeId, mut predicate: F) -> Vec<NodeId>
where
    F: FnMut(&NodeValue) -> bool,
{
    let mut results = Vec::new();
    collect_recursive(arena, root, &mut predicate, &mut results);
    results
}

fn collect_recursive<F>(
    arena: &NodeArena,
    node_id: NodeId,
    predicate: &mut F,
    results: &mut Vec<NodeId>,
) where
    F: FnMut(&NodeValue) -> bool,
{
    let node = arena.get(node_id);

    if predicate(&node.value) {
        results.push(node_id);
    }

    let mut child = node.first_child;
    while let Some(child_id) = child {
        collect_recursive(arena, child_id, predicate, results);
        child = arena.get(child_id).next;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arena::{Node, NodeArena, TreeOps};
    use crate::nodes::{NodeHeading, NodeValue};

    #[test]
    fn test_walk() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let heading = arena.alloc(Node::with_value(NodeValue::Heading(NodeHeading {
            level: 1,
            setext: false,
            closed: false,
        })));
        let text = arena.alloc(Node::with_value(NodeValue::Text("Hello".into())));

        TreeOps::append_child(&mut arena, root, heading);
        TreeOps::append_child(&mut arena, heading, text);

        let mut count = 0;
        walk(&mut arena, root, |_node| {
            count += 1;
        });

        assert_eq!(count, 3); // Document, Heading, Text
    }

    #[test]
    fn test_query() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let heading1 = arena.alloc(Node::with_value(NodeValue::Heading(NodeHeading {
            level: 1,
            setext: false,
            closed: false,
        })));
        let heading2 = arena.alloc(Node::with_value(NodeValue::Heading(NodeHeading {
            level: 2,
            setext: false,
            closed: false,
        })));

        TreeOps::append_child(&mut arena, root, heading1);
        TreeOps::append_child(&mut arena, root, heading2);

        let levels: Vec<u8> = query(&arena, root, |node| {
            if let NodeValue::Heading(h) = node {
                Some(h.level)
            } else {
                None
            }
        });

        assert_eq!(levels, vec![1, 2]);
    }

    #[test]
    fn test_query_any() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let heading = arena.alloc(Node::with_value(NodeValue::Heading(NodeHeading {
            level: 1,
            setext: false,
            closed: false,
        })));

        TreeOps::append_child(&mut arena, root, heading);

        let has_heading =
            query_any(&arena, root, |node| matches!(node, NodeValue::Heading(_)));

        assert!(has_heading);

        let has_table =
            query_any(&arena, root, |node| matches!(node, NodeValue::Table(_)));

        assert!(!has_table);
    }

    #[test]
    fn test_walker_pre_order() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let heading = arena.alloc(Node::with_value(NodeValue::Heading(NodeHeading {
            level: 1,
            setext: false,
            closed: false,
        })));
        let text = arena.alloc(Node::with_value(NodeValue::Text("Hello".into())));

        TreeOps::append_child(&mut arena, root, heading);
        TreeOps::append_child(&mut arena, heading, text);

        let walker = Walker::new(&arena, root);
        let mut order = Vec::new();

        walker.pre_order(|node| {
            order.push(format!("{:?}", node).chars().next().unwrap());
        });

        // Document -> Heading -> Text
        assert_eq!(order[0], 'D');
        assert_eq!(order[1], 'H');
        assert_eq!(order[2], 'T');
    }

    #[test]
    fn test_walker_post_order() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let heading = arena.alloc(Node::with_value(NodeValue::Heading(NodeHeading {
            level: 1,
            setext: false,
            closed: false,
        })));
        let text = arena.alloc(Node::with_value(NodeValue::Text("Hello".into())));

        TreeOps::append_child(&mut arena, root, heading);
        TreeOps::append_child(&mut arena, heading, text);

        let walker = Walker::new(&arena, root);
        let mut order = Vec::new();

        walker.post_order(|node| {
            order.push(format!("{:?}", node).chars().next().unwrap());
        });

        // Text -> Heading -> Document
        assert_eq!(order[0], 'T');
        assert_eq!(order[1], 'H');
        assert_eq!(order[2], 'D');
    }

    #[test]
    fn test_walker_level_order() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let heading = arena.alloc(Node::with_value(NodeValue::Heading(NodeHeading {
            level: 1,
            setext: false,
            closed: false,
        })));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text1 = arena.alloc(Node::with_value(NodeValue::Text("Hello".into())));
        let text2 = arena.alloc(Node::with_value(NodeValue::Text("World".into())));

        TreeOps::append_child(&mut arena, root, heading);
        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, heading, text1);
        TreeOps::append_child(&mut arena, para, text2);

        let walker = Walker::new(&arena, root);
        let mut depths = Vec::new();

        walker.level_order(|_node, depth| {
            depths.push(depth);
        });

        // Document (0) -> Heading (1), Para (1) -> Text1 (2), Text2 (2)
        assert_eq!(depths, vec![0, 1, 1, 2, 2]);
    }

    #[test]
    fn test_collect_nodes() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let heading = arena.alloc(Node::with_value(NodeValue::Heading(NodeHeading {
            level: 1,
            setext: false,
            closed: false,
        })));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));

        TreeOps::append_child(&mut arena, root, heading);
        TreeOps::append_child(&mut arena, root, para);

        let headings =
            collect_nodes(&arena, root, |node| matches!(node, NodeValue::Heading(_)));

        assert_eq!(headings.len(), 1);
        assert_eq!(headings[0], heading);
    }

    #[test]
    fn test_walk_m() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let heading = arena.alloc(Node::with_value(NodeValue::Heading(NodeHeading {
            level: 1,
            setext: false,
            closed: false,
        })));

        TreeOps::append_child(&mut arena, root, heading);

        let result = walk_m(&mut arena, root, |_node| -> Result<(), ()> { Ok(()) });

        assert!(result.is_ok());
    }

    #[test]
    fn test_walk_with_context() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let heading = arena.alloc(Node::with_value(NodeValue::Heading(NodeHeading {
            level: 1,
            setext: false,
            closed: false,
        })));
        let text = arena.alloc(Node::with_value(NodeValue::Text("Hello".into())));

        TreeOps::append_child(&mut arena, root, heading);
        TreeOps::append_child(&mut arena, heading, text);

        let mut max_depth = 0;
        walk_with_context(&mut arena, root, |_node, depth, _is_leaf| {
            max_depth = max_depth.max(depth);
        });

        assert_eq!(max_depth, 2); // Document (0) -> Heading (1) -> Text (2)
    }
}
