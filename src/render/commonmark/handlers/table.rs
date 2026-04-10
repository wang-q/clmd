//! Table element handlers for CommonMark formatting
//!
//! This module contains handlers for table elements like Table, TableRow,
//! and TableCell.

use crate::core::arena::NodeId;
use crate::core::nodes::TableAlignment;
use crate::render::commonmark::escaping::{
    escape_markdown_for_table_simple, escape_string, escape_url,
};
use crate::render::commonmark::writer::MarkdownWriter;

/// Collect text content from a table cell node
///
/// This function recursively collects text content from a node and its children,
/// applying appropriate markdown formatting for inline elements.
pub fn collect_cell_text_content(
    arena: &crate::core::arena::NodeArena,
    node_id: NodeId,
) -> String {
    use crate::core::nodes::NodeValue;

    let mut result = String::new();
    let node = arena.get(node_id);

    match &node.value {
        NodeValue::Text(text) => {
            // Escape markdown special chars but not pipe
            result.push_str(&escape_markdown_for_table_simple(text));
        }
        NodeValue::Code(code) => {
            // Inline code
            let backticks = get_backtick_sequence(&code.literal);
            result.push_str(&backticks);
            result.push_str(&code.literal);
            result.push_str(&backticks);
        }
        NodeValue::Emph => {
            result.push('*');
            // Recursively collect children
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                result.push_str(&collect_cell_text_content(arena, child_id));
                child_opt = arena.get(child_id).next;
            }
            result.push('*');
        }
        NodeValue::Strong => {
            result.push_str("**");
            // Recursively collect children
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                result.push_str(&collect_cell_text_content(arena, child_id));
                child_opt = arena.get(child_id).next;
            }
            result.push_str("**");
        }
        NodeValue::Strikethrough => {
            result.push_str("~~");
            // Recursively collect children
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                result.push_str(&collect_cell_text_content(arena, child_id));
                child_opt = arena.get(child_id).next;
            }
            result.push_str("~~");
        }
        NodeValue::Link(link) => {
            result.push('[');
            // Recursively collect children for link text
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                result.push_str(&collect_cell_text_content(arena, child_id));
                child_opt = arena.get(child_id).next;
            }
            result.push_str("](");
            result.push_str(&escape_url(&link.url));
            if !link.title.is_empty() {
                result.push_str(&format!(" \"{}\"", escape_string(&link.title)));
            }
            result.push(')');
        }
        NodeValue::SoftBreak => {
            result.push(' ');
        }
        NodeValue::HardBreak => {
            result.push_str("  ");
        }
        _ => {
            // For other node types, just recursively collect children
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                result.push_str(&collect_cell_text_content(arena, child_id));
                child_opt = arena.get(child_id).next;
            }
        }
    }

    result
}

/// Render a formatted table using table.rs
///
/// Takes collected row and cell data, builds table lines,
/// and uses table::format_table_lines for Unicode-aware formatting.
pub fn render_formatted_table(
    rows: &[Vec<String>],
    alignments: &[TableAlignment],
    writer: &mut MarkdownWriter,
) {
    use crate::render::commonmark::table;

    // Filter out empty rows (e.g., header end markers)
    let rows: Vec<&Vec<String>> = rows.iter().filter(|row| !row.is_empty()).collect();

    if rows.is_empty() {
        return;
    }

    // Build table lines from collected data
    // format_table_lines expects: [header_row, delimiter_row, data_rows...]
    // It will skip the delimiter row at index 1 and generate its own
    let mut lines: Vec<String> = Vec::new();

    // Add header row (first row)
    if let Some(header_row) = rows.first() {
        let cells: Vec<String> = header_row.to_vec();
        let line = format!("| {} |", cells.join(" | "));
        lines.push(line);
    }

    // Add a placeholder delimiter row (will be skipped by format_table_lines)
    // We use a simple delimiter that matches the number of columns
    let num_cols = alignments.len().max(1);
    let placeholder_delimiter: Vec<String> =
        (0..num_cols).map(|_| "---".to_string()).collect();
    lines.push(format!("| {} |", placeholder_delimiter.join(" | ")));

    // Add data rows (remaining rows)
    for row in rows.iter().skip(1) {
        let cells: Vec<String> = row.to_vec();
        let line = format!("| {} |", cells.join(" | "));
        lines.push(line);
    }

    // Convert to &str slice for format_table_lines
    let line_refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();

    // Format the table using table.rs
    let formatted = table::format_table_lines(&line_refs, alignments);

    // Write the formatted table
    for line in formatted.lines() {
        writer.append(line);
        writer.line();
    }

    // Add blank line after table to separate from following content
    writer.blank_line();
}

/// Get the appropriate backtick sequence for inline code
///
/// Determines the minimum number of backticks needed to wrap the content
/// without conflicting with backticks inside the content.
///
/// # Examples
///
/// ```ignore
/// use clmd::render::commonmark::handlers::table::get_backtick_sequence;
///
/// assert_eq!(get_backtick_sequence("code"), "`");
/// assert_eq!(get_backtick_sequence("code `with` backticks"), "``");
/// assert_eq!(get_backtick_sequence("``double``"), "```");
/// ```
pub fn get_backtick_sequence(content: &str) -> String {
    let mut max_backticks = 0;
    let mut current = 0;

    for c in content.chars() {
        if c == '`' {
            current += 1;
            max_backticks = max_backticks.max(current);
        } else {
            current = 0;
        }
    }

    let count = (max_backticks + 1).max(1);
    "`".repeat(count)
}

/// Create handlers for table elements
///
/// This function returns an empty vector for now.
/// Full implementation will be added in a future phase.
pub fn create_table_handlers() -> Vec<()> {
    vec![]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_backtick_sequence() {
        assert_eq!(get_backtick_sequence("code"), "`");
        assert_eq!(get_backtick_sequence("code `with` backticks"), "``");
        assert_eq!(get_backtick_sequence("``double``"), "```");
        assert_eq!(get_backtick_sequence("```triple```"), "````");
    }

    #[test]
    fn test_get_backtick_sequence_empty() {
        assert_eq!(get_backtick_sequence(""), "`");
    }
}
