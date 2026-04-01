//! BibTeX document reader.
//!
//! This module provides a reader for BibTeX format, used for bibliography management.
//!
//! # Example
//!
//! ```ignore
//! use clmd::readers::BibTeXReader;
//! use clmd::options::ReaderOptions;
//!
//! let reader = BibTeXReader;
//! let input = r#"@article{key, author = {Author}, title = {Title}}"#;
//! let (arena, root) = reader.read(input, &ReaderOptions::default()).unwrap();
//! ```

use crate::core::arena::{Node, NodeArena, NodeId, TreeOps};
use crate::core::error::ClmdResult;
use crate::core::nodes::NodeValue;
use crate::options::{InputFormat, ReaderOptions};
use crate::io::reader::Reader;

/// BibTeX document reader.
#[derive(Debug, Clone, Copy)]
pub struct BibTeXReader;

impl BibTeXReader {
    /// Create a new BibTeX reader.
    pub fn new() -> Self {
        Self
    }
}

impl Default for BibTeXReader {
    fn default() -> Self {
        Self::new()
    }
}

impl Reader for BibTeXReader {
    fn read(
        &self,
        input: &str,
        _options: &ReaderOptions,
    ) -> ClmdResult<(NodeArena, NodeId)> {
        let mut arena = NodeArena::new();
        let root = parse_bibtex(input, &mut arena)?;
        Ok((arena, root))
    }

    fn format(&self) -> &'static str {
        "bibtex"
    }

    fn extensions(&self) -> &[&'static str] {
        &["bib", "bibtex"]
    }

    fn input_format(&self) -> InputFormat {
        InputFormat::Bibtex
    }
}

/// Parse BibTeX content into an AST.
fn parse_bibtex(input: &str, arena: &mut NodeArena) -> ClmdResult<NodeId> {
    let root = arena.alloc(Node::with_value(NodeValue::Document));
    let mut chars = input.chars().peekable();

    while chars.peek().is_some() {
        // Skip whitespace and comments
        skip_whitespace_and_comments(&mut chars);

        if chars.peek().is_none() {
            break;
        }

        // Parse entry
        if let Some(entry) = parse_entry(&mut chars, arena)? {
            TreeOps::append_child(arena, root, entry);
        }
    }

    Ok(root)
}

/// Skip whitespace and comments.
fn skip_whitespace_and_comments(chars: &mut std::iter::Peekable<std::str::Chars>) {
    loop {
        // Skip whitespace
        while let Some(&c) = chars.peek() {
            if c.is_whitespace() {
                chars.next();
            } else {
                break;
            }
        }

        // Skip comments (lines starting with %)
        if let Some(&'%') = chars.peek() {
            while let Some(c) = chars.next() {
                if c == '\n' {
                    break;
                }
            }
        } else {
            break;
        }
    }
}

/// Parse a BibTeX entry.
fn parse_entry(
    chars: &mut std::iter::Peekable<std::str::Chars>,
    arena: &mut NodeArena,
) -> ClmdResult<Option<NodeId>> {
    // Expect @
    if chars.peek() != Some(&'@') {
        // Skip unknown content
        while let Some(c) = chars.next() {
            if c == '@' {
                break;
            }
        }
        return Ok(None);
    }
    chars.next(); // consume @

    // Parse entry type
    let entry_type = parse_identifier(chars);
    if entry_type.is_empty() {
        return Ok(None);
    }

    skip_whitespace_and_comments(chars);

    // Expect {
    if chars.peek() != Some(&'{') {
        return Ok(None);
    }
    chars.next(); // consume {

    // Parse citation key
    let cite_key = parse_identifier(chars);

    skip_whitespace_and_comments(chars);

    // Create entry node
    let entry = Node::with_value(NodeValue::make_text(format!(
        "@{}{{{},",
        entry_type, cite_key
    )));
    let entry_id = arena.alloc(entry);

    // Parse fields
    let mut first = true;
    while chars.peek().is_some() {
        skip_whitespace_and_comments(chars);

        if chars.peek() == Some(&'}') {
            chars.next(); // consume }
            break;
        }

        if !first {
            // Expect comma
            if chars.peek() == Some(&',') {
                chars.next();
                skip_whitespace_and_comments(chars);
            }
        }
        first = false;

        // Check for closing brace again after comma
        if chars.peek() == Some(&'}') {
            chars.next();
            break;
        }

        // Parse field
        if let Some(field) = parse_field(chars, arena)? {
            TreeOps::append_child(arena, entry_id, field);
        }
    }

    Ok(Some(entry_id))
}

/// Parse an identifier.
fn parse_identifier(chars: &mut std::iter::Peekable<std::str::Chars>) -> String {
    let mut ident = String::new();

    while let Some(&c) = chars.peek() {
        if c.is_alphanumeric() || c == '_' || c == '-' || c == ':' {
            ident.push(c);
            chars.next();
        } else {
            break;
        }
    }

    ident
}

/// Parse a field.
fn parse_field(
    chars: &mut std::iter::Peekable<std::str::Chars>,
    arena: &mut NodeArena,
) -> ClmdResult<Option<NodeId>> {
    skip_whitespace_and_comments(chars);

    // Parse field name
    let field_name = parse_identifier(chars);
    if field_name.is_empty() {
        return Ok(None);
    }

    skip_whitespace_and_comments(chars);

    // Expect =
    if chars.peek() != Some(&'=') {
        return Ok(None);
    }
    chars.next(); // consume =

    skip_whitespace_and_comments(chars);

    // Parse value
    let value = parse_value(chars)?;

    // Create field node
    let field = Node::with_value(NodeValue::make_text(format!(
        "    {} = {{{}}},",
        field_name, value
    )));
    let field_id = arena.alloc(field);

    Ok(Some(field_id))
}

/// Parse a field value.
fn parse_value(chars: &mut std::iter::Peekable<std::str::Chars>) -> ClmdResult<String> {
    let mut value = String::new();

    skip_whitespace_and_comments(chars);

    // Handle brace-delimited value
    if chars.peek() == Some(&'{') {
        chars.next(); // consume {
        let mut depth = 1;

        while let Some(c) = chars.next() {
            match c {
                '{' => {
                    depth += 1;
                    value.push(c);
                }
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                    value.push(c);
                }
                _ => value.push(c),
            }
        }
    }
    // Handle quoted value
    else if chars.peek() == Some(&'"') {
        chars.next(); // consume "

        while let Some(c) = chars.next() {
            match c {
                '"' => break,
                '\\' => {
                    // Escape sequence
                    if let Some(next) = chars.next() {
                        value.push(next);
                    }
                }
                _ => value.push(c),
            }
        }
    }
    // Handle numeric value
    else {
        while let Some(&c) = chars.peek() {
            if c.is_numeric() {
                value.push(c);
                chars.next();
            } else {
                break;
            }
        }
    }

    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bibtex_reader_basic() {
        let reader = BibTeXReader::new();
        let options = ReaderOptions::default();

        let input = r#"@article{key, author = {Author}, title = {Title}}"#;
        let (arena, root) = reader.read(input, &options).unwrap();

        let node = arena.get(root);
        assert!(matches!(node.value, NodeValue::Document));
    }

    #[test]
    fn test_bibtex_reader_multiple_entries() {
        let reader = BibTeXReader::new();
        let options = ReaderOptions::default();

        let input = r#"@article{key1, author = {Author1}, title = {Title1}}
@book{key2, author = {Author2}, title = {Title2}}"#;
        let (arena, root) = reader.read(input, &options).unwrap();

        let doc = arena.get(root);
        assert!(doc.first_child.is_some());
    }

    #[test]
    fn test_bibtex_reader_format() {
        let reader = BibTeXReader::new();
        assert_eq!(reader.format(), "bibtex");
        assert!(reader.extensions().contains(&"bib"));
        assert!(reader.extensions().contains(&"bibtex"));
    }

    #[test]
    fn test_parse_identifier() {
        let mut chars = "hello_world ".chars().peekable();
        assert_eq!(parse_identifier(&mut chars), "hello_world");

        let mut chars = "123abc".chars().peekable();
        assert_eq!(parse_identifier(&mut chars), "123abc");
    }
}
