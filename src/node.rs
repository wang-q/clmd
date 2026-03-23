/// Node types in the CommonMark AST
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    // Block types
    Document,
    BlockQuote,
    List,
    Item,
    CodeBlock,
    HtmlBlock,
    CustomBlock,
    Paragraph,
    Heading,
    ThematicBreak,

    // Inline types
    Text,
    SoftBreak,
    LineBreak,
    Code,
    HtmlInline,
    CustomInline,
    Emph,
    Strong,
    Link,
    Image,

    None,
}

impl NodeType {
    /// Check if this is a block type
    pub fn is_block(&self) -> bool {
        matches!(
            self,
            NodeType::Document
                | NodeType::BlockQuote
                | NodeType::List
                | NodeType::Item
                | NodeType::CodeBlock
                | NodeType::HtmlBlock
                | NodeType::CustomBlock
                | NodeType::Paragraph
                | NodeType::Heading
                | NodeType::ThematicBreak
        )
    }

    /// Check if this is an inline type
    pub fn is_inline(&self) -> bool {
        matches!(
            self,
            NodeType::Text
                | NodeType::SoftBreak
                | NodeType::LineBreak
                | NodeType::Code
                | NodeType::HtmlInline
                | NodeType::CustomInline
                | NodeType::Emph
                | NodeType::Strong
                | NodeType::Link
                | NodeType::Image
        )
    }

    /// Check if this is a leaf type (cannot have children)
    pub fn is_leaf(&self) -> bool {
        matches!(
            self,
            NodeType::Text
                | NodeType::SoftBreak
                | NodeType::LineBreak
                | NodeType::Code
                | NodeType::HtmlInline
                | NodeType::CodeBlock
                | NodeType::HtmlBlock
                | NodeType::ThematicBreak
        )
    }
}

/// List type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListType {
    Bullet,
    Ordered,
    None,
}

/// Delimiter type for ordered lists
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DelimType {
    Period,
    Paren,
    None,
}

/// Source position information
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SourcePos {
    pub start_line: u32,
    pub start_column: u32,
    pub end_line: u32,
    pub end_column: u32,
}

/// Node data variants
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeData {
    Document,
    BlockQuote,
    List {
        list_type: ListType,
        delim: DelimType,
        start: u32,
        tight: bool,
    },
    Item,
    CodeBlock {
        info: String,
        literal: String,
    },
    HtmlBlock {
        literal: String,
    },
    CustomBlock {
        on_enter: String,
        on_exit: String,
    },
    Paragraph,
    Heading {
        level: u32,
    },
    ThematicBreak,
    Text {
        literal: String,
    },
    SoftBreak,
    LineBreak,
    Code {
        literal: String,
    },
    HtmlInline {
        literal: String,
    },
    CustomInline {
        on_enter: String,
        on_exit: String,
    },
    Emph,
    Strong,
    Link {
        url: String,
        title: String,
    },
    Image {
        url: String,
        title: String,
    },
    None,
}

/// A node in the AST
pub struct Node {
    pub node_type: NodeType,
    pub data: NodeData,
    pub source_pos: SourcePos,
    pub parent: RefCell<Option<std::rc::Weak<RefCell<Node>>>>,
    pub first_child: RefCell<Option<std::rc::Rc<RefCell<Node>>>>,
    pub last_child: RefCell<Option<std::rc::Rc<RefCell<Node>>>>,
    pub next: RefCell<Option<std::rc::Rc<RefCell<Node>>>>,
    pub prev: RefCell<Option<std::rc::Weak<RefCell<Node>>>>,
}

use std::cell::RefCell;
use std::rc::Rc;

impl std::fmt::Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node")
            .field("node_type", &self.node_type)
            .field("data", &self.data)
            .field("source_pos", &self.source_pos)
            .finish()
    }
}

impl Node {
    pub fn new(node_type: NodeType) -> Self {
        let data = match node_type {
            NodeType::Document => NodeData::Document,
            NodeType::BlockQuote => NodeData::BlockQuote,
            NodeType::List => NodeData::List {
                list_type: ListType::None,
                delim: DelimType::None,
                start: 0,
                tight: false,
            },
            NodeType::Item => NodeData::Item,
            NodeType::CodeBlock => NodeData::CodeBlock {
                info: String::new(),
                literal: String::new(),
            },
            NodeType::HtmlBlock => NodeData::HtmlBlock {
                literal: String::new(),
            },
            NodeType::CustomBlock => NodeData::CustomBlock {
                on_enter: String::new(),
                on_exit: String::new(),
            },
            NodeType::Paragraph => NodeData::Paragraph,
            NodeType::Heading => NodeData::Heading { level: 0 },
            NodeType::ThematicBreak => NodeData::ThematicBreak,
            NodeType::Text => NodeData::Text {
                literal: String::new(),
            },
            NodeType::SoftBreak => NodeData::SoftBreak,
            NodeType::LineBreak => NodeData::LineBreak,
            NodeType::Code => NodeData::Code {
                literal: String::new(),
            },
            NodeType::HtmlInline => NodeData::HtmlInline {
                literal: String::new(),
            },
            NodeType::CustomInline => NodeData::CustomInline {
                on_enter: String::new(),
                on_exit: String::new(),
            },
            NodeType::Emph => NodeData::Emph,
            NodeType::Strong => NodeData::Strong,
            NodeType::Link => NodeData::Link {
                url: String::new(),
                title: String::new(),
            },
            NodeType::Image => NodeData::Image {
                url: String::new(),
                title: String::new(),
            },
            NodeType::None => NodeData::None,
        };

        Node {
            node_type,
            data,
            source_pos: SourcePos::default(),
            parent: RefCell::new(None),
            first_child: RefCell::new(None),
            last_child: RefCell::new(None),
            next: RefCell::new(None),
            prev: RefCell::new(None),
        }
    }

    pub fn new_with_data(node_type: NodeType, data: NodeData) -> Self {
        Node {
            node_type,
            data,
            source_pos: SourcePos::default(),
            parent: RefCell::new(None),
            first_child: RefCell::new(None),
            last_child: RefCell::new(None),
            next: RefCell::new(None),
            prev: RefCell::new(None),
        }
    }

    pub fn is_block(&self) -> bool {
        self.node_type.is_block()
    }

    pub fn is_inline(&self) -> bool {
        self.node_type.is_inline()
    }

    pub fn is_leaf(&self) -> bool {
        self.node_type.is_leaf()
    }
}

/// Append a child to a parent node
pub fn append_child(parent: &Rc<RefCell<Node>>, child: Rc<RefCell<Node>>) {
    // Set child's parent first
    *child.borrow_mut().parent.borrow_mut() = Some(Rc::downgrade(parent));

    // Get the last child of parent (if any)
    let last_child_opt = parent.borrow().last_child.borrow().clone();

    if let Some(last_child) = last_child_opt {
        // Link child to previous last child
        *child.borrow_mut().prev.borrow_mut() = Some(Rc::downgrade(&last_child));
        *last_child.borrow_mut().next.borrow_mut() = Some(child.clone());
    } else {
        // No children yet, set as first child
        *parent.borrow_mut().first_child.borrow_mut() = Some(child.clone());
    }

    // Always update last_child
    *parent.borrow_mut().last_child.borrow_mut() = Some(child);
}

/// Prepend a child to a parent node
pub fn prepend_child(parent: &Rc<RefCell<Node>>, child: Rc<RefCell<Node>>) {
    // Set child's parent first
    *child.borrow_mut().parent.borrow_mut() = Some(Rc::downgrade(parent));

    // Get the first child of parent (if any)
    let first_child_opt = parent.borrow().first_child.borrow().clone();

    if let Some(first_child) = first_child_opt {
        // Link child to current first child
        *child.borrow_mut().next.borrow_mut() = Some(first_child.clone());
        *first_child.borrow_mut().prev.borrow_mut() = Some(Rc::downgrade(&child));
    } else {
        // No children yet, set as last child too
        *parent.borrow_mut().last_child.borrow_mut() = Some(child.clone());
    }

    // Always update first_child
    *parent.borrow_mut().first_child.borrow_mut() = Some(child);
}

/// Insert a sibling after a node
pub fn insert_after(node: &Rc<RefCell<Node>>, sibling: Rc<RefCell<Node>>) {
    // Set sibling's parent from node
    let parent_weak = node.borrow().parent.borrow().clone();
    *sibling.borrow_mut().parent.borrow_mut() = parent_weak.clone();

    // Get node's next sibling (if any)
    let next_opt = node.borrow().next.borrow().clone();

    if let Some(next) = next_opt {
        // Link sibling between node and next
        *sibling.borrow_mut().next.borrow_mut() = Some(next.clone());
        *next.borrow_mut().prev.borrow_mut() = Some(Rc::downgrade(&sibling));
    } else if let Some(parent_weak) = parent_weak {
        // Node was the last child, update parent's last_child
        if let Some(parent) = parent_weak.upgrade() {
            *parent.borrow_mut().last_child.borrow_mut() = Some(sibling.clone());
        }
    }

    // Link sibling to node
    *sibling.borrow_mut().prev.borrow_mut() = Some(Rc::downgrade(node));
    *node.borrow_mut().next.borrow_mut() = Some(sibling);
}

/// Insert a sibling before a node
pub fn insert_before(node: &Rc<RefCell<Node>>, sibling: Rc<RefCell<Node>>) {
    // Set sibling's parent from node
    let parent_weak = node.borrow().parent.borrow().clone();
    *sibling.borrow_mut().parent.borrow_mut() = parent_weak.clone();

    // Get node's previous sibling (if any)
    let prev_weak_opt = node.borrow().prev.borrow().clone();

    if let Some(prev_weak) = prev_weak_opt {
        // Link sibling between prev and node
        if let Some(prev) = prev_weak.upgrade() {
            *sibling.borrow_mut().prev.borrow_mut() = Some(Rc::downgrade(&prev));
            *prev.borrow_mut().next.borrow_mut() = Some(sibling.clone());
        }
    } else if let Some(parent_weak) = parent_weak {
        // Node was the first child, update parent's first_child
        if let Some(parent) = parent_weak.upgrade() {
            *parent.borrow_mut().first_child.borrow_mut() = Some(sibling.clone());
        }
    }

    // Link sibling to node
    *sibling.borrow_mut().next.borrow_mut() = Some(node.clone());
    *node.borrow_mut().prev.borrow_mut() = Some(Rc::downgrade(&sibling));
}

/// Unlink a node from its parent and siblings
pub fn unlink(node: &Rc<RefCell<Node>>) {
    // Get references we need before making any changes
    let prev_weak_opt = node.borrow().prev.borrow().clone();
    let next_opt = node.borrow().next.borrow().clone();
    let parent_weak_opt = node.borrow().parent.borrow().clone();

    // Update previous node's next pointer
    if let Some(ref prev_weak) = prev_weak_opt {
        if let Some(prev) = prev_weak.upgrade() {
            *prev.borrow_mut().next.borrow_mut() = next_opt.clone();
        }
    } else if let Some(parent_weak) = &parent_weak_opt {
        // Node is first child, update parent's first_child
        if let Some(parent) = parent_weak.upgrade() {
            *parent.borrow_mut().first_child.borrow_mut() = next_opt.clone();
        }
    }

    // Update next node's prev pointer
    if let Some(next) = &next_opt {
        *next.borrow_mut().prev.borrow_mut() = prev_weak_opt.clone();
    } else if let Some(parent_weak) = &parent_weak_opt {
        // Node is last child, update parent's last_child
        if let Some(parent) = parent_weak.upgrade() {
            *parent.borrow_mut().last_child.borrow_mut() =
                prev_weak_opt.as_ref().and_then(|w| w.upgrade());
        }
    }

    // Clear this node's connections
    *node.borrow_mut().parent.borrow_mut() = None;
    *node.borrow_mut().next.borrow_mut() = None;
    *node.borrow_mut().prev.borrow_mut() = None;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_type_classification() {
        assert!(NodeType::Document.is_block());
        assert!(NodeType::Paragraph.is_block());
        assert!(!NodeType::Text.is_block());

        assert!(NodeType::Text.is_inline());
        assert!(NodeType::Link.is_inline());
        assert!(!NodeType::Paragraph.is_inline());

        assert!(NodeType::Text.is_leaf());
        assert!(NodeType::CodeBlock.is_leaf());
        assert!(!NodeType::Paragraph.is_leaf());
    }

    #[test]
    fn test_node_creation() {
        let node = Node::new(NodeType::Paragraph);
        assert_eq!(node.node_type, NodeType::Paragraph);
        assert!(node.is_block());
    }

    #[test]
    fn test_append_child() {
        let parent = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let child = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));

        append_child(&parent, child.clone());

        assert!(parent.borrow().first_child.borrow().is_some());
        assert!(parent.borrow().last_child.borrow().is_some());
        assert!(child.borrow().parent.borrow().is_some());
    }

    #[test]
    fn test_unlink() {
        let parent = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let child = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));

        append_child(&parent, child.clone());
        unlink(&child);

        assert!(parent.borrow().first_child.borrow().is_none());
        assert!(child.borrow().parent.borrow().is_none());
    }
}
