//! Block start detection for block parsing
//!
//! This module handles detecting and opening new block-level elements.

use crate::arena::NodeId;
use crate::blocks::BlockParser;
use crate::inlines::unescape_string;
use crate::lexer::{is_space_or_tab, CODE_INDENT};
use crate::node::{DelimType, ListType, NodeData, NodeType};

/// Result of trying to open a new block
enum BlockStartResult {
    /// Block was opened, continue processing
    Opened(NodeId),
    /// No block opened, continue with other checks
    None,
    /// Block opened and consumed the line, return immediately
    Done(NodeId),
}

impl<'a> BlockParser<'a> {
    /// Try to open new blocks
    pub(crate) fn open_new_blocks(
        &mut self,
        container: NodeId,
        all_matched: bool,
    ) -> NodeId {
        let mut current_container = container;
        let mut maybe_lazy = self.arena.get(self.tip).node_type == NodeType::Paragraph;

        loop {
            self.find_next_nonspace();
            let indented = self.indent >= CODE_INDENT;

            // Check if we're inside a leaf block that accepts lines
            if self.is_in_leaf_block(current_container) {
                return current_container;
            }

            // Try indented code block first
            if let BlockStartResult::Done(node) =
                self.try_open_indented_code_block(indented, maybe_lazy)
            {
                return node;
            }

            // Try block quote
            if let BlockStartResult::Opened(node) = self.try_open_block_quote(indented) {
                current_container = node;
                maybe_lazy = false;
                continue;
            }

            // Try ATX heading
            if let BlockStartResult::Done(node) = self.try_open_atx_heading(indented) {
                return node;
            }

            // Try fenced code block
            if let BlockStartResult::Done(node) =
                self.try_open_fenced_code_block(indented)
            {
                return node;
            }

            // Try HTML block
            if let BlockStartResult::Done(node) =
                self.try_open_html_block(indented, current_container, maybe_lazy)
            {
                return node;
            }

            // Try setext heading
            if let BlockStartResult::Done(node) =
                self.try_open_setext_heading(indented, current_container)
            {
                return node;
            }

            // Try thematic break
            if let BlockStartResult::Done(node) =
                self.try_open_thematic_break(indented, current_container, all_matched)
            {
                return node;
            }

            // Try list item
            if let BlockStartResult::Opened(node) =
                self.try_open_list_item(indented, current_container)
            {
                current_container = node;
                maybe_lazy = false;
                continue;
            }

            // No new block started
            break;
        }

        current_container
    }

    /// Check if we're inside a leaf block that doesn't allow nested blocks
    fn is_in_leaf_block(&self, container: NodeId) -> bool {
        let container_type = self.arena.get(container).node_type;
        matches!(container_type, NodeType::HtmlBlock | NodeType::CodeBlock)
    }

    /// Try to open an indented code block
    fn try_open_indented_code_block(
        &mut self,
        indented: bool,
        maybe_lazy: bool,
    ) -> BlockStartResult {
        if indented && !maybe_lazy && !self.blank {
            self.close_unmatched_blocks();
            let code_block = self.add_child(NodeType::CodeBlock, self.offset);
            self.set_fence_info(code_block, '\0', 0, 0);
            self.advance_offset(CODE_INDENT, true);
            BlockStartResult::Done(code_block)
        } else {
            BlockStartResult::None
        }
    }

    /// Try to open a block quote
    fn try_open_block_quote(&mut self, indented: bool) -> BlockStartResult {
        if indented || self.peek_next_nonspace() != Some('>') {
            return BlockStartResult::None;
        }

        self.close_unmatched_blocks();
        self.advance_next_nonspace();
        self.advance_offset(1, false);
        if self.peek_current().is_some_and(is_space_or_tab) {
            self.advance_offset(1, true);
        }
        let block_quote = self.add_child(NodeType::BlockQuote, self.next_nonspace);
        BlockStartResult::Opened(block_quote)
    }

    /// Try to open an ATX heading
    fn try_open_atx_heading(&mut self, indented: bool) -> BlockStartResult {
        if indented {
            return BlockStartResult::None;
        }

        let line = &self.current_line[self.next_nonspace..];
        let level = self.scan_atx_heading_level(line);

        if level == 0 {
            return BlockStartResult::None;
        }

        let after_hashes = &line[level..];
        if !self.is_valid_atx_heading_suffix(after_hashes) {
            return BlockStartResult::None;
        }

        self.close_unmatched_blocks();
        self.advance_next_nonspace();
        self.advance_offset(level, false);

        let content = self.extract_atx_heading_content();
        let heading = self.add_child(NodeType::Heading, self.next_nonspace);

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

        self.advance_offset(self.current_line.len() - self.offset, false);
        BlockStartResult::Done(heading)
    }

    /// Scan for ATX heading level (number of # characters)
    fn scan_atx_heading_level(&self, line: &str) -> usize {
        let mut level = 0;
        for c in line.chars() {
            if c == '#' {
                level += 1;
                if level > 6 {
                    return 0; // Invalid: more than 6 #
                }
            } else {
                break;
            }
        }
        level
    }

    /// Check if the suffix after ATX heading markers is valid
    fn is_valid_atx_heading_suffix(&self, suffix: &str) -> bool {
        suffix.is_empty()
            || suffix.starts_with(' ')
            || suffix.starts_with('\t')
            || suffix.starts_with('\n')
            || suffix.starts_with('\r')
            || suffix.starts_with('#')
    }

    /// Extract content from ATX heading, removing closing sequence
    fn extract_atx_heading_content(&self) -> String {
        let content_start = self.offset;
        let mut content = self.current_line[content_start..].to_string();

        // Remove trailing newlines
        content = content.trim_end_matches('\n').to_string();
        content = content.trim_end_matches('\r').to_string();

        // Remove closing sequence
        content = self.remove_atx_closing_sequence(content);

        // Trim leading whitespace
        content.trim_start_matches([' ', '\t']).to_string()
    }

    /// Remove ATX heading closing sequence (trailing #s)
    fn remove_atx_closing_sequence(&self, content: String) -> String {
        // Pattern 1: content is only whitespace + #s
        let trimmed_start = content.trim_start_matches([' ', '\t']);
        let trimmed_end = trimmed_start.trim_end_matches([' ', '\t']);
        if trimmed_end.chars().all(|c| c == '#') && !trimmed_end.is_empty() {
            return String::new();
        }

        // Pattern 2: closing sequence at end (preceded by whitespace)
        if let Some(start) = self.find_closing_hash_sequence(&content) {
            if start > 0 {
                let before_hash = &content[..start];
                if before_hash.ends_with(' ') || before_hash.ends_with('\t') {
                    return before_hash
                        .trim_end_matches([' ', '\t'])
                        .to_string();
                }
            }
        }

        content
    }

    /// Find the start of closing hash sequence in ATX heading
    fn find_closing_hash_sequence(&self, content: &str) -> Option<usize> {
        let mut hash_start = None;
        let mut in_hashes = false;

        for (i, c) in content.char_indices().rev() {
            if c == '#' {
                if !in_hashes {
                    in_hashes = true;
                }
            } else if c == ' ' || c == '\t' {
                if in_hashes {
                    hash_start = Some(i + 1);
                    break;
                }
            } else {
                break;
            }
        }

        // Entire content is hashes
        if in_hashes && hash_start.is_none() {
            hash_start = Some(0);
        }

        hash_start
    }

    /// Try to open a fenced code block
    fn try_open_fenced_code_block(&mut self, indented: bool) -> BlockStartResult {
        if indented {
            return BlockStartResult::None;
        }

        let line = &self.current_line[self.next_nonspace..];
        let first_char = match line.chars().next() {
            Some(c) if c == '`' || c == '~' => c,
            _ => return BlockStartResult::None,
        };

        let fence_length = self.scan_fence_length(line, first_char);
        if fence_length < 3 {
            return BlockStartResult::None;
        }

        let rest = &line[fence_length..];
        if first_char == '`' && rest.contains('`') {
            return BlockStartResult::None;
        }

        let info = unescape_string(rest.trim());
        self.close_unmatched_blocks();
        let code_block = self.add_child(NodeType::CodeBlock, self.next_nonspace);

        {
            let code_mut = self.arena.get_mut(code_block);
            if let NodeData::CodeBlock {
                info: ref mut i, ..
            } = code_mut.data
            {
                *i = info;
            }
        }

        self.set_fence_info(code_block, first_char, fence_length, self.next_nonspace);
        self.advance_next_nonspace();
        self.advance_offset(fence_length, false);

        BlockStartResult::Done(code_block)
    }

    /// Scan fence length (consecutive fence characters)
    fn scan_fence_length(&self, line: &str, fence_char: char) -> usize {
        line.chars().take_while(|&c| c == fence_char).count()
    }

    /// Try to open an HTML block
    fn try_open_html_block(
        &mut self,
        indented: bool,
        container: NodeId,
        maybe_lazy: bool,
    ) -> BlockStartResult {
        if indented || self.peek_next_nonspace() != Some('<') {
            return BlockStartResult::None;
        }

        // Don't start a new HTML block if we're already inside one
        let in_html_block = self.arena.get(container).node_type == NodeType::HtmlBlock;
        if in_html_block {
            return BlockStartResult::None;
        }

        let line = &self.current_line[self.next_nonspace..];
        if let Some(block_type) = self.scan_html_block_start(line, container, maybe_lazy)
        {
            self.close_unmatched_blocks();
            let html_block = self.add_child(NodeType::HtmlBlock, self.offset);
            self.set_html_block_type(html_block, block_type);
            BlockStartResult::Done(html_block)
        } else {
            BlockStartResult::None
        }
    }

    /// Try to open a setext heading
    fn try_open_setext_heading(
        &mut self,
        indented: bool,
        container: NodeId,
    ) -> BlockStartResult {
        if indented {
            return BlockStartResult::None;
        }

        if self.arena.get(container).node_type != NodeType::Paragraph {
            return BlockStartResult::None;
        }

        let line = &self.current_line[self.next_nonspace..];
        let level = match self.scan_setext_heading_line(line) {
            Some(l) => l,
            None => return BlockStartResult::None,
        };

        let content = self.get_string_content(container);
        let remaining_content = self.process_setext_content(content);

        if remaining_content.is_empty() {
            return BlockStartResult::None;
        }

        self.close_unmatched_blocks();
        {
            let container_mut = self.arena.get_mut(container);
            container_mut.node_type = NodeType::Heading;
            container_mut.data = NodeData::Heading {
                level,
                content: remaining_content.to_string(),
            };
        }
        self.set_setext(container, true);
        self.advance_offset(self.current_line.len() - self.offset, false);

        BlockStartResult::Done(container)
    }

    /// Process setext heading content, removing reference definitions
    fn process_setext_content(&mut self, content: String) -> String {
        let mut processed_content = content;

        while !processed_content.is_empty() {
            let trimmed = processed_content.trim_start();
            if !trimmed.starts_with('[') {
                break;
            }

            let consumed =
                crate::inlines::parse_reference(&processed_content, &mut self.refmap);
            if consumed == 0 {
                break;
            }

            processed_content = processed_content[consumed..].to_string();
            processed_content = processed_content.trim_start().to_string();
        }

        processed_content.trim().to_string()
    }

    /// Try to open a thematic break
    fn try_open_thematic_break(
        &mut self,
        indented: bool,
        container: NodeId,
        all_matched: bool,
    ) -> BlockStartResult {
        if indented {
            return BlockStartResult::None;
        }

        if self.arena.get(container).node_type == NodeType::Paragraph && !all_matched {
            return BlockStartResult::None;
        }

        let line = &self.current_line[self.next_nonspace..];
        if !self.scan_thematic_break(line) {
            return BlockStartResult::None;
        }

        self.close_unmatched_blocks();
        let thematic_break = self.add_child(NodeType::ThematicBreak, self.next_nonspace);
        self.advance_offset(self.current_line.len() - self.offset, false);

        BlockStartResult::Done(thematic_break)
    }

    /// Try to open a list item
    fn try_open_list_item(
        &mut self,
        indented: bool,
        container: NodeId,
    ) -> BlockStartResult {
        if indented && self.arena.get(container).node_type != NodeType::List {
            return BlockStartResult::None;
        }

        if self.indent >= 4 {
            return BlockStartResult::None;
        }

        let marker_result = self.parse_list_marker(container);
        let (list_type, delim, start, marker_offset, padding, bullet_char) =
            match marker_result {
                Some(r) => r,
                None => return BlockStartResult::None,
            };

        self.close_unmatched_blocks();

        // Check if we can continue an existing list
        let can_continue_list = self.arena.get(container).node_type == NodeType::List
            && self.lists_match(container, list_type, delim, start, bullet_char);

        let _list_container = if can_continue_list {
            container
        } else {
            let new_list = self.add_child(NodeType::List, self.next_nonspace);
            {
                let list_mut = self.arena.get_mut(new_list);
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
            new_list
        };

        // Add list item
        let item = self.add_child(NodeType::Item, self.next_nonspace);
        self.set_list_data(item, marker_offset, padding);

        BlockStartResult::Opened(item)
    }

    // HTML block detection methods remain unchanged...
    /// Scan for HTML block start
    fn scan_html_block_start(
        &self,
        line: &str,
        container: NodeId,
        maybe_lazy: bool,
    ) -> Option<u8> {
        // Type 1: <script, <pre, <textarea, <style
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

        // Type 4: <! followed by uppercase letter
        if line.starts_with("<!") && line.len() > 2 {
            if let Some(third_char) = line.chars().nth(2) {
                if third_char.is_ascii_alphabetic() {
                    return Some(4);
                }
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

        // Type 7: Complete HTML tag
        if line.starts_with('<')
            && !maybe_lazy
            && self.arena.get(container).node_type != NodeType::Paragraph
            && self.is_valid_html_tag_type7(line)
        {
            return Some(7);
        }

        None
    }

    /// Match HTML block type 1
    fn match_html_block_type1(&self, line: &str) -> bool {
        let tags = ["script", "pre", "textarea", "style"];
        for tag in &tags {
            if line.len() < tag.len() + 1 {
                continue;
            }
            if line[1..].to_lowercase().starts_with(tag) {
                let after = &line[1 + tag.len()..];
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

    /// Match HTML block type 6
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

        let type1_end_tags = ["script", "pre", "textarea", "style"];
        let line_lower = line.to_lowercase();

        for tag in &tags {
            // Check opening tag
            if line_lower.len() > tag.len() && line_lower[1..].starts_with(tag) {
                let after = &line_lower[1 + tag.len()..];
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

            // Check closing tag
            if line_lower.len() >= 2 + tag.len()
                && line_lower[2..].starts_with(tag)
                && !type1_end_tags.contains(tag)
            {
                let after = &line_lower[2 + tag.len()..];
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
        }

        false
    }

    /// Check if a line is a valid HTML tag for type 7
    fn is_valid_html_tag_type7(&self, line: &str) -> bool {
        if !line.starts_with('<') {
            return false;
        }

        if line.starts_with("</") {
            return self.is_valid_close_tag_type7(line);
        }

        self.is_valid_open_tag_type7(line)
    }

    /// Check if line is a valid open tag for type 7
    fn is_valid_open_tag_type7(&self, line: &str) -> bool {
        let mut chars = line.chars().peekable();

        if chars.next() != Some('<') {
            return false;
        }

        match chars.next() {
            Some(c) if c.is_ascii_alphabetic() => {}
            _ => return false,
        };

        loop {
            match chars.peek() {
                Some(&c) if c.is_ascii_alphanumeric() || c == '-' => {
                    chars.next();
                }
                _ => break,
            }
        }

        self.parse_tag_attributes_and_close(&mut chars)
    }

    /// Check if line is a valid close tag for type 7
    fn is_valid_close_tag_type7(&self, line: &str) -> bool {
        let mut chars = line.chars().peekable();

        if chars.next() != Some('<') || chars.next() != Some('/') {
            return false;
        }

        match chars.next() {
            Some(c) if c.is_ascii_alphabetic() => {}
            _ => return false,
        };

        loop {
            match chars.peek() {
                Some(&c) if c.is_ascii_alphanumeric() || c == '-' => {
                    chars.next();
                }
                _ => break,
            }
        }

        while let Some(&c) = chars.peek() {
            if c.is_whitespace() {
                chars.next();
            } else {
                break;
            }
        }

        match chars.next() {
            Some('>') => chars.all(|c| c.is_whitespace()),
            _ => false,
        }
    }

    /// Parse tag attributes and closing >
    fn parse_tag_attributes_and_close(
        &self,
        chars: &mut std::iter::Peekable<std::str::Chars>,
    ) -> bool {
        let mut seen_whitespace = true;

        loop {
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
                    return chars.all(|c| c.is_whitespace());
                }
                Some(&'/') => {
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
                    if !seen_whitespace {
                        return false;
                    }
                    seen_whitespace = false;

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

                    if let Some(&'=') = chars.peek() {
                        chars.next();
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
                            _ => loop {
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
                            },
                        }
                    }
                }
                _ => return false,
            }
        }
    }

    /// Scan for setext heading line
    fn scan_setext_heading_line(&self, line: &str) -> Option<u32> {
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            return None;
        }

        let first_char = trimmed.chars().next()?;
        if first_char != '=' && first_char != '-' {
            return None;
        }

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
        // Try bullet list marker first
        if let Some(result) = self.parse_bullet_marker(container) {
            return Some(result);
        }

        // Try ordered list marker
        self.parse_ordered_marker(container)
    }

    /// Parse bullet list marker
    fn parse_bullet_marker(
        &mut self,
        container: NodeId,
    ) -> Option<(ListType, DelimType, u32, usize, usize, char)> {
        let rest = &self.current_line[self.next_nonspace..];
        let first_char = rest.chars().next()?;
        if !"*+-".contains(first_char) {
            return None;
        }

        let after_marker = &rest[1..];
        if !after_marker.is_empty()
            && !after_marker.starts_with(' ')
            && !after_marker.starts_with('\t')
            && !after_marker.starts_with('\n')
        {
            return None;
        }

        // Check for non-blank content if interrupting paragraph
        if self.arena.get(container).node_type == NodeType::Paragraph {
            let content_after = after_marker.trim_start();
            if content_after.is_empty() || content_after.starts_with('\n') {
                return None;
            }
        }

        self.advance_next_nonspace();
        self.advance_offset(1, true);

        let spaces_start_col = self.column;
        let spaces_start_offset = self.offset;

        while self.column - spaces_start_col < 5
            && self.peek_current().is_some_and(is_space_or_tab)
        {
            self.advance_offset(1, true);
        }

        let blank_item =
            self.peek_current().is_none() || self.peek_current() == Some('\n');
        let spaces_after_marker = self.column - spaces_start_col;

        let padding = if !(1..5).contains(&spaces_after_marker) || blank_item {
            self.column = spaces_start_col;
            self.offset = spaces_start_offset;
            if self.peek_current().is_some_and(is_space_or_tab) {
                self.advance_offset(1, true);
            }
            2
        } else {
            1 + spaces_after_marker
        };

        Some((
            ListType::Bullet,
            DelimType::None,
            0,
            self.indent,
            padding,
            first_char,
        ))
    }

    /// Parse ordered list marker
    fn parse_ordered_marker(
        &mut self,
        container: NodeId,
    ) -> Option<(ListType, DelimType, u32, usize, usize, char)> {
        let rest = &self.current_line[self.next_nonspace..];
        let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        if digits.is_empty() || digits.len() > 9 {
            return None;
        }

        let start: u32 = digits.parse().ok()?;
        let after_digits = &rest[digits.len()..];

        let delim_char = after_digits.chars().next()?;
        if delim_char != '.' && delim_char != ')' {
            return None;
        }

        let after_delim = &after_digits[1..];
        if !after_delim.is_empty()
            && !after_delim.starts_with(' ')
            && !after_delim.starts_with('\t')
            && !after_delim.starts_with('\n')
        {
            return None;
        }

        // If interrupting paragraph, start must be 1
        if self.arena.get(container).node_type == NodeType::Paragraph && start != 1 {
            return None;
        }

        // Check for non-blank content if interrupting paragraph
        if self.arena.get(container).node_type == NodeType::Paragraph {
            let content_after = after_delim.trim_start();
            if content_after.is_empty() || content_after.starts_with('\n') {
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

        while self.column - spaces_start_col < 5
            && self.peek_current().is_some_and(is_space_or_tab)
        {
            self.advance_offset(1, true);
        }

        let blank_item =
            self.peek_current().is_none() || self.peek_current() == Some('\n');
        let spaces_after_marker = self.column - spaces_start_col;

        let padding = if !(1..5).contains(&spaces_after_marker) || blank_item {
            self.column = spaces_start_col;
            self.offset = spaces_start_offset;
            if self.peek_current().is_some_and(is_space_or_tab) {
                self.advance_offset(1, true);
            }
            digits.len() + 2
        } else {
            digits.len() + 1 + spaces_after_marker
        };

        Some((ListType::Ordered, delim, start, self.indent, padding, '\0'))
    }

    /// Lists match check
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
            if list_type == ListType::Bullet && *lt == ListType::Bullet {
                *bc == bullet_char
            } else {
                *lt == list_type && *d == delim
            }
        } else {
            false
        }
    }
}
