//! Footnote extension for CommonMark
//!
//! This module provides support for footnotes, similar to Pandoc's footnote syntax.
//!
//! # Syntax
//!
//! ```markdown
//! Here is some text with a footnote[^1].
//!
//! [^1]: This is the footnote content.
//! ```
//!
//! # Example
//!
//! ```markdown
//! You can write footnotes[^note1] in markdown.
//!
//! [^note1]: This is the footnote text.
//! ```

use crate::arena::{Node, NodeArena, NodeId, TreeOps};
use crate::nodes::{
    NodeFootnoteDefinition, NodeFootnoteReference, NodeValue, SourcePos,
};

/// Check if footnotes extension is enabled
pub fn is_enabled(options: u32) -> bool {
    // Footnotes are enabled when the 0x02 bit is set
    (options & 0x02) != 0
}

/// Create a footnote reference node
pub fn create_footnote_ref_node(
    arena: &mut NodeArena,
    label: &str,
    line: u32,
    col: u32,
) -> NodeId {
    let node = arena.alloc(Node::with_value(NodeValue::footnote_reference(
        NodeFootnoteReference {
            name: label.to_string(),
            ref_num: 0, // Will be set during rendering
            ix: 0,      // Will be set during rendering
        },
    )));

    {
        let node_ref = arena.get_mut(node);
        node_ref.source_pos = SourcePos::new(
            line as usize,
            col as usize,
            line as usize,
            (col + label.len() as u32 + 4) as usize, // +4 for [^ and ]
        );
    }

    node
}

/// Create a footnote definition node
pub fn create_footnote_def_node(
    arena: &mut NodeArena,
    label: &str,
    content: &str,
    line: u32,
    col: u32,
) -> NodeId {
    let node = arena.alloc(Node::with_value(NodeValue::footnote_definition(
        NodeFootnoteDefinition {
            name: label.to_string(),
            total_references: 0, // Will be set during processing
        },
    )));

    {
        let node_ref = arena.get_mut(node);
        node_ref.source_pos = SourcePos::new(
            line as usize,
            col as usize,
            line as usize,
            (col + label.len() as u32 + content.len() as u32 + 5) as usize, // +5 for [^, ], :, and space
        );
    }

    // Create a paragraph for the content
    let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
    let text = arena.alloc(Node::with_value(NodeValue::Text(
        content.to_string().into(),
    )));
    TreeOps::append_child(arena, para, text);
    TreeOps::append_child(arena, node, para);

    node
}

/// Render footnote reference as HTML
pub fn render_footnote_ref_html(label: &str, ref_num: usize) -> String {
    format!(
        "<sup class=\"footnote-ref\"><a href=\"#fn{}\" id=\"fnref{}\">[{}]</a></sup>",
        label, ref_num, label
    )
}

/// Render footnote definition as HTML
pub fn render_footnote_def_html(label: &str, content: &str, ref_count: usize) -> String {
    if ref_count == 0 {
        return String::new();
    }

    format!(
        "<li id=\"fn{}\">\n<p>{} <a href=\"#fnref{}\" class=\"footnote-backref\">↩</a></p>\n</li>",
        label, content, label
    )
}

/// Collect all footnote references in the document
pub fn collect_footnote_refs(arena: &NodeArena, root: NodeId) -> Vec<(String, NodeId)> {
    let mut refs = Vec::new();
    collect_footnote_refs_recursive(arena, root, &mut refs);
    refs
}

fn collect_footnote_refs_recursive(
    arena: &NodeArena,
    node_id: NodeId,
    refs: &mut Vec<(String, NodeId)>,
) {
    let node = arena.get(node_id);

    if let NodeValue::FootnoteReference(ref footnote_ref) = node.value {
        refs.push((footnote_ref.name.clone(), node_id));
    }

    // Recursively check children (first_child loop already handles siblings via next)
    if let Some(child_id) = node.first_child {
        let mut current = Some(child_id);
        while let Some(id) = current {
            collect_footnote_refs_recursive(arena, id, refs);
            current = arena.get(id).next;
        }
    }
}

/// Collect all footnote definitions in the document
pub fn collect_footnote_defs(arena: &NodeArena, root: NodeId) -> Vec<(String, NodeId)> {
    let mut defs = Vec::new();
    collect_footnote_defs_recursive(arena, root, &mut defs);
    defs
}

fn collect_footnote_defs_recursive(
    arena: &NodeArena,
    node_id: NodeId,
    defs: &mut Vec<(String, NodeId)>,
) {
    let node = arena.get(node_id);

    if let NodeValue::FootnoteDefinition(ref footnote_def) = node.value {
        defs.push((footnote_def.name.clone(), node_id));
    }

    // Recursively check children (first_child loop already handles siblings via next)
    if let Some(child_id) = node.first_child {
        let mut current = Some(child_id);
        while let Some(id) = current {
            collect_footnote_defs_recursive(arena, id, defs);
            current = arena.get(id).next;
        }
    }
}

/// Get footnote content as string
pub fn get_footnote_content(arena: &NodeArena, def_node: NodeId) -> String {
    let mut content = String::new();

    // Get first child (should be paragraph)
    if let Some(para_id) = arena.get(def_node).first_child {
        // Get first child of paragraph (should be text)
        if let Some(text_id) = arena.get(para_id).first_child {
            if let NodeValue::Text(ref text) = arena.get(text_id).value {
                content = text.to_string();
            }
        }
    }

    content
}

/// Count references to a footnote
pub fn count_footnote_refs(refs: &[(String, NodeId)], label: &str) -> usize {
    refs.iter().filter(|(l, _)| l == label).count()
}

/// Get all referenced footnote labels
pub fn get_referenced_labels(refs: &[(String, NodeId)]) -> Vec<String> {
    let mut labels: Vec<String> = refs.iter().map(|(l, _)| l.clone()).collect();
    labels.sort();
    labels.dedup();
    labels
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_enabled() {
        assert!(is_enabled(0x02));
        assert!(!is_enabled(0));
        assert!(is_enabled(0x02 | 0x01));
    }

    #[test]
    fn test_create_footnote_ref_node() {
        let mut arena = NodeArena::new();
        let node_id = create_footnote_ref_node(&mut arena, "1", 1, 1);
        let node = arena.get(node_id);

        // Check it's a FootnoteReference node
        assert!(matches!(node.value, NodeValue::FootnoteReference(..)));

        // Check the label
        if let NodeValue::FootnoteReference(ref footnote) = node.value {
            assert_eq!(footnote.name, "1");
        } else {
            panic!("Expected FootnoteReference value");
        }
    }

    #[test]
    fn test_create_footnote_def_node() {
        let mut arena = NodeArena::new();
        let node_id = create_footnote_def_node(&mut arena, "1", "content", 1, 1);
        let node = arena.get(node_id);

        // Check it's a FootnoteDefinition node
        assert!(matches!(node.value, NodeValue::FootnoteDefinition(..)));

        // Check the label
        if let NodeValue::FootnoteDefinition(ref footnote) = node.value {
            assert_eq!(footnote.name, "1");
        } else {
            panic!("Expected FootnoteDefinition value");
        }
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
        let html = render_footnote_def_html("1", "content", 1);
        assert!(html.contains("fn1"));
        assert!(html.contains("content"));
        assert!(html.contains("fnref1"));
    }

    #[test]
    fn test_render_footnote_def_html_zero_refs() {
        let html = render_footnote_def_html("1", "content", 0);
        assert!(html.is_empty());
    }

    #[test]
    fn test_collect_footnote_refs() {
        let mut arena = NodeArena::new();
        let doc = arena.alloc(Node::with_value(NodeValue::Document));

        let ref1 = create_footnote_ref_node(&mut arena, "1", 1, 1);
        let ref2 = create_footnote_ref_node(&mut arena, "2", 2, 1);

        TreeOps::append_child(&mut arena, doc, ref1);
        TreeOps::append_child(&mut arena, doc, ref2);

        let refs = collect_footnote_refs(&arena, doc);
        assert_eq!(refs.len(), 2);
        assert_eq!(refs[0].0, "1");
        assert_eq!(refs[1].0, "2");
    }

    #[test]
    fn test_collect_footnote_defs() {
        let mut arena = NodeArena::new();
        let doc = arena.alloc(Node::with_value(NodeValue::Document));

        let def1 = create_footnote_def_node(&mut arena, "1", "content1", 1, 1);
        let def2 = create_footnote_def_node(&mut arena, "2", "content2", 2, 1);

        TreeOps::append_child(&mut arena, doc, def1);
        TreeOps::append_child(&mut arena, doc, def2);

        let defs = collect_footnote_defs(&arena, doc);
        assert_eq!(defs.len(), 2);
        assert_eq!(defs[0].0, "1");
        assert_eq!(defs[1].0, "2");
    }

    #[test]
    fn test_get_footnote_content() {
        let mut arena = NodeArena::new();
        let node_id = create_footnote_def_node(&mut arena, "1", "test content", 1, 1);

        let content = get_footnote_content(&arena, node_id);
        assert_eq!(content, "test content");
    }

    #[test]
    fn test_count_footnote_refs() {
        let refs = vec![
            ("1".to_string(), 0),
            ("1".to_string(), 1),
            ("2".to_string(), 2),
        ];

        assert_eq!(count_footnote_refs(&refs, "1"), 2);
        assert_eq!(count_footnote_refs(&refs, "2"), 1);
        assert_eq!(count_footnote_refs(&refs, "3"), 0);
    }

    #[test]
    fn test_get_referenced_labels() {
        let refs = vec![
            ("b".to_string(), 0),
            ("a".to_string(), 1),
            ("b".to_string(), 2),
            ("c".to_string(), 3),
        ];

        let labels = get_referenced_labels(&refs);
        assert_eq!(labels, vec!["a", "b", "c"]);
    }
}
