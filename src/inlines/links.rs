//! Link and image processing for inline parsing
//!
//! This module handles link (`[text](url)`) and image (`![alt](url)`) parsing
//! according to the CommonMark specification (section 6.2-6.9).
//!
//! # Algorithm Overview
//!
//! Link parsing is one of the most complex parts of CommonMark due to the
//! various link types and nesting rules:
//!
//! ## Link Types
//!
//! 1. **Inline Links**: `[text](url "title")`
//!    - URL and optional title provided inline
//!
//! 2. **Reference Links**: `[text][label]` or `[text][]`
//!    - Label refers to a link reference definition elsewhere in the document
//!    - Short form `[text][]` uses the text itself as the label
//!
//! 3. **Autolinks**: `<http://example.com>`
//!    - URL enclosed in angle brackets
//!
//! ## Parsing Process
//!
//! 1. **Bracket Scanning**: `[` and `![` are identified as potential link starts
//! 2. **Link Text Parsing**: Content between brackets is parsed as inline content
//! 3. **Link Destination Parsing**: After `]`, look for `(` for inline links
//!    or `[` for reference links
//! 4. **Reference Resolution**: For reference links, look up the label in refmap
//!
//! ## Special Rules
//!
//! - Links cannot contain other links (but can contain images)
//! - Images can contain links
//! - Link text can contain emphasis and other inline elements
//! - Nested brackets are allowed but must be balanced
//!
//! # CommonMark Compliance
//!
//! This implementation follows CommonMark 0.31.2 specification:
//! - Link text parsing with proper nesting
//! - Reference link matching (case-insensitive, whitespace collapsed)
//! - Link destination normalization
//! - Title parsing with various delimiters
//!
//! Reference: https://spec.commonmark.org/0.31.2/#links

use crate::core::arena::{Node, NodeArena, NodeId, TreeOps};
use crate::core::nodes::{NodeLink, NodeValue};
use crate::inlines::entities::unescape_string;
use crate::inlines::utils::{
    is_escapable, is_punctuation, normalize_reference, normalize_uri,
};
use rustc_hash::FxHashMap;
use smallvec::SmallVec;

/// Bracket struct for tracking link/image brackets
#[derive(Debug)]
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
    /// Previous delimiter in stack (for emphasis processing)
    /// This is a marker to identify which delimiter was on the stack before this bracket
    pub previous_delimiter_marker: Option<(NodeId, usize)>, // (inl_text, orig_delims)
}

/// Link parsing context
pub struct LinkContext<'a> {
    pub input: &'a str,
    pub pos: usize,
    pub refmap: &'a FxHashMap<String, (String, String)>,
}

impl<'a> LinkContext<'a> {
    /// Create a new link context
    pub fn new(
        input: &'a str,
        pos: usize,
        refmap: &'a FxHashMap<String, (String, String)>,
    ) -> Self {
        Self { input, pos, refmap }
    }

    /// Peek at the current character
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

    /// Advance position by one character
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

    /// Skip spaces and at most one newline
    pub fn skip_spaces_and_newlines(&mut self) {
        while let Some(c) = self.peek() {
            if c == ' ' || c == '\t' || c == '\n' || c == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Look up a reference in the refmap
    pub fn lookup_reference(&self, label: &str) -> Option<(String, String)> {
        self.refmap.get(label).cloned()
    }
}

/// Parse link destination (URL)
/// Based on commonmark.js parseLinkDestination
/// Returns Some((destination, ended_with_space)) on success, None on failure
/// ended_with_space is true if the destination ended due to a space (not close paren)
pub fn parse_link_destination(ctx: &mut LinkContext) -> Option<(String, bool)> {
    // Try angle-bracketed destination: <url>
    if ctx.peek() == Some('<') {
        let start_pos = ctx.pos;
        ctx.advance(); // skip <
        let content_start = ctx.pos;

        while let Some(c) = ctx.peek() {
            if c == '>' {
                let dest = ctx.input[content_start..ctx.pos].to_string();
                ctx.advance(); // skip >
                               // Unescape and normalize the destination
                let unescaped = unescape_string(&dest);
                return Some((normalize_uri(&unescaped), false));
            } else if c == '<' || c == '\n' || c == '\r' {
                // Newlines and < not allowed in angle-bracketed destinations
                // Rewind to start
                ctx.pos = start_pos;
                return None;
            } else if c == '\\' {
                // Backslash escape - check if there's a character after it
                ctx.advance();
                if let Some(next_c) = ctx.peek() {
                    if is_escapable(next_c) {
                        ctx.advance();
                    }
                } else {
                    // Backslash at end of input - invalid
                    // Rewind to start
                    ctx.pos = start_pos;
                    return None;
                }
            } else {
                ctx.advance();
            }
        }
        // Reached end of input without finding >
        // Rewind to start
        ctx.pos = start_pos;
        return None;
    }

    // Try unbracketed destination
    let start = ctx.pos;
    let mut paren_depth = 0;
    let mut ended_with_space = false;
    let mut has_newline = false;

    while let Some(c) = ctx.peek() {
        if c == '\\' {
            ctx.advance();
            if let Some(next_c) = ctx.peek() {
                if is_escapable(next_c) {
                    ctx.advance();
                }
            }
        } else if c == '(' {
            paren_depth += 1;
            ctx.advance();
        } else if c == ')' {
            if paren_depth == 0 {
                break;
            }
            paren_depth -= 1;
            ctx.advance();
        } else if c == ' ' || c == '\t' {
            ended_with_space = true;
            break;
        } else if c == '\n' || c == '\r' {
            // Newlines not allowed in link destinations (even for reference definitions)
            has_newline = true;
            break;
        } else {
            ctx.advance();
        }
    }

    // Allow empty destination if we're at a close paren (like [link]())
    if ctx.pos == start {
        // Check if we're at a close paren (empty destination case)
        if ctx.peek() == Some(')') {
            return Some((String::new(), false));
        }
        return None;
    }

    if paren_depth != 0 {
        return None;
    }

    let dest = ctx.input[start..ctx.pos].to_string();
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
pub fn parse_link_title(ctx: &mut LinkContext) -> Option<String> {
    let quote = match ctx.peek() {
        Some('"') => '"',
        Some('\'') => '\'',
        Some('(') => '(',
        _ => return None,
    };

    let close_quote = if quote == '(' { ')' } else { quote };
    ctx.advance(); // skip opening quote

    let start = ctx.pos;

    while let Some(c) = ctx.peek() {
        if c == close_quote {
            let title = ctx.input[start..ctx.pos].to_string();
            ctx.advance(); // skip closing quote
                           // Unescape the title
            return Some(unescape_string(&title));
        } else if c == '\\' {
            ctx.advance();
            if let Some(next_c) = ctx.peek() {
                if is_escapable(next_c) {
                    ctx.advance();
                }
            }
        } else if c == '\n' || c == '\r' {
            // For reference definitions, newlines are allowed in titles
            ctx.advance();
        } else {
            ctx.advance();
        }
    }

    None
}

/// Parse a link label like [label]
/// Returns the length of the label including brackets, or 0 if no match
pub fn parse_link_label(ctx: &mut LinkContext) -> usize {
    let start_pos = ctx.pos;

    // Must start with [
    if ctx.peek() != Some('[') {
        return 0;
    }
    ctx.advance(); // skip [

    let label_start = ctx.pos;

    while let Some(c) = ctx.peek() {
        if c == '\\' {
            // Escaped character - include both backslash and next char in label
            // According to CommonMark spec, backslash escapes are preserved in link labels
            ctx.advance(); // skip \
            if ctx.peek().is_some() {
                ctx.advance(); // include escaped char
            }
        } else if c == '[' {
            // Unescaped [ is not allowed in link labels
            // Rewind and return 0
            ctx.pos = start_pos;
            return 0;
        } else if c == ']' {
            // Found closing bracket
            ctx.advance(); // skip ]
            let label_len = ctx.pos - start_pos;
            // Label max 999 characters (excluding brackets)
            // Empty label ([]) is allowed for collapsed reference links
            let content_len = ctx.pos - label_start - 1;
            if content_len > 999 {
                ctx.pos = start_pos;
                return 0;
            }
            return label_len;
        } else if c == '\n' {
            // Labels can contain newlines, but they are normalized to spaces
            // during reference normalization. We just need to continue parsing.
            ctx.advance();
        } else {
            ctx.advance();
        }
    }

    // No closing bracket found
    ctx.pos = start_pos;
    0
}

/// Parse inline link: [text](url "title")
/// Returns (dest, title, consumed_chars) if successful
pub fn parse_inline_link(ctx: &mut LinkContext) -> Option<(String, String, usize)> {
    let start_pos = ctx.pos;

    if ctx.peek() != Some('(') {
        return None;
    }
    ctx.advance(); // skip (
    ctx.skip_spaces_and_newlines();

    // Parse link destination
    let dest = match parse_link_destination(ctx) {
        Some((d, _)) => d,
        None => {
            ctx.pos = start_pos;
            return None;
        }
    };

    ctx.skip_spaces_and_newlines();

    // Try to parse optional title
    let title = parse_link_title(ctx);

    ctx.skip_spaces_and_newlines();

    if ctx.peek() == Some(')') {
        ctx.advance(); // skip )
        Some((dest, title.unwrap_or_default(), ctx.pos - start_pos))
    } else {
        ctx.pos = start_pos;
        None
    }
}

/// Parse reference link: [text][label] or [text][]
/// Returns (dest, title, consumed_chars) if successful
///
/// # Arguments
/// * `ctx` - The link context
/// * `opener_position` - The position of the opening bracket `[` in the input
/// * `closer_position` - The position of the closing bracket `]` in the input
/// * `is_image` - Whether this is an image link
pub fn parse_reference_link(
    ctx: &mut LinkContext,
    opener_position: usize,
    closer_position: usize,
    is_image: bool,
) -> Option<(String, String, usize)> {
    let start_pos = ctx.pos;
    let label_len = parse_link_label(ctx);

    if label_len > 2 {
        // Full reference link [text][label] with non-empty label
        let label = ctx.input[start_pos..start_pos + label_len].to_string();
        let norm_label = normalize_reference(&label);

        if let Some((dest, title)) = ctx.lookup_reference(&norm_label) {
            return Some((dest, title, ctx.pos - start_pos));
        }

        // Reference not found
        ctx.pos = start_pos;
        return None;
    } else if label_len == 2 {
        // Collapsed reference link [text][] - use the link text as label
        // For images, opener.position points to '!', so text starts at position + 2
        // For links, opener.position points to '[', so text starts at position + 1
        let label_start = if is_image {
            opener_position + 2
        } else {
            opener_position + 1
        };
        // Use closer_position to get the correct label end (before the closing bracket)
        let label_end = closer_position;

        if label_start < label_end {
            let label = ctx.input[label_start..label_end].to_string();
            let norm_label = normalize_reference(&label);

            if let Some((dest, title)) = ctx.lookup_reference(&norm_label) {
                return Some((dest, title, ctx.pos - start_pos));
            }
        }

        ctx.pos = start_pos;
        return None;
    } else if label_len == 0 {
        // Shortcut reference link [text] - only if at end of line or followed by punctuation
        // Check the character after the closing bracket (closer_position + 1)
        let after_closer = closer_position + 1;
        let at_line_end = if after_closer >= ctx.input.len() {
            true
        } else {
            let c = ctx.input.as_bytes()[after_closer] as char;
            c == '\n' || c == '\r' || c == ' ' || c == '\t'
        };

        // Also allow shortcut reference links followed by punctuation
        let followed_by_punct = if after_closer >= ctx.input.len() {
            false
        } else {
            let c = ctx.input.as_bytes()[after_closer] as char;
            is_punctuation(c)
        };

        if at_line_end || followed_by_punct {
            let label_start = if is_image {
                opener_position + 2
            } else {
                opener_position + 1
            };
            // Use closer_position to get the correct label end (before the closing bracket)
            let label_end = closer_position;

            if label_start < label_end {
                let label = ctx.input[label_start..label_end].to_string();
                let norm_label = normalize_reference(&label);

                if let Some((dest, title)) = ctx.lookup_reference(&norm_label) {
                    return Some((dest, title, 0)); // No characters consumed for shortcut
                }
            }
        }
    }

    ctx.pos = start_pos;
    None
}

/// Parse a reference definition: [label]: url "title"
/// Returns the number of characters consumed, or 0 if no reference was found
pub fn parse_reference_definition(
    input: &str,
    refmap: &mut FxHashMap<String, (String, String)>,
) -> usize {
    // Create a temporary empty refmap for the context (we won't use it for lookup)
    let empty_refmap = FxHashMap::default();
    let mut ctx = LinkContext::new(input, 0, &empty_refmap);
    let start_pos = ctx.pos;

    // Parse label: [label]
    let label_len = parse_link_label(&mut ctx);
    if label_len == 0 {
        return 0;
    }

    let raw_label = ctx.input[start_pos..start_pos + label_len].to_string();

    // Empty label ([]) or label with only whitespace is not allowed for reference definitions
    let label_content = &raw_label[1..raw_label.len() - 1]; // Remove brackets
    if label_content.trim().is_empty() {
        return 0;
    }

    // Expect colon
    if ctx.peek() != Some(':') {
        return 0;
    }
    ctx.advance(); // skip :

    // Skip spaces and newlines
    ctx.skip_spaces_and_newlines();

    // Parse link destination
    let dest = match parse_link_destination(&mut ctx) {
        Some((d, _)) => d,
        None => return 0,
    };

    let before_title = ctx.pos;
    ctx.skip_spaces_and_newlines();

    // Try to parse optional title
    // Title can be on the same line or on the next line (with indentation)
    // But only if we actually skipped some whitespace (title must be separated from destination)
    let title = if ctx.pos > before_title {
        parse_link_title(&mut ctx)
    } else {
        None
    };

    if title.is_none() {
        ctx.pos = before_title;
    }

    // Must be at end of line or only whitespace/newlines remain
    let remaining = &ctx.input[ctx.pos..];
    let at_line_end = remaining.is_empty()
        || remaining.starts_with('\n')
        || remaining.starts_with('\r')
        || remaining.chars().all(|c| c.is_ascii_whitespace());

    if !at_line_end {
        // Check if we can still match without title
        ctx.pos = before_title;
        let remaining = &ctx.input[ctx.pos..];
        let at_line_end_without_title = remaining.is_empty()
            || remaining.starts_with('\n')
            || remaining.starts_with('\r')
            || remaining.chars().all(|c| c.is_ascii_whitespace());
        if !at_line_end_without_title {
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

    ctx.pos
}

/// Create a link or image node
pub fn create_link_node(
    arena: &mut NodeArena,
    is_image: bool,
    dest: String,
    title: String,
) -> NodeId {
    let value = if is_image {
        NodeValue::image(NodeLink { url: dest, title })
    } else {
        NodeValue::link(NodeLink { url: dest, title })
    };

    arena.alloc(Node::with_value(value))
}

/// Move nodes between opener and closer into the link/image node
pub fn move_nodes_to_link(
    arena: &mut NodeArena,
    link_node: NodeId,
    opener_inl: NodeId,
    parent: NodeId,
) {
    // Use SmallVec to avoid heap allocation for small stacks
    let mut nodes_to_move: SmallVec<[NodeId; 16]> = SmallVec::new();

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
}
