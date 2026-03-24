//! Definition list extension for Markdown
//!
//! This module implements definition list parsing.
//!
//! Syntax:
//! ```markdown
//! Term
//! : Definition 1
//! : Definition 2
//!
//! Another Term
//! : Another definition
//! ```

use crate::node::{append_child, Node, NodeData, NodeType, SourcePos};
use std::cell::RefCell;
use std::rc::Rc;

/// A definition list entry
#[derive(Debug, Clone)]
pub struct DefinitionEntry {
    /// The term being defined
    pub term: String,
    /// The definitions for this term
    pub definitions: Vec<String>,
}

/// Check if a line is a definition term (not starting with ':')
pub fn is_definition_term(line: &str) -> bool {
    let trimmed = line.trim_start();
    !trimmed.is_empty() && !trimmed.starts_with(':')
}

/// Check if a line is a definition
/// Format: : Definition text
pub fn is_definition_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with(':')
}

/// Parse a definition line
/// Returns the definition text (without the leading ':')
pub fn parse_definition_line(line: &str) -> Option<String> {
    if !is_definition_line(line) {
        return None;
    }

    let trimmed = line.trim_start();
    if trimmed.len() < 2 {
        return Some(String::new());
    }

    // Skip ':' and optional whitespace
    let content = &trimmed[1..].trim_start();
    Some(content.to_string())
}

/// Try to parse a definition list from lines
/// Returns (entries, lines_consumed) if successful
pub fn try_parse_definition_list(
    lines: &[&str],
    _start_line: usize,
) -> Option<(Vec<DefinitionEntry>, usize)> {
    if lines.is_empty() {
        return None;
    }

    let mut entries = Vec::new();
    let mut i = 0;
    let mut lines_consumed = 0;

    while i < lines.len() {
        // Look for a term
        if !is_definition_term(lines[i]) {
            break;
        }

        let term = lines[i].trim().to_string();
        i += 1;
        lines_consumed += 1;

        // Collect definitions for this term
        let mut definitions = Vec::new();

        while i < lines.len() && is_definition_line(lines[i]) {
            if let Some(def) = parse_definition_line(lines[i]) {
                definitions.push(def);
            }
            i += 1;
            lines_consumed += 1;
        }

        // Skip empty lines between entries
        while i < lines.len() && lines[i].trim().is_empty() {
            i += 1;
            lines_consumed += 1;
        }

        if !term.is_empty() && !definitions.is_empty() {
            entries.push(DefinitionEntry { term, definitions });
        }
    }

    if entries.is_empty() {
        None
    } else {
        Some((entries, lines_consumed))
    }
}

/// Create a definition list node
pub fn create_definition_list_node(
    entries: &[DefinitionEntry],
    start_line: u32,
    _start_col: u32,
) -> Rc<RefCell<Node>> {
    let list_node = Rc::new(RefCell::new(Node::new(NodeType::CustomBlock)));
    {
        let mut node_ref = list_node.borrow_mut();
        node_ref.data = NodeData::CustomBlock {
            on_enter: "<dl>".to_string(),
            on_exit: "</dl>".to_string(),
        };
        node_ref.source_pos = SourcePos {
            start_line,
            start_column: 1,
            end_line: start_line,
            end_column: 1,
        };
    }

    for entry in entries {
        // Create term node (<dt>)
        let term_node = Rc::new(RefCell::new(Node::new(NodeType::CustomInline)));
        term_node.borrow_mut().data = NodeData::CustomInline {
            on_enter: "<dt>".to_string(),
            on_exit: "</dt>".to_string(),
        };

        // Create text node for term
        let text_node = Rc::new(RefCell::new(Node::new(NodeType::Text)));
        text_node.borrow_mut().data = NodeData::Text {
            literal: entry.term.clone(),
        };

        append_child(&term_node, text_node);
        append_child(&list_node, term_node);

        // Create definition nodes (<dd>)
        for def in &entry.definitions {
            let def_node = Rc::new(RefCell::new(Node::new(NodeType::CustomInline)));
            def_node.borrow_mut().data = NodeData::CustomInline {
                on_enter: "<dd>".to_string(),
                on_exit: "</dd>".to_string(),
            };

            // Create text node for definition
            let text_node = Rc::new(RefCell::new(Node::new(NodeType::Text)));
            text_node.borrow_mut().data = NodeData::Text {
                literal: def.clone(),
            };

            append_child(&def_node, text_node);
            append_child(&list_node, def_node);
        }
    }

    list_node
}

/// Render definition list to HTML
pub fn render_definition_list_html(entries: &[DefinitionEntry]) -> String {
    if entries.is_empty() {
        return String::new();
    }

    let mut html = String::from("<dl>\n");

    for entry in entries {
        html.push_str(&format!(
            "<dt>{}</dt>\n",
            crate::html_utils::escape_html(&entry.term)
        ));

        for def in &entry.definitions {
            html.push_str(&format!(
                "<dd>{}</dd>\n",
                crate::html_utils::escape_html(def)
            ));
        }
    }

    html.push_str("</dl>");
    html
}

/// Render definition list to CommonMark
pub fn render_definition_list_commonmark(entries: &[DefinitionEntry]) -> String {
    if entries.is_empty() {
        return String::new();
    }

    let mut md = String::new();

    for entry in entries {
        md.push_str(&format!("{}\n", entry.term));

        for def in &entry.definitions {
            md.push_str(&format!(": {}\n", def));
        }

        md.push('\n');
    }

    md
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_definition_term() {
        assert!(is_definition_term("Term"));
        assert!(is_definition_term("  Term with spaces"));
        assert!(!is_definition_term(": Definition"));
        assert!(!is_definition_term(""));
        assert!(!is_definition_term("  "));
    }

    #[test]
    fn test_is_definition_line() {
        assert!(is_definition_line(": Definition"));
        assert!(is_definition_line("  : Indented definition"));
        assert!(!is_definition_line("Term"));
        assert!(!is_definition_line(""));
    }

    #[test]
    fn test_parse_definition_line() {
        assert_eq!(
            parse_definition_line(": Definition text"),
            Some("Definition text".to_string())
        );
        assert_eq!(
            parse_definition_line("  :  Indented definition  "),
            Some("Indented definition  ".to_string())
        );
        assert_eq!(parse_definition_line(":"), Some(String::new()));
        assert_eq!(parse_definition_line("Not a definition"), None);
    }

    #[test]
    fn test_try_parse_definition_list() {
        let lines = vec![
            "Term 1",
            ": Definition 1",
            ": Definition 2",
            "",
            "Term 2",
            ": Definition 3",
        ];

        let result = try_parse_definition_list(&lines, 1);
        assert!(result.is_some());

        let (entries, _lines_consumed) = result.unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].term, "Term 1");
        assert_eq!(entries[0].definitions.len(), 2);
        assert_eq!(entries[1].term, "Term 2");
    }

    #[test]
    fn test_render_definition_list_html() {
        let entries = vec![DefinitionEntry {
            term: "HTML".to_string(),
            definitions: vec![
                "Hyper Text Markup Language".to_string(),
                "A standard markup language".to_string(),
            ],
        }];

        let html = render_definition_list_html(&entries);
        assert!(html.contains("<dl>"));
        assert!(html.contains("<dt>HTML</dt>"));
        assert!(html.contains("<dd>Hyper Text Markup Language</dd>"));
        assert!(html.contains("</dl>"));
    }

    #[test]
    fn test_render_definition_list_commonmark() {
        let entries = vec![DefinitionEntry {
            term: "Term".to_string(),
            definitions: vec!["Definition".to_string()],
        }];

        let md = render_definition_list_commonmark(&entries);
        assert!(md.contains("Term"));
        assert!(md.contains(": Definition"));
    }

    #[test]
    fn test_empty_definition_list() {
        let entries: Vec<DefinitionEntry> = vec![];
        assert!(render_definition_list_html(&entries).is_empty());
        assert!(render_definition_list_commonmark(&entries).is_empty());
    }
}
