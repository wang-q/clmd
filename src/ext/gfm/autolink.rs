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

use crate::core::arena::{Node, NodeArena, NodeId, TreeOps};
use crate::core::nodes::{NodeLink, NodeValue, SourcePos};
use once_cell::sync::Lazy;
use regex::Regex;

/// URL regex pattern - matches common URL schemes
/// Based on GFM autolink specification
static URL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"https?://[a-zA-Z0-9][-a-zA-Z0-9]*\.[-a-zA-Z0-9.]+[^\s<>"{}\[\]]*"#)
        .unwrap()
});

/// Email regex pattern - matches email addresses
/// Based on GFM autolink specification
static EMAIL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*"#).unwrap()
});

/// Find URL or email patterns in text and return positions
/// Uses regex for efficient matching
pub fn find_autolinks(text: &str) -> Vec<(usize, usize, bool)> {
    let mut results = Vec::new();

    // Find all URLs
    for mat in URL_REGEX.find_iter(text) {
        let start = mat.start();
        let mut end = mat.end();

        // Trim trailing punctuation that shouldn't be part of the URL
        let matched_text = &text[start..end];
        let trimmed_end = matched_text.trim_end_matches(|c: char| {
            c == '.' || c == '!' || c == '?' || c == ',' || c == ':' || c == ';'
        });
        end = start + trimmed_end.len();

        if end > start {
            results.push((start, end, false));
        }
    }

    // Find all emails
    for mat in EMAIL_REGEX.find_iter(text) {
        results.push((mat.start(), mat.end(), true));
    }

    // Sort by position and remove duplicates/overlapping matches
    results.sort_by_key(|(start, _, _)| *start);

    // Remove overlapping matches (prefer URLs over emails if they overlap)
    let mut filtered = Vec::new();
    for (start, end, is_email) in results {
        // Check if this overlaps with any already filtered result
        let mut overlaps = false;
        for (fs, fe, _) in &filtered {
            if start < *fe && *fs < end {
                overlaps = true;
                break;
            }
        }
        if !overlaps {
            filtered.push((start, end, is_email));
        }
    }

    filtered
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

    let link_node = arena.alloc(Node::with_value(NodeValue::link(NodeLink {
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
    let text_node = arena.alloc(Node::with_value(NodeValue::Text(display_url.into())));

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
        nodes.push(
            arena.alloc(Node::with_value(NodeValue::Text(text.to_string().into()))),
        );
        return nodes;
    }

    let mut last_end = 0;
    let chars: Vec<char> = text.chars().collect();

    for (start, end, is_email) in links {
        // Add text before the link
        if start > last_end {
            let before: String = chars[last_end..start].iter().collect();
            if !before.is_empty() {
                nodes
                    .push(arena.alloc(Node::with_value(NodeValue::Text(before.into()))));
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
            nodes.push(arena.alloc(Node::with_value(NodeValue::Text(after.into()))));
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
    fn test_url_regex() {
        // Test URL regex matching
        assert!(URL_REGEX.is_match("https://example.com"));
        assert!(URL_REGEX.is_match("http://example.com"));
        assert!(!URL_REGEX.is_match("just text"));
        assert!(!URL_REGEX.is_match("example.com")); // No scheme
    }

    #[test]
    fn test_email_regex() {
        // Test email regex matching
        assert!(EMAIL_REGEX.is_match("test@example.com"));
        assert!(EMAIL_REGEX.is_match("user.name@sub.domain.org"));
        assert!(!EMAIL_REGEX.is_match("not an email"));
        assert!(!EMAIL_REGEX.is_match("@example.com"));
        assert!(!EMAIL_REGEX.is_match("test@"));
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
