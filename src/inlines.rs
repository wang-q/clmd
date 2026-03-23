/// Inline parsing for CommonMark documents
///
/// This module implements the inline parsing algorithm based on the CommonMark spec.
/// It processes the content of leaf blocks (paragraphs, headings, etc.) to produce
/// inline elements like emphasis, links, code, etc.

use crate::node::{append_child, Node, NodeData, NodeType};
use std::cell::RefCell;
use std::rc::Rc;

/// Maximum number of backticks to track
const MAX_BACKTICKS: usize = 1000;

/// Subject represents the string being parsed and tracks position
pub struct Subject {
    /// The input string
    pub input: String,
    /// Current position in the input
    pub pos: usize,
    /// Line number (for source positions)
    pub line: usize,
    /// Column offset (for source positions)
    pub column_offset: usize,
    /// Block offset (for source positions)
    pub block_offset: usize,
    /// Stack of delimiters for emphasis/strong
    pub delimiters: Option<Box<Delimiter>>,
    /// Stack of brackets for links/images
    pub brackets: Option<Box<Bracket>>,
    /// Position of backtick sequences
    pub backticks: Vec<usize>,
    /// Whether we've scanned for backticks
    pub scanned_for_backticks: bool,
    /// Whether there are no link openers
    pub no_link_openers: bool,
    /// Reference map for link references
    pub refmap: std::collections::HashMap<String, (String, String)>,
}

/// Delimiter struct for tracking emphasis markers
#[derive(Clone)]
pub struct Delimiter {
    /// Previous delimiter in stack
    pub previous: Option<Box<Delimiter>>,
    /// The inline text node containing the delimiter
    pub inl_text: Rc<RefCell<Node>>,
    /// Position in the subject
    pub position: usize,
    /// Number of delimiter characters
    pub num_delims: usize,
    /// Original number of delimiter characters
    pub orig_delims: usize,
    /// The delimiter character (* or _)
    pub delim_char: char,
    /// Whether this can open emphasis
    pub can_open: bool,
    /// Whether this can close emphasis
    pub can_close: bool,
}

/// Bracket struct for tracking link/image brackets
#[derive(Clone)]
pub struct Bracket {
    /// Previous bracket in stack
    pub previous: Option<Box<Bracket>>,
    /// The inline text node containing the bracket
    pub inl_text: Rc<RefCell<Node>>,
    /// Position in the subject
    pub position: usize,
    /// Whether this is an image (![)
    pub image: bool,
    /// Whether this bracket is still active
    pub active: bool,
    /// Whether there was a bracket after this one
    pub bracket_after: bool,
    /// Previous delimiter in stack (for emphasis processing)
    pub previous_delimiter: Option<Box<Delimiter>>,
}

impl Subject {
    /// Create a new subject from a string
    pub fn new(input: &str, line: usize, block_offset: usize) -> Self {
        Subject {
            input: input.to_string(),
            pos: 0,
            line,
            column_offset: 0,
            block_offset,
            delimiters: None,
            brackets: None,
            backticks: vec![0; MAX_BACKTICKS + 1],
            scanned_for_backticks: false,
            no_link_openers: false,
            refmap: std::collections::HashMap::new(),
        }
    }

    /// Create a new subject with a reference map
    pub fn with_refmap(input: &str, line: usize, block_offset: usize, refmap: std::collections::HashMap<String, (String, String)>) -> Self {
        Subject {
            input: input.to_string(),
            pos: 0,
            line,
            column_offset: 0,
            block_offset,
            delimiters: None,
            brackets: None,
            backticks: vec![0; MAX_BACKTICKS + 1],
            scanned_for_backticks: false,
            no_link_openers: false,
            refmap,
        }
    }

    /// Peek at the current character without advancing
    pub fn peek(&self) -> Option<char> {
        self.input.chars().nth(self.pos)
    }

    /// Peek at the next character code
    pub fn peek_char_code(&self) -> i32 {
        if self.pos < self.input.len() {
            self.input.as_bytes()[self.pos] as i32
        } else {
            -1
        }
    }

    /// Advance position by one character
    pub fn advance(&mut self) {
        if self.pos < self.input.len() {
            self.pos += 1;
        }
    }

    /// Check if we're at the end of the input
    pub fn end(&self) -> bool {
        self.pos >= self.input.len()
    }

    /// Parse the subject and append inline children to the given node
    pub fn parse_inlines(&mut self, parent: &Rc<RefCell<Node>>) {
        while !self.end() {
            if !self.parse_inline(parent) {
                // If no inline was parsed, advance to avoid infinite loop
                self.advance();
            }
        }

        // Process any remaining emphasis
        self.process_emphasis(None);
    }

    /// Parse a single inline element
    fn parse_inline(&mut self, parent: &Rc<RefCell<Node>>) -> bool {
        let c = match self.peek() {
            Some(c) => c,
            None => return false,
        };

        match c {
            '`' => self.parse_backticks(parent),
            '\\' => self.parse_backslash(parent),
            '&' => self.parse_entity(parent),
            '<' => self.parse_lt(parent),
            '*' | '_' => self.handle_delim(c, parent),
            '[' => self.parse_open_bracket(parent),
            ']' => self.parse_close_bracket(parent),
            '!' => self.parse_bang(parent),
            _ => self.parse_string(parent),
        }
    }

    /// Parse backtick-delimited code span
    fn parse_backticks(&mut self, parent: &Rc<RefCell<Node>>) -> bool {
        let start_pos = self.pos;
        let mut ticks = String::new();

        // Count opening backticks
        while self.peek() == Some('`') {
            ticks.push('`');
            self.advance();
        }

        let tick_len = ticks.len();
        let after_open_ticks = self.pos;

        // Look for closing backticks
        loop {
            if self.end() {
                break;
            }

            if self.peek() == Some('`') {
                let mut close_ticks = String::new();
                let close_start = self.pos;

                while self.peek() == Some('`') {
                    close_ticks.push('`');
                    self.advance();
                }

                if close_ticks.len() == tick_len {
                    // Found matching close
                    let content = self.input[after_open_ticks..close_start].to_string();
                    let content = content.replace('\n', " ");

                    // Trim single leading/trailing space if both exist
                    let content = if content.len() >= 2
                        && content.starts_with(' ')
                        && content.ends_with(' ')
                        && content.trim() != ""
                    {
                        content[1..content.len() - 1].to_string()
                    } else {
                        content
                    };

                    let code_node = Rc::new(RefCell::new(Node::new(NodeType::Code)));
                    {
                        let mut code_mut = code_node.borrow_mut();
                        if let NodeData::Code { ref mut literal, .. } = code_mut.data {
                            *literal = content;
                        }
                    }
                    append_child(parent, code_node);
                    return true;
                }
            } else {
                self.advance();
            }
        }

        // No matching close found, treat as literal
        self.pos = after_open_ticks;
        self.append_text(parent, &ticks);
        true
    }

    /// Parse backslash escape or hard line break
    fn parse_backslash(&mut self, parent: &Rc<RefCell<Node>>) -> bool {
        self.advance(); // skip backslash

        if self.peek() == Some('\n') {
            // Hard line break
            self.advance();
            let line_break = Rc::new(RefCell::new(Node::new(NodeType::LineBreak)));
            append_child(parent, line_break);
        } else if let Some(c) = self.peek() {
            if is_escapable(c) {
                self.append_text(parent, &c.to_string());
                self.advance();
            } else {
                self.append_text(parent, "\\");
            }
        } else {
            self.append_text(parent, "\\");
        }

        true
    }

    /// Parse entity or numeric character reference
    fn parse_entity(&mut self, parent: &Rc<RefCell<Node>>) -> bool {
        if let Some((c, len)) = parse_entity_char(&self.input[self.pos..]) {
            self.append_text(parent, &c.to_string());
            self.pos += len;
            true
        } else {
            // Not a valid entity, treat & as literal
            self.parse_string(parent)
        }
    }

    /// Parse less-than sign (could be autolink or HTML tag)
    fn parse_lt(&mut self, parent: &Rc<RefCell<Node>>) -> bool {
        // Try autolink first
        if self.parse_autolink(parent) {
            return true;
        }

        // Try HTML tag
        if self.parse_html_tag(parent) {
            return true;
        }

        // Just a literal <
        self.parse_string(parent)
    }

    /// Parse autolink (URL or email in angle brackets)
    fn parse_autolink(&mut self, parent: &Rc<RefCell<Node>>) -> bool {
        let remaining = &self.input[self.pos..];

        // Try email autolink first
        if let Some((email, len)) = match_email_autolink(remaining) {
            let link_node = Rc::new(RefCell::new(Node::new(NodeType::Link)));
            {
                let mut link_mut = link_node.borrow_mut();
                if let NodeData::Link { ref mut url, ref mut title } = link_mut.data {
                    *url = format!("mailto:{}", email);
                    *title = String::new();
                }
            }

            // Add text content
            let text_node = Rc::new(RefCell::new(Node::new(NodeType::Text)));
            {
                let mut text_mut = text_node.borrow_mut();
                if let NodeData::Text { ref mut literal } = text_mut.data {
                    *literal = email;
                }
            }
            append_child(&link_node, text_node);
            append_child(parent, link_node);

            self.pos += len;
            return true;
        }

        // Try URL autolink
        if let Some((url, len)) = match_url_autolink(remaining) {
            let link_node = Rc::new(RefCell::new(Node::new(NodeType::Link)));
            {
                let mut link_mut = link_node.borrow_mut();
                if let NodeData::Link { url: ref mut link_url, title: ref mut link_title } = link_mut.data {
                    *link_url = url.clone();
                    *link_title = String::new();
                }
            }

            // Add text content
            let text_node = Rc::new(RefCell::new(Node::new(NodeType::Text)));
            {
                let mut text_mut = text_node.borrow_mut();
                if let NodeData::Text { ref mut literal } = text_mut.data {
                    *literal = url;
                }
            }
            append_child(&link_node, text_node);
            append_child(parent, link_node);

            self.pos += len;
            return true;
        }

        false
    }

    /// Parse raw HTML tag
    fn parse_html_tag(&mut self, _parent: &Rc<RefCell<Node>>) -> bool {
        // Simplified HTML tag detection
        false
    }

    /// Handle delimiter character (* or _)
    fn handle_delim(&mut self, c: char, parent: &Rc<RefCell<Node>>) -> bool {
        let start_pos = self.pos;
        let res = self.scan_delims(c);

        if res.num_delims == 0 {
            return false;
        }

        // Add delimiter text
        let delim_text: String = std::iter::repeat(c).take(res.num_delims).collect();
        let text_node = self.append_text(parent, &delim_text);

        // Add to delimiter stack if it can open or close
        if res.can_open || res.can_close {
            let delim = Box::new(Delimiter {
                previous: self.delimiters.take(),
                inl_text: text_node.clone(),
                position: start_pos,
                num_delims: res.num_delims,
                orig_delims: res.num_delims,
                delim_char: c,
                can_open: res.can_open,
                can_close: res.can_close,
            });

            self.delimiters = Some(delim);
        }

        true
    }

    /// Scan delimiter sequence and determine if it can open/close
    fn scan_delims(&mut self, c: char) -> DelimScanResult {
        let start_pos = self.pos;
        let mut num_delims = 0;

        // Count delimiters
        while self.peek() == Some(c) {
            num_delims += 1;
            self.advance();
        }

        if num_delims == 0 {
            return DelimScanResult {
                num_delims: 0,
                can_open: false,
                can_close: false,
            };
        }

        // Determine char before and after
        let char_before = if start_pos == 0 {
            '\n'
        } else {
            self.input.chars().nth(start_pos - 1).unwrap_or('\n')
        };

        let char_after = self.peek().unwrap_or('\n');

        let before_is_whitespace = char_before.is_whitespace();
        let before_is_punctuation = is_punctuation(char_before);
        let after_is_whitespace = char_after.is_whitespace();
        let after_is_punctuation = is_punctuation(char_after);

        let left_flanking = !after_is_whitespace
            && (!after_is_punctuation || before_is_whitespace || before_is_punctuation);
        let right_flanking = !before_is_whitespace
            && (!before_is_punctuation || after_is_whitespace || after_is_punctuation);

        let (can_open, can_close) = if c == '_' {
            (
                left_flanking && (!right_flanking || before_is_punctuation),
                right_flanking && (!left_flanking || after_is_punctuation),
            )
        } else {
            (left_flanking, right_flanking)
        };

        // Reset position for text extraction
        self.pos = start_pos + num_delims;

        DelimScanResult {
            num_delims,
            can_open,
            can_close,
        }
    }

    /// Process emphasis delimiters
    fn process_emphasis(&mut self, stack_bottom: Option<*const Delimiter>) {
        // Find first closer above stack_bottom
        let mut closer = self.delimiters.clone();

        // Move to the first delimiter above stack_bottom
        while let Some(ref c) = closer {
            if let Some(sb) = stack_bottom {
                let c_ptr: *const Delimiter = c.as_ref();
                if c_ptr == sb {
                    break;
                }
            }
            if c.previous.is_none() {
                break;
            }
            closer = c.previous.clone();
        }

        // Process closers from top of stack down
        while let Some(ref mut closer_box) = closer {
            if !closer_box.can_close {
                closer = closer_box.previous.clone();
                continue;
            }

            let closer_char = closer_box.delim_char;
            let mut opener_found = false;
            let mut opener: Option<Box<Delimiter>> = None;

            // Look back for matching opener
            let mut search = closer_box.previous.clone();
            while let Some(ref s) = search {
                if let Some(sb) = stack_bottom {
                    let s_ptr: *const Delimiter = s.as_ref();
                    if s_ptr == sb {
                        break;
                    }
                }

                // Check for odd match rule
                let odd_match = (closer_box.can_open || s.can_close)
                    && closer_box.orig_delims % 3 != 0
                    && (s.orig_delims + closer_box.orig_delims) % 3 == 0;

                if s.delim_char == closer_char && s.can_open && !odd_match {
                    opener_found = true;
                    opener = Some(s.clone());
                    break;
                }

                search = s.previous.clone();
            }

            if opener_found {
                if let Some(ref mut op) = opener {
                    // Calculate number of delimiters to use
                    let use_delims = if closer_box.num_delims >= 2 && op.num_delims >= 2 {
                        2
                    } else {
                        1
                    };

                    // Create emphasis or strong node
                    let emph_type = if use_delims == 1 {
                        NodeType::Emph
                    } else {
                        NodeType::Strong
                    };

                    let emph_node = Rc::new(RefCell::new(Node::new(emph_type)));

                    // Move content between opener and closer into emph node
                    let opener_inl = op.inl_text.clone();
                    let closer_inl = closer_box.inl_text.clone();

                    // Collect nodes to move
                    let mut nodes_to_move: Vec<Rc<RefCell<Node>>> = Vec::new();
                    {
                        let opener_ref = opener_inl.borrow();
                        let current_opt = opener_ref.next.borrow().clone();

                        let mut current_ptr = current_opt;
                        while let Some(curr) = current_ptr {
                            if Rc::ptr_eq(&curr, &closer_inl) {
                                break;
                            }
                            let next_opt = curr.borrow().next.borrow().clone();
                            nodes_to_move.push(curr);
                            current_ptr = next_opt;
                        }
                    }

                    // Unlink nodes from parent and add to emph
                    for node in nodes_to_move {
                        crate::node::unlink(&node);
                        append_child(&emph_node, node);
                    }

                    // Insert emph node after opener
                    crate::node::insert_after(&opener_inl, emph_node);

                    // Update delimiter counts
                    op.num_delims -= use_delims;
                    closer_box.num_delims -= use_delims;

                    // Remove used delimiters from text nodes
                    {
                        let mut op_mut = op.inl_text.borrow_mut();
                        if let NodeData::Text { ref mut literal, .. } = op_mut.data {
                            let len = literal.len();
                            if len >= use_delims {
                                *literal = literal[..len - use_delims].to_string();
                            }
                        }
                    }
                    {
                        let mut closer_mut = closer_box.inl_text.borrow_mut();
                        if let NodeData::Text { ref mut literal, .. } = closer_mut.data {
                            let len = literal.len();
                            if len >= use_delims {
                                *literal = literal[..len - use_delims].to_string();
                            }
                        }
                    }

                    // Remove delimiter entries if no delims left
                    if op.num_delims == 0 {
                        crate::node::unlink(&op.inl_text);
                        self.remove_delimiter(op);
                    }

                    if closer_box.num_delims == 0 {
                        crate::node::unlink(&closer_box.inl_text);
                        let next_closer = closer_box.previous.clone();
                        self.remove_delimiter(closer_box);
                        closer = next_closer;
                        continue;
                    }
                }
            }

            closer = closer_box.previous.clone();
        }
    }

    /// Remove a delimiter from the stack
    fn remove_delimiter(&mut self, delim: &Delimiter) {
        // This is a simplified removal - in full implementation we'd update links
        // For now, we just leave it in place but mark it as processed
    }

    /// Parse open bracket (start of link or image)
    fn parse_open_bracket(&mut self, parent: &Rc<RefCell<Node>>) -> bool {
        self.advance(); // skip [

        let text_node = self.append_text(parent, "[");

        // Add to bracket stack
        let bracket = Box::new(Bracket {
            previous: self.brackets.take(),
            inl_text: text_node.clone(),
            position: self.pos - 1,
            image: false,
            active: true,
            bracket_after: false,
            previous_delimiter: self.delimiters.clone(),
        });

        self.brackets = Some(bracket);
        self.no_link_openers = false;

        true
    }

    /// Parse close bracket (end of link or image)
    fn parse_close_bracket(&mut self, parent: &Rc<RefCell<Node>>) -> bool {
        self.advance(); // skip ]

        // Get the opener from bracket stack
        let opener = match self.brackets.take() {
            Some(b) => b,
            None => {
                // No matching opener, just add as text
                self.append_text(parent, "]");
                return true;
            }
        };

        if !opener.active {
            // Opener is not active, just add as text
            self.append_text(parent, "]");
            self.brackets = opener.previous;
            return true;
        }

        let is_image = opener.image;
        let start_pos = self.pos;
        let mut matched = false;
        let mut dest: Option<String> = None;
        let mut title: Option<String> = None;
        let mut reflabel: Option<String> = None;

        // Try inline link: [text](url "title")
        if self.peek() == Some('(') {
            self.advance(); // skip (
            self.skip_spaces_and_newlines();

            // Parse link destination
            if let Some(d) = self.parse_link_destination() {
                dest = Some(d);
                self.skip_spaces_and_newlines();

                // Try to parse title
                if self.peek() == Some('"') || self.peek() == Some('\'') || self.peek() == Some('(') {
                    title = self.parse_link_title();
                    self.skip_spaces_and_newlines();
                }

                if self.peek() == Some(')') {
                    self.advance(); // skip )
                    matched = true;
                }
            }
        }

        // Try reference link [text][label] or [text][]
        if !matched {
            let before_label = self.pos;
            let label_len = self.parse_link_label();

            if label_len > 2 {
                // Collapsed reference link [text][] or full [text][label]
                let label = self.input[before_label..before_label + label_len].to_string();
                reflabel = Some(label);
            } else if label_len == 0 {
                // Shortcut reference link [text]
                self.pos = before_label; // rewind
                // Use the text between brackets as label
                if !opener.bracket_after {
                    reflabel = Some(self.input[opener.position..start_pos - 1].to_string());
                }
            }

            if let Some(label) = reflabel {
                // Normalize the label and look up in refmap
                let norm_label = normalize_reference(&label);
                if let Some((dest_url, dest_title)) = self.lookup_reference(&norm_label) {
                    dest = Some(dest_url);
                    title = Some(dest_title);
                    matched = true;
                }
            }
        }

        if matched {
            // Create link or image node
            let node_type = if is_image { NodeType::Image } else { NodeType::Link };
            let link_node = Rc::new(RefCell::new(Node::new(node_type)));

            {
                let mut link_mut = link_node.borrow_mut();
                match &mut link_mut.data {
                    NodeData::Link { url, title: link_title } => {
                        *url = dest.unwrap_or_default();
                        *link_title = title.unwrap_or_default();
                    }
                    NodeData::Image { url, title: img_title } => {
                        *url = dest.unwrap_or_default();
                        *img_title = title.unwrap_or_default();
                    }
                    _ => {}
                }
            }

            // Move content between opener and closer into link node
            let opener_inl = opener.inl_text.clone();
            let mut nodes_to_move: Vec<Rc<RefCell<Node>>> = Vec::new();

            {
                let opener_ref = opener_inl.borrow();
                let current_opt = opener_ref.next.borrow().clone();

                let mut current_ptr = current_opt;
                while let Some(curr) = current_ptr {
                    let next_opt = curr.borrow().next.borrow().clone();
                    nodes_to_move.push(curr);
                    current_ptr = next_opt;
                }
            }

            // Unlink nodes from parent and add to link
            for node in nodes_to_move {
                crate::node::unlink(&node);
                append_child(&link_node, node);
            }

            // Insert link node after opener
            crate::node::insert_after(&opener_inl, link_node);

            // Unlink the opener text node
            crate::node::unlink(&opener_inl);

            // Process emphasis with opener's previous delimiter
            self.process_emphasis(opener.previous_delimiter.as_ref().map(|d| d.as_ref() as *const _));

            // For links (not images), deactivate previous link openers
            if !is_image {
                let mut current = self.brackets.clone();
                while let Some(ref b) = current {
                    if !b.image {
                        // Mark as inactive - we'd need to modify Bracket to have a way to do this
                        // For now, we'll just leave it
                    }
                    current = b.previous.clone();
                }
            }

            // Restore bracket stack
            self.brackets = opener.previous;
        } else {
            // No match, restore bracket and add ]
            self.brackets = Some(opener);
            self.append_text(parent, "]");
        }

        true
    }

    /// Skip spaces and at most one newline
    fn skip_spaces_and_newlines(&mut self) {
        let mut saw_newline = false;
        while let Some(c) = self.peek() {
            if c == ' ' || c == '\t' {
                self.advance();
            } else if c == '\n' && !saw_newline {
                self.advance();
                saw_newline = true;
            } else {
                break;
            }
        }
    }

    /// Parse link destination (URL)
    fn parse_link_destination(&mut self) -> Option<String> {
        // Try angle-bracketed destination: <url>
        if self.peek() == Some('<') {
            self.advance(); // skip <
            let start = self.pos;

            while let Some(c) = self.peek() {
                if c == '>' {
                    let dest = self.input[start..self.pos].to_string();
                    self.advance(); // skip >
                    return Some(dest);
                } else if c == '\n' || c == '<' {
                    return None;
                } else if c == '\\' {
                    self.advance();
                    if let Some(next_c) = self.peek() {
                        if is_escapable(next_c) {
                            self.advance();
                        }
                    }
                } else {
                    self.advance();
                }
            }
            return None;
        }

        // Try unbracketed destination
        let start = self.pos;
        let mut paren_depth = 0;

        while let Some(c) = self.peek() {
            if c == '\\' {
                self.advance();
                if let Some(next_c) = self.peek() {
                    if is_escapable(next_c) {
                        self.advance();
                    }
                }
            } else if c == '(' {
                paren_depth += 1;
                self.advance();
            } else if c == ')' {
                if paren_depth == 0 {
                    break;
                }
                paren_depth -= 1;
                self.advance();
            } else if c.is_ascii_whitespace() {
                break;
            } else {
                self.advance();
            }
        }

        if self.pos > start {
            Some(self.input[start..self.pos].to_string())
        } else {
            None
        }
    }

    /// Parse link title
    fn parse_link_title(&mut self) -> Option<String> {
        let quote = match self.peek() {
            Some('"') => '"',
            Some('\'') => '\'',
            Some('(') => '(',
            _ => return None,
        };

        let close_quote = if quote == '(' { ')' } else { quote };
        self.advance(); // skip opening quote

        let start = self.pos;

        while let Some(c) = self.peek() {
            if c == close_quote {
                let title = self.input[start..self.pos].to_string();
                self.advance(); // skip closing quote
                return Some(title);
            } else if c == '\n' {
                return None;
            } else if c == '\\' {
                self.advance();
                if let Some(next_c) = self.peek() {
                    if is_escapable(next_c) {
                        self.advance();
                    }
                }
            } else {
                self.advance();
            }
        }

        None
    }

    /// Parse a link label like [label]
    /// Returns the length of the label including brackets, or 0 if no match
    fn parse_link_label(&mut self) -> usize {
        let start_pos = self.pos;

        // Must start with [
        if self.peek() != Some('[') {
            return 0;
        }
        self.advance(); // skip [

        let label_start = self.pos;
        let mut bracket_depth = 1;

        while let Some(c) = self.peek() {
            if c == '\\' {
                self.advance();
                if self.peek().is_some() {
                    self.advance();
                }
            } else if c == '[' {
                bracket_depth += 1;
                self.advance();
            } else if c == ']' {
                bracket_depth -= 1;
                if bracket_depth == 0 {
                    self.advance(); // skip ]
                    let label_len = self.pos - start_pos;
                    // Label must not be empty and max 999 characters (excluding brackets)
                    let content_len = self.pos - label_start - 1;
                    if content_len == 0 || content_len > 999 {
                        self.pos = start_pos;
                        return 0;
                    }
                    return label_len;
                }
                self.advance();
            } else if c == '\n' {
                // Labels cannot contain newlines
                self.pos = start_pos;
                return 0;
            } else {
                self.advance();
            }
        }

        // No closing bracket found
        self.pos = start_pos;
        0
    }

    /// Look up a reference in the refmap
    fn lookup_reference(&self, label: &str) -> Option<(String, String)> {
        self.refmap.get(label).cloned()
    }

    /// Parse bang (!, could be start of image)
    fn parse_bang(&mut self, parent: &Rc<RefCell<Node>>) -> bool {
        self.advance(); // skip !

        if self.peek() == Some('[') {
            self.advance(); // skip [
            let text_node = self.append_text(parent, "![");

            // Add to bracket stack as image
            let bracket = Box::new(Bracket {
                previous: self.brackets.take(),
                inl_text: text_node.clone(),
                position: self.pos - 2,
                image: true,
                active: true,
                bracket_after: false,
                previous_delimiter: self.delimiters.clone(),
            });

            self.brackets = Some(bracket);
            true
        } else {
            self.append_text(parent, "!");
            true
        }
    }

    /// Parse a string of non-special characters
    fn parse_string(&mut self, parent: &Rc<RefCell<Node>>) -> bool {
        let start = self.pos;

        while let Some(c) = self.peek() {
            if is_special_char(c) {
                break;
            }
            self.advance();
        }

        if self.pos > start {
            let text = self.input[start..self.pos].to_string();
            self.append_text(parent, &text);
            true
        } else {
            false
        }
    }

    /// Append text to parent, merging with previous text node if possible
    fn append_text(&mut self, parent: &Rc<RefCell<Node>>, text: &str) -> Rc<RefCell<Node>> {
        // Check if last child is a text node we can merge with
        let last_child_opt = parent.borrow().last_child.borrow().clone();

        if let Some(last_child) = last_child_opt {
            if last_child.borrow().node_type == NodeType::Text {
                // Merge with existing text node
                let mut last_mut = last_child.borrow_mut();
                if let NodeData::Text { ref mut literal, .. } = last_mut.data {
                    literal.push_str(text);
                }
                return last_child.clone();
            }
        }

        // Create new text node
        let text_node = Rc::new(RefCell::new(Node::new(NodeType::Text)));
        {
            let mut text_mut = text_node.borrow_mut();
            if let NodeData::Text { ref mut literal, .. } = text_mut.data {
                *literal = text.to_string();
            }
        }
        append_child(parent, text_node.clone());
        text_node
    }
}

/// Result of scanning delimiters
struct DelimScanResult {
    num_delims: usize,
    can_open: bool,
    can_close: bool,
}

/// Check if a character is special (has special meaning in inline parsing)
fn is_special_char(c: char) -> bool {
    matches!(c, '`' | '\\' | '&' | '<' | '*' | '_' | '[' | ']' | '!')
}

/// Check if a character can be escaped
fn is_escapable(c: char) -> bool {
    matches!(c, '!' | '"' | '#' | '$' | '%' | '&' | '\'' | '(' | ')' | '*' | '+' | ',' | '-' | '.' | '/' | ':' | ';' | '<' | '=' | '>' | '?' | '@' | '[' | '\\' | ']' | '^' | '_' | '`' | '{' | '|' | '}' | '~')
}

/// Normalize a reference label for lookup
/// - Collapses internal whitespace to a single space
/// - Removes leading/trailing whitespace
/// - Converts to uppercase (for case-insensitive comparison)
pub fn normalize_reference(label: &str) -> String {
    // Remove surrounding brackets if present
    let label = if label.starts_with('[') && label.ends_with(']') {
        &label[1..label.len() - 1]
    } else {
        label
    };

    label
        .trim()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_uppercase()
}

/// Check if a character is punctuation
fn is_punctuation(c: char) -> bool {
    matches!(c, '!' | '"' | '#' | '$' | '%' | '&' | '\'' | '(' | ')' | '*' | '+' | ',' | '-' | '.' | '/' | ':' | ';' | '<' | '=' | '>' | '?' | '@' | '[' | '\\' | ']' | '^' | '_' | '`' | '{' | '|' | '}' | '~')
        || c.is_ascii_punctuation()
}

/// Parse inline content into the given parent node
pub fn parse_inlines(parent: &Rc<RefCell<Node>>, content: &str, line: usize, block_offset: usize) {
    let mut subject = Subject::new(content, line, block_offset);
    subject.parse_inlines(parent);
}

/// Parse inline content with reference map
pub fn parse_inlines_with_refmap(
    parent: &Rc<RefCell<Node>>,
    content: &str,
    line: usize,
    block_offset: usize,
    refmap: std::collections::HashMap<String, (String, String)>,
) {
    let mut subject = Subject::with_refmap(content, line, block_offset, refmap);
    subject.parse_inlines(parent);
}

/// Parse a reference definition from the beginning of a string.
/// Returns the number of characters consumed, or 0 if no reference was found.
/// If a reference is found, it is added to the refmap.
pub fn parse_reference(s: &str, refmap: &mut std::collections::HashMap<String, (String, String)>) -> usize {
    let mut subject = Subject::new(s, 1, 0);
    subject.parse_reference_definition(refmap)
}

impl Subject {
    /// Parse a reference definition: [label]: url "title"
    /// Returns the number of characters consumed, or 0 if no reference was found
    fn parse_reference_definition(&mut self, refmap: &mut std::collections::HashMap<String, (String, String)>) -> usize {
        let start_pos = self.pos;

        // Parse label: [label]
        let label_len = self.parse_link_label();
        if label_len == 0 {
            return 0;
        }

        let raw_label = self.input[start_pos..start_pos + label_len].to_string();

        // Expect colon
        if self.peek() != Some(':') {
            return 0;
        }
        self.advance(); // skip :

        // Skip spaces and newlines
        self.skip_spaces_and_newlines();

        // Parse link destination
        let dest = match self.parse_link_destination() {
            Some(d) => d,
            None => return 0,
        };

        let before_title = self.pos;
        self.skip_spaces_and_newlines();

        // Try to parse optional title
        let title = if self.pos != before_title {
            self.parse_link_title()
        } else {
            None
        };

        if title.is_none() {
            self.pos = before_title;
        }

        // Must be at end of line
        let at_line_end = self.peek().map(|c| c == '\n' || c == '\r').unwrap_or(true)
            || self.pos == self.input.len();

        if !at_line_end {
            // Check if we can still match without title
            self.pos = before_title;
            let at_line_end_without_title = self.peek().map(|c| c == '\n' || c == '\r').unwrap_or(true)
                || self.pos == self.input.len();
            if !at_line_end_without_title {
                return 0;
            }
        }

        // Normalize label and add to refmap
        let norm_label = normalize_reference(&raw_label);
        if !norm_label.is_empty() {
            // Only add if not already present (first definition wins)
            refmap.entry(norm_label).or_insert((dest, title.unwrap_or_default()));
        }

        self.pos
    }
}

/// HTML entity patterns
const ENTITY_PATTERN: &str = r"&#x[a-fA-F0-9]{1,6};|&#[0-9]{1,7};|&[a-zA-Z][a-zA-Z0-9]{1,31};";

/// Parse an HTML entity and return the character it represents
fn parse_entity_char(input: &str) -> Option<(char, usize)> {
    if !input.starts_with('&') {
        return None;
    }

    // Try numeric entity: &#123; or &#x7B;
    if input.len() >= 3 && input.as_bytes()[1] == b'#' {
        let hex = input.as_bytes().get(2).copied() == Some(b'x') || input.as_bytes().get(2).copied() == Some(b'X');

        if hex {
            // Hex entity: &#x7B;
            let end = input.find(';').unwrap_or(input.len());
            if end > 3 && end <= 9 {
                let hex_str = &input[3..end];
                if let Ok(code) = u32::from_str_radix(hex_str, 16) {
                    if let Some(c) = char::from_u32(code) {
                        return Some((c, end + 1));
                    }
                }
            }
        } else {
            // Decimal entity: &#123;
            let end = input.find(';').unwrap_or(input.len());
            if end > 2 && end <= 8 {
                let num_str = &input[2..end];
                if let Ok(code) = num_str.parse::<u32>() {
                    if let Some(c) = char::from_u32(code) {
                        return Some((c, end + 1));
                    }
                }
            }
        }
    }

    // Try named entity
    let end = input.find(';').unwrap_or(input.len());
    if end > 1 && end <= 33 {
        let name = &input[1..end];
        if let Some(c) = lookup_named_entity(name) {
            return Some((c, end + 1));
        }
    }

    None
}

/// Look up a named HTML entity
fn lookup_named_entity(name: &str) -> Option<char> {
    match name {
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" => Some('"'),
        "apos" => Some('\''),
        "nbsp" => Some('\u{00A0}'),
        "copy" => Some('\u{00A9}'),
        "reg" => Some('\u{00AE}'),
        "trade" => Some('\u{2122}'),
        "mdash" => Some('\u{2014}'),
        "ndash" => Some('\u{2013}'),
        "hellip" => Some('\u{2026}'),
        "laquo" => Some('\u{00AB}'),
        "raquo" => Some('\u{00BB}'),
        "ldquo" => Some('\u{201C}'),
        "rdquo" => Some('\u{201D}'),
        "lsquo" => Some('\u{2018}'),
        "rsquo" => Some('\u{2019}'),
        _ => None,
    }
}

/// Email autolink pattern
fn match_email_autolink(input: &str) -> Option<(String, usize)> {
    if !input.starts_with('<') {
        return None;
    }

    // Simple email pattern: <local@domain.tld>
    let rest = &input[1..];

    let mut at_pos = None;
    let mut end_pos = 0;

    for (i, c) in rest.chars().enumerate() {
        if c == '@' && at_pos.is_none() && i > 0 {
            at_pos = Some(i);
        } else if c == '>' {
            end_pos = i;
            break;
        } else if c == '\n' || c == '<' {
            return None;
        }
    }

    if let Some(at) = at_pos {
        if at > 0 && end_pos > at + 1 {
            let email = &rest[..end_pos];
            // Basic validation: must have @ and at least one dot after @
            let domain_part = &rest[at + 1..end_pos];
            if domain_part.contains('.') {
                return Some((email.to_string(), end_pos + 2)); // +2 for < and >
            }
        }
    }

    None
}

/// URL autolink pattern
fn match_url_autolink(input: &str) -> Option<(String, usize)> {
    if !input.starts_with('<') {
        return None;
    }

    // URL pattern: <scheme:...>
    let rest = &input[1..];

    // Must start with a letter, then letters/digits/+-.
    let mut i = 0;
    let mut has_colon = false;

    for c in rest.chars() {
        if c == ':' {
            has_colon = true;
            i += 1;
            break;
        } else if c.is_ascii_alphabetic() || c.is_ascii_digit() || c == '+' || c == '-' || c == '.' {
            if i == 0 && !c.is_ascii_alphabetic() {
                return None;
            }
            i += 1;
            if i > 32 {
                return None; // Scheme too long
            }
        } else {
            return None;
        }
    }

    if !has_colon || i < 2 {
        return None;
    }

    // Now parse the rest of the URL
    let url_start = i;
    let mut end_pos = 0;

    for (j, c) in rest[url_start..].chars().enumerate() {
        if c == '>' {
            end_pos = url_start + j;
            break;
        } else if c == '\n' || c == '<' || c.is_ascii_control() {
            return None;
        }
    }

    if end_pos > url_start {
        let url = &rest[..end_pos];
        return Some((url.to_string(), end_pos + 2)); // +2 for < and >
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn test_parse_inline_code() {
        let parent = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        parse_inlines(&parent, "`code`", 1, 0);

        let parent_ref = parent.borrow();
        let first_child = parent_ref.first_child.borrow();
        assert!(first_child.is_some());
        assert_eq!(first_child.as_ref().unwrap().borrow().node_type, NodeType::Code);
    }

    #[test]
    fn test_parse_text() {
        let parent = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        parse_inlines(&parent, "hello world", 1, 0);

        let parent_ref = parent.borrow();
        let first_child = parent_ref.first_child.borrow();
        assert!(first_child.is_some());
        assert_eq!(first_child.as_ref().unwrap().borrow().node_type, NodeType::Text);
    }

    #[test]
    fn test_parse_emphasis() {
        let parent = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        parse_inlines(&parent, "*emphasis*", 1, 0);

        // Should create text nodes for now (emphasis processing not fully implemented)
        let parent_ref = parent.borrow();
        let first_child = parent_ref.first_child.borrow();
        assert!(first_child.is_some());
    }

    #[test]
    fn test_parse_link() {
        let parent = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        parse_inlines(&parent, "[link text](https://example.com)", 1, 0);

        let parent_ref = parent.borrow();
        let first_child = parent_ref.first_child.borrow();
        assert!(first_child.is_some());
        assert_eq!(first_child.as_ref().unwrap().borrow().node_type, NodeType::Link);

        // Check link URL
        let link = first_child.as_ref().unwrap().borrow();
        match &link.data {
            NodeData::Link { url, .. } => {
                assert_eq!(url, "https://example.com");
            }
            _ => panic!("Expected Link node"),
        }
    }

    #[test]
    fn test_parse_link_with_title() {
        let parent = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        parse_inlines(&parent, "[link](https://example.com \"title\")", 1, 0);

        let parent_ref = parent.borrow();
        let first_child = parent_ref.first_child.borrow();
        assert!(first_child.is_some());
        assert_eq!(first_child.as_ref().unwrap().borrow().node_type, NodeType::Link);

        let link = first_child.as_ref().unwrap().borrow();
        match &link.data {
            NodeData::Link { url, title, .. } => {
                assert_eq!(url, "https://example.com");
                assert_eq!(title, "title");
            }
            _ => panic!("Expected Link node"),
        }
    }

    #[test]
    fn test_parse_image() {
        let parent = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        parse_inlines(&parent, "![alt text](https://example.com/image.png)", 1, 0);

        let parent_ref = parent.borrow();
        let first_child = parent_ref.first_child.borrow();
        assert!(first_child.is_some());
        assert_eq!(first_child.as_ref().unwrap().borrow().node_type, NodeType::Image);

        let img = first_child.as_ref().unwrap().borrow();
        match &img.data {
            NodeData::Image { url, .. } => {
                assert_eq!(url, "https://example.com/image.png");
            }
            _ => panic!("Expected Image node"),
        }
    }

    #[test]
    fn test_parse_entity() {
        let parent = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        parse_inlines(&parent, "&amp; &lt; &gt; &quot;", 1, 0);

        let parent_ref = parent.borrow();
        let first_child = parent_ref.first_child.borrow();
        assert!(first_child.is_some());

        let text = first_child.as_ref().unwrap().borrow();
        match &text.data {
            NodeData::Text { literal } => {
                assert_eq!(literal, "& < > \"");
            }
            _ => panic!("Expected Text node"),
        }
    }

    #[test]
    fn test_parse_numeric_entity() {
        let parent = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        parse_inlines(&parent, "&#60; &#x3C;", 1, 0);

        let parent_ref = parent.borrow();
        let first_child = parent_ref.first_child.borrow();
        assert!(first_child.is_some());

        let text = first_child.as_ref().unwrap().borrow();
        match &text.data {
            NodeData::Text { literal } => {
                assert_eq!(literal, "< <");
            }
            _ => panic!("Expected Text node"),
        }
    }

    #[test]
    fn test_parse_url_autolink() {
        let parent = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        parse_inlines(&parent, "<https://example.com>", 1, 0);

        let parent_ref = parent.borrow();
        let first_child = parent_ref.first_child.borrow();
        assert!(first_child.is_some());
        assert_eq!(first_child.as_ref().unwrap().borrow().node_type, NodeType::Link);

        let link = first_child.as_ref().unwrap().borrow();
        match &link.data {
            NodeData::Link { url, .. } => {
                assert_eq!(url, "https://example.com");
            }
            _ => panic!("Expected Link node"),
        }
    }

    #[test]
    fn test_parse_email_autolink() {
        let parent = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        parse_inlines(&parent, "<test@example.com>", 1, 0);

        let parent_ref = parent.borrow();
        let first_child = parent_ref.first_child.borrow();
        assert!(first_child.is_some());
        assert_eq!(first_child.as_ref().unwrap().borrow().node_type, NodeType::Link);

        let link = first_child.as_ref().unwrap().borrow();
        match &link.data {
            NodeData::Link { url, .. } => {
                assert_eq!(url, "mailto:test@example.com");
            }
            _ => panic!("Expected Link node"),
        }
    }

    #[test]
    fn test_parse_reference() {
        let mut refmap = std::collections::HashMap::new();

        // Parse a reference definition
        let consumed = parse_reference("[label]: https://example.com \"title\"", &mut refmap);
        assert!(consumed > 0);

        // Check that the reference was added
        let (url, title) = refmap.get("LABEL").expect("Reference should be in map");
        assert_eq!(url, "https://example.com");
        assert_eq!(title, "title");
    }

    #[test]
    fn test_parse_reference_no_title() {
        let mut refmap = std::collections::HashMap::new();

        // Parse a reference definition without title
        let consumed = parse_reference("[label]: https://example.com", &mut refmap);
        assert!(consumed > 0);

        let (url, title) = refmap.get("LABEL").expect("Reference should be in map");
        assert_eq!(url, "https://example.com");
        assert_eq!(title, "");
    }

    #[test]
    fn test_reference_link() {
        let mut refmap = std::collections::HashMap::new();
        refmap.insert("LABEL".to_string(), ("https://example.com".to_string(), "title".to_string()));

        let parent = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        parse_inlines_with_refmap(&parent, "[text][label]", 1, 0, refmap);

        let parent_ref = parent.borrow();
        let first_child = parent_ref.first_child.borrow();
        assert!(first_child.is_some());
        assert_eq!(first_child.as_ref().unwrap().borrow().node_type, NodeType::Link);

        let link = first_child.as_ref().unwrap().borrow();
        match &link.data {
            NodeData::Link { url, title, .. } => {
                assert_eq!(url, "https://example.com");
                assert_eq!(title, "title");
            }
            _ => panic!("Expected Link node"),
        }
    }
}
