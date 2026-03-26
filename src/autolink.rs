//! Autolink extension for GitHub Flavored Markdown
//!
//! This module implements GFM autolink parsing.
//!
//! GFM enables the autolink extension, where absolute URIs and email addresses
//! are automatically turned into links.
//!
//! Syntax:
//! ```markdown
//! Visit www.example.com for more info.
//!
//! Contact us at email@example.com
//!
//! https://github.com/user/repo
//! ```

use crate::arena::{Node, NodeArena, NodeId, TreeOps};
use crate::node::{NodeData, NodeType, SourcePos};

/// Regex patterns for URL detection (simplified)
const URL_SCHEMES: &[&str] = &["http://", "https://", "ftp://"];

/// Check if text contains a potential URL
pub fn contains_url(text: &str) -> bool {
    text.contains("www.")
        || text.contains('@')
        || URL_SCHEMES.iter().any(|s| text.contains(s))
}

/// Check if a character is valid in a URL
fn is_url_char(c: char) -> bool {
    matches!(c,
        'a'..='z' | 'A'..='Z' | '0'..='9' |
        '-' | '_' | '.' | '~' | ':' | '/' | '?' | '#' |
        '[' | ']' | '@' | '!' | '$' | '&' | '\'' |
        '(' | ')' | '*' | '+' | ',' | ';' | '=' | '%'
    )
}

/// Check if a character is trailing punctuation that should be excluded
fn is_trailing_punctuation(c: char) -> bool {
    matches!(c, '.' | ',' | '!' | '?' | ';' | ':' | ')' | ']' | '}')
}

/// Find URLs in text and return their positions
/// Returns vector of (start, end, url, is_email)
pub fn find_urls(text: &str) -> Vec<(usize, usize, String, bool)> {
    let mut urls = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // Check for scheme-based URLs (http://, https://, ftp://)
        let remaining: String = chars[i..].iter().collect();

        for scheme in URL_SCHEMES {
            if remaining.to_lowercase().starts_with(scheme) {
                let start = i;
                i += scheme.len();

                // Find end of URL
                while i < chars.len() && is_url_char(chars[i]) {
                    i += 1;
                }

                // Back up if we ended on trailing punctuation
                while i > start && is_trailing_punctuation(chars[i - 1]) {
                    i -= 1;
                }

                if i > start {
                    let url: String = chars[start..i].iter().collect();
                    urls.push((start, i, url, false));
                }
                break;
            }
        }

        // Check for www URLs
        if i < chars.len() && chars[i..].len() >= 4 {
            let next_chars: String =
                chars[i..i + 4.min(chars.len() - i)].iter().collect();
            if next_chars.eq_ignore_ascii_case("www.") {
                let start = i;
                i += 4;

                // Find end of URL
                while i < chars.len() && is_url_char(chars[i]) {
                    i += 1;
                }

                // Back up if we ended on trailing punctuation
                while i > start && is_trailing_punctuation(chars[i - 1]) {
                    i -= 1;
                }

                if i > start {
                    let url: String = chars[start..i].iter().collect();
                    urls.push((start, i, format!("https://{}", url), false));
                }
                continue;
            }
        }

        // Check for email addresses
        if i < chars.len() && chars[i].is_alphanumeric() {
            if let Some((start, end, email)) = try_parse_email(&chars, i) {
                urls.push((start, end, email, true));
                i = end;
                continue;
            }
        }

        i += 1;
    }

    urls
}

/// Try to parse an email address at the given position
fn try_parse_email(chars: &[char], start: usize) -> Option<(usize, usize, String)> {
    let mut i = start;

    // Local part (before @)
    while i < chars.len()
        && (chars[i].is_alphanumeric() || matches!(chars[i], '.' | '-' | '_'))
    {
        i += 1;
    }

    if i >= chars.len() || chars[i] != '@' {
        return None;
    }

    let _at_pos = i;
    i += 1; // Skip @

    // Domain part
    let domain_start = i;
    while i < chars.len()
        && (chars[i].is_alphanumeric() || matches!(chars[i], '.' | '-'))
    {
        i += 1;
    }

    // Must have at least one dot in domain
    if i <= domain_start + 1 {
        return None;
    }

    let domain: String = chars[domain_start..i].iter().collect();
    if !domain.contains('.') {
        return None;
    }

    let email: String = chars[start..i].iter().collect();
    Some((start, i, email))
}

/// Create an autolink node in the arena
/// Returns the NodeId of the created link node
pub fn create_autolink_node(
    arena: &mut NodeArena,
    url: &str,
    is_email: bool,
    line: u32,
    col: u32,
) -> NodeId {
    let display_url = url.to_string();

    let href = if is_email {
        format!("mailto:{}", url)
    } else {
        url.to_string()
    };

    let link_node = arena.alloc(Node::with_data(
        NodeType::Link,
        NodeData::Link {
            url: href,
            title: String::new(),
        },
    ));

    {
        let node = arena.get_mut(link_node);
        node.source_pos = SourcePos {
            start_line: line,
            start_column: col,
            end_line: line,
            end_column: col + display_url.len() as u32,
        };
    }

    // Create text node for the display text
    let text_node = arena.alloc(Node::with_data(
        NodeType::Text,
        NodeData::Text {
            literal: display_url,
        },
    ));

    TreeOps::append_child(arena, link_node, text_node);

    link_node
}

/// Process text for autolinks and return a list of node IDs
/// The nodes are allocated in the arena and returned as NodeIds
pub fn process_autolinks(
    arena: &mut NodeArena,
    text: &str,
    line: u32,
    col: u32,
) -> Vec<NodeId> {
    let urls = find_urls(text);
    if urls.is_empty() {
        // No URLs found, return single text node
        let node = arena.alloc(Node::with_data(
            NodeType::Text,
            NodeData::Text {
                literal: text.to_string(),
            },
        ));
        return vec![node];
    }

    let mut nodes = Vec::new();
    let mut last_end = 0;

    for (start, end, url, is_email) in urls {
        // Add text before URL
        if start > last_end {
            let before_text = &text[last_end..start];
            let node = arena.alloc(Node::with_data(
                NodeType::Text,
                NodeData::Text {
                    literal: before_text.to_string(),
                },
            ));
            nodes.push(node);
        }

        // Add autolink node
        let link_node = create_autolink_node(arena, &url, is_email, line, col + start as u32);
        nodes.push(link_node);

        last_end = end;
    }

    // Add remaining text after last URL
    if last_end < text.len() {
        let after_text = &text[last_end..];
        let node = arena.alloc(Node::with_data(
            NodeType::Text,
            NodeData::Text {
                literal: after_text.to_string(),
            },
        ));
        nodes.push(node);
    }

    nodes
}

/// Render autolink to HTML
pub fn render_autolink_html(url: &str, is_email: bool) -> String {
    let href = if is_email {
        format!("mailto:{}", url)
    } else {
        url.to_string()
    };

    format!(
        "<a href=\"{}\">{}</a>",
        crate::html_utils::escape_html(&href),
        crate::html_utils::escape_html(url)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains_url() {
        assert!(contains_url("Visit www.example.com"));
        assert!(contains_url("Check https://github.com"));
        assert!(contains_url("Email me@example.com"));
        assert!(!contains_url("Just plain text"));
    }

    #[test]
    fn test_find_http_url() {
        let urls = find_urls("Visit https://github.com/user/repo for more info.");
        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0].2, "https://github.com/user/repo");
        assert!(!urls[0].3); // not email
    }

    #[test]
    fn test_find_www_url() {
        let urls = find_urls("Visit www.example.com/page for more.");
        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0].2, "https://www.example.com/page");
    }

    #[test]
    fn test_find_email() {
        let urls = find_urls("Contact email@example.com for help.");
        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0].2, "email@example.com");
        assert!(urls[0].3); // is email
    }

    #[test]
    fn test_find_multiple_urls() {
        let urls = find_urls("Visit www.first.com and https://second.com.");
        assert_eq!(urls.len(), 2);
    }

    #[test]
    fn test_url_with_punctuation() {
        let urls = find_urls("Visit www.example.com.");
        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0].2, "https://www.example.com");
    }

    #[test]
    fn test_create_autolink_node() {
        let mut arena = NodeArena::new();
        let node_id = create_autolink_node(&mut arena, "https://example.com", false, 1, 1);
        let node = arena.get(node_id);
        assert_eq!(node.node_type, NodeType::Link);

        match &node.data {
            NodeData::Link { url, .. } => {
                assert_eq!(url, "https://example.com");
            }
            _ => panic!("Expected Link data"),
        }
    }

    #[test]
    fn test_create_email_autolink() {
        let mut arena = NodeArena::new();
        let node_id = create_autolink_node(&mut arena, "test@example.com", true, 1, 1);
        let node = arena.get(node_id);

        match &node.data {
            NodeData::Link { url, .. } => {
                assert_eq!(url, "mailto:test@example.com");
            }
            _ => panic!("Expected Link data"),
        }
    }

    #[test]
    fn test_process_autolinks() {
        let mut arena = NodeArena::new();
        let nodes = process_autolinks(&mut arena, "Visit https://example.com today", 1, 1);
        assert_eq!(nodes.len(), 3); // text, link, text

        assert_eq!(arena.get(nodes[0]).node_type, NodeType::Text);
        assert_eq!(arena.get(nodes[1]).node_type, NodeType::Link);
        assert_eq!(arena.get(nodes[2]).node_type, NodeType::Text);
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
    }
}
