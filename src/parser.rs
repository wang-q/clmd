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
        // BlockParser already handles inline processing during finalization
        let mut arena = NodeArena::new();
        let (doc, _refmap) = BlockParser::parse_with_limits_and_refmap(
            &mut arena,
            &normalized_input,
            self.options,
            self.limits,
        )?;

        Ok((arena, doc))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node_value::NodeValue;

    #[test]
    fn test_parser_creation() {
        let parser = Parser::new(0);
        let (arena, doc) = parser.parse("Hello world").unwrap();
        assert!(matches!(arena.get(doc).value, NodeValue::Document));
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
