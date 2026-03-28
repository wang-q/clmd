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

use crate::error::{ParseError, ParseResult, ParserLimits};
use crate::nodes::{self, Ast, LineColumn, Node, NodeValue, SourcePos};
use crate::scanners::{self, SetextChar};
use crate::strings::{self, Case};
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
    // Create root document node
    let root: Node<'a> = arena.alloc(nodes::NodeValue::Document.into());

    // Create parser and process document
    let mut parser = ParserInner::new(arena, root, options);
    parser.parse(md);

    root
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

/// Internal parser state and implementation.
///
/// This struct manages the parsing process, tracking the current position,
/// open blocks, and other state needed during parsing.
///
/// Inspired by comrak's Parser design, with additional fields for accurate
/// source position tracking and efficient parsing.
struct ParserInner<'a, 'o> {
    /// Arena for node allocation
    arena: &'a crate::Arena<'a>,

    /// Root document node
    root: Node<'a>,

    /// Current node being processed
    current: Node<'a>,

    /// Parser options
    options: &'o Options<'o>,

    /// Reference map for link references
    refmap: HashMap<String, nodes::NodeLink>,

    /// Line number (1-based)
    line_number: usize,

    /// Current byte offset in the current line
    offset: usize,

    /// Current column (accounting for tabs, 0-based)
    column: usize,

    /// Position of the first non-space character in the current line
    first_nonspace: usize,

    /// Column of the first non-space character (accounting for tabs)
    first_nonspace_column: usize,

    /// Current indentation level (difference between column and first_nonspace_column)
    indent: usize,

    /// Whether the last line was blank
    last_line_blank: bool,

    /// Whether the current line is blank
    blank: bool,

    /// Whether we've partially consumed a tab character
    partially_consumed_tab: bool,

    /// Position to kill thematic break attempts (for performance)
    thematic_break_kill_pos: usize,

    /// Length of the current line (in bytes)
    curline_len: usize,

    /// End column of the current line
    curline_end_col: usize,

    /// Length of the last processed line
    last_line_length: usize,

    /// Total size of the input (for limits and progress tracking)
    total_size: usize,

    /// Stack of open blocks
    open_blocks: Vec<Node<'a>>,
}

impl<'a, 'o> ParserInner<'a, 'o> {
    /// Create a new parser instance.
    fn new(arena: &'a crate::Arena<'a>, root: Node<'a>, options: &'o Options) -> Self {
        ParserInner {
            arena,
            root,
            current: root,
            options,
            refmap: HashMap::new(),
            line_number: 0,
            offset: 0,
            column: 0,
            first_nonspace: 0,
            first_nonspace_column: 0,
            indent: 0,
            last_line_blank: false,
            blank: false,
            partially_consumed_tab: false,
            thematic_break_kill_pos: 0,
            curline_len: 0,
            curline_end_col: 0,
            last_line_length: 0,
            total_size: 0,
            open_blocks: vec![root],
        }
    }

    /// Parse the input document.
    fn parse(&mut self, md: &str) {
        // Normalize line endings
        let md = strings::normalize_newlines(md);

        // Process front matter if enabled
        let content = if self.options.extension.front_matter_delimiter.is_some() {
            self.process_front_matter(&md)
        } else {
            md.as_ref()
        };

        // Split into lines and process each line
        let lines: Vec<&str> = content.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            self.line_number = i + 1;
            self.process_line(line);
        }

        // Finalize all open blocks
        self.finalize_document();

        // Process inline content
        self.process_inlines();
    }

    /// Process YAML front matter at the beginning of the document.
    fn process_front_matter<'b>(&mut self, md: &'b str) -> &'b str {
        if let Some(ref delimiter) = self.options.extension.front_matter_delimiter {
            if md.starts_with(delimiter) {
                if let Some(end) = md[delimiter.len()..].find(delimiter) {
                    let front_matter = &md[delimiter.len()..delimiter.len() + end];
                    let node = self.arena.alloc(
                        NodeValue::FrontMatter(
                            front_matter.to_string().into_boxed_str(),
                        )
                        .into(),
                    );
                    self.root.append(node);
                    return &md[delimiter.len() + end + delimiter.len()..];
                }
            }
        }
        md
    }

    /// Process a single line of input.
    fn process_line(&mut self, line: &str) {
        self.offset = 0;
        self.column = 0;
        self.indent = strings::count_indent(line);

        // Check if line is blank
        let is_blank = strings::is_blank(line);

        // Try to continue open blocks
        if !self.check_open_blocks(line, is_blank) {
            // Could not continue open blocks, close them and try to open new ones
            self.close_unmatched_blocks();

            if !is_blank {
                // Try to open new blocks
                self.open_new_blocks(line);
            }
        }

        self.last_line_blank = is_blank;
    }

    /// Check if open blocks can continue with the current line.
    ///
    /// Returns true if at least one block could continue.
    fn check_open_blocks(&mut self, line: &str, is_blank: bool) -> bool {
        // Start from the deepest open block
        for i in (0..self.open_blocks.len()).rev() {
            let node = self.open_blocks[i];
            let value = &node.data().value;

            let can_continue = match value {
                NodeValue::BlockQuote => self.check_block_quote_continuation(line),
                NodeValue::CodeBlock(code) => {
                    if code.fenced {
                        self.check_code_fence_continuation(
                            line,
                            code.fence_char,
                            code.fence_length,
                        )
                    } else {
                        // Indented code block continues if line is indented or blank
                        is_blank || self.indent >= 4
                    }
                }
                NodeValue::List(..) | NodeValue::Item(..) => {
                    // Lists can continue if the line is not a new list marker at same level
                    !is_blank || self.open_blocks.len() > i + 1
                }
                NodeValue::Paragraph => {
                    // Paragraph continues unless interrupted
                    !is_blank && !self.is_paragraph_interruptor(line)
                }
                NodeValue::Document => {
                    // Document is just a container, always allow opening new blocks
                    false
                }
                _ => true,
            };

            if !can_continue {
                // Close this and all deeper blocks
                self.open_blocks.truncate(i);
                if !self.open_blocks.is_empty() {
                    self.current = *self.open_blocks.last().unwrap();
                } else {
                    self.current = self.root;
                    self.open_blocks.push(self.root);
                }
                return false;
            }
        }

        true
    }

    /// Check if a block quote can continue with the given line.
    fn check_block_quote_continuation(&self, line: &str) -> bool {
        let bytes = line.as_bytes();
        let mut i = self.offset;

        // Skip up to 3 spaces of indentation
        while i < bytes.len() && i < self.offset + 4 && bytes[i] == b' ' {
            i += 1;
        }

        // Check for > marker
        i < bytes.len() && bytes[i] == b'>'
    }

    /// Check if a code fence can continue with the given line.
    fn check_code_fence_continuation(
        &self,
        line: &str,
        fence_char: u8,
        fence_length: usize,
    ) -> bool {
        // Check if this is a closing fence
        if let Some(len) = scanners::close_code_fence(line) {
            if len >= fence_length {
                return false; // Closing fence found
            }
        }
        true // Continue the code block
    }

    /// Check if a line would interrupt a paragraph.
    fn is_paragraph_interruptor(&self, line: &str) -> bool {
        scanners::atx_heading_start(line).is_some()
            || scanners::thematic_break(line).is_some()
            || scanners::open_code_fence(line).is_some()
            || self.is_list_marker(line)
            || scanners::html_block_start(line).is_some()
    }

    /// Check if a line starts a list marker.
    fn is_list_marker(&self, line: &str) -> bool {
        self.parse_list_marker(line).is_some()
    }

    /// Try to parse a list marker from the beginning of a line.
    fn parse_list_marker(
        &self,
        line: &str,
    ) -> Option<(nodes::ListType, u8, usize, usize)> {
        let bytes = line.as_bytes();
        let mut i = self.offset;

        // Skip up to 3 spaces of indentation
        while i < bytes.len() && i < self.offset + 4 && bytes[i] == b' ' {
            i += 1;
        }

        if i >= bytes.len() {
            return None;
        }

        // Try bullet list marker
        if matches!(bytes[i], b'-' | b'+' | b'*') {
            let bullet_char = bytes[i];
            i += 1;

            // Must be followed by space or tab
            if i < bytes.len() && scanners::is_space_or_tab(bytes[i]) {
                return Some((
                    nodes::ListType::Bullet,
                    bullet_char,
                    i + 1,
                    i - self.offset,
                ));
            }
            return None;
        }

        // Try ordered list marker
        if bytes[i].is_ascii_digit() {
            let start = i;
            let mut num = 0usize;

            while i < bytes.len() && bytes[i].is_ascii_digit() {
                num = num * 10 + (bytes[i] - b'0') as usize;
                i += 1;

                // Limit to 9 digits
                if i - start > 9 {
                    return None;
                }
            }

            // Must have at least one digit and not start with 0 (unless it's just 0)
            if i == start || (bytes[start] == b'0' && i - start > 1) {
                return None;
            }

            // Check for delimiter
            if i < bytes.len() && matches!(bytes[i], b'.' | b')') {
                let delim = bytes[i];
                i += 1;

                // Must be followed by space or tab
                if i < bytes.len() && scanners::is_space_or_tab(bytes[i]) {
                    let list_type = nodes::ListType::Ordered;
                    let marker_width = i - self.offset;
                    return Some((list_type, delim, i + 1, marker_width));
                }
            }
        }

        None
    }

    /// Close unmatched blocks and prepare for new blocks.
    fn close_unmatched_blocks(&mut self) {
        // Finalize any blocks that are being closed
        while let Some(node) = self.open_blocks.pop() {
            if !std::ptr::eq(node, self.root) {
                let mut data = node.data_mut();
                data.open = false;
            }
        }
        self.open_blocks.push(self.root);
        self.current = self.root;
    }

    /// Try to open new blocks starting with the current line.
    fn open_new_blocks(&mut self, line: &str) {
        let mut remaining = line;

        // Try each block type in order
        if let Some(level) = scanners::atx_heading_start(line) {
            self.add_atx_heading(line, level);
            return;
        }

        if scanners::thematic_break(line).is_some() {
            self.add_thematic_break();
            return;
        }

        if let Some(fence_len) = scanners::open_code_fence(line) {
            self.add_code_fence(line, fence_len);
            return;
        }

        if line.len() >= self.offset + 4
            && line.as_bytes()[self.offset..self.offset + 4]
                .iter()
                .all(|&b| b == b' ')
        {
            // Indented code block
            self.add_indented_code_block(line);
            return;
        }

        if let Some((list_type, marker, content_start, marker_width)) =
            self.parse_list_marker(line)
        {
            self.add_list_item(line, list_type, marker, content_start, marker_width);
            return;
        }

        if let Some(html_type) = scanners::html_block_start(line) {
            self.add_html_block(line, html_type);
            return;
        }

        // Default: add a paragraph
        self.add_paragraph(line);
    }

    /// Add an ATX heading.
    fn add_atx_heading(&mut self, line: &str, level: usize) {
        let content = self.extract_atx_content(line, level);

        let heading = nodes::NodeHeading {
            level: level as u8,
            setext: false,
            closed: true,
        };

        let node = self.arena.alloc(NodeValue::Heading(heading).into());

        // Add text content
        if !content.is_empty() {
            let text = self
                .arena
                .alloc(NodeValue::make_text(content.trim()).into());
            node.append(text);
        }

        self.current.append(node);
    }

    /// Extract content from an ATX heading line.
    fn extract_atx_content<'b>(&self, line: &'b str, level: usize) -> &'b str {
        let bytes = line.as_bytes();
        let mut i = 0;

        // Skip leading whitespace
        while i < bytes.len() && bytes[i] == b' ' {
            i += 1;
        }

        // Skip hashes
        i += level;

        // Skip space after hashes
        if i < bytes.len() && scanners::is_space_or_tab(bytes[i]) {
            i += 1;
        }

        let start = i;
        let mut end = bytes.len();

        // Strip trailing whitespace and closing hashes
        while end > start {
            let c = bytes[end - 1];
            if c == b' ' || c == b'\t' {
                end -= 1;
            } else if c == b'#' {
                // Check if all remaining trailing chars are hashes or whitespace
                let mut j = end - 1;
                while j > start
                    && (bytes[j] == b'#' || bytes[j] == b' ' || bytes[j] == b'\t')
                {
                    j -= 1;
                }
                if j == start || bytes[j + 1] == b' ' || bytes[j + 1] == b'\t' {
                    end = j + 1;
                    break;
                } else {
                    end -= 1;
                }
            } else {
                break;
            }
        }

        &line[start..end]
    }

    /// Add a thematic break.
    fn add_thematic_break(&mut self) {
        let node = self.arena.alloc(NodeValue::ThematicBreak.into());
        self.current.append(node);
    }

    /// Add a fenced code block.
    fn add_code_fence(&mut self, line: &str, fence_len: usize) {
        let bytes = line.as_bytes();
        let mut i = 0;

        // Skip leading whitespace
        while i < bytes.len() && bytes[i] == b' ' {
            i += 1;
        }

        let fence_char = bytes[i];
        i += fence_len;

        // Extract info string
        let info_start = i;
        while i < bytes.len() && !scanners::is_line_end_char(bytes[i]) {
            i += 1;
        }
        let info = line[info_start..i].trim().to_string();

        let code_block = nodes::NodeCodeBlock {
            fenced: true,
            fence_char,
            fence_length: fence_len,
            fence_offset: self.offset,
            info,
            literal: String::new(),
            closed: false,
        };

        let node = self.arena.alloc(NodeValue::code_block(code_block).into());
        self.current.append(node);
        self.open_blocks.push(node);
        self.current = node;
    }

    /// Add an indented code block.
    fn add_indented_code_block(&mut self, line: &str) {
        let code_block = nodes::NodeCodeBlock {
            fenced: false,
            fence_char: 0,
            fence_length: 0,
            fence_offset: 0,
            info: String::new(),
            literal: String::new(),
            closed: false,
        };

        let node = self.arena.alloc(NodeValue::code_block(code_block).into());
        self.current.append(node);
        self.open_blocks.push(node);
        self.current = node;

        // Add the line content (minus indentation)
        let content = if line.len() >= 4 { &line[4..] } else { "" };
        self.append_line_to_current(content);
    }

    /// Add a list item.
    fn add_list_item(
        &mut self,
        line: &str,
        list_type: nodes::ListType,
        marker: u8,
        content_start: usize,
        marker_width: usize,
    ) {
        // Check if we need to create a new list
        let needs_new_list = if let NodeValue::List(ref list) = self.current.data().value
        {
            list.list_type != list_type
        } else {
            true
        };

        if needs_new_list {
            let list = nodes::NodeList {
                list_type,
                marker_offset: self.indent,
                padding: marker_width + 1,
                start: 1,
                delimiter: if marker == b'.' {
                    nodes::ListDelimType::Period
                } else {
                    nodes::ListDelimType::Paren
                },
                bullet_char: marker,
                tight: true,
                is_task_list: false,
            };

            let list_node = self.arena.alloc(NodeValue::List(list).into());
            self.current.append(list_node);
            self.open_blocks.push(list_node);
            self.current = list_node;
        }

        // Create the list item
        let item = nodes::NodeList {
            list_type,
            marker_offset: self.indent,
            padding: marker_width + 1,
            start: 1,
            delimiter: if marker == b'.' {
                nodes::ListDelimType::Period
            } else {
                nodes::ListDelimType::Paren
            },
            bullet_char: marker,
            tight: true,
            is_task_list: false,
        };

        let item_node = self.arena.alloc(NodeValue::Item(item).into());
        self.current.append(item_node);
        self.open_blocks.push(item_node);
        self.current = item_node;

        // Add content after marker
        if content_start < line.len() {
            let content = &line[content_start..];
            self.add_paragraph(content);
        }
    }

    /// Add an HTML block.
    fn add_html_block(&mut self, line: &str, block_type: u8) {
        let html_block = nodes::NodeHtmlBlock {
            block_type,
            literal: line.to_string(),
        };

        let node = self.arena.alloc(NodeValue::html_block(html_block).into());
        self.current.append(node);

        // Types 1-5 need special handling for closing
        if block_type <= 5 {
            self.open_blocks.push(node);
            self.current = node;
        }
    }

    /// Add a paragraph.
    fn add_paragraph(&mut self, line: &str) {
        // Check if we can extend an existing paragraph
        if let Some(last_child) = self.current.last_child() {
            if matches!(last_child.data().value, NodeValue::Paragraph)
                && !self.last_line_blank
            {
                // Extend existing paragraph
                self.append_line_to_node(last_child, line);
                return;
            }
        }

        // Create new paragraph
        let node = self.arena.alloc(NodeValue::Paragraph.into());
        self.current.append(node);
        self.open_blocks.push(node);
        self.current = node;

        self.append_line_to_current(line);
    }

    /// Append a line to the current node's content.
    fn append_line_to_current(&self, line: &str) {
        if let Some(node) = self.open_blocks.last() {
            self.append_line_to_node(*node, line);
        }
    }

    /// Append a line to a specific node's content.
    fn append_line_to_node(&self, node: Node<'a>, line: &str) {
        let mut data = node.data_mut();

        match data.value {
            NodeValue::CodeBlock(ref mut code) => {
                if !code.literal.is_empty() {
                    code.literal.push('\n');
                }
                code.literal.push_str(line);
            }
            NodeValue::HtmlBlock(ref mut html) => {
                if !html.literal.is_empty() {
                    html.literal.push('\n');
                }
                html.literal.push_str(line);
            }
            NodeValue::Paragraph => {
                if !data.content.is_empty() {
                    data.content.push('\n');
                }
                data.content.push_str(line);
            }
            _ => {}
        }
    }

    /// Finalize the document after all lines are processed.
    fn finalize_document(&mut self) {
        // Close all open blocks
        while let Some(node) = self.open_blocks.pop() {
            let mut data = node.data_mut();
            data.open = false;

            // Finalize specific block types
            match data.value {
                NodeValue::CodeBlock(ref mut code) => {
                    // Remove trailing newlines from code blocks
                    while code.literal.ends_with('\n') {
                        code.literal.pop();
                    }
                }
                NodeValue::Paragraph => {
                    // Move content to text node
                    if !data.content.is_empty() {
                        let content = std::mem::take(&mut data.content);
                        drop(data); // Release borrow

                        let text =
                            self.arena.alloc(NodeValue::make_text(content).into());
                        node.append(text);
                    }
                }
                _ => {}
            }
        }
    }

    /// Process inline content in leaf blocks.
    fn process_inlines(&mut self) {
        // This would call the inline parser on each leaf block's content
        // For now, this is a placeholder
        // TODO: Implement inline parsing
    }
}

/// Parser for CommonMark documents using Arena allocation.
///
/// This struct provides a higher-level API for parsing with error handling.
#[derive(Debug, Clone, Copy)]
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
    pub fn parse<'a>(
        &self,
        arena: &'a crate::Arena<'a>,
        text: &str,
    ) -> ParseResult<Node<'a>> {
        // Check input size
        if text.len() > self.limits.max_input_size {
            return Err(ParseError::InputTooLarge {
                size: text.len(),
                max_size: self.limits.max_input_size,
            });
        }

        // Create options from flags (simplified)
        let options = Options::default();
        let root = parse_document(arena, text, &options);
        Ok(root)
    }
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
    fn test_parse_thematic_break() {
        let arena = crate::Arena::new();
        let options = Options::default();
        let root = parse_document(&arena, "---", &options);

        let br = root.first_child().expect("Should have a child");
        assert!(matches!(br.data().value, NodeValue::ThematicBreak));
    }

    #[test]
    fn test_parse_code_fence() {
        let arena = crate::Arena::new();
        let options = Options::default();
        let root = parse_document(&arena, "```\ncode\n```", &options);

        let code = root.first_child().expect("Should have a child");
        assert!(matches!(code.data().value, NodeValue::CodeBlock(_)));
    }

    #[test]
    fn test_parser_creation() {
        let options = Options::default();
        let parser = Parser::new(&options);
        let arena = crate::Arena::new();
        let root = parser.parse(&arena, "Hello world").unwrap();
        assert!(matches!(root.data().value, NodeValue::Document));
    }
}
