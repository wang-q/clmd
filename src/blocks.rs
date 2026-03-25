use crate::arena::Node;
/// Block-level parsing for CommonMark documents
///
/// This module implements the block parsing algorithm based on the CommonMark spec.
/// It processes input line by line, building the AST structure using Arena allocation.
use crate::arena::{NodeArena, NodeId, TreeOps};
use crate::inlines_arena as inlines;
use crate::inlines_arena::{parse_reference, unescape_string};
use crate::lexer::{is_space_or_tab, CODE_INDENT, TAB_STOP};
use crate::node::{DelimType, ListType, NodeData, NodeType, SourcePos};

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
    pub refmap: std::collections::HashMap<String, (String, String)>,
    /// Block info for each node
    block_info: Vec<Option<BlockInfo>>,
    /// Map from node ID to block_info index
    node_to_index: std::collections::HashMap<NodeId, usize>,
    /// Next available index in block_info
    pub next_index: usize,
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
            refmap: std::collections::HashMap::new(),
            block_info: Vec::with_capacity(64),
            node_to_index: std::collections::HashMap::with_capacity(64),
            next_index: 0,
        };

        // Initialize block info for document
        parser.set_block_info(doc, BlockInfo::new());

        parser
    }

    /// Parse a complete document
    pub fn parse(arena: &'a mut NodeArena, input: &str) -> NodeId {
        let mut parser = Self::new(arena);

        // Process lines directly without creating intermediate String
        // Handle CRLF line endings by splitting on '\n' and removing '\r' if present
        for line in input.split('\n') {
            // Remove trailing '\r' if present (CRLF handling)
            let line = if line.ends_with('\r') {
                &line[..line.len() - 1]
            } else {
                line
            };
            parser.process_line(line);
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

        if last_matched_container.is_none() {
            return;
        }

        let mut container = last_matched_container.unwrap();
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

    /// Check which open blocks can continue on this line
    fn check_open_blocks(&mut self, all_matched: &mut bool) -> Option<NodeId> {
        *all_matched = true;
        let mut container = self.doc;

        loop {
            let last_child_opt = self.arena.get(container).last_child;
            if let Some(last_child) = last_child_opt {
                if !self.is_open(last_child) {
                    break;
                }
                container = last_child;
            } else {
                break;
            }

            self.find_next_nonspace();

            let result = self.check_container_continuation(container);
            match result {
                0 => {} // Matched, continue
                1 => {
                    // Failed to match
                    *all_matched = false;
                    // Get parent before modifying container
                    let parent_opt = self.arena.get(container).parent;
                    container = parent_opt.unwrap_or(self.doc);
                    break;
                }
                2 => return None, // End of line for fenced code
                _ => panic!("Invalid continuation result"),
            }
        }

        self.all_closed = container == self.old_tip;

        Some(container)
    }

    /// Check if a container can continue on this line
    fn check_container_continuation(&mut self, container: NodeId) -> i32 {
        let node_type = self.arena.get(container).node_type;

        match node_type {
            NodeType::BlockQuote => self.continue_block_quote(),
            NodeType::List => self.continue_list(container),
            NodeType::Item => self.continue_item(container),
            NodeType::CodeBlock => self.continue_code_block(container),
            NodeType::HtmlBlock => self.continue_html_block(container),
            NodeType::Heading => {
                // Headings can only contain one line
                1
            }
            NodeType::ThematicBreak => {
                // Thematic breaks can only contain one line
                1
            }
            NodeType::Paragraph => {
                if self.blank {
                    1
                } else {
                    0
                }
            }
            _ => 0,
        }
    }

    /// Continue block quote
    fn continue_block_quote(&mut self) -> i32 {
        if !self.indented && self.peek_next_nonspace() == Some('>') {
            // Advance past the >
            self.advance_next_nonspace();
            self.advance_offset(1, false);
            // Optional following space
            if self.peek_current().map_or(false, is_space_or_tab) {
                self.advance_offset(1, true);
            }
            0
        } else {
            1
        }
    }

    /// Continue list
    fn continue_list(&mut self, _container: NodeId) -> i32 {
        // Lists always continue - new list items are handled in open_new_blocks
        // This matches commonmark.js behavior where list.continue returns 0
        0
    }

    /// Continue list item
    fn continue_item(&mut self, container: NodeId) -> i32 {
        if self.blank {
            if self.arena.get(container).first_child.is_none() {
                // Blank line after empty list item
                1
            } else {
                // Blank line in list item - mark as loose but continue
                // The list item ends when we encounter a non-blank line that doesn't match
                self.advance_next_nonspace();
                0
            }
        } else {
            // Check indent
            let (marker_offset, padding) = self.get_list_data(container);
            if self.indent >= marker_offset + padding {
                // Advance past the list marker padding
                self.advance_offset(marker_offset + padding, true);
                0
            } else {
                1
            }
        }
    }

    /// Continue code block
    fn continue_code_block(&mut self, container: NodeId) -> i32 {
        let is_fenced = self.is_fenced_code_block(container);

        if is_fenced {
            let (fence_char, fence_length, fence_offset) =
                self.get_fence_info(container);
            // Fenced code block
            if self.indent <= 3 {
                let line = &self.current_line[self.next_nonspace..];
                if line.starts_with(fence_char) {
                    // Check for closing fence
                    let fence_chars: String =
                        line.chars().take_while(|&c| c == fence_char).collect();
                    if fence_chars.len() >= fence_length {
                        // Check that only whitespace follows the fence
                        let after_fence = &line[fence_chars.len()..];
                        let is_closing = after_fence.trim().is_empty();
                        if is_closing {
                            // Closing fence found
                            self.finalize_block(container);
                            return 2;
                        }
                    }
                }
            }
            // Continue with the code block - skip optional spaces of fence offset
            let mut i = fence_offset;
            while i > 0 && self.peek_current().map_or(false, is_space_or_tab) {
                self.advance_offset(1, true);
                i -= 1;
            }
            0
        } else {
            // Indented code block
            if self.indent >= CODE_INDENT {
                self.advance_offset(CODE_INDENT, true);
                0
            } else if self.blank {
                self.advance_next_nonspace();
                0
            } else {
                1
            }
        }
    }

    /// Continue HTML block
    fn continue_html_block(&self, container: NodeId) -> i32 {
        let html_block_type = self.get_html_block_type(container);

        // HTML blocks 6 and 7 can be interrupted by blank lines
        if self.blank && (html_block_type == 6 || html_block_type == 7) {
            1
        } else {
            0
        }
    }

    /// Try to open new blocks
    fn open_new_blocks(&mut self, container: NodeId, all_matched: bool) -> NodeId {
        let mut current_container = container;
        let mut maybe_lazy = self.arena.get(self.tip).node_type == NodeType::Paragraph;

        loop {
            self.find_next_nonspace();
            let indented = self.indent >= CODE_INDENT;

            // Check if we're inside a leaf block that accepts lines
            // (HTML blocks and code blocks don't allow nested blocks)
            let container_type = self.arena.get(current_container).node_type;
            let in_leaf_block =
                matches!(container_type, NodeType::HtmlBlock | NodeType::CodeBlock);

            // If we're inside a leaf block, don't try to start any new blocks
            if in_leaf_block {
                return current_container;
            }

            // Check if we can start a new block
            // Don't create indented code block if we're inside a leaf block
            if indented && !maybe_lazy && !self.blank && !in_leaf_block {
                // Indented code block
                self.close_unmatched_blocks();
                let code_block = self.add_child(NodeType::CodeBlock, self.offset);
                self.set_fence_info(code_block, '\0', 0, 0);
                self.advance_offset(CODE_INDENT, true);
                return code_block;
            }

            // Try block quote
            if !indented && self.peek_next_nonspace() == Some('>') {
                self.close_unmatched_blocks();
                self.advance_next_nonspace();
                self.advance_offset(1, false);
                if self.peek_current().map_or(false, is_space_or_tab) {
                    self.advance_offset(1, true);
                }
                let block_quote =
                    self.add_child(NodeType::BlockQuote, self.next_nonspace);
                current_container = block_quote;
                maybe_lazy = false;
                continue;
            }

            // Try ATX heading
            if !indented {
                let line = &self.current_line[self.next_nonspace..];
                let mut level = 0;
                for c in line.chars() {
                    if c == '#' {
                        level += 1;
                    } else {
                        break;
                    }
                }

                // ATX heading must have 1-6 # characters
                if level > 0 && level <= 6 {
                    let after_hashes = &line[level..];
                    // Check if this is a valid ATX heading:
                    // - Empty after hashes (e.g., "#")
                    // - Starts with space, tab, newline, or carriage return
                    // - Starts with # (for closing sequence)
                    if after_hashes.is_empty()
                        || after_hashes.starts_with(' ')
                        || after_hashes.starts_with('\t')
                        || after_hashes.starts_with('\n')
                        || after_hashes.starts_with('\r')
                        || after_hashes.starts_with('#')
                    {
                        self.close_unmatched_blocks();
                        self.advance_next_nonspace();
                        self.advance_offset(level, false);

                        // Extract content from the rest of the line
                        let content_start = self.offset;
                        let mut content = self.current_line[content_start..].to_string();

                        // Remove trailing newlines
                        content = content.trim_end_matches('\n').to_string();
                        content = content.trim_end_matches('\r').to_string();

                        // Remove closing sequence using regex-like logic
                        // Pattern 1: ^[ \t]*#+[ \t]*$ - content is only whitespace + #s
                        let trimmed_start =
                            content.trim_start_matches(|c: char| c == ' ' || c == '\t');
                        let trimmed_end = trimmed_start
                            .trim_end_matches(|c: char| c == ' ' || c == '\t');
                        if trimmed_end.chars().all(|c| c == '#')
                            && !trimmed_end.is_empty()
                        {
                            content = String::new();
                        } else {
                            // Pattern 2: [ \t]+#+[ \t]*$ - closing sequence at end
                            // Find the last sequence of #s (must be preceded by whitespace)
                            // Scan from end to find the hash sequence
                            let mut hash_start = None;
                            let mut in_hashes = false;
                            for (i, c) in content.char_indices().rev() {
                                if c == '#' {
                                    if !in_hashes {
                                        in_hashes = true;
                                    }
                                } else if c == ' ' || c == '\t' {
                                    if in_hashes {
                                        // Found whitespace before hashes - this is the closing sequence
                                        hash_start = Some(i + 1);
                                        break;
                                    }
                                } else {
                                    // Non-space, non-hash - stop scanning
                                    break;
                                }
                            }

                            // Also check if we reached the start while in_hashes
                            if in_hashes && hash_start.is_none() {
                                // The entire content is hashes (should have been caught by pattern 1)
                                hash_start = Some(0);
                            }

                            if let Some(start) = hash_start {
                                // Check if there's whitespace before the hash sequence
                                if start > 0 {
                                    let before_hash = &content[..start];
                                    if before_hash.ends_with(' ')
                                        || before_hash.ends_with('\t')
                                    {
                                        content = before_hash
                                            .trim_end_matches(|c: char| {
                                                c == ' ' || c == '\t'
                                            })
                                            .to_string();
                                    }
                                }
                            }
                        }

                        // Trim leading whitespace from content
                        content = content
                            .trim_start_matches(|c: char| c == ' ' || c == '\t')
                            .to_string();

                        let heading =
                            self.add_child(NodeType::Heading, self.next_nonspace);
                        {
                            let heading_mut = self.arena.get_mut(heading);
                            if let NodeData::Heading {
                                level: ref mut l,
                                content: ref mut c,
                            } = heading_mut.data
                            {
                                *l = level as u32;
                                *c = content;
                            }
                        }
                        // Skip the rest of the line
                        self.advance_offset(
                            self.current_line.len() - self.offset,
                            false,
                        );
                        return heading;
                    }
                }
            }

            // Try fenced code block
            if !indented {
                let line = self.current_line[self.next_nonspace..].to_string();
                if let Some(first_char) = line.chars().next() {
                    if first_char == '`' || first_char == '~' {
                        let mut fence_length = 0;
                        for c in line.chars() {
                            if c == first_char {
                                fence_length += 1;
                            } else {
                                break;
                            }
                        }

                        if fence_length >= 3 {
                            let rest = &line[fence_length..];
                            if first_char != '`' || !rest.contains('`') {
                                self.close_unmatched_blocks();
                                let info = unescape_string(rest.trim());
                                let code_block = self
                                    .add_child(NodeType::CodeBlock, self.next_nonspace);
                                {
                                    let code_mut = self.arena.get_mut(code_block);
                                    if let NodeData::CodeBlock {
                                        info: ref mut i, ..
                                    } = code_mut.data
                                    {
                                        *i = info;
                                    }
                                }
                                // fence_offset should be the position of the fence character
                                // For fenced code blocks inside block quotes, this is self.next_nonspace
                                self.set_fence_info(
                                    code_block,
                                    first_char,
                                    fence_length,
                                    self.next_nonspace,
                                );
                                self.advance_next_nonspace();
                                self.advance_offset(fence_length, false);
                                return code_block;
                            }
                        }
                    }
                }
            }

            // Try HTML block
            // Don't start a new HTML block if we're already inside an HTML block
            // (HTML blocks can contain other tags)
            let in_html_block =
                self.arena.get(current_container).node_type == NodeType::HtmlBlock;

            if !indented && !in_html_block && self.peek_next_nonspace() == Some('<') {
                let line = &self.current_line[self.next_nonspace..];
                if let Some(block_type) =
                    self.scan_html_block_start(line, current_container, maybe_lazy)
                {
                    self.close_unmatched_blocks();
                    let html_block = self.add_child(NodeType::HtmlBlock, self.offset);
                    self.set_html_block_type(html_block, block_type);
                    return html_block;
                }
            }

            // Try setext heading
            if !indented
                && self.arena.get(current_container).node_type == NodeType::Paragraph
            {
                let line = &self.current_line[self.next_nonspace..];
                if let Some(level) = self.scan_setext_heading_line(line) {
                    // Get the content before converting
                    let content = self.get_string_content(current_container);

                    // Process link reference definitions at the beginning of the paragraph
                    let mut processed_content = content.clone();

                    while !processed_content.is_empty() {
                        // Skip leading whitespace
                        let trimmed = processed_content.trim_start();
                        if !trimmed.starts_with('[') {
                            break;
                        }

                        // Try to parse a reference definition
                        let consumed =
                            parse_reference(&processed_content, &mut self.refmap);

                        if consumed == 0 {
                            break;
                        }

                        // Remove the consumed reference definition from content
                        processed_content = processed_content[consumed..].to_string();

                        // Skip leading whitespace for next iteration
                        processed_content = processed_content.trim_start().to_string();
                    }

                    // Only convert to heading if there's remaining content after processing
                    // reference definitions
                    let remaining_content = processed_content.trim();
                    if !remaining_content.is_empty() {
                        self.close_unmatched_blocks();
                        {
                            let container_mut = self.arena.get_mut(current_container);
                            container_mut.node_type = NodeType::Heading;
                            container_mut.data = NodeData::Heading {
                                level,
                                content: remaining_content.to_string(),
                            };
                        }
                        self.set_setext(current_container, true);
                        self.advance_offset(
                            self.current_line.len() - self.offset,
                            false,
                        );
                        return current_container;
                    }
                    // If no remaining content, don't convert to heading
                    // The Setext line will be processed as normal text
                }
            }

            // Try thematic break
            if !indented
                && !(self.arena.get(current_container).node_type == NodeType::Paragraph
                    && !all_matched)
            {
                let line = &self.current_line[self.next_nonspace..];
                if self.scan_thematic_break(line) {
                    self.close_unmatched_blocks();
                    let thematic_break =
                        self.add_child(NodeType::ThematicBreak, self.next_nonspace);
                    self.advance_offset(self.current_line.len() - self.offset, false);
                    return thematic_break;
                }
            }

            // Try list item
            if (!indented
                || self.arena.get(current_container).node_type == NodeType::List)
                && self.indent < 4
            {
                if let Some((
                    list_type,
                    delim,
                    start,
                    marker_offset,
                    padding,
                    bullet_char,
                )) = self.parse_list_marker(current_container)
                {
                    self.close_unmatched_blocks();

                    // Check if we can continue an existing list
                    let can_continue_list = self.arena.get(current_container).node_type
                        == NodeType::List
                        && self.lists_match(
                            current_container,
                            list_type,
                            delim,
                            start,
                            bullet_char,
                        );

                    if !can_continue_list {
                        current_container =
                            self.add_child(NodeType::List, self.next_nonspace);
                        {
                            let list_mut = self.arena.get_mut(current_container);
                            if let NodeData::List {
                                list_type: ref mut lt,
                                delim: ref mut d,
                                start: ref mut s,
                                tight: ref mut t,
                                bullet_char: ref mut bc,
                            } = list_mut.data
                            {
                                *lt = list_type;
                                *d = delim;
                                *s = start;
                                *t = true;
                                *bc = bullet_char;
                            }
                        }
                    }

                    // Add list item
                    let item = self.add_child(NodeType::Item, self.next_nonspace);
                    self.set_list_data(item, marker_offset, padding);
                    current_container = item;
                    maybe_lazy = false;
                    continue;
                }
            }

            // No new block started
            break;
        }

        current_container
    }

    /// Scan for HTML block start
    /// Based on commonmark.js reHtmlBlockOpen patterns
    fn scan_html_block_start(
        &self,
        line: &str,
        container: NodeId,
        maybe_lazy: bool,
    ) -> Option<u8> {
        // Type 1: <script, <pre, <textarea, <style followed by space, >, or EOL
        if self.match_html_block_type1(line) {
            return Some(1);
        }

        // Type 2: <!--
        if line.starts_with("<!--") {
            return Some(2);
        }

        // Type 3: <?
        if line.starts_with("<?") {
            return Some(3);
        }

        // Type 4: <! followed by uppercase letter (declaration)
        // According to commonmark.js: /^<![A-Za-z]/
        if line.starts_with("<!") && line.len() > 2 {
            let third_char = line.chars().nth(2).unwrap();
            if third_char.is_ascii_alphabetic() {
                return Some(4);
            }
        }

        // Type 5: <![CDATA[
        if line.starts_with("<![CDATA[") {
            return Some(5);
        }

        // Type 6: Specific block-level tags
        if self.match_html_block_type6(line) {
            return Some(6);
        }

        // Type 7: Complete HTML tag (cannot interrupt paragraph, not lazy)
        if line.starts_with('<') && !maybe_lazy {
            if self.arena.get(container).node_type != NodeType::Paragraph {
                if self.is_valid_html_tag_type7(line) {
                    return Some(7);
                }
            }
        }

        None
    }

    /// Match HTML block type 1: <script, <pre, <textarea, <style
    /// Must be followed by space, >, newline, or end of line
    fn match_html_block_type1(&self, line: &str) -> bool {
        let tags = ["script", "pre", "textarea", "style"];
        for tag in &tags {
            if line.len() < tag.len() + 1 {
                continue;
            }
            if line[1..].to_lowercase().starts_with(tag) {
                let after = &line[1 + tag.len()..];
                // Must be followed by space, tab, >, newline, or end of line
                return after.is_empty()
                    || after.starts_with(' ')
                    || after.starts_with('\t')
                    || after.starts_with('>')
                    || after.starts_with('\n')
                    || after.starts_with('\r');
            }
        }
        false
    }

    /// Match HTML block type 6: Block-level HTML tags
    /// Matches: <tag ...> or </tag ...> where tag is in the specific list
    fn match_html_block_type6(&self, line: &str) -> bool {
        let tags = [
            "address",
            "article",
            "aside",
            "base",
            "basefont",
            "blockquote",
            "body",
            "caption",
            "center",
            "col",
            "colgroup",
            "dd",
            "details",
            "dialog",
            "dir",
            "div",
            "dl",
            "dt",
            "fieldset",
            "figcaption",
            "figure",
            "footer",
            "form",
            "frame",
            "frameset",
            "h1",
            "h2",
            "h3",
            "h4",
            "h5",
            "h6",
            "head",
            "header",
            "hr",
            "html",
            "iframe",
            "legend",
            "li",
            "link",
            "main",
            "menu",
            "menuitem",
            "nav",
            "noframes",
            "ol",
            "optgroup",
            "option",
            "p",
            "param",
            "section",
            "search",
            "summary",
            "table",
            "tbody",
            "td",
            "tfoot",
            "th",
            "thead",
            "title",
            "tr",
            "track",
            "ul",
        ];

        // Tags that should end HTML block type 1, not start type 6
        let type1_end_tags = ["script", "pre", "textarea", "style"];

        let line_lower = line.to_lowercase();

        for tag in &tags {
            // Check for opening tag: <tag
            if line_lower.len() >= 1 + tag.len() && line_lower[1..].starts_with(tag) {
                let after = &line_lower[1 + tag.len()..];
                // Must be followed by space, tab, >, newline, or />
                if after.is_empty()
                    || after.starts_with(' ')
                    || after.starts_with('\t')
                    || after.starts_with('>')
                    || after.starts_with('\n')
                    || after.starts_with('\r')
                    || after.starts_with("/>")
                {
                    return true;
                }
            }

            // Check for closing tag: </tag
            if line_lower.len() >= 2 + tag.len() && line_lower[2..].starts_with(tag) {
                let after = &line_lower[2 + tag.len()..];
                // Must be followed by space, tab, >, newline, or />
                if after.is_empty()
                    || after.starts_with(' ')
                    || after.starts_with('\t')
                    || after.starts_with('>')
                    || after.starts_with('\n')
                    || after.starts_with('\r')
                    || after.starts_with("/>")
                {
                    // Don't match closing tags for type 1 tags (they end type 1 blocks)
                    if !type1_end_tags.contains(&tag.as_ref()) {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Check if a line is a valid HTML tag for type 7 HTML blocks
    /// Type 7: The entire line must be a complete open tag or close tag
    /// Based on commonmark.js: new RegExp("^(?:" + OPENTAG + "|" + CLOSETAG + ")\\s*$", "i")
    fn is_valid_html_tag_type7(&self, line: &str) -> bool {
        if !line.starts_with('<') {
            return false;
        }

        // Check for closing tag first: </tagname>
        if line.starts_with("</") {
            return self.is_valid_close_tag_type7(line);
        }

        // Check for open tag: <tagname ...>
        self.is_valid_open_tag_type7(line)
    }

    /// Check if line is a valid open tag for type 7
    /// Format: <tagname> or <tagname attr="value"> etc.
    fn is_valid_open_tag_type7(&self, line: &str) -> bool {
        let mut chars = line.chars().peekable();

        // Must start with <
        if chars.next() != Some('<') {
            return false;
        }

        // Tag name must start with a letter
        let _first_char = match chars.next() {
            Some(c) if c.is_ascii_alphabetic() => c,
            _ => return false,
        };

        // Rest of tag name: letters, digits, hyphens
        loop {
            match chars.peek() {
                Some(&c) if c.is_ascii_alphanumeric() || c == '-' => {
                    chars.next();
                }
                _ => break,
            }
        }

        // Now parse attributes and closing >
        self.parse_tag_attributes_and_close(&mut chars)
    }

    /// Check if line is a valid close tag for type 7
    /// Format: </tagname>
    fn is_valid_close_tag_type7(&self, line: &str) -> bool {
        let mut chars = line.chars().peekable();

        // Must start with </
        if chars.next() != Some('<') || chars.next() != Some('/') {
            return false;
        }

        // Tag name must start with a letter
        let _first_char = match chars.next() {
            Some(c) if c.is_ascii_alphabetic() => c,
            _ => return false,
        };

        // Rest of tag name: letters, digits, hyphens
        loop {
            match chars.peek() {
                Some(&c) if c.is_ascii_alphanumeric() || c == '-' => {
                    chars.next();
                }
                _ => break,
            }
        }

        // Skip whitespace
        while let Some(&c) = chars.peek() {
            if c.is_whitespace() {
                chars.next();
            } else {
                break;
            }
        }

        // Must end with >
        match chars.next() {
            Some('>') => {
                // Rest must be whitespace only
                chars.all(|c| c.is_whitespace())
            }
            _ => false,
        }
    }

    /// Parse tag attributes and closing >
    fn parse_tag_attributes_and_close(
        &self,
        chars: &mut std::iter::Peekable<std::str::Chars>,
    ) -> bool {
        // Track if we've seen whitespace before an attribute (required between attributes)
        let mut seen_whitespace = true; // Start true to allow first attribute

        loop {
            // Skip whitespace
            let mut found_whitespace = false;
            while let Some(&c) = chars.peek() {
                if c.is_whitespace() {
                    chars.next();
                    found_whitespace = true;
                } else {
                    break;
                }
            }
            if found_whitespace {
                seen_whitespace = true;
            }

            match chars.peek() {
                Some(&'>') => {
                    chars.next();
                    // Rest must be whitespace only
                    return chars.all(|c| c.is_whitespace());
                }
                Some(&'/') => {
                    // Self-closing tag />
                    chars.next();
                    match chars.peek() {
                        Some(&'>') => {
                            chars.next();
                            return chars.all(|c| c.is_whitespace());
                        }
                        _ => return false,
                    }
                }
                Some(&c) if c.is_ascii_alphabetic() || c == '_' => {
                    // Attribute name - must be preceded by whitespace (except for first attribute)
                    if !seen_whitespace {
                        return false;
                    }
                    seen_whitespace = false; // Reset for next attribute

                    chars.next();
                    loop {
                        match chars.peek() {
                            Some(&c)
                                if c.is_ascii_alphanumeric()
                                    || c == ':'
                                    || c == '_'
                                    || c == '-'
                                    || c == '.' =>
                            {
                                chars.next();
                            }
                            _ => break,
                        }
                    }

                    // Check for =value
                    if let Some(&'=') = chars.peek() {
                        chars.next();
                        // Parse attribute value
                        match chars.peek() {
                            Some(&'"') => {
                                chars.next();
                                loop {
                                    match chars.next() {
                                        Some('"') => break,
                                        Some(_) => continue,
                                        None => return false,
                                    }
                                }
                            }
                            Some(&'\'') => {
                                chars.next();
                                loop {
                                    match chars.next() {
                                        Some('\'') => break,
                                        Some(_) => continue,
                                        None => return false,
                                    }
                                }
                            }
                            _ => {
                                // Unquoted value
                                loop {
                                    match chars.peek() {
                                        Some(&c)
                                            if !c.is_whitespace()
                                                && c != '>'
                                                && c != '/' =>
                                        {
                                            chars.next();
                                        }
                                        _ => break,
                                    }
                                }
                            }
                        }
                    }
                    // Continue to next attribute
                }
                _ => return false,
            }
        }
    }

    /// Scan for setext heading line
    /// Setext heading underline must be a sequence of = or - characters only
    /// No spaces or other characters allowed between them
    fn scan_setext_heading_line(&self, line: &str) -> Option<u32> {
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            return None;
        }

        let first_char = trimmed.chars().next()?;
        if first_char != '=' && first_char != '-' {
            return None;
        }

        // Check that all characters are the same marker character
        // Spaces are NOT allowed between markers (per CommonMark spec)
        for c in trimmed.chars() {
            if c != first_char {
                return None;
            }
        }

        Some(if first_char == '=' { 1 } else { 2 })
    }

    /// Scan for thematic break
    fn scan_thematic_break(&self, line: &str) -> bool {
        let mut chars = line.chars().peekable();
        let mut c_opt: Option<char> = None;
        let mut count = 0;

        while let Some(&c) = chars.peek() {
            if c == ' ' || c == '\t' {
                chars.next();
                continue;
            }
            // Allow line ending characters
            if c == '\n' || c == '\r' {
                break;
            }
            if c != '*' && c != '-' && c != '_' {
                return false;
            }
            if let Some(prev_c) = c_opt {
                if c != prev_c {
                    return false;
                }
            } else {
                c_opt = Some(c);
            }
            count += 1;
            chars.next();
        }

        count >= 3
    }

    /// Parse list marker
    fn parse_list_marker(
        &mut self,
        container: NodeId,
    ) -> Option<(ListType, DelimType, u32, usize, usize, char)> {
        let rest = &self.current_line[self.next_nonspace..];

        // Try bullet list marker
        if let Some(first_char) = rest.chars().next() {
            if "*+-".contains(first_char) {
                let after_marker = &rest[1..];
                // A list marker must be followed by whitespace or end of line
                // For bullet lists, the marker can be followed by:
                // - nothing (end of line)
                // - space
                // - tab
                // - newline (empty list item)
                if after_marker.is_empty()
                    || after_marker.starts_with(' ')
                    || after_marker.starts_with('\t')
                    || after_marker.starts_with('\n')
                {
                    // Check for non-blank content if interrupting paragraph
                    // According to CommonMark spec 0.31.2:
                    // A list marker can interrupt a paragraph only if the list marker
                    // is not empty (i.e., it has content after it on the same line).
                    // However, when not interrupting a paragraph (i.e., at document start
                    // or after a blank line), an empty list item is allowed.
                    if self.arena.get(container).node_type == NodeType::Paragraph {
                        let content_after = after_marker.trim_start();
                        // Empty list item cannot interrupt paragraph
                        if content_after.is_empty() || content_after.starts_with('\n') {
                            return None;
                        }
                    }

                    self.advance_next_nonspace();
                    self.advance_offset(1, true);

                    let spaces_start_col = self.column;
                    let spaces_start_offset = self.offset;

                    // Skip up to 5 spaces
                    while self.column - spaces_start_col < 5
                        && self.peek_current().map_or(false, is_space_or_tab)
                    {
                        self.advance_offset(1, true);
                    }

                    let blank_item = self.peek_current().is_none()
                        || self.peek_current() == Some('\n');
                    let spaces_after_marker = self.column - spaces_start_col;

                    let padding;
                    if spaces_after_marker >= 5 || spaces_after_marker < 1 || blank_item
                    {
                        padding = 2; // marker length (1) + 1 space
                        self.column = spaces_start_col;
                        self.offset = spaces_start_offset;
                        if self.peek_current().map_or(false, is_space_or_tab) {
                            self.advance_offset(1, true);
                        }
                    } else {
                        padding = 1 + spaces_after_marker;
                    }

                    return Some((
                        ListType::Bullet,
                        DelimType::None,
                        0,
                        self.indent,
                        padding,
                        first_char, // Return the bullet character
                    ));
                }
            }
        }

        // Try ordered list marker
        let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        if !digits.is_empty() && digits.len() <= 9 {
            let start: u32 = digits.parse().ok()?;
            let after_digits = &rest[digits.len()..];

            if let Some(delim_char) = after_digits.chars().next() {
                if delim_char == '.' || delim_char == ')' {
                    let after_delim = &after_digits[1..];
                    // A list marker must be followed by whitespace or end of line
                    // For ordered lists, the marker can be followed by:
                    // - nothing (end of line)
                    // - space
                    // - tab
                    // - newline (empty list item)
                    if after_delim.is_empty()
                        || after_delim.starts_with(' ')
                        || after_delim.starts_with('\t')
                        || after_delim.starts_with('\n')
                    {
                        // If interrupting paragraph, start must be 1
                        if self.arena.get(container).node_type == NodeType::Paragraph
                            && start != 1
                        {
                            return None;
                        }

                        // Check for non-blank content if interrupting paragraph
                        if self.arena.get(container).node_type == NodeType::Paragraph {
                            let content_after = after_delim.trim_start();
                            if content_after.is_empty()
                                || content_after.starts_with('\n')
                            {
                                return None;
                            }
                        }

                        let delim = if delim_char == '.' {
                            DelimType::Period
                        } else {
                            DelimType::Paren
                        };

                        self.advance_next_nonspace();
                        self.advance_offset(digits.len() + 1, true);

                        let spaces_start_col = self.column;
                        let spaces_start_offset = self.offset;

                        // Skip up to 5 spaces
                        while self.column - spaces_start_col < 5
                            && self.peek_current().map_or(false, is_space_or_tab)
                        {
                            self.advance_offset(1, true);
                        }

                        let blank_item = self.peek_current().is_none()
                            || self.peek_current() == Some('\n');
                        let spaces_after_marker = self.column - spaces_start_col;

                        let padding;
                        if spaces_after_marker >= 5
                            || spaces_after_marker < 1
                            || blank_item
                        {
                            padding = digits.len() + 2; // marker length + 1 space
                            self.column = spaces_start_col;
                            self.offset = spaces_start_offset;
                            if self.peek_current().map_or(false, is_space_or_tab) {
                                self.advance_offset(1, true);
                            }
                        } else {
                            padding = digits.len() + 1 + spaces_after_marker;
                        }

                        return Some((
                            ListType::Ordered,
                            delim,
                            start,
                            self.indent,
                            padding,
                            '\0', // No bullet character for ordered lists
                        ));
                    }
                }
            }
        }

        None
    }

    /// Add text to container
    fn add_text_to_container(&mut self, container: NodeId) {
        self.find_next_nonspace();

        // Set last_line_blank for appropriate nodes
        if self.blank {
            if let Some(last_child) = self.arena.get(container).last_child {
                self.set_last_line_blank(last_child, true);
            }
        }

        // Determine if this line makes the container last_line_blank
        // Based on commonmark.js: blank && !(block_quote || heading || thematicBreak ||
        //   (code_block && fenced) || (item && firstChild == null && startLine == lineNumber))
        let container_type = self.arena.get(container).node_type;
        let last_line_blank = self.blank
            && container_type != NodeType::BlockQuote
            && container_type != NodeType::Heading
            && container_type != NodeType::ThematicBreak
            && container_type != NodeType::HtmlBlock
            && !(container_type == NodeType::CodeBlock
                && self.is_fenced_code_block(container))
            && !(container_type == NodeType::Item
                && self.arena.get(container).first_child.is_none()
                && self.get_start_line(container) == self.line_number);

        self.set_last_line_blank(container, last_line_blank);

        // Propagate last_line_blank up the tree
        let mut tmp = container;
        loop {
            let parent_opt = self.arena.get(tmp).parent;
            if let Some(parent) = parent_opt {
                self.set_last_line_blank(parent, false);
                tmp = parent;
            } else {
                break;
            }
        }

        // Check for lazy continuation
        let is_lazy = self.tip != self.last_matched_container
            && container == self.last_matched_container
            && !self.blank
            && self.arena.get(self.tip).node_type == NodeType::Paragraph;

        if is_lazy {
            self.add_line();
        } else {
            // Not a lazy continuation
            self.close_unmatched_blocks();

            let container_type = self.arena.get(container).node_type;

            if container_type == NodeType::CodeBlock {
                // For fenced code blocks, check if this is a fence line (opening or closing)
                // These lines should not be added to content
                if self.is_fenced_code_block(container) {
                    let (fence_char, fence_length, fence_offset) =
                        self.get_fence_info(container);

                    // Check if this line is a fence line (could be opening or closing)
                    let is_fence_line = if self.indent <= 3 {
                        // Check from the first non-space character
                        let line = &self.current_line[self.next_nonspace..];
                        if line.starts_with(fence_char) {
                            let fence_chars: String =
                                line.chars().take_while(|&c| c == fence_char).collect();
                            if fence_chars.len() >= fence_length {
                                let after_fence = &line[fence_chars.len()..];
                                after_fence.trim().is_empty()
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                    // Check if this is the opening fence line
                    // The opening fence line is only the first line of the fenced code block
                    // We can detect this by checking if this is the first time we're adding content
                    // to this code block. We track this by checking if the current line is the
                    // same line where the block was created.
                    let block_start_line =
                        self.arena.get(container).source_pos.start_line;
                    let is_first_line = self.line_number == block_start_line as usize;

                    // Also check if this line matches the opening fence pattern
                    let matches_fence_pattern = fence_offset < self.current_line.len()
                        && self.current_line[fence_offset..].starts_with(fence_char)
                        && self.current_line[fence_offset..]
                            .chars()
                            .take_while(|&c| c == fence_char)
                            .count()
                            >= fence_length
                        && {
                            let after_fence: String = self.current_line[fence_offset..]
                                .chars()
                                .skip_while(|&c| c == fence_char)
                                .collect();
                            after_fence.trim().is_empty()
                                || !after_fence.starts_with('`')
                        };

                    // It's an opening fence only if it's the first line AND it matches the pattern
                    let is_opening_fence = is_first_line && matches_fence_pattern;

                    // Only add line if it's not a fence line
                    if !is_fence_line && !is_opening_fence {
                        self.add_line_to_node(container);
                    }
                } else {
                    // Indented code block - always add line
                    self.add_line_to_node(container);
                }
            } else if container_type == NodeType::HtmlBlock {
                // For HTML blocks type 1-5, check if this line ends the block
                // If so, add the line first, then finalize
                let should_end = self.check_html_block_end(container);
                self.add_line_to_node(container);
                if should_end {
                    self.finalize_block(container);
                }
            } else if self.blank {
                // Do nothing for blank lines
            } else if self.accepts_lines(container) {
                if container_type == NodeType::Heading && !self.is_setext(container) {
                    self.chop_trailing_hashtags();
                }
                self.advance_next_nonspace();
                self.add_line_to_node(container);
            } else {
                // Create paragraph container for line
                let new_para = self.add_child(NodeType::Paragraph, self.next_nonspace);
                self.advance_next_nonspace();
                self.add_line_to_node(new_para);
                // Update tip to the new paragraph
                self.tip = new_para;
            }
        }
    }

    /// Chop trailing hashtags from ATX heading
    fn chop_trailing_hashtags(&mut self) {
        // Use offset as the start of content (after ATX marker)
        // At this point, offset points to just after the # characters
        let line = &self.current_line[self.offset..];

        // Trim leading whitespace first
        let line = line.trim_start_matches(|c: char| c == ' ' || c == '\t');

        // Now trim trailing whitespace (including newline) and hashtags
        let trimmed = line
            .trim_end_matches(|c: char| c == ' ' || c == '\t' || c == '\n' || c == '\r');

        // Remove trailing hashtags (must be preceded by space/tab or start of content)
        let mut end = trimmed.len();
        let mut hash_count = 0;
        let mut found_space = false;

        for (i, c) in trimmed.char_indices().rev() {
            if c == '#' {
                hash_count += 1;
            } else if c == ' ' || c == '\t' {
                // Space/tab before hashes - this is a valid closing sequence
                end = i;
                found_space = true;
                break;
            } else {
                // Non-space, non-hash character - not a closing sequence
                hash_count = 0;
                break;
            }
        }

        // If we found hashes and either:
        // 1. Found a space before them (normal case), or
        // 2. The hashes go all the way to the start (content is only whitespace)
        if hash_count > 0 && (found_space || end == trimmed.len()) {
            // Truncate the line to remove closing hashes and trailing spaces
            // Calculate position: offset + (original line len - trimmed len) + end
            let leading_ws_len = self.current_line[self.offset..].len() - line.len();
            let new_len = self.offset + leading_ws_len + end;
            self.current_line.truncate(new_len);
            // Also trim trailing spaces from the truncated line
            while self.current_line.ends_with(' ') || self.current_line.ends_with('\t') {
                self.current_line.pop();
            }
        }
    }

    /// Check for HTML block end condition
    /// Returns true if the block should end after this line
    fn check_html_block_end(&self, container: NodeId) -> bool {
        let html_block_type = self.get_html_block_type(container);
        let line = &self.current_line[self.offset..];

        match html_block_type {
            1 => {
                line.to_lowercase().contains("</script>")
                    || line.to_lowercase().contains("</pre>")
                    || line.to_lowercase().contains("</textarea>")
                    || line.to_lowercase().contains("</style>")
            }
            2 => line.contains("-->"),
            3 => line.contains("?>"),
            4 => line.contains(">"),
            5 => line.contains("]]>"),
            _ => false,
        }
    }

    /// Add current line to tip's content
    fn add_line(&mut self) {
        self.add_line_to_node(self.tip);
    }

    /// Add current line to a specific node's content
    fn add_line_to_node(&mut self, node: NodeId) {
        let mut line_content = String::new();

        // Handle partially consumed tab
        if self.partially_consumed_tab {
            self.offset += 1; // skip over tab
                              // Add space characters
            let chars_to_tab = TAB_STOP - (self.column % TAB_STOP);
            for _ in 0..chars_to_tab {
                line_content.push(' ');
            }
        }

        // Add remaining line content
        if self.offset < self.current_line.len() {
            line_content.push_str(&self.current_line[self.offset..]);
        }

        // Append to node's string content
        // Note: current_line already ends with newline from process_line
        self.append_string_content(node, &line_content);
    }

    /// Close unmatched blocks
    fn close_unmatched_blocks(&mut self) {
        if !self.all_closed {
            while self.old_tip != self.last_matched_container {
                let parent = self.arena.get(self.old_tip).parent.unwrap_or(self.doc);
                self.finalize_block(self.old_tip);
                self.old_tip = parent;
            }
            self.all_closed = true;
        }
    }

    /// Add a child to the tip
    fn add_child(&mut self, block_type: NodeType, start_column: usize) -> NodeId {
        // If tip can't accept this child, finalize it and try its parent
        while !self.can_contain(self.tip, block_type) {
            let parent = self.arena.get(self.tip).parent.unwrap_or(self.doc);
            self.finalize_block(self.tip);
            self.tip = parent;
        }

        let mut new_block = Node::new(block_type);
        new_block.source_pos.start_line = self.line_number as u32;
        new_block.source_pos.start_column = start_column as u32;

        let new_block_id = self.arena.alloc(new_block);
        TreeOps::append_child(self.arena, self.tip, new_block_id);

        // Initialize block info
        self.set_block_info(new_block_id, BlockInfo::new());

        self.tip = new_block_id;
        new_block_id
    }

    /// Finalize a block
    fn finalize_block(&mut self, block: NodeId) {
        // Set end position
        {
            let block_mut = self.arena.get_mut(block);
            block_mut.source_pos.end_line = self.line_number.saturating_sub(1) as u32;
            block_mut.source_pos.end_column = self.last_line_length as u32;
        }

        // Mark as closed
        self.set_open(block, false);

        // Finalize based on block type
        let node_type = self.arena.get(block).node_type;
        match node_type {
            NodeType::CodeBlock => {
                let is_fenced = self.is_fenced_code_block(block);
                if is_fenced {
                    // Get the current info string (set during block creation from fence line)
                    let _current_info = {
                        let block_ref = self.arena.get(block);
                        match &block_ref.data {
                            NodeData::CodeBlock { info, .. } => info.clone(),
                            _ => String::new(),
                        }
                    };

                    // For fenced code blocks, the info string was already set from the opening fence line
                    // We just need to process the content
                    let content = self.get_string_content(block);

                    // The content should end with a newline unless the block is empty
                    let processed_content = if content.is_empty() {
                        String::new()
                    } else if !content.ends_with('\n') {
                        format!("{}\n", content)
                    } else {
                        content.to_string()
                    };

                    self.set_string_content(block, processed_content);
                } else {
                    // Indented code block - remove trailing blank lines
                    let mut content = self.get_string_content(block);
                    while content.ends_with("\n\n") {
                        content.pop();
                    }
                    if !content.ends_with('\n') {
                        content.push('\n');
                    }
                    self.set_string_content(block, content);
                }

                // Move string content to literal
                let content = self.get_string_content(block);
                {
                    let block_mut = self.arena.get_mut(block);
                    if let NodeData::CodeBlock {
                        literal: ref mut l, ..
                    } = block_mut.data
                    {
                        *l = content;
                    }
                }
            }
            NodeType::HtmlBlock => {
                let content = self.get_string_content(block);
                // Remove trailing newline only (like commonmark.js: replace(/\n$/, ''))
                let content = content.strip_suffix('\n').unwrap_or(&content);
                {
                    let block_mut = self.arena.get_mut(block);
                    if let NodeData::HtmlBlock { literal: ref mut l } = block_mut.data {
                        *l = content.to_string();
                    }
                }
            }
            NodeType::Heading => {
                // For ATX headings, content was already set during creation
                // For Setext headings, content was also set during creation
                // Only update from string_content if content is empty (fallback)
                let string_content = self.get_string_content(block);

                // For Setext headings, process link reference definitions
                if self.is_setext(block) {
                    let mut content = string_content.clone();
                    let mut has_reference_defs = false;

                    while !content.is_empty() {
                        // Skip leading whitespace
                        let trimmed = content.trim_start();
                        if !trimmed.starts_with('[') {
                            break;
                        }

                        // Try to parse a reference definition
                        let consumed = parse_reference(&content, &mut self.refmap);

                        if consumed == 0 {
                            break;
                        }

                        has_reference_defs = true;

                        // Remove the consumed reference definition from content
                        content = content[consumed..].to_string();

                        // Skip leading whitespace for next iteration
                        content = content.trim_start().to_string();
                    }

                    // Update heading content if reference definitions were found
                    if has_reference_defs {
                        let block_mut = self.arena.get_mut(block);
                        if let NodeData::Heading {
                            content: ref mut c, ..
                        } = block_mut.data
                        {
                            *c = content.trim().to_string();
                        }
                    }
                } else {
                    let block_mut = self.arena.get_mut(block);
                    if let NodeData::Heading {
                        content: ref mut c, ..
                    } = block_mut.data
                    {
                        if c.is_empty() && !string_content.is_empty() {
                            *c = string_content.trim_end().to_string();
                        }
                    }
                }
            }
            NodeType::Paragraph => {
                let mut content = self.get_string_content(block);

                // Process link reference definitions at the beginning of the paragraph
                let mut has_reference_defs = false;
                let mut total_lines_removed: usize = 0;

                while !content.is_empty() {
                    // Skip leading whitespace
                    let trimmed = content.trim_start();
                    if !trimmed.starts_with('[') {
                        break;
                    }

                    // Try to parse a reference definition
                    let consumed = parse_reference(&content, &mut self.refmap);

                    if consumed == 0 {
                        // Not a reference definition, stop processing
                        break;
                    }

                    // Count lines in removed text
                    let removed_text = &content[..consumed];
                    let lines: Vec<&str> = removed_text.lines().collect();
                    let lines_removed = if removed_text.ends_with('\n') {
                        lines.len()
                    } else {
                        lines.len().saturating_sub(1)
                    };
                    total_lines_removed += lines_removed;

                    // Remove the parsed reference definition from the content
                    content = content[consumed..].to_string();
                    has_reference_defs = true;
                }

                // Update source_pos if we removed any reference definitions
                if total_lines_removed > 0 {
                    let block_mut = self.arena.get_mut(block);
                    let source_pos = block_mut.source_pos;
                    block_mut.source_pos = SourcePos {
                        start_line: source_pos.start_line + total_lines_removed as u32,
                        start_column: source_pos.start_column,
                        end_line: source_pos.end_line,
                        end_column: source_pos.end_column,
                    };
                }

                // Remove leading and trailing whitespace/newlines
                let content = content.trim();

                // If paragraph is empty after removing reference definitions, mark it for deletion
                if has_reference_defs && content.is_empty() {
                    // Store empty content marker
                    self.set_string_content(block, "__EMPTY_PARAGRAPH__".to_string());
                } else {
                    {
                        let block_mut = self.arena.get_mut(block);
                        if let NodeData::Text { literal: ref mut l } = block_mut.data {
                            *l = content.to_string();
                        } else {
                            block_mut.data = NodeData::Text {
                                literal: content.to_string(),
                            };
                        }
                    }
                }
            }
            NodeType::List => {
                // Determine tight/loose status
                let mut tight = true;
                let block_ref = self.arena.get(block);
                let mut item_opt = block_ref.first_child;

                while let Some(item) = item_opt {
                    // Check for non-final list item ending with blank line
                    if self.get_last_line_blank(item)
                        && self.arena.get(item).next.is_some()
                    {
                        tight = false;
                        break;
                    }

                    // Check children of list item
                    let mut subitem_opt = self.arena.get(item).first_child;
                    while let Some(subitem) = subitem_opt {
                        let has_next = self.arena.get(subitem).next.is_some();
                        let item_has_next = self.arena.get(item).next.is_some();
                        if (item_has_next || has_next)
                            && self.ends_with_blank_line(subitem)
                        {
                            tight = false;
                            break;
                        }
                        subitem_opt = self.arena.get(subitem).next;
                    }

                    if !tight {
                        break;
                    }
                    item_opt = self.arena.get(item).next;
                }

                drop(block_ref);
                {
                    let block_mut = self.arena.get_mut(block);
                    if let NodeData::List {
                        tight: ref mut t, ..
                    } = block_mut.data
                    {
                        *t = tight;
                    }
                }
            }
            _ => {}
        }

        // Move tip to parent
        let parent = self.arena.get(block).parent;
        if let Some(parent) = parent {
            self.tip = parent;
        }
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

        // Process each leaf block
        for (node_id, content, line) in leaf_blocks {
            // Get a copy of refmap for this call
            let refmap = self.refmap.clone();
            crate::inlines_arena::parse_inlines_with_options(
                self.arena, node_id, &content, line, 0, refmap, false,
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
            NodeType::Paragraph | NodeType::Heading => {
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

    /// Remove link reference definitions from the document
    /// This processes paragraph nodes marked as empty during finalization
    fn remove_link_reference_definitions(&mut self) {
        self.collect_and_remove_empty_paragraphs(self.doc);
    }

    /// Recursively collect and remove empty paragraphs in a single pass
    fn collect_and_remove_empty_paragraphs(&mut self, node: NodeId) {
        // Process children first (depth-first), handling next pointers carefully
        // since we might unlink nodes during traversal
        let first_child_opt = self.arena.get(node).first_child;
        if let Some(first_child) = first_child_opt {
            let mut current_opt = Some(first_child);
            while let Some(current) = current_opt {
                // Get next before processing, since current might be unlinked
                let next_opt = self.arena.get(current).next;
                self.collect_and_remove_empty_paragraphs(current);
                current_opt = next_opt;
            }
        }

        // Check if this is a paragraph marked as empty and remove it
        let node_type = self.arena.get(node).node_type;
        if node_type == NodeType::Paragraph {
            let content = self.get_string_content(node);
            if content == "__EMPTY_PARAGRAPH__" {
                TreeOps::unlink(self.arena, node);
            }
        }
    }

    /// Check if a node ends with a blank line
    /// Based on commonmark.js: returns true if block ends with a blank line
    fn ends_with_blank_line(&self, node: NodeId) -> bool {
        // Check if this node has a next sibling and there's a gap between them
        if let Some(next) = self.arena.get(node).next {
            let node_end_line = self.arena.get(node).source_pos.end_line;
            let next_start_line = self.arena.get(next).source_pos.start_line;
            // If there's a gap between this node and the next, there's a blank line
            if node_end_line + 1 < next_start_line {
                return true;
            }
        }

        // Also check last_line_blank flag for leaf nodes
        if self.get_last_line_blank(node) {
            return true;
        }

        // Recursively check last child for list/item containers
        let node_type = self.arena.get(node).node_type;
        if (node_type == NodeType::List || node_type == NodeType::Item)
            && self.arena.get(node).last_child.is_some()
        {
            if let Some(last_child) = self.arena.get(node).last_child {
                return self.ends_with_blank_line(last_child);
            }
        }

        false
    }

    /// Check if parent can contain child
    fn can_contain(&self, parent: NodeId, child_type: NodeType) -> bool {
        let parent_type = self.arena.get(parent).node_type;

        match parent_type {
            NodeType::Document | NodeType::BlockQuote => child_type != NodeType::Item,
            NodeType::List => child_type == NodeType::Item,
            NodeType::Item => child_type != NodeType::Item,
            _ => false,
        }
    }

    /// Check if block accepts lines
    fn accepts_lines(&self, block: NodeId) -> bool {
        let block_type = self.arena.get(block).node_type;
        matches!(
            block_type,
            NodeType::Paragraph | NodeType::CodeBlock | NodeType::HtmlBlock
        )
    }

    /// Lists match check
    /// For ordered lists, we don't check the start number because subsequent list items
    /// can have different numbers and still belong to the same list
    fn lists_match(
        &self,
        list: NodeId,
        list_type: ListType,
        delim: DelimType,
        _start: u32,
        bullet_char: char,
    ) -> bool {
        let node = self.arena.get(list);
        if let NodeData::List {
            list_type: lt,
            delim: d,
            bullet_char: bc,
            ..
        } = &node.data
        {
            // For bullet lists, also check the bullet character
            // Different bullet characters (-, +, *) should create new lists
            if list_type == ListType::Bullet && *lt == ListType::Bullet {
                *bc == bullet_char
            } else {
                *lt == list_type && *d == delim
            }
        } else {
            false
        }
    }

    // Block info accessors

    /// Get the index for a node, creating a new slot if needed
    #[inline]
    fn get_or_create_index(&mut self, node_id: NodeId) -> usize {
        if let Some(&index) = self.node_to_index.get(&node_id) {
            index
        } else {
            let index = self.next_index;
            self.node_to_index.insert(node_id, index);
            self.block_info.push(None);
            self.next_index += 1;
            index
        }
    }

    #[inline]
    fn get_index(&self, node_id: NodeId) -> Option<usize> {
        self.node_to_index.get(&node_id).copied()
    }

    fn get_block_info(&self, node_id: NodeId) -> Option<&BlockInfo> {
        self.get_index(node_id)
            .and_then(|idx| self.block_info.get(idx))
            .and_then(|opt| opt.as_ref())
    }

    fn get_block_info_mut(&mut self, node_id: NodeId) -> Option<&mut BlockInfo> {
        if let Some(idx) = self.get_index(node_id) {
            if let Some(Some(ref mut info)) = self.block_info.get_mut(idx) {
                return Some(info);
            }
        }
        None
    }

    fn set_block_info(&mut self, node_id: NodeId, info: BlockInfo) {
        let idx = self.get_or_create_index(node_id);
        if idx < self.block_info.len() {
            self.block_info[idx] = Some(info);
        }
    }

    fn is_open(&self, node_id: NodeId) -> bool {
        self.get_block_info(node_id)
            .map_or(false, |info| info.is_open)
    }

    fn set_open(&mut self, node_id: NodeId, open: bool) {
        if let Some(info) = self.get_block_info_mut(node_id) {
            info.is_open = open;
        }
    }

    fn get_string_content(&self, node_id: NodeId) -> String {
        self.get_block_info(node_id)
            .map_or(String::new(), |info| info.string_content.clone())
    }

    fn set_string_content(&mut self, node_id: NodeId, content: String) {
        if let Some(info) = self.get_block_info_mut(node_id) {
            info.string_content = content;
        }
    }

    fn append_string_content(&mut self, node_id: NodeId, value: &str) {
        if let Some(info) = self.get_block_info_mut(node_id) {
            info.string_content.push_str(value);
        }
    }

    fn is_fenced_code_block(&self, node_id: NodeId) -> bool {
        self.get_block_info(node_id)
            .map_or(false, |info| info.fence_length > 0)
    }

    fn get_fence_info(&self, node_id: NodeId) -> (char, usize, usize) {
        self.get_block_info(node_id).map_or(('\0', 0, 0), |info| {
            (info.fence_char, info.fence_length, info.fence_offset)
        })
    }

    fn set_fence_info(
        &mut self,
        node_id: NodeId,
        fence_char: char,
        fence_length: usize,
        fence_offset: usize,
    ) {
        if let Some(info) = self.get_block_info_mut(node_id) {
            info.fence_char = fence_char;
            info.fence_length = fence_length;
            info.fence_offset = fence_offset;
        }
    }

    fn get_list_data(&self, item: NodeId) -> (usize, usize) {
        self.get_block_info(item)
            .map_or((0, 2), |info| (info.marker_offset, info.padding))
    }

    fn set_list_data(&mut self, item: NodeId, marker_offset: usize, padding: usize) {
        if let Some(info) = self.get_block_info_mut(item) {
            info.marker_offset = marker_offset;
            info.padding = padding;
        }
    }

    fn get_html_block_type(&self, node_id: NodeId) -> u8 {
        self.get_block_info(node_id)
            .map_or(0, |info| info.html_block_type)
    }

    fn set_html_block_type(&mut self, node_id: NodeId, block_type: u8) {
        if let Some(info) = self.get_block_info_mut(node_id) {
            info.html_block_type = block_type;
        }
    }

    fn is_setext(&self, node_id: NodeId) -> bool {
        self.get_block_info(node_id)
            .map_or(false, |info| info.is_setext)
    }

    fn set_setext(&mut self, node_id: NodeId, setext: bool) {
        if let Some(info) = self.get_block_info_mut(node_id) {
            info.is_setext = setext;
        }
    }

    fn get_last_line_blank(&self, node_id: NodeId) -> bool {
        self.get_block_info(node_id)
            .map_or(false, |info| info.last_line_blank)
    }

    fn set_last_line_blank(&mut self, node_id: NodeId, blank: bool) {
        if let Some(info) = self.get_block_info_mut(node_id) {
            info.last_line_blank = blank;
        }
    }

    fn get_start_line(&self, node_id: NodeId) -> usize {
        self.arena.get(node_id).source_pos.start_line as usize
    }

    // Position and parsing helpers

    /// Find next non-space character
    fn find_next_nonspace(&mut self) {
        let mut chars_to_tab = TAB_STOP - (self.column % TAB_STOP);

        self.next_nonspace = self.offset;
        self.next_nonspace_column = self.column;

        while self.next_nonspace < self.current_line.len() {
            let c = self.current_line.as_bytes()[self.next_nonspace] as char;
            if c == ' ' {
                self.next_nonspace += 1;
                self.next_nonspace_column += 1;
                chars_to_tab -= 1;
                if chars_to_tab == 0 {
                    chars_to_tab = TAB_STOP;
                }
            } else if c == '\t' {
                self.next_nonspace += 1;
                self.next_nonspace_column += chars_to_tab;
                chars_to_tab = TAB_STOP;
            } else {
                break;
            }
        }

        self.indent = self.next_nonspace_column - self.column;
        self.blank = self.next_nonspace >= self.current_line.len()
            || self.current_line.as_bytes()[self.next_nonspace] == b'\n'
            || self.current_line.as_bytes()[self.next_nonspace] == b'\r';
    }

    /// Advance offset
    fn advance_offset(&mut self, count: usize, columns: bool) {
        let mut count = count;
        while count > 0 && self.offset < self.current_line.len() {
            let c = self.current_line.as_bytes()[self.offset] as char;
            if c == '\t' {
                let chars_to_tab = TAB_STOP - (self.column % TAB_STOP);
                if columns {
                    self.partially_consumed_tab = chars_to_tab > count;
                    let chars_to_advance = count.min(chars_to_tab);
                    self.column += chars_to_advance;
                    if !self.partially_consumed_tab {
                        self.offset += 1;
                    }
                    count -= chars_to_advance;
                } else {
                    self.partially_consumed_tab = false;
                    self.column += chars_to_tab;
                    self.offset += 1;
                    count -= 1;
                }
            } else {
                self.partially_consumed_tab = false;
                self.offset += 1;
                self.column += 1;
                count -= 1;
            }
        }
    }

    /// Advance to next non-space
    fn advance_next_nonspace(&mut self) {
        self.offset = self.next_nonspace;
        self.column = self.next_nonspace_column;
        self.partially_consumed_tab = false;
    }

    /// Peek at next non-space
    fn peek_next_nonspace(&self) -> Option<char> {
        if self.next_nonspace < self.current_line.len() {
            Some(self.current_line.as_bytes()[self.next_nonspace] as char)
        } else {
            None
        }
    }

    /// Peek at current position
    fn peek_current(&self) -> Option<char> {
        if self.offset < self.current_line.len() {
            Some(self.current_line.as_bytes()[self.offset] as char)
        } else {
            None
        }
    }

    /// Check if line might start a special block
    #[allow(dead_code)]
    fn maybe_special(&self) -> bool {
        if let Some(c) = self.peek_next_nonspace() {
            matches!(
                c,
                '#' | '`' | '~' | '*' | '_' | '+' | '=' | '<' | '-' | '0'..='9'
            )
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_creation() {
        let mut arena = NodeArena::new();
        let parser = BlockParser::new(&mut arena);
        assert_eq!(parser.arena.get(parser.doc).node_type, NodeType::Document);
        assert_eq!(parser.arena.get(parser.tip).node_type, NodeType::Document);
    }

    #[test]
    fn test_process_empty_line() {
        let mut arena = NodeArena::new();
        let mut parser = BlockParser::new(&mut arena);
        parser.process_line("");
        // Should not panic
    }

    #[test]
    fn test_parse_simple_paragraph() {
        let mut arena = NodeArena::new();
        let doc = BlockParser::parse(&mut arena, "Hello world");
        let first_child = arena.get(doc).first_child;
        assert!(first_child.is_some());
        assert_eq!(
            arena.get(first_child.unwrap()).node_type,
            NodeType::Paragraph
        );

        // Check paragraph content
        let para = arena.get(first_child.unwrap());
        if let NodeData::Text { literal } = &para.data {
            assert_eq!(literal, "Hello world");
        } else {
            panic!("Expected Text data");
        }
    }

    #[test]
    fn test_parse_block_quote() {
        let mut arena = NodeArena::new();
        let doc = BlockParser::parse(&mut arena, "> Quote line");
        let first_child = arena.get(doc).first_child;
        assert!(first_child.is_some());
        assert_eq!(
            arena.get(first_child.unwrap()).node_type,
            NodeType::BlockQuote
        );
    }

    #[test]
    fn test_parse_heading() {
        let mut arena = NodeArena::new();
        let doc = BlockParser::parse(&mut arena, "## Heading");
        let first_child = arena.get(doc).first_child;
        assert!(first_child.is_some());
        assert_eq!(arena.get(first_child.unwrap()).node_type, NodeType::Heading);
    }

    #[test]
    fn test_parse_fenced_code_block() {
        let input = "```\ncode\n```";
        let mut arena = NodeArena::new();
        let doc = BlockParser::parse(&mut arena, input);
        let first_child = arena.get(doc).first_child;
        assert!(first_child.is_some());
        assert_eq!(
            arena.get(first_child.unwrap()).node_type,
            NodeType::CodeBlock
        );
    }

    #[test]
    fn test_parse_thematic_break() {
        let mut arena = NodeArena::new();
        let doc = BlockParser::parse(&mut arena, "---");
        let first_child = arena.get(doc).first_child;
        assert!(first_child.is_some());
        assert_eq!(
            arena.get(first_child.unwrap()).node_type,
            NodeType::ThematicBreak
        );
    }

    #[test]
    fn test_parse_bullet_list() {
        let mut arena = NodeArena::new();
        let doc = BlockParser::parse(&mut arena, "* Item 1\n* Item 2");
        let first_child = arena.get(doc).first_child;
        assert!(first_child.is_some());
        assert_eq!(arena.get(first_child.unwrap()).node_type, NodeType::List);
    }

    #[test]
    fn test_parse_ordered_list() {
        let mut arena = NodeArena::new();
        let doc = BlockParser::parse(&mut arena, "1. Item 1\n2. Item 2");
        let first_child = arena.get(doc).first_child;
        assert!(first_child.is_some());
        assert_eq!(arena.get(first_child.unwrap()).node_type, NodeType::List);
    }

    #[test]
    fn test_parse_nested_block_quote() {
        let mut arena = NodeArena::new();
        let doc = BlockParser::parse(&mut arena, "> Outer\n> > Inner");
        let first_child = arena.get(doc).first_child;
        assert!(first_child.is_some());
        assert_eq!(
            arena.get(first_child.unwrap()).node_type,
            NodeType::BlockQuote
        );
    }

    #[test]
    fn test_parse_setext_heading() {
        let mut arena = NodeArena::new();
        let doc = BlockParser::parse(&mut arena, "Heading\n===");
        let first_child = arena.get(doc).first_child;
        assert!(first_child.is_some());
        assert_eq!(arena.get(first_child.unwrap()).node_type, NodeType::Heading);
    }

    #[test]
    fn test_remove_link_reference_definitions() {
        let input = "[label]: https://example.com\n\nSome text";
        let mut arena = NodeArena::new();
        let doc = BlockParser::parse(&mut arena, input);

        // The reference definition paragraph should be removed
        // So the first child should be the "Some text" paragraph
        let first_child = arena.get(doc).first_child;
        assert!(first_child.is_some(), "Document should have a first child");

        let first_child_ref = arena.get(first_child.unwrap());
        assert_eq!(
            first_child_ref.node_type,
            NodeType::Paragraph,
            "First child should be a paragraph"
        );

        // Check the paragraph's data - it should have the text content
        match &first_child_ref.data {
            NodeData::Text { literal } => {
                assert_eq!(
                    literal, "Some text",
                    "Paragraph content should be 'Some text'"
                );
            }
            _ => {
                // If data is not Text, check first_child for inline content
                let para_content = first_child_ref.first_child;
                if let Some(content_node) = para_content {
                    let content_ref = arena.get(content_node);
                    if let NodeData::Text { literal } = &content_ref.data {
                        assert_eq!(
                            literal, "Some text",
                            "Paragraph content should be 'Some text'"
                        );
                    } else {
                        panic!("Expected Text node, got {:?}", content_ref.data);
                    }
                } else {
                    panic!(
                        "Paragraph should have content in either data or first_child"
                    );
                }
            }
        }
    }
}
