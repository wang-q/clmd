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
//! ```ignore
//! use clmd::{Arena, parse_document, format_html, parser::options::Options};
//!
//! let mut arena = Arena::new();
//! let options = Options::default();
//! let doc = parse_document(&mut arena, "Hello *world*", &options);
//! let mut html = String::new();
//! format_html(&arena, doc, &options, &mut html).unwrap();
//! assert!(html.contains("<em>world</em>"));
//! ```

mod autolinks;
mod emphasis;
pub mod entities;
mod html_tags;
mod links;
mod text;
mod utils;

use crate::arena::{Node, NodeArena, NodeId, TreeOps};
use crate::nodes::{NodeCode, NodeLink, NodeValue};
use autolinks::{match_email_autolink, match_url_autolink};
use emphasis::{
    process_emphasis, remove_delimiters_inside_link, scan_delims, Delimiter,
};
use entities::parse_entity_char;
use html_tags::match_html_tag;
use links::{
    create_link_node, move_nodes_to_link, parse_inline_link, parse_reference_definition,
    parse_reference_link, Bracket, LinkContext,
};
use rustc_hash::FxHashMap;
use std::borrow::Cow;
use text::apply_smart_punctuation;
use utils::normalize_uri;

/// Subject represents the string being parsed and tracks position
#[derive(Debug)]
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
    /// Reference map for link references (borrowed to avoid cloning)
    pub refmap: &'a FxHashMap<String, (String, String)>,
    /// Whether smart punctuation is enabled
    pub smart: bool,
}

/// Static empty refmap for Subject::new
static EMPTY_REFMAP: once_cell::sync::Lazy<FxHashMap<String, (String, String)>> =
    once_cell::sync::Lazy::new(FxHashMap::default);

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
            refmap: &EMPTY_REFMAP,
            smart: false,
        }
    }

    /// Create a new subject with a reference map
    pub fn with_refmap(
        input: &'a str,
        line: usize,
        block_offset: usize,
        refmap: &'a FxHashMap<String, (String, String)>,
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
        refmap: &'a FxHashMap<String, (String, String)>,
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
            // For UTF-8 multi-byte sequences, use char::len_utf8 for correctness
            self.pos += if b < 0x80 {
                1
            } else {
                // Use chars().next() to correctly calculate UTF-8 character length
                self.input[self.pos..]
                    .chars()
                    .next()
                    .map_or(1, |c| c.len_utf8())
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
        process_emphasis(arena, &mut self.delimiters, None);

        // Merge adjacent text nodes for cleaner output
        Self::merge_adjacent_text_nodes(arena, parent);

        // Remove trailing spaces from the last text node
        Self::remove_trailing_spaces(arena, parent);
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
            let line_break = arena.alloc(Node::with_value(NodeValue::HardBreak));
            TreeOps::append_child(arena, parent, line_break);
        } else {
            // Soft line break
            let soft_break = arena.alloc(Node::with_value(NodeValue::SoftBreak));
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
            if let NodeValue::Text(ref literal) = arena.get(last_child).value {
                let new_len = literal.len().saturating_sub(count);
                let new_literal = literal[..new_len].to_string();
                let node = arena.get_mut(last_child);
                if let NodeValue::Text(ref mut literal) = node.value {
                    *literal = new_literal.into_boxed_str();
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

        // Count opening backticks (no allocation)
        let tick_len = self.count_char('`');
        let after_open_ticks = self.pos;

        // Look for closing backticks
        loop {
            if self.end() {
                break;
            }

            if self.peek() == Some('`') {
                let close_start = self.pos;
                let close_len = self.count_char('`');

                if close_len == tick_len {
                    // Found matching close
                    let content = &self.input[after_open_ticks..close_start];

                    // Build content without allocation for simple cases
                    let content: String = if content.contains('\n') {
                        // Replace newlines with spaces
                        // Pre-allocate with same capacity to avoid reallocations
                        let mut result = String::with_capacity(content.len());
                        for c in content.chars() {
                            result.push(if c == '\n' { ' ' } else { c });
                        }
                        result
                    } else {
                        // No newlines, use the content directly without allocation
                        content.to_string()
                    };

                    // Trim single leading/trailing space if both exist
                    let content = if content.len() >= 2
                        && content.as_bytes().first() == Some(&b' ')
                        && content.as_bytes().last() == Some(&b' ')
                        && !content.trim().is_empty()
                    {
                        content[1..content.len() - 1].to_string()
                    } else {
                        content
                    };

                    let code_node =
                        arena.alloc(Node::with_value(NodeValue::code(NodeCode {
                            num_backticks: tick_len,
                            literal: content,
                        })));
                    TreeOps::append_child(arena, parent, code_node);
                    return true;
                }
            } else {
                self.advance();
            }
        }

        // No matching close found, treat as literal
        self.pos = after_open_ticks;
        // For small number of backticks, avoid allocation with repeat()
        let backticks: &str = match tick_len {
            1 => "`",
            2 => "``",
            3 => "```",
            4 => "````",
            5 => "`````",
            6 => "``````",
            7 => "```````",
            8 => "````````",
            _ => {
                // For larger numbers, fall back to repeat
                self.append_text(arena, parent, &"`".repeat(tick_len));
                return true;
            }
        };
        self.append_text(arena, parent, backticks);
        true
    }

    /// Count consecutive occurrences of a character
    fn count_char(&mut self, ch: char) -> usize {
        let mut count = 0;
        while self.peek() == Some(ch) {
            count += 1;
            self.advance();
        }
        count
    }

    /// Parse backslash escape or hard line break
    fn parse_backslash(&mut self, arena: &mut NodeArena, parent: NodeId) -> bool {
        use utils::is_escapable;

        self.advance(); // skip backslash

        if self.peek() == Some('\n') {
            // Hard line break
            self.advance();
            let line_break = arena.alloc(Node::with_value(NodeValue::HardBreak));
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
                let text_node = arena.alloc(Node::with_value(NodeValue::make_text("<")));
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
        let text_node = arena.alloc(Node::with_value(NodeValue::make_text("<")));
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
            let link_node = arena.alloc(Node::with_value(NodeValue::link(NodeLink {
                url: normalize_uri(&format!("mailto:{}", email)),
                title: String::new(),
            })));

            // Add text content
            let text_node =
                arena.alloc(Node::with_value(NodeValue::make_text(email.as_str())));
            TreeOps::append_child(arena, link_node, text_node);
            TreeOps::append_child(arena, parent, link_node);

            self.pos += len;
            return true;
        }

        // Try URL autolink
        if let Some((url, len)) = match_url_autolink(remaining) {
            let link_node = arena.alloc(Node::with_value(NodeValue::link(NodeLink {
                url: normalize_uri(&url),
                title: String::new(),
            })));

            // Add text content
            let text_node =
                arena.alloc(Node::with_value(NodeValue::make_text(url.as_str())));
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
            let html_node = arena.alloc(Node::with_value(NodeValue::HtmlInline(
                tag_content.into_boxed_str(),
            )));
            TreeOps::append_child(arena, parent, html_node);
            self.pos += len;
            return true;
        }

        false
    }

    /// Handle delimiter character (* or _)
    fn handle_delim(&mut self, arena: &mut NodeArena, c: char, parent: NodeId) -> bool {
        let start_pos = self.pos;
        let (res, new_pos) = scan_delims(self.input, self.pos, c);
        self.pos = new_pos;

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
            std::iter::repeat_n(c, res.num_delims).collect()
        };
        let text_node =
            arena.alloc(Node::with_value(NodeValue::Text(delim_text.into())));
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

        true
    }

    /// Parse open bracket (start of link or image)
    fn parse_open_bracket(&mut self, arena: &mut NodeArena, parent: NodeId) -> bool {
        self.advance(); // skip [

        // Create a new text node for the bracket (don't merge with previous)
        let text_node = arena.alloc(Node::with_value(NodeValue::Text("[".into())));
        TreeOps::append_child(arena, parent, text_node);

        // Add to bracket stack
        // Record the current top delimiter as the previous delimiter for this bracket
        let previous_delimiter_marker = self
            .delimiters
            .as_ref()
            .map(|d| (d.inl_text, d.orig_delims));
        let bracket = Box::new(Bracket {
            previous: self.brackets.take(),
            inl_text: text_node,
            position: self.pos - 1,
            image: false,
            active: true,
            bracket_after: false,
            previous_delimiter_marker,
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

        // Create link context for parsing
        let mut ctx = LinkContext::new(self.input, self.pos, self.refmap);

        // Try inline link: [text](url "title")
        if let Some((d, t, consumed)) = parse_inline_link(&mut ctx) {
            dest = Some(d);
            title = Some(t);
            self.pos = start_pos + consumed;
            matched = true;
        }

        // Try reference link [text][label] or [text][]
        // closer_position is start_pos - 1 (position of the closing bracket)
        if !matched {
            let closer_position = start_pos.saturating_sub(1);
            if let Some((d, t, consumed)) = parse_reference_link(
                &mut ctx,
                opener.position,
                closer_position,
                is_image,
            ) {
                dest = Some(d);
                title = Some(t);
                self.pos = start_pos + consumed;
                matched = true;
            }
        }

        if matched {
            // Create link or image node
            let link_node = create_link_node(
                arena,
                is_image,
                dest.unwrap_or_default(),
                title.unwrap_or_default(),
            );

            // Move content between opener and closer into link node
            move_nodes_to_link(arena, link_node, opener.inl_text, parent);

            // Process emphasis with opener's previous delimiter FIRST
            // This processes emphasis delimiters inside the link text
            // The previous_delimiter_marker identifies which delimiter was on the stack
            // before this bracket, so we only process delimiters added inside the link text
            process_emphasis(
                arena,
                &mut self.delimiters,
                opener.previous_delimiter_marker,
            );

            // Remove delimiters that are inside the link from the delimiter stack
            remove_delimiters_inside_link(
                &mut self.delimiters,
                opener.previous_delimiter_marker,
            );

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

    /// Parse bang (!, could be start of image)
    fn parse_bang(&mut self, arena: &mut NodeArena, parent: NodeId) -> bool {
        self.advance(); // skip !

        if self.peek() == Some('[') {
            self.advance(); // skip [
                            // Create a separate text node for "![" to avoid merging with previous text
                            // This is important because the opener node will be unlinked when the image is processed
            let text_node = arena.alloc(Node::with_value(NodeValue::Text("![".into())));
            TreeOps::append_child(arena, parent, text_node);

            // Add to bracket stack as image
            // Record the current top delimiter as the previous delimiter for this bracket
            let previous_delimiter_marker = self
                .delimiters
                .as_ref()
                .map(|d| (d.inl_text, d.orig_delims));
            let bracket = Box::new(Bracket {
                previous: self.brackets.take(),
                inl_text: text_node,
                position: self.pos - 2,
                image: true,
                active: true,
                bracket_after: false,
                previous_delimiter_marker,
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
        use utils::is_special_byte;

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
                    if utils::is_special_char(c, self.smart) {
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
            let literal = if self.smart {
                apply_smart_punctuation(text_slice)
            } else {
                text_slice.to_string()
            };
            let text_node =
                arena.alloc(Node::with_value(NodeValue::Text(literal.into())));
            TreeOps::append_child(arena, parent, text_node);
            true
        } else {
            false
        }
    }

    /// Remove trailing spaces from the last text node
    fn remove_trailing_spaces(arena: &mut NodeArena, parent: NodeId) {
        let parent_is_heading =
            matches!(arena.get(parent).value, NodeValue::Heading(..));

        // For headings, always remove trailing spaces
        if parent_is_heading {
            if let Some(last_child) = arena.get(parent).last_child {
                if let NodeValue::Text(ref literal) = arena.get(last_child).value {
                    let trimmed = literal.trim_end_matches(' ').to_string();
                    if trimmed != literal.as_ref() {
                        let node = arena.get_mut(last_child);
                        if let NodeValue::Text(ref mut literal) = node.value {
                            *literal = trimmed.into();
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
                let current_is_text =
                    matches!(arena.get(current).value, NodeValue::Text(..));
                let next_is_text = matches!(arena.get(next).value, NodeValue::Text(..));

                if current_is_text && next_is_text {
                    // Check if either is a smart quote without cloning
                    let can_merge = {
                        let current_literal = match &arena.get(current).value {
                            NodeValue::Text(literal) => literal.as_ref(),
                            _ => "",
                        };
                        let next_literal = match &arena.get(next).value {
                            NodeValue::Text(literal) => literal.as_ref(),
                            _ => "",
                        };
                        !Self::is_smart_quote(current_literal)
                            && !Self::is_smart_quote(next_literal)
                    };

                    if can_merge {
                        // Get next's literal and merge into current
                        let next_literal: Box<str> = {
                            match &arena.get(next).value {
                                NodeValue::Text(literal) => literal.clone(),
                                _ => "".into(),
                            }
                        };

                        {
                            let current_node = arena.get_mut(current);
                            if let NodeValue::Text(ref mut literal) = current_node.value
                            {
                                *literal = format!(
                                    "{}{}",
                                    literal.as_ref(),
                                    next_literal.as_ref()
                                )
                                .into_boxed_str();
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
            if matches!(arena.get(last_child).value, NodeValue::Text(..)) {
                // Merge with existing text node
                let last_node = arena.get_mut(last_child);
                if let NodeValue::Text(ref mut literal) = last_node.value {
                    *literal = format!("{}{}", literal.as_ref(), text).into();
                }
                return last_child;
            }
        }

        // Create new text node
        let text_node =
            arena.alloc(Node::with_value(NodeValue::Text(text.to_string().into())));
        TreeOps::append_child(arena, parent, text_node);
        text_node
    }
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
    refmap: &FxHashMap<String, (String, String)>,
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
    refmap: &FxHashMap<String, (String, String)>,
    smart: bool,
) {
    let mut subject =
        Subject::with_refmap_and_smart(content, line, block_offset, refmap, smart);
    subject.parse_inlines(arena, parent);

    // Clear the parent's literal content since it's now represented as child nodes
    // This prevents the renderer from using the literal instead of children
    // Note: For heading nodes, we don't clear the literal because heading nodes
    // should have NodeValue::Heading type, not NodeValue::Text type
    let parent_node = arena.get_mut(parent);
    if let NodeValue::Text(ref mut literal) = parent_node.value {
        *literal = "".into();
    }
}

/// Parse a reference definition from the beginning of a string.
/// Returns the number of characters consumed, or 0 if no reference was found.
/// If a reference is found, it is added to the refmap.
pub fn parse_reference(
    s: &str,
    refmap: &mut FxHashMap<String, (String, String)>,
) -> usize {
    // Skip leading whitespace and newlines
    let trimmed = s.trim_start_matches(|c: char| c.is_ascii_whitespace());
    let skipped = s.len() - trimmed.len();

    let consumed = parse_reference_definition(trimmed, refmap);

    if consumed > 0 {
        skipped + consumed
    } else {
        0
    }
}

// Re-export commonly used functions
pub use entities::unescape_string;
pub use utils::normalize_reference;
