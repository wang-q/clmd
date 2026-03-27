//! Strikethrough extension for GitHub Flavored Markdown
//!
//! This module implements strikethrough parsing according to GFM spec.
//! Strikethrough text is delimited by two tildes: ~~strikethrough text~~
//!
//! Example:
//! ```markdown
//! This is ~~deleted~~ text.
//! ```

use crate::arena::{Node, NodeArena, NodeId, TreeOps};
use crate::node::{NodeData, NodeType, SourcePos};
use crate::node_value::NodeValue;

/// The tilde character used for strikethrough
pub const STRIKETHROUGH_DELIM: char = '~';

/// The number of tildes required for strikethrough
pub const STRIKETHROUGH_COUNT: usize = 2;

/// Check if a string contains strikethrough syntax
pub fn contains_strikethrough(text: &str) -> bool {
    text.contains("~~")
}

/// Parse strikethrough delimiters in text and return positions
/// Returns vector of (start, end) positions for strikethrough spans
pub fn parse_strikethrough_spans(text: &str) -> Vec<(usize, usize)> {
    let mut spans = Vec::new();
    let mut chars = text.char_indices().peekable();
    let mut start_pos = None;

    while let Some((pos, ch)) = chars.next() {
        if ch == STRIKETHROUGH_DELIM {
            // Count consecutive tildes
            let mut count = 1;
            let start_idx = pos;

            while let Some(&(_, next_ch)) = chars.peek() {
                if next_ch == STRIKETHROUGH_DELIM {
                    count += 1;
                    chars.next();
                } else {
                    break;
                }
            }

            // Check if we have exactly 2 tildes (strikethrough)
            if count == STRIKETHROUGH_COUNT {
                if let Some(start) = start_pos {
                    // Closing delimiter found
                    spans.push((start, start_idx));
                    start_pos = None;
                } else {
                    // Opening delimiter found
                    start_pos = Some(start_idx + STRIKETHROUGH_COUNT);
                }
            }
        }
    }

    spans
}

/// Create a strikethrough node in the arena containing the given text
/// Returns the NodeId of the created node
pub fn create_strikethrough_node(
    arena: &mut NodeArena,
    text: &str,
    start_line: u32,
    start_col: u32,
) -> NodeId {
    let node = arena.alloc(Node::with_value(NodeValue::Strikethrough));

    {
        let node_ref = arena.get_mut(node);
        node_ref.source_pos = SourcePos {
            start_line,
            start_column: start_col,
            end_line: start_line,
            end_column: start_col + text.len() as u32 + 4, // +4 for the ~~ delimiters
        };
    }

    // Create text node for the content
    let text_node = arena.alloc(Node::with_value(NodeValue::Text(text.to_string())));

    // Add text node as child
    TreeOps::append_child(arena, node, text_node);

    node
}

/// Process text for strikethrough and return a list of node IDs
/// This splits the text into regular text nodes and strikethrough nodes
pub fn process_strikethrough(
    arena: &mut NodeArena,
    text: &str,
    line: u32,
    col: u32,
) -> Vec<NodeId> {
    let spans = parse_strikethrough_spans(text);
    if spans.is_empty() {
        // No strikethrough found, return single text node
        let node = arena.alloc(Node::with_value(NodeValue::Text(text.to_string())));
        return vec![node];
    }

    let mut nodes = Vec::new();
    let mut last_end = 0;

    for (start, end) in spans {
        // Add text before strikethrough (if any)
        if start > last_end + STRIKETHROUGH_COUNT {
            let before_text = &text[last_end..start - STRIKETHROUGH_COUNT];
            if !before_text.is_empty() {
                let node = arena
                    .alloc(Node::with_value(NodeValue::Text(before_text.to_string())));
                nodes.push(node);
            }
        }

        // Add strikethrough node
        let strike_text = &text[start..end];
        let strike_node =
            create_strikethrough_node(arena, strike_text, line, col + start as u32);
        nodes.push(strike_node);

        last_end = end + STRIKETHROUGH_COUNT;
    }

    // Add remaining text after last strikethrough (if any)
    if last_end < text.len() {
        let after_text = &text[last_end..];
        let node =
            arena.alloc(Node::with_value(NodeValue::Text(after_text.to_string())));
        nodes.push(node);
    }

    nodes
}

/// Render strikethrough text to HTML
pub fn render_strikethrough_html(content: &str) -> String {
    format!("<del>{}</del>", crate::html_utils::escape_html(content))
}

/// Render strikethrough text to CommonMark
pub fn render_strikethrough_commonmark(content: &str) -> String {
    format!("~~{}~~", content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains_strikethrough() {
        assert!(contains_strikethrough("This is ~~deleted~~ text"));
        assert!(contains_strikethrough("~~test~~"));
        assert!(!contains_strikethrough("This is normal text"));
        assert!(!contains_strikethrough("This is ~single~ tilde"));
    }

    #[test]
    fn test_parse_strikethrough_spans() {
        let spans = parse_strikethrough_spans("~~deleted~~");
        assert_eq!(spans, vec![(2, 9)]);

        let spans = parse_strikethrough_spans("This is ~~deleted~~ text");
        assert_eq!(spans, vec![(10, 17)]);
    }

    #[test]
    fn test_parse_multiple_strikethroughs() {
        let spans = parse_strikethrough_spans("~~one~~ and ~~two~~");
        assert_eq!(spans, vec![(2, 5), (14, 17)]);
    }

    #[test]
    fn test_parse_no_strikethrough() {
        let spans = parse_strikethrough_spans("normal text");
        assert!(spans.is_empty());

        let spans = parse_strikethrough_spans("~single tilde~");
        assert!(spans.is_empty());

        let spans = parse_strikethrough_spans("~~~three tildes~~~");
        // Should not match because we need exactly 2 tildes
        assert!(spans.is_empty());
    }

    #[test]
    fn test_create_strikethrough_node() {
        let mut arena = NodeArena::new();
        let node_id = create_strikethrough_node(&mut arena, "deleted", 1, 1);
        let node = arena.get(node_id);
        assert_eq!(node.node_type, NodeType::Strikethrough);
        match &node.data {
            NodeData::Strikethrough => {}
            _ => panic!("Expected Strikethrough data"),
        }
    }

    #[test]
    fn test_process_strikethrough() {
        let mut arena = NodeArena::new();
        let nodes = process_strikethrough(&mut arena, "This is ~~deleted~~ text", 1, 1);
        assert_eq!(nodes.len(), 3); // text, strikethrough, text

        assert_eq!(arena.get(nodes[0]).node_type, NodeType::Text);
        assert_eq!(arena.get(nodes[1]).node_type, NodeType::Strikethrough);
        assert_eq!(arena.get(nodes[2]).node_type, NodeType::Text);
    }

    #[test]
    fn test_process_no_strikethrough() {
        let mut arena = NodeArena::new();
        let nodes = process_strikethrough(&mut arena, "normal text", 1, 1);
        assert_eq!(nodes.len(), 1);
        assert_eq!(arena.get(nodes[0]).node_type, NodeType::Text);
    }

    #[test]
    fn test_render_strikethrough_html() {
        assert_eq!(render_strikethrough_html("deleted"), "<del>deleted</del>");
    }

    #[test]
    fn test_render_strikethrough_commonmark() {
        assert_eq!(render_strikethrough_commonmark("deleted"), "~~deleted~~");
    }
}
