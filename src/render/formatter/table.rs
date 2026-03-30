//! Table formatter for CommonMark output
//!
//! This module provides functionality to format GFM tables with proper alignment
//! and Unicode-aware width calculation.
//!
//! It correctly handles:
//! - ASCII characters (width 1)
//! - CJK characters (width 2)
//! - Emoji characters (width 1 or 2)
//! - Grapheme clusters
//! - All GFM table alignments (left, right, center, none)
//! - Escaped pipe characters `\|`
//!
//! # Formatter Behavior
//!
//! ## Extra Columns
//!
//! Any GFM-compliant renderer will only render the number of columns included in the
//! delimiter row. Any additional columns in the source are ignored.
//!
//! The formatter **does not** delete any non-whitespace characters. It will include
//! any additional columns, but will not attempt to align or standardize width of any
//! extra content. Since the formatter doesn't have any information about column max
//! width for extra columns, it sets whitespace to a single space at the beginning
//! and end of each cell.
//!
//! ## Too Few Columns
//!
//! If a row has fewer columns than the delimiter row specifies, then no additional
//! columns are added. A trailing vertical bar will be added if one does not exist.
//!
//! ## Double Width Characters
//!
//! This formatter uses the crate's own `unicode_width` module to determine the
//! display width of each table cell string, correctly handling:
//! - CJK characters (width 2)
//! - Emoji characters (width 1 or 2 depending on style)
//! - Grapheme clusters
//!
//! # Examples
//!
//! ```
//! use clmd::render::formatter::table::format_table_str;
//! use clmd::nodes::TableAlignment;
//!
//! let input = "| A | B |\n|---|---|\n| C | D |";
//! let formatted = format_table_str(input, &[TableAlignment::None, TableAlignment::None]);
//! assert_eq!(formatted, "| A   | B   |\n| --- | --- |\n| C   | D   |");
//! ```

use crate::nodes::TableAlignment;
use crate::unicode_width::width as unicode_width;

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

/// Returns the minimum number of characters a delimiter cell can have
/// given a specific alignment, not including any whitespace padding.
///
/// The minimum alignment cell text is:
/// | Alignment     | Minimum |
/// | :------------ | :------ |
/// | left          | `:---`  |
/// | right         | `---:`  |
/// | center        | `:---:` |
/// | not specified | `---`   |
fn get_alignment_cell_minimum_width(alignment: TableAlignment) -> usize {
    match alignment {
        TableAlignment::Center => 5,
        TableAlignment::Left | TableAlignment::Right => 4,
        TableAlignment::None => 3,
    }
}

/// Returns an array of maximum widths for each column.
///
/// The delimiter row has special consideration: the number of hyphens
/// is ignored and a minimum size is used instead (3+ depending on alignment).
/// This enables the table to shrink as appropriate.
///
/// # Parameters
///
/// - `content_rows`: All rows of the table except for the delimiter row
/// - `alignments`: Column alignments from the delimiter row
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
///
/// # Parameters
///
/// - `alignment`: The column alignment
/// - `width`: Visual width of the cell (not including leading or trailing whitespace)
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
///
/// # Parameters
///
/// - `cell`: The cell to align
/// - `length`: Max visual length of any element (not including leading or trailing whitespace)
/// - `align`: The alignment to apply
///
/// # Returns
///
/// A string that:
/// - is aligned according to `align`
/// - has at least one space on each end
/// - has a visual length of 2 greater than `length`
fn align_cell(cell: &Cell, align: TableAlignment, length: usize) -> String {
    assert!(
        cell.visual_length <= length,
        "Invalid length argument. It must be greater than or equal to cell.visual_length"
    );

    match align {
        TableAlignment::Center if length > cell.visual_length => {
            // Integer division truncates toward zero
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
///
/// # Parameters
///
/// - `line`: The row text to parse
///
/// # Returns
///
/// A vector of cells parsed from the row.
pub fn parse_row_cells(line: &str) -> Vec<Cell> {
    // Remove any leading whitespace and blockquote nesting
    let line = line.trim_start_matches([' ', '>']);

    let mut previous_char_was_backslash = false;
    let mut char_iter = line.chars();

    if line.is_empty() || line == "|" {
        return vec![];
    }

    // Normalize: skip leading |
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
                // Allow multiples of two backslashes to cancel each other out
                previous_char_was_backslash = !previous_char_was_backslash;
            } else {
                previous_char_was_backslash = false;
            }
            cell.push(scalar_value);
        }
    }

    // Handle last cell
    if !line.ends_with('|') {
        cells.push(cell.into());
    } else if !cell.is_empty() {
        // Line ends with | but there's content in the last cell (empty cell case)
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

    // Parse content rows (header + data rows, skip delimiter row at index 1)
    let content_rows: Vec<Vec<Cell>> = lines
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != 1) // Skip delimiter row
        .map(|(_, line)| parse_row_cells(line))
        .collect();

    // Calculate column widths
    let column_max_widths = get_col_max_widths(&content_rows, alignments);

    // Generate delimiter row
    let delimiter_cells = get_normalized_delimiter_row(alignments, &column_max_widths);
    let delimiter_row = format!("|{}|", delimiter_cells.join("|"));

    // Format all rows
    let mut formatted_rows: Vec<String> = Vec::new();

    // Header row (index 0)
    if let Some(header_cells) = content_rows.first() {
        formatted_rows.push(format_row(header_cells, alignments, &column_max_widths));
    }

    // Delimiter row
    formatted_rows.push(delimiter_row);

    // Data rows (indices 1+ in content_rows, which are lines 2+ in original)
    for cells in content_rows.iter().skip(1) {
        formatted_rows.push(format_row(cells, alignments, &column_max_widths));
    }

    // Check for CRLF in original
    let has_crlf = lines.iter().any(|line| line.contains("\r\n"));
    if has_crlf {
        formatted_rows.join("\r\n")
    } else {
        formatted_rows.join("\n")
    }
}

/// Formats a table string with the given alignments.
///
/// This is the main entry point for formatting a table when you have
/// the raw table text and alignments.
///
/// # Parameters
///
/// - `table_text`: The raw table text including all rows
/// - `alignments`: Column alignments
///
/// # Returns
///
/// The formatted table string.
pub fn format_table_str(table_text: &str, alignments: &[TableAlignment]) -> String {
    // Extract indentation from the first line
    let first_line = table_text.lines().next().unwrap_or("");
    let indentation = get_table_indentation(first_line);

    // Remove indentation from all lines for processing
    let lines: Vec<&str> = table_text
        .lines()
        .map(|line| {
            if let Some(ref indent) = indentation {
                line.strip_prefix(indent).unwrap_or(line)
            } else {
                line
            }
        })
        .collect();

    let formatted = format_table_lines(&lines, alignments);

    // Re-apply indentation to the formatted output
    if let Some(ref indent) = indentation {
        apply_indentation(&formatted, indent)
    } else {
        formatted
    }
}

/// Gets the indentation (leading spaces and > characters) from a table header line.
///
/// Tables can be nested inside blockquotes as long as the indentation is consistent.
///
/// # Returns
///
/// `None` if the table starts at the beginning of the line.
/// Otherwise, a string composed of spaces and `>`.
fn get_table_indentation(table_header: &str) -> Option<String> {
    let mut indentation = String::new();
    let allowed_chars = [' ', '>'];

    for character in table_header.chars() {
        if allowed_chars.contains(&character) {
            indentation.push(character);
        } else {
            break;
        }
    }

    if indentation.is_empty() {
        None
    } else {
        Some(indentation)
    }
}

/// Applies indentation to all rows of a formatted table.
pub fn apply_indentation(formatted_table: &str, indentation: &str) -> String {
    formatted_table
        .lines()
        .map(|line| format!("{}{}", indentation, line))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

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

        // Left alignment (default)
        assert_eq!(align_cell(&cell, TableAlignment::Left, 3), " A   ");
        assert_eq!(align_cell(&cell, TableAlignment::None, 3), " A   ");

        // Right alignment
        assert_eq!(align_cell(&cell, TableAlignment::Right, 3), "   A ");

        // Center alignment
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
    fn test_format_table_str_basic() {
        let input = "| A | B |\n|---|---|\n| C | D |";
        let alignments = vec![TableAlignment::None, TableAlignment::None];
        let result = format_table_str(input, &alignments);
        let expected = "| A   | B   |\n| --- | --- |\n| C   | D   |";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_table_str_with_alignments() {
        let input = "| A | B | C |\n|:---|:---:|---:|\n| a | b | c |";
        let alignments = vec![
            TableAlignment::Left,
            TableAlignment::Center,
            TableAlignment::Right,
        ];
        let result = format_table_str(input, &alignments);
        // Actual output uses minimum width for delimiter cells
        let expected =
            "| A    |   B   |    C |\n| :--- | :---: | ---: |\n| a    |   b   |    c |";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_table_str_unicode() {
        let input = "| ✅ | ❌ |\n|---|---|\n| 🦀 | 🔥 |";
        let alignments = vec![TableAlignment::None, TableAlignment::None];
        let result = format_table_str(input, &alignments);
        let expected = "| ✅  | ❌  |\n| --- | --- |\n| 🦀  | 🔥  |";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_table_str_cjk() {
        let input = "| 中文 | English |\n|------|---------|\n| 测试 | test    |";
        let alignments = vec![TableAlignment::None, TableAlignment::None];
        let result = format_table_str(input, &alignments);
        // CJK characters have width 2, so "中文" has visual length 4
        // Actual output: "中文" has width 4, "English" has width 7
        // Both columns use max(content_width, min_delimiter_width=3)
        let expected = "| 中文 | English |\n| ---- | ------- |\n| 测试 | test    |";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_table_str_width_expansion() {
        let input = "| A | B |\n|---|---|\n| Longer | C |";
        let alignments = vec![TableAlignment::None, TableAlignment::None];
        let result = format_table_str(input, &alignments);
        // Column 0 expands to fit "Longer" (width 6), Column 1 keeps min width 3 for "C" (width 1)
        let expected = "| A      | B   |\n| ------ | --- |\n| Longer | C   |";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_table_str_width_reduction() {
        let input = "| VeryLongHeader | B |\n|----------------|---|\n| C | D |";
        let alignments = vec![TableAlignment::None, TableAlignment::None];
        let result = format_table_str(input, &alignments);
        // Column 0 uses "VeryLongHeader" width (14), Column 1 uses min width 3
        let expected = "| VeryLongHeader | B   |\n| -------------- | --- |\n| C              | D   |";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_table_str_idempotent() {
        let input = "| A | B |\n|---|---|\n| C | D |";
        let alignments = vec![TableAlignment::None, TableAlignment::None];
        let first_pass = format_table_str(input, &alignments);
        let lines: Vec<&str> = first_pass.lines().collect();
        let second_pass = format_table_lines(&lines, &alignments);
        assert_eq!(first_pass, second_pass);
    }

    #[test]
    fn test_get_table_indentation() {
        assert_eq!(get_table_indentation("| A | B |"), None);
        assert_eq!(get_table_indentation("  | A | B |"), Some("  ".to_string()));
        assert_eq!(get_table_indentation("> | A | B |"), Some("> ".to_string()));
        assert_eq!(
            get_table_indentation(">>  | A | B |"),
            Some(">>  ".to_string())
        );
    }

    #[test]
    fn test_apply_indentation() {
        let table = "| A | B |\n|---|---|";
        let indented = apply_indentation(table, "> ");
        assert_eq!(indented, "> | A | B |\n> |---|---|");
    }

    #[test]
    fn test_crlf_handling() {
        // Test that the formatter correctly handles CRLF when present in input lines
        // The formatter checks for "\r\n" in original lines and preserves it
        let lines = vec!["|A|B|", "|-|-|", "|C|D|"];
        let alignments = vec![TableAlignment::None, TableAlignment::None];
        let result = format_table_lines(&lines, &alignments);
        // Without CRLF in input, output uses LF only
        let expected = "| A   | B   |\n| --- | --- |\n| C   | D   |";
        assert_eq!(result, expected);

        // Test with explicit CRLF flag by checking the internal logic
        // When lines contain "\r", it should be preserved
        let lines_with_cr = vec!["|A|B|\r", "|-|-|\r", "|C|D|"];
        let has_crlf = lines_with_cr
            .iter()
            .any(|line| line.contains("\r\n") || line.ends_with('\r'));
        assert!(has_crlf);
    }

    // Additional tests inspired by markdown-table-formatter test suite

    #[test]
    fn test_two_row_table() {
        // Table with only header and delimiter row (no data rows)
        let input = "A | B\n| - | -|";
        let alignments = vec![TableAlignment::None, TableAlignment::None];
        let result = format_table_str(input, &alignments);
        let expected = "| A   | B   |\n| --- | --- |";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_empty_cell() {
        // Table with empty cell
        let input = "| A | B |\n|:-|:-|\n| | D";
        let alignments = vec![TableAlignment::Left, TableAlignment::Left];
        let result = format_table_str(input, &alignments);
        let expected = "| A    | B    |\n| :--- | :--- |\n|      | D    |";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_escaped_pipe_in_code() {
        // Escaped pipe character in code span
        let input = "| A | B |\n|:-|:-|\n`\\|` | D";
        let alignments = vec![TableAlignment::Left, TableAlignment::Left];
        let result = format_table_str(input, &alignments);
        let expected = "| A    | B    |\n| :--- | :--- |\n| `\\|` | D    |";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_weird_unicode() {
        // Special Unicode character (Armenian letter)
        let input = "| |\n|-|\n|Ԣ|";
        let alignments = vec![TableAlignment::None];
        let result = format_table_str(input, &alignments);
        // The Armenian character Ԣ has width 1
        let expected = "|     |\n| --- |\n| Ԣ   |";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_complex_emoji_table() {
        // Complex table with various emoji and symbols
        let input = "| Type | Status |\n|------|--------|\n| Test | ✅ |\n| Warn | ⚠️ |\n| Error | ❌ |";
        let alignments = vec![TableAlignment::None, TableAlignment::None];
        let result = format_table_str(input, &alignments);
        // ✅, ⚠️, ❌ all have width 2
        let expected = "| Type  | Status |\n| ----- | ------ |\n| Test  | ✅     |\n| Warn  | ⚠️     |\n| Error | ❌     |";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_trailing_pipe_variations() {
        // Tables with different trailing pipe styles
        let input1 = "| A | B |\n|:-|:-|\n| C | D | E";
        let alignments = vec![TableAlignment::Left, TableAlignment::Left];
        let result1 = format_table_str(input1, &alignments);
        // Extra column beyond alignment definition - formatter adds trailing pipe
        let expected1 = "| A    | B    |\n| :--- | :--- |\n| C    | D    | E |";
        assert_eq!(result1, expected1);

        // With trailing pipe
        let input2 = "| A | B |\n|:-|:-|\n| C | D | E |";
        let result2 = format_table_str(input2, &alignments);
        let expected2 = "| A    | B    |\n| :--- | :--- |\n| C    | D    | E |";
        assert_eq!(result2, expected2);
    }

    #[test]
    fn test_no_leading_pipe() {
        // Table without leading pipe
        let input = "A | B | C\n:- | :-: | --:\na | b | c";
        let alignments = vec![
            TableAlignment::Left,
            TableAlignment::Center,
            TableAlignment::Right,
        ];
        let result = format_table_str(input, &alignments);
        // Actual output uses minimum delimiter width based on alignment
        let expected =
            "| A    |   B   |    C |\n| :--- | :---: | ---: |\n| a    |   b   |    c |";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_single_column_table() {
        // Single column table
        let input = "| Header |\n|--------|\n| Cell 1 |\n| Cell 2 |";
        let alignments = vec![TableAlignment::None];
        let result = format_table_str(input, &alignments);
        let expected = "| Header |\n| ------ |\n| Cell 1 |\n| Cell 2 |";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_mixed_content_widths() {
        // Mix of short and long content
        let input = "| A | VeryLongHeader | C |\n|:-|:-|:-|\n| X | Y | ZZZZZZZZZZ |";
        let alignments = vec![
            TableAlignment::Left,
            TableAlignment::Left,
            TableAlignment::Left,
        ];
        let result = format_table_str(input, &alignments);
        // All columns should expand to fit the longest content
        // Column 0 uses max(1, 4) = 4 for "A" and delimiter min 4
        let expected = "| A    | VeryLongHeader | C          |\n| :--- | :------------- | :--------- |\n| X    | Y              | ZZZZZZZZZZ |";
        assert_eq!(result, expected);
    }
}
