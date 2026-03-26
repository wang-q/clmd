//! AST iterator for traversing the CommonMark document tree
//!
//! This module provides an iterator for traversing the AST in a depth-first manner,
//! yielding enter and exit events for each node.
//!
//! # Example
//!
//! ```rust,ignore
//! use clmd::iterator::{ArenaNodeIterator, EventType};
//!
//! let iter = ArenaNodeIterator::new(&arena, root_id);
//! while iter.next() != EventType::Done {
//!     // Process enter/exit events
//! }
//! ```

use crate::arena::{NodeArena, NodeId, TreeOps};
use crate::node::{NodeData, NodeType};

/// Event type for tree iteration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    None,
    Done,
    Enter,
    Exit,
}

/// Iterator for traversing the Arena-based AST
pub struct ArenaNodeIterator<'a> {
    arena: &'a NodeArena,
    root: NodeId,
    current: Option<NodeId>,
    event_type: EventType,
}

impl<'a> ArenaNodeIterator<'a> {
    pub fn new(arena: &'a NodeArena, root: NodeId) -> Self {
        ArenaNodeIterator {
            arena,
            root,
            current: None,
            event_type: EventType::None,
        }
    }

    pub fn next(&mut self) -> EventType {
        if self.event_type == EventType::None {
            // First call - start at root
            self.current = Some(self.root);
            self.event_type = EventType::Enter;
            return EventType::Enter;
        }

        if let Some(current) = self.current {
            match self.event_type {
                EventType::Enter => {
                    // Try to go to first child
                    let first_child = self.arena.get(current).first_child;
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
                    if current == self.root {
                        self.event_type = EventType::Done;
                        return EventType::Done;
                    }

                    // Try to go to next sibling
                    let next = self.arena.get(current).next;
                    if let Some(next) = next {
                        self.current = Some(next);
                        self.event_type = EventType::Enter;
                        EventType::Enter
                    } else {
                        // Go back to parent
                        let parent = self.arena.get(current).parent;
                        if let Some(parent) = parent {
                            self.current = Some(parent);
                            self.event_type = EventType::Exit;
                            return EventType::Exit;
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

    pub fn get_node(&self) -> Option<NodeId> {
        self.current
    }

    pub fn get_event_type(&self) -> EventType {
        self.event_type
    }

    pub fn reset(&mut self, current: NodeId, event_type: EventType) {
        self.current = Some(current);
        self.event_type = event_type;
    }
}

/// Item type for the standard Iterator implementation
pub type ArenaIteratorItem = (NodeId, EventType);

impl<'a> Iterator for ArenaNodeIterator<'a> {
    type Item = ArenaIteratorItem;

    fn next(&mut self) -> Option<Self::Item> {
        let event = self.next_event();
        if event == EventType::Done {
            None
        } else {
            self.current.map(|node| (node, event))
        }
    }
}

impl<'a> ArenaNodeIterator<'a> {
    /// Original next method, renamed to avoid conflict with Iterator trait
    pub fn next_event(&mut self) -> EventType {
        if self.event_type == EventType::None {
            // First call - start at root
            self.current = Some(self.root);
            self.event_type = EventType::Enter;
            return EventType::Enter;
        }

        if let Some(current) = self.current {
            match self.event_type {
                EventType::Enter => {
                    // Try to go to first child
                    let first_child = self.arena.get(current).first_child;
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
                    if current == self.root {
                        self.event_type = EventType::Done;
                        return EventType::Done;
                    }

                    // Try to go to next sibling
                    let next = self.arena.get(current).next;
                    if let Some(next) = next {
                        self.current = Some(next);
                        self.event_type = EventType::Enter;
                        EventType::Enter
                    } else {
                        // Go back to parent
                        let parent = self.arena.get(current).parent;
                        if let Some(parent) = parent {
                            self.current = Some(parent);
                            self.event_type = EventType::Exit;
                            return EventType::Exit;
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
}

/// A walker that can be used to iterate through the node tree
pub struct ArenaNodeWalker<'a> {
    #[allow(dead_code)]
    root: NodeId,
    iterator: ArenaNodeIterator<'a>,
}

impl<'a> ArenaNodeWalker<'a> {
    pub fn new(arena: &'a NodeArena, root: NodeId) -> Self {
        ArenaNodeWalker {
            root,
            iterator: ArenaNodeIterator::new(arena, root),
        }
    }

    pub fn next(&mut self) -> Option<ArenaWalkerEvent> {
        let event_type = self.iterator.next();
        if event_type == EventType::Done {
            None
        } else {
            self.iterator.get_node().map(|node| ArenaWalkerEvent {
                node,
                entering: event_type == EventType::Enter,
            })
        }
    }

    pub fn resume_at(&mut self, node: NodeId, entering: bool) {
        let event_type = if entering {
            EventType::Enter
        } else {
            EventType::Exit
        };
        self.iterator.reset(node, event_type);
    }
}

/// Event returned by the walker
pub struct ArenaWalkerEvent {
    pub node: NodeId,
    pub entering: bool,
}

/// Consolidate adjacent text nodes in the tree
pub fn consolidate_text_nodes(arena: &mut NodeArena, root: NodeId) {
    // Collect text nodes first to avoid borrow issues
    let mut text_nodes = Vec::new();
    {
        let mut walker = ArenaNodeWalker::new(arena, root);
        while let Some(event) = walker.next() {
            if event.entering {
                let node = arena.get(event.node);
                if node.node_type == NodeType::Text {
                    text_nodes.push(event.node);
                }
            }
        }
    }

    // Now consolidate each text node
    for node_id in text_nodes {
        consolidate_text_node(arena, node_id);
    }
}

fn consolidate_text_node(arena: &mut NodeArena, node: NodeId) {
    let mut current = arena.get(node).next;

    while let Some(next_node_id) = current {
        let next_type = arena.get(next_node_id).node_type;
        if next_type != NodeType::Text {
            break;
        }

        // Append next node's literal to current node
        let next_literal = match &arena.get(next_node_id).data {
            NodeData::Text { literal } => literal.clone(),
            _ => String::new(),
        };

        if let NodeData::Text { ref mut literal } = arena.get_mut(node).data {
            literal.push_str(&next_literal);
        }

        // Remove next node
        let next_next = arena.get(next_node_id).next;
        TreeOps::unlink(arena, next_node_id);

        current = next_next;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arena::{Node, NodeArena, TreeOps};
    use crate::node::{NodeData, NodeType};

    #[test]
    fn test_iterator_basic() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::new(NodeType::Document));
        let para = arena.alloc(Node::new(NodeType::Paragraph));
        let text = arena.alloc(Node::with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "Hello".to_string(),
            },
        ));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, text);

        let mut iter = ArenaNodeIterator::new(&arena, root);

        assert_eq!(iter.next(), EventType::Enter);
        assert_eq!(iter.get_node(), Some(root));

        assert_eq!(iter.next(), EventType::Enter);
        assert_eq!(iter.get_node(), Some(para));

        assert_eq!(iter.next(), EventType::Enter);
        assert_eq!(iter.get_node(), Some(text));

        assert_eq!(iter.next(), EventType::Exit);
        assert_eq!(iter.get_node(), Some(text));

        assert_eq!(iter.next(), EventType::Exit);
        assert_eq!(iter.get_node(), Some(para));

        assert_eq!(iter.next(), EventType::Exit);
        assert_eq!(iter.get_node(), Some(root));

        assert_eq!(iter.next(), EventType::Done);
    }

    #[test]
    fn test_walker() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::new(NodeType::Document));
        let para = arena.alloc(Node::new(NodeType::Paragraph));
        let text = arena.alloc(Node::with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "Hello".to_string(),
            },
        ));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, text);

        let mut walker = ArenaNodeWalker::new(&arena, root);
        let mut events = Vec::new();

        while let Some(event) = walker.next() {
            let node_type = arena.get(event.node).node_type;
            events.push((node_type, event.entering));
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
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::new(NodeType::Document));
        let mut iter = ArenaNodeIterator::new(&arena, root);

        assert_eq!(iter.next(), EventType::Enter);
        assert_eq!(iter.next(), EventType::Exit);
        assert_eq!(iter.next(), EventType::Done);
    }

    #[test]
    fn test_iterator_multiple_siblings() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::new(NodeType::Document));
        let para1 = arena.alloc(Node::new(NodeType::Paragraph));
        let para2 = arena.alloc(Node::new(NodeType::Paragraph));
        let para3 = arena.alloc(Node::new(NodeType::Paragraph));

        TreeOps::append_child(&mut arena, root, para1);
        TreeOps::append_child(&mut arena, root, para2);
        TreeOps::append_child(&mut arena, root, para3);

        let mut walker = ArenaNodeWalker::new(&arena, root);
        let mut events = Vec::new();

        while let Some(event) = walker.next() {
            let node_type = arena.get(event.node).node_type;
            events.push((node_type, event.entering));
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
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::new(NodeType::Document));
        let blockquote = arena.alloc(Node::new(NodeType::BlockQuote));
        let list = arena.alloc(Node::new(NodeType::List));
        let item = arena.alloc(Node::new(NodeType::Item));
        let para = arena.alloc(Node::new(NodeType::Paragraph));
        let text = arena.alloc(Node::with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "Text".to_string(),
            },
        ));

        TreeOps::append_child(&mut arena, root, blockquote);
        TreeOps::append_child(&mut arena, blockquote, list);
        TreeOps::append_child(&mut arena, list, item);
        TreeOps::append_child(&mut arena, item, para);
        TreeOps::append_child(&mut arena, para, text);

        let mut walker = ArenaNodeWalker::new(&arena, root);
        let mut events = Vec::new();

        while let Some(event) = walker.next() {
            let node_type = arena.get(event.node).node_type;
            events.push((node_type, event.entering));
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
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::new(NodeType::Document));
        let para1 = arena.alloc(Node::new(NodeType::Paragraph));
        let para2 = arena.alloc(Node::new(NodeType::Paragraph));

        TreeOps::append_child(&mut arena, root, para1);
        TreeOps::append_child(&mut arena, root, para2);

        let mut walker = ArenaNodeWalker::new(&arena, root);

        // Get first event (Document Enter)
        let event1 = walker.next().unwrap();
        assert_eq!(arena.get(event1.node).node_type, NodeType::Document);
        assert!(event1.entering);

        // Get second event (Para1 Enter)
        let event2 = walker.next().unwrap();
        assert_eq!(arena.get(event2.node).node_type, NodeType::Paragraph);
        assert!(event2.entering);

        // Resume at para2, entering - this resets the iterator to para2
        walker.resume_at(para2, true);

        // After resume_at, the iterator returns the current node first
        let current = walker.iterator.get_node();
        assert_eq!(current, Some(para2));
    }

    #[test]
    fn test_iterator_get_event_type() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::new(NodeType::Document));
        let mut iter = ArenaNodeIterator::new(&arena, root);

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
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::new(NodeType::Document));
        let para = arena.alloc(Node::new(NodeType::Paragraph));
        TreeOps::append_child(&mut arena, root, para);

        let mut iter = ArenaNodeIterator::new(&arena, root);

        // Move through the tree
        iter.next(); // Enter Document
        iter.next(); // Enter Paragraph

        // Reset to root
        iter.reset(root, EventType::Enter);
        assert_eq!(iter.get_node(), Some(root));
        assert_eq!(iter.get_event_type(), EventType::Enter);
    }

    #[test]
    fn test_consolidate_text_nodes() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::new(NodeType::Document));
        let para = arena.alloc(Node::new(NodeType::Paragraph));
        let text1 = arena.alloc(Node::with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "Hello ".to_string(),
            },
        ));
        let text2 = arena.alloc(Node::with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "world".to_string(),
            },
        ));
        let text3 = arena.alloc(Node::with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "!".to_string(),
            },
        ));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, text1);
        TreeOps::append_child(&mut arena, para, text2);
        TreeOps::append_child(&mut arena, para, text3);

        consolidate_text_nodes(&mut arena, root);

        // text1 should now contain "Hello world!"
        if let NodeData::Text { literal } = &arena.get(text1).data {
            assert_eq!(literal, "Hello world!");
        }

        // text2 and text3 should be unlinked
        assert!(arena.get(text2).parent.is_none());
        assert!(arena.get(text3).parent.is_none());
    }

    #[test]
    fn test_consolidate_text_nodes_non_adjacent() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::new(NodeType::Document));
        let para = arena.alloc(Node::new(NodeType::Paragraph));
        let text1 = arena.alloc(Node::with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "Hello".to_string(),
            },
        ));
        let emph = arena.alloc(Node::new(NodeType::Emph));
        let text2 = arena.alloc(Node::with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "world".to_string(),
            },
        ));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, text1);
        TreeOps::append_child(&mut arena, para, emph);
        TreeOps::append_child(&mut arena, para, text2);

        consolidate_text_nodes(&mut arena, root);

        // text1 and text2 should not be consolidated (separated by emph)
        let text1_content = if let NodeData::Text { literal } = &arena.get(text1).data {
            literal.clone()
        } else {
            String::new()
        };
        assert_eq!(text1_content, "Hello");

        let text2_content = if let NodeData::Text { literal } = &arena.get(text2).data {
            literal.clone()
        } else {
            String::new()
        };
        assert_eq!(text2_content, "world");
    }

    #[test]
    fn test_walker_with_complex_tree() {
        // Create a more complex tree structure
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::new(NodeType::Document));

        // First child: Heading
        let heading = arena.alloc(Node::with_data(
            NodeType::Heading,
            NodeData::Heading {
                level: 1,
                content: "Title".to_string(),
            },
        ));
        TreeOps::append_child(&mut arena, root, heading);

        // Second child: List with items
        let list = arena.alloc(Node::new(NodeType::List));
        TreeOps::append_child(&mut arena, root, list);

        let item1 = arena.alloc(Node::new(NodeType::Item));
        let item2 = arena.alloc(Node::new(NodeType::Item));
        TreeOps::append_child(&mut arena, list, item1);
        TreeOps::append_child(&mut arena, list, item2);

        // Add text to items
        let text1 = arena.alloc(Node::with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "Item 1".to_string(),
            },
        ));
        let text2 = arena.alloc(Node::with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "Item 2".to_string(),
            },
        ));
        TreeOps::append_child(&mut arena, item1, text1);
        TreeOps::append_child(&mut arena, item2, text2);

        let mut walker = ArenaNodeWalker::new(&arena, root);
        let mut event_count = 0;

        while let Some(_event) = walker.next() {
            event_count += 1;
        }

        // Document(Enter) -> Heading(Enter) -> Heading(Exit) -> List(Enter) -> Item1(Enter) -> Text1(Enter) -> Text1(Exit) -> Item1(Exit) -> Item2(Enter) -> Text2(Enter) -> Text2(Exit) -> Item2(Exit) -> List(Exit) -> Document(Exit)
        assert_eq!(event_count, 14);
    }

    #[test]
    fn test_iterator_single_node() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::new(NodeType::Paragraph));
        let mut iter = ArenaNodeIterator::new(&arena, root);

        assert_eq!(iter.next(), EventType::Enter);
        assert_eq!(iter.get_node(), Some(root));

        assert_eq!(iter.next(), EventType::Exit);
        assert_eq!(iter.get_node(), Some(root));

        assert_eq!(iter.next(), EventType::Done);
    }

    #[test]
    fn test_walker_returns_none_when_done() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::new(NodeType::Document));
        let mut walker = ArenaNodeWalker::new(&arena, root);

        // Walk through all events
        while walker.next().is_some() {}

        // Should return None after Done
        assert!(walker.next().is_none());
        assert!(walker.next().is_none());
    }
}
