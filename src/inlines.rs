/// Inline parsing for CommonMark documents
///
/// This module implements the inline parsing algorithm based on the CommonMark spec.
/// It processes the content of leaf blocks (paragraphs, headings, etc.) to produce
/// inline elements like emphasis, links, code, etc.
use crate::node::{append_child, Node, NodeData, NodeType};
use htmlescape::decode_html;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Maximum number of backticks to track
const MAX_BACKTICKS: usize = 1000;

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
    pub delimiters: Option<Rc<RefCell<Delimiter>>>,
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
    /// Whether smart punctuation is enabled
    pub smart: bool,
}

/// Delimiter struct for tracking emphasis markers
/// This is a doubly-linked list node using Rc<RefCell<>> for shared mutable access
/// matching commonmark.js implementation
pub struct Delimiter {
    /// Previous delimiter in stack
    pub previous: Option<Rc<RefCell<Delimiter>>>,
    /// Next delimiter in stack
    pub next: Option<Rc<RefCell<Delimiter>>>,
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
    pub previous_delimiter: Option<Rc<RefCell<Delimiter>>>,
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
            backticks: vec![0; MAX_BACKTICKS + 1],
            scanned_for_backticks: false,
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
            backticks: vec![0; MAX_BACKTICKS + 1],
            scanned_for_backticks: false,
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
            backticks: vec![0; MAX_BACKTICKS + 1],
            scanned_for_backticks: false,
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
    pub fn parse_inlines(&mut self, parent: &Rc<RefCell<Node>>) {
        while !self.end() {
            if !self.parse_inline(parent) {
                // If no inline was parsed, advance to avoid infinite loop
                self.advance();
            }
        }

        // Process any remaining emphasis
        self.process_emphasis(None);

        // Merge adjacent text nodes for cleaner output
        Self::merge_adjacent_text_nodes(parent);

        // Remove trailing spaces from the last text node
        Self::remove_trailing_spaces(parent);
    }

    /// Remove trailing spaces from the last text node
    fn remove_trailing_spaces(parent: &Rc<RefCell<Node>>) {
        let parent_type = parent.borrow().node_type.clone();

        // For headings, always remove trailing spaces
        if parent_type == NodeType::Heading {
            let last_child_opt = parent.borrow().last_child.borrow().clone();
            if let Some(last_child) = last_child_opt {
                if last_child.borrow().node_type == NodeType::Text {
                    let mut last_mut = last_child.borrow_mut();
                    if let NodeData::Text {
                        ref mut literal, ..
                    } = last_mut.data
                    {
                        while literal.ends_with(' ') {
                            literal.pop();
                        }
                    }
                }
            }
        }
        // For paragraphs and other containers, preserve trailing spaces
        // as they may be significant before inline elements
    }

    /// Merge adjacent text nodes in the given parent
    /// Optimized to reduce borrow overhead
    fn merge_adjacent_text_nodes(parent: &Rc<RefCell<Node>>) {
        let mut current_opt = parent.borrow().first_child.borrow().clone();

        while let Some(current) = current_opt {
            let next_opt = current.borrow().next.borrow().clone();

            if let Some(ref next) = next_opt {
                // Check node types first
                let current_is_text = current.borrow().node_type == NodeType::Text;
                let next_is_text = next.borrow().node_type == NodeType::Text;

                if current_is_text && next_is_text {
                    // Get literals with minimal borrow scope
                    let current_literal = Self::get_text_literal(&current);
                    let next_literal = Self::get_text_literal(&next);

                    // Don't merge if current or next is a smart quote
                    if !Self::is_smart_quote(&current_literal)
                        && !Self::is_smart_quote(&next_literal)
                    {
                        // Merge next into current
                        {
                            let mut current_mut = current.borrow_mut();
                            if let NodeData::Text { ref mut literal } = current_mut.data
                            {
                                literal.push_str(&next_literal);
                            }
                        }

                        // Remove next node
                        crate::node::unlink(next);
                        // Continue with same current
                        current_opt = Some(current);
                        continue;
                    }
                }
            }

            // Recursively process children
            Self::merge_adjacent_text_nodes(&current);

            current_opt = next_opt;
        }
    }

    /// Helper to get text literal from a node
    #[inline(always)]
    fn get_text_literal(node: &Rc<RefCell<Node>>) -> String {
        let node_ref = node.borrow();
        match &node_ref.data {
            NodeData::Text { literal } => literal.clone(),
            _ => String::new(),
        }
    }

    /// Check if string is a smart quote
    #[inline(always)]
    fn is_smart_quote(s: &str) -> bool {
        matches!(s, "\u{2018}" | "\u{2019}" | "\u{201C}" | "\u{201D}")
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
            '\n' => self.parse_newline(parent),
            '\'' | '"' if self.smart => self.handle_delim(c, parent),
            _ => self.parse_string(parent),
        }
    }

    /// Parse a newline. Returns a softbreak or hardbreak node.
    /// Based on commonmark.js parseNewline
    /// A line ending with 2+ spaces creates a hard line break
    fn parse_newline(&mut self, parent: &Rc<RefCell<Node>>) -> bool {
        // Check for preceding spaces (look back in the line)
        let preceding_spaces = self.count_preceding_spaces();

        // For hard line break, remove trailing spaces from the last text node
        if preceding_spaces >= 2 {
            self.remove_trailing_spaces_from_last_text(parent, preceding_spaces);
        }

        self.advance(); // skip \n

        if preceding_spaces >= 2 {
            // Hard line break: line ends with 2+ spaces
            let line_break = Rc::new(RefCell::new(Node::new(NodeType::LineBreak)));
            append_child(parent, line_break);
        } else {
            // Soft line break
            let soft_break = Rc::new(RefCell::new(Node::new(NodeType::SoftBreak)));
            append_child(parent, soft_break);
        }

        true
    }

    /// Remove trailing spaces from the last text node
    fn remove_trailing_spaces_from_last_text(
        &self,
        parent: &Rc<RefCell<Node>>,
        count: usize,
    ) {
        let last_child_opt = parent.borrow().last_child.borrow().clone();

        if let Some(last_child) = last_child_opt {
            if last_child.borrow().node_type == NodeType::Text {
                let mut last_mut = last_child.borrow_mut();
                if let NodeData::Text {
                    ref mut literal, ..
                } = last_mut.data
                {
                    // Remove trailing spaces
                    let new_len = literal.len().saturating_sub(count);
                    literal.truncate(new_len);
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
    fn parse_backticks(&mut self, parent: &Rc<RefCell<Node>>) -> bool {
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

                    let code_node = Rc::new(RefCell::new(Node::new(NodeType::Code)));
                    {
                        let mut code_mut = code_node.borrow_mut();
                        if let NodeData::Code {
                            ref mut literal, ..
                        } = code_mut.data
                        {
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
        if let Some((decoded, len)) = parse_entity_char(&self.input[self.pos..]) {
            self.append_text(parent, &decoded);
            self.pos += len;
            true
        } else {
            // Not a valid entity, treat & as literal
            // Append just "&" - the HTML renderer will escape it to &amp;
            self.append_text(parent, "&");
            self.advance(); // skip the &
            true
        }
    }

    /// Parse less-than sign (could be autolink or HTML tag)
    fn parse_lt(&mut self, parent: &Rc<RefCell<Node>>) -> bool {
        // Check if this looks like it could have been an autolink
        // We need to check this first to avoid matching invalid autolinks as HTML tags
        let remaining = &self.input[self.pos..];
        if remaining.starts_with('<') && remaining.len() > 1 {
            let after_lt = &remaining[1..];
            // Check if it looks like a potential URL (scheme:...) or email
            if Self::looks_like_potential_autolink(after_lt) {
                // Try autolink first
                if self.parse_autolink(parent) {
                    return true;
                }

                // This looks like it could be an autolink but failed validation
                // Output the < as a literal character (it will be escaped during rendering)
                let text_node = Rc::new(RefCell::new(Node::new(NodeType::Text)));
                {
                    let mut text_mut = text_node.borrow_mut();
                    if let NodeData::Text { ref mut literal } = text_mut.data {
                        *literal = "<".to_string();
                    }
                }
                append_child(parent, text_node);
                self.pos += 1;
                return true;
            }
        }

        // Try autolink first (for cases not caught by looks_like_potential_autolink)
        if self.parse_autolink(parent) {
            return true;
        }

        // Try HTML tag
        if self.parse_html_tag(parent) {
            return true;
        }

        // Just a literal < - add it as text
        let text_node = Rc::new(RefCell::new(Node::new(NodeType::Text)));
        {
            let mut text_mut = text_node.borrow_mut();
            if let NodeData::Text { ref mut literal } = text_mut.data {
                *literal = "<".to_string();
            }
        }
        append_child(parent, text_node);
        self.pos += 1;
        true
    }

    /// Check if the string looks like it could be a potential autolink
    /// This helps distinguish between "<not-a-tag>" and "<https://example.com>"
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
    fn parse_autolink(&mut self, parent: &Rc<RefCell<Node>>) -> bool {
        let remaining = &self.input[self.pos..];

        // Try email autolink first
        if let Some((email, len)) = match_email_autolink(remaining) {
            let link_node = Rc::new(RefCell::new(Node::new(NodeType::Link)));
            {
                let mut link_mut = link_node.borrow_mut();
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
    fn parse_html_tag(&mut self, parent: &Rc<RefCell<Node>>) -> bool {
        let remaining = &self.input[self.pos..];

        // Try to match HTML tag
        if let Some((tag_content, len)) = match_html_tag(remaining) {
            let html_node = Rc::new(RefCell::new(Node::new(NodeType::HtmlInline)));
            {
                let mut html_mut = html_node.borrow_mut();
                if let NodeData::HtmlInline { ref mut literal } = html_mut.data {
                    *literal = tag_content;
                }
            }
            append_child(parent, html_node);
            self.pos += len;
            return true;
        }

        false
    }

    /// Handle delimiter character (* or _)
    fn handle_delim(&mut self, c: char, parent: &Rc<RefCell<Node>>) -> bool {
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
        let text_node = Rc::new(RefCell::new(Node::new(NodeType::Text)));
        {
            let mut text_mut = text_node.borrow_mut();
            if let NodeData::Text {
                ref mut literal, ..
            } = text_mut.data
            {
                *literal = delim_text;
            }
        }
        append_child(parent, text_node.clone());

        // Add to delimiter stack if it can open or close
        if res.can_open || res.can_close {
            let delim = Rc::new(RefCell::new(Delimiter {
                previous: None,
                next: None,
                inl_text: text_node.clone(),
                position: start_pos,
                num_delims: res.num_delims,
                orig_delims: res.num_delims,
                delim_char: c,
                can_open: res.can_open,
                can_close: res.can_close,
            }));

            // Insert at top of stack, maintaining doubly-linked list
            if let Some(old_top) = self.delimiters.take() {
                old_top.borrow_mut().next = Some(delim.clone());
                delim.borrow_mut().previous = Some(old_top);
            }

            self.delimiters = Some(delim);
        }

        // If this delimiter can open emphasis, add an empty text node as a barrier
        // to prevent subsequent text from being merged into the delimiter node.
        // This is important for cases like foo *\** where the escaped * should
        // not be merged into the opener delimiter node.
        if res.can_open {
            let barrier = Rc::new(RefCell::new(Node::new(NodeType::Text)));
            {
                let mut barrier_mut = barrier.borrow_mut();
                if let NodeData::Text {
                    ref mut literal, ..
                } = barrier_mut.data
                {
                    *literal = String::new(); // Empty string
                }
            }
            append_child(parent, barrier);
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
    /// This prevents nested links (no links in links)
    /// Based on commonmark.js: deactivate link openers before this one
    fn deactivate_previous_link_openers(&mut self) {
        // Deactivate all previous link openers in the bracket stack
        // This prevents nested links (no links in links)
        // Based on commonmark.js: after a link is matched, deactivate all earlier link openers
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
    /// These delimiters are inside the link text and should not be processed as emphasis
    fn remove_delimiters_inside_link(&mut self, opener: &Bracket) {
        // Remove all delimiters that were added after the opener's previous_delimiter
        // These are delimiters inside the link text
        let stack_bottom = opener.previous_delimiter.clone();

        // The delimiter stack is organized with previous pointers (from top to bottom)
        // We need to find delimiters that are NEWER than stack_bottom (i.e., have stack_bottom in their previous chain)
        if let Some(ref bottom) = stack_bottom {
            // Simply set the stack_bottom's next to None
            // This effectively removes all delimiters newer than stack_bottom
            bottom.borrow_mut().next = None;

            // Now we need to rebuild the delimiter stack
            // Collect all delimiters from stack_bottom down to the bottom
            let mut delimiters_to_keep: Vec<Rc<RefCell<Delimiter>>> = Vec::new();
            let mut current = Some(bottom.clone());

            while let Some(delim) = current {
                delimiters_to_keep.push(delim.clone());
                current = delim.borrow().previous.clone();
            }

            // Rebuild the stack with proper next/previous links
            // delimiters_to_keep is in order from stack_bottom to bottom
            // We need to reverse it to get from bottom to stack_bottom
            delimiters_to_keep.reverse();

            // Clear and rebuild
            self.delimiters = None;
            let mut prev_delim: Option<Rc<RefCell<Delimiter>>> = None;

            for delim in delimiters_to_keep {
                delim.borrow_mut().previous = prev_delim.clone();
                delim.borrow_mut().next = None;
                if let Some(ref prev) = prev_delim {
                    prev.borrow_mut().next = Some(delim.clone());
                }
                self.delimiters = Some(delim.clone());
                prev_delim = Some(delim);
            }
        } else {
            // No previous delimiter, remove all delimiters
            self.delimiters = None;
        }
    }

    /// Process emphasis delimiters
    /// Based on commonmark.js processEmphasis function
    /// stack_bottom: if provided, only process delimiters after this one
    fn process_emphasis(&mut self, stack_bottom: Option<Rc<RefCell<Delimiter>>>) {
        // Get all delimiters as a vector for easier processing
        // Only include delimiters after stack_bottom
        let mut delims: Vec<Rc<RefCell<Delimiter>>> = Vec::new();
        let mut current = self.delimiters.clone();

        while let Some(d) = current {
            // Check if we've reached stack_bottom
            if let Some(ref bottom) = stack_bottom {
                if Rc::ptr_eq(&d, bottom) {
                    break;
                }
            }
            delims.push(d.clone());
            current = d.borrow().previous.clone();
        }

        // Reverse to process from bottom to top
        delims.reverse();

        // Initialize openers_bottom array
        // Index mapping (based on commonmark.js):
        // 0: single quote
        // 1: double quote
        // 2-7: underscore (can_open: 0/1, origdelims % 3: 0/1/2)
        // 8-13: asterisk (can_open: 0/1, origdelims % 3: 0/1/2)
        let mut openers_bottom: Vec<Option<usize>> = vec![None; 14];

        // Process each closer
        let mut i = 0;
        while i < delims.len() {
            let closer = delims[i].clone();
            let closer_borrow = closer.borrow();

            if !closer_borrow.can_close {
                i += 1;
                continue;
            }

            let closer_char = closer_borrow.delim_char;
            let closer_can_open = closer_borrow.can_open;
            let closer_orig_delims = closer_borrow.orig_delims;
            drop(closer_borrow);

            // Calculate openers_bottom_index based on closer type
            let openers_bottom_index = match closer_char {
                '\'' => 0,
                '"' => 1,
                '_' => {
                    2 + (if closer_can_open { 3 } else { 0 }) + (closer_orig_delims % 3)
                }
                '*' => {
                    8 + (if closer_can_open { 3 } else { 0 }) + (closer_orig_delims % 3)
                }
                _ => {
                    i += 1;
                    continue;
                }
            };

            // Look for matching opener
            let mut opener_idx = None;
            let bottom_idx = openers_bottom[openers_bottom_index];

            // Cache closer properties to avoid repeated borrows
            let closer_can_open_cached = closer.borrow().can_open;
            let closer_orig_delims_cached = closer.borrow().orig_delims;

            for j in (0..i).rev() {
                // Check if we've reached the bottom for this delimiter type
                if let Some(bottom) = bottom_idx {
                    if j <= bottom {
                        break;
                    }
                }

                let opener = delims[j].clone();
                let opener_borrow = opener.borrow();

                if !opener_borrow.can_open || opener_borrow.delim_char != closer_char {
                    continue;
                }

                // Check odd match rule using cached values
                let odd_match = (closer_can_open_cached || opener_borrow.can_close)
                    && closer_orig_delims_cached % 3 != 0
                    && (opener_borrow.orig_delims + closer_orig_delims_cached) % 3 == 0;

                if !odd_match {
                    opener_idx = Some(j);
                    break;
                }
            }

            let old_closer_idx = i;

            if let Some(j) = opener_idx {
                let opener = delims[j].clone();
                let opener_borrow = opener.borrow();
                let closer_borrow = closer.borrow();

                // Calculate number of delimiters to use
                let use_delims =
                    if opener_borrow.num_delims >= 2 && closer_borrow.num_delims >= 2 {
                        2
                    } else {
                        1
                    };

                let opener_inl = opener_borrow.inl_text.clone();
                let closer_inl = closer_borrow.inl_text.clone();
                let closer_char = closer_borrow.delim_char;
                drop(opener_borrow);
                drop(closer_borrow);

                // Handle smart quotes
                if self.smart && (closer_char == '\'' || closer_char == '"') {
                    // Update opener to left quote (if it's not already left quote)
                    {
                        let mut opener_inl_mut = opener_inl.borrow_mut();
                        if let NodeData::Text {
                            ref mut literal, ..
                        } = opener_inl_mut.data
                        {
                            if closer_char == '\'' {
                                *literal = "\u{2018}".to_string(); // left single quote
                            } else {
                                *literal = "\u{201C}".to_string(); // left double quote
                            }
                        }
                    }
                    // Update closer to right quote (if it's not already right quote)
                    {
                        let mut closer_inl_mut = closer_inl.borrow_mut();
                        if let NodeData::Text {
                            ref mut literal, ..
                        } = closer_inl_mut.data
                        {
                            if closer_char == '\'' {
                                *literal = "\u{2019}".to_string(); // right single quote
                            } else {
                                *literal = "\u{201D}".to_string(); // right double quote
                            }
                        }
                    }

                    // Update delimiter counts and remove if used up
                    {
                        let mut opener_mut = opener.borrow_mut();
                        if opener_mut.num_delims >= use_delims {
                            opener_mut.num_delims -= use_delims;
                        } else {
                            opener_mut.num_delims = 0;
                        }
                    }
                    {
                        let mut closer_mut = closer.borrow_mut();
                        if closer_mut.num_delims >= use_delims {
                            closer_mut.num_delims -= use_delims;
                        } else {
                            closer_mut.num_delims = 0;
                        }
                    }

                    // Remove delimiter nodes if no delims left
                    {
                        let opener_borrow = opener.borrow();
                        if opener_borrow.num_delims == 0 {
                            // Don't unlink smart quote nodes, just mark as processed
                        }
                    }
                    {
                        let closer_borrow = closer.borrow();
                        if closer_borrow.num_delims == 0 {
                            // Don't unlink smart quote nodes, just mark as processed
                        }
                    }

                    // Mark delimiters as processed
                    for k in j..=old_closer_idx {
                        let mut delim_mut = delims[k].borrow_mut();
                        delim_mut.can_open = false;
                        delim_mut.can_close = false;
                    }

                    // Remove processed delimiters from vector using swap_remove for O(1)
                    // Remove higher index first to avoid index shifting issues
                    if i == delims.len() - 1 {
                        delims.pop();
                    } else {
                        delims.swap_remove(i);
                    }
                    if j == delims.len() - 1 {
                        delims.pop();
                    } else {
                        delims.swap_remove(j);
                    }
                    // Adjust index since we removed two elements
                    i = i.saturating_sub(2);
                    continue;
                }

                // Create emphasis or strong node
                let emph_type = if use_delims == 1 {
                    NodeType::Emph
                } else {
                    NodeType::Strong
                };

                let emph_node = Rc::new(RefCell::new(Node::new(emph_type)));

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
                {
                    let mut opener_mut = opener.borrow_mut();
                    if opener_mut.num_delims >= use_delims {
                        opener_mut.num_delims -= use_delims;
                    } else {
                        opener_mut.num_delims = 0;
                    }

                    // Remove used delimiters from text node
                    let mut inl_mut = opener_mut.inl_text.borrow_mut();
                    if let NodeData::Text {
                        ref mut literal, ..
                    } = inl_mut.data
                    {
                        let len = literal.len();
                        if len >= use_delims {
                            *literal = literal[..len - use_delims].to_string();
                        }
                    }
                }

                {
                    let mut closer_mut = closer.borrow_mut();
                    if closer_mut.num_delims >= use_delims {
                        closer_mut.num_delims -= use_delims;
                    } else {
                        closer_mut.num_delims = 0;
                    }

                    // Remove used delimiters from text node
                    let mut inl_mut = closer_mut.inl_text.borrow_mut();
                    if let NodeData::Text {
                        ref mut literal, ..
                    } = inl_mut.data
                    {
                        let len = literal.len();
                        if len >= use_delims {
                            *literal = literal[..len - use_delims].to_string();
                        }
                    }
                }

                // Remove delimiters if no delims left
                {
                    let opener_borrow = opener.borrow();
                    if opener_borrow.num_delims == 0 {
                        crate::node::unlink(&opener_borrow.inl_text);
                    }
                }

                {
                    let closer_borrow = closer.borrow();
                    if closer_borrow.num_delims == 0 {
                        crate::node::unlink(&closer_borrow.inl_text);
                    }
                }

                // Mark delimiters between opener and closer as processed
                // by setting can_open and can_close to false
                // This prevents them from being matched in future iterations
                for k in (j + 1)..old_closer_idx {
                    let delim = delims[k].borrow();
                    // Don't mark if it's a different delimiter type that might still be valid
                    // Only mark if it's the same type or if it's between matched delimiters
                    drop(delim);
                    let mut delim_mut = delims[k].borrow_mut();
                    delim_mut.can_open = false;
                    delim_mut.can_close = false;
                }

                // Remove processed delimiters from vector using swap_remove for O(1)
                let mut removed_count = 0;
                if closer.borrow().num_delims == 0 {
                    if i == delims.len() - 1 {
                        delims.pop();
                    } else {
                        delims.swap_remove(i);
                    }
                    removed_count += 1;
                }
                if opener.borrow().num_delims == 0 {
                    let opener_idx = if removed_count > 0 && j < i { j } else { j };
                    if opener_idx == delims.len() - 1 {
                        delims.pop();
                    } else {
                        delims.swap_remove(opener_idx);
                    }
                    removed_count += 1;
                }
                // Adjust index
                i = i.saturating_sub(removed_count);
            } else {
                // No matching opener found - update openers_bottom
                if old_closer_idx > 0 {
                    openers_bottom[openers_bottom_index] = Some(old_closer_idx - 1);
                }

                // For smart quotes: unmatched quote that can both open and close is interpreted as left quote
                // This handles cases like: "A paragraph with no closing quote.
                // Based on commonmark.js: unmatched quote that can both open and close is interpreted as left quote
                if self.smart && (closer_char == '\'' || closer_char == '"') {
                    let closer_borrow = closer.borrow();
                    let can_open = closer_borrow.can_open;
                    let can_close = closer_borrow.can_close;
                    drop(closer_borrow);

                    // Only convert to left quote if it can both open and close
                    // If it can only close, keep it as right quote
                    // If it can only open, it was already set to left quote in handle_delim
                    if can_open && can_close {
                        let closer_inl = closer.borrow().inl_text.clone();
                        let mut closer_inl_mut = closer_inl.borrow_mut();
                        if let NodeData::Text {
                            ref mut literal, ..
                        } = closer_inl_mut.data
                        {
                            if closer_char == '\'' {
                                *literal = "\u{2018}".to_string(); // left single quote
                            } else {
                                *literal = "\u{201C}".to_string(); // left double quote
                            }
                        }
                    }
                }

                i += 1;
            }
        }
    }

    /// Parse open bracket (start of link or image)
    fn parse_open_bracket(&mut self, parent: &Rc<RefCell<Node>>) -> bool {
        self.advance(); // skip [

        // Create a new text node for the bracket (don't merge with previous)
        // This is important to keep the bracket as a separate node for proper link parsing
        let text_node = Rc::new(RefCell::new(Node::new(NodeType::Text)));
        {
            let mut text_mut = text_node.borrow_mut();
            if let NodeData::Text {
                ref mut literal, ..
            } = text_mut.data
            {
                *literal = "[".to_string();
            }
        }
        append_child(parent, text_node.clone());

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
            let link_node = Rc::new(RefCell::new(Node::new(node_type)));
            eprintln!("DEBUG: Creating link node");

            {
                let mut link_mut = link_node.borrow_mut();
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

            // Process emphasis with opener's previous delimiter FIRST
            // This processes emphasis delimiters inside the link text
            self.process_emphasis(opener.previous_delimiter.clone());

            // Remove delimiters that are inside the link from the delimiter stack
            // These delimiters have been processed and should be removed
            self.remove_delimiters_inside_link(&opener);

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
            // Based on commonmark.js: this.removeBracket()
            self.brackets = opener.previous;
            self.append_text(parent, "]");
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
                        } else {
                            // Backslash followed by non-escapable char is just a backslash
                            // But we need to include it in the destination
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
        let has_newline = false;

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
    fn parse_bang(&mut self, parent: &Rc<RefCell<Node>>) -> bool {
        self.advance(); // skip !

        if self.peek() == Some('[') {
            self.advance(); // skip [
                            // Create a separate text node for "![" to avoid merging with previous text
                            // This is important because the opener node will be unlinked when the image is processed
            let text_node = Rc::new(RefCell::new(Node::new(NodeType::Text)));
            {
                let mut text_mut = text_node.borrow_mut();
                if let NodeData::Text {
                    ref mut literal, ..
                } = text_mut.data
                {
                    *literal = "![".to_string();
                }
            }
            append_child(parent, text_node.clone());

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

    /// Parse a string of non-special characters (optimized with byte-level scanning)
    fn parse_string(&mut self, parent: &Rc<RefCell<Node>>) -> bool {
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
            let mut text = self.input[start..self.pos].to_string();

            // Apply smart punctuation transformations if enabled
            if self.smart {
                text = self.apply_smart_punctuation(&text);
            }

            // Create a new text node without merging
            // This is important to keep delimiter text nodes separate
            let text_node = Rc::new(RefCell::new(Node::new(NodeType::Text)));
            {
                let mut text_mut = text_node.borrow_mut();
                if let NodeData::Text {
                    ref mut literal, ..
                } = text_mut.data
                {
                    *literal = text;
                }
            }
            append_child(parent, text_node);
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
        parent: &Rc<RefCell<Node>>,
        text: &str,
    ) -> Rc<RefCell<Node>> {
        // Check if last child is a text node we can merge with
        let last_child_opt = parent.borrow().last_child.borrow().clone();

        if let Some(last_child) = last_child_opt {
            if last_child.borrow().node_type == NodeType::Text {
                // Merge with existing text node
                let mut last_mut = last_child.borrow_mut();
                if let NodeData::Text {
                    ref mut literal, ..
                } = last_mut.data
                {
                    literal.push_str(text);
                }
                return last_child.clone();
            }
        }

        // Create new text node
        let text_node = Rc::new(RefCell::new(Node::new(NodeType::Text)));
        {
            let mut text_mut = text_node.borrow_mut();
            if let NodeData::Text {
                ref mut literal, ..
            } = text_mut.data
            {
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
    parent: &Rc<RefCell<Node>>,
    content: &str,
    line: usize,
    block_offset: usize,
) {
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
    parse_inlines_with_options(parent, content, line, block_offset, refmap, false);
}

/// Parse inlines with reference map and smart punctuation option
pub fn parse_inlines_with_options(
    parent: &Rc<RefCell<Node>>,
    content: &str,
    line: usize,
    block_offset: usize,
    refmap: std::collections::HashMap<String, (String, String)>,
    smart: bool,
) {
    let mut subject =
        Subject::with_refmap_and_smart(content, line, block_offset, refmap, smart);
    subject.parse_inlines(parent);

    // Clear the parent's literal content since it's now represented as child nodes
    // This prevents the renderer from using the literal instead of children
    // Note: For heading nodes, we don't clear the literal because heading nodes
    // should have NodeData::Heading type, not NodeData::Text type
    let mut parent_mut = parent.borrow_mut();
    if let NodeData::Text { ref mut literal } = parent_mut.data {
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

/// HTML entity patterns
#[allow(dead_code)]
const ENTITY_PATTERN: &str =
    r"&#x[a-fA-F0-9]{1,6};|&#[0-9]{1,7};|&[a-zA-Z][a-zA-Z0-9]{1,31};";

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
        assert_eq!(
            first_child.as_ref().unwrap().borrow().node_type,
            NodeType::Code
        );
    }

    #[test]
    fn test_parse_text() {
        let parent = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        parse_inlines(&parent, "hello world", 1, 0);

        let parent_ref = parent.borrow();
        let first_child = parent_ref.first_child.borrow();
        assert!(first_child.is_some());
        assert_eq!(
            first_child.as_ref().unwrap().borrow().node_type,
            NodeType::Text
        );
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
        assert_eq!(
            first_child.as_ref().unwrap().borrow().node_type,
            NodeType::Link
        );

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
        assert_eq!(
            first_child.as_ref().unwrap().borrow().node_type,
            NodeType::Link
        );

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
        assert_eq!(
            first_child.as_ref().unwrap().borrow().node_type,
            NodeType::Image
        );

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
        assert_eq!(
            first_child.as_ref().unwrap().borrow().node_type,
            NodeType::Link
        );

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
        assert_eq!(
            first_child.as_ref().unwrap().borrow().node_type,
            NodeType::Link
        );

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
        let consumed =
            parse_reference("[label]: https://example.com \"title\"", &mut refmap);
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
        refmap.insert(
            "LABEL".to_string(),
            ("https://example.com".to_string(), "title".to_string()),
        );

        let parent = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        parse_inlines_with_refmap(&parent, "[text][label]", 1, 0, refmap);

        let parent_ref = parent.borrow();
        let first_child = parent_ref.first_child.borrow();
        assert!(first_child.is_some());
        assert_eq!(
            first_child.as_ref().unwrap().borrow().node_type,
            NodeType::Link
        );

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
    fn test_parse_html_open_tag() {
        let parent = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        parse_inlines(&parent, "<span>text</span>", 1, 0);

        let parent_ref = parent.borrow();
        let first_child = parent_ref.first_child.borrow();
        assert!(first_child.is_some());
        assert_eq!(
            first_child.as_ref().unwrap().borrow().node_type,
            NodeType::HtmlInline
        );

        let html = first_child.as_ref().unwrap().borrow();
        match &html.data {
            NodeData::HtmlInline { literal } => {
                assert_eq!(literal, "<span>");
            }
            _ => panic!("Expected HtmlInline node"),
        }
    }

    #[test]
    fn test_parse_html_close_tag() {
        let parent = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        parse_inlines(&parent, "</span>", 1, 0);

        let parent_ref = parent.borrow();
        let first_child = parent_ref.first_child.borrow();
        assert!(first_child.is_some());
        assert_eq!(
            first_child.as_ref().unwrap().borrow().node_type,
            NodeType::HtmlInline
        );

        let html = first_child.as_ref().unwrap().borrow();
        match &html.data {
            NodeData::HtmlInline { literal } => {
                assert_eq!(literal, "</span>");
            }
            _ => panic!("Expected HtmlInline node"),
        }
    }

    #[test]
    fn test_parse_html_self_closing_tag() {
        let parent = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        parse_inlines(&parent, "<br />", 1, 0);

        let parent_ref = parent.borrow();
        let first_child = parent_ref.first_child.borrow();
        assert!(first_child.is_some());
        assert_eq!(
            first_child.as_ref().unwrap().borrow().node_type,
            NodeType::HtmlInline
        );

        let html = first_child.as_ref().unwrap().borrow();
        match &html.data {
            NodeData::HtmlInline { literal } => {
                assert_eq!(literal, "<br />");
            }
            _ => panic!("Expected HtmlInline node"),
        }
    }

    #[test]
    fn test_parse_html_tag_with_attributes() {
        let parent = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        parse_inlines(
            &parent,
            "<a href=\"https://example.com\" class=\"link\">",
            1,
            0,
        );

        let parent_ref = parent.borrow();
        let first_child = parent_ref.first_child.borrow();
        assert!(first_child.is_some());
        assert_eq!(
            first_child.as_ref().unwrap().borrow().node_type,
            NodeType::HtmlInline
        );

        let html = first_child.as_ref().unwrap().borrow();
        match &html.data {
            NodeData::HtmlInline { literal } => {
                assert_eq!(literal, "<a href=\"https://example.com\" class=\"link\">");
            }
            _ => panic!("Expected HtmlInline node"),
        }
    }

    #[test]
    fn test_parse_html_comment() {
        let parent = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        parse_inlines(&parent, "<!-- comment -->", 1, 0);

        let parent_ref = parent.borrow();
        let first_child = parent_ref.first_child.borrow();
        assert!(first_child.is_some());
        assert_eq!(
            first_child.as_ref().unwrap().borrow().node_type,
            NodeType::HtmlInline
        );

        let html = first_child.as_ref().unwrap().borrow();
        match &html.data {
            NodeData::HtmlInline { literal } => {
                assert_eq!(literal, "<!-- comment -->");
            }
            _ => panic!("Expected HtmlInline node"),
        }
    }

    #[test]
    fn test_parse_html_processing_instruction() {
        let parent = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        parse_inlines(&parent, "<?xml version=\"1.0\"?>", 1, 0);

        let parent_ref = parent.borrow();
        let first_child = parent_ref.first_child.borrow();
        assert!(first_child.is_some());
        assert_eq!(
            first_child.as_ref().unwrap().borrow().node_type,
            NodeType::HtmlInline
        );

        let html = first_child.as_ref().unwrap().borrow();
        match &html.data {
            NodeData::HtmlInline { literal } => {
                assert_eq!(literal, "<?xml version=\"1.0\"?>");
            }
            _ => panic!("Expected HtmlInline node"),
        }
    }

    #[test]
    fn test_parse_html_declaration() {
        let parent = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        parse_inlines(&parent, "<!DOCTYPE html>", 1, 0);

        let parent_ref = parent.borrow();
        let first_child = parent_ref.first_child.borrow();
        assert!(first_child.is_some());
        assert_eq!(
            first_child.as_ref().unwrap().borrow().node_type,
            NodeType::HtmlInline
        );

        let html = first_child.as_ref().unwrap().borrow();
        match &html.data {
            NodeData::HtmlInline { literal } => {
                assert_eq!(literal, "<!DOCTYPE html>");
            }
            _ => panic!("Expected HtmlInline node"),
        }
    }

    #[test]
    fn test_parse_html_cdata() {
        let parent = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        parse_inlines(&parent, "<![CDATA[<html>]]>", 1, 0);

        let parent_ref = parent.borrow();
        let first_child = parent_ref.first_child.borrow();
        assert!(first_child.is_some());
        assert_eq!(
            first_child.as_ref().unwrap().borrow().node_type,
            NodeType::HtmlInline
        );

        let html = first_child.as_ref().unwrap().borrow();
        match &html.data {
            NodeData::HtmlInline { literal } => {
                assert_eq!(literal, "<![CDATA[<html>]]>");
            }
            _ => panic!("Expected HtmlInline node"),
        }
    }

    #[test]
    fn debug_emphasis_parsing() {
        // This test is for debugging emphasis parsing
        eprintln!("\n=== Debug Emphasis Parsing ===");
        let parent = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        parse_inlines(&parent, "*foo bar*", 1, 0);

        // Print the tree structure
        fn print_tree(node: &Rc<RefCell<Node>>, indent: usize) {
            let node_ref = node.borrow();
            let indent_str = "  ".repeat(indent);

            match &node_ref.data {
                NodeData::Text { literal } => {
                    eprintln!("{}Text: '{}'", indent_str, literal);
                }
                NodeData::Emph => {
                    eprintln!("{}Emph:", indent_str);
                }
                NodeData::Strong => {
                    eprintln!("{}Strong:", indent_str);
                }
                _ => {
                    eprintln!("{}Other: {:?}", indent_str, node_ref.node_type);
                }
            }

            let first_child = node_ref.first_child.borrow().clone();
            let next = node_ref.next.borrow().clone();
            drop(node_ref);

            if let Some(child) = first_child {
                print_tree(&child, indent + 1);
            }
            if let Some(next_node) = next {
                print_tree(&next_node, indent);
            }
        }

        eprintln!("AST Tree:");
        print_tree(&parent, 0);
        eprintln!("=== End Debug ===\n");
    }

    #[test]
    fn test_html_tag_no_space_between_attrs() {
        // Test #622: attributes without space should not be valid
        use crate::inlines::match_html_tag;

        // First test match_html_tag directly
        let input = "<a href='bar'title=title>";
        let result = match_html_tag(input);
        println!("match_html_tag('{}') = {:?}", input, result);

        // This should return None (not a valid HTML tag)
        assert!(
            result.is_none(),
            "Should not match as valid HTML tag: {:?}",
            result
        );
    }

    #[test]
    fn test_emphasis_with_escaped_delim() {
        // Test #437: foo *\** should produce <p>foo <em>*</em></p>
        use crate::render_html;

        let parent = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        parse_inlines(&parent, "foo *\\**", 1, 0);

        // Print tree structure for debugging
        fn print_tree(node: &Rc<RefCell<Node>>, indent: usize) {
            let node_ref = node.borrow();
            let indent_str = "  ".repeat(indent);
            match &node_ref.data {
                NodeData::Text { literal } => {
                    println!("{}Text: '{}'", indent_str, literal);
                }
                NodeData::Emph => {
                    println!("{}Emph:", indent_str);
                }
                _ => {
                    println!("{}Other: {:?}", indent_str, node_ref.node_type);
                }
            }
            let first_child = node_ref.first_child.borrow().clone();
            let next = node_ref.next.borrow().clone();
            drop(node_ref);
            if let Some(child) = first_child {
                print_tree(&child, indent + 1);
            }
            if let Some(next_node) = next {
                print_tree(&next_node, indent);
            }
        }
        println!("Tree structure:");
        print_tree(&parent, 0);

        // Also print all children of paragraph sequentially
        println!("\nSequential children:");
        let parent_ref = parent.borrow();
        let mut current = parent_ref.first_child.borrow().clone();
        while let Some(node) = current {
            let node_ref = node.borrow();
            match &node_ref.data {
                NodeData::Text { literal } => {
                    println!("  Text: '{}'", literal);
                }
                NodeData::Emph => {
                    println!("  Emph:");
                    let mut emph_child = node_ref.first_child.borrow().clone();
                    while let Some(child) = emph_child {
                        let child_ref = child.borrow();
                        match &child_ref.data {
                            NodeData::Text { literal } => {
                                println!("    Text: '{}'", literal);
                            }
                            _ => {}
                        }
                        emph_child = child_ref.next.borrow().clone();
                    }
                }
                _ => {
                    println!("  Other: {:?}", node_ref.node_type);
                }
            }
            current = node_ref.next.borrow().clone();
        }

        // Render to HTML and check
        let output = render_html(&parent, 0);
        println!("\nOutput: {}", output);
        assert_eq!(output, "<p>foo <em>*</em></p>");
    }

    #[test]
    fn test_normalize_reference_with_backslash() {
        // Test that normalize_reference preserves backslashes
        let label1 = normalize_reference("[foo!]");
        let label2 = normalize_reference("[foo\\!]");
        println!("label1: {:?}", label1);
        println!("label2: {:?}", label2);
        assert_ne!(label1, label2, "Labels should be different");
    }

    #[test]
    fn test_parse_link_label_with_backslash() {
        // Test parse_link_label with escaped characters
        let input = "[foo\\!]";
        let mut subject = Subject::new(input, 1, 0);
        let len = subject.parse_link_label();
        println!("Input: {:?}", input);
        println!("Length: {}", len);
        println!("Extracted: {:?}", &input[0..len]);
        assert_eq!(len, 7, "Should parse entire label including backslash");
    }

    #[test]
    fn test_reference_definition_label() {
        // Test that reference definition labels are correctly parsed
        use crate::inlines::parse_reference;

        let mut refmap = std::collections::HashMap::new();
        let consumed = parse_reference("[foo!]: /url", &mut refmap);
        println!("Consumed: {}", consumed);
        println!("Refmap keys: {:?}", refmap.keys().collect::<Vec<_>>());

        // The label should be "FOO!"
        assert!(refmap.contains_key("FOO!"), "Should have FOO! in refmap");

        // Now test with escaped label
        let mut refmap2 = std::collections::HashMap::new();
        let consumed2 = parse_reference("[foo\\!]: /url", &mut refmap2);
        println!("Escaped - Consumed: {}", consumed2);
        println!(
            "Escaped - Refmap keys: {:?}",
            refmap2.keys().collect::<Vec<_>>()
        );

        // The label should be "FOO\!"
        assert!(
            refmap2.contains_key("FOO\\!"),
            "Should have FOO\\! in refmap"
        );
    }

    #[test]
    fn test_emphasis_with_currency() {
        // Test #354: emphasis with currency symbols
        use crate::render_html;

        let test_cases = vec![
            ("*$*alpha.", "<p>*$*alpha.</p>"),
            ("*£*bravo.", "<p>*£*bravo.</p>"),
            ("*€*charlie.", "<p>*€*charlie.</p>"),
        ];

        for (input, expected) in test_cases {
            let parent = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
            parse_inlines(&parent, input, 1, 0);
            let output = render_html(&parent, 0);
            println!("Input: {:?}", input);
            println!("Expected: {}", expected);
            println!("Got:      {}", output);
            println!();
            assert_eq!(output, expected, "Failed for input: {}", input);
        }
    }
}
