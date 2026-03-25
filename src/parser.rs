//! Main parser for CommonMark documents
//!
//! This module provides the main entry point for parsing CommonMark documents.
//! It coordinates block-level and inline parsing phases to produce a complete AST.
//!
//! # Example
//!
//! ```rust,ignore
//! use clmd::{Parser, Options};
//!
//! let parser = Parser::new(Options::DEFAULT);
//! let (arena, doc_id) = parser.parse("# Hello\n\nWorld");
//! ```

use crate::arena::{NodeArena, NodeId};
use crate::blocks::BlockParser;
use crate::inlines::parse_inlines_with_options;
use crate::node::{NodeData, NodeType};
use crate::options;
use std::collections::HashMap;

/// Parser for CommonMark documents using Arena allocation
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
    /// Returns a tuple of (arena, document_node_id)
    pub fn parse(&self, text: &str) -> (NodeArena, NodeId) {
        // Handle CRLF line endings
        let normalized_input = text.replace("\r\n", "\n").replace('\r', "\n");

        // Create arena and parse blocks
        let mut arena = NodeArena::new();
        let doc = BlockParser::parse(&mut arena, &normalized_input);

        // Process inlines
        self.process_inlines(&mut arena, doc);

        (arena, doc)
    }

    /// Process inline content for all leaf blocks
    fn process_inlines(&self, arena: &mut NodeArena, root: NodeId) {
        // Collect leaf blocks that need inline processing
        let mut nodes_to_process: Vec<(NodeId, String)> = Vec::new();
        self.collect_leaf_blocks(arena, root, &mut nodes_to_process);

        // Check if smart punctuation is enabled
        let smart = (self.options & options::SMART) != 0;

        // Process collected nodes
        let empty_refmap = HashMap::new();
        for (node_id, content) in nodes_to_process {
            parse_inlines_with_options(
                arena,
                node_id,
                &content,
                1,             // line number
                0,             // block offset
                &empty_refmap, // refmap - TODO: extract from document
                smart,
            );
        }
    }

    /// Recursively collect leaf blocks that need inline processing
    fn collect_leaf_blocks(
        &self,
        arena: &NodeArena,
        node_id: NodeId,
        nodes_to_process: &mut Vec<(NodeId, String)>,
    ) {
        let node = arena.get(node_id);

        match node.node_type {
            NodeType::Paragraph | NodeType::Heading => {
                // Get content from the node
                let content = if let NodeData::Text { literal } = &node.data {
                    literal.clone()
                } else if let NodeData::Heading { content, .. } = &node.data {
                    content.clone()
                } else {
                    String::new()
                };

                if !content.is_empty() {
                    nodes_to_process.push((node_id, content));
                }
            }
            _ => {
                // Recursively process children
                if let Some(child_id) = node.first_child {
                    let mut current = Some(child_id);
                    while let Some(id) = current {
                        self.collect_leaf_blocks(arena, id, nodes_to_process);
                        current = arena.get(id).next;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_creation() {
        let parser = Parser::new(0);
        let (arena, doc) = parser.parse("Hello world");
        assert_eq!(arena.get(doc).node_type, NodeType::Document);
    }
}
