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
        let event_type = if entering { EventType::Enter } else { EventType::Exit };
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
}
