//! Table of Contents (TOC) extension for Markdown
//!
//! This module implements TOC generation from document headings.
//!
//! Syntax:
//! ```markdown
//! [TOC]
//! ```
//!
//! Or using HTML comment:
//! ```markdown
//! <!-- TOC -->
//! ```

use crate::arena::{NodeArena, NodeId};
use crate::node::{NodeData, NodeType};

/// A TOC entry representing a heading
#[derive(Debug, Clone)]
pub struct TocEntry {
    /// Heading level (1-6)
    pub level: u32,
    /// Heading text content
    pub text: String,
    /// Anchor ID for linking
    pub anchor: String,
}

/// Check if text is a TOC marker
pub fn is_toc_marker(text: &str) -> bool {
    let trimmed = text.trim();
    trimmed == "[TOC]"
        || trimmed == "[[TOC]]"
        || trimmed.eq_ignore_ascii_case("<!-- toc -->")
        || trimmed.eq_ignore_ascii_case("<!--toc-->")
}

/// Generate an anchor ID from heading text
pub fn generate_anchor(text: &str) -> String {
    let mut anchor = String::new();

    for ch in text.to_lowercase().chars() {
        match ch {
            'a'..='z' | '0'..='9' => anchor.push(ch),
            ' ' | '-' | '_' => {
                if !anchor.ends_with('-') {
                    anchor.push('-');
                }
            }
            _ => {} // Skip other characters
        }
    }

    // Remove trailing dash
    while anchor.ends_with('-') {
        anchor.pop();
    }

    // Ensure it starts with a letter
    if anchor.is_empty() {
        anchor = "heading".to_string();
    } else if anchor.chars().next().unwrap().is_ascii_digit() {
        anchor = format!("h-{}", anchor);
    }

    anchor
}

/// Extract text content from a heading node in the arena
pub fn extract_heading_text(arena: &NodeArena, node_id: NodeId) -> String {
    let mut text = String::new();

    // Recursively collect text from children
    fn collect_text(arena: &NodeArena, node_id: NodeId, text: &mut String) {
        let node = arena.get(node_id);

        if let NodeData::Text { literal } = &node.data {
            text.push_str(literal);
        }

        // Process children
        let mut current_opt = node.first_child;
        while let Some(child_id) = current_opt {
            collect_text(arena, child_id, text);
            current_opt = arena.get(child_id).next;
        }
    }

    let node = arena.get(node_id);
    if let Some(first_child) = node.first_child {
        collect_text(arena, first_child, &mut text);
    }

    text
}

/// Build TOC entries from document headings in the arena
pub fn build_toc(arena: &NodeArena, document_id: NodeId) -> Vec<TocEntry> {
    let mut entries = Vec::new();

    fn collect_headings(
        arena: &NodeArena,
        node_id: NodeId,
        entries: &mut Vec<TocEntry>,
    ) {
        let node = arena.get(node_id);

        // Check if this is a heading
        if matches!(node.data, NodeData::Heading { .. }) {
            let text = extract_heading_text(arena, node_id);
            let anchor = generate_anchor(&text);
            let level = match &node.data {
                NodeData::Heading { level, .. } => *level,
                _ => 1,
            };

            entries.push(TocEntry {
                level,
                text,
                anchor,
            });
        }

        // Process children
        let mut current_opt = node.first_child;
        while let Some(child_id) = current_opt {
            collect_headings(arena, child_id, entries);
            current_opt = arena.get(child_id).next;
        }
    }

    collect_headings(arena, document_id, &mut entries);
    entries
}

/// Render TOC entries to HTML
pub fn render_toc_html(entries: &[TocEntry]) -> String {
    if entries.is_empty() {
        return String::new();
    }

    let mut html = String::from("<nav class=\"toc\">\n<ul>\n");
    let mut prev_level = 1u32;

    for entry in entries {
        let level = entry.level;

        // Handle nesting
        if level > prev_level {
            for _ in prev_level..level {
                html.push_str("<ul>\n");
            }
        } else if level < prev_level {
            for _ in level..prev_level {
                html.push_str("</ul>\n");
            }
        }

        html.push_str(&format!(
            "<li><a href=\"#{}\">{}</a></li>\n",
            entry.anchor,
            crate::html_utils::escape_html(&entry.text)
        ));

        prev_level = level;
    }

    // Close any remaining open lists
    for _ in 1..prev_level {
        html.push_str("</ul>\n");
    }

    html.push_str("</ul>\n</nav>");
    html
}

/// Render TOC entries to CommonMark
pub fn render_toc_commonmark(entries: &[TocEntry]) -> String {
    if entries.is_empty() {
        return String::new();
    }

    let mut md = String::from("## Table of Contents\n\n");

    for entry in entries {
        let indent = "  ".repeat((entry.level - 1) as usize);
        md.push_str(&format!(
            "{}- [{}](#{})\n",
            indent, entry.text, entry.anchor
        ));
    }

    md
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arena::{Node, NodeArena, TreeOps};

    #[test]
    fn test_is_toc_marker() {
        assert!(is_toc_marker("[TOC]"));
        assert!(is_toc_marker("[[TOC]]"));
        assert!(is_toc_marker("<!-- TOC -->"));
        assert!(is_toc_marker("<!--toc-->"));
        assert!(is_toc_marker("  [TOC]  "));
        assert!(!is_toc_marker("Not TOC"));
        assert!(!is_toc_marker("[toc]")); // Case sensitive for brackets
    }

    #[test]
    fn test_generate_anchor() {
        assert_eq!(generate_anchor("Hello World"), "hello-world");
        assert_eq!(generate_anchor("Multiple   Spaces"), "multiple-spaces");
        assert_eq!(generate_anchor("Special!@#Chars"), "specialchars");
        assert_eq!(generate_anchor("123 Number"), "h-123-number");
        assert_eq!(generate_anchor(""), "heading");
    }

    #[test]
    fn test_extract_heading_text() {
        let mut arena = NodeArena::new();
        let heading = arena.alloc(Node::with_data(
            NodeType::Heading,
            NodeData::Heading {
                level: 1,
                content: "Test Heading".to_string(),
            },
        ));
        let text = arena.alloc(Node::with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "Test Heading".to_string(),
            },
        ));
        TreeOps::append_child(&mut arena, heading, text);

        let extracted = extract_heading_text(&arena, heading);
        assert_eq!(extracted, "Test Heading");
    }

    #[test]
    fn test_build_toc() {
        let mut arena = NodeArena::new();
        let doc = arena.alloc(Node::new(NodeType::Document));

        // Create headings
        for i in 1..=3 {
            let heading = arena.alloc(Node::with_data(
                NodeType::Heading,
                NodeData::Heading {
                    level: i,
                    content: format!("Heading {}", i),
                },
            ));
            let text = arena.alloc(Node::with_data(
                NodeType::Text,
                NodeData::Text {
                    literal: format!("Heading {}", i),
                },
            ));
            TreeOps::append_child(&mut arena, heading, text);
            TreeOps::append_child(&mut arena, doc, heading);
        }

        let entries = build_toc(&arena, doc);
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].level, 1);
        assert_eq!(entries[1].level, 2);
        assert_eq!(entries[2].level, 3);
    }

    #[test]
    fn test_render_toc_html() {
        let entries = vec![
            TocEntry {
                level: 1,
                text: "Introduction".to_string(),
                anchor: "introduction".to_string(),
            },
            TocEntry {
                level: 2,
                text: "Getting Started".to_string(),
                anchor: "getting-started".to_string(),
            },
            TocEntry {
                level: 2,
                text: "Advanced Usage".to_string(),
                anchor: "advanced-usage".to_string(),
            },
        ];

        let html = render_toc_html(&entries);
        assert!(html.contains("<nav class=\"toc\">"));
        assert!(html.contains("<a href=\"#introduction\">Introduction</a>"));
        assert!(html.contains("<a href=\"#getting-started\">Getting Started</a>"));
    }

    #[test]
    fn test_render_toc_commonmark() {
        let entries = vec![
            TocEntry {
                level: 1,
                text: "Introduction".to_string(),
                anchor: "introduction".to_string(),
            },
            TocEntry {
                level: 2,
                text: "Section".to_string(),
                anchor: "section".to_string(),
            },
        ];

        let md = render_toc_commonmark(&entries);
        assert!(md.contains("## Table of Contents"));
        assert!(md.contains("- [Introduction](#introduction)"));
        assert!(md.contains("  - [Section](#section)"));
    }

    #[test]
    fn test_empty_toc() {
        let entries: Vec<TocEntry> = vec![];
        assert!(render_toc_html(&entries).is_empty());
        assert!(render_toc_commonmark(&entries).is_empty());
    }
}
