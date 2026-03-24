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

use crate::node::{Node, NodeData, NodeType, SourcePos};
use std::cell::RefCell;
use std::rc::Rc;

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

/// Extract text content from a heading node
pub fn extract_heading_text(node: &Rc<RefCell<Node>>) -> String {
    let mut text = String::new();

    // Recursively collect text from children
    fn collect_text(node: &Rc<RefCell<Node>>, text: &mut String) {
        let node_ref = node.borrow();

        match &node_ref.data {
            NodeData::Text { literal } => {
                text.push_str(literal);
            }
            _ => {}
        }

        // Process children
        let first_child_opt = {
            let node_ref = node.borrow();
            let child = node_ref.first_child.borrow().as_ref().cloned();
            child
        };

        if let Some(first_child) = first_child_opt {
            collect_text(&first_child, text);

            // Collect siblings
            let first_next = {
                let child_ref = first_child.borrow();
                let next = child_ref.next.borrow().as_ref().cloned();
                next
            };
            let mut current_opt = first_next;
            while let Some(current) = current_opt {
                collect_text(&current, text);
                let next_opt = {
                    let curr_ref = current.borrow();
                    let next = curr_ref.next.borrow().as_ref().cloned();
                    next
                };
                current_opt = next_opt;
            }
        }
    }

    let first_child_opt = {
        let node_ref = node.borrow();
        let child = node_ref.first_child.borrow().as_ref().cloned();
        child
    };
    if let Some(first_child) = first_child_opt {
        collect_text(&first_child, &mut text);
    }

    text
}

/// Build TOC entries from document headings
pub fn build_toc(document: &Rc<RefCell<Node>>) -> Vec<TocEntry> {
    let mut entries = Vec::new();

    fn collect_headings(node: &Rc<RefCell<Node>>, entries: &mut Vec<TocEntry>) {
        // Check if this is a heading
        let is_heading = {
            let node_ref = node.borrow();
            matches!(node_ref.data, NodeData::Heading { .. })
        };

        if is_heading {
            let text = extract_heading_text(node);
            let anchor = generate_anchor(&text);
            let level = {
                let node_ref = node.borrow();
                match &node_ref.data {
                    NodeData::Heading { level, .. } => *level,
                    _ => 1,
                }
            };

            entries.push(TocEntry {
                level,
                text,
                anchor,
            });
        }

        // Process children
        let first_child_opt = {
            let node_ref = node.borrow();
            let child = node_ref.first_child.borrow().as_ref().cloned();
            child
        };

        if let Some(first_child) = first_child_opt {
            collect_headings(&first_child, entries);

            // Collect siblings
            let first_next = {
                let child_ref = first_child.borrow();
                let next = child_ref.next.borrow().as_ref().cloned();
                next
            };
            let mut current_opt = first_next;
            while let Some(current) = current_opt {
                collect_headings(&current, entries);
                let next_opt = {
                    let curr_ref = current.borrow();
                    let next = curr_ref.next.borrow().as_ref().cloned();
                    next
                };
                current_opt = next_opt;
            }
        }
    }

    collect_headings(document, &mut entries);
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

/// Create a TOC placeholder node
pub fn create_toc_node(line: u32, col: u32) -> Rc<RefCell<Node>> {
    let node = Rc::new(RefCell::new(Node::new(NodeType::CustomBlock)));
    {
        let mut node_ref = node.borrow_mut();
        node_ref.data = NodeData::CustomBlock {
            on_enter: "<nav class=\"toc\">".to_string(),
            on_exit: "</nav>".to_string(),
        };
        node_ref.source_pos = SourcePos {
            start_line: line,
            start_column: col,
            end_line: line,
            end_column: col + 5, // [TOC] is 5 chars
        };
    }
    node
}

#[cfg(test)]
mod tests {
    use super::*;

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
