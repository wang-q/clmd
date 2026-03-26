//! Block parser core implementation
//!
//! This module provides the main BlockParser struct and its core parsing logic.

use crate::arena::{Node, NodeArena, NodeId};
use crate::blocks::BlockInfo;
use crate::error::ParserLimits;
use crate::node::{NodeData, NodeType};
use rustc_hash::FxHashMap;

/// Block parser state using Arena allocation
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
    /// Block info for each node
    pub(crate) block_info: Vec<Option<BlockInfo>>,
    /// Map from node ID to block_info index
    pub(crate) node_to_index: FxHashMap<NodeId, usize>,
    /// Next available index in block_info
    pub next_index: usize,
    /// Options for parsing
    pub options: u32,
    /// Parser limits for input validation
    pub limits: ParserLimits,
    /// Current nesting depth
    pub nesting_depth: usize,
}

impl<'a> BlockParser<'a> {
    /// Create a new block parser with the given arena
    pub fn new(arena: &'a mut NodeArena) -> Self {
        Self::new_with_options(arena, 0)
    }

    /// Create a new block parser with the given arena and options
    pub fn new_with_options(arena: &'a mut NodeArena, options: u32) -> Self {
        Self::new_with_limits(arena, options, ParserLimits::default())
    }

    /// Create a new block parser with custom limits
    pub fn new_with_limits(
        arena: &'a mut NodeArena,
        options: u32,
        limits: ParserLimits,
    ) -> Self {
        let doc = arena.alloc(Node::new(NodeType::Document));
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
            block_info: Vec::with_capacity(64),
            node_to_index: FxHashMap::default(),
            next_index: 0,
            options,
            limits,
            nesting_depth: 0,
        };

        // Initialize block info for document
        parser.set_block_info(doc, BlockInfo::new());

        parser
    }

    /// Parse a complete document
    pub fn parse(arena: &'a mut NodeArena, input: &str) -> NodeId {
        Self::parse_with_options(arena, input, 0)
    }

    /// Parse a complete document with options
    pub fn parse_with_options(
        arena: &'a mut NodeArena,
        input: &str,
        options: u32,
    ) -> NodeId {
        Self::parse_with_limits(arena, input, options, ParserLimits::default())
    }

    /// Parse a complete document with custom limits
    pub fn parse_with_limits(
        arena: &'a mut NodeArena,
        input: &str,
        options: u32,
        limits: ParserLimits,
    ) -> NodeId {
        // Validate input size
        let input_size = input.len();
        if input_size > limits.max_input_size {
            // For now, we truncate the input instead of failing
            // In the future, this could return an error
            eprintln!(
                "Warning: Input size ({} bytes) exceeds maximum ({} bytes). Truncating.",
                input_size, limits.max_input_size
            );
        }

        let mut parser = Self::new_with_limits(arena, options, limits);

        // Process lines directly without creating intermediate String
        // Handle CRLF line endings by splitting on '\n' and removing '\r' if present
        for line in input.split('\n') {
            // Check line length limit
            if line.len() > parser.limits.max_line_length {
                eprintln!(
                    "Warning: Line {} exceeds maximum length ({} > {}). Truncating.",
                    parser.line_number + 1,
                    line.len(),
                    parser.limits.max_line_length
                );
                let truncated = &line[..parser.limits.max_line_length];
                let line = truncated.strip_suffix('\r').unwrap_or(truncated);
                parser.process_line(line);
            } else {
                // Remove trailing '\r' if present (CRLF handling)
                let line = line.strip_suffix('\r').unwrap_or(line);
                parser.process_line(line);
            }
        }

        // Finalize all blocks before processing the empty line
        // This ensures that fenced code blocks are closed properly
        // and don't get an extra newline added to their content
        parser.finalize_document();

        parser.doc
    }

    /// Process a single line
    pub fn process_line(&mut self, line: &str) {
        self.line_number += 1;
        self.offset = 0;
        self.column = 0;
        self.blank = false;
        self.partially_consumed_tab = false;

        // Check if we need to modify the line (NUL replacement)
        if line.contains('\u{0000}') {
            self.current_line = line.replace('\u{0000}', "\u{FFFD}");
        } else {
            self.current_line.clear();
            self.current_line.push_str(line);
        }

        // Ensure line ends with newline
        if !self.current_line.ends_with('\n') {
            self.current_line.push('\n');
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

        // Check if container is a leaf that accepts lines
        let container_type = self.arena.get(container).node_type;
        let _accepts_lines = self.accepts_lines(container);
        let is_leaf =
            matches!(container_type, NodeType::Heading | NodeType::ThematicBreak);

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
        let smart = (self.options & crate::options::SMART) != 0;

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
            );
        }
    }

    /// Recursively collect leaf blocks that need inline processing
    fn collect_leaf_blocks(
        &self,
        node: NodeId,
        leaf_blocks: &mut Vec<(NodeId, String, usize)>,
    ) {
        let node_type = self.arena.get(node).node_type;

        // Check if this is a leaf block that needs inline processing
        match node_type {
            NodeType::Paragraph => {
                // For paragraphs, content is stored in NodeData::Text after finalization
                let node_ref = self.arena.get(node);
                let content = match &node_ref.data {
                    NodeData::Text { literal } => literal.clone(),
                    _ => self.get_string_content(node),
                };
                let line = self.get_start_line(node);
                if !content.is_empty() && content != "__EMPTY_PARAGRAPH__" {
                    leaf_blocks.push((node, content, line));
                }
            }
            NodeType::Heading => {
                // For headings, content is stored in NodeData::Heading
                let node_ref = self.arena.get(node);
                let content = match &node_ref.data {
                    NodeData::Heading { content, .. } => content.clone(),
                    _ => self.get_string_content(node),
                };
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
