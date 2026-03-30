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
//! use clmd::{parse_document, parser::options::Options};
//!
//! let options = Options::default();
//! let (arena, root) = parse_document("Hello **world**!", &options);
//! ```

pub mod options;

use crate::arena::{NodeArena, NodeId};
use crate::blocks::BlockParser;
use crate::error::{ParseResult, ParserLimits};
use options::Options;

/// Parse a Markdown document to an AST.
///
/// This is the main entry point for parsing. It takes the Markdown text to parse
/// and options for configuring the parser. Returns a tuple of (NodeArena, root_node_id).
///
/// # Example
///
/// ```ignore
/// use clmd::{parse_document, parser::options::Options};
///
/// let options = Options::default();
/// let (arena, root) = parse_document("# Hello\n\nWorld", &options);
/// ```
pub fn parse_document(md: &str, options: &Options) -> (NodeArena, NodeId) {
    // Use BlockParser for parsing (which includes proper inline parsing)
    let mut node_arena = NodeArena::new();
    let options_flags = options_to_flags(options);
    let doc_id = BlockParser::parse_with_options(&mut node_arena, md, options_flags);

    (node_arena, doc_id)
}

/// Parse a Markdown document with custom limits.
///
/// # Example
///
/// ```ignore
/// use clmd::{parse_document_with_limits, parser::options::Options};
/// use clmd::error::ParserLimits;
///
/// let options = Options::default();
/// let limits = ParserLimits::default();
/// let (arena, root) = parse_document_with_limits("Hello", &options, limits).unwrap();
/// ```
pub fn parse_document_with_limits(
    md: &str,
    options: &Options,
    limits: ParserLimits,
) -> ParseResult<(NodeArena, NodeId)> {
    // Use BlockParser with limits for full limit checking
    let mut node_arena = NodeArena::new();
    let options_flags = options_to_flags(options);

    let doc_id =
        BlockParser::parse_with_limits(&mut node_arena, md, options_flags, limits)?;

    Ok((node_arena, doc_id))
}

// Legacy option flags (for backward compatibility)
pub(crate) const OPT_SOURCEPOS: u32 = 1 << 0;
pub(crate) const OPT_HARDBREAKS: u32 = 1 << 1;
pub(crate) const OPT_NOBREAKS: u32 = 1 << 2;
pub(crate) const OPT_VALIDATE_UTF8: u32 = 1 << 3;
pub(crate) const OPT_SMART: u32 = 1 << 4;
pub(crate) const OPT_UNSAFE: u32 = 1 << 5;
pub(crate) const OPT_TABLE: u32 = 1 << 6;

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

    // Extension options
    if options.extension.table {
        flags |= OPT_TABLE;
    }

    flags
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ParserLimits;
    use crate::nodes::NodeValue;

    #[test]
    fn test_parse_document() {
        let options = Options::default();
        let (arena, root) = parse_document("Hello world", &options);
        assert!(matches!(arena.get(root).value, NodeValue::Document));
    }

    #[test]
    fn test_parse_heading() {
        let options = Options::default();
        let (arena, root) = parse_document("# Heading", &options);

        // Should have one child (the heading)
        let heading = arena.first_child(root).expect("Should have a child");
        assert!(matches!(arena.get(heading).value, NodeValue::Heading(_)));
    }

    #[test]
    fn test_parse_paragraph() {
        let options = Options::default();
        let (arena, root) = parse_document("Hello world", &options);

        let para = arena.first_child(root).expect("Should have a child");
        assert!(matches!(arena.get(para).value, NodeValue::Paragraph));
    }

    #[test]
    fn test_parse_multiple_paragraphs() {
        let options = Options::default();
        let (arena, root) = parse_document("First para\n\nSecond para", &options);

        let first = arena.first_child(root).expect("Should have first child");
        assert!(matches!(arena.get(first).value, NodeValue::Paragraph));

        let second = arena.next_sibling(first).expect("Should have second child");
        assert!(matches!(arena.get(second).value, NodeValue::Paragraph));
    }

    #[test]
    fn test_parse_link_with_title() {
        let options = Options::default();
        let (arena, root) = parse_document("[foo]\n\n[foo]: /url \"title\"", &options);

        // Get the paragraph
        let para = arena.first_child(root).expect("Should have a child");
        assert!(matches!(arena.get(para).value, NodeValue::Paragraph));

        // Get the link inside the paragraph
        let link = arena.first_child(para).expect("Should have link");
        if let NodeValue::Link(link_data) = &arena.get(link).value {
            assert_eq!(link_data.url, "/url");
            assert_eq!(link_data.title, "title");

            // Also test HTML output
            let html = crate::html::render(&arena, root, 0);
            println!("HTML output: {:?}", html);
            assert!(
                html.contains("title=\"title\""),
                "HTML should contain title attribute"
            );
        } else {
            panic!("Expected link node");
        }
    }

    // Tests for parser limits
    #[test]
    fn test_parse_document_with_limits_input_size() {
        let options = Options::default();
        let limits = ParserLimits {
            max_input_size: 100, // Very small limit
            ..ParserLimits::default()
        };

        // Small input should succeed
        let result = parse_document_with_limits("Hello", &options, limits);
        assert!(result.is_ok());

        // Large input should fail
        let large_input = "a".repeat(101);
        let result = parse_document_with_limits(&large_input, &options, limits);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too large"));
    }

    #[test]
    fn test_parse_document_with_limits_line_length() {
        let options = Options::default();
        let limits = ParserLimits {
            max_line_length: 50, // Short line limit
            max_input_size: 1000,
            ..ParserLimits::default()
        };

        // Short lines should succeed
        let result = parse_document_with_limits("Hello\nWorld", &options, limits);
        assert!(result.is_ok());

        // Long line should fail (exactly 51 characters)
        let long_line = "a".repeat(51);
        let result = parse_document_with_limits(&long_line, &options, limits);
        assert!(
            result.is_err(),
            "Expected error for line with {} characters, but got Ok",
            long_line.len()
        );

        // Line exactly at limit should succeed
        let exact_line = "a".repeat(50);
        let result = parse_document_with_limits(&exact_line, &options, limits);
        assert!(
            result.is_ok(),
            "Expected Ok for line with {} characters, but got error",
            exact_line.len()
        );
    }

    #[test]
    fn test_parse_document_with_limits_default() {
        let options = Options::default();
        let limits = ParserLimits::default();

        // Normal document should succeed with default limits
        let input = "# Heading\n\nParagraph with **bold** text.";
        let result = parse_document_with_limits(input, &options, limits);
        assert!(result.is_ok());

        let (arena, root) = result.unwrap();
        assert!(matches!(arena.get(root).value, NodeValue::Document));
    }

    #[test]
    fn test_parse_document_with_limits_large_document() {
        let options = Options::default();
        let limits = ParserLimits {
            max_input_size: 10 * 1024 * 1024, // 10MB
            max_line_length: 2 * 1024 * 1024, // Allow 2MB lines for this test
            ..ParserLimits::default()
        };

        // Create a 1MB document (split into multiple lines)
        let large_content = "a".repeat(1024 * 1024);
        // Split into lines of 1000 characters each
        let lines: Vec<&str> = large_content
            .as_bytes()
            .chunks(1000)
            .map(|chunk| std::str::from_utf8(chunk).unwrap())
            .collect();
        let input = format!("# Large Document\n\n{}", lines.join("\n"));

        let result = parse_document_with_limits(&input, &options, limits);
        assert!(
            result.is_ok(),
            "Should parse large document: {:?}",
            result.err()
        );
    }
}
