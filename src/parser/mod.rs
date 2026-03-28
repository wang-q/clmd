//! Main parser for CommonMark documents
//!
//! This module provides the main entry point for parsing CommonMark documents.
//! It coordinates block-level and inline parsing phases to produce a complete AST.
//!
//! The parser implementation follows the CommonMark specification and is inspired
//! by comrak's design, using arena allocation for efficient memory management.
//!
//! # Example
//!
//! ```ignore
//! use clmd::{Arena, parse_document, parser::options::Options};
//!
//! let arena = Arena::new();
//! let options = Options::default();
//! let root = parse_document(&arena, "Hello **world**!", &options);
//! ```

pub mod options;

use crate::arena::{NodeArena, NodeId};
use crate::blocks::BlockParser;
use crate::error::{ParseError, ParseResult, ParserLimits};
use crate::nodes::{Ast, Node};
use options::Options;
use std::cell::RefCell;
use std::collections::HashMap;

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
/// let arena = Arena::new();
/// let options = Options::default();
/// let root = parse_document(&arena, "# Hello\n\nWorld", &options);
/// ```
pub fn parse_document<'a>(
    arena: &'a crate::Arena<'a>,
    md: &str,
    options: &Options,
) -> Node<'a> {
    // Use BlockParser for parsing (which includes proper inline parsing)
    let mut node_arena = NodeArena::new();
    let options_flags = options_to_flags(options);
    let doc_id = BlockParser::parse_with_options(&mut node_arena, md, options_flags);

    // Convert NodeArena to arena_tree::Node
    convert_node_arena_to_ast(arena, &node_arena, doc_id)
}

/// Parse a Markdown document with custom limits.
///
/// # Example
///
/// ```ignore
/// use clmd::{Arena, parse_document_with_limits, parser::options::Options};
/// use clmd::error::ParserLimits;
///
/// let arena = Arena::new();
/// let options = Options::default();
/// let limits = ParserLimits::default();
/// let root = parse_document_with_limits(&arena, "Hello", &options, limits).unwrap();
/// ```
pub fn parse_document_with_limits<'a>(
    arena: &'a crate::Arena<'a>,
    md: &str,
    options: &Options,
    limits: ParserLimits,
) -> ParseResult<Node<'a>> {
    // Check input size limit
    if md.len() > limits.max_input_size {
        return Err(ParseError::InputTooLarge {
            size: md.len(),
            max_size: limits.max_input_size,
        });
    }

    let root = parse_document(arena, md, options);
    Ok(root)
}

// Legacy option flags (for backward compatibility)
const OPT_SOURCEPOS: u32 = 1 << 0;
const OPT_HARDBREAKS: u32 = 1 << 1;
const OPT_NOBREAKS: u32 = 1 << 2;
const OPT_VALIDATE_UTF8: u32 = 1 << 3;
const OPT_SMART: u32 = 1 << 4;
const OPT_UNSAFE: u32 = 1 << 5;

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

/// Convert NodeArena to arena_tree::Node.
///
/// This function converts the AST from BlockParser's NodeArena format to
/// the arena_tree::Node format used by the rest of the library.
fn convert_node_arena_to_ast<'a>(
    arena: &'a crate::Arena<'a>,
    node_arena: &NodeArena,
    node_id: NodeId,
) -> Node<'a> {
    let node_count = node_arena.len() as NodeId;

    // Create a mapping from NodeId to AstNode
    let mut id_to_node: HashMap<NodeId, Node<'a>> = HashMap::new();

    // First pass: create all nodes
    for id in 0..node_count {
        let node = node_arena.get(id);
        let ast_node = create_ast_node(arena, node);
        id_to_node.insert(id, ast_node);
    }

    // Second pass: establish parent-child and sibling relationships
    for id in 0..node_count {
        let node = node_arena.get(id);
        if let Some(&ast_node) = id_to_node.get(&id) {
            // Add all children (traverse the child linked list)
            let mut child_id = node.first_child;
            while let Some(child_id_val) = child_id {
                if let Some(&child_node) = id_to_node.get(&child_id_val) {
                    ast_node.append(child_node);
                }
                child_id = node_arena.get(child_id_val).next;
            }
        }
    }

    // Return the root node
    id_to_node
        .get(&node_id)
        .copied()
        .expect("Root node should exist")
}

/// Create an AstNode from a NodeArena Node.
fn create_ast_node<'a>(
    arena: &'a crate::Arena<'a>,
    node: &crate::arena::Node,
) -> Node<'a> {
    let node_value = node.value.clone();
    let ast = Ast::new(node_value, node.source_pos.start);
    let ast_node = crate::arena_tree::Node::new(RefCell::new(ast));
    arena.alloc(ast_node)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nodes::NodeValue;

    #[test]
    fn test_parse_document() {
        let arena = crate::Arena::new();
        let options = Options::default();
        let root = parse_document(&arena, "Hello world", &options);
        assert!(matches!(root.data().value, NodeValue::Document));
    }

    #[test]
    fn test_parse_heading() {
        let arena = crate::Arena::new();
        let options = Options::default();
        let root = parse_document(&arena, "# Heading", &options);

        // Should have one child (the heading)
        let heading = root.first_child().expect("Should have a child");
        assert!(matches!(heading.data().value, NodeValue::Heading(_)));
    }

    #[test]
    fn test_parse_paragraph() {
        let arena = crate::Arena::new();
        let options = Options::default();
        let root = parse_document(&arena, "Hello world", &options);

        let para = root.first_child().expect("Should have a child");
        assert!(matches!(para.data().value, NodeValue::Paragraph));
    }

    #[test]
    fn test_parse_multiple_paragraphs() {
        let arena = crate::Arena::new();
        let options = Options::default();
        let root = parse_document(&arena, "First para\n\nSecond para", &options);

        let first = root.first_child().expect("Should have first child");
        assert!(matches!(first.data().value, NodeValue::Paragraph));

        let second = first.next_sibling().expect("Should have second child");
        assert!(matches!(second.data().value, NodeValue::Paragraph));
    }
}
