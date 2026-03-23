use crate::blocks::BlockParser;
use crate::inlines::parse_inlines_with_refmap;
use crate::iterator::NodeWalker;
use crate::node::{Node, NodeData, NodeType};
use std::cell::RefCell;
use std::rc::Rc;

/// Parser for CommonMark documents
pub struct Parser {
    #[allow(dead_code)]
    options: u32,
}

impl Parser {
    /// Create a new parser with the given options
    pub fn new(options: u32) -> Self {
        Parser { options }
    }

    /// Parse a CommonMark document
    ///
    /// This method performs both block-level and inline parsing.
    pub fn parse(&self, text: &str) -> Rc<RefCell<Node>> {
        // Step 1: Block-level parsing
        let mut block_parser = BlockParser::new();

        // Handle CRLF line endings
        let normalized_input = text.replace("\r\n", "\n").replace('\r', "\n");
        let lines: Vec<&str> = normalized_input.lines().collect();

        for line in &lines {
            block_parser.process_line(line);
        }
        block_parser.process_line(""); // Finalize
        block_parser.finalize_document(); // Finalize all blocks

        let doc = block_parser.doc.clone();
        let refmap = block_parser.refmap.clone();

        // Step 2: Inline parsing for leaf blocks
        self.process_inlines(&doc, &refmap);

        doc
    }

    /// Process inline content for all container blocks that contain text
    fn process_inlines(&self, root: &Rc<RefCell<Node>>, refmap: &std::collections::HashMap<String, (String, String)>) {
        // Collect all container nodes that have text content
        // These are paragraph, heading nodes that store text in their data field
        let mut nodes_to_process: Vec<(Rc<RefCell<Node>>, String)> = Vec::new();

        {
            let mut walker = NodeWalker::new(root.clone());

            while let Some(event) = walker.next() {
                if event.entering {
                    let node = event.node.borrow();

                    // Process container blocks that contain inline text
                    // Paragraph and Heading store text content in their data field
                    match node.node_type {
                        NodeType::Paragraph | NodeType::Heading => {
                            if let NodeData::Text { literal } = &node.data {
                                if !literal.is_empty() {
                                    nodes_to_process.push((event.node.clone(), literal.clone()));
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        // Process collected nodes
        for (node, content) in nodes_to_process {
            parse_inlines_with_refmap(
                &node,
                &content,
                1, // line number
                0, // block offset
                refmap.clone(),
            );
        }
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
