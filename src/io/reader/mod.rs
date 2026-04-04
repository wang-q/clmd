//! Document readers for various input formats.
//!
//! This module provides a unified interface for reading documents from different
//! formats, inspired by Pandoc's Reader system. Readers convert input data into
//! the internal AST representation.
//!
//! # Example
//!
//! ```ignore
//! use clmd::io::reader::{ReaderRegistry, Reader};
//! use clmd::options::{ReaderOptions, InputFormat};
//!
//! let registry = ReaderRegistry::new();
//! let reader = registry.get("markdown").unwrap();
//!
//! let options = ReaderOptions::default();
//! let (arena, root) = reader.read("# Hello World", &options).unwrap();
//! ```

use crate::core::arena::NodeArena;
use crate::core::error::{ClmdError, ClmdResult};
use crate::options::ReaderOptions;

pub mod bibtex;
pub mod html;
pub mod latex;

mod registry;
pub use registry::{
    BibTeXReader, BoxedReader, HtmlReader, LaTeXReader, MarkdownReader, Reader,
    ReaderRegistry,
};

/// Read a document from a string with format detection.
///
/// # Arguments
///
/// * `input` - The input string
/// * `format` - Optional format hint (if None, attempts to detect from content)
/// * `options` - Parsing options
///
/// # Returns
///
/// A tuple of (arena, root_node_id) on success, or an error on failure.
///
/// # Example
///
/// ```ignore
/// use clmd::io::reader::read_document;
/// use clmd::options::ReaderOptions;
///
/// let options = ReaderOptions::default();
/// let (arena, root) = read_document("# Hello", Some("markdown"), &options).unwrap();
/// ```
pub fn read_document(
    input: &str,
    format: Option<&str>,
    options: &ReaderOptions,
) -> ClmdResult<(NodeArena, crate::core::arena::NodeId)> {
    let registry = ReaderRegistry::new();

    let format = format.unwrap_or("markdown");

    let reader = registry
        .get(format)
        .ok_or_else(|| ClmdError::unknown_reader(format))?;

    reader.read(input, options)
}

/// Read a document from a file.
///
/// # Arguments
///
/// * `path` - The file path
/// * `format` - Optional format override (if None, detects from extension)
/// * `options` - Parsing options
///
/// # Returns
///
/// A tuple of (arena, root_node_id) on success, or an error on failure.
///
/// # Example
///
/// ```ignore
/// use clmd::io::reader::read_file;
/// use clmd::options::ReaderOptions;
///
/// let options = ReaderOptions::default();
/// let (arena, root) = read_file("document.md", None, &options).unwrap();
/// ```
pub fn read_file(
    path: &std::path::Path,
    format: Option<&str>,
    options: &ReaderOptions,
) -> ClmdResult<(NodeArena, crate::core::arena::NodeId)> {
    use std::fs;

    let content = fs::read_to_string(path)
        .map_err(|e| ClmdError::io_error(format!("Failed to read file: {}", e)))?;

    // Detect format from extension if not specified
    let format = match format {
        Some(f) => Some(f),
        None => detect_format_from_path(path),
    };

    read_document(&content, format, options)
}

/// Detect format from file path by extension.
fn detect_format_from_path(path: &std::path::Path) -> Option<&'static str> {
    path.extension().and_then(|e| e.to_str()).map(|ext| {
        match ext.to_lowercase().as_str() {
            "md" | "markdown" | "mkd" | "mdown" => "markdown",
            "html" | "htm" => "html",
            "bib" | "bibtex" => "bibtex",
            "tex" | "latex" => "latex",
            _ => "markdown", // default to markdown
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nodes::NodeValue;

    #[test]
    fn test_reader_registry() {
        let registry = ReaderRegistry::new();

        assert!(registry.supports_format("markdown"));
        assert!(registry.supports_format("html"));
        assert!(!registry.supports_format("pdf"));

        assert!(registry.supports_extension("md"));
        assert!(registry.supports_extension("html"));
        assert!(!registry.supports_extension("pdf"));
    }

    #[test]
    fn test_registry_get() {
        let registry = ReaderRegistry::new();

        let reader = registry.get("markdown");
        assert!(reader.is_some());
        assert_eq!(reader.unwrap().format(), "markdown");

        let reader = registry.get("unknown");
        assert!(reader.is_none());
    }

    #[test]
    fn test_registry_get_by_extension() {
        let registry = ReaderRegistry::new();

        let reader = registry.get_by_extension("md");
        assert!(reader.is_some());

        let reader = registry.get_by_extension("unknown");
        assert!(reader.is_none());
    }

    #[test]
    fn test_detect_format() {
        let registry = ReaderRegistry::new();

        let path = std::path::Path::new("test.md");
        assert_eq!(registry.detect_format(path), Some("markdown"));

        let path = std::path::Path::new("test.html");
        assert_eq!(registry.detect_format(path), Some("html"));

        let path = std::path::Path::new("test");
        assert_eq!(registry.detect_format(path), None);
    }

    #[test]
    fn test_read_document() {
        let options = ReaderOptions::default();
        let (arena, root) = read_document("# Test", Some("markdown"), &options).unwrap();

        let node = arena.get(root);
        assert!(matches!(node.value, NodeValue::Document));
    }

    #[test]
    fn test_read_document_unknown_format() {
        let options = ReaderOptions::default();
        let result = read_document("# Test", Some("unknown"), &options);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown reader"));
    }

    #[test]
    fn test_reader_input_format() {
        let registry = ReaderRegistry::new();

        let markdown_reader = registry.get("markdown").unwrap();
        assert_eq!(
            markdown_reader.input_format(),
            crate::options::InputFormat::Markdown
        );

        let html_reader = registry.get("html").unwrap();
        assert_eq!(
            html_reader.input_format(),
            crate::options::InputFormat::Html
        );
    }
}
