//! AST iterator for traversing the CommonMark document tree
//!
//! This module provides an iterator for traversing the AST in a depth-first manner,
//! yielding enter and exit events for each node.
//!
//! # Example
//!
//! ```rust,ignore
//! use clmd::iterator::{NodeIterator, EventType};
//!
//! let iter = NodeIterator::new(root_node);
//! while iter.next() != EventType::Done {
//!     // Process enter/exit events
//! }
//! ```

use crate::node::{unlink, Node, NodeType};
use std::cell::RefCell;
use std::rc::Rc;

/// Event type for tree iteration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    None,
    Done,
    Enter,
    Exit,
}

/// Iterator for traversing the AST
pub struct NodeIterator {
    root: Rc<RefCell<Node>>,
    current: Option<Rc<RefCell<Node>>>,
    event_type: EventType,
}

impl NodeIterator {
    pub fn new(root: Rc<RefCell<Node>>) -> Self {
        NodeIterator {
            root: root.clone(),
            current: None,
            event_type: EventType::None,
        }
    }

    pub fn next(&mut self) -> EventType {
        if self.event_type == EventType::None {
            // First call - start at root
            self.current = Some(self.root.clone());
            self.event_type = EventType::Enter;
            return EventType::Enter;
        }

        if let Some(ref current) = self.current {
            match self.event_type {
                EventType::Enter => {
                    // Try to go to first child
                    let first_child = current.borrow().first_child.borrow().clone();
                    if let Some(first_child) = first_child {
                        self.current = Some(first_child);
                        self.event_type = EventType::Enter;
                        EventType::Enter
                    } else {
                        // Leaf node - return Exit immediately
                        self.event_type = EventType::Exit;
                        EventType::Exit
                    }
                }
                EventType::Exit => {
                    // Check if we're back at root after exiting it
                    if Rc::ptr_eq(current, &self.root) {
                        self.event_type = EventType::Done;
                        return EventType::Done;
                    }

                    // Try to go to next sibling
                    let next = current.borrow().next.borrow().clone();
                    if let Some(next) = next {
                        self.current = Some(next);
                        self.event_type = EventType::Enter;
                        EventType::Enter
                    } else {
                        // Go back to parent
                        let parent = current.borrow().parent.borrow().clone();
                        if let Some(parent) = parent {
                            if let Some(parent) = parent.upgrade() {
                                self.current = Some(parent);
                                self.event_type = EventType::Exit;
                                return EventType::Exit;
                            }
                        }
                        self.event_type = EventType::Done;
                        EventType::Done
                    }
                }
                _ => EventType::Done,
            }
        } else {
            EventType::Done
        }
    }

    pub fn get_node(&self) -> Option<Rc<RefCell<Node>>> {
        self.current.clone()
    }

    pub fn get_event_type(&self) -> EventType {
        self.event_type
    }

    pub fn reset(&mut self, current: Rc<RefCell<Node>>, event_type: EventType) {
        self.current = Some(current);
        self.event_type = event_type;
    }
}

/// A walker that can be used to iterate through the node tree
pub struct NodeWalker {
    #[allow(dead_code)]
    root: Rc<RefCell<Node>>,
    iterator: NodeIterator,
}

impl NodeWalker {
    pub fn new(root: Rc<RefCell<Node>>) -> Self {
        NodeWalker {
            root: root.clone(),
            iterator: NodeIterator::new(root),
        }
    }

    pub fn next(&mut self) -> Option<WalkerEvent> {
        let event_type = self.iterator.next();
        if event_type == EventType::Done {
            None
        } else {
            self.iterator.get_node().map(|node| WalkerEvent {
                node,
                entering: event_type == EventType::Enter,
            })
        }
    }

    pub fn resume_at(&mut self, node: Rc<RefCell<Node>>, entering: bool) {
        let event_type = if entering {
            EventType::Enter
        } else {
            EventType::Exit
        };
        self.iterator.reset(node, event_type);
    }
}

/// Event returned by the walker
pub struct WalkerEvent {
    pub node: Rc<RefCell<Node>>,
    pub entering: bool,
}

/// Consolidate adjacent text nodes in the tree
pub fn consolidate_text_nodes(root: &Rc<RefCell<Node>>) {
    let mut walker = NodeWalker::new(root.clone());

    while let Some(event) = walker.next() {
        if event.entering {
            let node = event.node.borrow();
            if node.node_type == NodeType::Text {
                drop(node);
                consolidate_text_node(&event.node);
            }
        }
    }
}

fn consolidate_text_node(node: &Rc<RefCell<Node>>) {
    let mut current = node.borrow().next.borrow().clone();

    while let Some(ref next_node) = current {
        let next_type = next_node.borrow().node_type;
        if next_type != NodeType::Text {
            break;
        }

        // Append next node's literal to current node
        let next_literal = match &next_node.borrow().data {
            crate::node::NodeData::Text { literal } => literal.clone(),
            _ => String::new(),
        };

        if let crate::node::NodeData::Text { ref mut literal } = node.borrow_mut().data {
            literal.push_str(&next_literal);
        }

        // Remove next node
        let next_next = next_node.borrow().next.borrow().clone();
        unlink(next_node);

        current = next_next;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::{append_child, Node, NodeData, NodeType};

    #[test]
    fn test_iterator_basic() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let para = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        let text = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "Hello".to_string(),
            },
        )));

        append_child(&root, para.clone());
        append_child(&para, text.clone());

        let mut iter = NodeIterator::new(root.clone());

        assert_eq!(iter.next(), EventType::Enter);
        assert!(Rc::ptr_eq(&iter.get_node().unwrap(), &root));

        assert_eq!(iter.next(), EventType::Enter);
        assert!(Rc::ptr_eq(&iter.get_node().unwrap(), &para));

        assert_eq!(iter.next(), EventType::Enter);
        assert!(Rc::ptr_eq(&iter.get_node().unwrap(), &text));

        assert_eq!(iter.next(), EventType::Exit);
        assert!(Rc::ptr_eq(&iter.get_node().unwrap(), &text));

        assert_eq!(iter.next(), EventType::Exit);
        assert!(Rc::ptr_eq(&iter.get_node().unwrap(), &para));

        assert_eq!(iter.next(), EventType::Exit);
        assert!(Rc::ptr_eq(&iter.get_node().unwrap(), &root));

        assert_eq!(iter.next(), EventType::Done);
    }

    #[test]
    fn test_walker() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let para = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        let text = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "Hello".to_string(),
            },
        )));

        append_child(&root, para.clone());
        append_child(&para, text.clone());

        let mut walker = NodeWalker::new(root.clone());
        let mut events = Vec::new();

        while let Some(event) = walker.next() {
            events.push((event.node.borrow().node_type, event.entering));
        }

        // Document(Enter) -> Paragraph(Enter) -> Text(Enter) -> Text(Exit) -> Paragraph(Exit) -> Document(Exit)
        assert_eq!(events.len(), 6);
        assert_eq!(events[0], (NodeType::Document, true));
        assert_eq!(events[1], (NodeType::Paragraph, true));
        assert_eq!(events[2], (NodeType::Text, true));
        assert_eq!(events[3], (NodeType::Text, false));
        assert_eq!(events[4], (NodeType::Paragraph, false));
        assert_eq!(events[5], (NodeType::Document, false));
    }

    #[test]
    fn test_iterator_empty_document() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let mut iter = NodeIterator::new(root.clone());

        assert_eq!(iter.next(), EventType::Enter);
        assert_eq!(iter.next(), EventType::Exit);
        assert_eq!(iter.next(), EventType::Done);
    }

    #[test]
    fn test_iterator_multiple_siblings() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let para1 = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        let para2 = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        let para3 = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));

        append_child(&root, para1.clone());
        append_child(&root, para2.clone());
        append_child(&root, para3.clone());

        let mut walker = NodeWalker::new(root.clone());
        let mut events = Vec::new();

        while let Some(event) = walker.next() {
            events.push((event.node.borrow().node_type, event.entering));
        }

        // Document(Enter) -> Para1(Enter) -> Para1(Exit) -> Para2(Enter) -> Para2(Exit) -> Para3(Enter) -> Para3(Exit) -> Document(Exit)
        assert_eq!(events.len(), 8);
        assert_eq!(events[0], (NodeType::Document, true));
        assert_eq!(events[1], (NodeType::Paragraph, true));
        assert_eq!(events[2], (NodeType::Paragraph, false));
        assert_eq!(events[3], (NodeType::Paragraph, true));
        assert_eq!(events[4], (NodeType::Paragraph, false));
        assert_eq!(events[5], (NodeType::Paragraph, true));
        assert_eq!(events[6], (NodeType::Paragraph, false));
        assert_eq!(events[7], (NodeType::Document, false));
    }

    #[test]
    fn test_iterator_nested_structure() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let blockquote = Rc::new(RefCell::new(Node::new(NodeType::BlockQuote)));
        let list = Rc::new(RefCell::new(Node::new(NodeType::List)));
        let item = Rc::new(RefCell::new(Node::new(NodeType::Item)));
        let para = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        let text = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "Text".to_string(),
            },
        )));

        append_child(&root, blockquote.clone());
        append_child(&blockquote, list.clone());
        append_child(&list, item.clone());
        append_child(&item, para.clone());
        append_child(&para, text.clone());

        let mut walker = NodeWalker::new(root.clone());
        let mut events = Vec::new();

        while let Some(event) = walker.next() {
            events.push((event.node.borrow().node_type, event.entering));
        }

        // Should visit all nodes in depth-first order
        assert_eq!(events.len(), 12);
        assert_eq!(events[0], (NodeType::Document, true));
        assert_eq!(events[1], (NodeType::BlockQuote, true));
        assert_eq!(events[2], (NodeType::List, true));
        assert_eq!(events[3], (NodeType::Item, true));
        assert_eq!(events[4], (NodeType::Paragraph, true));
        assert_eq!(events[5], (NodeType::Text, true));
        assert_eq!(events[6], (NodeType::Text, false));
        assert_eq!(events[7], (NodeType::Paragraph, false));
        assert_eq!(events[8], (NodeType::Item, false));
        assert_eq!(events[9], (NodeType::List, false));
        assert_eq!(events[10], (NodeType::BlockQuote, false));
        assert_eq!(events[11], (NodeType::Document, false));
    }

    #[test]
    fn test_walker_resume_at() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let para1 = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        let para2 = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));

        append_child(&root, para1.clone());
        append_child(&root, para2.clone());

        let mut walker = NodeWalker::new(root.clone());

        // Get first event (Document Enter)
        let event1 = walker.next().unwrap();
        assert_eq!(event1.node.borrow().node_type, NodeType::Document);
        assert!(event1.entering);

        // Get second event (Para1 Enter)
        let event2 = walker.next().unwrap();
        assert_eq!(event2.node.borrow().node_type, NodeType::Paragraph);
        assert!(event2.entering);

        // Resume at para2, entering - this resets the iterator to para2
        walker.resume_at(para2.clone(), true);

        // After resume_at, the iterator returns the current node first
        let current = walker.iterator.get_node();
        assert!(current.is_some());
        assert_eq!(
            current.as_ref().unwrap().borrow().node_type,
            NodeType::Paragraph
        );
        assert!(Rc::ptr_eq(&current.unwrap(), &para2));
    }

    #[test]
    fn test_iterator_get_event_type() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let mut iter = NodeIterator::new(root.clone());

        assert_eq!(iter.get_event_type(), EventType::None);
        iter.next();
        assert_eq!(iter.get_event_type(), EventType::Enter);
        iter.next();
        assert_eq!(iter.get_event_type(), EventType::Exit);
        iter.next();
        assert_eq!(iter.get_event_type(), EventType::Done);
    }

    #[test]
    fn test_iterator_reset() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let para = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        append_child(&root, para.clone());

        let mut iter = NodeIterator::new(root.clone());

        // Move through the tree
        iter.next(); // Enter Document
        iter.next(); // Enter Paragraph

        // Reset to root
        iter.reset(root.clone(), EventType::Enter);
        assert!(Rc::ptr_eq(&iter.get_node().unwrap(), &root));
        assert_eq!(iter.get_event_type(), EventType::Enter);
    }

    #[test]
    fn test_consolidate_text_nodes() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let para = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        let text1 = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "Hello ".to_string(),
            },
        )));
        let text2 = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "world".to_string(),
            },
        )));
        let text3 = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "!".to_string(),
            },
        )));

        append_child(&root, para.clone());
        append_child(&para, text1.clone());
        append_child(&para, text2.clone());
        append_child(&para, text3.clone());

        consolidate_text_nodes(&root);

        // text1 should now contain "Hello world!"
        if let NodeData::Text { literal } = &text1.borrow().data {
            assert_eq!(literal, "Hello world!");
        }

        // text2 and text3 should be unlinked
        assert!(text2.borrow().parent.borrow().is_none());
        assert!(text3.borrow().parent.borrow().is_none());
    }

    #[test]
    fn test_consolidate_text_nodes_non_adjacent() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let para = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        let text1 = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "Hello".to_string(),
            },
        )));
        let emph = Rc::new(RefCell::new(Node::new(NodeType::Emph)));
        let text2 = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "world".to_string(),
            },
        )));

        append_child(&root, para.clone());
        append_child(&para, text1.clone());
        append_child(&para, emph.clone());
        append_child(&para, text2.clone());

        consolidate_text_nodes(&root);

        // text1 and text2 should not be consolidated (separated by emph)
        let text1_content = if let NodeData::Text { literal } = &text1.borrow().data {
            literal.clone()
        } else {
            String::new()
        };
        assert_eq!(text1_content, "Hello");

        let text2_content = if let NodeData::Text { literal } = &text2.borrow().data {
            literal.clone()
        } else {
            String::new()
        };
        assert_eq!(text2_content, "world");
    }

    #[test]
    fn test_walker_with_complex_tree() {
        // Create a more complex tree structure
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));

        // First child: Heading
        let heading = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Heading,
            NodeData::Heading {
                level: 1,
                content: "Title".to_string(),
            },
        )));
        append_child(&root, heading.clone());

        // Second child: List with items
        let list = Rc::new(RefCell::new(Node::new(NodeType::List)));
        append_child(&root, list.clone());

        let item1 = Rc::new(RefCell::new(Node::new(NodeType::Item)));
        let item2 = Rc::new(RefCell::new(Node::new(NodeType::Item)));
        append_child(&list, item1.clone());
        append_child(&list, item2.clone());

        // Add text to items
        let text1 = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "Item 1".to_string(),
            },
        )));
        let text2 = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "Item 2".to_string(),
            },
        )));
        append_child(&item1, text1.clone());
        append_child(&item2, text2.clone());

        let mut walker = NodeWalker::new(root.clone());
        let mut event_count = 0;

        while let Some(_event) = walker.next() {
            event_count += 1;
        }

        // Document(Enter) -> Heading(Enter) -> Heading(Exit) -> List(Enter) -> Item1(Enter) -> Text1(Enter) -> Text1(Exit) -> Item1(Exit) -> Item2(Enter) -> Text2(Enter) -> Text2(Exit) -> Item2(Exit) -> List(Exit) -> Document(Exit)
        assert_eq!(event_count, 14);
    }

    #[test]
    fn test_iterator_single_node() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        let mut iter = NodeIterator::new(root.clone());

        assert_eq!(iter.next(), EventType::Enter);
        assert!(Rc::ptr_eq(&iter.get_node().unwrap(), &root));

        assert_eq!(iter.next(), EventType::Exit);
        assert!(Rc::ptr_eq(&iter.get_node().unwrap(), &root));

        assert_eq!(iter.next(), EventType::Done);
    }

    #[test]
    fn test_walker_returns_none_when_done() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let mut walker = NodeWalker::new(root.clone());

        // Walk through all events
        while walker.next().is_some() {}

        // Should return None after Done
        assert!(walker.next().is_none());
        assert!(walker.next().is_none());
    }
}
