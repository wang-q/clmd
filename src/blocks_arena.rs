//! Block-level parsing with Arena allocation
//!
//! This is the Arena-based version of block parsing,
//! intended to replace the Rc<RefCell> version for better performance.

use crate::arena::Node;
use crate::arena::{NodeArena, NodeId, TreeOps};
use crate::node::NodeType;
use std::collections::HashMap;

/// Block info for tracking fenced code blocks and list items
#[derive(Debug, Clone)]
pub struct BlockInfo {
    /// Block is open
    pub is_open: bool,
    /// For fenced code blocks: fence character
    pub fence_char: char,
    /// For fenced code blocks: fence length
    pub fence_length: usize,
    /// For fenced code blocks: fence offset
    pub fence_offset: usize,
    /// For list items: marker offset
    pub marker_offset: usize,
    /// For list items: padding
    pub padding: usize,
    /// For HTML blocks: block type (1-7)
    pub html_block_type: u8,
    /// For headings: setext flag
    pub is_setext: bool,
    /// Last line blank flag
    pub last_line_blank: bool,
    /// String content accumulator
    pub string_content: String,
}

impl BlockInfo {
    fn new() -> Self {
        BlockInfo {
            is_open: true,
            fence_char: '\0',
            fence_length: 0,
            fence_offset: 0,
            marker_offset: 0,
            padding: 0,
            html_block_type: 0,
            is_setext: false,
            last_line_blank: false,
            string_content: String::new(),
        }
    }
}

/// Block parser state using Arena allocation
pub struct BlockParser<'a> {
    /// Arena for node allocation
    arena: &'a mut NodeArena,
    /// Root document node ID
    pub doc: NodeId,
    /// Current tip (last open block)
    pub tip: NodeId,
    /// Old tip for tracking unmatched blocks
    pub old_tip: NodeId,
    /// Last matched container
    pub last_matched_container: NodeId,
    /// Current line being processed
    pub current_line: String,
    /// Current line number
    pub line_number: usize,
    /// Reference map for link references
    pub refmap: HashMap<String, (String, String)>,
    /// Block info storage
    block_info: Vec<Option<BlockInfo>>,
    /// Map from node ID to block_info index
    node_to_index: HashMap<NodeId, usize>,
    /// Next available index
    next_index: usize,
}

impl<'a> BlockParser<'a> {
    /// Create a new block parser with the given arena
    pub fn new(arena: &'a mut NodeArena) -> Self {
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
            refmap: HashMap::new(),
            block_info: Vec::with_capacity(64),
            node_to_index: HashMap::with_capacity(64),
            next_index: 0,
        };

        // Initialize block info for document
        parser.set_block_info(doc, BlockInfo::new());

        parser
    }

    /// Parse a complete document
    pub fn parse(arena: &'a mut NodeArena, input: &str) -> NodeId {
        let mut parser = Self::new(arena);

        for line in input.split('\n') {
            let line = if line.ends_with('\r') {
                &line[..line.len() - 1]
            } else {
                line
            };
            parser.process_line(line);
        }

        parser.finalize_document();
        parser.doc
    }

    /// Process a single line
    pub fn process_line(&mut self, line: &str) {
        self.line_number += 1;

        // Handle NUL characters
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
    }

    /// Incorporate a line into the document
    fn incorporate_line(&mut self) {
        // Simple implementation: create paragraph nodes for non-empty lines
        let trimmed = self.current_line.trim();

        if trimmed.is_empty() {
            // Blank line - finalize current tip if it's a paragraph
            if self.get_node_type(self.tip) == NodeType::Paragraph {
                let tip = self.tip;
                self.finalize_block(tip);
            }
        } else {
            // Check for ATX heading
            if let Some(level) = self.parse_atx_heading() {
                self.create_heading(level);
            } else if self.current_line.starts_with("```") {
                // Fenced code block
                self.handle_code_fence();
            } else if self.current_line.starts_with("> ") {
                // Block quote
                self.handle_block_quote();
            } else if self.current_line.starts_with("- ")
                || self.current_line.starts_with("* ")
            {
                // List item
                self.handle_list_item();
            } else {
                // Regular paragraph
                self.handle_paragraph();
            }
        }
    }

    /// Parse ATX heading and return level if found
    fn parse_atx_heading(&self) -> Option<u32> {
        let line = self.current_line.trim_start();
        let mut level = 0;

        for c in line.chars() {
            if c == '#' && level < 6 {
                level += 1;
            } else {
                break;
            }
        }

        // Must have space after #
        if level > 0 && line.chars().nth(level as usize) == Some(' ') {
            Some(level)
        } else {
            None
        }
    }

    /// Create a heading node
    fn create_heading(&mut self, level: u32) {
        // Extract heading content
        let content = self
            .current_line
            .trim_start()
            .trim_start_matches('#')
            .trim_start()
            .trim_end()
            .to_string();

        let heading = self.arena.alloc(Node::with_data(
            NodeType::Heading,
            crate::node::NodeData::Heading { level, content },
        ));

        TreeOps::append_child(self.arena, self.doc, heading);
        self.set_block_info(heading, BlockInfo::new());
    }

    /// Handle code fence
    fn handle_code_fence(&mut self) {
        // Simplified: just create a code block node
        let code_block = self.arena.alloc(Node::with_data(
            NodeType::CodeBlock,
            crate::node::NodeData::CodeBlock {
                info: String::new(),
                literal: String::new(),
            },
        ));

        TreeOps::append_child(self.arena, self.doc, code_block);
        self.set_block_info(code_block, BlockInfo::new());
    }

    /// Handle block quote
    fn handle_block_quote(&mut self) {
        let content = self
            .current_line
            .trim_start_matches("> ")
            .trim_end()
            .to_string();

        // Check if we need to create new block quote or append to existing
        let bq = self.arena.alloc(Node::new(NodeType::BlockQuote));
        TreeOps::append_child(self.arena, self.doc, bq);
        self.set_block_info(bq, BlockInfo::new());

        // Add paragraph inside block quote
        let para = self.arena.alloc(Node::with_data(
            NodeType::Paragraph,
            crate::node::NodeData::Paragraph,
        ));
        TreeOps::append_child(self.arena, bq, para);
        self.set_block_info(para, BlockInfo::new());
        self.append_string_content(para, &content);
        self.append_string_content(para, "\n");
    }

    /// Handle list item
    fn handle_list_item(&mut self) {
        let content = self
            .current_line
            .trim_start_matches("- ")
            .trim_start_matches("* ")
            .trim_end()
            .to_string();

        // Create list if needed
        let list = self.arena.alloc(Node::with_data(
            NodeType::List,
            crate::node::NodeData::List {
                list_type: crate::node::ListType::Bullet,
                delim: crate::node::DelimType::None,
                start: 0,
                tight: false,
                bullet_char: '-',
            },
        ));
        TreeOps::append_child(self.arena, self.doc, list);
        self.set_block_info(list, BlockInfo::new());

        // Create item
        let item = self.arena.alloc(Node::new(NodeType::Item));
        TreeOps::append_child(self.arena, list, item);
        self.set_block_info(item, BlockInfo::new());

        // Add paragraph inside item
        let para = self.arena.alloc(Node::with_data(
            NodeType::Paragraph,
            crate::node::NodeData::Paragraph,
        ));
        TreeOps::append_child(self.arena, item, para);
        self.set_block_info(para, BlockInfo::new());
        self.append_string_content(para, &content);
        self.append_string_content(para, "\n");
    }

    /// Handle paragraph
    fn handle_paragraph(&mut self) {
        // Check if current tip is a paragraph we can continue
        if self.get_node_type(self.tip) != NodeType::Paragraph {
            // Create new paragraph
            let para = self.arena.alloc(Node::with_data(
                NodeType::Paragraph,
                crate::node::NodeData::Paragraph,
            ));
            TreeOps::append_child(self.arena, self.doc, para);
            self.set_block_info(para, BlockInfo::new());
            self.tip = para;
        }

        // Append content to current paragraph
        let content = self.current_line.trim_end().to_string();
        self.append_string_content(self.tip, &content);
        self.append_string_content(self.tip, "\n");
    }

    /// Finalize the document
    pub fn finalize_document(&mut self) {
        while self.tip != self.doc {
            let tip = self.tip;
            self.finalize_block(tip);
        }
        self.finalize_block(self.doc);
    }

    /// Finalize a block
    fn finalize_block(&mut self, block: NodeId) {
        // Convert string content to text node for paragraphs
        let node_type = self.get_node_type(block);
        if node_type == NodeType::Paragraph || node_type == NodeType::Heading {
            if let Some(content) = self.get_string_content(block) {
                if !content.is_empty() {
                    // Create text node with content
                    let text_node = self.arena.alloc(Node::with_data(
                        NodeType::Text,
                        crate::node::NodeData::Text { literal: content },
                    ));
                    TreeOps::append_child(self.arena, block, text_node);
                }
            }
        }

        // Move tip to parent
        if let Some(parent) = self.arena.get(block).parent {
            self.tip = parent;
        }

        // Mark as closed
        if let Some(info) = self.get_block_info_mut(block) {
            info.is_open = false;
        }
    }

    // Helper methods
    fn get_node_type(&self, node_id: NodeId) -> NodeType {
        self.arena.get(node_id).node_type
    }

    // Block info management
    fn set_block_info(&mut self, node_id: NodeId, info: BlockInfo) {
        let index = self.next_index;
        self.next_index += 1;

        if index >= self.block_info.len() {
            self.block_info.resize_with(index + 1, || None);
        }
        self.block_info[index] = Some(info);
        self.node_to_index.insert(node_id, index);
    }

    fn get_block_info(&self, node_id: NodeId) -> Option<&BlockInfo> {
        if let Some(&index) = self.node_to_index.get(&node_id) {
            self.block_info[index].as_ref()
        } else {
            None
        }
    }

    fn get_block_info_mut(&mut self, node_id: NodeId) -> Option<&mut BlockInfo> {
        if let Some(&index) = self.node_to_index.get(&node_id) {
            self.block_info[index].as_mut()
        } else {
            None
        }
    }

    fn get_string_content(&self, node_id: NodeId) -> Option<String> {
        self.get_block_info(node_id)
            .map(|info| info.string_content.clone())
    }

    fn append_string_content(&mut self, node_id: NodeId, value: &str) {
        if let Some(info) = self.get_block_info_mut(node_id) {
            info.string_content.push_str(value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arena_parser_creation() {
        let mut arena = NodeArena::new();
        let parser = BlockParser::new(&mut arena);

        assert_eq!(parser.doc, 0);
        assert_eq!(parser.tip, 0);
    }

    #[test]
    fn test_arena_parse_simple() {
        let mut arena = NodeArena::new();
        let doc = BlockParser::parse(&mut arena, "Hello world");

        assert_eq!(doc, 0);
        assert_eq!(arena.get(doc).node_type, NodeType::Document);
    }

    #[test]
    fn test_arena_parse_heading() {
        let mut arena = NodeArena::new();
        let doc = BlockParser::parse(&mut arena, "# Heading 1");

        assert_eq!(arena.get(doc).node_type, NodeType::Document);
        // Check that heading was created as child
        let first_child = arena.get(doc).first_child;
        assert!(first_child.is_some());
        assert_eq!(arena.get(first_child.unwrap()).node_type, NodeType::Heading);
    }

    #[test]
    fn test_arena_parse_paragraphs() {
        let mut arena = NodeArena::new();
        let doc = BlockParser::parse(&mut arena, "Para 1\n\nPara 2");

        assert_eq!(arena.get(doc).node_type, NodeType::Document);
        // Should have two paragraph children
        let first_child = arena.get(doc).first_child;
        assert!(first_child.is_some());
    }
}
