//! Visitor pattern for AST traversal (deprecated)
//!
//! ⚠️ **DEPRECATED**: This module is deprecated. Use `crate::iterator::ArenaNodeWalker` instead.
//!
//! This module uses the old Rc<RefCell>-based AST. It will be removed in a future version.
//!
//! Design inspired by flexmark-java's Visitor pattern.
//! Provides a flexible way to traverse and operate on the AST.

use crate::ast::node::Node;
use std::cell::RefCell;
use std::rc::Rc;

/// Visitor trait for AST traversal
///
/// Implement this trait to perform operations on the AST.
/// The visitor will be called for each node in the tree,
/// both when entering and exiting the node.
///
/// Design reference: flexmark-java's Visitor interface
pub trait Visitor {
    /// Called when entering a node
    ///
    /// Returns true to continue traversing children, false to skip children
    fn visit_enter(&mut self, node: &Rc<RefCell<Node>>) -> bool;

    /// Called when exiting a node
    fn visit_exit(&mut self, node: &Rc<RefCell<Node>>);
}

/// A visitor that can be used to traverse the AST
pub struct NodeVisitor;

impl NodeVisitor {
    /// Visit all nodes in the tree starting from root
    ///
    /// # Arguments
    ///
    /// * `root` - The root node to start traversal from
    /// * `visitor` - The visitor implementation
    pub fn visit(root: &Rc<RefCell<Node>>, visitor: &mut impl Visitor) {
        Self::visit_recursive(root, visitor);
    }

    fn visit_recursive(node: &Rc<RefCell<Node>>, visitor: &mut impl Visitor) -> bool {
        // Enter the node
        if !visitor.visit_enter(node) {
            return false; // Visitor wants to skip children
        }

        // Visit children
        if let Some(first_child) = node.borrow().first_child() {
            let mut current = Some(first_child);
            while let Some(child) = current {
                if !Self::visit_recursive(&child, visitor) {
                    return false;
                }
                current = child.borrow().next();
            }
        }

        // Exit the node
        visitor.visit_exit(node);
        true
    }
}

/// A visitor that collects all nodes of a specific type
pub struct CollectingVisitor<F> {
    predicate: F,
    collected: Vec<Rc<RefCell<Node>>>,
}

impl<F> CollectingVisitor<F>
where
    F: Fn(&Rc<RefCell<Node>>) -> bool,
{
    pub fn new(predicate: F) -> Self {
        Self {
            predicate,
            collected: Vec::new(),
        }
    }

    pub fn collected(&self) -> &[Rc<RefCell<Node>>] {
        &self.collected
    }

    pub fn into_collected(self) -> Vec<Rc<RefCell<Node>>> {
        self.collected
    }
}

impl<F> Visitor for CollectingVisitor<F>
where
    F: Fn(&Rc<RefCell<Node>>) -> bool,
{
    fn visit_enter(&mut self, node: &Rc<RefCell<Node>>) -> bool {
        if (self.predicate)(node) {
            self.collected.push(node.clone());
        }
        true
    }

    fn visit_exit(&mut self, _node: &Rc<RefCell<Node>>) {}
}

/// A visitor that finds the first node matching a predicate
pub struct FindVisitor<F> {
    predicate: F,
    found: Option<Rc<RefCell<Node>>>,
}

impl<F> FindVisitor<F>
where
    F: Fn(&Rc<RefCell<Node>>) -> bool,
{
    pub fn new(predicate: F) -> Self {
        Self {
            predicate,
            found: None,
        }
    }

    pub fn found(&self) -> Option<Rc<RefCell<Node>>> {
        self.found.clone()
    }
}

impl<F> Visitor for FindVisitor<F>
where
    F: Fn(&Rc<RefCell<Node>>) -> bool,
{
    fn visit_enter(&mut self, node: &Rc<RefCell<Node>>) -> bool {
        if self.found.is_some() {
            return false; // Already found, stop traversing
        }
        if (self.predicate)(node) {
            self.found = Some(node.clone());
            return false; // Found it, stop traversing
        }
        true
    }

    fn visit_exit(&mut self, _node: &Rc<RefCell<Node>>) {}
}

/// A visitor that transforms nodes
pub struct TransformVisitor<F> {
    transform: F,
}

impl<F> TransformVisitor<F>
where
    F: FnMut(&Rc<RefCell<Node>>),
{
    pub fn new(transform: F) -> Self {
        Self { transform }
    }
}

impl<F> Visitor for TransformVisitor<F>
where
    F: FnMut(&Rc<RefCell<Node>>),
{
    fn visit_enter(&mut self, node: &Rc<RefCell<Node>>) -> bool {
        (self.transform)(node);
        true
    }

    fn visit_exit(&mut self, _node: &Rc<RefCell<Node>>) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::node::{append_child, Node};

    /// A simple visitor that counts nodes
    struct CountingVisitor {
        enter_count: usize,
        exit_count: usize,
    }

    impl CountingVisitor {
        fn new() -> Self {
            Self {
                enter_count: 0,
                exit_count: 0,
            }
        }
    }

    impl Visitor for CountingVisitor {
        fn visit_enter(&mut self, _node: &Rc<RefCell<Node>>) -> bool {
            self.enter_count += 1;
            true
        }

        fn visit_exit(&mut self, _node: &Rc<RefCell<Node>>) {
            self.exit_count += 1;
        }
    }

    #[test]
    fn test_visitor_basic() {
        let root = Rc::new(RefCell::new(Node::new()));
        let child1 = Rc::new(RefCell::new(Node::new()));
        let child2 = Rc::new(RefCell::new(Node::new()));

        append_child(&root, child1.clone());
        append_child(&root, child2.clone());

        let mut visitor = CountingVisitor::new();
        NodeVisitor::visit(&root, &mut visitor);

        assert_eq!(visitor.enter_count, 3);
        assert_eq!(visitor.exit_count, 3);
    }

    #[test]
    fn test_collecting_visitor() {
        let root = Rc::new(RefCell::new(Node::new()));
        let child1 = Rc::new(RefCell::new(Node::new()));
        let child2 = Rc::new(RefCell::new(Node::new()));
        let grandchild = Rc::new(RefCell::new(Node::new()));

        append_child(&root, child1.clone());
        append_child(&root, child2.clone());
        append_child(&child1, grandchild.clone());

        // Collect all nodes (predicate always returns true)
        let mut visitor = CollectingVisitor::new(|_| true);
        NodeVisitor::visit(&root, &mut visitor);

        assert_eq!(visitor.collected().len(), 4);
    }

    #[test]
    fn test_find_visitor() {
        let root = Rc::new(RefCell::new(Node::new()));
        let child1 = Rc::new(RefCell::new(Node::new()));
        let child2 = Rc::new(RefCell::new(Node::new()));

        append_child(&root, child1.clone());
        append_child(&root, child2.clone());

        // Find the second child
        let mut visitor = FindVisitor::new(|node| {
            if let Some(parent) = node.borrow().parent() {
                if Rc::ptr_eq(&parent, &root) {
                    // Check if this is child2 by comparing with child1
                    if let Some(first) = parent.borrow().first_child() {
                        return !Rc::ptr_eq(&first, node);
                    }
                }
            }
            false
        });

        NodeVisitor::visit(&root, &mut visitor);

        assert!(visitor.found().is_some());
        assert!(Rc::ptr_eq(&visitor.found().unwrap(), &child2));
    }

    #[test]
    fn test_transform_visitor() {
        let root = Rc::new(RefCell::new(Node::new()));
        let child = Rc::new(RefCell::new(Node::new()));

        append_child(&root, child.clone());

        let mut count = 0;
        let mut visitor = TransformVisitor::new(|_| {
            count += 1;
        });

        NodeVisitor::visit(&root, &mut visitor);

        assert_eq!(count, 2);
    }

    #[test]
    fn test_visitor_skip_children() {
        /// A visitor that skips children after visiting the first level
        struct SkipAfterFirstLevelVisitor {
            enter_count: usize,
            depth: usize,
        }

        impl Visitor for SkipAfterFirstLevelVisitor {
            fn visit_enter(&mut self, _node: &Rc<RefCell<Node>>) -> bool {
                self.enter_count += 1;
                self.depth += 1;
                // Continue only if we're at depth 1 or less
                let should_continue = self.depth <= 2;
                should_continue
            }

            fn visit_exit(&mut self, _node: &Rc<RefCell<Node>>) {
                self.depth -= 1;
            }
        }

        let root = Rc::new(RefCell::new(Node::new()));
        let child1 = Rc::new(RefCell::new(Node::new()));
        let child2 = Rc::new(RefCell::new(Node::new()));
        let grandchild = Rc::new(RefCell::new(Node::new()));

        append_child(&root, child1.clone());
        append_child(&root, child2.clone());
        append_child(&child1, grandchild.clone());

        let mut visitor = SkipAfterFirstLevelVisitor {
            enter_count: 0,
            depth: 0,
        };
        NodeVisitor::visit(&root, &mut visitor);

        // Should visit root, child1, child2 (skip grandchild because depth > 2)
        assert_eq!(visitor.enter_count, 3);
    }
}
