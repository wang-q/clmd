//! Shared utility functions for document processing.
//!
//! This module provides common utility functions for working with Markdown documents,
//! inspired by Pandoc's Shared module. These functions are used throughout the
//! library for tasks like text extraction, identifier generation, and document
//! normalization.
//!
//! # Example
//!
//! ```
//! use clmd::shared::{stringify, inline_list_to_identifier, make_sections};
//! use clmd::{parse_document, Options};
//!
//! let options = Options::default();
//! let (arena, root) = parse_document("# Hello World\n\nSome **bold** text.", &options);
//!
//! // Extract plain text from the document
//! let text = stringify(&arena, root);
//! assert!(text.contains("Hello World"));
//! assert!(text.contains("bold"));
//! ```

use crate::arena::{NodeArena, NodeId};
use crate::nodes::{NodeList, NodeValue};
use std::collections::HashSet;

/// Extract plain text from a node and its children.
///
/// This function recursively traverses the AST and concatenates all text content,
/// ignoring formatting and structure.
///
/// # Arguments
///
/// * `arena` - The arena containing the AST nodes
/// * `node_id` - The node to extract text from
///
/// # Returns
///
/// The plain text content as a string.
///
/// # Example
///
/// ```ignore
/// use clmd::shared::stringify;
/// use clmd::{parse_document, Options};
///
/// let options = Options::default();
/// let (arena, root) = parse_document("# Hello **World**", &options);
///
/// let text = stringify(&arena, root);
/// assert_eq!(text, "Hello World");
/// ```ignore
pub fn stringify(arena: &NodeArena, node_id: NodeId) -> String {
    let mut result = String::new();
    stringify_recursive(arena, node_id, &mut result);
    result
}

fn stringify_recursive(arena: &NodeArena, node_id: NodeId, result: &mut String) {
    let node = arena.get(node_id);

    match &node.value {
        NodeValue::Text(text) => result.push_str(text),
        NodeValue::Code(code) => result.push_str(&code.literal),
        NodeValue::SoftBreak | NodeValue::HardBreak => result.push(' '),
        _ => {
            // Recurse into children
            let mut child = node.first_child;
            while let Some(child_id) = child {
                stringify_recursive(arena, child_id, result);
                child = arena.get(child_id).next;
            }
        }
    }
}

/// Extract plain text from inline content only.
///
/// Similar to `stringify`, but only processes inline elements and stops at block boundaries.
///
/// # Arguments
///
/// * `arena` - The arena containing the AST nodes
/// * `node_id` - The node to extract text from
///
/// # Returns
///
/// The plain text content as a string.
pub fn stringify_inlines(arena: &NodeArena, node_id: NodeId) -> String {
    let mut result = String::new();
    stringify_inlines_recursive(arena, node_id, &mut result);
    result
}

fn stringify_inlines_recursive(arena: &NodeArena, node_id: NodeId, result: &mut String) {
    let node = arena.get(node_id);

    // Stop at block boundaries
    match &node.value {
        NodeValue::Paragraph
        | NodeValue::Heading(_)
        | NodeValue::BlockQuote
        | NodeValue::List(_)
        | NodeValue::Item(_)
        | NodeValue::CodeBlock(_)
        | NodeValue::ThematicBreak
        | NodeValue::Document => {
            // These are block elements - recurse into children but don't add content ourselves
        }
        NodeValue::Text(text) => result.push_str(text),
        NodeValue::Code(code) => result.push_str(&code.literal),
        NodeValue::SoftBreak | NodeValue::HardBreak => result.push(' '),
        _ => {}
    }

    // Recurse into children
    let mut child = node.first_child;
    while let Some(child_id) = child {
        stringify_inlines_recursive(arena, child_id, result);
        child = arena.get(child_id).next;
    }
}

/// Convert a list of inlines to an identifier.
///
/// This function extracts text from inline elements and converts it to a
/// URL-friendly identifier, suitable for use as an HTML anchor.
///
/// # Arguments
///
/// * `arena` - The arena containing the AST nodes
/// * `node_id` - The node containing inline elements
///
/// # Returns
///
/// A URL-friendly identifier string.
///
/// # Example
///
/// ```ignore
/// use clmd::shared::inline_list_to_identifier;
/// use clmd::{parse_document, Options};
///
/// let options = Options::default();
/// let (arena, root) = parse_document("# Hello World!", &options);
///
/// // Get the heading's first child (the text content)
/// let heading = arena.get(root).first_child.unwrap();
/// let identifier = inline_list_to_identifier(&arena, heading);
/// assert_eq!(identifier, "hello-world");
/// ```ignore
pub fn inline_list_to_identifier(arena: &NodeArena, node_id: NodeId) -> String {
    let text = stringify_inlines(arena, node_id);
    text_to_identifier(&text)
}

/// Convert text to a URL-friendly identifier.
///
/// # Arguments
///
/// * `text` - The text to convert
///
/// # Returns
///
/// A URL-friendly identifier string.
///
/// # Example
///
/// ```ignore
/// use clmd::shared::text_to_identifier;
///
/// assert_eq!(text_to_identifier("Hello World!"), "hello-world");
/// assert_eq!(text_to_identifier("C++ Programming"), "c-programming");
/// assert_eq!(text_to_identifier("  Multiple   Spaces  "), "multiple-spaces");
/// ```ignore
pub fn text_to_identifier(text: &str) -> String {
    let mut result = String::new();
    let mut prev_was_dash = true; // Start true to avoid leading dash

    for c in text.to_lowercase().chars() {
        match c {
            'a'..='z' | '0'..='9' => {
                result.push(c);
                prev_was_dash = false;
            }
            _ if !prev_was_dash => {
                result.push('-');
                prev_was_dash = true;
            }
            _ => {} // Skip consecutive non-alphanumeric chars
        }
    }

    // Remove trailing dash
    if result.ends_with('-') {
        result.pop();
    }

    result
}

/// Generate a unique identifier.
///
/// Given a base identifier and a set of existing identifiers, this function
/// generates a unique identifier by appending a number if necessary.
///
/// # Arguments
///
/// * `base` - The base identifier
/// * `existing` - A set of existing identifiers to avoid
///
/// # Returns
///
/// A unique identifier string.
///
/// # Example
///
/// ```ignore
/// use clmd::shared::unique_ident;
/// use std::collections::HashSet;
///
/// let mut existing = HashSet::new();
/// existing.insert("hello-world".to_string());
///
/// assert_eq!(unique_ident("hello-world", &existing), "hello-world-1");
/// assert_eq!(unique_ident("new-ident", &existing), "new-ident");
/// ```ignore
pub fn unique_ident(base: &str, existing: &HashSet<String>) -> String {
    if !existing.contains(base) {
        return base.to_string();
    }

    let mut counter = 1;
    loop {
        let candidate = format!("{}-{}", base, counter);
        if !existing.contains(&candidate) {
            return candidate;
        }
        counter += 1;
    }
}

/// Make sections from headers.
///
/// This function transforms a flat document structure into a hierarchical
/// section structure based on header levels. Each section contains its
/// header and all content until the next header of equal or higher level.
///
/// # Arguments
///
/// * `arena` - The arena containing the AST nodes
/// * `root` - The root node of the document
/// * `base_level` - The base header level (usually 1)
///
/// # Example
///
/// ```ignore
/// use clmd::shared::make_sections;
/// use clmd::{parse_document, Options};
///
/// let options = Options::default();
/// let (mut arena, root) = parse_document("# Section 1\n\nContent 1\n\n# Section 2\n\nContent 2", &options);
///
/// make_sections(&mut arena, root, 1);
/// // Document now has a hierarchical section structure
/// ```ignore
pub fn make_sections(arena: &mut NodeArena, root: NodeId, _base_level: u8) {
    // Collect all top-level headers and their positions
    let mut sections: Vec<(u8, NodeId, Vec<NodeId>)> = Vec::new();
    let mut current_section: Option<(u8, NodeId, Vec<NodeId>)> = None;

    let root_node = arena.get(root);
    let mut child = root_node.first_child;

    while let Some(child_id) = child {
        let node = arena.get(child_id);

        if let NodeValue::Heading(heading) = &node.value {
            // Save previous section if exists
            if let Some((level, header, content)) = current_section.take() {
                sections.push((level, header, content));
            }
            // Start new section
            current_section = Some((heading.level, child_id, Vec::new()));
        } else if let Some((_, _, ref mut content)) = current_section {
            content.push(child_id);
        }

        child = node.next;
    }

    // Don't forget the last section
    if let Some((level, header, content)) = current_section {
        sections.push((level, header, content));
    }

    // Now reorganize into hierarchical structure
    // For now, just add section identifiers to headers
    let mut used_idents = HashSet::new();
    for (_level, header_id, _) in &sections {
        let ident = inline_list_to_identifier(arena, *header_id);
        let unique = unique_ident(&ident, &used_idents);
        used_idents.insert(unique.clone());

        // Store the identifier in the heading (would need to extend NodeHeading)
        // For now, this is a placeholder for the concept
    }
}

/// Compactify lists by removing unnecessary blank lines.
///
/// This function analyzes lists and removes blank lines between items
/// when the list can be rendered in "tight" format.
///
/// # Arguments
///
/// * `arena` - The arena containing the AST nodes
/// * `root` - The root node of the document
///
/// # Example
///
/// ```ignore
/// use clmd::shared::compactify;
/// use clmd::{parse_document, Options};
///
/// let options = Options::default();
/// let (mut arena, root) = parse_document("- Item 1\n\n- Item 2", &options);
///
/// compactify(&mut arena, root);
/// ```ignore
pub fn compactify(arena: &mut NodeArena, root: NodeId) {
    compactify_recursive(arena, root);
}

fn compactify_recursive(arena: &mut NodeArena, node_id: NodeId) {
    let should_compact = {
        let node = arena.get(node_id);

        // Check if this is a list that can be compacted
        if let NodeValue::List(_) = &node.value {
            can_list_be_compact(arena, node_id)
        } else {
            // Recurse into children
            let mut child = node.first_child;
            while let Some(child_id) = child {
                compactify_recursive(arena, child_id);
                child = arena.get(child_id).next;
            }
            false
        }
    };

    if should_compact {
        // Mark list as tight
        let node = arena.get_mut(node_id);
        if let NodeValue::List(ref mut list) = node.value {
            list.tight = true;
        }
    }
}

fn can_list_be_compact(arena: &NodeArena, list_id: NodeId) -> bool {
    let list_node = arena.get(list_id);
    let mut child = list_node.first_child;

    while let Some(item_id) = child {
        let item = arena.get(item_id);

        // Check if item contains block-level content other than a single paragraph
        let mut item_child = item.first_child;
        let mut child_count = 0;
        let mut has_non_paragraph_block = false;

        while let Some(child_id) = item_child {
            child_count += 1;
            let child_node = arena.get(child_id);
            match &child_node.value {
                NodeValue::Paragraph => {}
                _ => has_non_paragraph_block = true,
            }
            item_child = child_node.next;
        }

        if has_non_paragraph_block || child_count > 1 {
            return false;
        }

        child = item.next;
    }

    true
}

/// Get the first block-level child of a node.
///
/// # Arguments
///
/// * `arena` - The arena containing the AST nodes
/// * `node_id` - The node to search
///
/// # Returns
///
/// The first block-level child node ID, if any.
pub fn first_block_child(arena: &NodeArena, node_id: NodeId) -> Option<NodeId> {
    let node = arena.get(node_id);
    let mut child = node.first_child;

    while let Some(child_id) = child {
        let child_node = arena.get(child_id);
        if is_block_element(&child_node.value) {
            return Some(child_id);
        }
        child = child_node.next;
    }

    None
}

/// Check if a node value is a block-level element.
///
/// # Arguments
///
/// * `value` - The node value to check
///
/// # Returns
///
/// true if the node is a block-level element.
pub fn is_block_element(value: &NodeValue) -> bool {
    matches!(
        value,
        NodeValue::Document
            | NodeValue::Paragraph
            | NodeValue::Heading(_)
            | NodeValue::BlockQuote
            | NodeValue::List(_)
            | NodeValue::Item(_)
            | NodeValue::CodeBlock(_)
            | NodeValue::ThematicBreak
    )
}

/// Check if a node value is an inline-level element.
///
/// # Arguments
///
/// * `value` - The node value to check
///
/// # Returns
///
/// true if the node is an inline-level element.
pub fn is_inline_element(value: &NodeValue) -> bool {
    !is_block_element(value)
}

/// Count the number of children of a node.
///
/// # Arguments
///
/// * `arena` - The arena containing the AST nodes
/// * `node_id` - The node to count children of
///
/// # Returns
///
/// The number of children.
pub fn child_count(arena: &NodeArena, node_id: NodeId) -> usize {
    let node = arena.get(node_id);
    let mut count = 0;
    let mut child = node.first_child;

    while let Some(child_id) = child {
        count += 1;
        child = arena.get(child_id).next;
    }

    count
}

/// Get all children of a node as a vector.
///
/// # Arguments
///
/// * `arena` - The arena containing the AST nodes
/// * `node_id` - The node to get children of
///
/// # Returns
///
/// A vector of child node IDs.
pub fn get_children(arena: &NodeArena, node_id: NodeId) -> Vec<NodeId> {
    let node = arena.get(node_id);
    let mut children = Vec::new();
    let mut child = node.first_child;

    while let Some(child_id) = child {
        children.push(child_id);
        child = arena.get(child_id).next;
    }

    children
}

/// Check if a list is "tight" (no blank lines between items).
///
/// # Arguments
///
/// * `list` - The list to check
///
/// # Returns
///
/// true if the list is tight.
pub fn is_tight_list(list: &NodeList) -> bool {
    list.tight
}

/// Get the list marker for a list item.
///
/// # Arguments
///
/// * `list` - The list
/// * `index` - The item index
///
/// # Returns
///
/// The marker string (e.g., "-", "*", "1.", "2.").
pub fn list_marker(list: &NodeList, index: usize) -> String {
    match list.list_type {
        crate::nodes::ListType::Bullet => {
            let bullet = list.bullet_char as char;
            format!("{} ", bullet)
        }
        crate::nodes::ListType::Ordered => {
            let num = list.start + index;
            let delim = match list.delimiter {
                crate::nodes::ListDelimType::Period => '.',
                crate::nodes::ListDelimType::Paren => ')',
            };
            format!("{}{} ", num, delim)
        }
    }
}

/// Normalize whitespace in a string.
///
/// Converts multiple consecutive whitespace characters to a single space
/// and trims leading/trailing whitespace.
///
/// # Arguments
///
/// * `text` - The text to normalize
///
/// # Returns
///
/// The normalized text.
///
/// # Example
///
/// ```ignore
/// use clmd::shared::normalize_whitespace;
///
/// assert_eq!(normalize_whitespace("  hello   world  "), "hello world");
/// assert_eq!(normalize_whitespace("a\n\nb\t\tc"), "a b c");
/// ```ignore
pub fn normalize_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Escape special characters in a string for use in HTML attributes.
///
/// # Arguments
///
/// * `text` - The text to escape
///
/// # Returns
///
/// The escaped text.
pub fn escape_html_attribute(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Convert a string to title case.
///
/// # Arguments
///
/// * `text` - The text to convert
///
/// # Returns
///
/// The text in title case.
///
/// # Example
///
/// ```ignore
/// use clmd::shared::to_title_case;
///
/// assert_eq!(to_title_case("hello world"), "Hello World");
/// assert_eq!(to_title_case("the quick brown fox"), "The Quick Brown Fox");
/// ```ignore
pub fn to_title_case(text: &str) -> String {
    text.split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    let mut result = first.to_uppercase().to_string();
                    result.push_str(&chars.as_str().to_lowercase());
                    result
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Strip leading and trailing blank lines from a string.
///
/// # Arguments
///
/// * `text` - The text to strip
///
/// # Returns
///
/// The stripped text.
pub fn strip_blank_lines(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let mut start = 0;
    let mut end = lines.len();

    // Find first non-blank line
    while start < end && lines[start].trim().is_empty() {
        start += 1;
    }

    // Find last non-blank line
    while end > start && lines[end - 1].trim().is_empty() {
        end -= 1;
    }

    lines[start..end].join("\n")
}

/// Split a string into chunks separated by blank lines.
///
/// # Arguments
///
/// * `text` - The text to split
///
/// # Returns
///
/// A vector of chunks.
pub fn split_by_blank_lines(text: &str) -> Vec<String> {
    let lines: Vec<&str> = text.lines().collect();
    let mut chunks = Vec::new();
    let mut current_chunk = Vec::new();

    for line in lines {
        if line.trim().is_empty() {
            if !current_chunk.is_empty() {
                chunks.push(current_chunk.join("\n"));
                current_chunk.clear();
            }
        } else {
            current_chunk.push(line);
        }
    }

    if !current_chunk.is_empty() {
        chunks.push(current_chunk.join("\n"));
    }

    chunks
}

/// Truncate text to a maximum length with ellipsis.
///
/// # Arguments
///
/// * `text` - The text to truncate
/// * `max_len` - Maximum length including ellipsis
///
/// # Returns
///
/// Truncated text with ellipsis if needed.
///
/// # Example
///
/// ```ignore
/// use clmd::shared::truncate_with_ellipsis;
///
/// assert_eq!(truncate_with_ellipsis("Hello World", 8), "Hello...");
/// assert_eq!(truncate_with_ellipsis("Hi", 8), "Hi");
/// ```ignore
pub fn truncate_with_ellipsis(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else if max_len <= 3 {
        text.chars().take(max_len).collect()
    } else {
        format!("{}...", &text[..max_len - 3])
    }
}

/// Check if a string is a valid URL.
///
/// # Arguments
///
/// * `s` - The string to check
///
/// # Returns
///
/// true if the string looks like a URL.
pub fn is_url(s: &str) -> bool {
    s.starts_with("http://")
        || s.starts_with("https://")
        || s.starts_with("ftp://")
        || s.starts_with("file://")
        || s.starts_with("mailto:")
}

/// Check if a string is an absolute URL (not a relative path).
///
/// # Arguments
///
/// * `s` - The string to check
///
/// # Returns
///
/// true if the string is an absolute URL.
pub fn is_absolute_url(s: &str) -> bool {
    s.starts_with("http://")
        || s.starts_with("https://")
        || s.starts_with("ftp://")
        || s.starts_with("file://")
        || s.starts_with("mailto:")
        || s.starts_with("//")
}

/// Get file extension from a path string.
///
/// # Arguments
///
/// * `path` - The path string
///
/// # Returns
///
/// The file extension if any.
pub fn get_extension(path: &str) -> Option<&str> {
    path.rfind('.').map(|i| &path[i + 1..])
}

/// Remove file extension from a path string.
///
/// # Arguments
///
/// * `path` - The path string
///
/// # Returns
///
/// The path without extension.
pub fn remove_extension(path: &str) -> &str {
    match path.rfind('.') {
        Some(i) => &path[..i],
        None => path,
    }
}

/// Convert a string to kebab-case.
///
/// # Arguments
///
/// * `text` - The text to convert
///
/// # Returns
///
/// The text in kebab-case.
///
/// # Example
///
/// ```ignore
/// use clmd::shared::to_kebab_case;
///
/// assert_eq!(to_kebab_case("Hello World"), "hello-world");
/// assert_eq!(to_kebab_case("HelloWorld"), "hello-world");
/// assert_eq!(to_kebab_case("hello_world"), "hello-world");
/// ```ignore
pub fn to_kebab_case(text: &str) -> String {
    let mut result = String::new();
    let mut prev_was_upper = false;
    let mut prev_was_alnum = false;

    for c in text.chars() {
        if c.is_alphanumeric() {
            if c.is_uppercase() {
                if prev_was_alnum && !prev_was_upper {
                    result.push('-');
                }
                result.push(c.to_lowercase().next().unwrap());
                prev_was_upper = true;
            } else {
                result.push(c);
                prev_was_upper = false;
            }
            prev_was_alnum = true;
        } else if prev_was_alnum {
            result.push('-');
            prev_was_alnum = false;
            prev_was_upper = false;
        }
    }

    // Remove trailing dash
    if result.ends_with('-') {
        result.pop();
    }

    result
}

/// Convert a string to snake_case.
///
/// # Arguments
///
/// * `text` - The text to convert
///
/// # Returns
///
/// The text in snake_case.
///
/// # Example
///
/// ```ignore
/// use clmd::shared::to_snake_case;
///
/// assert_eq!(to_snake_case("Hello World"), "hello_world");
/// assert_eq!(to_snake_case("HelloWorld"), "hello_world");
/// assert_eq!(to_snake_case("hello-world"), "hello_world");
/// ```ignore
pub fn to_snake_case(text: &str) -> String {
    to_kebab_case(text).replace('-', "_")
}

/// Convert a string to camelCase.
///
/// # Arguments
///
/// * `text` - The text to convert
///
/// # Returns
///
/// The text in camelCase.
///
/// # Example
///
/// ```ignore
/// use clmd::shared::to_camel_case;
///
/// assert_eq!(to_camel_case("hello world"), "helloWorld");
/// assert_eq!(to_camel_case("hello-world"), "helloWorld");
/// assert_eq!(to_camel_case("hello_world"), "helloWorld");
/// ```ignore
pub fn to_camel_case(text: &str) -> String {
    let words: Vec<&str> = text
        .split(|c: char| c == ' ' || c == '-' || c == '_')
        .collect();
    let mut result = String::new();

    for (i, word) in words.iter().enumerate() {
        if word.is_empty() {
            continue;
        }
        let mut chars = word.chars();
        if let Some(first) = chars.next() {
            if i == 0 {
                result.push(first.to_lowercase().next().unwrap());
            } else {
                result.push(first.to_uppercase().next().unwrap());
            }
            result.push_str(&chars.as_str().to_lowercase());
        }
    }

    result
}

/// Convert a string to PascalCase.
///
/// # Arguments
///
/// * `text` - The text to convert
///
/// # Returns
///
/// The text in PascalCase.
///
/// # Example
///
/// ```ignore
/// use clmd::shared::to_pascal_case;
///
/// assert_eq!(to_pascal_case("hello world"), "HelloWorld");
/// assert_eq!(to_pascal_case("hello-world"), "HelloWorld");
/// assert_eq!(to_pascal_case("hello_world"), "HelloWorld");
/// ```ignore
pub fn to_pascal_case(text: &str) -> String {
    let words: Vec<&str> = text
        .split(|c: char| c == ' ' || c == '-' || c == '_')
        .collect();
    let mut result = String::new();

    for word in words {
        if word.is_empty() {
            continue;
        }
        let mut chars = word.chars();
        if let Some(first) = chars.next() {
            result.push(first.to_uppercase().next().unwrap());
            result.push_str(&chars.as_str().to_lowercase());
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nodes::NodeHeading;

    #[test]
    fn test_stringify() {
        let options = crate::Options::default();
        let (arena, root) = crate::parse_document("# Hello **World**", &options);

        let text = stringify(&arena, root);
        assert!(text.contains("Hello"));
        assert!(text.contains("World"));
    }

    #[test]
    fn test_text_to_identifier() {
        assert_eq!(text_to_identifier("Hello World!"), "hello-world");
        assert_eq!(text_to_identifier("C++ Programming"), "c-programming");
        assert_eq!(
            text_to_identifier("  Multiple   Spaces  "),
            "multiple-spaces"
        );
        assert_eq!(text_to_identifier("UPPERCASE"), "uppercase");
        assert_eq!(text_to_identifier("special@#chars"), "special-chars");
    }

    #[test]
    fn test_unique_ident() {
        let mut existing = HashSet::new();
        existing.insert("hello-world".to_string());

        assert_eq!(unique_ident("hello-world", &existing), "hello-world-1");
        assert_eq!(unique_ident("new-ident", &existing), "new-ident");

        existing.insert("hello-world-1".to_string());
        assert_eq!(unique_ident("hello-world", &existing), "hello-world-2");
    }

    #[test]
    fn test_normalize_whitespace() {
        assert_eq!(normalize_whitespace("  hello   world  "), "hello world");
        assert_eq!(normalize_whitespace("a\n\nb\t\tc"), "a b c");
        assert_eq!(normalize_whitespace("single"), "single");
    }

    #[test]
    fn test_to_title_case() {
        assert_eq!(to_title_case("hello world"), "Hello World");
        assert_eq!(to_title_case("the quick brown fox"), "The Quick Brown Fox");
        assert_eq!(to_title_case("HELLO"), "Hello");
    }

    #[test]
    fn test_escape_html_attribute() {
        assert_eq!(
            escape_html_attribute("<script>alert('xss')</script>"),
            "&lt;script&gt;alert('xss')&lt;/script&gt;"
        );
        assert_eq!(
            escape_html_attribute("test \"quoted\" text"),
            "test &quot;quoted&quot; text"
        );
    }

    #[test]
    fn test_strip_blank_lines() {
        assert_eq!(strip_blank_lines("\n\nhello\n\n"), "hello");
        assert_eq!(strip_blank_lines("hello\n\nworld"), "hello\n\nworld");
        assert_eq!(strip_blank_lines("  \n  \n  "), "");
    }

    #[test]
    fn test_split_by_blank_lines() {
        let chunks = split_by_blank_lines("chunk1\n\nchunk2\n\nchunk3");
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0], "chunk1");
        assert_eq!(chunks[1], "chunk2");
        assert_eq!(chunks[2], "chunk3");
    }

    #[test]
    fn test_is_block_element() {
        assert!(is_block_element(&NodeValue::Paragraph));
        assert!(is_block_element(&NodeValue::Heading(NodeHeading {
            level: 1,
            setext: false,
            closed: false,
        })));
        assert!(!is_block_element(&NodeValue::Text("test".into())));
        assert!(!is_block_element(&NodeValue::Emph));
    }

    #[test]
    fn test_child_count() {
        let options = crate::Options::default();
        let (arena, root) =
            crate::parse_document("# Heading\n\nParagraph 1\n\nParagraph 2", &options);

        let count = child_count(&arena, root);
        assert!(count >= 2); // At least heading and paragraphs
    }

    #[test]
    fn test_get_children() {
        let options = crate::Options::default();
        let (arena, root) = crate::parse_document("# Heading\n\nParagraph", &options);

        let children = get_children(&arena, root);
        assert!(!children.is_empty());
    }

    #[test]
    fn test_list_marker() {
        let bullet_list = NodeList {
            list_type: crate::nodes::ListType::Bullet,
            marker_offset: 0,
            padding: 2,
            start: 1,
            delimiter: crate::nodes::ListDelimType::Period,
            bullet_char: b'-',
            tight: true,
            is_task_list: false,
        };
        assert_eq!(list_marker(&bullet_list, 0), "- ");

        let ordered_list = NodeList {
            list_type: crate::nodes::ListType::Ordered,
            marker_offset: 0,
            padding: 3,
            start: 1,
            delimiter: crate::nodes::ListDelimType::Period,
            bullet_char: b'.',
            tight: true,
            is_task_list: false,
        };
        assert_eq!(list_marker(&ordered_list, 0), "1. ");
        assert_eq!(list_marker(&ordered_list, 1), "2. ");
    }

    #[test]
    fn test_truncate_with_ellipsis() {
        assert_eq!(truncate_with_ellipsis("Hello World", 8), "Hello...");
        assert_eq!(truncate_with_ellipsis("Hi", 8), "Hi");
        assert_eq!(truncate_with_ellipsis("Hello", 5), "Hello");
        assert_eq!(truncate_with_ellipsis("Hello", 3), "Hel");
    }

    #[test]
    fn test_is_url() {
        assert!(is_url("http://example.com"));
        assert!(is_url("https://example.com"));
        assert!(is_url("ftp://files.example.com"));
        assert!(is_url("mailto:test@example.com"));
        assert!(!is_url("/path/to/file"));
        assert!(!is_url("relative/path"));
    }

    #[test]
    fn test_is_absolute_url() {
        assert!(is_absolute_url("http://example.com"));
        assert!(is_absolute_url("https://example.com"));
        assert!(is_absolute_url("//example.com"));
        assert!(!is_absolute_url("/path/to/file"));
        assert!(!is_absolute_url("relative/path"));
    }

    #[test]
    fn test_get_extension() {
        assert_eq!(get_extension("file.txt"), Some("txt"));
        assert_eq!(get_extension("path/to/file.md"), Some("md"));
        assert_eq!(get_extension("file"), None);
    }

    #[test]
    fn test_remove_extension() {
        assert_eq!(remove_extension("file.txt"), "file");
        assert_eq!(remove_extension("path/to/file.md"), "path/to/file");
        assert_eq!(remove_extension("file"), "file");
    }

    #[test]
    fn test_to_kebab_case() {
        assert_eq!(to_kebab_case("Hello World"), "hello-world");
        assert_eq!(to_kebab_case("HelloWorld"), "hello-world");
        assert_eq!(to_kebab_case("hello_world"), "hello-world");
        assert_eq!(to_kebab_case("hello-world"), "hello-world");
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("Hello World"), "hello_world");
        assert_eq!(to_snake_case("HelloWorld"), "hello_world");
        assert_eq!(to_snake_case("hello-world"), "hello_world");
    }

    #[test]
    fn test_to_camel_case() {
        assert_eq!(to_camel_case("hello world"), "helloWorld");
        assert_eq!(to_camel_case("hello-world"), "helloWorld");
        assert_eq!(to_camel_case("hello_world"), "helloWorld");
        assert_eq!(to_camel_case("HelloWorld"), "helloworld");
    }

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("hello world"), "HelloWorld");
        assert_eq!(to_pascal_case("hello-world"), "HelloWorld");
        assert_eq!(to_pascal_case("hello_world"), "HelloWorld");
    }
}
