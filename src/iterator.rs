//! AST iterator for traversing the CommonMark document tree
//!
//! This module provides iterators and traits for traversing the AST in various ways:
//! - Depth-first traversal with enter/exit events
//! - Walkable trait for bottom-up and top-down transformations
//! - Queryable trait for searching and extracting nodes
//!
//! Inspired by Pandoc's Walk.hs and Generic.hs modules.
//!
//! # Examples
//!
//! ## Basic Iterator
//!
//! ```rust,ignore
//! use clmd::iterator::{ArenaNodeIterator, EventType};
//!
//! let iter = ArenaNodeIterator::new(&arena, root_id);
//! while iter.next() != EventType::Done {
//!     // Process enter/exit events
//! }
//! ```
//!
//! ## Walkable Trait
//!
//! ```rust,ignore
//! use clmd::iterator::Walkable;
//!
//! // Bottom-up transformation
//! arena.walk_bottom_up(root_id, |node_id, value| {
//!     if let NodeValue::Text(text) = value {
//!         *text = text.to_uppercase().into_boxed_str();
//!     }
//! });
//!
//! // Top-down transformation
//! arena.walk_top_down(root_id, |node_id, value| {
//!     // Process node before children
//! });
//! ```
//!
//! ## Queryable Trait
//!
//! ```rust,ignore
//! use clmd::iterator::Queryable;
//!
//! // Find all links
//! let links: Vec<NodeId> = arena.query(root_id, |value| {
//!     matches!(value, NodeValue::Link(_)).then_some(node_id)
//! });
//!
//! // Extract all text content
//! let text_content: Vec<String> = arena.query_collect(root_id, |value| {
//!     if let NodeValue::Text(text) = value {
//!         Some(text.to_string())
//!     } else {
//!         None
//!     }
//! });
//! ```

use crate::arena::{NodeArena, NodeId, TreeOps};
use crate::nodes::NodeValue;

/// Event type for tree iteration
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    /// No event (initial state)
    None,
    /// Iteration complete
    Done,
    /// Entering a node
    Enter,
    /// Exiting a node
    Exit,
}

/// Iterator for traversing the Arena-based AST
#[derive(Debug)]
pub struct ArenaNodeIterator<'a> {
    arena: &'a NodeArena,
    root: NodeId,
    current: Option<NodeId>,
    event_type: EventType,
}

impl<'a> ArenaNodeIterator<'a> {
    /// Create a new iterator for the given arena and root node
    pub fn new(arena: &'a NodeArena, root: NodeId) -> Self {
        ArenaNodeIterator {
            arena,
            root,
            current: None,
            event_type: EventType::None,
        }
    }

    /// Advance the iterator and return the next event type
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> EventType {
        self.advance()
    }

    /// Get the current node ID
    pub fn get_node(&self) -> Option<NodeId> {
        self.current
    }

    /// Get the current event type
    pub fn get_event_type(&self) -> EventType {
        self.event_type
    }

    /// Reset the iterator to a specific node and event type
    pub fn reset(&mut self, current: NodeId, event_type: EventType) {
        self.current = Some(current);
        self.event_type = event_type;
    }

    /// Internal method to advance the iterator and return the event type.
    /// This is the core traversal logic used by both `next()` and `Iterator::next()`.
    fn advance(&mut self) -> EventType {
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

/// Item type for the standard Iterator implementation
pub type ArenaIteratorItem = (NodeId, EventType);

impl<'a> Iterator for ArenaNodeIterator<'a> {
    type Item = ArenaIteratorItem;

    fn next(&mut self) -> Option<Self::Item> {
        let event = self.advance();
        if event == EventType::Done {
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
        if event_type == EventType::Done {
            None
        } else {
            self.iterator.get_node().map(|node| ArenaWalkerEvent {
                node,
                entering: event_type == EventType::Enter,
            })
        }
    }

    /// Resume iteration at a specific node
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
#[derive(Debug, Clone, Copy)]
pub struct ArenaWalkerEvent {
    /// The node ID
    pub node: NodeId,
    /// Whether we are entering (true) or exiting (false) the node
    pub entering: bool,
}

/// Walk direction for tree traversal
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WalkDirection {
    /// Bottom-up: process children before parent
    BottomUp,
    /// Top-down: process parent before children
    TopDown,
}

/// Trait for walking/transforming the AST
///
/// This trait provides methods for traversing the AST and applying
/// transformations to nodes. It supports both bottom-up and top-down
/// traversal patterns.
///
/// Inspired by Pandoc's Walkable type class.
pub trait Walkable {
    /// Walk the tree bottom-up, applying a function to each node
    ///
    /// The function is called after processing all children.
    fn walk_bottom_up<F>(&mut self, root: NodeId, f: &mut F)
    where
        F: FnMut(NodeId, &mut NodeValue);

    /// Walk the tree top-down, applying a function to each node
    ///
    /// The function is called before processing children.
    fn walk_top_down<F>(&mut self, root: NodeId, f: &mut F)
    where
        F: FnMut(NodeId, &mut NodeValue);

    /// Walk with direction control
    fn walk_with_direction<F>(&mut self, root: NodeId, direction: WalkDirection, f: &mut F)
    where
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

    fn walk_with_direction<F>(&mut self, root: NodeId, direction: WalkDirection, f: &mut F)
    where
        F: FnMut(NodeId, &mut NodeValue),
    {
        match direction {
            WalkDirection::BottomUp => {
                // Process children first, then parent
                let children: Vec<NodeId> = self.children(root).collect();
                for child in children {
                    self.walk_bottom_up(child, f);
                }
                let value = &mut self.get_mut(root).value;
                f(root, value);
            }
            WalkDirection::TopDown => {
                // Process parent first, then children
                let value = &mut self.get_mut(root).value;
                f(root, value);

                // Collect children to avoid borrow issues
                let children: Vec<NodeId> = self.children(root).collect();
                for child in children {
                    self.walk_top_down(child, f);
                }
            }
        }
    }
}

/// Trait for querying the AST
///
/// This trait provides methods for searching and extracting information
/// from the AST without modifying it.
///
/// Inspired by Pandoc's query and listify functions.
pub trait Queryable {
    /// Query the tree and collect results
    ///
    /// The query function is applied to each node. If it returns Some(value),
    /// the value is collected into the result vector.
    fn query<T, F>(&self, root: NodeId, f: &mut F) -> Vec<T>
    where
        F: FnMut(NodeId, &NodeValue) -> Option<T>;

    /// Query with early termination
    ///
    /// Stops traversing as soon as the predicate returns true.
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
}

impl NodeType {
    /// Check if a NodeValue matches this node type
    pub fn matches(&self, value: &NodeValue) -> bool {
        match (self, value) {
            (NodeType::Document, NodeValue::Document) => true,
            (NodeType::BlockQuote, NodeValue::BlockQuote) => true,
            (NodeType::List, NodeValue::List(_)) => true,
            (NodeType::Item, NodeValue::Item(_)) => true,
            (NodeType::CodeBlock, NodeValue::CodeBlock(_)) => true,
            (NodeType::HtmlBlock, NodeValue::HtmlBlock(_)) => true,
            (NodeType::Paragraph, NodeValue::Paragraph) => true,
            (NodeType::Heading, NodeValue::Heading(_)) => true,
            (NodeType::ThematicBreak, NodeValue::ThematicBreak) => true,
            (NodeType::FootnoteDefinition, NodeValue::FootnoteDefinition(_)) => true,
            (NodeType::Table, NodeValue::Table(_)) => true,
            (NodeType::TableRow, NodeValue::TableRow(_)) => true,
            (NodeType::TableCell, NodeValue::TableCell) => true,
            (NodeType::Text, NodeValue::Text(_)) => true,
            (NodeType::TaskItem, NodeValue::TaskItem(_)) => true,
            (NodeType::SoftBreak, NodeValue::SoftBreak) => true,
            (NodeType::HardBreak, NodeValue::HardBreak) => true,
            (NodeType::Code, NodeValue::Code(_)) => true,
            (NodeType::HtmlInline, NodeValue::HtmlInline(_)) => true,
            (NodeType::Emph, NodeValue::Emph) => true,
            (NodeType::Strong, NodeValue::Strong) => true,
            (NodeType::Strikethrough, NodeValue::Strikethrough) => true,
            (NodeType::Superscript, NodeValue::Superscript) => true,
            (NodeType::Subscript, NodeValue::Subscript) => true,
            (NodeType::Link, NodeValue::Link(_)) => true,
            (NodeType::Image, NodeValue::Image(_)) => true,
            (NodeType::FootnoteReference, NodeValue::FootnoteReference(_)) => true,
            (NodeType::Math, NodeValue::Math(_)) => true,
            (NodeType::Raw, NodeValue::Raw(_)) => true,
            _ => false,
        }
    }
}

impl Queryable for NodeArena {
    fn query<T, F>(&self, root: NodeId, f: &mut F) -> Vec<T>
    where
        F: FnMut(NodeId, &NodeValue) -> Option<T>,
    {
        let mut results = Vec::new();
        self.query_recursive(root, f, &mut results);
        results
    }

    fn query_first<T, F>(&self, root: NodeId, f: &mut F) -> Option<T>
    where
        F: FnMut(NodeId, &NodeValue) -> Option<T>,
    {
        // Check current node
        let value = &self.get(root).value;
        if let Some(result) = f(root, value) {
            return Some(result);
        }

        // Recursively check children
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
        !self.any(root, &mut |id, value| !f(id, value))
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
}

impl NodeArena {
    /// Helper method for recursive querying
    fn query_recursive<T, F>(&self, node: NodeId, f: &mut F, results: &mut Vec<T>)
    where
        F: FnMut(NodeId, &NodeValue) -> Option<T>,
    {
        let value = &self.get(node).value;
        if let Some(result) = f(node, value) {
            results.push(result);
        }

        let children: Vec<NodeId> = self.children(node).collect();
        for child in children {
            self.query_recursive(child, f, results);
        }
    }
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
                if matches!(node.value, NodeValue::Text(..)) {
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
        let next_is_text = matches!(arena.get(next_node_id).value, NodeValue::Text(..));
        if !next_is_text {
            break;
        }

        // Append next node's literal to current node
        let next_literal: Box<str> =
            if let NodeValue::Text(literal) = &arena.get(next_node_id).value {
                literal.clone()
            } else {
                "".into()
            };

        if let NodeValue::Text(ref mut literal) = arena.get_mut(node).value {
            *literal = format!("{}{}", literal.as_ref(), next_literal.as_ref())
                .into_boxed_str();
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

    #[test]
    fn test_iterator_basic() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("Hello")));

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
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("Hello")));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, text);

        let mut walker = ArenaNodeWalker::new(&arena, root);
        let mut events = Vec::new();

        while let Some(event) = walker.next() {
            let value = &arena.get(event.node).value;
            events.push((value.clone(), event.entering));
        }

        // Document(Enter) -> Paragraph(Enter) -> Text(Enter) -> Text(Exit) -> Paragraph(Exit) -> Document(Exit)
        assert_eq!(events.len(), 6);
        assert!(matches!(events[0].0, NodeValue::Document) && events[0].1);
        assert!(matches!(events[1].0, NodeValue::Paragraph) && events[1].1);
        assert!(matches!(events[2].0, NodeValue::Text(..)) && events[2].1);
        assert!(matches!(events[3].0, NodeValue::Text(..)) && !events[3].1);
        assert!(matches!(events[4].0, NodeValue::Paragraph) && !events[4].1);
        assert!(matches!(events[5].0, NodeValue::Document) && !events[5].1);
    }

    #[test]
    fn test_iterator_empty_document() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let mut iter = ArenaNodeIterator::new(&arena, root);

        assert_eq!(iter.next(), EventType::Enter);
        assert_eq!(iter.next(), EventType::Exit);
        assert_eq!(iter.next(), EventType::Done);
    }

    #[test]
    fn test_iterator_multiple_siblings() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para1 = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let para2 = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let para3 = arena.alloc(Node::with_value(NodeValue::Paragraph));

        TreeOps::append_child(&mut arena, root, para1);
        TreeOps::append_child(&mut arena, root, para2);
        TreeOps::append_child(&mut arena, root, para3);

        let mut walker = ArenaNodeWalker::new(&arena, root);
        let mut events = Vec::new();

        while let Some(event) = walker.next() {
            let value = &arena.get(event.node).value;
            events.push((value.clone(), event.entering));
        }

        // Document(Enter) -> Para1(Enter) -> Para1(Exit) -> Para2(Enter) -> Para2(Exit) -> Para3(Enter) -> Para3(Exit) -> Document(Exit)
        assert_eq!(events.len(), 8);
        assert!(matches!(events[0].0, NodeValue::Document) && events[0].1);
        assert!(matches!(events[1].0, NodeValue::Paragraph) && events[1].1);
        assert!(matches!(events[2].0, NodeValue::Paragraph) && !events[2].1);
        assert!(matches!(events[3].0, NodeValue::Paragraph) && events[3].1);
        assert!(matches!(events[4].0, NodeValue::Paragraph) && !events[4].1);
        assert!(matches!(events[5].0, NodeValue::Paragraph) && events[5].1);
        assert!(matches!(events[6].0, NodeValue::Paragraph) && !events[6].1);
        assert!(matches!(events[7].0, NodeValue::Document) && !events[7].1);
    }

    #[test]
    fn test_iterator_nested_structure() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let blockquote = arena.alloc(Node::with_value(NodeValue::BlockQuote));
        let list = arena.alloc(Node::with_value(NodeValue::List(Default::default())));
        let item = arena.alloc(Node::with_value(NodeValue::Item(Default::default())));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("Text")));

        TreeOps::append_child(&mut arena, root, blockquote);
        TreeOps::append_child(&mut arena, blockquote, list);
        TreeOps::append_child(&mut arena, list, item);
        TreeOps::append_child(&mut arena, item, para);
        TreeOps::append_child(&mut arena, para, text);

        let mut walker = ArenaNodeWalker::new(&arena, root);
        let mut events = Vec::new();

        while let Some(event) = walker.next() {
            let value = &arena.get(event.node).value;
            events.push((value.clone(), event.entering));
        }

        // Should visit all nodes in depth-first order
        assert_eq!(events.len(), 12);
        assert!(matches!(events[0].0, NodeValue::Document) && events[0].1);
        assert!(matches!(events[1].0, NodeValue::BlockQuote) && events[1].1);
        assert!(matches!(events[2].0, NodeValue::List(..)) && events[2].1);
        assert!(matches!(events[3].0, NodeValue::Item(..)) && events[3].1);
        assert!(matches!(events[4].0, NodeValue::Paragraph) && events[4].1);
        assert!(matches!(events[5].0, NodeValue::Text(..)) && events[5].1);
        assert!(matches!(events[6].0, NodeValue::Text(..)) && !events[6].1);
        assert!(matches!(events[7].0, NodeValue::Paragraph) && !events[7].1);
        assert!(matches!(events[8].0, NodeValue::Item(..)) && !events[8].1);
        assert!(matches!(events[9].0, NodeValue::List(..)) && !events[9].1);
        assert!(matches!(events[10].0, NodeValue::BlockQuote) && !events[10].1);
        assert!(matches!(events[11].0, NodeValue::Document) && !events[11].1);
    }

    #[test]
    fn test_walker_resume_at() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para1 = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let para2 = arena.alloc(Node::with_value(NodeValue::Paragraph));

        TreeOps::append_child(&mut arena, root, para1);
        TreeOps::append_child(&mut arena, root, para2);

        let mut walker = ArenaNodeWalker::new(&arena, root);

        // Get first event (Document Enter)
        let event1 = walker.next().unwrap();
        assert!(matches!(arena.get(event1.node).value, NodeValue::Document));
        assert!(event1.entering);

        // Get second event (Para1 Enter)
        let event2 = walker.next().unwrap();
        assert!(matches!(arena.get(event2.node).value, NodeValue::Paragraph));
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
        let root = arena.alloc(Node::with_value(NodeValue::Document));
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
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
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
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text1 = arena.alloc(Node::with_value(NodeValue::make_text("Hello ")));
        let text2 = arena.alloc(Node::with_value(NodeValue::make_text("world")));
        let text3 = arena.alloc(Node::with_value(NodeValue::make_text("!")));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, text1);
        TreeOps::append_child(&mut arena, para, text2);
        TreeOps::append_child(&mut arena, para, text3);

        consolidate_text_nodes(&mut arena, root);

        // text1 should now contain "Hello world!"
        if let NodeValue::Text(literal) = &arena.get(text1).value {
            assert_eq!(literal.as_ref(), "Hello world!");
        }

        // text2 and text3 should be unlinked
        assert!(arena.get(text2).parent.is_none());
        assert!(arena.get(text3).parent.is_none());
    }

    #[test]
    fn test_consolidate_text_nodes_non_adjacent() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text1 = arena.alloc(Node::with_value(NodeValue::make_text("Hello")));
        let emph = arena.alloc(Node::with_value(NodeValue::Emph));
        let text2 = arena.alloc(Node::with_value(NodeValue::make_text("world")));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, text1);
        TreeOps::append_child(&mut arena, para, emph);
        TreeOps::append_child(&mut arena, para, text2);

        consolidate_text_nodes(&mut arena, root);

        // text1 and text2 should not be consolidated (separated by emph)
        let text1_content: String =
            if let NodeValue::Text(literal) = &arena.get(text1).value {
                literal.as_ref().to_string()
            } else {
                String::new()
            };
        assert_eq!(text1_content, "Hello");

        let text2_content: String =
            if let NodeValue::Text(literal) = &arena.get(text2).value {
                literal.as_ref().to_string()
            } else {
                String::new()
            };
        assert_eq!(text2_content, "world");
    }

    #[test]
    fn test_walker_with_complex_tree() {
        // Create a more complex tree structure
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        // First child: Heading
        let heading = arena.alloc(Node::with_value(NodeValue::Heading(
            crate::nodes::NodeHeading {
                level: 1,
                setext: false,
                closed: false,
            },
        )));
        TreeOps::append_child(&mut arena, root, heading);

        // Second child: List with items
        let list = arena.alloc(Node::with_value(NodeValue::List(Default::default())));
        TreeOps::append_child(&mut arena, root, list);

        let item1 = arena.alloc(Node::with_value(NodeValue::Item(Default::default())));
        let item2 = arena.alloc(Node::with_value(NodeValue::Item(Default::default())));
        TreeOps::append_child(&mut arena, list, item1);
        TreeOps::append_child(&mut arena, list, item2);

        // Add text to items
        let text1 = arena.alloc(Node::with_value(NodeValue::make_text("Item 1")));
        let text2 = arena.alloc(Node::with_value(NodeValue::make_text("Item 2")));
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
        let root = arena.alloc(Node::with_value(NodeValue::Paragraph));
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
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let mut walker = ArenaNodeWalker::new(&arena, root);

        // Walk through all events
        while walker.next().is_some() {}

        // Should return None after Done
        assert!(walker.next().is_none());
        assert!(walker.next().is_none());
    }

    // =========================================================================
    // Walkable trait tests
    // =========================================================================

    #[test]
    fn test_walkable_bottom_up() {
        use super::Walkable;

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text1 = arena.alloc(Node::with_value(NodeValue::make_text("Hello ")));
        let text2 = arena.alloc(Node::with_value(NodeValue::make_text("world")));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, text1);
        TreeOps::append_child(&mut arena, para, text2);

        // Collect visit order
        let mut visit_order = Vec::new();
        arena.walk_bottom_up(root, &mut |_id, value| {
            let name = match value {
                NodeValue::Document => "Document",
                NodeValue::Paragraph => "Paragraph",
                NodeValue::Text(_) => "Text",
                _ => "Other",
            };
            visit_order.push(name.to_string());
        });

        // Bottom-up: children before parent
        assert_eq!(visit_order, vec!["Text", "Text", "Paragraph", "Document"]);
    }

    #[test]
    fn test_walkable_top_down() {
        use super::Walkable;

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text1 = arena.alloc(Node::with_value(NodeValue::make_text("Hello ")));
        let text2 = arena.alloc(Node::with_value(NodeValue::make_text("world")));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, text1);
        TreeOps::append_child(&mut arena, para, text2);

        // Collect visit order
        let mut visit_order = Vec::new();
        arena.walk_top_down(root, &mut |_id, value| {
            let name = match value {
                NodeValue::Document => "Document",
                NodeValue::Paragraph => "Paragraph",
                NodeValue::Text(_) => "Text",
                _ => "Other",
            };
            visit_order.push(name.to_string());
        });

        // Top-down: parent before children
        assert_eq!(visit_order, vec!["Document", "Paragraph", "Text", "Text"]);
    }

    #[test]
    fn test_walkable_transform() {
        use super::Walkable;

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("hello world")));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, text);

        // Transform text to uppercase
        arena.walk_bottom_up(root, &mut |_id, value| {
            if let NodeValue::Text(ref mut text) = value {
                *text = text.to_uppercase().into_boxed_str();
            }
        });

        // Verify transformation
        if let NodeValue::Text(text) = &arena.get(text).value {
            assert_eq!(text.as_ref(), "HELLO WORLD");
        }
    }

    // =========================================================================
    // Queryable trait tests
    // =========================================================================

    #[test]
    fn test_queryable_query() {
        use super::Queryable;

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para1 = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let para2 = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("Hello")));

        TreeOps::append_child(&mut arena, root, para1);
        TreeOps::append_child(&mut arena, root, para2);
        TreeOps::append_child(&mut arena, para1, text);

        // Query for all paragraphs
        let paragraphs: Vec<NodeId> = arena.query(root, &mut |id, value| {
            if matches!(value, NodeValue::Paragraph) {
                Some(id)
            } else {
                None
            }
        });

        assert_eq!(paragraphs.len(), 2);
        assert!(paragraphs.contains(&para1));
        assert!(paragraphs.contains(&para2));
    }

    #[test]
    fn test_queryable_query_first() {
        use super::Queryable;

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para1 = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let para2 = arena.alloc(Node::with_value(NodeValue::Paragraph));

        TreeOps::append_child(&mut arena, root, para1);
        TreeOps::append_child(&mut arena, root, para2);

        // Find first paragraph
        let first_para = arena.query_first(root, &mut |id, value| {
            if matches!(value, NodeValue::Paragraph) {
                Some(id)
            } else {
                None
            }
        });

        assert_eq!(first_para, Some(para1));
    }

    #[test]
    fn test_queryable_any() {
        use super::Queryable;

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("Hello")));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, text);

        // Check if any node is a paragraph
        assert!(arena.any(root, &mut |_, value| {
            matches!(value, NodeValue::Paragraph)
        }));

        // Check if any node is a heading (should be false)
        assert!(!arena.any(root, &mut |_, value| {
            matches!(value, NodeValue::Heading(_))
        }));
    }

    #[test]
    fn test_queryable_all() {
        use super::Queryable;

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("Hello")));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, text);

        // All nodes should be valid (non-empty check)
        assert!(arena.all(root, &mut |_, _| true));

        // Not all nodes are paragraphs
        assert!(!arena.all(root, &mut |_, value| {
            matches!(value, NodeValue::Paragraph)
        }));
    }

    #[test]
    fn test_queryable_count() {
        use super::Queryable;

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para1 = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let para2 = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("Hello")));

        TreeOps::append_child(&mut arena, root, para1);
        TreeOps::append_child(&mut arena, root, para2);
        TreeOps::append_child(&mut arena, para1, text);

        // Count paragraphs
        let para_count = arena.count(root, &mut |_, value| {
            matches!(value, NodeValue::Paragraph)
        });
        assert_eq!(para_count, 2);

        // Count text nodes
        let text_count = arena.count(root, &mut |_, value| {
            matches!(value, NodeValue::Text(_))
        });
        assert_eq!(text_count, 1);
    }

    #[test]
    fn test_queryable_find_by_type() {
        use super::{Queryable, NodeType};

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para1 = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let para2 = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("Hello")));

        TreeOps::append_child(&mut arena, root, para1);
        TreeOps::append_child(&mut arena, root, para2);
        TreeOps::append_child(&mut arena, para1, text);

        // Find all paragraphs
        let paragraphs = arena.find_by_type(root, NodeType::Paragraph);
        assert_eq!(paragraphs.len(), 2);
        assert!(paragraphs.contains(&para1));
        assert!(paragraphs.contains(&para2));

        // Find all text nodes
        let texts = arena.find_by_type(root, NodeType::Text);
        assert_eq!(texts.len(), 1);
        assert!(texts.contains(&text));
    }

    #[test]
    fn test_queryable_extract_text() {
        use super::Queryable;

        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text1 = arena.alloc(Node::with_value(NodeValue::make_text("Hello ")));
        let text2 = arena.alloc(Node::with_value(NodeValue::make_text("world")));
        let text3 = arena.alloc(Node::with_value(NodeValue::make_text("!")));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, text1);
        TreeOps::append_child(&mut arena, para, text2);
        TreeOps::append_child(&mut arena, para, text3);

        // Extract all text
        let extracted = arena.extract_text(root);
        assert_eq!(extracted, "Hello world!");
    }

    #[test]
    fn test_node_type_matches() {
        use super::NodeType;

        assert!(NodeType::Document.matches(&NodeValue::Document));
        assert!(NodeType::Paragraph.matches(&NodeValue::Paragraph));
        assert!(NodeType::Text.matches(&NodeValue::make_text("test")));
        assert!(NodeType::Heading.matches(&NodeValue::Heading(crate::nodes::NodeHeading {
            level: 1,
            setext: false,
            closed: false,
        })));

        assert!(!NodeType::Paragraph.matches(&NodeValue::Document));
        assert!(!NodeType::Text.matches(&NodeValue::Paragraph));
    }

}
