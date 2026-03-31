//! CSV parsing utilities for table data import.
//!
//! This module provides a simple CSV parser for importing table data into
//! Markdown documents. It supports customizable delimiters, quote characters,
//! and escape sequences.

use crate::error::{ClmdError, ClmdResult, Position};

/// Options for CSV parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CsvOptions {
    /// Field delimiter character (default: ',')
    pub delimiter: char,
    /// Quote character for quoted fields (default: Some('"'))
    pub quote: Option<char>,
    /// Whether to keep whitespace after delimiter (default: false)
    pub keep_space: bool,
    /// Escape character (default: None, meaning double quotes)
    pub escape: Option<char>,
}

impl Default for CsvOptions {
    fn default() -> Self {
        Self {
            delimiter: ',',
            quote: Some('"'),
            keep_space: false,
            escape: None,
        }
    }
}

impl CsvOptions {
    /// Create a new CSV options with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the delimiter character.
    pub fn delimiter(mut self, delim: char) -> Self {
        self.delimiter = delim;
        self
    }

    /// Set the quote character.
    pub fn quote(mut self, quote: Option<char>) -> Self {
        self.quote = quote;
        self
    }

    /// Set whether to keep whitespace after delimiter.
    pub fn keep_space(mut self, keep: bool) -> Self {
        self.keep_space = keep;
        self
    }

    /// Set the escape character.
    pub fn escape(mut self, escape: Option<char>) -> Self {
        self.escape = escape;
        self
    }
}

/// Parse CSV text into a 2D vector of strings.
///
/// # Arguments
///
/// * `input` - The CSV text to parse
/// * `options` - CSV parsing options
///
/// # Returns
///
/// Returns a `ClmdResult` containing a vector of rows, where each row
/// is a vector of field strings.
///
/// # Examples
///
/// ```ignore
/// use clmd::formats::csv::{parse_csv, CsvOptions};
///
/// let csv = "Name,Age,City\nAlice,30,NYC\nBob,25,LA";
/// let result = parse_csv(csv, &CsvOptions::default()).unwrap();
/// assert_eq!(result.len(), 3);
/// assert_eq!(result[0], vec!["Name", "Age", "City"]);
/// ```ignore
pub fn parse_csv(input: &str, options: &CsvOptions) -> ClmdResult<Vec<Vec<String>>> {
    let mut rows = Vec::new();
    let mut current_row = Vec::new();
    let mut current_field = String::new();
    let mut in_quotes = false;
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            _ if Some(c) == options.quote && !in_quotes => {
                // Start of quoted field
                in_quotes = true;
            }
            _ if Some(c) == options.quote && in_quotes => {
                // Check for escaped quote (double quote)
                if chars.peek() == Some(&c) {
                    chars.next(); // Consume second quote
                    current_field.push(c);
                } else {
                    in_quotes = false;
                }
            }
            _ if Some(c) == options.escape && in_quotes => {
                // Escape character handling
                if let Some(next) = chars.next() {
                    current_field.push(next);
                }
            }
            c if c == options.delimiter && !in_quotes => {
                // End of field
                current_row.push(current_field);
                current_field = String::new();

                // Handle whitespace after delimiter
                if !options.keep_space {
                    while chars.peek() == Some(&' ') || chars.peek() == Some(&'\t') {
                        chars.next();
                    }
                }
            }
            '\r' if !in_quotes => {
                // Handle CRLF
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                current_row.push(current_field);
                if !current_row.is_empty() {
                    rows.push(current_row);
                }
                current_row = Vec::new();
                current_field = String::new();
            }
            '\n' if !in_quotes => {
                // End of line
                current_row.push(current_field);
                if !current_row.is_empty() {
                    rows.push(current_row);
                }
                current_row = Vec::new();
                current_field = String::new();
            }
            _ => {
                current_field.push(c);
            }
        }
    }

    // Handle last field and row
    if !current_field.is_empty() || !current_row.is_empty() {
        current_row.push(current_field);
    }
    if !current_row.is_empty() {
        rows.push(current_row);
    }

    // Validate that all rows have the same number of columns
    if let Some(first_row) = rows.first() {
        let expected_cols = first_row.len();
        for (i, row) in rows.iter().enumerate() {
            if row.len() != expected_cols {
                return Err(ClmdError::parse_error(
                    Position::new(i + 1, 1),
                    format!(
                        "CSV row {} has {} columns, expected {}",
                        i + 1,
                        row.len(),
                        expected_cols
                    ),
                ));
            }
        }
    }

    Ok(rows)
}

/// Parse CSV text with default options.
///
/// # Arguments
///
/// * `input` - The CSV text to parse
///
/// # Examples
///
/// ```ignore
/// use clmd::formats::csv::parse_csv_default;
///
/// let csv = "a,b,c\n1,2,3";
/// let result = parse_csv_default(csv).unwrap();
/// assert_eq!(result[0], vec!["a", "b", "c"]);
/// ```ignore
pub fn parse_csv_default(input: &str) -> ClmdResult<Vec<Vec<String>>> {
    parse_csv(input, &CsvOptions::default())
}

/// Parse TSV (tab-separated values) text.
///
/// # Arguments
///
/// * `input` - The TSV text to parse
///
/// # Examples
///
/// ```ignore
/// use clmd::formats::csv::parse_tsv;
///
/// let tsv = "a\tb\tc\n1\t2\t3";
/// let result = parse_tsv(tsv).unwrap();
/// assert_eq!(result[0], vec!["a", "b", "c"]);
/// ```ignore
pub fn parse_tsv(input: &str) -> ClmdResult<Vec<Vec<String>>> {
    let options = CsvOptions::new().delimiter('\t');
    parse_csv(input, &options)
}

/// Convert parsed CSV data to Markdown table format.
///
/// # Arguments
///
/// * `data` - The parsed CSV data
/// * `has_header` - Whether the first row should be treated as a header
///
/// # Returns
///
/// Returns a Markdown table string.
///
/// # Examples
///
/// ```ignore
/// use clmd::formats::csv::{parse_csv_default, csv_to_markdown};
///
/// let csv = "Name,Age\nAlice,30\nBob,25";
/// let data = parse_csv_default(csv).unwrap();
/// let markdown = csv_to_markdown(&data, true);
/// assert!(markdown.contains("| Name "));
/// assert!(markdown.contains("| Age |"));
/// ```ignore
pub fn csv_to_markdown(data: &[Vec<String>], has_header: bool) -> String {
    if data.is_empty() {
        return String::new();
    }

    let mut result = String::new();
    let col_count = data[0].len();

    // Calculate column widths for alignment
    let mut col_widths = vec![0; col_count];
    for row in data {
        for (i, cell) in row.iter().enumerate() {
            if i < col_count {
                col_widths[i] = col_widths[i].max(cell.len());
            }
        }
    }

    // Helper function to format a cell
    let format_cell = |cell: &str, width: usize| -> String {
        format!("| {:<width$} ", cell, width = width)
    };

    // Write header or first row
    for (i, cell) in data[0].iter().enumerate() {
        if i < col_count {
            result.push_str(&format_cell(cell, col_widths[i]));
        }
    }
    result.push_str("|\n");

    // Write separator line if header
    if has_header {
        for width in &col_widths {
            result.push_str(&format!("|{:-<width$}", "-", width = width + 2));
        }
        result.push_str("|\n");
    }

    // Write data rows
    let start_idx = if has_header { 1 } else { 0 };
    for row in &data[start_idx..] {
        for (i, cell) in row.iter().enumerate() {
            if i < col_count {
                result.push_str(&format_cell(cell, col_widths[i]));
            }
        }
        result.push_str("|\n");
    }

    result
}

/// Parse CSV and convert directly to Markdown table.
///
/// # Arguments
///
/// * `input` - The CSV text to parse
/// * `options` - CSV parsing options
/// * `has_header` - Whether the first row should be treated as a header
///
/// # Examples
///
/// ```ignore
/// use clmd::formats::csv::{parse_csv_to_markdown, CsvOptions};
///
/// let csv = "Name,Age\nAlice,30\nBob,25";
/// let markdown = parse_csv_to_markdown(csv, &CsvOptions::default(), true).unwrap();
/// assert!(markdown.contains("| Name "));
/// assert!(markdown.contains("| Age |"));
/// ```ignore
pub fn parse_csv_to_markdown(
    input: &str,
    options: &CsvOptions,
    has_header: bool,
) -> ClmdResult<String> {
    let data = parse_csv(input, options)?;
    Ok(csv_to_markdown(&data, has_header))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_csv() {
        let csv = "a,b,c\n1,2,3\n4,5,6";
        let result = parse_csv_default(csv).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], vec!["a", "b", "c"]);
        assert_eq!(result[1], vec!["1", "2", "3"]);
        assert_eq!(result[2], vec!["4", "5", "6"]);
    }

    #[test]
    fn test_parse_quoted_fields() {
        let csv = r#""hello, world","foo, bar""#;
        let result = parse_csv_default(csv).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], vec!["hello, world", "foo, bar"]);
    }

    #[test]
    fn test_parse_escaped_quotes() {
        let csv = r#""say ""hello""","test""#;
        let result = parse_csv_default(csv).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], vec!["say \"hello\"", "test"]);
    }

    #[test]
    fn test_parse_tsv() {
        let tsv = "a\tb\tc\n1\t2\t3";
        let result = parse_tsv(tsv).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["a", "b", "c"]);
        assert_eq!(result[1], vec!["1", "2", "3"]);
    }

    #[test]
    fn test_parse_with_custom_delimiter() {
        let input = "a;b;c\n1;2;3";
        let options = CsvOptions::new().delimiter(';');
        let result = parse_csv(input, &options).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["a", "b", "c"]);
    }

    #[test]
    fn test_csv_to_markdown() {
        let data = vec![
            vec!["Name".to_string(), "Age".to_string()],
            vec!["Alice".to_string(), "30".to_string()],
            vec!["Bob".to_string(), "25".to_string()],
        ];
        let markdown = csv_to_markdown(&data, true);
        eprintln!("Generated markdown:\n{}", markdown);
        assert!(markdown.contains("| Name "), "Missing header row");
        assert!(markdown.contains("| Age |"), "Missing Age header");
        assert!(markdown.contains("|---"), "Missing separator line");
        assert!(markdown.contains("| Alice "), "Missing Alice row");
        assert!(markdown.contains("| Bob "), "Missing Bob row");
    }

    #[test]
    fn test_empty_csv() {
        let csv = "";
        let result = parse_csv_default(csv).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_crlf_line_endings() {
        let csv = "a,b\r\n1,2\r\n3,4";
        let result = parse_csv_default(csv).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], vec!["a", "b"]);
        assert_eq!(result[1], vec!["1", "2"]);
    }
}
