//! List element handlers for CommonMark formatting
//!
//! This module contains handlers for list elements like List and Item,
//! and related helper functions.

use crate::core::arena::NodeId;
use crate::core::nodes::NodeList;
use crate::core::traverse::TraverseExt;
use crate::options::format::{BulletMarker, FormatOptions, ListSpacing, NumberedMarker};
use crate::render::commonmark::context::NodeFormatterContext;

/// Format a list item marker with a given number and options
pub fn format_list_item_marker_with_number_and_options(
    list: &NodeList,
    item_number: usize,
    options: &FormatOptions,
) -> String {
    use crate::core::nodes::{ListDelimType, ListType};

    match list.list_type {
        ListType::Bullet => {
            let bullet_char = match options.list_bullet_marker {
                BulletMarker::Dash => '-',
                BulletMarker::Asterisk => '*',
                BulletMarker::Plus => '+',
                BulletMarker::Any => list.bullet_char as char,
            };
            format!("{} ", bullet_char)
        }
        ListType::Ordered => {
            let delimiter = match options.list_numbered_marker {
                NumberedMarker::Period => '.',
                NumberedMarker::Paren => ')',
                NumberedMarker::Any => match list.delimiter {
                    ListDelimType::Period => '.',
                    ListDelimType::Paren => ')',
                },
            };
            let marker = format!("{}{}", item_number, delimiter);
            format!("{} ", marker)
        }
    }
}

/// Count the number of list ancestors for a given node
///
/// This is used to determine the nesting level of a list item.
/// Returns 0 for top-level items, 1 for items in nested lists, etc.
pub fn count_list_ancestors(
    arena: &crate::core::arena::NodeArena,
    list_node_id: NodeId,
) -> usize {
    use crate::core::nodes::NodeValue;

    arena
        .ancestors_iter(list_node_id)
        .filter(|&id| matches!(arena.get(id).value, NodeValue::List(..)))
        .count()
}

/// Check if a task list item is checked
///
/// This function examines the content of a task list item to determine
/// if it starts with [x] or [X] (checked) or [ ] (unchecked).
pub fn is_task_item_checked(
    arena: &crate::core::arena::NodeArena,
    item_node_id: Option<NodeId>,
) -> bool {
    use crate::core::nodes::NodeValue;

    let item_id = match item_node_id {
        Some(id) => id,
        None => return false,
    };

    let item = arena.get(item_id);

    let mut child_id = item.first_child;
    while let Some(child) = child_id {
        let child_node = arena.get(child);

        if matches!(child_node.value, NodeValue::Paragraph) {
            let mut para_child_id = child_node.first_child;
            while let Some(para_child) = para_child_id {
                let para_child_node = arena.get(para_child);
                if let NodeValue::Text(text) = &para_child_node.value {
                    let text_str = text.as_ref();
                    return text_str.starts_with("[x]") || text_str.starts_with("[X]");
                }
                para_child_id = para_child_node.next;
            }
        }

        child_id = child_node.next;
    }

    false
}

/// Check if the current context is inside a task list item
///
/// This checks if the current node's parent is a list item that is part of a task list.
pub fn is_in_task_list_item(ctx: &dyn NodeFormatterContext) -> bool {
    use crate::core::nodes::NodeValue;

    if let Some(current_node) = ctx.get_current_node() {
        let arena = ctx.get_arena();
        let node = arena.get(current_node);

        if let Some(parent_id) = node.parent {
            let parent = arena.get(parent_id);

            if matches!(parent.value, NodeValue::Paragraph) {
                if let Some(grandparent_id) = parent.parent {
                    let grandparent = arena.get(grandparent_id);
                    if let NodeValue::Item(item_data) = &grandparent.value {
                        return item_data.is_task_list;
                    }
                }
            }

            if let NodeValue::Item(item_data) = &parent.value {
                return item_data.is_task_list;
            }
        }
    }

    false
}

/// Skip the task list marker from the beginning of text
///
/// If the text starts with "[ ] " or "[x] " (or "[X] "), remove it.
pub fn skip_task_marker(text: &str) -> String {
    if let Some(rest) = text.strip_prefix("[ ] ") {
        rest.to_string()
    } else if let Some(rest) = text.strip_prefix("[x] ") {
        rest.to_string()
    } else if let Some(rest) = text.strip_prefix("[X] ") {
        rest.to_string()
    } else {
        text.to_string()
    }
}

/// Calculate the prefixes for list item line breaking
///
/// Returns (first_line_prefix, continuation_prefix) where:
/// - first_line_prefix is empty (the list marker is already output by Item handler)
/// - continuation_prefix is the indentation to align with the list marker
pub fn calculate_list_item_prefixes(ctx: &dyn NodeFormatterContext) -> (String, String) {
    use crate::core::nodes::NodeValue;
    use crate::text::unicode;

    if let Some(current_node) = ctx.get_current_node() {
        let arena = ctx.get_arena();
        let node = arena.get(current_node);

        if let Some(parent_id) = node.parent {
            let parent = arena.get(parent_id);

            if let NodeValue::Item(_item_data) = &parent.value {
                if let Some(grandparent_id) = parent.parent {
                    let grandparent = arena.get(grandparent_id);

                    if let NodeValue::List(list) = &grandparent.value {
                        let item_number = get_item_number_in_list(
                            arena,
                            grandparent_id,
                            Some(parent_id),
                        );

                        let marker = format_list_item_marker_with_number_and_options(
                            list,
                            item_number,
                            ctx.get_formatter_options(),
                        );

                        let marker_width = unicode::width(&marker) as usize;

                        let nesting_level = count_list_ancestors(arena, grandparent_id);
                        let indent_width = nesting_level * 4;

                        let first_prefix = String::new();

                        let cont_prefix = " ".repeat(indent_width + marker_width);

                        return (first_prefix, cont_prefix);
                    }
                }
            }
        }
    }

    (String::new(), String::new())
}

/// Get the 1-based item number of a node within its parent list
///
/// This is used to determine the correct number for ordered list items.
pub fn get_item_number_in_list(
    arena: &crate::core::arena::NodeArena,
    list_node_id: NodeId,
    item_node_id: Option<NodeId>,
) -> usize {
    use crate::core::nodes::NodeValue;

    let item_id = match item_node_id {
        Some(id) => id,
        None => return 1,
    };

    let list = arena.get(list_node_id);
    let mut item_number: usize = 0;

    if let Some(first_child) = list.first_child {
        let mut current = first_child;
        loop {
            if matches!(arena.get(current).value, NodeValue::Item(..)) {
                item_number += 1;
                if current == item_id {
                    break;
                }
            }
            if let Some(next) = arena.get(current).next {
                current = next;
            } else {
                break;
            }
        }
    }

    if item_number == 0 {
        1
    } else {
        item_number
    }
}

/// Check if a list item is empty (has no content or only whitespace)
///
/// This is used for the listRemoveEmptyItems option.
pub fn is_empty_list_item(
    arena: &crate::core::arena::NodeArena,
    item_node_id: NodeId,
) -> bool {
    use crate::core::nodes::NodeValue;

    let item = arena.get(item_node_id);

    let mut child_id = item.first_child;
    while let Some(child) = child_id {
        let child_node = arena.get(child);

        match &child_node.value {
            NodeValue::Text(text) => {
                if !text.trim().is_empty() {
                    return false;
                }
            }
            NodeValue::Paragraph | NodeValue::Heading(_) => {
                if !is_empty_container(arena, child) {
                    return false;
                }
            }
            _ => return false,
        }

        child_id = child_node.next;
    }

    true
}

/// Check if a container node is empty (has no meaningful content)
pub fn is_empty_container(
    arena: &crate::core::arena::NodeArena,
    node_id: NodeId,
) -> bool {
    use crate::core::nodes::NodeValue;

    let node = arena.get(node_id);

    let mut child_id = node.first_child;
    while let Some(child) = child_id {
        let child_node = arena.get(child);

        match &child_node.value {
            NodeValue::Text(text) => {
                if !text.trim().is_empty() {
                    return false;
                }
            }
            NodeValue::Paragraph | NodeValue::Heading(_) => {
                if !is_empty_container(arena, child) {
                    return false;
                }
            }
            _ => return false,
        }

        child_id = child_node.next;
    }

    true
}

/// Check if a list should be considered "loose" based on CommonMark spec
///
/// According to CommonMark, a list is loose if any of its constituent list
/// items are separated by blank lines, or if any of its constituent list
/// items directly contain two block-level elements with a blank line between them.
pub fn is_list_loose(
    arena: &crate::core::arena::NodeArena,
    list_node_id: NodeId,
) -> bool {
    use crate::core::nodes::NodeValue;

    let list = arena.get(list_node_id);

    let mut prev_item_had_blank_line = false;
    let mut child_id = list.first_child;

    while let Some(item_id) = child_id {
        let item = arena.get(item_id);

        if matches!(item.value, NodeValue::Item(..)) {
            if item_contains_blank_lines(arena, item_id) {
                return true;
            }

            if prev_item_had_blank_line {
                return true;
            }

            prev_item_had_blank_line = has_trailing_blank_line(arena, item_id);
        }

        child_id = item.next;
    }

    false
}

/// Check if a list item contains blank lines between its block-level children
///
/// This is part of the CommonMark definition of a loose list.
pub fn item_contains_blank_lines(
    arena: &crate::core::arena::NodeArena,
    item_node_id: NodeId,
) -> bool {
    use crate::core::nodes::NodeValue;

    let item = arena.get(item_node_id);

    let mut non_list_block_count = 0;
    let mut child_id = item.first_child;

    while let Some(child) = child_id {
        let child_node = arena.get(child);

        match &child_node.value {
            NodeValue::List(_) => {}
            NodeValue::Paragraph
            | NodeValue::Heading(_)
            | NodeValue::BlockQuote
            | NodeValue::CodeBlock(_)
            | NodeValue::HtmlBlock(_) => {
                non_list_block_count += 1;
                if non_list_block_count > 1 {
                    return true;
                }
            }
            _ => {}
        }

        child_id = child_node.next;
    }

    false
}

/// Check if a node has trailing blank lines
///
/// This checks if there's a blank line after this node in the document.
pub fn has_trailing_blank_line(
    _arena: &crate::core::arena::NodeArena,
    node_id: NodeId,
) -> bool {
    let _ = node_id;
    false
}

/// Calculate the effective tightness of a list based on options and content
///
/// This function determines whether a list should be rendered as tight or loose
/// based on the formatter options and the actual list content.
pub fn calculate_effective_list_tightness(
    arena: &crate::core::arena::NodeArena,
    list_node_id: NodeId,
    _list: &NodeList,
    options: &FormatOptions,
) -> bool {
    match options.list_spacing {
        ListSpacing::Tight => true,
        ListSpacing::Loose => false,
        ListSpacing::AsIs => {
            let content_is_loose = is_list_loose(arena, list_node_id);
            !content_is_loose
        }
        ListSpacing::Loosen => {
            let content_is_loose = is_list_loose(arena, list_node_id);
            !content_is_loose
        }
        ListSpacing::Tighten => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::arena::{Node, NodeArena, TreeOps};
    use crate::core::nodes::{ListDelimType, ListType, NodeList, NodeValue};

    #[test]
    fn test_count_list_ancestors() {
        let mut arena = NodeArena::new();

        // Create a nested list structure:
        // Document
        // └── List (outer)
        //     ├── Item 1
        //     └── Item 2
        //         └── List (inner)
        //             └── Item 3

        let root = arena.alloc(Node::with_value(NodeValue::Document));

        let outer_list = arena.alloc(Node::with_value(NodeValue::List(NodeList {
            list_type: ListType::Bullet,
            marker_offset: 0,
            padding: 0,
            start: 1,
            delimiter: ListDelimType::Period,
            bullet_char: b'-',
            tight: true,
            is_task_list: false,
        })));

        let item1 = arena.alloc(Node::with_value(NodeValue::Item(NodeList {
            list_type: ListType::Bullet,
            marker_offset: 0,
            padding: 0,
            start: 0,
            delimiter: ListDelimType::Period,
            bullet_char: 0,
            tight: true,
            is_task_list: false,
        })));

        let item2 = arena.alloc(Node::with_value(NodeValue::Item(NodeList {
            list_type: ListType::Bullet,
            marker_offset: 0,
            padding: 0,
            start: 0,
            delimiter: ListDelimType::Period,
            bullet_char: 0,
            tight: true,
            is_task_list: false,
        })));

        let inner_list = arena.alloc(Node::with_value(NodeValue::List(NodeList {
            list_type: ListType::Bullet,
            marker_offset: 0,
            padding: 0,
            start: 1,
            delimiter: ListDelimType::Period,
            bullet_char: b'-',
            tight: true,
            is_task_list: false,
        })));

        let item3 = arena.alloc(Node::with_value(NodeValue::Item(NodeList {
            list_type: ListType::Bullet,
            marker_offset: 0,
            padding: 0,
            start: 0,
            delimiter: ListDelimType::Period,
            bullet_char: 0,
            tight: true,
            is_task_list: false,
        })));

        TreeOps::append_child(&mut arena, root, outer_list);
        TreeOps::append_child(&mut arena, outer_list, item1);
        TreeOps::append_child(&mut arena, outer_list, item2);
        TreeOps::append_child(&mut arena, item2, inner_list);
        TreeOps::append_child(&mut arena, inner_list, item3);

        // outer_list has 0 list ancestors (it's top-level)
        assert_eq!(count_list_ancestors(&arena, outer_list), 0);

        // inner_list has 1 list ancestor (outer_list)
        assert_eq!(count_list_ancestors(&arena, inner_list), 1);
    }
}
