//! Task list extension for GitHub Flavored Markdown
//!
//! This module implements task list (checkbox) parsing according to GFM spec.
//! Task list items are list items with a checkbox at the beginning.
//!
//! Examples:
//! ```markdown
//! - [ ] Unchecked task
//! - [x] Checked task
//! - [X] Also checked task
//! ```

use crate::arena::{Node, NodeArena, NodeId, TreeOps};
use crate::node_value::{NodeTaskItem, NodeValue, SourcePos};

/// Check if a string is a task list item marker
/// Returns Some(checked) if it's a task item, None otherwise
pub fn parse_task_marker(text: &str) -> Option<bool> {
    let trimmed = text.trim_start();

    // Check for pattern: [ ] or [x] or [X]
    if let Some(rest) = trimmed.strip_prefix('[') {
        if let Some(close_bracket) = rest.find(']') {
            let content = &rest[..close_bracket];
            let content_trimmed = content.trim();

            if content_trimmed.is_empty() {
                return Some(false); // [ ] - unchecked
            } else if content_trimmed.eq_ignore_ascii_case("x") {
                return Some(true); // [x] or [X] - checked
            }
        }
    }

    None
}

/// Extract task marker from the beginning of text and return (checked, remaining_text)
pub fn extract_task_marker(text: &str) -> Option<(bool, &str)> {
    let trimmed = text.trim_start();

    if let Some(checked) = parse_task_marker(text) {
        // Find the position after the closing bracket
        if let Some(start) = trimmed.find('[') {
            if let Some(end) = trimmed[start..].find(']') {
                let after_marker = &trimmed[start + end + 1..];
                // Skip leading whitespace after the marker
                let remaining = after_marker.trim_start();
                return Some((checked, remaining));
            }
        }
    }

    None
}

/// Create a task item node in the arena
/// Returns the NodeId of the created node
pub fn create_task_item(
    arena: &mut NodeArena,
    checked: bool,
    content: &str,
    line: u32,
    col: u32,
) -> NodeId {
    let symbol = if checked { Some('x') } else { None };
    let node = arena.alloc(Node::with_value(NodeValue::TaskItem(NodeTaskItem {
        symbol,
    })));

    {
        let node_ref = arena.get_mut(node);
        node_ref.source_pos = SourcePos::new(
            line as usize,
            col as usize,
            line as usize,
            (col + content.len() as u32) as usize,
        );
    }

    // Create text node for the content
    if !content.is_empty() {
        let text_node =
            arena.alloc(Node::with_value(NodeValue::Text(content.to_string())));
        TreeOps::append_child(arena, node, text_node);
    }

    node
}

/// Check if a line is a task list item
pub fn is_task_list_item(line: &str) -> bool {
    let trimmed = line.trim_start();

    // Check for bullet list marker followed by task marker
    if trimmed.starts_with("- [")
        || trimmed.starts_with("* [")
        || trimmed.starts_with("+ [")
    {
        return parse_task_marker(&trimmed[1..]).is_some();
    }

    // Check for ordered list marker followed by task marker
    if let Some(dot_pos) = trimmed.find('.') {
        let before_dot = &trimmed[..dot_pos];
        if before_dot.parse::<u32>().is_ok() {
            let after_dot = &trimmed[dot_pos + 1..];
            if parse_task_marker(after_dot).is_some() {
                return true;
            }
        }
    }

    false
}

/// Render task item to HTML
pub fn render_task_item_html(checked: bool, content: &str) -> String {
    let checkbox = if checked {
        r#"<input type="checkbox" checked="" disabled="" />"#
    } else {
        r#"<input type="checkbox" disabled="" />"#
    };

    format!("{} {}", checkbox, content)
}

/// Render task item to CommonMark
pub fn render_task_item_commonmark(checked: bool, content: &str) -> String {
    let marker = if checked { "[x]" } else { "[ ]" };
    format!("{} {}", marker, content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_task_marker() {
        assert_eq!(parse_task_marker("[ ]"), Some(false));
        assert_eq!(parse_task_marker("[x]"), Some(true));
        assert_eq!(parse_task_marker("[X]"), Some(true));
        assert_eq!(parse_task_marker("[ ] task"), Some(false));
        assert_eq!(parse_task_marker("normal text"), None);
        assert_eq!(parse_task_marker("[y]"), None);
    }

    #[test]
    fn test_extract_task_marker() {
        let result = extract_task_marker("[ ] task content");
        assert_eq!(result, Some((false, "task content")));

        let result = extract_task_marker("[x] task content");
        assert_eq!(result, Some((true, "task content")));

        let result = extract_task_marker("no marker");
        assert_eq!(result, None);
    }

    #[test]
    fn test_is_task_list_item() {
        assert!(is_task_list_item("- [ ] unchecked"));
        assert!(is_task_list_item("- [x] checked"));
        assert!(is_task_list_item("* [ ] asterisk"));
        assert!(is_task_list_item("+ [ ] plus"));
        assert!(is_task_list_item("1. [ ] ordered"));
        assert!(!is_task_list_item("- normal list item"));
        assert!(!is_task_list_item("normal text"));
    }

    #[test]
    fn test_create_task_item() {
        let mut arena = NodeArena::new();
        let node_id = create_task_item(&mut arena, true, "task content", 1, 1);
        let node = arena.get(node_id);
        assert!(matches!(node.value, NodeValue::TaskItem(..)));
    }

    #[test]
    fn test_render_task_item_html() {
        assert_eq!(
            render_task_item_html(false, "unchecked task"),
            r#"<input type="checkbox" disabled="" /> unchecked task"#
        );
        assert_eq!(
            render_task_item_html(true, "checked task"),
            r#"<input type="checkbox" checked="" disabled="" /> checked task"#
        );
    }

    #[test]
    fn test_render_task_item_commonmark() {
        assert_eq!(
            render_task_item_commonmark(false, "unchecked task"),
            "[ ] unchecked task"
        );
        assert_eq!(
            render_task_item_commonmark(true, "checked task"),
            "[x] checked task"
        );
    }
}
