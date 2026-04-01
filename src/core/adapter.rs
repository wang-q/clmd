//! AST adapters for converting between different AST representations.
//!
//! This module provides adapters for converting between the Comrak-compatible AST
//! and the Pandoc-style AST defined in the `ast` module.
//!
//! # Example
//!
//! ```ignore
//! use clmd::adapters::{to_pandoc_ast, from_pandoc_ast};
//! use clmd::{parse_document, Options};
//!
//! let options = Options::default();
//! let (arena, root) = parse_document("# Hello World", &options);
//!
//! // Convert to Pandoc-style AST
//! let doc = to_pandoc_ast(&arena, root);
//!
//! // Convert back to Comrak AST
//! let (new_arena, new_root) = from_pandoc_ast(&doc);
//! ```

use crate::core::arena::{NodeArena, NodeId};
use crate::core::ast as pandoc;
use crate::core::nodes::NodeValue;

/// Adapter trait for syntax highlighting.
///
/// This trait allows customization of how code blocks are highlighted
/// during HTML rendering.
pub trait SyntaxHighlighterAdapter {
    /// Write highlighted code to the output.
    ///
    /// # Arguments
    ///
    /// * `output` - The output writer
    /// * `lang` - The language of the code (e.g., "rust", "python")
    /// * `code` - The code to highlight
    ///
    /// # Returns
    ///
    /// The result of the write operation
    fn write_highlighted(
        &self,
        output: &mut dyn std::fmt::Write,
        lang: Option<&str>,
        code: &str,
    ) -> std::fmt::Result;

    /// Write the opening `<pre>` tag.
    ///
    /// # Arguments
    ///
    /// * `output` - The output writer
    /// * `attributes` - The attributes for the tag
    ///
    /// # Returns
    ///
    /// The result of the write operation
    fn write_pre_tag<'s>(
        &self,
        output: &mut dyn std::fmt::Write,
        attributes: std::collections::HashMap<&str, std::borrow::Cow<'s, str>>,
    ) -> std::fmt::Result;

    /// Write the opening `<code>` tag.
    ///
    /// # Arguments
    ///
    /// * `output` - The output writer
    /// * `attributes` - The attributes for the tag
    ///
    /// # Returns
    ///
    /// The result of the write operation
    fn write_code_tag<'s>(
        &self,
        output: &mut dyn std::fmt::Write,
        attributes: std::collections::HashMap<&str, std::borrow::Cow<'s, str>>,
    ) -> std::fmt::Result;
}

/// Adapter trait for code fence rendering.
///
/// This trait allows customization of how code fences (fenced code blocks)
/// are rendered.
pub trait CodefenceRendererAdapter {
    /// Check if this adapter can handle the given code fence info string.
    ///
    /// # Arguments
    ///
    /// * `info` - The info string from the code fence (e.g., "rust", "python")
    ///
    /// # Returns
    ///
    /// `true` if this adapter can handle the code fence
    fn is_codefence(&self, info: &str) -> bool;

    /// Render the code fence.
    ///
    /// # Arguments
    ///
    /// * `info` - The info string from the code fence
    /// * `content` - The content of the code fence
    ///
    /// # Returns
    ///
    /// The rendered HTML string, or `None` to use default rendering
    fn render_codefence(&self, info: &str, content: &str) -> Option<String>;
}

/// Adapter trait for heading rendering.
///
/// This trait allows customization of how headings are rendered.
pub trait HeadingAdapter {
    /// Render a heading.
    ///
    /// # Arguments
    ///
    /// * `level` - The heading level (1-6)
    /// * `content` - The heading content
    /// * `id` - An optional ID for the heading
    ///
    /// # Returns
    ///
    /// The rendered HTML string, or `None` to use default rendering
    fn render_heading(
        &self,
        level: u8,
        content: &str,
        id: Option<&str>,
    ) -> Option<String>;
}

/// Adapter trait for URL rewriting.
///
/// This trait allows customization of how URLs are rewritten during rendering.
pub trait UrlRewriter {
    /// Rewrite a URL.
    ///
    /// # Arguments
    ///
    /// * `url` - The original URL
    ///
    /// # Returns
    ///
    /// The rewritten URL
    fn rewrite_url(&self, url: &str) -> String;
}

/// Convert a Comrak AST to a Pandoc-style AST.
///
/// # Arguments
///
/// * `arena` - The arena containing the Comrak AST nodes
/// * `root` - The root node of the Comrak AST
///
/// # Returns
///
/// A Pandoc-style Document.
pub fn to_pandoc_ast(_arena: &NodeArena, _root: NodeId) -> pandoc::Document {
    // TODO: Implement proper conversion
    pandoc::Document {
        meta: pandoc::MetaData::new(),
        blocks: Vec::new(),
    }
}

/// Convert a Pandoc-style AST back to a Comrak AST.
///
/// # Arguments
///
/// * `doc` - The Pandoc-style Document
///
/// # Returns
///
/// A tuple of (arena, root_node_id) for the Comrak AST.
pub fn from_pandoc_ast(_doc: &pandoc::Document) -> (NodeArena, NodeId) {
    // TODO: Implement proper conversion
    let mut arena = NodeArena::new();
    let root = arena.alloc(crate::core::arena::Node::with_value(NodeValue::Document));
    (arena, root)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_pandoc_ast_basic() {
        // TODO: Add proper test once conversion is implemented
        let mut arena = NodeArena::new();
        let root =
            arena.alloc(crate::core::arena::Node::with_value(NodeValue::Document));
        let doc = to_pandoc_ast(&arena, root);
        assert!(doc.blocks.is_empty());
    }

    #[test]
    fn test_from_pandoc_ast() {
        // TODO: Add proper test once conversion is implemented
        let doc = pandoc::Document {
            meta: pandoc::MetaData::new(),
            blocks: Vec::new(),
        };
        let (_arena, _root) = from_pandoc_ast(&doc);
    }
}
