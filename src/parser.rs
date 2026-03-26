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
//! let (arena, doc_id) = parser.parse("# Hello\n\nWorld").unwrap();
//! ```

use crate::arena::{NodeArena, NodeId};
use crate::blocks::BlockParser;
use crate::error::{ParseError, ParseResult, ParserLimits};
use crate::inlines::parse_inlines_with_options;
use crate::node::{NodeData, NodeType};
use crate::options;
use rustc_hash::FxHashMap;

/// Parser for CommonMark documents using Arena allocation
pub struct Parser {
    options: u32,
    limits: ParserLimits,
}

impl Parser {
    /// Create a new parser with the given options
    pub fn new(options: u32) -> Self {
        Parser {
            options,
            limits: ParserLimits::default(),
        }
    }

    /// Create a new parser with custom limits
    pub fn with_limits(options: u32, limits: ParserLimits) -> Self {
        Parser { options, limits }
    }

    /// Parse a CommonMark document
    ///
    /// This method performs both block-level and inline parsing.
    /// Returns a Result containing a tuple of (arena, document_node_id)
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if:
    /// - Input exceeds maximum allowed size
    /// - Nesting depth exceeds maximum allowed
    /// - Other parsing errors occur
    pub fn parse(&self, text: &str) -> ParseResult<(NodeArena, NodeId)> {
        // Check input size limit
        let input_size = text.len();
        if input_size > self.limits.max_input_size {
            return Err(ParseError::InputTooLarge {
                size: input_size,
                max_size: self.limits.max_input_size,
            });
        }

        // Handle CRLF line endings with single-pass replacement
        let normalized_input = if text.contains('\r') {
            text.replace("\r\n", "\n").replace('\r', "\n")
        } else {
            text.to_string()
        };

        // Create arena and parse blocks
        let mut arena = NodeArena::new();
        let (doc, refmap) = BlockParser::parse_with_limits_and_refmap(
            &mut arena,
            &normalized_input,
            self.options,
            self.limits,
        )?;

        // Process inlines with the refmap extracted from the document
        self.process_inlines(&mut arena, doc, &refmap);

        Ok((arena, doc))
    }

    /// Process inline content for all leaf blocks
    fn process_inlines(
        &self,
        arena: &mut NodeArena,
        root: NodeId,
        refmap: &FxHashMap<String, (String, String)>,
    ) {
        // Collect leaf blocks that need inline processing
        let mut nodes_to_process: Vec<(NodeId, String)> = Vec::new();
        self.collect_leaf_blocks(arena, root, &mut nodes_to_process);

        // Check if smart punctuation is enabled
        let smart = (self.options & options::SMART) != 0;

        // Process collected nodes with the refmap from the document
        for (node_id, content) in nodes_to_process {
            parse_inlines_with_options(
                arena, node_id, &content, 1, // line number
                0, // block offset
                refmap, smart,
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
        let (arena, doc) = parser.parse("Hello world").unwrap();
        assert_eq!(arena.get(doc).node_type, NodeType::Document);
    }

    #[test]
    fn test_parser_input_too_large() {
        let parser = Parser::with_limits(0, ParserLimits::new().max_input_size(10));
        let result = parser.parse("This is a long text that exceeds the limit");
        assert!(result.is_err());
        match result {
            Err(ParseError::InputTooLarge { size, max_size }) => {
                assert!(size > 10);
                assert_eq!(max_size, 10);
            }
            _ => panic!("Expected InputTooLarge error"),
        }
    }

    #[test]
    fn test_parser_line_too_long() {
        let parser = Parser::with_limits(0, ParserLimits::new().max_line_length(10));
        let result = parser.parse("This is a very long line");
        assert!(result.is_err());
    }
}
