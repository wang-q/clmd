//! Autolink extension for automatically linking URLs and email addresses
//!
//! This module provides functionality to automatically detect and link URLs
//! and email addresses in text content, similar to GFM autolinks.
//!
//! # Example
//!
//! ```markdown
//! Visit https://example.com for more info.
//! Contact us at support@example.com
//! ```
//!
//! Will be rendered as:
//! ```html
//! <p>Visit <a href="https://example.com">https://example.com</a> for more info.
//! Contact us at <a href="mailto:support@example.com">support@example.com</a></p>
//! ```

use crate::arena::{Node, NodeArena, NodeId, TreeOps};
use crate::node_value::{NodeLink, NodeValue, SourcePos};

/// Regex patterns for URL detection (simplified)
/// In a full implementation, this would use more sophisticated pattern matching
const URL_SCHEMES: &[&str] = &["http://", "https://", "ftp://", "mailto:"];

/// Check if text looks like a URL
fn looks_like_url(text: &str) -> bool {
    URL_SCHEMES.iter().any(|scheme| text.starts_with(scheme))
        || (text.contains('.')
            && text.contains("//")
            && !text.contains(' ')
            && text.len() > 4)
}

/// Check if text looks like an email address
fn looks_like_email(text: &str) -> bool {
    // Simple email validation: contains @ and a dot after @
    if let Some(at_pos) = text.find('@') {
        // Must have at least one character before @
        if at_pos == 0 {
            return false;
        }
        let after_at = &text[at_pos + 1..];
        // Must have at least one character after @ and contain a dot
        !after_at.is_empty() && after_at.contains('.') && !text.contains(' ')
    } else {
        false
    }
}

/// Find URL or email patterns in text and return positions
pub fn find_autolinks(text: &str) -> Vec<(usize, usize, bool)> {
    let mut results = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // Try to find start of potential URL/email
        if chars[i].is_alphanumeric() || chars[i] == '/' {
            // Find the end of the current word (whitespace or certain punctuation)
            let word_start = i;
            while i < chars.len()
                && !chars[i].is_whitespace()
                && chars[i] != '<'
                && chars[i] != '>'
                && chars[i] != '"'
            {
                i += 1;
            }
            let word_end = i;

            // Get the word
            let word: String = chars[word_start..word_end].iter().collect();

            // Check for URLs
            if looks_like_url(&word) {
                // Trim trailing punctuation
                let mut end = word_end;
                while end > word_start && ".!?,:;".contains(chars[end - 1]) {
                    end -= 1;
                }
                if end > word_start {
                    results.push((word_start, end, false));
                }
                continue;
            }

            // Check for emails
            if looks_like_email(&word) {
                results.push((word_start, word_end, true));
                continue;
            }
        } else {
            i += 1;
        }
    }

    results
}

/// Create an autolink node
pub fn create_autolink_node(
    arena: &mut NodeArena,
    url: &str,
    is_email: bool,
    line: u32,
    col: u32,
) -> NodeId {
    let display_url = url.to_string();
    let actual_url = if is_email {
        format!("mailto:{}", url)
    } else {
        url.to_string()
    };

    let link_node = arena.alloc(Node::with_value(NodeValue::Link(NodeLink {
        url: actual_url,
        title: String::new(),
    })));

    {
        let node = arena.get_mut(link_node);
        node.source_pos = SourcePos::new(
            line as usize,
            col as usize,
            line as usize,
            (col + display_url.len() as u32 + 4) as usize, // +4 for the ~~ delimiters
        );
    }

    // Create text node for the display text
    let text_node = arena.alloc(Node::with_value(NodeValue::Text(display_url)));

    TreeOps::append_child(arena, link_node, text_node);

    link_node
}

/// Process text for autolinks and return a list of node IDs
pub fn process_autolinks(
    arena: &mut NodeArena,
    text: &str,
    line: u32,
    col: u32,
) -> Vec<NodeId> {
    let mut nodes = Vec::new();
    let links = find_autolinks(text);

    if links.is_empty() {
        // No autolinks found, return single text node
        nodes.push(arena.alloc(Node::with_value(NodeValue::Text(text.to_string()))));
        return nodes;
    }

    let mut last_end = 0;
    let chars: Vec<char> = text.chars().collect();

    for (start, end, is_email) in links {
        // Add text before the link
        if start > last_end {
            let before: String = chars[last_end..start].iter().collect();
            if !before.is_empty() {
                nodes.push(arena.alloc(Node::with_value(NodeValue::Text(before))));
            }
        }

        // Add the link
        let link_text: String = chars[start..end].iter().collect();
        let link_node =
            create_autolink_node(arena, &link_text, is_email, line, col + start as u32);
        nodes.push(link_node);

        last_end = end;
    }

    // Add remaining text after last link
    if last_end < chars.len() {
        let after: String = chars[last_end..].iter().collect();
        if !after.is_empty() {
            nodes.push(arena.alloc(Node::with_value(NodeValue::Text(after))));
        }
    }

    nodes
}

/// Render autolink as HTML
pub fn render_autolink_html(url: &str, is_email: bool) -> String {
    let actual_url = if is_email {
        format!("mailto:{}", url)
    } else {
        url.to_string()
    };

    format!(
        r#"<a href="{}">{}</a>"#,
        escape_html(&actual_url),
        escape_html(url)
    )
}

/// Simple HTML escaping
fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Check if autolinks extension is enabled
pub fn is_enabled(options: u32) -> bool {
    // Autolinks are enabled when the 0x01 bit is set
    (options & 0x01) != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_looks_like_url() {
        assert!(looks_like_url("https://example.com"));
        assert!(looks_like_url("http://example.com"));
        assert!(looks_like_url("ftp://files.example.com"));
        assert!(!looks_like_url("just text"));
        assert!(!looks_like_url("example.com")); // No scheme
    }

    #[test]
    fn test_looks_like_email() {
        assert!(looks_like_email("test@example.com"));
        assert!(looks_like_email("user.name@sub.domain.org"));
        assert!(!looks_like_email("not an email"));
        assert!(!looks_like_email("@example.com"));
        assert!(!looks_like_email("test@"));
    }

    #[test]
    fn test_find_autolinks() {
        let text = "Visit https://example.com for more info";
        let links = find_autolinks(text);
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].0, 6); // Start position
        assert_eq!(links[0].1, 25); // End position
        assert!(!links[0].2); // Not email
    }

    #[test]
    fn test_find_email_autolinks() {
        let text = "Contact support@example.com for help";
        let links = find_autolinks(text);
        assert_eq!(links.len(), 1);
        assert!(links[0].2); // Is email
    }

    #[test]
    fn test_find_no_autolinks() {
        let text = "Just some plain text without any links";
        let links = find_autolinks(text);
        assert!(links.is_empty());
    }

    #[test]
    fn test_create_autolink_node() {
        let mut arena = NodeArena::new();
        let node_id =
            create_autolink_node(&mut arena, "https://example.com", false, 1, 1);
        let node = arena.get(node_id);

        // Check it's a Link node
        assert!(matches!(node.value, NodeValue::Link(..)));

        // Check the URL
        if let NodeValue::Link(ref link) = node.value {
            assert_eq!(link.url, "https://example.com");
        } else {
            panic!("Expected Link value");
        }
    }

    #[test]
    fn test_create_email_autolink() {
        let mut arena = NodeArena::new();
        let node_id = create_autolink_node(&mut arena, "test@example.com", true, 1, 1);
        let node = arena.get(node_id);

        // Check the URL has mailto: prefix
        if let NodeValue::Link(ref link) = node.value {
            assert_eq!(link.url, "mailto:test@example.com");
        } else {
            panic!("Expected Link value");
        }
    }

    #[test]
    fn test_process_autolinks() {
        let mut arena = NodeArena::new();
        let nodes =
            process_autolinks(&mut arena, "Visit https://example.com today", 1, 1);
        assert_eq!(nodes.len(), 3); // text, link, text

        // Check first node is text
        assert!(matches!(arena.get(nodes[0]).value, NodeValue::Text(..)));
        // Check second node is link
        assert!(matches!(arena.get(nodes[1]).value, NodeValue::Link(..)));
        // Check third node is text
        assert!(matches!(arena.get(nodes[2]).value, NodeValue::Text(..)));
    }

    #[test]
    fn test_render_autolink_html() {
        let html = render_autolink_html("https://example.com", false);
        assert!(html.contains("<a href=\"https://example.com\""));
        assert!(html.contains(">https://example.com</a>"));
    }

    #[test]
    fn test_render_email_html() {
        let html = render_autolink_html("test@example.com", true);
        assert!(html.contains("<a href=\"mailto:test@example.com\""));
        assert!(html.contains(">test@example.com</a>"));
    }

    #[test]
    fn test_escape_html() {
        assert_eq!(escape_html("<script>"), "&lt;script&gt;");
        assert_eq!(escape_html("\"quoted\""), "&quot;quoted&quot;");
        assert_eq!(escape_html("A & B"), "A &amp; B");
    }

    #[test]
    fn test_is_enabled() {
        assert!(is_enabled(0x01));
        assert!(!is_enabled(0));
        assert!(is_enabled(0x01 | 0x02));
    }
}
