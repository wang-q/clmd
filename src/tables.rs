//! Table extension for GitHub Flavored Markdown
//!
//! This module implements table parsing according to GFM spec.
//! Tables are defined by a header row, a delimiter row, and zero or more data rows.
//!
//! Example:
//! ```markdown
//! | Header 1 | Header 2 |
//! |----------|----------|
//! | Cell 1   | Cell 2   |
//! | Cell 3   | Cell 4   |
//! ```

use crate::node::{append_child, Node, NodeData, NodeType, SourcePos, TableAlignment};
use std::cell::RefCell;
use std::rc::Rc;

/// Check if a line looks like a table row (contains |)
pub fn is_table_row(line: &str) -> bool {
    line.contains('|')
}

/// Check if a line is a table delimiter row
/// Format: | --- | :--- | :---: | ---: |
pub fn is_delimiter_row(line: &str) -> bool {
    let trimmed = line.trim();
    if !trimmed.starts_with('|') && !trimmed.contains('|') {
        return false;
    }

    // Split by | and check each cell
    let cells: Vec<&str> = trimmed.split('|').collect();
    let mut valid_cells = 0;

    for cell in cells.iter().skip(1) {
        let cell_trimmed = cell.trim();
        if cell_trimmed.is_empty() {
            continue;
        }

        // Check if it's a valid delimiter cell
        // Pattern: optional :, then 3+ dashes, then optional :
        let bytes = cell_trimmed.as_bytes();
        let mut i = 0;

        // Check for leading :
        if i < bytes.len() && bytes[i] == b':' {
            i += 1;
        }

        // Need at least 3 dashes
        let dash_start = i;
        while i < bytes.len() && bytes[i] == b'-' {
            i += 1;
        }
        let dash_count = i - dash_start;

        if dash_count < 3 {
            return false;
        }

        // Check for trailing :
        if i < bytes.len() {
            if bytes[i] != b':' || i != bytes.len() - 1 {
                return false;
            }
        }

        valid_cells += 1;
    }

    valid_cells > 0
}

/// Parse alignment from a delimiter cell
/// - `---` -> None
/// - `:---` -> Left
/// - `---:` -> Right
/// - `:---:` -> Center
fn parse_alignment(cell: &str) -> TableAlignment {
    let trimmed = cell.trim();
    let has_left = trimmed.starts_with(':');
    let has_right = trimmed.ends_with(':');

    match (has_left, has_right) {
        (true, true) => TableAlignment::Center,
        (true, false) => TableAlignment::Left,
        (false, true) => TableAlignment::Right,
        (false, false) => TableAlignment::None,
    }
}

/// Parse a table row and return cell contents
fn parse_row_cells(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    let mut cells = Vec::new();

    // Handle leading |
    let content = if trimmed.starts_with('|') {
        &trimmed[1..]
    } else {
        trimmed
    };

    // Handle trailing |
    let content = if content.ends_with('|') {
        &content[..content.len() - 1]
    } else {
        content
    };

    // Split by | and trim each cell
    for cell in content.split('|') {
        cells.push(cell.trim().to_string());
    }

    cells
}

/// Parse alignments from delimiter row
fn parse_alignments(line: &str) -> Vec<TableAlignment> {
    let trimmed = line.trim();
    let mut alignments = Vec::new();

    // Handle leading |
    let content = if trimmed.starts_with('|') {
        &trimmed[1..]
    } else {
        trimmed
    };

    // Handle trailing |
    let content = if content.ends_with('|') {
        &content[..content.len() - 1]
    } else {
        content
    };

    // Split by | and parse each cell
    for cell in content.split('|') {
        alignments.push(parse_alignment(cell));
    }

    alignments
}

/// Try to parse a table from the given lines
/// Returns (table_node, lines_consumed) if successful
pub fn try_parse_table(
    lines: &[&str],
    start_line: usize,
) -> Option<(Rc<RefCell<Node>>, usize)> {
    if lines.is_empty() || !is_table_row(lines[0]) {
        return None;
    }

    // Need at least 2 lines: header and delimiter
    if lines.len() < 2 {
        return None;
    }

    let header_line = lines[0];
    let delimiter_line = lines[1];

    // Second line must be a delimiter row
    if !is_delimiter_row(delimiter_line) {
        return None;
    }

    // Parse header cells
    let header_cells = parse_row_cells(header_line);
    let num_columns = header_cells.len();

    if num_columns == 0 {
        return None;
    }

    // Parse alignments
    let alignments = parse_alignments(delimiter_line);

    // Ensure alignments match header columns
    let alignments = if alignments.len() < num_columns {
        let mut extended = alignments;
        extended.resize(num_columns, TableAlignment::None);
        extended
    } else {
        alignments.into_iter().take(num_columns).collect()
    };

    // Create table node
    let table_node = Rc::new(RefCell::new(Node::new(NodeType::Table)));
    {
        let mut table = table_node.borrow_mut();
        table.data = NodeData::Table {
            num_columns,
            alignments: alignments.clone(),
        };
        table.source_pos = SourcePos {
            start_line: start_line as u32,
            start_column: 1,
            end_line: start_line as u32,
            end_column: header_line.len() as u32,
        };
    }

    // Create table head
    let thead_node = Rc::new(RefCell::new(Node::new(NodeType::TableHead)));
    thead_node.borrow_mut().data = NodeData::TableHead;
    thead_node.borrow_mut().source_pos = SourcePos {
        start_line: start_line as u32,
        start_column: 1,
        end_line: start_line as u32,
        end_column: header_line.len() as u32,
    };

    // Create header row
    let header_row = Rc::new(RefCell::new(Node::new(NodeType::TableRow)));
    header_row.borrow_mut().data = NodeData::TableRow;

    // Create header cells
    for (i, cell_content) in header_cells.iter().enumerate() {
        let cell = Rc::new(RefCell::new(Node::new(NodeType::TableCell)));
        cell.borrow_mut().data = NodeData::TableCell {
            column_index: i,
            alignment: alignments.get(i).copied().unwrap_or(TableAlignment::None),
            is_header: true,
        };

        // Create paragraph for cell content
        let para = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        para.borrow_mut().data = NodeData::Paragraph;

        // Create text node for content
        let text = Rc::new(RefCell::new(Node::new(NodeType::Text)));
        text.borrow_mut().data = NodeData::Text {
            literal: cell_content.clone(),
        };

        // Build tree: cell -> para -> text
        append_child(&para, text);
        append_child(&cell, para);
        append_child(&header_row, cell);
    }

    append_child(&thead_node, header_row);
    append_child(&table_node, thead_node);

    // Parse body rows (optional)
    let mut lines_consumed = 2;
    let mut body_rows = Vec::new();

    for (i, line) in lines[2..].iter().enumerate() {
        if !is_table_row(line) || line.trim().is_empty() {
            break;
        }

        let row_cells = parse_row_cells(line);
        let row_line_num = start_line + 2 + i;

        let row = Rc::new(RefCell::new(Node::new(NodeType::TableRow)));
        row.borrow_mut().data = NodeData::TableRow;
        row.borrow_mut().source_pos = SourcePos {
            start_line: row_line_num as u32,
            start_column: 1,
            end_line: row_line_num as u32,
            end_column: line.len() as u32,
        };

        // Create cells for this row
        for (j, cell_content) in row_cells.iter().enumerate().take(num_columns) {
            let cell = Rc::new(RefCell::new(Node::new(NodeType::TableCell)));
            cell.borrow_mut().data = NodeData::TableCell {
                column_index: j,
                alignment: alignments.get(j).copied().unwrap_or(TableAlignment::None),
                is_header: false,
            };

            // Create paragraph for cell content
            let para = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
            para.borrow_mut().data = NodeData::Paragraph;

            // Create text node for content
            let text = Rc::new(RefCell::new(Node::new(NodeType::Text)));
            text.borrow_mut().data = NodeData::Text {
                literal: cell_content.clone(),
            };

            // Build tree: cell -> para -> text
            append_child(&para, text);
            append_child(&cell, para);
            append_child(&row, cell);
        }

        body_rows.push(row);
        lines_consumed += 1;
    }

    // Add body rows to table (without tbody wrapper for simplicity)
    for row in body_rows {
        append_child(&table_node, row);
    }

    // Update table end position
    let end_line = start_line + lines_consumed - 1;
    let last_line = lines[lines_consumed - 1];
    table_node.borrow_mut().source_pos.end_line = end_line as u32;
    table_node.borrow_mut().source_pos.end_column = last_line.len() as u32;

    Some((table_node, lines_consumed))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_table_row() {
        assert!(is_table_row("| a | b |"));
        assert!(is_table_row("a | b"));
        assert!(!is_table_row("just text"));
    }

    #[test]
    fn test_is_delimiter_row() {
        assert!(is_delimiter_row("| --- | --- |"));
        assert!(is_delimiter_row("|:---|:---:|---:|"));
        assert!(!is_delimiter_row("| - | - |")); // Should fail - only 1 dash
        assert!(!is_delimiter_row("| a | b |"));
    }

    #[test]
    fn test_parse_alignment() {
        assert_eq!(parse_alignment("---"), TableAlignment::None);
        assert_eq!(parse_alignment(":---"), TableAlignment::Left);
        assert_eq!(parse_alignment("---:"), TableAlignment::Right);
        assert_eq!(parse_alignment(":---:"), TableAlignment::Center);
    }

    #[test]
    fn test_parse_row_cells() {
        let cells = parse_row_cells("| a | b | c |");
        assert_eq!(cells, vec!["a", "b", "c"]);

        let cells = parse_row_cells("a | b | c");
        assert_eq!(cells, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_try_parse_table() {
        let lines = vec![
            "| Header 1 | Header 2 |",
            "|----------|----------|",
            "| Cell 1   | Cell 2   |",
            "| Cell 3   | Cell 4   |",
        ];

        let result = try_parse_table(&lines, 1);
        assert!(result.is_some());

        let (table, lines_consumed) = result.unwrap();
        assert_eq!(lines_consumed, 4);

        let table_ref = table.borrow();
        match &table_ref.data {
            NodeData::Table {
                num_columns,
                alignments,
            } => {
                assert_eq!(*num_columns, 2);
                assert_eq!(alignments.len(), 2);
                assert_eq!(alignments[0], TableAlignment::None);
                assert_eq!(alignments[1], TableAlignment::None);
            }
            _ => panic!("Expected Table data"),
        }
    }

    #[test]
    fn test_table_with_alignments() {
        let lines = vec![
            "| Left | Center | Right |",
            "|:-----|:------:|------:|",
            "| a    | b      | c     |",
        ];

        let result = try_parse_table(&lines, 1);
        assert!(result.is_some());

        let (table, _) = result.unwrap();
        let table_ref = table.borrow();
        match &table_ref.data {
            NodeData::Table { alignments, .. } => {
                assert_eq!(alignments[0], TableAlignment::Left);
                assert_eq!(alignments[1], TableAlignment::Center);
                assert_eq!(alignments[2], TableAlignment::Right);
            }
            _ => panic!("Expected Table data"),
        }
    }
}
