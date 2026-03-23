use crate::node::{append_child, Node, NodeType};
use std::cell::RefCell;
use std::rc::Rc;

/// Parser for CommonMark documents
pub struct Parser {
    options: u32,
}

impl Parser {
    /// Create a new parser with the given options
    pub fn new(options: u32) -> Self {
        Parser { options }
    }

    /// Parse a CommonMark document
    pub fn parse(&self, text: &str) -> Rc<RefCell<Node>> {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));

        // TODO: Implement actual parsing
        // For now, create a simple paragraph with text
        let paragraph = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        let text_node = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Text,
            crate::node::NodeData::Text {
                literal: text.to_string(),
            },
        )));

        append_child(&root, paragraph.clone());
        append_child(&paragraph, text_node);

        root
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_creation() {
        let parser = Parser::new(0);
        let root = parser.parse("Hello world");
        assert_eq!(root.borrow().node_type, NodeType::Document);
    }
}
