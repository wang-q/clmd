//! Table element handlers for CommonMark formatting
//!
//! This module contains handlers for table elements like Table, TableRow,
//! and TableCell. It also provides table formatting functionality with
//! proper alignment and Unicode-aware width calculation.
//!
//! It correctly handles:
//! - ASCII characters (width 1)
//! - CJK characters (width 2)
//! - Emoji characters (width 1 or 2)
//! - Grapheme clusters
//! - All GFM table alignments (left, right, center, none)
//! - Escaped pipe characters `\|`

use crate::core::arena::{NodeId, TraverseExt};
use crate::core::nodes::TableAlignment;
use crate::render::commonmark::escaping::{
    escape_markdown_for_table_simple, escape_string, escape_url,
};
use crate::render::commonmark::writer::MarkdownWriter;
use crate::text::unicode::width as unicode_width;

// ============================================================================
// Table Cell Structure
// ============================================================================

/// Represents a table cell with its content and visual display width.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cell {
    /// The cell content without leading/trailing whitespace.
    pub content: String,
    /// The number of columns required to display the content in a monospace font.
    pub visual_length: usize,
}

impl From<Vec<char>> for Cell {
    fn from(value: Vec<char>) -> Self {
        let cell_string: String = value.into_iter().collect();
        let cell_string = cell_string.trim().to_string();

        Cell {
            visual_length: unicode_width(&cell_string) as usize,
            content: cell_string,
        }
    }
}

impl From<&str> for Cell {
    fn from(value: &str) -> Self {
        let trimmed = value.trim().to_string();
        Cell {
            visual_length: unicode_width(&trimmed) as usize,
            content: trimmed,
        }
    }
}

// ============================================================================
// AST Node Processing
// ============================================================================

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
            result.push_str(&escape_markdown_for_table_simple(text));
        }
        NodeValue::Code(code) => {
            let backticks = get_backtick_sequence(&code.literal);
            result.push_str(&backticks);
            result.push_str(&code.literal);
            result.push_str(&backticks);
        }
        NodeValue::Emph => {
            result.push('*');
            for child_id in arena.children_iter(node_id) {
                result.push_str(&collect_cell_text_content(arena, child_id));
            }
            result.push('*');
        }
        NodeValue::Strong => {
            result.push_str("**");
            for child_id in arena.children_iter(node_id) {
                result.push_str(&collect_cell_text_content(arena, child_id));
            }
            result.push_str("**");
        }
        NodeValue::Strikethrough => {
            result.push_str("~~");
            for child_id in arena.children_iter(node_id) {
                result.push_str(&collect_cell_text_content(arena, child_id));
            }
            result.push_str("~~");
        }
        NodeValue::Link(link) => {
            result.push('[');
            for child_id in arena.children_iter(node_id) {
                result.push_str(&collect_cell_text_content(arena, child_id));
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
            for child_id in arena.children_iter(node_id) {
                result.push_str(&collect_cell_text_content(arena, child_id));
            }
        }
    }

    result
}

/// Render a formatted table
///
/// Takes collected row and cell data, builds table lines,
/// and uses format_table_lines for Unicode-aware formatting.
pub fn render_formatted_table(
    rows: &[Vec<String>],
    alignments: &[TableAlignment],
    writer: &mut MarkdownWriter,
) {
    let rows: Vec<&Vec<String>> = rows.iter().filter(|row| !row.is_empty()).collect();

    if rows.is_empty() {
        return;
    }

    let mut lines: Vec<String> = Vec::new();

    if let Some(header_row) = rows.first() {
        let cells: Vec<String> = header_row.to_vec();
        let line = format!("| {} |", cells.join(" | "));
        lines.push(line);
    }

    let num_cols = alignments.len().max(1);
    let placeholder_delimiter: Vec<String> =
        (0..num_cols).map(|_| "---".to_string()).collect();
    lines.push(format!("| {} |", placeholder_delimiter.join(" | ")));

    for row in rows.iter().skip(1) {
        let cells: Vec<String> = row.to_vec();
        let line = format!("| {} |", cells.join(" | "));
        lines.push(line);
    }

    let line_refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();

    let formatted = format_table_lines(&line_refs, alignments);

    for line in formatted.lines() {
        writer.append(line);
        writer.line();
    }

    writer.blank_line();
}

/// Get the appropriate backtick sequence for inline code
///
/// Determines the minimum number of backticks needed to wrap the content
/// without conflicting with backticks inside the content.
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

// ============================================================================
// Table Formatting Core
// ============================================================================

/// Returns the minimum number of characters a delimiter cell can have
/// given a specific alignment, not including any whitespace padding.
fn get_alignment_cell_minimum_width(alignment: TableAlignment) -> usize {
    match alignment {
        TableAlignment::Center => 5,
        TableAlignment::Left | TableAlignment::Right => 4,
        TableAlignment::None => 3,
    }
}

/// Returns an array of maximum widths for each column.
fn get_col_max_widths(
    content_rows: &[Vec<Cell>],
    alignments: &[TableAlignment],
) -> Vec<usize> {
    let mut max_widths: Vec<usize> = Vec::with_capacity(alignments.len());

    for (index, alignment) in alignments.iter().enumerate() {
        let max_content_width = content_rows
            .iter()
            .filter(|row| row.get(index).is_some())
            .map(|row| row[index].visual_length)
            .max()
            .unwrap_or(0);

        let min_delimiter_width = get_alignment_cell_minimum_width(*alignment);
        max_widths.push(max_content_width.max(min_delimiter_width));
    }

    max_widths
}

/// Returns the formatted string for a delimiter cell.
fn format_delimiter_cell(alignment: TableAlignment, width: usize) -> String {
    match alignment {
        TableAlignment::Center => {
            format!(" :{}: ", "-".repeat(width.saturating_sub(2)))
        }
        TableAlignment::Left => {
            format!(" :{} ", "-".repeat(width.saturating_sub(1)))
        }
        TableAlignment::Right => {
            format!(" {}: ", "-".repeat(width.saturating_sub(1)))
        }
        TableAlignment::None => {
            format!(" {} ", "-".repeat(width))
        }
    }
}

/// Returns an array of strings with the delimiter cells normalized to align
/// with maximum column widths.
fn get_normalized_delimiter_row(
    alignments: &[TableAlignment],
    column_max_widths: &[usize],
) -> Vec<String> {
    assert_eq!(
        alignments.len(),
        column_max_widths.len(),
        "The length of alignments and column_max_widths must be equal"
    );

    alignments
        .iter()
        .zip(column_max_widths.iter())
        .map(|(alignment, width)| format_delimiter_cell(*alignment, *width))
        .collect()
}

/// Adds whitespace as necessary according to `align`.
fn align_cell(cell: &Cell, align: TableAlignment, length: usize) -> String {
    assert!(
        cell.visual_length <= length,
        "Invalid length argument. It must be greater than or equal to cell.visual_length"
    );

    match align {
        TableAlignment::Center if length > cell.visual_length => {
            let leading_whitespace = (length - cell.visual_length) / 2;
            let trailing_whitespace = length - cell.visual_length - leading_whitespace;

            format!(
                " {}{}{} ",
                " ".repeat(leading_whitespace),
                cell.content,
                " ".repeat(trailing_whitespace)
            )
        }
        TableAlignment::Right => {
            format!(
                " {}{} ",
                " ".repeat(length - cell.visual_length),
                cell.content
            )
        }
        _ => {
            format!(
                " {}{} ",
                cell.content,
                " ".repeat(length - cell.visual_length)
            )
        }
    }
}

/// Parses a table row and returns cells.
///
/// The GFM specification does not distinguish between vertical bars in code
/// blocks or regular vertical bars. Even in code blocks, they need to be escaped.
pub fn parse_row_cells(line: &str) -> Vec<Cell> {
    let line = line.trim_start_matches([' ', '>']);

    let mut previous_char_was_backslash = false;
    let mut char_iter = line.chars();

    if line.is_empty() || line == "|" {
        return vec![];
    }

    if line.starts_with('|') {
        char_iter.next();
    }

    let mut cells: Vec<Cell> = Vec::new();
    let mut cell: Vec<char> = Vec::new();

    for scalar_value in char_iter {
        if scalar_value == '|' && !previous_char_was_backslash {
            cells.push(cell.into());
            cell = Vec::new();
        } else {
            if scalar_value == '\\' {
                previous_char_was_backslash = !previous_char_was_backslash;
            } else {
                previous_char_was_backslash = false;
            }
            cell.push(scalar_value);
        }
    }

    if !line.ends_with('|') || !cell.is_empty() {
        cells.push(cell.into());
    }

    cells
}

/// Formats a single table row given cells, alignments, and column widths.
fn format_row(
    cells: &[Cell],
    alignments: &[TableAlignment],
    column_max_widths: &[usize],
) -> String {
    let mut aligned_cells: Vec<String> = Vec::new();

    for (index, cell) in cells.iter().enumerate() {
        let align = alignments
            .get(index)
            .copied()
            .unwrap_or(TableAlignment::None);
        let length = column_max_widths
            .get(index)
            .copied()
            .unwrap_or_else(|| unicode_width(&cell.content) as usize);

        aligned_cells.push(align_cell(cell, align, length));
    }

    format!("|{}|", aligned_cells.join("|"))
}

/// Formats a complete GFM table.
///
/// # Parameters
///
/// - `lines`: The table lines (header, delimiter, and data rows)
/// - `alignments`: Column alignments parsed from the delimiter row
///
/// # Returns
///
/// The formatted table as a string.
pub fn format_table_lines(lines: &[&str], alignments: &[TableAlignment]) -> String {
    if lines.len() < 2 {
        return lines.join("\n");
    }

    let content_rows: Vec<Vec<Cell>> = lines
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != 1)
        .map(|(_, line)| parse_row_cells(line))
        .collect();

    let column_max_widths = get_col_max_widths(&content_rows, alignments);

    let delimiter_cells = get_normalized_delimiter_row(alignments, &column_max_widths);
    let delimiter_row = format!("|{}|", delimiter_cells.join("|"));

    let mut formatted_rows: Vec<String> = Vec::new();

    if let Some(header_cells) = content_rows.first() {
        formatted_rows.push(format_row(header_cells, alignments, &column_max_widths));
    }

    formatted_rows.push(delimiter_row);

    for cells in content_rows.iter().skip(1) {
        formatted_rows.push(format_row(cells, alignments, &column_max_widths));
    }

    let has_crlf = lines.iter().any(|line| line.contains("\r\n"));
    if has_crlf {
        formatted_rows.join("\r\n")
    } else {
        formatted_rows.join("\n")
    }
}

// ============================================================================
// Tests
// ============================================================================

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

    #[test]
    fn test_cell_from_str() {
        let cell = Cell::from("hello");
        assert_eq!(cell.content, "hello");
        assert_eq!(cell.visual_length, 5);

        let cell = Cell::from("  world  ");
        assert_eq!(cell.content, "world");
        assert_eq!(cell.visual_length, 5);
    }

    #[test]
    fn test_cell_from_chars() {
        let chars: Vec<char> = "test".chars().collect();
        let cell = Cell::from(chars);
        assert_eq!(cell.content, "test");
        assert_eq!(cell.visual_length, 4);
    }

    #[test]
    fn test_get_alignment_cell_minimum_width() {
        assert_eq!(get_alignment_cell_minimum_width(TableAlignment::None), 3);
        assert_eq!(get_alignment_cell_minimum_width(TableAlignment::Left), 4);
        assert_eq!(get_alignment_cell_minimum_width(TableAlignment::Right), 4);
        assert_eq!(get_alignment_cell_minimum_width(TableAlignment::Center), 5);
    }

    #[test]
    fn test_format_delimiter_cell() {
        assert_eq!(format_delimiter_cell(TableAlignment::None, 3), " --- ");
        assert_eq!(format_delimiter_cell(TableAlignment::Left, 4), " :--- ");
        assert_eq!(format_delimiter_cell(TableAlignment::Right, 4), " ---: ");
        assert_eq!(format_delimiter_cell(TableAlignment::Center, 5), " :---: ");
    }

    #[test]
    fn test_align_cell() {
        let cell = Cell::from("A");

        assert_eq!(align_cell(&cell, TableAlignment::Left, 3), " A   ");
        assert_eq!(align_cell(&cell, TableAlignment::None, 3), " A   ");
        assert_eq!(align_cell(&cell, TableAlignment::Right, 3), "   A ");
        assert_eq!(align_cell(&cell, TableAlignment::Center, 3), "  A  ");
    }

    #[test]
    fn test_parse_row_cells_simple() {
        let cells = parse_row_cells("| a | b | c |");
        assert_eq!(cells.len(), 3);
        assert_eq!(cells[0].content, "a");
        assert_eq!(cells[1].content, "b");
        assert_eq!(cells[2].content, "c");
    }

    #[test]
    fn test_parse_row_cells_no_pipes() {
        let cells = parse_row_cells("a | b | c");
        assert_eq!(cells.len(), 3);
        assert_eq!(cells[0].content, "a");
        assert_eq!(cells[1].content, "b");
        assert_eq!(cells[2].content, "c");
    }

    #[test]
    fn test_parse_row_cells_escaped_pipe() {
        let cells = parse_row_cells("| a | b | \\| |");
        assert_eq!(cells.len(), 3);
        assert_eq!(cells[0].content, "a");
        assert_eq!(cells[1].content, "b");
        assert_eq!(cells[2].content, "\\|");
    }

    #[test]
    fn test_parse_row_cells_empty() {
        let cells = parse_row_cells("| | D");
        assert_eq!(cells.len(), 2);
        assert_eq!(cells[0].content, "");
        assert_eq!(cells[1].content, "D");
    }

    #[test]
    fn test_parse_row_cells_blockquote() {
        let cells = parse_row_cells("   > >>  ✅ | ❌");
        assert_eq!(cells.len(), 2);
        assert_eq!(cells[0].content, "✅");
        assert_eq!(cells[1].content, "❌");
    }

    #[test]
    fn test_format_table_lines_basic() {
        let lines = vec!["| A | B |", "|---|---|", "| C | D |"];
        let alignments = vec![TableAlignment::None, TableAlignment::None];
        let result = format_table_lines(&lines, &alignments);
        let expected = "| A   | B   |\n| --- | --- |\n| C   | D   |";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_table_lines_with_alignments() {
        let lines = vec!["| A | B | C |", "|:---|:---:|---:|", "| a | b | c |"];
        let alignments = vec![
            TableAlignment::Left,
            TableAlignment::Center,
            TableAlignment::Right,
        ];
        let result = format_table_lines(&lines, &alignments);
        let expected =
            "| A    |   B   |    C |\n| :--- | :---: | ---: |\n| a    |   b   |    c |";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_table_lines_unicode() {
        let lines = vec!["| ✅ | ❌ |", "|---|---|", "| 🦀 | 🔥 |"];
        let alignments = vec![TableAlignment::None, TableAlignment::None];
        let result = format_table_lines(&lines, &alignments);
        let expected = "| ✅  | ❌  |\n| --- | --- |\n| 🦀  | 🔥  |";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_table_lines_cjk() {
        let lines = vec![
            "| 中文 | English |",
            "|------|---------|",
            "| 测试 | test    |",
        ];
        let alignments = vec![TableAlignment::None, TableAlignment::None];
        let result = format_table_lines(&lines, &alignments);
        let expected = "| 中文 | English |\n| ---- | ------- |\n| 测试 | test    |";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_table_lines_width_expansion() {
        let lines = vec!["| A | B |", "|---|---|", "| Longer | C |"];
        let alignments = vec![TableAlignment::None, TableAlignment::None];
        let result = format_table_lines(&lines, &alignments);
        let expected = "| A      | B   |\n| ------ | --- |\n| Longer | C   |";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_table_lines_idempotent() {
        let lines = vec!["| A | B |", "|---|---|", "| C | D |"];
        let alignments = vec![TableAlignment::None, TableAlignment::None];
        let first_pass = format_table_lines(&lines, &alignments);
        let lines2: Vec<&str> = first_pass.lines().collect();
        let second_pass = format_table_lines(&lines2, &alignments);
        assert_eq!(first_pass, second_pass);
    }

    #[test]
    fn test_crlf_handling() {
        let lines = vec!["|A|B|", "|-|-|", "|C|D|"];
        let alignments = vec![TableAlignment::None, TableAlignment::None];
        let result = format_table_lines(&lines, &alignments);
        let expected = "| A   | B   |\n| --- | --- |\n| C   | D   |";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_two_row_table() {
        let lines = vec!["A | B", "| - | -|"];
        let alignments = vec![TableAlignment::None, TableAlignment::None];
        let result = format_table_lines(&lines, &alignments);
        let expected = "| A   | B   |\n| --- | --- |";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_empty_cell() {
        let lines = vec!["| A | B |", "|:-|:-|", "| | D"];
        let alignments = vec![TableAlignment::Left, TableAlignment::Left];
        let result = format_table_lines(&lines, &alignments);
        let expected = "| A    | B    |\n| :--- | :--- |\n|      | D    |";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_escaped_pipe_in_code() {
        let lines = vec!["| A | B |", "|:-|:-|", "`\\|` | D"];
        let alignments = vec![TableAlignment::Left, TableAlignment::Left];
        let result = format_table_lines(&lines, &alignments);
        let expected = "| A    | B    |\n| :--- | :--- |\n| `\\|` | D    |";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_weird_unicode() {
        let lines = vec!["| |", "|-|", "|Ԣ|"];
        let alignments = vec![TableAlignment::None];
        let result = format_table_lines(&lines, &alignments);
        let expected = "|     |\n| --- |\n| Ԣ   |";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_complex_emoji_table() {
        let lines = vec![
            "| Type | Status |",
            "|------|--------|",
            "| Test | ✅ |",
            "| Warn | ⚠️ |",
            "| Error | ❌ |",
        ];
        let alignments = vec![TableAlignment::None, TableAlignment::None];
        let result = format_table_lines(&lines, &alignments);
        let expected = "| Type  | Status |\n| ----- | ------ |\n| Test  | ✅     |\n| Warn  | ⚠️     |\n| Error | ❌     |";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_trailing_pipe_variations() {
        let lines1 = vec!["| A | B |", "|:-|:-|", "| C | D | E"];
        let alignments = vec![TableAlignment::Left, TableAlignment::Left];
        let result1 = format_table_lines(&lines1, &alignments);
        let expected1 = "| A    | B    |\n| :--- | :--- |\n| C    | D    | E |";
        assert_eq!(result1, expected1);

        let lines2 = vec!["| A | B |", "|:-|:-|", "| C | D | E |"];
        let result2 = format_table_lines(&lines2, &alignments);
        let expected2 = "| A    | B    |\n| :--- | :--- |\n| C    | D    | E |";
        assert_eq!(result2, expected2);
    }

    #[test]
    fn test_no_leading_pipe() {
        let lines = vec!["A | B | C", ":- | :-: | --:", "a | b | c"];
        let alignments = vec![
            TableAlignment::Left,
            TableAlignment::Center,
            TableAlignment::Right,
        ];
        let result = format_table_lines(&lines, &alignments);
        let expected =
            "| A    |   B   |    C |\n| :--- | :---: | ---: |\n| a    |   b   |    c |";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_single_column_table() {
        let lines = vec!["| Header |", "|--------|", "| Cell 1 |", "| Cell 2 |"];
        let alignments = vec![TableAlignment::None];
        let result = format_table_lines(&lines, &alignments);
        let expected = "| Header |\n| ------ |\n| Cell 1 |\n| Cell 2 |";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_mixed_content_widths() {
        let lines = vec![
            "| A | VeryLongHeader | C |",
            "|:-|:-|:-|",
            "| X | Y | ZZZZZZZZZZ |",
        ];
        let alignments = vec![
            TableAlignment::Left,
            TableAlignment::Left,
            TableAlignment::Left,
        ];
        let result = format_table_lines(&lines, &alignments);
        let expected = "| A    | VeryLongHeader | C          |\n| :--- | :------------- | :--------- |\n| X    | Y              | ZZZZZZZZZZ |";
        assert_eq!(result, expected);
    }
}
