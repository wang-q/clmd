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

/// Normalize whitespace in text.
///
/// Converts multiple consecutive whitespace characters into a single space,
/// and trims leading/trailing whitespace.
///
/// # Arguments
///
/// * `text` - The text to normalize
///
/// # Returns
///
/// The normalized text
///
/// # Example
///
/// ```ignore
/// use clmd::io::writer::shared::normalize_whitespace;
///
/// let normalized = normalize_whitespace("  hello   world  ");
/// assert_eq!(normalized, "hello world");
/// ```
pub fn normalize_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Wrap text at a specified width.
///
/// # Arguments
///
/// * `text` - The text to wrap
/// * `width` - The maximum line width
///
/// # Returns
///
/// The wrapped text with newlines inserted
///
/// # Example
///
/// ```ignore
/// use clmd::io::writer::shared::wrap_text;
///
/// let wrapped = wrap_text("This is a long sentence that needs wrapping.", 20);
/// assert!(wrapped.contains('\n'));
/// ```
pub fn wrap_text(text: &str, width: usize) -> String {
    let mut result = String::new();
    let mut line_len = 0;

    for word in text.split_whitespace() {
        let word_len = word.chars().count();

        if line_len + word_len + 1 > width && line_len > 0 {
            result.push('\n');
            line_len = 0;
        }

        if line_len > 0 {
            result.push(' ');
            line_len += 1;
        }

        result.push_str(word);
        line_len += word_len;
    }

    result
}

/// Find the first heading in the document.
///
/// # Arguments
///
/// * `arena` - The arena containing the AST nodes
/// * `root` - The root node ID
///
/// # Returns
///
/// The ID of the first heading node, or None if no heading found
///
/// # Example
///
/// ```ignore
/// use clmd::io::writer::shared::find_first_heading;
///
/// let heading_id = find_first_heading(&arena, root);
/// ```
pub fn find_first_heading(arena: &NodeArena, root: NodeId) -> Option<NodeId> {
    let root_node = arena.get(root);
    let mut child_opt = root_node.first_child;

    while let Some(child_id) = child_opt {
        let child = arena.get(child_id);
        if matches!(child.value, NodeValue::Heading(_)) {
            return Some(child_id);
        }
        child_opt = child.next;
    }

    None
}

/// Check if a node has children.
///
/// # Arguments
///
/// * `arena` - The arena containing the AST nodes
/// * `node_id` - The node to check
///
/// # Returns
///
/// `true` if the node has at least one child
///
/// # Example
///
/// ```ignore
/// use clmd::io::writer::shared::has_children;
///
/// if has_children(&arena, node_id) {
///     // Process children
/// }
/// ```
pub fn has_children(arena: &NodeArena, node_id: NodeId) -> bool {
    arena.get(node_id).first_child.is_some()
}

/// Get the text content of a node and its children.
///
/// This is similar to `collect_text` but returns an empty string
/// if the node has no text content.
///
/// # Arguments
///
/// * `arena` - The arena containing the AST nodes
/// * `node_id` - The node to get text from
///
/// # Returns
///
/// The text content, or empty string if none
///
/// # Example
///
/// ```ignore
/// use clmd::io::writer::shared::get_node_text;
///
/// let text = get_node_text(&arena, node_id);
/// ```
pub fn get_node_text(arena: &NodeArena, node_id: NodeId) -> String {
    collect_text(arena, node_id)
}

/// Escape LaTeX special characters.
///
/// Escapes characters that have special meaning in LaTeX.
///
/// # Arguments
///
/// * `text` - The text to escape
///
/// # Returns
///
/// The escaped text safe for LaTeX
///
/// # Example
///
/// ```ignore
/// use clmd::io::writer::shared::escape_latex;
///
/// let escaped = escape_latex("100% & more");
/// assert_eq!(escaped, "100\\% \\& more");
/// ```
pub fn escape_latex(text: &str) -> String {
    let mut result = String::with_capacity(text.len() * 2);
    for c in text.chars() {
        match c {
            '\\' => result.push_str("\\textbackslash{}"),
            '{' => result.push_str("\\{"),
            '}' => result.push_str("\\}"),
            '$' => result.push_str("\\$"),
            '&' => result.push_str("\\&"),
            '#' => result.push_str("\\#"),
            '^' => result.push_str("\\^{}"),
            '_' => result.push_str("\\_"),
            '%' => result.push_str("\\%"),
            '~' => result.push_str("\\textasciitilde{}"),
            '<' => result.push_str("\\textless{}"),
            '>' => result.push_str("\\textgreater{}"),
            '|' => result.push_str("\\textbar{}"),
            '"' => result.push_str("\\textquotedbl{}"),
            '`' => result.push_str("\\textasciigrave{}"),
            '\'' => result.push_str("\\textquotesingle{}"),
            _ => result.push(c),
        }
    }
    result
}

/// Escape Typst special characters.
///
/// Escapes characters that have special meaning in Typst.
///
/// # Arguments
///
/// * `text` - The text to escape
///
/// # Returns
///
/// The escaped text safe for Typst
///
/// # Example
///
/// ```ignore
/// use clmd::io::writer::shared::escape_typst;
///
/// let escaped = escape_typst("#heading *bold*");
/// assert!(escaped.contains("\\#"));
/// ```
pub fn escape_typst(text: &str) -> String {
    let mut result = String::with_capacity(text.len() * 2);
    for c in text.chars() {
        match c {
            '\\' => result.push('\\'),
            '*' => result.push_str("\\*"),
            '_' => result.push_str("\\_"),
            '`' => result.push_str("\\`"),
            '$' => result.push_str("\\$"),
            '#' => result.push_str("\\#"),
            '@' => result.push_str("\\@"),
            '<' => result.push_str("\\<"),
            '>' => result.push_str("\\>"),
            '"' => result.push_str("\\\""),
            _ => result.push(c),
        }
    }
    result
}

/// Escape man page (roff) special characters.
///
/// Escapes characters that have special meaning in roff/troff.
///
/// # Arguments
///
/// * `text` - The text to escape
///
/// # Returns
///
/// The escaped text safe for man pages
///
/// # Example
///
/// ```ignore
/// use clmd::io::writer::shared::escape_man;
///
/// let escaped = escape_man("use \\fBbold\\fP");
/// assert!(escaped.contains("\\\\"));
/// ```
pub fn escape_man(text: &str) -> String {
    let mut result = String::with_capacity(text.len() * 2);
    for c in text.chars() {
        match c {
            '\\' => result.push_str("\\e"),
            '-' => result.push_str("\\-"),
            '.' => result.push_str("\\&."),
            '\'' => result.push_str("\\&'"),
            _ => result.push(c),
        }
    }
    result
}

/// Count words in text.
///
/// # Arguments
///
/// * `text` - The text to count words in
///
/// # Returns
///
/// The number of words
///
/// # Example
///
/// ```ignore
/// use clmd::io::writer::shared::count_words;
///
/// assert_eq!(count_words("hello world"), 2);
/// assert_eq!(count_words(""), 0);
/// ```
pub fn count_words(text: &str) -> usize {
    text.split_whitespace().count()
}

/// Truncate text to a maximum length with ellipsis.
///
/// # Arguments
///
/// * `text` - The text to truncate
/// * `max_len` - The maximum length
///
/// # Returns
///
/// The truncated text with "..." if it was truncated
///
/// # Example
///
/// ```ignore
/// use clmd::io::writer::shared::truncate_text;
///
/// assert_eq!(truncate_text("hello world", 8), "hello...");
/// assert_eq!(truncate_text("hi", 8), "hi");
/// ```
pub fn truncate_text(text: &str, max_len: usize) -> String {
    if text.chars().count() <= max_len {
        text.to_string()
    } else {
        let mut result: String = text.chars().take(max_len.saturating_sub(3)).collect();
        result.push_str("...");
        result
    }
}

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

    #[test]
    fn test_normalize_whitespace() {
        assert_eq!(normalize_whitespace("  hello   world  "), "hello world");
        assert_eq!(normalize_whitespace("hello\tworld"), "hello world");
        assert_eq!(normalize_whitespace("hello\nworld"), "hello world");
        assert_eq!(normalize_whitespace(""), "");
        assert_eq!(normalize_whitespace("   "), "");
    }

    #[test]
    fn test_wrap_text() {
        let wrapped = wrap_text("This is a long sentence that needs wrapping.", 20);
        assert!(wrapped.contains('\n'));

        let short = wrap_text("Short text", 50);
        assert!(!short.contains('\n'));
        assert_eq!(short, "Short text");
    }

    #[test]
    fn test_find_first_heading() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        // Add paragraph first
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        TreeOps::append_child(&mut arena, root, para);

        // Then add heading
        let heading = arena.alloc(Node::with_value(NodeValue::Heading(NodeHeading {
            level: 1,
            setext: false,
            closed: false,
        })));
        TreeOps::append_child(&mut arena, root, heading);

        let found = find_first_heading(&arena, root);
        assert_eq!(found, Some(heading));
    }

    #[test]
    fn test_find_first_heading_none() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        TreeOps::append_child(&mut arena, root, para);

        let found = find_first_heading(&arena, root);
        assert_eq!(found, None);
    }

    #[test]
    fn test_has_children() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));

        assert!(!has_children(&arena, root));

        let child = arena.alloc(Node::with_value(NodeValue::Paragraph));
        TreeOps::append_child(&mut arena, root, child);

        assert!(has_children(&arena, root));
    }

    #[test]
    fn test_escape_latex() {
        assert_eq!(escape_latex("100%"), "100\\%");
        assert_eq!(escape_latex("a & b"), "a \\& b");
        assert_eq!(escape_latex("$100"), "\\$100");
        assert_eq!(escape_latex("test_"), "test\\_");
        assert_eq!(escape_latex("x^2"), "x\\^{}2");
        assert_eq!(escape_latex("#tag"), "\\#tag");
        assert_eq!(escape_latex("{brace}"), "\\{brace\\}");
    }

    #[test]
    fn test_escape_typst() {
        assert_eq!(escape_typst("#heading"), "\\#heading");
        assert_eq!(escape_typst("*bold*"), "\\*bold\\*");
        assert_eq!(escape_typst("_italic_"), "\\_italic\\_");
        assert_eq!(escape_typst("$math$"), "\\$math\\$");
    }

    #[test]
    fn test_escape_man() {
        assert_eq!(escape_man("\\fB"), "\\efB");
        assert_eq!(escape_man("test-file"), "test\\-file");
        assert!(escape_man(".SH").starts_with("\\&"));
    }

    #[test]
    fn test_count_words() {
        assert_eq!(count_words("hello world"), 2);
        assert_eq!(count_words(""), 0);
        assert_eq!(count_words("one"), 1);
        assert_eq!(count_words("a b c d e"), 5);
    }

    #[test]
    fn test_truncate_text() {
        assert_eq!(truncate_text("hello world", 8), "hello...");
        assert_eq!(truncate_text("hi", 8), "hi");
        assert_eq!(truncate_text("exactly ten", 11), "exactly ten");
        assert_eq!(truncate_text("more than ten", 10), "more th...");
    }
}
