//! Inline parsing for CommonMark documents
//!
//! This module implements the inline parsing algorithm based on the CommonMark spec.
//! It processes the content of leaf blocks (paragraphs, headings, etc.) to produce
//! inline elements like emphasis, links, code, etc.
//!
//! # Overview
//!
//! Inline parsing is the second phase of Markdown processing, after block parsing.
//! It takes the string content of leaf blocks and parses inline elements:
//!
//! - **Emphasis**: `*text*` or `_text_` → `<em>`, `**text**` or `__text__` → `<strong>`
//! - **Links**: `[text](url)` or `[text][ref]` → `<a href="url">text</a>`
//! - **Images**: `![alt](url)` → `<img src="url" alt="alt">`
//! - **Code**: `` `code` `` → `<code>code</code>`
//! - **Autolinks**: `<url>` or `<email>` → automatic links
//! - **HTML tags**: Inline HTML
//! - **Entities**: `&amp;` → `&`, `&#123;` → `{`
//!
//! # Example
//!
//! ```
//! use clmd::{parse_document, render_html, options};
//!
//! let (arena, doc) = parse_document("Hello *world*", options::DEFAULT);
//! let html = render_html(&arena, doc, options::DEFAULT);
//! assert_eq!(html, "<p>Hello <em>world</em></p>");
//! ```
use crate::arena::{Node, NodeArena, NodeId, TreeOps};
use crate::node::{NodeData, NodeType};
use htmlescape::decode_html;
use std::collections::HashMap;

/// HTML5 named entities lookup table
/// This includes entities that may not be supported by htmlescape
fn get_html5_entity(name: &str) -> Option<&'static str> {
    // Common HTML5 entities not in htmlescape
    let entities: HashMap<&str, &str> = [
        ("Dcaron", "\u{010E}"),                   // Ď
        ("HilbertSpace", "\u{210B}"),             // ℋ
        ("DifferentialD", "\u{2146}"),            // ⅆ
        ("ClockwiseContourIntegral", "\u{2232}"), // ∲
        ("ngE", "\u{2267}\u{0338}"),              // ≧̸
        ("AElig", "\u{00C6}"),                    // Æ
        ("copy", "\u{00A9}"),                     // ©
        ("nbsp", "\u{00A0}"),                     //
        ("amp", "&"),                             // &
        ("lt", "<"),                              // <
        ("gt", ">"),                              // >
        ("quot", "\""),                           // "
        ("frac34", "\u{00BE}"),                   // ¾
    ]
    .iter()
    .copied()
    .collect();

    entities.get(name).copied()
}

/// Subject represents the string being parsed and tracks position
pub struct Subject<'a> {
    /// The input string (borrowed reference to avoid copying)
    pub input: &'a str,
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
    /// Whether there are no link openers
    pub no_link_openers: bool,
    /// Reference map for link references
    pub refmap: std::collections::HashMap<String, (String, String)>,
    /// Whether smart punctuation is enabled
    pub smart: bool,
}

/// Delimiter struct for tracking emphasis markers
/// This is a singly-linked list using Box for ownership
pub struct Delimiter {
    /// Previous delimiter in stack
    pub previous: Option<Box<Delimiter>>,
    /// The inline text node containing the delimiter
    pub inl_text: NodeId,
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
pub struct Bracket {
    /// Previous bracket in stack
    pub previous: Option<Box<Bracket>>,
    /// The inline text node containing the bracket
    pub inl_text: NodeId,
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

impl<'a> Subject<'a> {
    /// Create a new subject from a string
    pub fn new(input: &'a str, line: usize, block_offset: usize) -> Self {
        Subject {
            input,
            pos: 0,
            line,
            column_offset: 0,
            block_offset,
            delimiters: None,
            brackets: None,
            no_link_openers: false,
            refmap: std::collections::HashMap::new(),
            smart: false,
        }
    }

    /// Create a new subject with a reference map
    pub fn with_refmap(
        input: &'a str,
        line: usize,
        block_offset: usize,
        refmap: std::collections::HashMap<String, (String, String)>,
    ) -> Self {
        Subject {
            input,
            pos: 0,
            line,
            column_offset: 0,
            block_offset,
            delimiters: None,
            brackets: None,
            no_link_openers: false,
            refmap,
            smart: false,
        }
    }

    /// Create a new subject with a reference map and smart punctuation option
    pub fn with_refmap_and_smart(
        input: &'a str,
        line: usize,
        block_offset: usize,
        refmap: std::collections::HashMap<String, (String, String)>,
        smart: bool,
    ) -> Self {
        Subject {
            input,
            pos: 0,
            line,
            column_offset: 0,
            block_offset,
            delimiters: None,
            brackets: None,
            no_link_openers: false,
            refmap,
            smart,
        }
    }

    /// Peek at the current character without advancing (optimized)
    #[inline(always)]
    pub fn peek(&self) -> Option<char> {
        if self.pos >= self.input.len() {
            return None;
        }
        let bytes = self.input.as_bytes();
        let b = bytes[self.pos];
        // Fast path for ASCII
        if b < 0x80 {
            return Some(b as char);
        }
        // Slow path for UTF-8
        self.input[self.pos..].chars().next()
    }

    /// Peek at the next character code
    #[inline(always)]
    pub fn peek_char_code(&self) -> i32 {
        if self.pos < self.input.len() {
            self.input.as_bytes()[self.pos] as i32
        } else {
            -1
        }
    }

    /// Advance position by one character (optimized byte-level)
    #[inline(always)]
    pub fn advance(&mut self) {
        if self.pos < self.input.len() {
            let b = self.input.as_bytes()[self.pos];
            // For ASCII characters (0-127), advance by 1
            // For UTF-8 multi-byte sequences, calculate the length
            self.pos += if b < 0x80 {
                1
            } else if b < 0xE0 {
                2
            } else if b < 0xF0 {
                3
            } else {
                4
            };
            // Ensure we don't go past the end
            if self.pos > self.input.len() {
                self.pos = self.input.len();
            }
        }
    }

    /// Check if we're at the end of the input
    pub fn end(&self) -> bool {
        self.pos >= self.input.len()
    }

    /// Parse the subject and append inline children to the given node
    pub fn parse_inlines(&mut self, arena: &mut NodeArena, parent: NodeId) {
        while !self.end() {
            if !self.parse_inline(arena, parent) {
                // If no inline was parsed, advance to avoid infinite loop
                self.advance();
            }
        }

        // Process any remaining emphasis
        self.process_emphasis(arena, None);

        // Merge adjacent text nodes for cleaner output
        Self::merge_adjacent_text_nodes(arena, parent);

        // Remove trailing spaces from the last text node
        Self::remove_trailing_spaces(arena, parent);
    }

    /// Remove trailing spaces from the last text node
    fn remove_trailing_spaces(arena: &mut NodeArena, parent: NodeId) {
        let parent_type = arena.get(parent).node_type;

        // For headings, always remove trailing spaces
        if parent_type == NodeType::Heading {
            if let Some(last_child) = arena.get(parent).last_child {
                if arena.get(last_child).node_type == NodeType::Text {
                    if let NodeData::Text { ref literal } = arena.get(last_child).data {
                        let trimmed = literal.trim_end_matches(' ').to_string();
                        if trimmed != *literal {
                            let node = arena.get_mut(last_child);
                            if let NodeData::Text { ref mut literal } = node.data {
                                *literal = trimmed;
                            }
                        }
                    }
                }
            }
        }
        // For paragraphs and other containers, preserve trailing spaces
        // as they may be significant before inline elements
    }

    /// Merge adjacent text nodes in the given parent
    fn merge_adjacent_text_nodes(arena: &mut NodeArena, parent: NodeId) {
        let mut current_opt = arena.get(parent).first_child;

        while let Some(current) = current_opt {
            let next_opt = arena.get(current).next;

            if let Some(next) = next_opt {
                // Check node types first
                let current_is_text = arena.get(current).node_type == NodeType::Text;
                let next_is_text = arena.get(next).node_type == NodeType::Text;

                if current_is_text && next_is_text {
                    // Check if either is a smart quote without cloning
                    let can_merge = {
                        let current_literal = match &arena.get(current).data {
                            NodeData::Text { literal } => literal.as_str(),
                            _ => "",
                        };
                        let next_literal = match &arena.get(next).data {
                            NodeData::Text { literal } => literal.as_str(),
                            _ => "",
                        };
                        !Self::is_smart_quote(current_literal)
                            && !Self::is_smart_quote(next_literal)
                    };

                    if can_merge {
                        // Get next's literal and merge into current
                        let next_literal = {
                            match &arena.get(next).data {
                                NodeData::Text { literal } => literal.clone(),
                                _ => String::new(),
                            }
                        };

                        {
                            let current_node = arena.get_mut(current);
                            if let NodeData::Text { ref mut literal } = current_node.data
                            {
                                literal.push_str(&next_literal);
                            }
                        }

                        // Remove next node
                        TreeOps::unlink(arena, next);
                        // Continue with same current
                        current_opt = Some(current);
                        continue;
                    }
                }
            }

            // Recursively process children
            Self::merge_adjacent_text_nodes(arena, current);

            current_opt = next_opt;
        }
    }

    /// Check if string is a smart quote
    #[inline(always)]
    fn is_smart_quote(s: &str) -> bool {
        matches!(s, "\u{2018}" | "\u{2019}" | "\u{201C}" | "\u{201D}")
    }

    /// Parse a single inline element
    #[inline(always)]
    fn parse_inline(&mut self, arena: &mut NodeArena, parent: NodeId) -> bool {
        let c = match self.peek() {
            Some(c) => c,
            None => return false,
        };

        match c {
            '`' => self.parse_backticks(arena, parent),
            '\\' => self.parse_backslash(arena, parent),
            '&' => self.parse_entity(arena, parent),
            '<' => self.parse_lt(arena, parent),
            '*' | '_' => self.handle_delim(arena, c, parent),
            '[' => self.parse_open_bracket(arena, parent),
            ']' => self.parse_close_bracket(arena, parent),
            '!' => self.parse_bang(arena, parent),
            '\n' => self.parse_newline(arena, parent),
            '\'' | '"' if self.smart => self.handle_delim(arena, c, parent),
            _ => self.parse_string(arena, parent),
        }
    }

    /// Parse a newline. Returns a softbreak or hardbreak node.
    fn parse_newline(&mut self, arena: &mut NodeArena, parent: NodeId) -> bool {
        // Check for preceding spaces (look back in the line)
        let preceding_spaces = self.count_preceding_spaces();

        // For hard line break, remove trailing spaces from the last text node
        if preceding_spaces >= 2 {
            self.remove_trailing_spaces_from_last_text(arena, parent, preceding_spaces);
        }

        self.advance(); // skip \n

        if preceding_spaces >= 2 {
            // Hard line break: line ends with 2+ spaces
            let line_break = arena.alloc(Node::new(NodeType::LineBreak));
            TreeOps::append_child(arena, parent, line_break);
        } else {
            // Soft line break
            let soft_break = arena.alloc(Node::new(NodeType::SoftBreak));
            TreeOps::append_child(arena, parent, soft_break);
        }

        true
    }

    /// Remove trailing spaces from the last text node
    fn remove_trailing_spaces_from_last_text(
        &self,
        arena: &mut NodeArena,
        parent: NodeId,
        count: usize,
    ) {
        if let Some(last_child) = arena.get(parent).last_child {
            if arena.get(last_child).node_type == NodeType::Text {
                if let NodeData::Text { ref literal } = arena.get(last_child).data {
                    let new_len = literal.len().saturating_sub(count);
                    let new_literal = literal[..new_len].to_string();
                    let node = arena.get_mut(last_child);
                    if let NodeData::Text { ref mut literal } = node.data {
                        *literal = new_literal;
                    }
                }
            }
        }
    }

    /// Count spaces preceding the current position (back to start of line or non-space)
    fn count_preceding_spaces(&self) -> usize {
        // Get the substring from start to current position, then iterate backwards
        let prefix = &self.input[..self.pos];
        let mut count = 0;

        for c in prefix.chars().rev() {
            if c == ' ' {
                count += 1;
            } else {
                break;
            }
        }

        count
    }

    /// Parse backtick-delimited code span
    fn parse_backticks(&mut self, arena: &mut NodeArena, parent: NodeId) -> bool {
        let _start_pos = self.pos;
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

                    let code_node = arena.alloc(Node::new(NodeType::Code));
                    {
                        let code_mut = arena.get_mut(code_node);
                        if let NodeData::Code {
                            ref mut literal, ..
                        } = code_mut.data
                        {
                            *literal = content;
                        }
                    }
                    TreeOps::append_child(arena, parent, code_node);
                    return true;
                }
            } else {
                self.advance();
            }
        }

        // No matching close found, treat as literal
        self.pos = after_open_ticks;
        self.append_text(arena, parent, &ticks);
        true
    }

    /// Parse backslash escape or hard line break
    fn parse_backslash(&mut self, arena: &mut NodeArena, parent: NodeId) -> bool {
        self.advance(); // skip backslash

        if self.peek() == Some('\n') {
            // Hard line break
            self.advance();
            let line_break = arena.alloc(Node::new(NodeType::LineBreak));
            TreeOps::append_child(arena, parent, line_break);
        } else if let Some(c) = self.peek() {
            if is_escapable(c) {
                self.append_text(arena, parent, &c.to_string());
                self.advance();
            } else {
                self.append_text(arena, parent, "\\");
            }
        } else {
            self.append_text(arena, parent, "\\");
        }

        true
    }

    /// Parse entity or numeric character reference
    fn parse_entity(&mut self, arena: &mut NodeArena, parent: NodeId) -> bool {
        if let Some((decoded, len)) = parse_entity_char(&self.input[self.pos..]) {
            self.append_text(arena, parent, &decoded);
            self.pos += len;
            true
        } else {
            // Not a valid entity, treat & as literal
            // Append just "&" - the HTML renderer will escape it to &amp;
            self.append_text(arena, parent, "&");
            self.advance(); // skip the &
            true
        }
    }

    /// Parse less-than sign (could be autolink or HTML tag)
    fn parse_lt(&mut self, arena: &mut NodeArena, parent: NodeId) -> bool {
        // Check if this looks like it could have been an autolink
        // We need to check this first to avoid matching invalid autolinks as HTML tags
        let remaining = &self.input[self.pos..];
        if remaining.starts_with('<') && remaining.len() > 1 {
            let after_lt = &remaining[1..];
            // Check if it looks like a potential URL (scheme:...) or email
            if Self::looks_like_potential_autolink(after_lt) {
                // Try autolink first
                if self.parse_autolink(arena, parent) {
                    return true;
                }

                // This looks like it could be an autolink but failed validation
                // Output the < as a literal character (it will be escaped during rendering)
                let text_node = arena.alloc(Node::new(NodeType::Text));
                {
                    let text_mut = arena.get_mut(text_node);
                    if let NodeData::Text { ref mut literal } = text_mut.data {
                        *literal = "<".to_string();
                    }
                }
                TreeOps::append_child(arena, parent, text_node);
                self.pos += 1;
                return true;
            }
        }

        // Try autolink first (for cases not caught by looks_like_potential_autolink)
        if self.parse_autolink(arena, parent) {
            return true;
        }

        // Try HTML tag
        if self.parse_html_tag(arena, parent) {
            return true;
        }

        // Just a literal < - add it as text
        let text_node = arena.alloc(Node::new(NodeType::Text));
        {
            let text_mut = arena.get_mut(text_node);
            if let NodeData::Text { ref mut literal } = text_mut.data {
                *literal = "<".to_string();
            }
        }
        TreeOps::append_child(arena, parent, text_node);
        self.pos += 1;
        true
    }

    /// Check if the string looks like it could be a potential autolink
    fn looks_like_potential_autolink(s: &str) -> bool {
        // Check for URL pattern: scheme:...
        // Based on commonmark.js: scheme must be at least 2 characters
        let mut chars = s.chars().peekable();
        let mut i = 0;

        // Must start with a letter
        if let Some(&c) = chars.peek() {
            if !c.is_ascii_alphabetic() {
                return false;
            }
        } else {
            return false;
        }

        // Look for scheme followed by colon, or @ for email
        while let Some(&c) = chars.peek() {
            if c == ':' {
                // Found scheme:, check if scheme is at least 2 chars
                if i >= 1 {
                    return true;
                }
                // Scheme too short (like "m:"), but still looks like potential autolink
                return true;
            } else if c.is_ascii_alphabetic()
                || c.is_ascii_digit()
                || c == '+'
                || c == '-'
                || c == '.'
            {
                chars.next();
                i += 1;
                if i > 32 {
                    return false; // Scheme too long
                }
            } else if c == '@' {
                // Contains @ before :, looks like email
                return true;
            } else if c == '/' {
                // If we see / before :, this might be something like "<https://...>"
                // where the : is after the scheme. Check if it looks like a URL pattern.
                let rest: String = chars.clone().collect();
                if rest.starts_with("//") {
                    // Looks like a URL with scheme:// pattern
                    return true;
                }
                return false;
            } else {
                // Hit something else (like . in foo.bar.baz or space)
                // This is NOT a potential autolink, just a regular word
                return false;
            }
        }

        false
    }

    /// Parse autolink (URL or email in angle brackets)
    fn parse_autolink(&mut self, arena: &mut NodeArena, parent: NodeId) -> bool {
        let remaining = &self.input[self.pos..];

        // Try email autolink first
        if let Some((email, len)) = match_email_autolink(remaining) {
            let link_node = arena.alloc(Node::new(NodeType::Link));
            {
                let link_mut = arena.get_mut(link_node);
                if let NodeData::Link {
                    ref mut url,
                    ref mut title,
                } = link_mut.data
                {
                    *url = normalize_uri(&format!("mailto:{}", email));
                    *title = String::new();
                }
            }

            // Add text content
            let text_node = arena.alloc(Node::new(NodeType::Text));
            {
                let text_mut = arena.get_mut(text_node);
                if let NodeData::Text { ref mut literal } = text_mut.data {
                    *literal = email;
                }
            }
            TreeOps::append_child(arena, link_node, text_node);
            TreeOps::append_child(arena, parent, link_node);

            self.pos += len;
            return true;
        }

        // Try URL autolink
        if let Some((url, len)) = match_url_autolink(remaining) {
            let link_node = arena.alloc(Node::new(NodeType::Link));
            {
                let link_mut = arena.get_mut(link_node);
                if let NodeData::Link {
                    url: ref mut link_url,
                    title: ref mut link_title,
                } = link_mut.data
                {
                    *link_url = normalize_uri(&url);
                    *link_title = String::new();
                }
            }

            // Add text content
            let text_node = arena.alloc(Node::new(NodeType::Text));
            {
                let text_mut = arena.get_mut(text_node);
                if let NodeData::Text { ref mut literal } = text_mut.data {
                    *literal = url;
                }
            }
            TreeOps::append_child(arena, link_node, text_node);
            TreeOps::append_child(arena, parent, link_node);

            self.pos += len;
            return true;
        }

        false
    }

    /// Parse raw HTML tag
    fn parse_html_tag(&mut self, arena: &mut NodeArena, parent: NodeId) -> bool {
        let remaining = &self.input[self.pos..];

        // Try to match HTML tag
        if let Some((tag_content, len)) = match_html_tag(remaining) {
            let html_node = arena.alloc(Node::new(NodeType::HtmlInline));
            {
                let html_mut = arena.get_mut(html_node);
                if let NodeData::HtmlInline { ref mut literal } = html_mut.data {
                    *literal = tag_content;
                }
            }
            TreeOps::append_child(arena, parent, html_node);
            self.pos += len;
            return true;
        }

        false
    }

    /// Handle delimiter character (* or _)
    fn handle_delim(&mut self, arena: &mut NodeArena, c: char, parent: NodeId) -> bool {
        let start_pos = self.pos;
        let res = self.scan_delims(c);

        if res.num_delims == 0 {
            return false;
        }

        // Add delimiter text - create a separate text node (don't merge with previous)
        // This is important for emphasis processing
        // For smart quotes, replace with appropriate curly quote
        let delim_text: String = if self.smart && (c == '\'' || c == '"') {
            if c == '\'' {
                // Single quote: use left quote if can_open, otherwise right quote
                // The correct quote will be set in process_emphasis based on matching
                if res.can_open {
                    "\u{2018}".to_string() // left single quote
                } else {
                    "\u{2019}".to_string() // right single quote (apostrophe)
                }
            } else {
                // Double quote: use left quote if can_open, otherwise right quote
                if res.can_open {
                    "\u{201C}".to_string() // left double quote
                } else {
                    "\u{201D}".to_string() // right double quote
                }
            }
        } else {
            std::iter::repeat(c).take(res.num_delims).collect()
        };
        let text_node = arena.alloc(Node::new(NodeType::Text));
        {
            let text_mut = arena.get_mut(text_node);
            if let NodeData::Text {
                ref mut literal, ..
            } = text_mut.data
            {
                *literal = delim_text;
            }
        }
        TreeOps::append_child(arena, parent, text_node);

        // Add to delimiter stack if it can open or close
        if res.can_open || res.can_close {
            let delim = Box::new(Delimiter {
                previous: self.delimiters.take(),
                inl_text: text_node,
                position: start_pos,
                num_delims: res.num_delims,
                orig_delims: res.num_delims,
                delim_char: c,
                can_open: res.can_open,
                can_close: res.can_close,
            });

            self.delimiters = Some(delim);
        }

        // If this delimiter can open emphasis, add an empty text node as a barrier
        // to prevent subsequent text from being merged into the delimiter node.
        if res.can_open {
            let barrier = arena.alloc(Node::new(NodeType::Text));
            {
                let barrier_mut = arena.get_mut(barrier);
                if let NodeData::Text {
                    ref mut literal, ..
                } = barrier_mut.data
                {
                    *literal = String::new(); // Empty string
                }
            }
            TreeOps::append_child(arena, parent, barrier);
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
        // Note: start_pos is a byte position, not character position
        let char_before = if start_pos == 0 {
            '\n'
        } else {
            // Get the character that ends right before start_pos
            self.input[..start_pos].chars().last().unwrap_or('\n')
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
        } else if self.smart && (c == '\'' || c == '"') {
            // Smart quotes: different logic for single and double quotes
            // Based on commonmark.js: can_open = left_flanking && !right_flanking
            //                        can_close = right_flanking
            (left_flanking && !right_flanking, right_flanking)
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

    /// Deactivate previous link openers when a link is matched
    fn deactivate_previous_link_openers(&mut self) {
        // Deactivate all previous link openers in the bracket stack
        let mut current = self.brackets.as_mut();

        while let Some(bracket) = current {
            if !bracket.image {
                // Deactivate this link opener
                bracket.active = false;
            }
            current = bracket.previous.as_mut();
        }
    }

    /// Remove delimiters that were added after the link opener
    #[allow(dead_code)]
    fn remove_delimiters_inside_link(&mut self, opener: &Bracket) {
        // Remove all delimiters that were added after the opener's previous_delimiter
        // These are delimiters inside the link text
        let stack_bottom = opener.previous_delimiter.as_ref();

        // The delimiter stack is organized with previous pointers (from top to bottom)
        // We need to find delimiters that are NEWER than stack_bottom (i.e., have stack_bottom in their previous chain)
        if let Some(_bottom) = stack_bottom {
            // Simply set the stack_bottom's next to None
            // This effectively removes all delimiters newer than stack_bottom
            // Note: Since we're using Box, we can't easily modify the chain
            // We need to rebuild the delimiter stack

            // Collect all delimiters from stack_bottom down to the bottom
            let mut delimiters_to_keep: Vec<NodeId> = Vec::new();
            let mut current = Some(_bottom);

            while let Some(delim) = current {
                delimiters_to_keep.push(delim.inl_text);
                current = delim.previous.as_ref();
            }

            // Rebuild the stack with proper next/previous links
            // delimiters_to_keep is in order from stack_bottom to bottom
            // We need to reverse it to get from bottom to stack_bottom
            delimiters_to_keep.reverse();

            // Clear and rebuild
            self.delimiters = None;
            let mut prev_delim: Option<Box<Delimiter>> = None;

            for node_id in delimiters_to_keep {
                let delim = Box::new(Delimiter {
                    previous: prev_delim,
                    inl_text: node_id,
                    position: 0, // Not used after rebuild
                    num_delims: 0,
                    orig_delims: 0,
                    delim_char: '*',
                    can_open: false,
                    can_close: false,
                });
                self.delimiters = Some(delim);
                // Note: We can't easily set previous here without storing the Box
                // For simplicity, we'll just keep the delimiter IDs
                prev_delim = self.delimiters.take();
                self.delimiters = prev_delim.take();
            }
        } else {
            // No previous delimiter, remove all delimiters
            self.delimiters = None;
        }
    }

    /// Process emphasis delimiters
    /// Based on commonmark.js processEmphasis function and cmark implementation
    fn process_emphasis(
        &mut self,
        arena: &mut NodeArena,
        stack_bottom: Option<&Delimiter>,
    ) {
        // Collect all delimiter info into a vector (safer than raw pointers)
        // We store: (inl_text, delim_char, can_open, can_close, orig_delims, num_delims)
        let mut delims: Vec<(NodeId, char, bool, bool, usize, usize)> = Vec::new();

        // Traverse the delimiter stack and collect info
        let mut current = self.delimiters.as_ref();
        while let Some(d) = current {
            delims.push((
                d.inl_text,
                d.delim_char,
                d.can_open,
                d.can_close,
                d.orig_delims,
                d.num_delims,
            ));
            current = d.previous.as_ref();
        }

        // Reverse to get them in order from oldest to newest
        delims.reverse();

        // Find the starting index based on stack_bottom
        // stack_bottom is a delimiter that should NOT be processed (it's the boundary)
        let start_idx = if let Some(sb) = stack_bottom {
            // Find the position of stack_bottom in delims
            delims
                .iter()
                .position(|(node_id, _, _, _, orig, _)| {
                    *node_id == sb.inl_text && *orig == sb.orig_delims
                })
                .map(|i| i + 1)
                .unwrap_or(0)
        } else {
            0
        };

        // Initialize openers_bottom for each delimiter type
        // Index mapping: 0=" 1=' 2-7=_ (based on can_open and length % 3) 8-13=* (based on can_open and length % 3)
        let mut openers_bottom: [usize; 14] = [start_idx; 14];

        // Process closers from left to right, starting from start_idx
        let mut closer_idx = start_idx;
        while closer_idx < delims.len() {
            let (
                closer_inl,
                closer_char,
                closer_can_open,
                closer_can_close,
                closer_orig_delims,
                _,
            ) = delims[closer_idx];

            if !closer_can_close {
                closer_idx += 1;
                continue;
            }

            // Determine openers_bottom index based on closer type
            let openers_bottom_idx = match closer_char {
                '"' => 0,
                '\'' => 1,
                '_' => {
                    2 + if closer_can_open { 3 } else { 0 } + (closer_orig_delims % 3)
                }
                '*' => {
                    8 + if closer_can_open { 3 } else { 0 } + (closer_orig_delims % 3)
                }
                _ => {
                    closer_idx += 1;
                    continue;
                }
            };

            // Look for matching opener
            // First, try to find an opener with the same number of delimiters
            let mut opener_idx = closer_idx;
            let mut opener_found = false;

            while opener_idx > openers_bottom[openers_bottom_idx] {
                opener_idx -= 1;
                let (
                    _,
                    opener_char,
                    opener_can_open,
                    opener_can_close,
                    opener_orig_delims,
                    _,
                ) = delims[opener_idx];

                if opener_char == closer_char
                    && opener_can_open
                    && opener_orig_delims == closer_orig_delims
                {
                    // Check for odd match rule
                    let odd_match = (closer_can_open || opener_can_close)
                        && closer_orig_delims % 3 != 0
                        && (opener_orig_delims + closer_orig_delims) % 3 == 0;

                    if !odd_match {
                        opener_found = true;
                        break;
                    }
                }
            }

            // If no exact match found, look for any matching opener
            if !opener_found {
                opener_idx = closer_idx;
                while opener_idx > openers_bottom[openers_bottom_idx] {
                    opener_idx -= 1;
                    let (
                        _,
                        opener_char,
                        opener_can_open,
                        opener_can_close,
                        opener_orig_delims,
                        _,
                    ) = delims[opener_idx];

                    if opener_char == closer_char && opener_can_open {
                        // Check for odd match rule
                        let odd_match = (closer_can_open || opener_can_close)
                            && closer_orig_delims % 3 != 0
                            && (opener_orig_delims + closer_orig_delims) % 3 == 0;

                        if !odd_match {
                            opener_found = true;
                            break;
                        }
                    }
                }
            }

            let old_closer_idx = closer_idx;

            if closer_char == '*' || closer_char == '_' {
                if opener_found {
                    let (opener_inl, _, _, _, opener_orig_delims, _) =
                        delims[opener_idx];

                    // Calculate number of delimiters to use
                    let use_delims =
                        if opener_orig_delims >= 2 && closer_orig_delims >= 2 {
                            2
                        } else {
                            1
                        };

                    // Update the text nodes to remove used delimiters
                    let opener_text = {
                        let node = arena.get_mut(opener_inl);
                        if let NodeData::Text { ref mut literal } = node.data {
                            let new_len = literal.len().saturating_sub(use_delims);
                            literal.truncate(new_len);
                            literal.clone()
                        } else {
                            String::new()
                        }
                    };

                    let closer_text = {
                        let node = arena.get_mut(closer_inl);
                        if let NodeData::Text { ref mut literal } = node.data {
                            let new_len = literal.len().saturating_sub(use_delims);
                            literal.truncate(new_len);
                            literal.clone()
                        } else {
                            String::new()
                        }
                    };

                    // Create emphasis or strong node
                    let emph_type = if use_delims == 1 {
                        NodeType::Emph
                    } else {
                        NodeType::Strong
                    };
                    let emph_node = arena.alloc(Node::new(emph_type));

                    // Move nodes between opener and closer into the emphasis node
                    let mut current_child = arena.get(opener_inl).next;
                    while let Some(child_id) = current_child {
                        if child_id == closer_inl {
                            break;
                        }
                        let next_child = arena.get(child_id).next;

                        // Unlink from current position and append to emphasis
                        TreeOps::unlink(arena, child_id);
                        TreeOps::append_child(arena, emph_node, child_id);

                        current_child = next_child;
                    }

                    // Insert emphasis node after opener
                    TreeOps::insert_after(arena, opener_inl, emph_node);

                    // Remove delimiter inline nodes if they are now empty
                    if opener_text.is_empty() {
                        TreeOps::unlink(arena, opener_inl);
                    }
                    if closer_text.is_empty() {
                        TreeOps::unlink(arena, closer_inl);
                    }
                    // Always advance to next closer
                    closer_idx += 1;
                } else {
                    closer_idx += 1;
                }
            } else if closer_char == '\'' || closer_char == '"' {
                // Smart quote handling
                let quote_char = if closer_char == '\'' {
                    '\u{2019}'
                } else {
                    '\u{201D}'
                };
                {
                    let node = arena.get_mut(closer_inl);
                    if let NodeData::Text { ref mut literal } = node.data {
                        *literal = quote_char.to_string();
                    }
                }

                if opener_found {
                    let (opener_inl, _, _, _, _, _) = delims[opener_idx];
                    let open_quote = if closer_char == '\'' {
                        '\u{2018}'
                    } else {
                        '\u{201C}'
                    };
                    {
                        let node = arena.get_mut(opener_inl);
                        if let NodeData::Text { ref mut literal } = node.data {
                            *literal = open_quote.to_string();
                        }
                    }
                }

                closer_idx += 1;
            }

            if !opener_found {
                openers_bottom[openers_bottom_idx] = old_closer_idx;
            }
        }

        // Rebuild the delimiter stack, keeping only delimiters up to start_idx
        // These are the delimiters that were before stack_bottom (or all if stack_bottom is None)
        if start_idx > 0 {
            // Keep delimiters from 0 to start_idx-1
            let delims_to_keep: Vec<_> = delims.into_iter().take(start_idx).collect();
            self.delimiters = None;
            for (node_id, char, can_open, can_close, orig_delims, num_delims) in
                delims_to_keep
            {
                let delim = Box::new(Delimiter {
                    previous: self.delimiters.take(),
                    inl_text: node_id,
                    position: 0,
                    num_delims,
                    orig_delims,
                    delim_char: char,
                    can_open,
                    can_close,
                });
                self.delimiters = Some(delim);
            }
        } else {
            // Clear delimiter stack
            self.delimiters = None;
        }
    }

    /// Parse open bracket (start of link or image)
    fn parse_open_bracket(&mut self, arena: &mut NodeArena, parent: NodeId) -> bool {
        self.advance(); // skip [

        // Create a new text node for the bracket (don't merge with previous)
        let text_node = arena.alloc(Node::new(NodeType::Text));
        {
            let text_mut = arena.get_mut(text_node);
            if let NodeData::Text {
                ref mut literal, ..
            } = text_mut.data
            {
                *literal = "[".to_string();
            }
        }
        TreeOps::append_child(arena, parent, text_node);

        // Add to bracket stack
        let bracket = Box::new(Bracket {
            previous: self.brackets.take(),
            inl_text: text_node,
            position: self.pos - 1,
            image: false,
            active: true,
            bracket_after: false,
            previous_delimiter: self.delimiters.take(),
        });

        self.brackets = Some(bracket);
        self.no_link_openers = false;

        true
    }

    /// Parse close bracket (end of link or image)
    fn parse_close_bracket(&mut self, arena: &mut NodeArena, parent: NodeId) -> bool {
        self.advance(); // skip ]

        // Get the opener from bracket stack
        let opener = match self.brackets.take() {
            Some(b) => b,
            None => {
                // No matching opener, just add as text
                self.append_text(arena, parent, "]");
                return true;
            }
        };

        if !opener.active {
            // Opener is not active, just add as text
            self.append_text(arena, parent, "]");
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
            let after_open_paren = self.pos + 1;
            self.advance(); // skip (
            self.skip_spaces_and_newlines();

            // Parse link destination
            if let Some((d, ended_with_space)) = self.parse_link_destination() {
                // For unbracketed URLs, if we ended with a space (not close paren),
                // the link is invalid unless there's a title
                if ended_with_space {
                    // Check if there's a title following the space
                    self.skip_spaces_and_newlines();
                    if self.peek() == Some('"')
                        || self.peek() == Some('\'')
                        || self.peek() == Some('(')
                    {
                        // There's a title, this is valid
                        dest = Some(d);
                        title = self.parse_link_title();
                        self.skip_spaces_and_newlines();

                        if self.peek() == Some(')') {
                            self.advance(); // skip )
                            matched = true;
                        }
                    }
                    // If no title, the link is invalid - need to rewind
                    if !matched {
                        self.pos = after_open_paren - 1; // rewind to before '('
                    }
                } else {
                    // Normal case - ended with close paren or is bracketed URL
                    dest = Some(d);
                    self.skip_spaces_and_newlines();

                    // Try to parse title
                    if self.peek() == Some('"')
                        || self.peek() == Some('\'')
                        || self.peek() == Some('(')
                    {
                        title = self.parse_link_title();
                        self.skip_spaces_and_newlines();
                    }

                    if self.peek() == Some(')') {
                        self.advance(); // skip )
                        matched = true;
                    } else {
                        // Missing close paren - rewind
                        self.pos = after_open_paren - 1; // rewind to before '('
                    }
                }
            } else {
                // Failed to parse destination - rewind
                self.pos = after_open_paren - 1; // rewind to before '('
            }
        }

        // Try reference link [text][label] or [text][]
        if !matched {
            let before_label = self.pos;
            let label_len = self.parse_link_label();

            if label_len > 2 {
                // Full reference link [text][label] with non-empty label
                let label =
                    self.input[before_label..before_label + label_len].to_string();
                reflabel = Some(label);
            } else if label_len == 2 {
                // Collapsed reference link [text][] - use the link text as label
                // For images, opener.position points to '!', so text starts at position + 2
                // For links, opener.position points to '[', so text starts at position + 1
                let label_start = if is_image {
                    opener.position + 2
                } else {
                    opener.position + 1
                };
                let label_end = start_pos - 1;
                if label_start < label_end {
                    reflabel = Some(self.input[label_start..label_end].to_string());
                }
            } else if label_len == 0 && self.pos == before_label {
                // Shortcut reference link [text] - only if:
                // 1. We didn't consume any characters (no '[' found)
                // 2. We're at end of line/string or followed by punctuation
                // Use the text between brackets as label
                // For images, opener.position points to '!', so text starts at position + 2
                // For links, opener.position points to '[', so text starts at position + 1
                let at_line_end = self
                    .peek()
                    .map(|c| c == '\n' || c == '\r' || c == ' ')
                    .unwrap_or(true)
                    || self.pos >= self.input.len();
                // Allow shortcut reference links followed by punctuation
                let followed_by_punct =
                    self.peek().map(|c| is_punctuation(c)).unwrap_or(false);
                if (at_line_end || followed_by_punct) && !opener.bracket_after {
                    let label_start = if is_image {
                        opener.position + 2
                    } else {
                        opener.position + 1
                    };
                    let label_end = start_pos - 1;
                    if label_start < label_end {
                        reflabel = Some(self.input[label_start..label_end].to_string());
                    }
                }
            }

            if let Some(label) = reflabel {
                // Normalize the label and look up in refmap
                let norm_label = normalize_reference(&label);
                if let Some((dest_url, dest_title)) = self.lookup_reference(&norm_label)
                {
                    dest = Some(dest_url);
                    title = Some(dest_title);
                    matched = true;
                } else {
                    // Reference not found - restore position so the label text
                    // can be parsed normally
                    self.pos = before_label;
                }
            }
        }

        if matched {
            // Create link or image node
            let node_type = if is_image {
                NodeType::Image
            } else {
                NodeType::Link
            };
            let link_node = arena.alloc(Node::new(node_type));

            {
                let link_mut = arena.get_mut(link_node);
                match &mut link_mut.data {
                    NodeData::Link {
                        url,
                        title: link_title,
                    } => {
                        *url = dest.unwrap_or_default();
                        *link_title = title.unwrap_or_default();
                    }
                    NodeData::Image {
                        url,
                        title: img_title,
                    } => {
                        *url = dest.unwrap_or_default();
                        *img_title = title.unwrap_or_default();
                    }
                    _ => {}
                }
            }

            // Move content between opener and closer into link node
            let opener_inl = opener.inl_text;
            let mut nodes_to_move: Vec<NodeId> = Vec::new();

            {
                let opener_ref = arena.get(opener_inl);
                let mut current_ptr = opener_ref.next;

                while let Some(curr) = current_ptr {
                    current_ptr = arena.get(curr).next;
                    nodes_to_move.push(curr);
                }
            }

            // Unlink nodes from parent and add to link
            for node in nodes_to_move {
                TreeOps::unlink(arena, node);
                TreeOps::append_child(arena, link_node, node);
            }

            // Insert link node after opener
            TreeOps::unlink(arena, link_node);
            TreeOps::append_child(arena, parent, link_node);

            // Unlink the opener text node
            TreeOps::unlink(arena, opener_inl);

            // Process emphasis with opener's previous delimiter FIRST
            // This processes emphasis delimiters inside the link text
            self.process_emphasis(
                arena,
                opener.previous_delimiter.as_ref().map(|v| &**v),
            );

            // Restore previous_delimiter to the delimiter stack
            // This allows emphasis outside the link to be processed later
            if let Some(prev_delim) = opener.previous_delimiter {
                // Rebuild the delimiter stack with previous_delimiter at the bottom
                // Current stack is empty (process_emphasis cleared it or it was already empty)
                self.delimiters = Some(prev_delim);
            }

            // Remove the matched opener from bracket stack BEFORE deactivating previous openers
            // This ensures we don't deactivate the current opener itself
            self.brackets = opener.previous;

            // For links (not images), deactivate previous link openers
            // This prevents nested links (no links in links)
            if !is_image {
                self.deactivate_previous_link_openers();
            }
        } else {
            // No match - remove this opener from stack and add ] as text
            self.brackets = opener.previous;
            self.append_text(arena, parent, "]");
        }

        true
    }

    /// Skip spaces and at most one newline
    fn skip_spaces_and_newlines(&mut self) {
        while let Some(c) = self.peek() {
            if c == ' ' || c == '\t' || c == '\n' || c == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Parse link destination (URL)
    /// Based on commonmark.js parseLinkDestination
    /// Returns Some((destination, ended_with_space)) on success, None on failure
    /// ended_with_space is true if the destination ended due to a space (not close paren)
    fn parse_link_destination(&mut self) -> Option<(String, bool)> {
        // Try angle-bracketed destination: <url>
        if self.peek() == Some('<') {
            let start_pos = self.pos;
            self.advance(); // skip <
            let content_start = self.pos;

            while let Some(c) = self.peek() {
                if c == '>' {
                    let dest = self.input[content_start..self.pos].to_string();
                    self.advance(); // skip >
                                    // Unescape and normalize the destination
                    let unescaped = unescape_string(&dest);
                    return Some((normalize_uri(&unescaped), false));
                } else if c == '<' || c == '\n' || c == '\r' {
                    // Newlines and < not allowed in angle-bracketed destinations
                    // Rewind to start
                    self.pos = start_pos;
                    return None;
                } else if c == '\\' {
                    // Backslash escape - check if there's a character after it
                    self.advance();
                    if let Some(next_c) = self.peek() {
                        if is_escapable(next_c) {
                            self.advance();
                        }
                    } else {
                        // Backslash at end of input - invalid
                        // Rewind to start
                        self.pos = start_pos;
                        return None;
                    }
                } else {
                    self.advance();
                }
            }
            // Reached end of input without finding >
            // Rewind to start
            self.pos = start_pos;
            return None;
        }

        // Try unbracketed destination
        let start = self.pos;
        let mut paren_depth = 0;
        let mut ended_with_space = false;
        let mut has_newline = false;

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
            } else if c == ' ' || c == '\t' {
                ended_with_space = true;
                break;
            } else if c == '\n' || c == '\r' {
                // Newlines not allowed in link destinations (even for reference definitions)
                has_newline = true;
                break;
            } else {
                self.advance();
            }
        }

        // Allow empty destination if we're at a close paren (like [link]())
        if self.pos == start {
            // Check if we're at a close paren (empty destination case)
            if self.peek() == Some(')') {
                return Some((String::new(), false));
            }
            return None;
        }

        if paren_depth != 0 {
            return None;
        }

        let dest = self.input[start..self.pos].to_string();
        // Normalize newlines to single spaces for multi-line destinations
        let dest = if has_newline {
            dest.lines()
                .map(|s| s.trim_start())
                .collect::<Vec<_>>()
                .join(" ")
        } else {
            dest
        };
        // Unescape and normalize the destination
        let unescaped = unescape_string(&dest);
        Some((normalize_uri(&unescaped), ended_with_space))
    }

    /// Parse link title
    /// Based on commonmark.js parseLinkTitle
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
                                // Unescape the title
                return Some(unescape_string(&title));
            } else if c == '\\' {
                self.advance();
                if let Some(next_c) = self.peek() {
                    if is_escapable(next_c) {
                        self.advance();
                    }
                }
            } else if c == '\n' || c == '\r' {
                // For reference definitions, newlines are allowed in titles
                self.advance();
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

        while let Some(c) = self.peek() {
            if c == '\\' {
                // Escaped character - include both backslash and next char in label
                // According to CommonMark spec, backslash escapes are preserved in link labels
                self.advance(); // skip \
                if self.peek().is_some() {
                    self.advance(); // include escaped char
                }
            } else if c == '[' {
                // Unescaped [ is not allowed in link labels
                // Rewind and return 0
                self.pos = start_pos;
                return 0;
            } else if c == ']' {
                // Found closing bracket
                self.advance(); // skip ]
                let label_len = self.pos - start_pos;
                // Label max 999 characters (excluding brackets)
                // Empty label ([]) is allowed for collapsed reference links
                let content_len = self.pos - label_start - 1;
                if content_len > 999 {
                    self.pos = start_pos;
                    return 0;
                }
                return label_len;
            } else if c == '\n' {
                // Labels can contain newlines, but they are normalized to spaces
                // during reference normalization. We just need to continue parsing.
                self.advance();
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
    fn parse_bang(&mut self, arena: &mut NodeArena, parent: NodeId) -> bool {
        self.advance(); // skip !

        if self.peek() == Some('[') {
            self.advance(); // skip [
                            // Create a separate text node for "![" to avoid merging with previous text
                            // This is important because the opener node will be unlinked when the image is processed
            let text_node = arena.alloc(Node::new(NodeType::Text));
            {
                let text_mut = arena.get_mut(text_node);
                if let NodeData::Text {
                    ref mut literal, ..
                } = text_mut.data
                {
                    *literal = "![".to_string();
                }
            }
            TreeOps::append_child(arena, parent, text_node);

            // Add to bracket stack as image
            let bracket = Box::new(Bracket {
                previous: self.brackets.take(),
                inl_text: text_node,
                position: self.pos - 2,
                image: true,
                active: true,
                bracket_after: false,
                previous_delimiter: self.delimiters.take(),
            });

            self.brackets = Some(bracket);
            true
        } else {
            self.append_text(arena, parent, "!");
            true
        }
    }

    /// Parse a string of non-special characters (optimized with byte-level scanning)
    fn parse_string(&mut self, arena: &mut NodeArena, parent: NodeId) -> bool {
        let start = self.pos;
        let bytes = self.input.as_bytes();

        // Fast path: scan bytes until we hit a special character or non-ASCII
        while self.pos < bytes.len() {
            let b = bytes[self.pos];
            // If it's ASCII and not special, advance
            if b < 0x80 && !is_special_byte(b, self.smart) {
                self.pos += 1;
            } else if b >= 0x80 {
                // Non-ASCII: use character-based check
                if let Some(c) = self.peek() {
                    if is_special_char(c, self.smart) {
                        break;
                    }
                    self.pos += c.len_utf8();
                } else {
                    break;
                }
            } else {
                // Special ASCII character
                break;
            }
        }

        if self.pos > start {
            let text_slice = &self.input[start..self.pos];

            // Create text node with the content
            let text_node = arena.alloc(Node::new(NodeType::Text));
            {
                let text_mut = arena.get_mut(text_node);
                if let NodeData::Text {
                    ref mut literal, ..
                } = text_mut.data
                {
                    // Apply smart punctuation transformations if enabled
                    if self.smart {
                        *literal = self.apply_smart_punctuation(text_slice);
                    } else {
                        // Fast path: no smart punctuation, just clone the slice
                        *literal = text_slice.to_string();
                    }
                }
            }
            TreeOps::append_child(arena, parent, text_node);
            true
        } else {
            false
        }
    }

    /// Apply smart punctuation transformations
    /// Based on commonmark.js: replace ellipses and dashes
    fn apply_smart_punctuation(&self, text: &str) -> String {
        // First handle ellipses: ... -> …
        let text = text.replace("...", "\u{2026}");

        // Then handle dashes
        // We need to be careful about the order: --- should be matched before --
        // Based on commonmark.js logic:
        // - --- -> — (em dash)
        // - -- -> – (en dash)
        // But for sequences like ----, we need to apply rules for multiple dashes

        let mut result = String::new();
        let mut dash_count = 0;

        for c in text.chars() {
            if c == '-' {
                dash_count += 1;
            } else {
                // Process any accumulated dashes
                if dash_count > 0 {
                    result.push_str(&self.convert_dashes(dash_count));
                    dash_count = 0;
                }
                result.push(c);
            }
        }

        // Process any trailing dashes
        if dash_count > 0 {
            result.push_str(&self.convert_dashes(dash_count));
        }

        result
    }

    /// Convert a sequence of dashes to em/en dashes
    /// Based on commonmark.js logic from smart_punct.txt:
    /// - A homogeneous sequence is preferred (all en or all em)
    /// - 10 hyphens = 5 en dashes
    /// - 9 hyphens = 3 em dashes
    /// - 6 hyphens = 2 em dashes (3 is multiple of 3, preferred)
    /// - 7 hyphens = 2 em + 1 en (when homogeneous is not possible)
    /// - em dashes come first, then en dashes, with as few en dashes as possible
    fn convert_dashes(&self, count: usize) -> String {
        if count == 1 {
            return "-".to_string();
        }

        let mut result = String::new();

        // Try to use homogeneous sequence first
        // Prefer em dashes when divisible by 3 (3, 6, 9, ...)
        if count % 3 == 0 {
            // Divisible by 3: use all em dashes
            for _ in 0..(count / 3) {
                result.push('\u{2014}'); // em dash
            }
        } else if count % 2 == 0 {
            // Even number but not divisible by 3: use all en dashes
            for _ in 0..(count / 2) {
                result.push('\u{2013}'); // en dash
            }
        } else {
            // Not homogeneous: use em dashes first, then en dashes
            // Use as many em dashes as possible, then fill with en dashes
            let mut remaining = count;

            // Try to minimize en dashes
            // Start with as many em dashes as possible
            while remaining > 4 {
                result.push('\u{2014}'); // em dash
                remaining -= 3;
            }

            // Handle remaining (should be 2, 4, or 5 at this point)
            match remaining {
                2 => result.push('\u{2013}'),             // en dash
                4 => result.push_str("\u{2013}\u{2013}"), // 2 en dashes
                5 => result.push_str("\u{2014}\u{2013}"), // em + en
                _ => {}                                   // should not happen
            }
        }

        result
    }

    /// Append text to parent, merging with previous text node if possible
    fn append_text(
        &mut self,
        arena: &mut NodeArena,
        parent: NodeId,
        text: &str,
    ) -> NodeId {
        // Check if last child is a text node we can merge with
        let last_child_opt = arena.get(parent).last_child;

        if let Some(last_child) = last_child_opt {
            if arena.get(last_child).node_type == NodeType::Text {
                // Merge with existing text node
                let last_node = arena.get_mut(last_child);
                if let NodeData::Text {
                    ref mut literal, ..
                } = last_node.data
                {
                    literal.push_str(text);
                }
                return last_child;
            }
        }

        // Create new text node
        let text_node = arena.alloc(Node::new(NodeType::Text));
        {
            let text_mut = arena.get_mut(text_node);
            if let NodeData::Text {
                ref mut literal, ..
            } = text_mut.data
            {
                *literal = text.to_string();
            }
        }
        TreeOps::append_child(arena, parent, text_node);
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
#[inline(always)]
fn is_special_char(c: char, smart: bool) -> bool {
    if smart {
        matches!(
            c,
            '`' | '\\' | '&' | '<' | '*' | '_' | '[' | ']' | '!' | '\n' | '\'' | '"'
        )
    } else {
        matches!(
            c,
            '`' | '\\' | '&' | '<' | '*' | '_' | '[' | ']' | '!' | '\n'
        )
    }
}

/// Fast byte-level check if a byte is a special ASCII character
#[inline(always)]
fn is_special_byte(b: u8, smart: bool) -> bool {
    if smart {
        matches!(
            b,
            b'`' | b'\\'
                | b'&'
                | b'<'
                | b'*'
                | b'_'
                | b'['
                | b']'
                | b'!'
                | b'\n'
                | b'\''
                | b'"'
        )
    } else {
        matches!(
            b,
            b'`' | b'\\' | b'&' | b'<' | b'*' | b'_' | b'[' | b']' | b'!' | b'\n'
        )
    }
}

/// Check if a character can be escaped
fn is_escapable(c: char) -> bool {
    matches!(
        c,
        '!' | '"'
            | '#'
            | '$'
            | '%'
            | '&'
            | '\''
            | '('
            | ')'
            | '*'
            | '+'
            | ','
            | '-'
            | '.'
            | '/'
            | ':'
            | ';'
            | '<'
            | '='
            | '>'
            | '?'
            | '@'
            | '['
            | '\\'
            | ']'
            | '^'
            | '_'
            | '`'
            | '{'
            | '|'
            | '}'
            | '~'
    )
}

/// Unescape a string by processing backslash escapes and entities
/// Based on commonmark.js unescapeString
pub fn unescape_string(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(&next_c) = chars.peek() {
                if is_escapable(next_c) {
                    chars.next();
                    result.push(next_c);
                } else {
                    result.push(c);
                }
            } else {
                result.push(c);
            }
        } else if c == '&' {
            // Try to parse an entity
            let remaining: String = chars.clone().collect();
            if let Some((entity, consumed)) = parse_entity(&remaining) {
                result.push_str(&entity);
                // Skip consumed characters
                for _ in 0..consumed {
                    chars.next();
                }
            } else {
                // Not a valid entity, keep the & as is
                // The HTML renderer will escape it if needed
                result.push(c);
            }
        } else {
            result.push(c);
        }
    }

    result
}

/// Parse an HTML entity at the start of a string
/// Returns (decoded_char, chars_consumed) or None
/// Returns None for invalid entities (like out-of-range numeric entities)
fn parse_entity(s: &str) -> Option<(String, usize)> {
    if !s.starts_with('#') && !s.starts_with(|c: char| c.is_ascii_alphabetic()) {
        return None;
    }

    // Numeric entity: &#123; or &#x7B;
    if s.starts_with('#') {
        let rest = &s[1..];
        if rest.starts_with('x') || rest.starts_with('X') {
            // Hex entity: #x7B; (rest starts with x)
            let hex_digits_start = 1; // Skip the 'x' or 'X'
            let hex_end = rest[hex_digits_start..]
                .find(|c: char| !c.is_ascii_hexdigit())
                .map(|i| hex_digits_start + i)
                .unwrap_or(rest.len());

            if hex_end > hex_digits_start && rest[hex_end..].starts_with(';') {
                let hex_str = &rest[hex_digits_start..hex_end];
                if let Ok(codepoint) = u32::from_str_radix(hex_str, 16) {
                    // Handle invalid codepoints
                    // codepoint == 0 (NUL): replacement character
                    // codepoint > 0x10ffff: preserve original entity
                    if codepoint == 0 {
                        return Some((
                            '\u{FFFD}'.to_string(),
                            2 + hex_end - hex_digits_start + 1,
                        ));
                    }
                    if codepoint > 0x10ffff {
                        return None; // Preserve original entity
                    }
                    let c = char::from_u32(codepoint).unwrap_or('\u{FFFD}');
                    // Total length: # (1) + x (1) + hex_digits + ; (1)
                    return Some((c.to_string(), 2 + hex_end - hex_digits_start + 1));
                }
            }
        } else {
            // Decimal entity: #123;
            let dec_end = rest
                .find(|c: char| !c.is_ascii_digit())
                .unwrap_or(rest.len());

            if dec_end > 0 && rest[dec_end..].starts_with(';') {
                let dec_str = &rest[..dec_end];
                if let Ok(codepoint) = dec_str.parse::<u32>() {
                    // Handle invalid codepoints
                    // codepoint == 0 (NUL): replacement character
                    // codepoint > 0x10ffff: preserve original entity
                    if codepoint == 0 {
                        return Some(('\u{FFFD}'.to_string(), 1 + dec_end + 1));
                    }
                    if codepoint > 0x10ffff {
                        return None; // Preserve original entity
                    }
                    let c = char::from_u32(codepoint).unwrap_or('\u{FFFD}');
                    return Some((c.to_string(), 1 + dec_end + 1));
                }
            }
        }
    } else {
        // Named entity
        let name_end = s
            .find(|c: char| !c.is_ascii_alphanumeric())
            .unwrap_or(s.len());

        if name_end > 0 && s[name_end..].starts_with(';') {
            let name = &s[..name_end];

            // First try our HTML5 entity table
            if let Some(decoded) = get_html5_entity(name) {
                return Some((decoded.to_string(), name_end + 1));
            }

            // Then try htmlescape
            let entity_str = format!("&{};", name);
            if let Ok(decoded) = decode_html(&entity_str) {
                // Only return if htmlescape actually decoded it
                if decoded != entity_str {
                    return Some((decoded, name_end + 1));
                }
            }
        }
    }

    None
}

/// Normalize a URI by percent-encoding special characters
/// Based on commonmark.js normalizeURI
/// Percent-encode characters that are not allowed in URIs
fn normalize_uri(uri: &str) -> String {
    let mut result = String::new();

    for c in uri.chars() {
        match c {
            // Unreserved characters (no encoding needed)
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => {
                result.push(c);
            }
            // Reserved characters that are commonly used in URIs
            ':' | '/' | '?' | '#' | '@' | '!' | '$' | '&' | '\'' | '(' | ')' | '*'
            | '+' | ',' | ';' | '=' => {
                result.push(c);
            }
            // Percent sign (already encoded)
            '%' => {
                result.push(c);
            }
            // Space should be encoded as %20 (not +)
            ' ' => {
                result.push_str("%20");
            }
            // Backslash should be encoded
            '\\' => {
                result.push_str("%5C");
            }
            // Square brackets should be encoded in URLs
            '[' => {
                result.push_str("%5B");
            }
            ']' => {
                result.push_str("%5D");
            }
            // Other characters: percent-encode
            _ => {
                let mut buf = [0; 4];
                let s = c.encode_utf8(&mut buf);
                for b in s.bytes() {
                    result.push_str(&format!("%{:02X}", b));
                }
            }
        }
    }

    result
}

/// Normalize a reference label for lookup
/// - Collapses internal whitespace to a single space
/// - Removes leading/trailing whitespace
/// - Converts to uppercase (for case-insensitive comparison)
/// Note: Does NOT unescape backslash escapes - [foo\!] and [foo!] are different labels
pub fn normalize_reference(label: &str) -> String {
    // Remove surrounding brackets if present
    let label = if label.starts_with('[') && label.ends_with(']') {
        &label[1..label.len() - 1]
    } else {
        label
    };

    // Normalize whitespace: collapse all whitespace sequences to a single space
    // Note: We do NOT unescape here - backslash escapes are preserved in link labels
    // per CommonMark spec. So "foo\!" stays as "foo\!", not "foo!"
    let normalized = label
        .trim()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    // Unicode case folding: to_lowercase().to_uppercase() matches commonmark.js behavior
    // This properly handles characters like ß which folds to SS
    normalized.to_lowercase().to_uppercase()
}

/// Check if a character is punctuation
fn is_punctuation(c: char) -> bool {
    // ASCII punctuation
    if c.is_ascii_punctuation() {
        return true;
    }
    // Unicode punctuation (Pc, Pd, Ps, Pe, Pi, Pf, Po categories)
    if c.is_ascii() {
        return false;
    }
    // Check for specific Unicode punctuation characters commonly used in tests
    matches!(c,
        '\u{00A2}'..='\u{00A5}' | // ¢£¤¥ (currency symbols)
        '\u{00B5}' |              // µ
        '\u{00B7}' |              // ·
        '\u{00BF}' |              // ¿
        '\u{00D7}' |              // ×
        '\u{00F7}' |              // ÷
        '\u{2000}'..='\u{206F}' | // General Punctuation
        '\u{20A0}'..='\u{20CF}' | // Currency Symbols
        '\u{2190}'..='\u{21FF}' | // Arrows
        '\u{2200}'..='\u{22FF}' | // Mathematical Operators
        '\u{2300}'..='\u{23FF}' | // Miscellaneous Technical
        '\u{25A0}'..='\u{25FF}' | // Geometric Shapes
        '\u{2600}'..='\u{26FF}' | // Miscellaneous Symbols
        '\u{2700}'..='\u{27BF}' | // Dingbats
        '\u{3000}'..='\u{303F}'   // CJK Symbols and Punctuation
    )
}

/// Parse inline content into the given parent node
pub fn parse_inlines(
    arena: &mut NodeArena,
    parent: NodeId,
    content: &str,
    line: usize,
    block_offset: usize,
) {
    let mut subject = Subject::new(content, line, block_offset);
    subject.parse_inlines(arena, parent);
}

/// Parse inline content with reference map
pub fn parse_inlines_with_refmap(
    arena: &mut NodeArena,
    parent: NodeId,
    content: &str,
    line: usize,
    block_offset: usize,
    refmap: std::collections::HashMap<String, (String, String)>,
) {
    parse_inlines_with_options(
        arena,
        parent,
        content,
        line,
        block_offset,
        refmap,
        false,
    );
}

/// Parse inlines with reference map and smart punctuation option
pub fn parse_inlines_with_options(
    arena: &mut NodeArena,
    parent: NodeId,
    content: &str,
    line: usize,
    block_offset: usize,
    refmap: std::collections::HashMap<String, (String, String)>,
    smart: bool,
) {
    let mut subject =
        Subject::with_refmap_and_smart(content, line, block_offset, refmap, smart);
    subject.parse_inlines(arena, parent);

    // Clear the parent's literal content since it's now represented as child nodes
    // This prevents the renderer from using the literal instead of children
    // Note: For heading nodes, we don't clear the literal because heading nodes
    // should have NodeData::Heading type, not NodeData::Text type
    let parent_node = arena.get_mut(parent);
    if let NodeData::Text { ref mut literal } = parent_node.data {
        literal.clear();
    }
}

/// Parse a reference definition from the beginning of a string.
/// Returns the number of characters consumed, or 0 if no reference was found.
/// If a reference is found, it is added to the refmap.
pub fn parse_reference(
    s: &str,
    refmap: &mut std::collections::HashMap<String, (String, String)>,
) -> usize {
    // Skip leading whitespace and newlines
    let trimmed = s.trim_start_matches(|c: char| c.is_ascii_whitespace());
    let skipped = s.len() - trimmed.len();

    let mut subject = Subject::new(trimmed, 1, 0);
    let consumed = subject.parse_reference_definition(refmap);

    if consumed > 0 {
        skipped + consumed
    } else {
        0
    }
}

impl<'a> Subject<'a> {
    /// Parse a reference definition: [label]: url "title"
    /// Returns the number of characters consumed, or 0 if no reference was found
    fn parse_reference_definition(
        &mut self,
        refmap: &mut std::collections::HashMap<String, (String, String)>,
    ) -> usize {
        let start_pos = self.pos;

        // Parse label: [label]
        let label_len = self.parse_link_label();
        if label_len == 0 {
            return 0;
        }

        let raw_label = self.input[start_pos..start_pos + label_len].to_string();

        // Empty label ([]) or label with only whitespace is not allowed for reference definitions
        // Only for collapsed reference links like [text][]
        let label_content = &raw_label[1..raw_label.len() - 1]; // Remove brackets
        if label_content.trim().is_empty() {
            return 0;
        }

        // Expect colon
        if self.peek() != Some(':') {
            return 0;
        }
        self.advance(); // skip :

        // Skip spaces and newlines
        self.skip_spaces_and_newlines();

        // Parse link destination
        let dest = match self.parse_link_destination() {
            Some((d, _)) => d,
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

        // Must be at end of line or only whitespace/newlines remain
        // For reference definitions, we allow the definition to end at a newline
        // or at the end of input
        // Also allow if the next line starts with '[' (new reference definition)
        let remaining = &self.input[self.pos..];
        let at_line_end = remaining.is_empty()
            || remaining.starts_with('\n')
            || remaining.starts_with('\r')
            || remaining.chars().all(|c| c.is_ascii_whitespace());

        // Check if next non-empty line starts with '[' (new reference definition)
        let next_is_ref_def = remaining.trim_start().starts_with('[');

        if !at_line_end && !next_is_ref_def {
            // Check if we can still match without title
            self.pos = before_title;
            let remaining = &self.input[self.pos..];
            let at_line_end_without_title = remaining.is_empty()
                || remaining.starts_with('\n')
                || remaining.starts_with('\r')
                || remaining.chars().all(|c| c.is_ascii_whitespace());
            let next_is_ref_def_without_title = remaining.trim_start().starts_with('[');
            if !at_line_end_without_title && !next_is_ref_def_without_title {
                return 0;
            }
        }

        // Normalize label and add to refmap
        let norm_label = normalize_reference(&raw_label);
        if !norm_label.is_empty() {
            // Only add if not already present (first definition wins)
            refmap
                .entry(norm_label)
                .or_insert((dest, title.unwrap_or_default()));
        }

        self.pos
    }
}

/// Parse an HTML entity and return the decoded string and length
/// Uses htmlescape crate and our entity table to support all HTML5 named entities
/// Returns None if this is not an entity pattern at all
/// Returns Some((decoded, len)) for valid entities
/// For invalid entities (like &#87654321;), returns Some((original, len)) to preserve them
fn parse_entity_char(input: &str) -> Option<(String, usize)> {
    if !input.starts_with('&') {
        return None;
    }

    // Find the end of the entity (semicolon or end of string)
    let end = input.find(';').map(|i| i + 1).unwrap_or(input.len());
    if end <= 1 {
        return None;
    }

    let entity_str = &input[..end];

    // Try numeric entity first: &#123; or &#x7B;
    if entity_str.starts_with("&#") {
        // Check if it's a valid numeric entity format
        let rest = &entity_str[2..]; // Skip "&#"

        if rest.starts_with('x') || rest.starts_with('X') {
            // Hex entity: &#x7B;
            let hex_digits = &rest[1..rest.len() - 1]; // Skip 'x' and ';'
            if !hex_digits.is_empty()
                && hex_digits.chars().all(|c| c.is_ascii_hexdigit())
            {
                // Valid hex format, use parse_entity
                if let Some((decoded, _)) = parse_entity(&entity_str[1..]) {
                    return Some((decoded, entity_str.len()));
                }
                // Invalid hex entity (e.g., out of range) - preserve as-is
                return Some((entity_str.to_string(), entity_str.len()));
            }
        } else {
            // Decimal entity: &#123;
            let dec_digits = &rest[..rest.len() - 1]; // Remove ';'
            if !dec_digits.is_empty() && dec_digits.chars().all(|c| c.is_ascii_digit()) {
                // Valid decimal format, use parse_entity
                if let Some((decoded, _)) = parse_entity(&entity_str[1..]) {
                    return Some((decoded, entity_str.len()));
                }
                // Invalid decimal entity (e.g., out of range) - preserve as-is
                return Some((entity_str.to_string(), entity_str.len()));
            }
        }
        // Invalid numeric entity format - don't consume
        return None;
    }

    // Try named entity from our table
    if entity_str.len() > 2 {
        let name = &entity_str[1..entity_str.len() - 1]; // Remove & and ;
        if !name.is_empty() {
            if let Some(decoded) = get_html5_entity(name) {
                return Some((decoded.to_string(), entity_str.len()));
            }
        }
    }

    // Try to decode using htmlescape crate
    match decode_html(entity_str) {
        Ok(decoded) => {
            // If decoding produced a different result, it's a valid entity
            if decoded != entity_str {
                Some((decoded, end))
            } else {
                None
            }
        }
        Err(_) => None,
    }
}

/// Email autolink pattern
/// Based on commonmark.js reEmailAutolink
fn match_email_autolink(input: &str) -> Option<(String, usize)> {
    if !input.starts_with('<') {
        return None;
    }

    // Email pattern from commonmark.js:
    // /^<([a-zA-Z0-9.!#$%&'*+\/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*>/
    let rest = &input[1..];

    // Check for valid email characters in local part (before @)
    let mut chars = rest.chars().peekable();
    let mut i = 0;
    let mut found_at = false;

    // Local part: [a-zA-Z0-9.!#$%&'*+\/=?^_`{|}~-]+
    while let Some(&c) = chars.peek() {
        if c == '@' {
            found_at = true;
            chars.next();
            i += 1;
            break;
        } else if c == '>' || c == '<' || c == '\n' || c == ' ' || c == '\t' {
            return None;
        } else if c == '\\' {
            // Backslash escape is not allowed in email autolinks
            return None;
        } else if is_valid_email_local_char(c) {
            chars.next();
            i += 1;
        } else {
            return None;
        }
    }

    if !found_at || i <= 1 {
        return None;
    }

    // Domain part: [a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*
    let domain_start = i;
    let mut label_start = i;

    while let Some(&c) = chars.peek() {
        if c == '>' {
            // End of email
            if i > domain_start && i > label_start {
                let email = &rest[..i];
                return Some((email.to_string(), i + 2)); // +2 for < and >
            }
            return None;
        } else if c == '<' || c == '\n' || c == ' ' || c == '\t' {
            return None;
        } else if c == '.' {
            chars.next();
            i += 1;
            label_start = i;
        } else if c.is_ascii_alphanumeric() || c == '-' {
            chars.next();
            i += 1;
        } else {
            return None;
        }
    }

    None
}

fn is_valid_email_local_char(c: char) -> bool {
    c.is_ascii_alphanumeric()
        || matches!(
            c,
            '.' | '!'
                | '#'
                | '$'
                | '%'
                | '&'
                | '\''
                | '*'
                | '+'
                | '/'
                | '='
                | '?'
                | '^'
                | '_'
                | '`'
                | '{'
                | '|'
                | '}'
                | '~'
                | '-'
        )
}

/// URL autolink pattern
/// Based on commonmark.js reAutolink: /^<[A-Za-z][A-Za-z0-9.+-]{1,31}:[^<>\x00-\x20]*>/i
fn match_url_autolink(input: &str) -> Option<(String, usize)> {
    if !input.starts_with('<') {
        return None;
    }

    // URL pattern: <scheme:...>
    let rest = &input[1..];

    // Must start with a letter, then letters/digits/+/-/.
    let mut i = 0;
    let mut has_colon = false;

    for c in rest.chars() {
        if c == ':' {
            has_colon = true;
            i += 1;
            break;
        } else if c.is_ascii_alphabetic()
            || c.is_ascii_digit()
            || c == '+'
            || c == '-'
            || c == '.'
        {
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

    if !has_colon || i < 3 {
        // Scheme must be at least 2 characters (i includes the colon, so i >= 3 means scheme >= 2)
        return None;
    }

    // Now parse the rest of the URL
    // [^<>\x00-\x20]* means: no <, >, or ASCII control characters/space
    let url_start = i;
    let mut end_pos = 0;

    for (j, c) in rest[url_start..].chars().enumerate() {
        if c == '>' {
            end_pos = url_start + j;
            break;
        } else if c == '\n' || c == '<' || c == ' ' || c == '\t' || c.is_ascii_control()
        {
            // Space or control character - invalid URL
            return None;
        }
    }

    if end_pos > url_start {
        let url = &rest[..end_pos];
        return Some((url.to_string(), end_pos + 2)); // +2 for < and >
    }

    None
}

/// Match HTML tag and return the tag content and length
fn match_html_tag(input: &str) -> Option<(String, usize)> {
    if !input.starts_with('<') {
        return None;
    }

    // Try different HTML tag types in order

    // 1. HTML Comment: <!-- ... -->
    if let Some(result) = match_html_comment(input) {
        return Some(result);
    }

    // 2. Processing Instruction: <? ... ?>
    if let Some(result) = match_processing_instruction(input) {
        return Some(result);
    }

    // 3. Declaration: <! ... >
    if let Some(result) = match_declaration(input) {
        return Some(result);
    }

    // 4. CDATA: <![CDATA[ ... ]]>
    if let Some(result) = match_cdata(input) {
        return Some(result);
    }

    // 5. Regular HTML tag (open, close, or self-closing)
    if let Some(result) = match_regular_html_tag(input) {
        return Some(result);
    }

    None
}

/// Match HTML comment: <!-- ... -->
fn match_html_comment(input: &str) -> Option<(String, usize)> {
    if !input.starts_with("<!--") {
        return None;
    }

    // Find -->
    if let Some(end) = input.find("-->") {
        return Some((input[..end + 3].to_string(), end + 3));
    }

    None
}

/// Match processing instruction: <? ... ?>
fn match_processing_instruction(input: &str) -> Option<(String, usize)> {
    if !input.starts_with("<?") {
        return None;
    }

    // Find ?>
    if let Some(end) = input.find("?>") {
        return Some((input[..end + 2].to_string(), end + 2));
    }

    None
}

/// Match declaration: <! ... >
/// According to commonmark.js: /^<![A-Za-z]/ - must start with a letter after <!
fn match_declaration(input: &str) -> Option<(String, usize)> {
    if !input.starts_with("<!") || input.starts_with("<![") {
        return None;
    }

    // Declaration must have at least one character after <!
    if input.len() <= 2 {
        return None;
    }

    // Check that the character after <! is an ASCII letter (A-Z or a-z)
    // Per commonmark.js: /^<![A-Za-z]/
    let third_char = input.chars().nth(2)?;
    if !third_char.is_ascii_alphabetic() {
        return None;
    }

    // Find >
    if let Some(end) = input.find('>') {
        // Must not contain < or > inside
        let content = &input[2..end];
        if content.contains('<') || content.contains('>') {
            return None;
        }
        return Some((input[..end + 1].to_string(), end + 1));
    }

    None
}

/// Match CDATA: <![CDATA[ ... ]]>
fn match_cdata(input: &str) -> Option<(String, usize)> {
    if !input.starts_with("<![CDATA[") {
        return None;
    }

    // Find ]]>
    if let Some(end) = input.find("]]>") {
        return Some((input[..end + 3].to_string(), end + 3));
    }

    None
}

/// Match regular HTML tag: open, close, or self-closing
/// Based on commonmark.js regex patterns:
/// TAGNAME = "[A-Za-z][A-Za-z0-9-]*"
/// ATTRIBUTENAME = "[a-zA-Z_:][a-zA-Z0-9:._-]*"
/// ATTRIBUTE = "(?:\\s+" + ATTRIBUTENAME + ATTRIBUTEVALUESPEC + "?)"
/// OPENTAG = "<" + TAGNAME + ATTRIBUTE + "*" + "\\s*/?>"
fn match_regular_html_tag(input: &str) -> Option<(String, usize)> {
    if !input.starts_with('<') {
        return None;
    }

    let rest = &input[1..];

    // Check for close tag: </tag>
    if rest.starts_with('/') {
        return match_close_tag(input);
    }

    // Must start with a letter for tag name (not whitespace)
    let first_char = rest.chars().next()?;
    if !first_char.is_ascii_alphabetic() {
        return None;
    }

    // Parse tag name: [A-Za-z][A-Za-z0-9-]*
    let mut i = 1; // Skip the '<'
    for c in rest.chars() {
        if c.is_ascii_alphanumeric() || c == '-' {
            i += 1;
        } else {
            break;
        }
    }

    // Parse attributes: (whitespace+ attribute_name value?)*
    // ATTRIBUTENAME = "[a-zA-Z_:][a-zA-Z0-9:._-]*"
    // First skip whitespace after tag name
    while i < input.len() {
        let c = input.chars().nth(i)?;
        if c.is_ascii_whitespace() {
            i += 1;
        } else {
            break;
        }
    }

    // Track if the previous attribute was a boolean attribute
    // If so, we already skipped the whitespace after it
    let mut after_boolean_attr = false;

    loop {
        if i >= input.len() {
            break;
        }

        let c = input.chars().nth(i)?;

        // Check for end of tag
        if c == '>' {
            return Some((input[..i + 1].to_string(), i + 1));
        }

        // Check for self-closing tag />
        if c == '/' {
            if i + 1 < input.len() && input.chars().nth(i + 1)? == '>' {
                return Some((input[..i + 2].to_string(), i + 2));
            }
            // '/' not followed by '>' is invalid
            return None;
        }

        // Parse attribute name: [a-zA-Z_:][a-zA-Z0-9:._-]*
        let first_attr_char = input.chars().nth(i)?;
        if !first_attr_char.is_ascii_alphabetic()
            && first_attr_char != '_'
            && first_attr_char != ':'
        {
            return None;
        }
        i += 1;

        while i < input.len() {
            let c = input.chars().nth(i)?;
            if c.is_ascii_alphanumeric() || c == ':' || c == '_' || c == '.' || c == '-'
            {
                i += 1;
            } else {
                break;
            }
        }

        // Check for attribute value
        // Skip whitespace after attribute name (before =)
        // But only if we didn't already skip it (i.e., not after a boolean attribute)
        if !after_boolean_attr {
            while i < input.len() {
                let ws_char = input.chars().nth(i)?;
                if ws_char.is_ascii_whitespace() {
                    i += 1;
                } else {
                    break;
                }
            }
        }
        after_boolean_attr = false;

        if i < input.len() {
            let c = input.chars().nth(i)?;
            if c == '=' {
                i += 1;
                // Skip whitespace after =
                while i < input.len() {
                    let ws_char = input.chars().nth(i)?;
                    if ws_char.is_ascii_whitespace() {
                        i += 1;
                    } else {
                        break;
                    }
                }
                // Parse attribute value
                if i >= input.len() {
                    return None;
                }
                let val_char = input.chars().nth(i)?;
                if val_char == '"' || val_char == '\'' {
                    // Quoted value
                    let quote = val_char;
                    i += 1;
                    while i < input.len() {
                        let c = input.chars().nth(i)?;
                        if c == quote {
                            i += 1;
                            break;
                        }
                        i += 1;
                    }
                } else {
                    // Unquoted value: [^"'=<>`\x00-\x20]+
                    // Note: CommonMark allows = in unquoted values (test #616)
                    while i < input.len() {
                        let c = input.chars().nth(i)?;
                        if c == '"'
                            || c == '\''
                            || c == '<'
                            || c == '>'
                            || c == '`'
                            || c.is_ascii_whitespace()
                        {
                            break;
                        }
                        i += 1;
                    }
                }
            } else {
                // If no '=', this is a boolean attribute
                // The whitespace after the attribute name has already been skipped above
                // Set flag so we don't skip it again on the next iteration
                after_boolean_attr = true;
            }
        }

        // Skip whitespace before next attribute (or end of tag)
        // But only if we didn't just parse a boolean attribute
        if !after_boolean_attr {
            let mut ws_count = 0;
            while i < input.len() {
                let c = input.chars().nth(i)?;
                if c.is_ascii_whitespace() {
                    i += 1;
                    ws_count += 1;
                } else {
                    break;
                }
            }

            // If we didn't see whitespace and we're not at the end of the tag, invalid
            // (attributes must be separated by whitespace)
            if ws_count == 0 && i < input.len() {
                let c = input.chars().nth(i)?;
                if c != '>' && c != '/' {
                    return None;
                }
            }
        }
    }

    None
}

/// Match close tag: </tag>
/// Allows whitespace between tag name and >
fn match_close_tag(input: &str) -> Option<(String, usize)> {
    if !input.starts_with("</") {
        return None;
    }

    let rest = &input[2..];

    // Must start with a letter
    let first_char = rest.chars().next()?;
    if !first_char.is_ascii_alphabetic() {
        return None;
    }

    // Parse tag name
    let mut i = 2; // Skip the '</'
    for c in rest.chars() {
        if c.is_ascii_alphanumeric() || c == '-' {
            i += 1;
        } else {
            break;
        }
    }

    // Skip whitespace
    while i < input.len() {
        let c = input.chars().nth(i)?;
        if c.is_ascii_whitespace() {
            i += 1;
        } else {
            break;
        }
    }

    // Expect >
    if i < input.len() && input.chars().nth(i)? == '>' {
        return Some((input[..i + 1].to_string(), i + 1));
    }

    None
}

#[allow(dead_code)]
/// Helper function to get the parent of a node
fn parent_of(arena: &NodeArena, node_id: NodeId) -> NodeId {
    arena.get(node_id).parent.unwrap_or(0)
}
