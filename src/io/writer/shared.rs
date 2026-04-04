//! Shared utilities for document writers.
//!
//! This module provides common functionality used by multiple writers,
//! including HTML escaping, AST traversal helpers, and shared rendering logic.

use crate::core::arena::{NodeArena, NodeId};
use crate::core::nodes::NodeValue;

/// Escape HTML special characters.
///
/// Converts characters like `<`, `>`, `&`, `"` and `'` to their HTML entities.
///
/// # Example
///
/// ```ignore
/// use clmd::io::writer::shared::escape_html;
///
/// let escaped = escape_html("<script>alert('xss')</script>");
/// assert_eq!(escaped, "&lt;script&gt;alert(&#x27;xss&#x27;)&lt;/script&gt;");
/// ```
pub fn escape_html(text: &str) -> String {
    let mut result = String::with_capacity(text.len() * 2);
    escape_html_to(text, &mut result);
    result
}

/// Escape HTML special characters to an existing output buffer.
///
/// This is more efficient than `escape_html` when appending to an existing string.
///
/// # Example
///
/// ```ignore
/// use clmd::io::writer::shared::escape_html_to;
///
/// let mut output = String::new();
/// escape_html_to("<div>", &mut output);
/// assert_eq!(output, "&lt;div&gt;");
/// ```
pub fn escape_html_to(text: &str, output: &mut String) {
    for c in text.chars() {
        match c {
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '&' => output.push_str("&amp;"),
            '"' => output.push_str("&quot;"),
            '\'' => output.push_str("&#x27;"),
            _ => output.push(c),
        }
    }
}

/// Extract title from the first level 1 heading in the document.
///
/// This is useful for slide formats and document titles.
///
/// # Arguments
///
/// * `arena` - The arena containing the AST nodes
/// * `root` - The root node ID
///
/// # Returns
///
/// The title string if found, None otherwise.
pub fn extract_title(arena: &NodeArena, root: NodeId) -> Option<String> {
    let root_node = arena.get(root);
    let mut child_opt = root_node.first_child;

    while let Some(child_id) = child_opt {
        let child = arena.get(child_id);
        if let NodeValue::Heading(heading) = &child.value {
            if heading.level == 1 {
                let mut title = String::new();
                let mut text_opt = child.first_child;
                while let Some(text_id) = text_opt {
                    let text_node = arena.get(text_id);
                    if let NodeValue::Text(t) = &text_node.value {
                        title.push_str(t);
                    }
                    text_opt = text_node.next;
                }
                return Some(title);
            }
        }
        child_opt = child.next;
    }

    None
}

/// Collect text content from a node and its children.
///
/// # Arguments
///
/// * `arena` - The arena containing the AST nodes
/// * `node_id` - The node to collect text from
///
/// # Returns
///
/// The concatenated text content.
pub fn collect_text(arena: &NodeArena, node_id: NodeId) -> String {
    let mut result = String::new();
    collect_text_recursive(arena, node_id, &mut result);
    result
}

fn collect_text_recursive(arena: &NodeArena, node_id: NodeId, output: &mut String) {
    let node = arena.get(node_id);

    if let NodeValue::Text(text) = &node.value {
        output.push_str(text);
    }

    let mut child_opt = node.first_child;
    while let Some(child_id) = child_opt {
        collect_text_recursive(arena, child_id, output);
        let child = arena.get(child_id);
        child_opt = child.next;
    }
}

/// Standard HTML document preamble.
pub const HTML_PREAMBLE: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
"#;

/// Standard HTML head end and body start.
pub const HTML_HEAD_END: &str = r#"</head>
<body>
"#;

/// Standard HTML footer.
pub const HTML_FOOTER: &str = r#"</body>
</html>
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::arena::{Node, NodeArena, TreeOps};
    use crate::core::nodes::{NodeHeading, NodeValue};

    #[test]
    fn test_escape_html() {
        assert_eq!(escape_html("hello"), "hello");
        assert_eq!(escape_html("<script>"), "&lt;script&gt;");
        assert_eq!(escape_html("a & b"), "a &amp; b");
        assert_eq!(escape_html("\"quote\""), "&quot;quote&quot;");
        assert_eq!(escape_html("it's"), "it&#x27;s");
    }

    #[test]
    fn test_escape_html_to() {
        let mut output = String::new();
        escape_html_to("<>&\"'", &mut output);
        assert_eq!(output, "&lt;&gt;&amp;&quot;&#x27;");
    }

    #[test]
    fn test_extract_title() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        let heading = arena.alloc(Node::with_value(NodeValue::Heading(NodeHeading {
            level: 1,
            setext: false,
            closed: false,
        })));
        let text = arena.alloc(Node::with_value(NodeValue::Text("Title".into())));
        TreeOps::append_child(&mut arena, heading, text);
        TreeOps::append_child(&mut arena, root, heading);

        let title = extract_title(&arena, root);
        assert_eq!(title, Some("Title".to_string()));
    }

    #[test]
    fn test_extract_title_no_heading() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::Text("Just text".into())));
        TreeOps::append_child(&mut arena, para, text);
        TreeOps::append_child(&mut arena, root, para);

        let title = extract_title(&arena, root);
        assert_eq!(title, None);
    }

    #[test]
    fn test_collect_text() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text1 = arena.alloc(Node::with_value(NodeValue::Text("Hello ".into())));
        let text2 = arena.alloc(Node::with_value(NodeValue::Text("world".into())));
        TreeOps::append_child(&mut arena, root, text1);
        TreeOps::append_child(&mut arena, root, text2);

        let collected = collect_text(&arena, root);
        assert_eq!(collected, "Hello world");
    }

    #[test]
    fn test_html_constants() {
        assert!(HTML_PREAMBLE.contains("<!DOCTYPE html>"));
        assert!(HTML_HEAD_END.contains("<body>"));
        assert!(HTML_FOOTER.contains("</html>"));
    }
}
