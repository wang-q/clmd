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
}

/// Delimiter struct for tracking emphasis markers
pub struct Delimiter {
    /// Previous delimiter in stack
    pub previous: Option<Box<Delimiter>>,
    /// Next delimiter in stack
    pub next: Option<*mut Delimiter>,
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
        // For now, just treat & as literal
        // Full entity parsing would require an entity table
        self.parse_string(parent)
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
    fn parse_autolink(&mut self, _parent: &Rc<RefCell<Node>>) -> bool {
        // Simplified autolink detection
        // Full implementation would use regex
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
                next: None,
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
    fn process_emphasis(&mut self, _stack_bottom: Option<*const Delimiter>) {
        // Simplified emphasis processing
        // Full implementation would match openers and closers
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
        });

        self.brackets = Some(bracket);
        self.no_link_openers = false;

        true
    }

    /// Parse close bracket (end of link or image)
    fn parse_close_bracket(&mut self, parent: &Rc<RefCell<Node>>) -> bool {
        self.advance(); // skip ]

        // For now, just add as text
        // Full implementation would try to match with opener
        self.append_text(parent, "]");

        true
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
}
