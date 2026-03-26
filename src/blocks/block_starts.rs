//! Block start detection for block parsing
//!
//! This module handles detecting and opening new block-level elements.

use crate::arena::NodeId;
use crate::blocks::BlockParser;
use crate::inlines::unescape_string;
use crate::lexer::{is_space_or_tab, CODE_INDENT};
use crate::node::{DelimType, ListType, NodeData, NodeType};

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
                if self.peek_current().is_some_and(is_space_or_tab) {
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
                        let trimmed_start = content.trim_start_matches([' ', '\t']);
                        let trimmed_end = trimmed_start.trim_end_matches([' ', '\t']);
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
                        content = content.trim_start_matches([' ', '\t']).to_string();

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
                        let consumed = crate::inlines::parse_reference(
                            &processed_content,
                            &mut self.refmap,
                        );

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
        if line.starts_with('<')
            && !maybe_lazy
            && self.arena.get(container).node_type != NodeType::Paragraph
            && self.is_valid_html_tag_type7(line)
        {
            return Some(7);
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
            if line_lower.len() > tag.len() && line_lower[1..].starts_with(tag) {
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
                    if !type1_end_tags.contains(&tag) {
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
                        && self.peek_current().is_some_and(is_space_or_tab)
                    {
                        self.advance_offset(1, true);
                    }

                    let blank_item = self.peek_current().is_none()
                        || self.peek_current() == Some('\n');
                    let spaces_after_marker = self.column - spaces_start_col;

                    let padding;
                    if !(1..5).contains(&spaces_after_marker) || blank_item {
                        padding = 2; // marker length (1) + 1 space
                        self.column = spaces_start_col;
                        self.offset = spaces_start_offset;
                        if self.peek_current().is_some_and(is_space_or_tab) {
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
                            && self.peek_current().is_some_and(is_space_or_tab)
                        {
                            self.advance_offset(1, true);
                        }

                        let blank_item = self.peek_current().is_none()
                            || self.peek_current() == Some('\n');
                        let spaces_after_marker = self.column - spaces_start_col;

                        let padding;
                        if !(1..5).contains(&spaces_after_marker) || blank_item {
                            padding = digits.len() + 2; // marker length + 1 space
                            self.column = spaces_start_col;
                            self.offset = spaces_start_offset;
                            if self.peek_current().is_some_and(is_space_or_tab) {
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
}
