//! Footnote extension for Markdown
//!
//! This module implements footnote parsing.
//! Footnotes allow referencing additional content at the bottom of the document.
//!
//! Syntax:
//! ```markdown
//! Here is some text with a footnote[^1].
//!
//! [^1]: This is the footnote content.
//! ```
//!
//! Or using inline footnotes:
//! ```markdown
//! Here is some text with an inline footnote^[This is the footnote content.].
//! ```

use crate::node::{append_child, Node, NodeData, NodeType, SourcePos};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// A footnote reference in the text: [^label]
#[derive(Debug, Clone)]
pub struct FootnoteRef {
    /// The footnote label/id
    pub label: String,
    /// The ordinal number (assigned during rendering)
    pub ordinal: usize,
}

/// A footnote definition block: [^label]: content
#[derive(Debug, Clone)]
pub struct FootnoteDef {
    /// The footnote label/id
    pub label: String,
    /// The ordinal number (assigned during rendering)
    pub ordinal: usize,
    /// Number of references to this footnote
    pub ref_count: usize,
}

/// Check if text contains a footnote reference
pub fn contains_footnote_ref(text: &str) -> bool {
    text.contains("[^")
}

/// Parse a footnote reference from text
/// Returns Some(label) if found at the beginning
pub fn parse_footnote_ref(text: &str) -> Option<(String, usize)> {
    let trimmed = text.trim_start();
    
    if let Some(start) = trimmed.find("[^") {
        let after_bracket = start + 2;
        if let Some(end) = trimmed[after_bracket..].find(']') {
            let label = trimmed[after_bracket..after_bracket + end].trim().to_string();
            if !label.is_empty() {
                let full_len = after_bracket + end + 1 - start;
                return Some((label, full_len));
            }
        }
    }
    
    None
}

/// Find all footnote references in text
/// Returns vector of (start_pos, end_pos, label)
pub fn find_footnote_refs(text: &str) -> Vec<(usize, usize, String)> {
    let mut refs = Vec::new();
    let mut chars = text.char_indices().peekable();
    
    while let Some((pos, ch)) = chars.next() {
        if ch == '[' {
            if let Some(&(_, next_ch)) = chars.peek() {
                if next_ch == '^' {
                    chars.next(); // consume '^'
                    let label_start = pos + 2;
                    
                    // Find closing bracket
                    let mut label_end = label_start;
                    let mut found_close = false;
                    
                    while let Some((end_pos, end_ch)) = chars.next() {
                        if end_ch == ']' {
                            label_end = end_pos;
                            found_close = true;
                            break;
                        }
                    }
                    
                    if found_close && label_end > label_start {
                        let label = text[label_start..label_end].trim().to_string();
                        if !label.is_empty() {
                            refs.push((pos, label_end + 1, label));
                        }
                    }
                }
            }
        }
    }
    
    refs
}

/// Check if a line is a footnote definition
/// Format: [^label]: content
pub fn is_footnote_def(line: &str) -> bool {
    let trimmed = line.trim_start();
    
    if let Some(start) = trimmed.find("[^") {
        if start > 0 {
            return false; // Must be at the start of the line
        }
        
        let after_bracket = start + 2;
        if let Some(bracket_end) = trimmed[after_bracket..].find(']') {
            let label = trimmed[after_bracket..after_bracket + bracket_end].trim();
            if !label.is_empty() {
                let after_label = after_bracket + bracket_end + 1;
                if trimmed[after_label..].starts_with(':') {
                    return true;
                }
            }
        }
    }
    
    false
}

/// Parse a footnote definition line
/// Returns (label, content, consumed_chars)
pub fn parse_footnote_def(line: &str) -> Option<(String, String)> {
    if !is_footnote_def(line) {
        return None;
    }
    
    let trimmed = line.trim_start();
    let start = trimmed.find("[^").unwrap();
    let after_bracket = start + 2;
    let bracket_end = trimmed[after_bracket..].find(']').unwrap();
    
    let label = trimmed[after_bracket..after_bracket + bracket_end].trim().to_string();
    let after_label = after_bracket + bracket_end + 1;
    
    // Skip the colon and whitespace
    let content_start = after_label + 1;
    let content = trimmed[content_start..].trim().to_string();
    
    Some((label, content))
}

/// Create a footnote reference node
pub fn create_footnote_ref_node(label: &str, line: u32, col: u32) -> Rc<RefCell<Node>> {
    let node = Rc::new(RefCell::new(Node::new(NodeType::FootnoteRef)));
    {
        let mut node_ref = node.borrow_mut();
        node_ref.data = NodeData::FootnoteRef {
            label: label.to_string(),
            ordinal: 0, // Will be assigned during rendering
        };
        node_ref.source_pos = SourcePos {
            start_line: line,
            start_column: col,
            end_line: line,
            end_column: col + label.len() as u32 + 4, // +4 for [^ and ]
        };
    }
    node
}

/// Create a footnote definition node
pub fn create_footnote_def_node(label: &str, content: &str, line: u32, col: u32) -> Rc<RefCell<Node>> {
    let node = Rc::new(RefCell::new(Node::new(NodeType::FootnoteDef)));
    {
        let mut node_ref = node.borrow_mut();
        node_ref.data = NodeData::FootnoteDef {
            label: label.to_string(),
            ordinal: 0, // Will be assigned during rendering
            ref_count: 0,
        };
        node_ref.source_pos = SourcePos {
            start_line: line,
            start_column: col,
            end_line: line,
            end_column: col + label.len() as u32 + content.len() as u32 + 5, // +5 for [^, ], :, and space
        };
    }
    
    // Create text node for the content
    if !content.is_empty() {
        let text_node = Rc::new(RefCell::new(Node::new(NodeType::Text)));
        text_node.borrow_mut().data = NodeData::Text {
            literal: content.to_string(),
        };
        append_child(&node, text_node);
    }
    
    node
}

/// Footnote registry for managing footnotes during parsing
#[derive(Debug, Default)]
pub struct FootnoteRegistry {
    /// Map from label to footnote definition
    pub defs: HashMap<String, FootnoteDef>,
    /// Ordered list of labels
    pub ordered_labels: Vec<String>,
    /// Next ordinal to assign
    next_ordinal: usize,
}

impl FootnoteRegistry {
    /// Create a new footnote registry
    pub fn new() -> Self {
        Self {
            defs: HashMap::new(),
            ordered_labels: Vec::new(),
            next_ordinal: 1,
        }
    }
    
    /// Register a footnote definition
    pub fn register_def(&mut self, label: &str) -> usize {
        if let Some(def) = self.defs.get(label) {
            return def.ordinal;
        }
        
        let ordinal = self.next_ordinal;
        self.next_ordinal += 1;
        
        self.defs.insert(label.to_string(), FootnoteDef {
            label: label.to_string(),
            ordinal,
            ref_count: 0,
        });
        self.ordered_labels.push(label.to_string());
        
        ordinal
    }
    
    /// Increment reference count for a footnote
    pub fn add_ref(&mut self, label: &str) -> Option<usize> {
        if let Some(def) = self.defs.get_mut(label) {
            def.ref_count += 1;
            Some(def.ordinal)
        } else {
            None
        }
    }
    
    /// Get the ordinal for a label
    pub fn get_ordinal(&self, label: &str) -> Option<usize> {
        self.defs.get(label).map(|d| d.ordinal)
    }
    
    /// Check if a footnote is defined
    pub fn is_defined(&self, label: &str) -> bool {
        self.defs.contains_key(label)
    }
    
    /// Get all referenced footnotes in order
    pub fn get_referenced_footnotes(&self) -> Vec<&FootnoteDef> {
        self.ordered_labels
            .iter()
            .filter_map(|label| self.defs.get(label))
            .filter(|def| def.ref_count > 0)
            .collect()
    }
}

/// Render footnote reference to HTML
pub fn render_footnote_ref_html(_label: &str, ordinal: usize) -> String {
    format!(
        "<sup class=\"footnote-ref\"><a href=\"#fn{}\" id=\"fnref{}\">[{}]</a></sup>",
        ordinal, ordinal, ordinal
    )
}

/// Render footnote definition to HTML
pub fn render_footnote_def_html(ordinal: usize, content: &str) -> String {
    format!(
        "<li id=\"fn{}\">\n<p>{} <a href=\"#fnref{}\" class=\"footnote-backref\">↩</a></p>\n</li>",
        ordinal, content, ordinal
    )
}

/// Render footnote list to HTML
pub fn render_footnote_list_html(items: &[String]) -> String {
    if items.is_empty() {
        return String::new();
    }
    
    let items_html = items.join("\n");
    format!(
        r#"<section class="footnotes">
<ol>
{}
</ol>
</section>"#,
        items_html
    )
}

/// Render footnote reference to CommonMark
pub fn render_footnote_ref_commonmark(label: &str) -> String {
    format!("[^{}]", label)
}

/// Render footnote definition to CommonMark
pub fn render_footnote_def_commonmark(label: &str, content: &str) -> String {
    format!("[^{}]: {}", label, content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains_footnote_ref() {
        assert!(contains_footnote_ref("Here is a footnote[^1]."));
        assert!(contains_footnote_ref("Multiple[^a][^b] refs."));
        assert!(!contains_footnote_ref("No footnote here."));
        assert!(!contains_footnote_ref("Just [a link](url)."));
    }

    #[test]
    fn test_parse_footnote_ref() {
        let result = parse_footnote_ref("[^1]");
        assert_eq!(result, Some(("1".to_string(), 4)));

        let result = parse_footnote_ref("[^label]");
        assert_eq!(result, Some(("label".to_string(), 8)));

        let result = parse_footnote_ref("no ref");
        assert_eq!(result, None);
    }

    #[test]
    fn test_find_footnote_refs() {
        let refs = find_footnote_refs("Here[^1] are[^2] refs.");
        assert_eq!(refs.len(), 2);
        assert_eq!(refs[0], (4, 8, "1".to_string()));
        // The second ref starts at position 12 (after "Here[^1] are")
        assert_eq!(refs[1], (12, 16, "2".to_string()));
    }

    #[test]
    fn test_is_footnote_def() {
        assert!(is_footnote_def("[^1]: Footnote content."));
        assert!(is_footnote_def("  [^label]: Indented definition."));
        assert!(!is_footnote_def("Not a [^1]: definition."));
        assert!(!is_footnote_def("[^1] just a reference"));
        assert!(!is_footnote_def("[^]: Empty label"));
    }

    #[test]
    fn test_parse_footnote_def() {
        let result = parse_footnote_def("[^1]: This is the content.");
        assert_eq!(result, Some(("1".to_string(), "This is the content.".to_string())));

        let result = parse_footnote_def("[^label]: Multi word content here.");
        assert_eq!(result, Some(("label".to_string(), "Multi word content here.".to_string())));
    }

    #[test]
    fn test_footnote_registry() {
        let mut registry = FootnoteRegistry::new();
        
        // Register definitions
        let ord1 = registry.register_def("first");
        let ord2 = registry.register_def("second");
        
        assert_eq!(ord1, 1);
        assert_eq!(ord2, 2);
        
        // Add references
        registry.add_ref("first");
        registry.add_ref("first");
        registry.add_ref("second");
        
        assert_eq!(registry.defs["first"].ref_count, 2);
        assert_eq!(registry.defs["second"].ref_count, 1);
        
        // Get referenced footnotes
        let referenced = registry.get_referenced_footnotes();
        assert_eq!(referenced.len(), 2);
    }

    #[test]
    fn test_render_footnote_ref_html() {
        let html = render_footnote_ref_html("1", 1);
        assert!(html.contains("fn1"));
        assert!(html.contains("fnref1"));
        assert!(html.contains("[1]"));
    }

    #[test]
    fn test_render_footnote_def_html() {
        let html = render_footnote_def_html(1, "Footnote content.");
        assert!(html.contains("id=\"fn1\""));
        assert!(html.contains("Footnote content."));
        assert!(html.contains("↩"));
    }

    #[test]
    fn test_render_footnote_commonmark() {
        assert_eq!(render_footnote_ref_commonmark("1"), "[^1]");
        assert_eq!(render_footnote_def_commonmark("1", "content"), "[^1]: content");
    }
}
