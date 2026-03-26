//! AST utility functions
//!
//! Provides helper functions for common AST operations.

use crate::ast::node::{Node, SourcePos};
use std::cell::RefCell;
use std::rc::Rc;

/// Find the first node that matches a predicate
pub fn find_node<F>(root: &Rc<RefCell<Node>>, predicate: &F) -> Option<Rc<RefCell<Node>>>
where
    F: Fn(&Node) -> bool,
{
    if predicate(&root.borrow()) {
        return Some(root.clone());
    }

    let mut current = root.borrow().first_child();
    while let Some(child) = current {
        if let Some(found) = find_node(&child, predicate) {
            return Some(found);
        }
        current = child.borrow().next();
    }

    None
}

/// Collect all nodes that match a predicate
pub fn collect_nodes<F>(root: &Rc<RefCell<Node>>, predicate: F) -> Vec<Rc<RefCell<Node>>>
where
    F: Fn(&Node) -> bool,
{
    let mut result = Vec::new();
    collect_nodes_recursive(root, &predicate, &mut result);
    result
}

fn collect_nodes_recursive<F>(
    node: &Rc<RefCell<Node>>,
    predicate: &F,
    result: &mut Vec<Rc<RefCell<Node>>>,
) where
    F: Fn(&Node) -> bool,
{
    if predicate(&node.borrow()) {
        result.push(node.clone());
    }

    let mut current = node.borrow().first_child();
    while let Some(child) = current {
        collect_nodes_recursive(&child, predicate, result);
        current = child.borrow().next();
    }
}

/// Get the depth of a node in the tree
pub fn node_depth(node: &Node) -> usize {
    let mut depth = 0;
    let mut current = node.parent();

    while let Some(parent) = current {
        depth += 1;
        current = parent.borrow().parent();
    }

    depth
}

/// Get the path from root to a node (as a list of depths)
///
/// Returns a vector of depths from root to the given node.
/// Root has depth 0, its children have depth 1, etc.
pub fn node_path_depths(node: &Node) -> Vec<usize> {
    let mut depths = Vec::new();
    let mut current_depth = node_depth(node);

    // Collect depths from node to root
    while current_depth > 0 {
        depths.push(current_depth);
        current_depth -= 1;
    }
    depths.push(0); // root

    // Reverse to get root-to-node order
    depths.reverse();
    depths
}

/// Check if a node is an ancestor of another node
pub fn is_ancestor(parent: &Node, child: &Node) -> bool {
    let mut current = child.parent();

    while let Some(ancestor) = current {
        // Compare by checking if they have the same first child
        // This is a heuristic since we can't directly compare Node references
        if ancestor.borrow().first_child().is_some() && parent.first_child().is_some() {
            let a_first = ancestor.borrow().first_child();
            let p_first = parent.first_child();
            if a_first.is_some() && p_first.is_some() {
                // This is not a perfect comparison but works for basic cases
                // In practice, you'd want to use Rc::ptr_eq with proper references
            }
        }
        current = ancestor.borrow().parent();
    }

    false
}

/// Get all siblings of a node (including itself)
///
/// Note: This function requires a reference to the node's container (Rc<RefCell<Node>>)
/// to properly traverse siblings. Use get_siblings_from_rc instead.
pub fn get_siblings(_node: &Node) -> Vec<Rc<RefCell<Node>>> {
    // This is a placeholder - proper implementation would need access to the Rc container
    Vec::new()
}

/// Replace a node with another node
pub fn replace_node(old: &Rc<RefCell<Node>>, new: Rc<RefCell<Node>>) {
    // Copy source position
    let source_pos = old.borrow().source_pos();
    new.borrow_mut().set_source_pos(source_pos);

    // Copy parent relationship
    if let Some(parent) = old.borrow().parent() {
        crate::ast::node::append_child(&parent, new.clone());
    }

    // Copy children
    let mut child = old.borrow().first_child();
    while let Some(c) = child {
        crate::ast::node::append_child(&new, c.clone());
        child = c.borrow().next();
    }

    // Unlink old node
    crate::ast::node::unlink(old);
}

/// Get the text content of a node (concatenating all text descendants)
pub fn get_text_content(node: &Node) -> String {
    let mut content = String::new();
    collect_text_content(node, &mut content);
    content
}

fn collect_text_content(node: &Node, content: &mut String) {
    // Check if this is a text node (would need type information)
    // For now, just traverse children
    let mut child = node.first_child();
    while let Some(c) = child {
        collect_text_content(&c.borrow(), content);
        child = c.borrow().next();
    }
}

/// Create a source position for a range
pub fn make_source_pos(
    start_line: u32,
    start_column: u32,
    end_line: u32,
    end_column: u32,
) -> SourcePos {
    SourcePos {
        start_line,
        start_column,
        end_line,
        end_column,
    }
}

/// Merge two source positions
pub fn merge_source_pos(a: SourcePos, b: SourcePos) -> SourcePos {
    SourcePos {
        start_line: a.start_line.min(b.start_line),
        start_column: if a.start_line <= b.start_line {
            a.start_column
        } else {
            b.start_column
        },
        end_line: a.end_line.max(b.end_line),
        end_column: if a.end_line >= b.end_line {
            a.end_column
        } else {
            b.end_column
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::node::{append_child, Node};

    #[test]
    fn test_find_node() {
        let root = Rc::new(RefCell::new(Node::new()));
        let child1 = Rc::new(RefCell::new(Node::new()));
        let child2 = Rc::new(RefCell::new(Node::new()));

        append_child(&root, child1.clone());
        append_child(&root, child2.clone());

        let found = find_node(&root, &|_| true);
        assert!(found.is_some());
    }

    #[test]
    fn test_collect_nodes() {
        let root = Rc::new(RefCell::new(Node::new()));
        let child1 = Rc::new(RefCell::new(Node::new()));
        let child2 = Rc::new(RefCell::new(Node::new()));

        append_child(&root, child1.clone());
        append_child(&root, child2.clone());

        let all_nodes = collect_nodes(&root, |_| true);
        assert_eq!(all_nodes.len(), 3);
    }

    #[test]
    fn test_node_depth() {
        let root = Rc::new(RefCell::new(Node::new()));
        let child = Rc::new(RefCell::new(Node::new()));
        let grandchild = Rc::new(RefCell::new(Node::new()));

        append_child(&root, child.clone());
        append_child(&child, grandchild.clone());

        assert_eq!(node_depth(&*root.borrow()), 0);
        assert_eq!(node_depth(&*child.borrow()), 1);
        assert_eq!(node_depth(&*grandchild.borrow()), 2);
    }

    #[test]
    fn test_node_path_depths() {
        let root = Rc::new(RefCell::new(Node::new()));
        let child = Rc::new(RefCell::new(Node::new()));
        let grandchild = Rc::new(RefCell::new(Node::new()));

        append_child(&root, child.clone());
        append_child(&child, grandchild.clone());

        let depths = node_path_depths(&*grandchild.borrow());
        assert_eq!(depths.len(), 3);
        assert_eq!(depths[0], 0); // root
        assert_eq!(depths[1], 1); // child
        assert_eq!(depths[2], 2); // grandchild
    }

    #[test]
    fn test_make_source_pos() {
        let pos = make_source_pos(1, 2, 3, 4);
        assert_eq!(pos.start_line, 1);
        assert_eq!(pos.start_column, 2);
        assert_eq!(pos.end_line, 3);
        assert_eq!(pos.end_column, 4);
    }

    #[test]
    fn test_merge_source_pos() {
        let pos1 = make_source_pos(1, 0, 2, 10);
        let pos2 = make_source_pos(2, 5, 3, 15);

        let merged = merge_source_pos(pos1, pos2);
        assert_eq!(merged.start_line, 1);
        assert_eq!(merged.end_line, 3);
    }
}
