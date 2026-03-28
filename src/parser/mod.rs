//! Main parser for CommonMark documents
//!
//! This module provides the main entry point for parsing CommonMark documents.
//! It coordinates block-level and inline parsing phases to produce a complete AST.
//!
//! # Example
//!
//! ```ignore
//! use clmd::{Arena, parse_document, parser::options::Options};
//!
//! let mut arena = Arena::new();
//! let options = Options::default();
//! let root = parse_document(&mut arena, "Hello **world**!", &options);
//! ```

pub mod options;

use crate::arena::{NodeArena, NodeId};
use crate::blocks::BlockParser;
use crate::error::{ParseResult, ParserLimits};
use options::Options;

/// Parse a Markdown document to an AST.
///
/// This is the main entry point for parsing. It takes an arena for node allocation,
/// the Markdown text to parse, and options for configuring the parser.
///
/// # Example
///
/// ```ignore
/// use clmd::{Arena, parse_document, parser::options::Options};
///
/// let mut arena = Arena::new();
/// let options = Options::default();
/// let root = parse_document(&mut arena, "# Hello\n\nWorld", &options);
/// ```
pub fn parse_document<'a>(
    arena: &'a mut NodeArena,
    md: &str,
    options: &Options,
) -> NodeId {
    // Convert options to legacy u32 flags for now
    let legacy_options = options_to_flags(options);
    BlockParser::parse_with_options(arena, md, legacy_options)
}

/// Parse a Markdown document with custom limits.
///
/// # Example
///
/// ```ignore
/// use clmd::{Arena, parse_document_with_limits, parser::options::Options};
/// use clmd::error::ParserLimits;
///
/// let mut arena = Arena::new();
/// let options = Options::default();
/// let limits = ParserLimits::default();
/// let root = parse_document_with_limits(&mut arena, "Hello", &options, limits).unwrap();
/// ```
pub fn parse_document_with_limits<'a>(
    arena: &'a mut NodeArena,
    md: &str,
    options: &Options,
    limits: ParserLimits,
) -> ParseResult<NodeId> {
    let legacy_options = options_to_flags(options);
    BlockParser::parse_with_limits(arena, md, legacy_options, limits)
}

/// Convert Options to legacy u32 flags.
///
/// This is a temporary bridge until all components use the new Options system.
fn options_to_flags(options: &Options) -> u32 {
    let mut flags = 0u32;

    // Parse options
    if options.parse.sourcepos {
        flags |= OPT_SOURCEPOS;
    }
    if options.parse.smart {
        flags |= OPT_SMART;
    }
    if options.parse.validate_utf8 {
        flags |= OPT_VALIDATE_UTF8;
    }

    // Render options
    if options.render.hardbreaks {
        flags |= OPT_HARDBREAKS;
    }
    if options.render.nobreaks {
        flags |= OPT_NOBREAKS;
    }
    if options.render.r#unsafe {
        flags |= OPT_UNSAFE;
    }

    flags
}

// Legacy option flags (for backward compatibility)
const OPT_SOURCEPOS: u32 = 1 << 0;
const OPT_HARDBREAKS: u32 = 1 << 1;
const OPT_NOBREAKS: u32 = 1 << 2;
const OPT_VALIDATE_UTF8: u32 = 1 << 3;
const OPT_SMART: u32 = 1 << 4;
const OPT_UNSAFE: u32 = 1 << 5;

/// Parser for CommonMark documents using Arena allocation.
///
/// This struct provides a higher-level API for parsing with error handling.
pub struct Parser {
    options: u32,
    limits: ParserLimits,
}

impl Parser {
    /// Create a new parser with the given options.
    pub fn new(options: &Options) -> Self {
        Parser {
            options: options_to_flags(options),
            limits: ParserLimits::default(),
        }
    }

    /// Create a new parser with custom limits.
    pub fn with_limits(options: &Options, limits: ParserLimits) -> Self {
        Parser {
            options: options_to_flags(options),
            limits,
        }
    }

    /// Parse a CommonMark document.
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if:
    /// - Input exceeds maximum allowed size
    /// - Nesting depth exceeds maximum allowed
    /// - Other parsing errors occur
    pub fn parse<'a>(&self, arena: &'a mut NodeArena, text: &str) -> ParseResult<NodeId> {
        BlockParser::parse_with_limits(arena, text, self.options, self.limits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node_value::NodeValue;

    #[test]
    fn test_parse_document() {
        let mut arena = NodeArena::new();
        let options = Options::default();
        let root = parse_document(&mut arena, "Hello world", &options);
        assert!(matches!(arena.get(root).value, NodeValue::Document));
    }

    #[test]
    fn test_parser_creation() {
        let options = Options::default();
        let parser = Parser::new(&options);
        let mut arena = NodeArena::new();
        let root = parser.parse(&mut arena, "Hello world").unwrap();
        assert!(matches!(arena.get(root).value, NodeValue::Document));
    }
}
