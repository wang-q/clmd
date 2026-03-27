//! Block finalization for block parsing
//!
//! This module handles finalizing blocks, adding text content,
//! and post-processing after parsing is complete.

use crate::arena::{NodeId, TreeOps};
use crate::blocks::BlockParser;
use crate::inlines::parse_reference;
use crate::lexer::TAB_STOP;
use crate::node_value::NodeValue;
use crate::{NodeData, NodeType};

impl<'a> BlockParser<'a> {
    /// Add text to container
    pub(crate) fn add_text_to_container(&mut self, container: NodeId) {
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
        let line = line.trim_start_matches([' ', '\t']);

        // Now trim trailing whitespace (including newline) and hashtags
        let trimmed = line.trim_end_matches([' ', '\t', '\n', '\r']);

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
        let tip = self.tip;
        self.add_line_to_node(tip);
    }

    /// Add current line to a specific node's content
    fn add_line_to_node(&mut self, node: NodeId) {
        let mut line_content = String::new();

        // Handle partially consumed tab
        if self.partially_consumed_tab {
            let offset = self.offset;
            self.offset = offset + 1; // skip over tab
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
    pub(crate) fn close_unmatched_blocks(&mut self) {
        if !self.all_closed {
            let doc = self.doc;
            let last_matched = self.last_matched_container;
            let mut old_tip = self.old_tip;
            while old_tip != last_matched {
                let parent = self.arena.get(old_tip).parent.unwrap_or(doc);
                self.finalize_block(old_tip);
                old_tip = parent;
            }
            self.old_tip = old_tip;
            self.all_closed = true;
        }
    }

    /// Add a child to the tip
    pub(crate) fn add_child(
        &mut self,
        block_type: NodeType,
        start_column: usize,
    ) -> NodeId {
        use crate::arena::Node;

        let doc = self.doc;
        let mut tip = self.tip;

        // If tip can't accept this child, finalize it and try its parent
        while !self.can_contain(tip, block_type) {
            let parent = self.arena.get(tip).parent.unwrap_or(doc);
            self.finalize_block(tip);
            tip = parent;
        }

        let mut new_block = Node::with_value(block_type.into());
        new_block.source_pos.start_line = self.line_number as u32;
        new_block.source_pos.start_column = start_column as u32;

        let new_block_id = self.arena.alloc(new_block);
        TreeOps::append_child(self.arena, tip, new_block_id);

        // Initialize block info
        self.set_block_info(new_block_id, crate::blocks::BlockInfo::new());

        self.tip = new_block_id;
        new_block_id
    }

    /// Finalize a block
    pub(crate) fn finalize_block(&mut self, block: NodeId) {
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
                    block_mut.source_pos = crate::node::SourcePos {
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

                // block_ref is dropped here implicitly
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

    /// Remove link reference definitions from the document
    /// This processes paragraph nodes marked as empty during finalization
    pub(crate) fn remove_link_reference_definitions(&mut self) {
        let doc = self.doc;
        self.collect_and_remove_empty_paragraphs(doc);
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
}
