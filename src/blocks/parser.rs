//! Block parser core implementation
//!
//! This module provides the main BlockParser struct and its core parsing logic.

use crate::core::arena::{Node, NodeArena, NodeId};
use crate::blocks::BlockInfo;
use crate::core::error::{ParseError, ParseResult, ParserLimits};
use crate::core::nodes::NodeValue;
use crate::parser::OPT_VALIDATE_UTF8;
use rustc_hash::FxHashMap;

/// Maximum input size: 100MB
#[allow(dead_code)]
const MAX_INPUT_SIZE: usize = 100 * 1024 * 1024;

/// Maximum nesting depth
#[allow(dead_code)]
const MAX_NESTING_DEPTH: usize = 1000;

/// Maximum line length: 10MB
#[allow(dead_code)]
const MAX_LINE_LENGTH: usize = 10_000_000;

/// Maximum list items
#[allow(dead_code)]
const MAX_LIST_ITEMS: usize = 1_000_000;

/// Maximum links
#[allow(dead_code)]
const MAX_LINKS: usize = 1_000_000;

/// Type alias for the reference map: label -> (url, title)
pub type RefMap = FxHashMap<String, (String, String)>;

/// Block parser state using Arena allocation
#[derive(Debug)]
pub struct BlockParser<'a> {
    /// Arena for node allocation
    pub(crate) arena: &'a mut NodeArena,
    /// Root document node ID
    pub doc: NodeId,
    /// Current tip (last open block)
    pub tip: NodeId,
    /// Old tip for tracking unmatched blocks
    pub old_tip: NodeId,
    /// Last matched container
    pub last_matched_container: NodeId,
    /// Current line being processed
    pub(crate) current_line: String,
    /// Current line number
    pub line_number: usize,
    /// Current offset in line
    pub offset: usize,
    /// Current column
    pub column: usize,
    /// Next non-space position
    pub next_nonspace: usize,
    /// Next non-space column
    pub next_nonspace_column: usize,
    /// Current indent level
    pub indent: usize,
    /// Whether current line is indented
    pub indented: bool,
    /// Whether current line is blank
    pub blank: bool,
    /// Whether we partially consumed a tab
    pub partially_consumed_tab: bool,
    /// Whether all containers are closed
    pub all_closed: bool,
    /// Last line length
    pub last_line_length: usize,
    /// Reference map for link references: label -> (url, title)
    pub refmap: FxHashMap<String, (String, String)>,
    /// Block info for each node (NodeId -> BlockInfo)
    pub(crate) block_info: FxHashMap<NodeId, BlockInfo>,
    /// Options for parsing
    pub options: u32,
    /// Parser limits for input validation
    pub limits: ParserLimits,
}

impl<'a> BlockParser<'a> {
    /// Create a new block parser with the given arena
    #[cfg(test)]
    pub fn new(arena: &'a mut NodeArena) -> Self {
        Self::new_with_options(arena, 0)
    }

    /// Create a new block parser with the given arena and options
    #[cfg(test)]
    pub fn new_with_options(arena: &'a mut NodeArena, options: u32) -> Self {
        Self::new_with_limits(arena, options, ParserLimits::default())
    }

    /// Create a new block parser with custom limits
    pub fn new_with_limits(
        arena: &'a mut NodeArena,
        options: u32,
        limits: ParserLimits,
    ) -> Self {
        let doc = arena.alloc(Node::with_value(NodeValue::Document));
        let tip = doc;
        let old_tip = doc;
        let last_matched_container = doc;

        let mut parser = BlockParser {
            arena,
            doc,
            tip,
            old_tip,
            last_matched_container,
            current_line: String::new(),
            line_number: 0,
            offset: 0,
            column: 0,
            next_nonspace: 0,
            next_nonspace_column: 0,
            indent: 0,
            indented: false,
            blank: false,
            partially_consumed_tab: false,
            all_closed: true,
            last_line_length: 0,
            refmap: FxHashMap::default(),
            block_info: FxHashMap::default(),
            options,
            limits,
        };

        // Initialize block info for document
        parser.set_block_info(doc, BlockInfo::new());

        parser
    }

    /// Parse a complete document
    #[cfg(test)]
    pub fn parse(arena: &'a mut NodeArena, input: &str) -> NodeId {
        Self::parse_with_options(arena, input, 0)
    }

    /// Parse a complete document with options
    ///
    /// Uses relaxed limits for backward compatibility.
    /// For strict limits, use `parse_with_limits`.
    ///
    /// # Panics
    ///
    /// This function will panic if parsing fails (e.g., input too large).
    /// For error handling, use `parse_with_limits` instead.
    pub fn parse_with_options(
        arena: &'a mut NodeArena,
        input: &str,
        options: u32,
    ) -> NodeId {
        // Use relaxed limits for backward compatibility
        let limits = ParserLimits::new();
        // For backward compatibility, unwrap the result
        // In new code, use parse_with_limits which returns ParseResult
        match Self::parse_with_limits(arena, input, options, limits) {
            Ok(doc) => doc,
            Err(e) => {
                // Log the error and panic with details
                eprintln!("BlockParser::parse_with_options failed: {}", e);
                panic!("Parsing failed: {}", e);
            }
        }
    }

    /// Parse a complete document with custom limits
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if:
    /// - Input exceeds maximum allowed size
    /// - Line length exceeds maximum allowed
    /// - Nesting depth exceeds maximum allowed
    pub fn parse_with_limits(
        arena: &'a mut NodeArena,
        input: &str,
        options: u32,
        limits: ParserLimits,
    ) -> ParseResult<NodeId> {
        Self::parse_with_limits_and_refmap(arena, input, options, limits)
            .map(|(doc, _)| doc)
    }

    /// Parse a complete document with custom limits and return the refmap
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if:
    /// - Input exceeds maximum allowed size
    /// - Line length exceeds maximum allowed
    /// - Nesting depth exceeds maximum allowed
    pub fn parse_with_limits_and_refmap(
        arena: &'a mut NodeArena,
        input: &str,
        options: u32,
        limits: ParserLimits,
    ) -> ParseResult<(NodeId, RefMap)> {
        // Validate input size (0 means unlimited)
        let input_size = input.len();
        if limits.max_input_size > 0 && input_size > limits.max_input_size {
            return Err(ParseError::InputTooLarge {
                size: input_size,
                max_size: limits.max_input_size,
            });
        }

        let mut parser = Self::new_with_limits(arena, options, limits);

        // Process lines directly without creating intermediate String
        // Handle CRLF line endings by splitting on '\n' and removing '\r' if present
        for line in input.split('\n') {
            // Check line length limit
            if parser.limits.max_line_length > 0
                && line.len() > parser.limits.max_line_length
            {
                return Err(ParseError::ParseError {
                    position: crate::error::Position::new(parser.line_number + 1, 1),
                    message: format!(
                        "Line exceeds maximum length ({} > {})",
                        line.len(),
                        parser.limits.max_line_length
                    ),
                });
            }

            // Remove trailing '\r' if present (CRLF handling)
            let line = line.strip_suffix('\r').unwrap_or(line);
            parser.process_line(line);
        }

        // Finalize all blocks before processing the empty line
        // This ensures that fenced code blocks are closed properly
        // and don't get an extra newline added to their content
        parser.finalize_document();

        Ok((parser.doc, parser.refmap))
    }

    /// Process a single line
    pub fn process_line(&mut self, line: &str) {
        self.line_number += 1;
        self.offset = 0;
        self.column = 0;
        self.blank = false;
        self.partially_consumed_tab = false;

        // Check if we need to validate UTF-8
        let validate_utf8 = (self.options & OPT_VALIDATE_UTF8) != 0;

        // Optimization: avoid String allocation for common case (no NUL characters)
        let has_nul = line.contains('\u{0000}');

        // Ensure line ends with newline
        let needs_newline = !line.ends_with('\n');

        // If validate_utf8 is enabled, we always process the line to ensure
        // proper sanitization (even if no NUL characters are present)
        if validate_utf8 || has_nul || needs_newline {
            // Need to create a modified line
            self.current_line.clear();
            if validate_utf8 || has_nul {
                // Replace NUL characters with U+FFFD
                for c in line.chars() {
                    if c == '\u{0000}' {
                        self.current_line.push('\u{FFFD}');
                    } else {
                        self.current_line.push(c);
                    }
                }
            } else {
                self.current_line.push_str(line);
            }
            if needs_newline {
                self.current_line.push('\n');
            }
        } else {
            // Optimization: use the line directly without copying
            // This avoids the String allocation entirely
            self.current_line.clear();
            self.current_line.push_str(line);
        }

        self.incorporate_line();
        self.last_line_length = line.len();
    }

    /// Incorporate a line into the document
    fn incorporate_line(&mut self) {
        let mut all_matched = true;

        self.old_tip = self.tip;
        self.all_closed = true;

        // Try to match existing containers
        let last_matched_container = self.check_open_blocks(&mut all_matched);

        let mut container = match last_matched_container {
            Some(c) => c,
            None => return,
        };
        self.last_matched_container = container;

        // Check if container is a leaf block
        let container_value = &self.arena.get(container).value;
        let is_leaf = matches!(
            container_value,
            NodeValue::Heading(..) | NodeValue::ThematicBreak
        );

        // Try new block starts if not a leaf block
        if !is_leaf {
            self.find_next_nonspace();

            // Try each block start function
            container = self.open_new_blocks(container, all_matched);
        }

        // Add line content to appropriate container
        self.add_text_to_container(container);
    }

    /// Finalize the entire document
    pub fn finalize_document(&mut self) {
        // Finalize all remaining open blocks
        while self.tip != self.doc {
            let tip = self.tip;
            self.finalize_block(tip);
        }
        self.finalize_block(self.doc);

        // Remove link reference definitions from the document
        self.remove_link_reference_definitions();

        // Process inline content for leaf blocks
        self.process_inlines();
    }

    /// Process inline content for all leaf blocks in the document
    fn process_inlines(&mut self) {
        // Collect all leaf blocks that need inline processing
        let mut leaf_blocks: Vec<(NodeId, String, usize)> = Vec::new();
        self.collect_leaf_blocks(self.doc, &mut leaf_blocks);

        // Check if smart punctuation is enabled
        let smart = (self.options & crate::parser::OPT_SMART) != 0;
        // Check if math dollars is enabled
        let math_dollars = (self.options & crate::parser::OPT_MATH_DOLLARS) != 0;

        // Process each leaf block
        for (node_id, content, line) in leaf_blocks {
            // Pass refmap by reference to avoid cloning
            crate::inlines::parse_inlines_with_options(
                self.arena,
                node_id,
                &content,
                line,
                0,
                &self.refmap,
                smart,
                math_dollars,
            );
        }
    }

    /// Recursively collect leaf blocks that need inline processing
    fn collect_leaf_blocks(
        &self,
        node: NodeId,
        leaf_blocks: &mut Vec<(NodeId, String, usize)>,
    ) {
        let node_value = &self.arena.get(node).value;

        // Check if this is a leaf block that needs inline processing
        match node_value {
            NodeValue::Paragraph => {
                // For paragraphs, content is stored in string_content after finalization
                let content = self.get_string_content(node);
                let line = self.get_start_line(node);
                if !content.is_empty() && content != super::EMPTY_PARAGRAPH_MARKER {
                    leaf_blocks.push((node, content, line));
                }
            }
            NodeValue::Heading(_heading) => {
                // For headings, get content from string_content
                // The heading content will be processed by inlines
                let content = self.get_string_content(node);
                let line = self.get_start_line(node);
                if !content.is_empty() {
                    leaf_blocks.push((node, content, line));
                }
            }
            _ => {
                // For container blocks, recursively process children
                let mut current = self.arena.get(node).first_child;
                while let Some(child) = current {
                    self.collect_leaf_blocks(child, leaf_blocks);
                    current = self.arena.get(child).next;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_utf8_replaces_nul() {
        let arena = &mut NodeArena::new();
        let options = OPT_VALIDATE_UTF8;

        // Test with NUL character (should be replaced with U+FFFD)
        let input = "Hello\x00World";
        let doc = BlockParser::parse_with_options(arena, input, options);

        // Verify the document was parsed
        assert_eq!(arena.get(doc).value, NodeValue::Document);

        // Check that the paragraph content has the NUL replaced
        let first_child = arena.get(doc).first_child;
        assert!(first_child.is_some());

        let paragraph = first_child.unwrap();
        assert_eq!(arena.get(paragraph).value, NodeValue::Paragraph);

        // The text node should contain the replacement character
        let text_node = arena.get(paragraph).first_child;
        assert!(text_node.is_some());

        if let NodeValue::Text(text) = &arena.get(text_node.unwrap()).value {
            assert!(text.contains('\u{FFFD}'));
            assert!(!text.contains('\u{0000}'));
        } else {
            panic!("Expected text node");
        }
    }

    #[test]
    fn test_validate_utf8_without_option_preserves_nul() {
        // Note: Currently clmd always replaces NUL, but with validate_utf8 disabled
        // it might preserve them in the future. This test documents current behavior.
        let arena = &mut NodeArena::new();
        let options = 0; // No VALIDATE_UTF8

        let input = "Hello\x00World";
        let doc = BlockParser::parse_with_options(arena, input, options);

        // Even without the option, NUL is currently replaced for security
        let first_child = arena.get(doc).first_child.unwrap();
        let paragraph = first_child;
        let text_node = arena.get(paragraph).first_child.unwrap();

        if let NodeValue::Text(text) = &arena.get(text_node).value {
            // Current behavior: NUL is always replaced
            assert!(text.contains('\u{FFFD}'));
        } else {
            panic!("Expected text node");
        }
    }

    #[test]
    fn test_validate_utf8_valid_utf8() {
        let arena = &mut NodeArena::new();
        let options = OPT_VALIDATE_UTF8;

        // Test with valid UTF-8 (should parse normally)
        let input = "Hello 世界 🌍";
        let doc = BlockParser::parse_with_options(arena, input, options);

        assert_eq!(arena.get(doc).value, NodeValue::Document);

        let first_child = arena.get(doc).first_child.unwrap();
        let paragraph = first_child;
        let text_node = arena.get(paragraph).first_child.unwrap();

        if let NodeValue::Text(text) = &arena.get(text_node).value {
            assert_eq!(text.as_ref(), "Hello 世界 🌍");
        } else {
            panic!("Expected text node");
        }
    }
}
