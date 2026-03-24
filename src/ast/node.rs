//! AST Node base class
//!
//! Design inspired by flexmark-java's Node class.
//! Provides a doubly-linked tree structure with parent-child relationships.

use std::cell::RefCell;
use std::rc::{Rc, Weak};

/// Source position information for a node
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SourcePos {
    pub start_line: u32,
    pub start_column: u32,
    pub end_line: u32,
    pub end_column: u32,
}

/// A node in the AST
///
/// This is the base class for all AST nodes. It provides:
/// - Parent-child relationships (doubly-linked)
/// - Sibling relationships (doubly-linked)
/// - Source position tracking
/// - Tree traversal methods
///
/// Design reference: flexmark-java's Node class
pub struct Node {
    // Node links (using RefCell for interior mutability)
    parent: RefCell<Option<Weak<RefCell<Node>>>>,
    first_child: RefCell<Option<Rc<RefCell<Node>>>>,
    last_child: RefCell<Option<Rc<RefCell<Node>>>>,
    prev: RefCell<Option<Weak<RefCell<Node>>>>,
    next: RefCell<Option<Rc<RefCell<Node>>>>,

    // Source position information
    source_pos: RefCell<SourcePos>,
}

impl Node {
    /// Create a new node
    pub fn new() -> Self {
        Self {
            parent: RefCell::new(None),
            first_child: RefCell::new(None),
            last_child: RefCell::new(None),
            prev: RefCell::new(None),
            next: RefCell::new(None),
            source_pos: RefCell::new(SourcePos::default()),
        }
    }

    /// Get the parent node
    pub fn parent(&self) -> Option<Rc<RefCell<Node>>> {
        self.parent.borrow().as_ref().and_then(|w| w.upgrade())
    }

    /// Set the parent node (internal use)
    pub(crate) fn set_parent(&self, parent: Option<Weak<RefCell<Node>>>) {
        *self.parent.borrow_mut() = parent;
    }

    /// Get the first child node
    pub fn first_child(&self) -> Option<Rc<RefCell<Node>>> {
        self.first_child.borrow().clone()
    }

    /// Set the first child node (internal use)
    pub(crate) fn set_first_child(&self, child: Option<Rc<RefCell<Node>>>) {
        *self.first_child.borrow_mut() = child;
    }

    /// Get the last child node
    pub fn last_child(&self) -> Option<Rc<RefCell<Node>>> {
        self.last_child.borrow().clone()
    }

    /// Set the last child node (internal use)
    pub(crate) fn set_last_child(&self, child: Option<Rc<RefCell<Node>>>) {
        *self.last_child.borrow_mut() = child;
    }

    /// Get the previous sibling node
    pub fn prev(&self) -> Option<Rc<RefCell<Node>>> {
        self.prev.borrow().as_ref().and_then(|w| w.upgrade())
    }

    /// Set the previous sibling (internal use)
    pub(crate) fn set_prev(&self, prev: Option<Weak<RefCell<Node>>>) {
        *self.prev.borrow_mut() = prev;
    }

    /// Get the next sibling node
    pub fn next(&self) -> Option<Rc<RefCell<Node>>> {
        self.next.borrow().clone()
    }

    /// Set the next sibling (internal use)
    pub(crate) fn set_next(&self, next: Option<Rc<RefCell<Node>>>) {
        *self.next.borrow_mut() = next;
    }

    /// Get the source position
    pub fn source_pos(&self) -> SourcePos {
        *self.source_pos.borrow()
    }

    /// Set the source position
    pub fn set_source_pos(&self, pos: SourcePos) {
        *self.source_pos.borrow_mut() = pos;
    }

    /// Get an iterator over the children of this node
    pub fn children(&self) -> ChildrenIterator {
        ChildrenIterator {
            current: self.first_child.borrow().clone(),
        }
    }

    /// Check if this node has children
    pub fn has_children(&self) -> bool {
        self.first_child.borrow().is_some()
    }

    /// Get the number of children
    pub fn child_count(&self) -> usize {
        let mut count = 0;
        let mut current = self.first_child.borrow().clone();
        while let Some(node) = current {
            count += 1;
            current = node.borrow().next();
        }
        count
    }
}

impl Default for Node {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node")
            .field("source_pos", &self.source_pos())
            .field("has_children", &self.has_children())
            .field("child_count", &self.child_count())
            .finish()
    }
}

/// Iterator over the children of a node
pub struct ChildrenIterator {
    current: Option<Rc<RefCell<Node>>>,
}

impl Iterator for ChildrenIterator {
    type Item = Rc<RefCell<Node>>;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current.clone()?;
        self.current = current.borrow().next();
        Some(current)
    }
}

/// Iterator over all descendants of a node (depth-first)
pub struct DescendantIterator {
    stack: Vec<(Rc<RefCell<Node>>, bool)>, // (node, entering)
}

impl DescendantIterator {
    pub fn new(root: Rc<RefCell<Node>>) -> Self {
        Self {
            stack: vec![(root, true)],
        }
    }
}

impl Iterator for DescendantIterator {
    type Item = (Rc<RefCell<Node>>, bool); // (node, entering)

    fn next(&mut self) -> Option<Self::Item> {
        let (node, entering) = self.stack.pop()?;

        if entering {
            // Push the exit event first (will be processed after children)
            self.stack.push((node.clone(), false));

            // Push children in reverse order (so first child is processed first)
            let mut child = node.borrow().last_child();
            while let Some(c) = child {
                self.stack.push((c.clone(), true));
                child = c.borrow().prev();
            }
        }

        Some((node, entering))
    }
}

/// Helper functions for node operations

/// Append a child to a parent node and set up proper parent relationship
pub fn append_child(parent: &Rc<RefCell<Node>>, child: Rc<RefCell<Node>>) {
    // Set child's parent
    child.borrow_mut().set_parent(Some(Rc::downgrade(parent)));

    let last_child_opt = parent.borrow().last_child();

    if let Some(last_child) = last_child_opt {
        // Link child to previous last child
        child.borrow_mut().set_prev(Some(Rc::downgrade(&last_child)));
        last_child.borrow_mut().set_next(Some(child.clone()));
    } else {
        // No children yet, set as first child
        parent.borrow_mut().set_first_child(Some(child.clone()));
    }

    // Always update last_child
    parent.borrow_mut().set_last_child(Some(child));
}

/// Prepend a child to a parent node
pub fn prepend_child(parent: &Rc<RefCell<Node>>, child: Rc<RefCell<Node>>) {
    // Set child's parent
    child.borrow_mut().set_parent(Some(Rc::downgrade(parent)));

    let first_child_opt = parent.borrow().first_child();

    if let Some(first_child) = first_child_opt {
        // Link child to current first child
        child.borrow_mut().set_next(Some(first_child.clone()));
        first_child.borrow_mut().set_prev(Some(Rc::downgrade(&child)));
    } else {
        // No children yet, set as last child too
        parent.borrow_mut().set_last_child(Some(child.clone()));
    }

    // Always update first_child
    parent.borrow_mut().set_first_child(Some(child));
}

/// Insert a sibling after a node
pub fn insert_after(node: &Rc<RefCell<Node>>, sibling: Rc<RefCell<Node>>) {
    // Set sibling's parent from node
    let parent_weak = node.borrow().parent.clone().into_inner();
    sibling.borrow_mut().set_parent(parent_weak.clone());

    // Get node's next sibling (if any)
    let next_opt = node.borrow().next();

    if let Some(next) = next_opt {
        // Link sibling between node and next
        sibling.borrow_mut().set_next(Some(next.clone()));
        next.borrow_mut().set_prev(Some(Rc::downgrade(&sibling)));
    } else if let Some(parent_weak) = parent_weak {
        // Node was the last child, update parent's last_child
        if let Some(parent) = parent_weak.upgrade() {
            parent.borrow_mut().set_last_child(Some(sibling.clone()));
        }
    }

    // Link sibling to node
    sibling.borrow_mut().set_prev(Some(Rc::downgrade(node)));
    node.borrow_mut().set_next(Some(sibling));
}

/// Insert a sibling before a node
pub fn insert_before(node: &Rc<RefCell<Node>>, sibling: Rc<RefCell<Node>>) {
    // Set sibling's parent from node
    let parent_weak = node.borrow().parent.clone().into_inner();
    sibling.borrow_mut().set_parent(parent_weak.clone());

    // Get node's previous sibling (if any)
    let prev_weak_opt = node.borrow().prev().map(|p| Rc::downgrade(&p));

    if let Some(prev_weak) = prev_weak_opt {
        // Link sibling between prev and node
        if let Some(prev) = prev_weak.upgrade() {
            sibling.borrow_mut().set_prev(Some(Rc::downgrade(&prev)));
            prev.borrow_mut().set_next(Some(sibling.clone()));
        }
    } else if let Some(parent_weak) = parent_weak {
        // Node was the first child, update parent's first_child
        if let Some(parent) = parent_weak.upgrade() {
            parent.borrow_mut().set_first_child(Some(sibling.clone()));
        }
    }

    // Link sibling to node
    sibling.borrow_mut().set_next(Some(node.clone()));
    node.borrow_mut().set_prev(Some(Rc::downgrade(&sibling)));
}

/// Unlink a node from its parent and siblings
pub fn unlink(node: &Rc<RefCell<Node>>) {
    // Get references we need before making any changes
    let prev_weak_opt = node.borrow().prev().map(|p| Rc::downgrade(&p));
    let next_opt = node.borrow().next();
    let parent_weak_opt = node.borrow().parent.clone().into_inner();

    // Update previous node's next pointer
    if let Some(ref prev_weak) = prev_weak_opt {
        if let Some(prev) = prev_weak.upgrade() {
            prev.borrow_mut().set_next(next_opt.clone());
        }
    } else if let Some(parent_weak) = &parent_weak_opt {
        // Node is first child, update parent's first_child
        if let Some(parent) = parent_weak.upgrade() {
            parent.borrow_mut().set_first_child(next_opt.clone());
        }
    }

    // Update next node's prev pointer
    if let Some(next) = &next_opt {
        let prev_for_next = prev_weak_opt.clone().and_then(|w| w.upgrade()).map(|n| Rc::downgrade(&n));
        next.borrow_mut().set_prev(prev_for_next);
    } else if let Some(parent_weak) = &parent_weak_opt {
        // Node is last child, update parent's last_child
        if let Some(parent) = parent_weak.upgrade() {
            let new_last = prev_weak_opt.as_ref().and_then(|w| w.upgrade());
            parent.borrow_mut().set_last_child(new_last);
        }
    }

    // Clear this node's connections
    node.borrow_mut().set_parent(None);
    node.borrow_mut().set_next(None);
    node.borrow_mut().set_prev(None);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_creation() {
        let node = Node::new();
        assert!(!node.has_children());
        assert_eq!(node.child_count(), 0);
    }

    #[test]
    fn test_append_child() {
        let parent = Rc::new(RefCell::new(Node::new()));
        let child = Rc::new(RefCell::new(Node::new()));

        append_child(&parent, child.clone());

        assert!(parent.borrow().first_child().is_some());
        assert!(parent.borrow().last_child().is_some());
        assert!(child.borrow().parent().is_some());
        assert_eq!(parent.borrow().child_count(), 1);
    }

    #[test]
    fn test_prepend_child() {
        let parent = Rc::new(RefCell::new(Node::new()));
        let child1 = Rc::new(RefCell::new(Node::new()));
        let child2 = Rc::new(RefCell::new(Node::new()));

        append_child(&parent, child1.clone());
        prepend_child(&parent, child2.clone());

        // child2 should be first, child1 should be last
        assert!(Rc::ptr_eq(
            parent.borrow().first_child().as_ref().unwrap(),
            &child2
        ));
        assert!(Rc::ptr_eq(
            parent.borrow().last_child().as_ref().unwrap(),
            &child1
        ));
    }

    #[test]
    fn test_insert_after() {
        let parent = Rc::new(RefCell::new(Node::new()));
        let child1 = Rc::new(RefCell::new(Node::new()));
        let child2 = Rc::new(RefCell::new(Node::new()));
        let child3 = Rc::new(RefCell::new(Node::new()));

        append_child(&parent, child1.clone());
        append_child(&parent, child2.clone());
        insert_after(&child1, child3.clone());

        // Order should be: child1 -> child3 -> child2
        assert!(Rc::ptr_eq(
            child1.borrow().next().as_ref().unwrap(),
            &child3
        ));
        let child3_prev_binding = child3.borrow().prev();
        let child3_prev = child3_prev_binding.as_ref().unwrap();
        assert!(Rc::ptr_eq(child3_prev, &child1));
        assert!(Rc::ptr_eq(
            child3.borrow().next().as_ref().unwrap(),
            &child2
        ));
    }

    #[test]
    fn test_insert_before() {
        let parent = Rc::new(RefCell::new(Node::new()));
        let child1 = Rc::new(RefCell::new(Node::new()));
        let child2 = Rc::new(RefCell::new(Node::new()));
        let child3 = Rc::new(RefCell::new(Node::new()));

        append_child(&parent, child1.clone());
        append_child(&parent, child2.clone());
        insert_before(&child2, child3.clone());

        // Order should be: child1 -> child3 -> child2
        assert!(Rc::ptr_eq(
            child1.borrow().next().as_ref().unwrap(),
            &child3
        ));
        let child3_prev_binding = child3.borrow().prev();
        let child3_prev = child3_prev_binding.as_ref().unwrap();
        assert!(Rc::ptr_eq(child3_prev, &child1));
        assert!(Rc::ptr_eq(
            child3.borrow().next().as_ref().unwrap(),
            &child2
        ));
    }

    #[test]
    fn test_unlink() {
        let parent = Rc::new(RefCell::new(Node::new()));
        let child = Rc::new(RefCell::new(Node::new()));

        append_child(&parent, child.clone());
        unlink(&child);

        assert!(parent.borrow().first_child().is_none());
        assert!(child.borrow().parent().is_none());
    }

    #[test]
    fn test_children_iterator() {
        let parent = Rc::new(RefCell::new(Node::new()));
        let child1 = Rc::new(RefCell::new(Node::new()));
        let child2 = Rc::new(RefCell::new(Node::new()));
        let child3 = Rc::new(RefCell::new(Node::new()));

        append_child(&parent, child1.clone());
        append_child(&parent, child2.clone());
        append_child(&parent, child3.clone());

        let children: Vec<_> = parent.borrow().children().collect();
        assert_eq!(children.len(), 3);
        assert!(Rc::ptr_eq(&children[0], &child1));
        assert!(Rc::ptr_eq(&children[1], &child2));
        assert!(Rc::ptr_eq(&children[2], &child3));
    }

    #[test]
    fn test_descendant_iterator() {
        let root = Rc::new(RefCell::new(Node::new()));
        let child = Rc::new(RefCell::new(Node::new()));
        let grandchild = Rc::new(RefCell::new(Node::new()));

        append_child(&root, child.clone());
        append_child(&child, grandchild.clone());

        let events: Vec<_> = DescendantIterator::new(root.clone()).collect();
        // Should be: root(enter), child(enter), grandchild(enter), grandchild(exit), child(exit), root(exit)
        assert_eq!(events.len(), 6);
        assert!(Rc::ptr_eq(&events[0].0, &root) && events[0].1); // root enter
        assert!(Rc::ptr_eq(&events[1].0, &child) && events[1].1); // child enter
        assert!(Rc::ptr_eq(&events[2].0, &grandchild) && events[2].1); // grandchild enter
        assert!(Rc::ptr_eq(&events[3].0, &grandchild) && !events[3].1); // grandchild exit
        assert!(Rc::ptr_eq(&events[4].0, &child) && !events[4].1); // child exit
        assert!(Rc::ptr_eq(&events[5].0, &root) && !events[5].1); // root exit
    }

    #[test]
    fn test_source_pos() {
        let node = Node::new();
        let pos = SourcePos {
            start_line: 1,
            start_column: 2,
            end_line: 3,
            end_column: 4,
        };
        node.set_source_pos(pos);
        assert_eq!(node.source_pos(), pos);
    }
}
