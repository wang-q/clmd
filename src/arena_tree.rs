//! A DOM-like tree data structure based on `&Node` references.
//!
//! This module is inspired by comrak's arena_tree implementation.
//! It provides a more ergonomic API for tree operations compared to NodeId-based approaches.
//!
//! The key advantage is using direct references with `Cell` for interior mutability,
//! allowing natural tree operations without mutable references.
//!
//! # Example
//!
//! ```
//! use clmd::arena_tree::{Node, TreeOperations};
//! use std::cell::RefCell;
//!
//! let arena = typed_arena::Arena::new();
//! let root = arena.alloc(Node::new(RefCell::new("root")));
//! let child = arena.alloc(Node::new(RefCell::new("child")));
//!
//! root.append(child);
//!
//! assert_eq!(root.first_child().map(|n| *n.data.borrow()), Some("child"));
//! ```

use std::cell::{Cell, Ref, RefCell, RefMut};
use std::fmt;

/// A node inside a DOM-like tree.
///
/// This struct represents a node in an arena-allocated tree structure.
/// It uses `Cell` for interior mutability, allowing tree modifications
/// through shared references.
pub struct Node<'a, T: 'a> {
    parent: Cell<Option<&'a Node<'a, T>>>,
    previous_sibling: Cell<Option<&'a Node<'a, T>>>,
    next_sibling: Cell<Option<&'a Node<'a, T>>>,
    first_child: Cell<Option<&'a Node<'a, T>>>,
    last_child: Cell<Option<&'a Node<'a, T>>>,

    /// The data held by the node.
    pub data: T,
}

impl<'a, T: 'a> fmt::Debug for Node<'a, T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Node").field("data", &self.data).finish()
    }
}

impl<'a, T: 'a> Node<'a, T> {
    /// Create a new node from its associated data.
    ///
    /// Typically, this node needs to be moved into an arena allocator
    /// before it can be used in a tree.
    ///
    /// # Example
    ///
    /// ```
    /// use clmd::arena_tree::Node;
    /// use std::cell::RefCell;
    ///
    /// let node = Node::new(RefCell::new(42));
    /// assert_eq!(*node.data.borrow(), 42);
    /// ```
    pub fn new(data: T) -> Node<'a, T> {
        Node {
            parent: Cell::new(None),
            first_child: Cell::new(None),
            last_child: Cell::new(None),
            previous_sibling: Cell::new(None),
            next_sibling: Cell::new(None),
            data,
        }
    }

    /// Return a reference to the parent node, unless this node is the root of the tree.
    ///
    /// # Example
    ///
    /// ```
    /// use clmd::arena_tree::{Node, TreeOperations};
    /// use std::cell::RefCell;
    ///
    /// let arena = typed_arena::Arena::new();
    /// let parent = arena.alloc(Node::new(RefCell::new("parent")));
    /// let child = arena.alloc(Node::new(RefCell::new("child")));
    ///
    /// parent.append(child);
    ///
    /// assert_eq!(child.parent().map(|n| *n.data.borrow()), Some("parent"));
    /// ```
    pub fn parent(&self) -> Option<&'a Node<'a, T>> {
        self.parent.get()
    }

    /// Return a reference to the first child of this node, unless it has no child.
    pub fn first_child(&self) -> Option<&'a Node<'a, T>> {
        self.first_child.get()
    }

    /// Return a reference to the last child of this node, unless it has no child.
    pub fn last_child(&self) -> Option<&'a Node<'a, T>> {
        self.last_child.get()
    }

    /// Return a reference to the previous sibling of this node, unless it is a first child.
    pub fn previous_sibling(&self) -> Option<&'a Node<'a, T>> {
        self.previous_sibling.get()
    }

    /// Return a reference to the next sibling of this node, unless it is a last child.
    pub fn next_sibling(&self) -> Option<&'a Node<'a, T>> {
        self.next_sibling.get()
    }

    /// Returns whether two references point to the same node.
    pub fn same_node(&self, other: &Node<'a, T>) -> bool {
        std::ptr::eq(self, other)
    }

    /// Return an iterator of references to this node and its ancestors.
    ///
    /// Call `.next().unwrap()` once on the iterator to skip the node itself.
    pub fn ancestors(&'a self) -> Ancestors<'a, T> {
        Ancestors(Some(self))
    }

    /// Return an iterator of references to this node and the siblings before it.
    ///
    /// Call `.next().unwrap()` once on the iterator to skip the node itself.
    pub fn preceding_siblings(&'a self) -> PrecedingSiblings<'a, T> {
        PrecedingSiblings(Some(self))
    }

    /// Return an iterator of references to this node and the siblings after it.
    ///
    /// Call `.next().unwrap()` once on the iterator to skip the node itself.
    pub fn following_siblings(&'a self) -> FollowingSiblings<'a, T> {
        FollowingSiblings(Some(self))
    }

    /// Return an iterator of references to this node's children.
    pub fn children(&'a self) -> Children<'a, T> {
        Children(self.first_child.get())
    }

    /// Return an iterator of references to this node's children, in reverse order.
    pub fn reverse_children(&'a self) -> ReverseChildren<'a, T> {
        ReverseChildren(self.last_child.get())
    }

    /// Return an iterator of references to this `Node` and its descendants, in tree order.
    ///
    /// Parent nodes appear before the descendants.
    /// Call `.next().unwrap()` once on the iterator to skip the node itself.
    pub fn descendants(&'a self) -> Descendants<'a, T> {
        Descendants(self.traverse())
    }

    /// Return an iterator of references to `NodeEdge` enums for each `Node` and its descendants,
    /// in tree order.
    ///
    /// `NodeEdge` enums represent the `Start` or `End` of each node.
    pub fn traverse(&'a self) -> Traverse<'a, T> {
        Traverse {
            root: self,
            next: Some(NodeEdge::Start(self)),
        }
    }

    /// Return an iterator of references to `NodeEdge` enums for each `Node` and its descendants,
    /// in *reverse* order.
    ///
    /// `NodeEdge` enums represent the `Start` or `End` of each node.
    pub fn reverse_traverse(&'a self) -> ReverseTraverse<'a, T> {
        ReverseTraverse {
            root: self,
            next: Some(NodeEdge::End(self)),
        }
    }
}

impl<'a, T: 'a> Node<'a, T> {
    /// Detach a node from its parent and siblings. Children are not affected.
    pub fn detach(&self) {
        let parent = self.parent.take();
        let previous_sibling = self.previous_sibling.take();
        let next_sibling = self.next_sibling.take();

        if let Some(next_sibling) = next_sibling {
            next_sibling.previous_sibling.set(previous_sibling);
        } else if let Some(parent) = parent {
            parent.last_child.set(previous_sibling);
        }

        if let Some(previous_sibling) = previous_sibling {
            previous_sibling.next_sibling.set(next_sibling);
        } else if let Some(parent) = parent {
            parent.first_child.set(next_sibling);
        }
    }

    /// Append a new child to this node, after existing children.
    pub fn append(&'a self, new_child: &'a Node<'a, T>) {
        new_child.detach();
        new_child.parent.set(Some(self));
        if let Some(last_child) = self.last_child.take() {
            new_child.previous_sibling.set(Some(last_child));
            debug_assert!(last_child.next_sibling.get().is_none());
            last_child.next_sibling.set(Some(new_child));
        } else {
            debug_assert!(self.first_child.get().is_none());
            self.first_child.set(Some(new_child));
        }
        self.last_child.set(Some(new_child));
    }

    /// Append multiple new children to this node, after existing children.
    pub fn extend(&'a self, new_children: impl IntoIterator<Item = &'a Node<'a, T>>) {
        for child in new_children.into_iter() {
            self.append(child);
        }
    }

    /// Prepend a new child to this node, before existing children.
    pub fn prepend(&'a self, new_child: &'a Node<'a, T>) {
        new_child.detach();
        new_child.parent.set(Some(self));
        if let Some(first_child) = self.first_child.take() {
            debug_assert!(first_child.previous_sibling.get().is_none());
            first_child.previous_sibling.set(Some(new_child));
            new_child.next_sibling.set(Some(first_child));
        } else {
            debug_assert!(self.first_child.get().is_none());
            self.last_child.set(Some(new_child));
        }
        self.first_child.set(Some(new_child));
    }

    /// Insert a new sibling after this node.
    pub fn insert_after(&'a self, new_sibling: &'a Node<'a, T>) {
        new_sibling.detach();
        new_sibling.parent.set(self.parent.get());
        new_sibling.previous_sibling.set(Some(self));
        if let Some(next_sibling) = self.next_sibling.take() {
            debug_assert!(std::ptr::eq(
                next_sibling.previous_sibling.get().unwrap(),
                self
            ));
            next_sibling.previous_sibling.set(Some(new_sibling));
            new_sibling.next_sibling.set(Some(next_sibling));
        } else if let Some(parent) = self.parent.get() {
            debug_assert!(std::ptr::eq(parent.last_child.get().unwrap(), self));
            parent.last_child.set(Some(new_sibling));
        }
        self.next_sibling.set(Some(new_sibling));
    }

    /// Insert a new sibling before this node.
    pub fn insert_before(&'a self, new_sibling: &'a Node<'a, T>) {
        new_sibling.detach();
        new_sibling.parent.set(self.parent.get());
        new_sibling.next_sibling.set(Some(self));
        if let Some(previous_sibling) = self.previous_sibling.take() {
            new_sibling.previous_sibling.set(Some(previous_sibling));
            debug_assert!(std::ptr::eq(
                previous_sibling.next_sibling.get().unwrap(),
                self
            ));
            previous_sibling.next_sibling.set(Some(new_sibling));
        } else if let Some(parent) = self.parent.get() {
            debug_assert!(std::ptr::eq(parent.first_child.get().unwrap(), self));
            parent.first_child.set(Some(new_sibling));
        }
        self.previous_sibling.set(Some(new_sibling));
    }
}

impl<T> Node<'_, RefCell<T>> {
    /// Shorthand for `node.data.borrow()`.
    pub fn data(&self) -> Ref<'_, T> {
        self.data.borrow()
    }

    /// Shorthand for `node.data.borrow_mut()`.
    pub fn data_mut(&self) -> RefMut<'_, T> {
        self.data.borrow_mut()
    }
}

/// An edge of the node graph returned by a traversal iterator.
#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
pub enum NodeEdge<T> {
    /// Indicates that start of a node that has children.
    /// Yielded by `Traverse::next` before the node's descendants.
    /// In HTML or XML, this corresponds to an opening tag like `<div>`
    Start(T),

    /// Indicates that end of a node that has children.
    /// Yielded by `Traverse::next` after the node's descendants.
    /// In HTML or XML, this corresponds to a closing tag like `</div>`
    End(T),
}

macro_rules! axis_iterator {
    (#[$attr:meta] $name:ident : $next:ident) => {
        #[$attr]
        pub struct $name<'a, T: 'a>(Option<&'a Node<'a, T>>);

        impl<'a, T: 'a> fmt::Debug for $name<'a, T> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_tuple(stringify!($name))
                    .field(&self.0.map(|_| "..."))
                    .finish()
            }
        }

        impl<'a, T> Iterator for $name<'a, T> {
            type Item = &'a Node<'a, T>;

            fn next(&mut self) -> Option<&'a Node<'a, T>> {
                match self.0.take() {
                    Some(node) => {
                        self.0 = node.$next.get();
                        Some(node)
                    }
                    None => None,
                }
            }
        }
    };
}

axis_iterator! {
    /// An iterator of references to the ancestors of a given node.
    Ancestors: parent
}

axis_iterator! {
    /// An iterator of references to the siblings before a given node.
    PrecedingSiblings: previous_sibling
}

axis_iterator! {
    /// An iterator of references to the siblings after a given node.
    FollowingSiblings: next_sibling
}

axis_iterator! {
    /// An iterator of references to the children of a given node.
    Children: next_sibling
}

axis_iterator! {
    /// An iterator of references to the children of a given node, in reverse order.
    ReverseChildren: previous_sibling
}

/// An iterator of references to a given node and its descendants, in tree order.
pub struct Descendants<'a, T: 'a>(Traverse<'a, T>);

impl<'a, T: 'a> fmt::Debug for Descendants<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Descendants").field(&"...").finish()
    }
}

impl<'a, T> Iterator for Descendants<'a, T> {
    type Item = &'a Node<'a, T>;

    fn next(&mut self) -> Option<&'a Node<'a, T>> {
        loop {
            match self.0.next() {
                Some(NodeEdge::Start(node)) => return Some(node),
                Some(NodeEdge::End(_)) => {}
                None => return None,
            }
        }
    }
}

/// An iterator of the start and end edges of a given node and its descendants,
/// in tree order.
pub struct Traverse<'a, T: 'a> {
    root: &'a Node<'a, T>,
    next: Option<NodeEdge<&'a Node<'a, T>>>,
}

impl<'a, T: 'a> fmt::Debug for Traverse<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Traverse").field("next", &"...").finish()
    }
}

impl<'a, T> Iterator for Traverse<'a, T> {
    type Item = NodeEdge<&'a Node<'a, T>>;

    fn next(&mut self) -> Option<NodeEdge<&'a Node<'a, T>>> {
        match self.next.take() {
            Some(item) => {
                self.next = match item {
                    NodeEdge::Start(node) => match node.first_child.get() {
                        Some(child) => Some(NodeEdge::Start(child)),
                        None => Some(NodeEdge::End(node)),
                    },
                    NodeEdge::End(node) => {
                        if node.same_node(self.root) {
                            None
                        } else {
                            match node.next_sibling.get() {
                                Some(sibling) => Some(NodeEdge::Start(sibling)),
                                None => match node.parent.get() {
                                    Some(parent) => Some(NodeEdge::End(parent)),
                                    None => panic!("tree modified during iteration"),
                                },
                            }
                        }
                    }
                };
                Some(item)
            }
            None => None,
        }
    }
}

/// An iterator of the start and end edges of a given node and its descendants,
/// in reverse tree order.
pub struct ReverseTraverse<'a, T: 'a> {
    root: &'a Node<'a, T>,
    next: Option<NodeEdge<&'a Node<'a, T>>>,
}

impl<'a, T: 'a> fmt::Debug for ReverseTraverse<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ReverseTraverse")
            .field("next", &"...")
            .finish()
    }
}

impl<'a, T> Iterator for ReverseTraverse<'a, T> {
    type Item = NodeEdge<&'a Node<'a, T>>;

    fn next(&mut self) -> Option<NodeEdge<&'a Node<'a, T>>> {
        match self.next.take() {
            Some(item) => {
                self.next = match item {
                    NodeEdge::End(node) => match node.last_child.get() {
                        Some(child) => Some(NodeEdge::End(child)),
                        None => Some(NodeEdge::Start(node)),
                    },
                    NodeEdge::Start(node) => {
                        if node.same_node(self.root) {
                            None
                        } else {
                            match node.previous_sibling.get() {
                                Some(sibling) => Some(NodeEdge::End(sibling)),
                                None => match node.parent.get() {
                                    Some(parent) => Some(NodeEdge::Start(parent)),
                                    None => panic!("tree modified during iteration"),
                                },
                            }
                        }
                    }
                };
                Some(item)
            }
            None => None,
        }
    }
}

/// Trait alias for tree operations (for backward compatibility).
pub trait TreeOperations<'a, T: 'a> {
    /// Append a child to a parent node.
    fn append_child(&'a self, child: &'a Node<'a, T>);
    /// Unlink a node from its parent and siblings.
    fn unlink(&self);
}

impl<'a, T: 'a> TreeOperations<'a, T> for Node<'a, T> {
    fn append_child(&'a self, child: &'a Node<'a, T>) {
        self.append(child);
    }

    fn unlink(&self) {
        self.detach();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_creation() {
        let node = Node::new(RefCell::new(42));
        assert_eq!(*node.data.borrow(), 42);
    }

    #[test]
    fn test_append() {
        let arena = typed_arena::Arena::new();
        let parent = arena.alloc(Node::new(RefCell::new("parent")));
        let child = arena.alloc(Node::new(RefCell::new("child")));

        parent.append(child);

        assert_eq!(
            parent.first_child().map(|n| *n.data.borrow()),
            Some("child")
        );
        assert_eq!(parent.last_child().map(|n| *n.data.borrow()), Some("child"));
        assert_eq!(child.parent().map(|n| *n.data.borrow()), Some("parent"));
    }

    #[test]
    fn test_prepend() {
        let arena = typed_arena::Arena::new();
        let parent = arena.alloc(Node::new(RefCell::new("parent")));
        let child1 = arena.alloc(Node::new(RefCell::new("child1")));
        let child2 = arena.alloc(Node::new(RefCell::new("child2")));

        parent.append(child1);
        parent.prepend(child2);

        assert_eq!(
            parent.first_child().map(|n| *n.data.borrow()),
            Some("child2")
        );
        assert_eq!(
            parent.last_child().map(|n| *n.data.borrow()),
            Some("child1")
        );
    }

    #[test]
    fn test_detach() {
        let arena = typed_arena::Arena::new();
        let parent = arena.alloc(Node::new(RefCell::new("parent")));
        let child = arena.alloc(Node::new(RefCell::new("child")));

        parent.append(child);
        child.detach();

        assert!(parent.first_child().is_none());
        assert!(child.parent().is_none());
    }

    #[test]
    fn test_insert_after() {
        let arena = typed_arena::Arena::new();
        let parent = arena.alloc(Node::new(RefCell::new("parent")));
        let child1 = arena.alloc(Node::new(RefCell::new("child1")));
        let child2 = arena.alloc(Node::new(RefCell::new("child2")));

        parent.append(child1);
        child1.insert_after(child2);

        assert_eq!(
            child1.next_sibling().map(|n| *n.data.borrow()),
            Some("child2")
        );
        assert_eq!(
            child2.previous_sibling().map(|n| *n.data.borrow()),
            Some("child1")
        );
    }

    #[test]
    fn test_insert_before() {
        let arena = typed_arena::Arena::new();
        let parent = arena.alloc(Node::new(RefCell::new("parent")));
        let child1 = arena.alloc(Node::new(RefCell::new("child1")));
        let child2 = arena.alloc(Node::new(RefCell::new("child2")));

        parent.append(child1);
        child1.insert_before(child2);

        assert_eq!(
            child2.next_sibling().map(|n| *n.data.borrow()),
            Some("child1")
        );
        assert_eq!(
            child1.previous_sibling().map(|n| *n.data.borrow()),
            Some("child2")
        );
    }

    #[test]
    fn test_descendants() {
        let arena = typed_arena::Arena::new();
        let root = arena.alloc(Node::new(RefCell::new(1)));
        let child1 = arena.alloc(Node::new(RefCell::new(2)));
        let child2 = arena.alloc(Node::new(RefCell::new(3)));
        let grandchild = arena.alloc(Node::new(RefCell::new(4)));

        root.append(child1);
        root.append(child2);
        child1.append(grandchild);

        let values: Vec<i32> = root.descendants().map(|n| *n.data.borrow()).collect();

        assert_eq!(values, vec![1, 2, 4, 3]);
    }

    #[test]
    fn test_children() {
        let arena = typed_arena::Arena::new();
        let parent = arena.alloc(Node::new(RefCell::new("parent")));
        let child1 = arena.alloc(Node::new(RefCell::new("child1")));
        let child2 = arena.alloc(Node::new(RefCell::new("child2")));

        parent.append(child1);
        parent.append(child2);

        let children: Vec<&str> = parent.children().map(|n| *n.data.borrow()).collect();

        assert_eq!(children, vec!["child1", "child2"]);
    }

    #[test]
    fn test_ancestors() {
        let arena = typed_arena::Arena::new();
        let grandparent = arena.alloc(Node::new(RefCell::new("grandparent")));
        let parent = arena.alloc(Node::new(RefCell::new("parent")));
        let child = arena.alloc(Node::new(RefCell::new("child")));

        grandparent.append(parent);
        parent.append(child);

        let ancestors: Vec<&str> = child.ancestors().map(|n| *n.data.borrow()).collect();

        assert_eq!(ancestors, vec!["child", "parent", "grandparent"]);
    }

    #[test]
    fn test_same_node() {
        let arena = typed_arena::Arena::new();
        let node1 = arena.alloc(Node::new(RefCell::new(1)));
        let node2 = arena.alloc(Node::new(RefCell::new(1)));

        assert!(node1.same_node(node1));
        assert!(!node1.same_node(node2));
    }
}
