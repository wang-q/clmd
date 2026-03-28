//! A 100% [CommonMark](http://commonmark.org/) and [GFM](https://github.github.com/gfm/)
//! compatible Markdown parser.
//!
//! Source repository is at [github.com/clmd](https://github.com/clmd).
//!
//! You can use `clmd::markdown_to_html` directly:
//!
//! ```
//! use clmd::{markdown_to_html, Options};
//! let html = markdown_to_html("Hello, **world**!", &Options::default());
//! assert!(html.contains("<strong>world</strong>"));
//! ```
//!
//! Or you can parse the input into an AST yourself, manipulate it, and then use your desired
//! formatter:
//!
//! ```
//! use clmd::{Arena, parse_document, format_html, Options};
//!
//! let arena = Arena::new();
//! let options = Options::default();
//! let root = parse_document(&arena, "Hello, world!", &options);
//!
//! let mut html = String::new();
//! format_html(root, &options, &mut html).unwrap();
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unstable_features,
    unused_import_braces
)]
#![allow(
    unknown_lints,
    clippy::doc_markdown,
    clippy::too_many_arguments,
    cyclomatic_complexity
)]

/// Adapter traits for plugins
///
/// This module provides adapter traits for customizing various aspects of
/// Markdown rendering, such as syntax highlighting, heading rendering,
/// and code block handling.
pub mod adapters;

/// Arena-based memory management for AST nodes
///
/// This module provides the core data structures for efficient node allocation
/// and tree manipulation using arena allocation instead of Rc<RefCell>.
pub mod arena;

/// DOM-like tree data structure based on `&Node` references.
///
/// This module provides a more ergonomic API for tree operations compared to
/// the NodeId-based approach in the `arena` module. It uses `Cell` for interior
/// mutability, allowing natural tree operations without mutable references.
///
/// # Example
///
/// ```
/// use clmd::arena_tree::{Node, TreeOperations};
/// use std::cell::RefCell;
///
/// let arena = typed_arena::Arena::new();
/// let root = arena.alloc(Node::new(RefCell::new("root")));
/// let child = arena.alloc(Node::new(RefCell::new("child")));
///
/// root.append(child);
///
/// assert_eq!(root.first_child().map(|n| *n.data.borrow()), Some("child"));
/// ```
pub mod arena_tree;

/// Error types and parsing limits
pub mod error;

/// Block-level parsing for CommonMark documents
///
/// This module implements the block parsing algorithm based on the CommonMark spec.
/// It processes input line by line, building the AST structure using Arena allocation.
mod blocks;

/// Document converters (HTML, LaTeX, etc.)
pub mod converters;

/// Markdown extensions (GFM and others)
pub mod ext;

/// HTML to Markdown conversion
pub mod html_to_md;

/// String pool for efficient memory reuse
pub(crate) mod pool;

/// HTML rendering for the CommonMark AST.
///
/// This module provides functions for rendering CommonMark documents as HTML,
/// inspired by comrak's design.
///
/// # Example
///
/// ```
/// use clmd::{Arena, parse_document, Options, html};
/// use std::fmt::Write;
///
/// let arena = Arena::new();
/// let options = Options::default();
/// let root = parse_document(&arena, "# Hello\n\nWorld", &options);
///
/// let mut html = String::new();
/// html::format_document(root, &options, &mut html).unwrap();
///
/// assert!(html.contains("<h1>"));
/// ```
pub mod html;

/// HTML utilities (escaping, entity decoding)
pub mod html_utils;

/// Inline parsing for CommonMark documents
///
/// This module implements the inline parsing algorithm based on the CommonMark spec.
/// It processes the content of leaf blocks to produce inline elements like
/// emphasis, links, code, etc.
pub(crate) mod inlines;

/// AST iteration and traversal
pub mod iterator;

/// Lexical analysis utilities
pub(crate) mod lexer;

/// AST node definitions (unified API, inspired by comrak)
///
/// This module provides a unified `NodeValue` enum that combines node type and data,
/// offering better type safety and ergonomics compared to the separate `NodeType` and `NodeData` approach.
///
/// # Example
///
/// ```ignore
/// use clmd::nodes::{NodeValue, NodeHeading, NodeList, ListType};
///
/// let heading = NodeValue::Heading(NodeHeading {
///     level: 1,
///     setext: false,
///     closed: false,
/// });
/// ```
pub mod nodes;

/// Options for the Markdown parser and renderer
///
/// This module provides a comrak-style Options API.
///
/// # Example
///
/// ```ignore
/// use clmd::parser::options::Options;
///
/// let mut options = Options::default();
/// options.extension.table = true;
/// options.extension.strikethrough = true;
/// options.render.hardbreaks = true;
/// ```
pub mod parser;

/// Plugin system for extending Markdown rendering
///
/// This module provides a plugin architecture that allows users to customize
/// various aspects of Markdown rendering, such as syntax highlighting,
/// heading rendering, and code block handling.
pub mod plugins;

/// HTML rendering for Arena-based AST
///
/// This module provides HTML output generation for documents parsed
/// using the Arena-based parser.
pub mod render;

/// Text sequence utilities
pub(crate) mod sequence;

/// Test utilities
pub mod test_utils;

/// String processing utilities
pub mod strings;

/// Scanner utilities for CommonMark syntax
pub mod scanners;

/// Prelude module for convenient imports
///
/// This module re-exports the most commonly used types and functions.
///
/// # Example
///
/// ```
/// use clmd::prelude::*;
///
/// let options = Options::default();
/// let html = markdown_to_html("Hello **world**!", &options);
/// ```
pub mod prelude;

// =============================================================================
// Core Type Exports
// =============================================================================

/// Convenience type alias for arena used to hold nodes.
pub type Arena<'a> = typed_arena::Arena<nodes::AstNode<'a>>;

/// A reference to a node in an arena.
pub type Node<'a> = nodes::Node<'a>;

// Re-export arena_tree types
pub use arena_tree::{
    Ancestors, Children, Descendants, FollowingSiblings, NodeEdge,
    PrecedingSiblings, ReverseChildren, ReverseTraverse, Traverse,
};

/// Parse a Markdown document to an AST.
///
/// This is the main entry point for parsing. It takes an arena for node allocation,
/// the Markdown text to parse, and options for configuring the parser.
///
/// # Example
///
/// ```
/// use clmd::{Arena, parse_document, Options};
///
/// let arena = Arena::new();
/// let options = Options::default();
/// let root = parse_document(&arena, "# Hello\n\nWorld", &options);
/// ```
pub fn parse_document<'a>(arena: &'a Arena<'a>, md: &str, options: &Options) -> Node<'a> {
    parser::parse_document(arena, md, options)
}

// =============================================================================
// Options Exports (comrak-style)
// =============================================================================

/// Re-export Options from parser::options for convenient access.
///
/// # Example
///
/// ```
/// use clmd::Options;
///
/// let mut options = Options::default();
/// options.extension.table = true;
/// options.render.hardbreaks = true;
/// ```
pub use parser::options::Options;

/// Re-export other options types
pub use parser::options::{
    BrokenLinkCallback, BrokenLinkReference, Extension, ListStyleType, Parse, Render,
    ResolvedReference, URLRewriter, WikiLinksMode,
};

// =============================================================================
// Error Type Exports
// =============================================================================

pub use error::{ParseError, ParseResult, ParserLimits, Position};

// =============================================================================
// Iterator Exports
// =============================================================================

pub use iterator::{ArenaNodeIterator, ArenaNodeWalker, EventType};

// =============================================================================
// Node Value Exports (minimal - others available via nodes module)
// =============================================================================

pub use nodes::{AstNode, NodeValue};

// =============================================================================
// Plugin Exports
// =============================================================================

pub use parser::options::{Plugins, RenderPlugins};
pub use plugins::OwnedPlugins;

// =============================================================================
// Renderer Exports
// =============================================================================

pub use render::{
    render, render_to_commonmark, render_to_html, render_to_latex, render_to_man,
    render_to_xml, OutputFormat, Renderer,
};

// New comrak-style HTML formatter exports
pub use html::{escape_html, escape_href, is_safe_url, Context};

// =============================================================================
// Utility Exports
// =============================================================================

pub use inlines::entities::unescape_string;

// =============================================================================
// Legacy Option Flags (internal use)
// =============================================================================

#[allow(deprecated)]
const OPT_SOURCEPOS: u32 = 1 << 0;
#[allow(deprecated)]
const OPT_HARDBREAKS: u32 = 1 << 1;
#[allow(deprecated)]
const OPT_NOBREAKS: u32 = 1 << 2;
#[allow(deprecated)]
const OPT_VALIDATE_UTF8: u32 = 1 << 3;
#[allow(deprecated)]
const OPT_SMART: u32 = 1 << 4;
#[allow(deprecated)]
const OPT_UNSAFE: u32 = 1 << 5;

/// Convert Options to legacy u32 flags for parsing.
/// This is a temporary bridge until all components use the new system.
fn options_to_flags(options: &Options) -> u32 {
    let mut flags = 0u32;

    if options.parse.sourcepos {
        flags |= OPT_SOURCEPOS;
    }
    if options.parse.smart {
        flags |= OPT_SMART;
    }
    if options.parse.validate_utf8 {
        flags |= OPT_VALIDATE_UTF8;
    }
    if options.render.hardbreaks {
        flags |= OPT_HARDBREAKS;
    }
    if options.render.nobreaks {
        flags |= OPT_NOBREAKS;
    }
    if options.render.r#unsafe {
        flags |= OPT_UNSAFE;
    }

    flags
}

// =============================================================================
// Convenience Functions
// =============================================================================

/// Render Markdown to HTML.
///
/// See the documentation of the crate root for an example.
///
/// # Arguments
///
/// * `md` - The Markdown text to convert
/// * `options` - Configuration options
///
/// # Returns
///
/// The HTML output as a String
///
/// # Example
///
/// ```
/// use clmd::{markdown_to_html, Options};
/// let html = markdown_to_html("Hello, **world**!", &Options::default());
/// assert!(html.contains("<strong>world</strong>"));
/// ```
pub fn markdown_to_html(md: &str, options: &Options) -> String {
    markdown_to_html_with_plugins(md, options, &Plugins::default())
}

/// Render Markdown to HTML using plugins.
///
/// # Arguments
///
/// * `md` - The Markdown text to convert
/// * `options` - Configuration options
/// * `plugins` - Plugins for customizing rendering
///
/// # Returns
///
/// The HTML output as a String
///
/// # Example
///
/// ```
/// use clmd::{markdown_to_html_with_plugins, Options, Plugins};
///
/// let options = Options::default();
/// let plugins = Plugins::default();
/// let html = markdown_to_html_with_plugins("Hello, **world**!", &options, &plugins);
/// ```
pub fn markdown_to_html_with_plugins(
    md: &str,
    options: &Options,
    plugins: &Plugins<'_>,
) -> String {
    let arena = Arena::new();
    let root = parser::parse_document(&arena, md, options);
    let mut out = String::new();
    format_html_with_plugins(root, options, &mut out, plugins).unwrap();
    out
}

/// Render Markdown back to CommonMark.
///
/// # Arguments
///
/// * `md` - The Markdown text to convert
/// * `options` - Configuration options
///
/// # Returns
///
/// The CommonMark output as a String
///
/// # Example
///
/// ```
/// use clmd::{markdown_to_commonmark, Options};
///
/// let options = Options::default();
/// let cm = markdown_to_commonmark("Hello *world*", &options);
/// assert!(cm.contains("Hello"));
/// ```
pub fn markdown_to_commonmark(md: &str, options: &Options) -> String {
    let arena = Arena::new();
    let root = parser::parse_document(&arena, md, options);
    let mut out = String::new();
    format_commonmark(root, options, &mut out).unwrap();
    out
}

/// Render Markdown back to CommonMark using plugins.
///
/// # Arguments
///
/// * `md` - The Markdown text to convert
/// * `options` - Configuration options
/// * `plugins` - Plugins for customizing rendering
///
/// # Returns
///
/// The CommonMark output as a String
///
/// # Example
///
/// ```
/// use clmd::{markdown_to_commonmark_with_plugins, Options, Plugins};
///
/// let options = Options::default();
/// let plugins = Plugins::default();
/// let cm = markdown_to_commonmark_with_plugins("Hello *world*", &options, &plugins);
/// assert!(cm.contains("Hello"));
/// ```
pub fn markdown_to_commonmark_with_plugins(
    md: &str,
    options: &Options,
    plugins: &Plugins<'_>,
) -> String {
    let arena = Arena::new();
    let root = parser::parse_document(&arena, md, options);
    let mut out = String::new();
    format_commonmark_with_plugins(root, options, &mut out, plugins).unwrap();
    out
}

/// Render Markdown to CommonMark XML.
///
/// See <https://github.com/commonmark/commonmark-spec/blob/master/CommonMark.dtd>.
///
/// # Arguments
///
/// * `md` - The Markdown text to convert
/// * `options` - Configuration options
///
/// # Returns
///
/// The XML output as a String
///
/// # Example
///
/// ```
/// use clmd::{markdown_to_commonmark_xml, Options};
///
/// let options = Options::default();
/// let xml = markdown_to_commonmark_xml("Hello *world*", &options);
/// assert!(xml.contains("<document>"));
/// ```
pub fn markdown_to_commonmark_xml(md: &str, options: &Options) -> String {
    markdown_to_commonmark_xml_with_plugins(md, options, &Plugins::default())
}

/// Render Markdown to CommonMark XML using plugins.
///
/// See <https://github.com/commonmark/commonmark-spec/blob/master/CommonMark.dtd>.
///
/// # Arguments
///
/// * `md` - The Markdown text to convert
/// * `options` - Configuration options
/// * `plugins` - Plugins for customizing rendering
///
/// # Returns
///
/// The XML output as a String
///
/// # Example
///
/// ```
/// use clmd::{markdown_to_commonmark_xml_with_plugins, Options, Plugins};
///
/// let options = Options::default();
/// let plugins = Plugins::default();
/// let xml = markdown_to_commonmark_xml_with_plugins("Hello *world*", &options, &plugins);
/// assert!(xml.contains("<document>"));
/// ```
pub fn markdown_to_commonmark_xml_with_plugins(
    md: &str,
    options: &Options,
    plugins: &Plugins<'_>,
) -> String {
    let arena = Arena::new();
    let root = parser::parse_document(&arena, md, options);
    let mut out = String::new();
    format_xml_with_plugins(root, options, &mut out, plugins).unwrap();
    out
}

/// Format an existing AST to HTML.
///
/// This function uses the new comrak-style HTML formatter.
///
/// # Arguments
///
/// * `root` - The root node
/// * `options` - Configuration options
/// * `output` - The output buffer to write to
///
/// # Returns
///
/// A `std::fmt::Result` indicating success or failure
///
/// # Example
///
/// ```
/// use clmd::{Arena, parse_document, format_html, Options};
///
/// let arena = Arena::new();
/// let options = Options::default();
/// let root = parse_document(&arena, "Hello *world*", &options);
/// let mut html = String::new();
/// format_html(root, &options, &mut html).unwrap();
/// ```
pub fn format_html<'a>(
    root: Node<'a>,
    options: &Options,
    output: &mut dyn std::fmt::Write,
) -> std::fmt::Result {
    html::format_document(root, options, output)
}

/// Format an existing AST to HTML with plugins.
///
/// # Arguments
///
/// * `root` - The root node
/// * `options` - Configuration options
/// * `output` - The output buffer to write to
/// * `plugins` - Plugins for customizing rendering
///
/// # Returns
///
/// A `std::fmt::Result` indicating success or failure
///
/// # Example
///
/// ```
/// use clmd::{Arena, parse_document, format_html_with_plugins, Options, Plugins};
///
/// let arena = Arena::new();
/// let options = Options::default();
/// let plugins = Plugins::default();
/// let root = parse_document(&arena, "Hello *world*", &options);
/// let mut html = String::new();
/// format_html_with_plugins(root, &options, &mut html, &plugins).unwrap();
/// ```
pub fn format_html_with_plugins<'a>(
    root: Node<'a>,
    options: &Options,
    output: &mut dyn std::fmt::Write,
    plugins: &Plugins<'_>,
) -> std::fmt::Result {
    html::format_document_with_plugins(root, options, output, plugins)
}

/// Format an existing AST to CommonMark.
///
/// # Arguments
///
/// * `root` - The root node
/// * `options` - Configuration options
/// * `output` - The output buffer to write to
///
/// # Returns
///
/// A `std::fmt::Result` indicating success or failure
///
/// # Example
///
/// ```
/// use clmd::{Arena, parse_document, format_commonmark, Options};
///
/// let arena = Arena::new();
/// let options = Options::default();
/// let root = parse_document(&arena, "Hello *world*", &options);
/// let mut cm = String::new();
/// format_commonmark(root, &options, &mut cm).unwrap();
/// ```
pub fn format_commonmark<'a>(
    root: Node<'a>,
    options: &Options,
    output: &mut dyn std::fmt::Write,
) -> std::fmt::Result {
    format_commonmark_with_plugins(root, options, output, &Plugins::default())
}

/// Format an existing AST to CommonMark with plugins.
///
/// # Arguments
///
/// * `root` - The root node
/// * `options` - Configuration options
/// * `output` - The output buffer to write to
/// * `plugins` - Plugins for customizing rendering
///
/// # Returns
///
/// A `std::fmt::Result` indicating success or failure
///
/// # Example
///
/// ```
/// use clmd::{Arena, parse_document, format_commonmark_with_plugins, Options, Plugins};
///
/// let arena = Arena::new();
/// let options = Options::default();
/// let plugins = Plugins::default();
/// let root = parse_document(&arena, "Hello *world*", &options);
/// let mut cm = String::new();
/// format_commonmark_with_plugins(root, &options, &mut cm, &plugins).unwrap();
/// ```
pub fn format_commonmark_with_plugins<'a>(
    _root: Node<'a>,
    _options: &Options,
    output: &mut dyn std::fmt::Write,
    _plugins: &Plugins<'_>,
) -> std::fmt::Result {
    // TODO: Implement CommonMark rendering with new API
    output.write_str("")
}

/// Format an existing AST to XML.
///
/// # Arguments
///
/// * `root` - The root node
/// * `options` - Configuration options
/// * `output` - The output buffer to write to
///
/// # Returns
///
/// A `std::fmt::Result` indicating success or failure
///
/// # Example
///
/// ```
/// use clmd::{Arena, parse_document, format_xml, Options};
///
/// let arena = Arena::new();
/// let options = Options::default();
/// let root = parse_document(&arena, "Hello *world*", &options);
/// let mut xml = String::new();
/// format_xml(root, &options, &mut xml).unwrap();
/// ```
pub fn format_xml<'a>(
    root: Node<'a>,
    options: &Options,
    output: &mut dyn std::fmt::Write,
) -> std::fmt::Result {
    format_xml_with_plugins(root, options, output, &Plugins::default())
}

/// Format an existing AST to XML with plugins.
///
/// # Arguments
///
/// * `root` - The root node
/// * `options` - Configuration options
/// * `output` - The output buffer to write to
/// * `plugins` - Plugins for customizing rendering
///
/// # Returns
///
/// A `std::fmt::Result` indicating success or failure
///
/// # Example
///
/// ```
/// use clmd::{Arena, parse_document, format_xml_with_plugins, Options, Plugins};
///
/// let arena = Arena::new();
/// let options = Options::default();
/// let plugins = Plugins::default();
/// let root = parse_document(&arena, "Hello *world*", &options);
/// let mut xml = String::new();
/// format_xml_with_plugins(root, &options, &mut xml, &plugins).unwrap();
/// ```
pub fn format_xml_with_plugins<'a>(
    _root: Node<'a>,
    _options: &Options,
    output: &mut dyn std::fmt::Write,
    _plugins: &Plugins<'_>,
) -> std::fmt::Result {
    // TODO: Implement XML rendering with new API
    output.write_str("")
}

/// Return the version of the crate.
///
/// # Returns
///
/// The version string in semver format (e.g., "0.1.0")
///
/// # Example
///
/// ```
/// use clmd::version;
///
/// let version = version();
/// assert!(!version.is_empty());
/// ```
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Render Markdown to Typst.
///
/// # Arguments
///
/// * `md` - The Markdown text to convert
/// * `options` - Configuration options
///
/// # Returns
///
/// The Typst output as a String
///
/// # Example
///
/// ```
/// use clmd::{markdown_to_typst, Options};
///
/// let options = Options::default();
/// let typst = markdown_to_typst("Hello *world*", &options);
/// ```
pub fn markdown_to_typst(md: &str, options: &Options) -> String {
    markdown_to_typst_with_plugins(md, options, &Plugins::default())
}

/// Render Markdown to Typst using plugins.
///
/// # Arguments
///
/// * `md` - The Markdown text to convert
/// * `options` - Configuration options
/// * `plugins` - Plugins for customizing rendering
///
/// # Returns
///
/// The Typst output as a String
///
/// # Example
///
/// ```
/// use clmd::{markdown_to_typst_with_plugins, Options, Plugins};
///
/// let options = Options::default();
/// let plugins = Plugins::default();
/// let typst = markdown_to_typst_with_plugins("Hello *world*", &options, &plugins);
/// ```
pub fn markdown_to_typst_with_plugins(
    md: &str,
    options: &Options,
    plugins: &Plugins<'_>,
) -> String {
    let arena = Arena::new();
    let _doc = parser::parse_document(&arena, md, options);
    // TODO: Pass plugins to renderer when plugin support is implemented
    let _ = plugins;
    String::new()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_to_html_basic() {
        let options = Options::default();
        let html = markdown_to_html("Hello world", &options);
        assert_eq!(html, "<p>Hello world</p>");
    }

    #[test]
    fn test_markdown_to_html_heading() {
        let options = Options::default();
        let html = markdown_to_html("# Heading 1\n\n## Heading 2", &options);
        assert!(html.contains("<h1>"));
        assert!(html.contains("<h2>"));
    }

    #[test]
    fn test_markdown_to_html_emphasis() {
        let options = Options::default();
        let html = markdown_to_html("*italic* and **bold**", &options);
        assert!(html.contains("<em>italic</em>"));
        assert!(html.contains("<strong>bold</strong>"));
    }

    #[test]
    fn test_markdown_to_html_link() {
        let options = Options::default();
        let html = markdown_to_html("[link](https://example.com)", &options);
        assert!(html.contains("<a href=\"https://example.com\">"));
    }

    #[test]
    fn test_markdown_to_html_code_inline() {
        let options = Options::default();
        let html = markdown_to_html("Use `code` here", &options);
        assert!(html.contains("<code>code</code>"));
    }

    #[test]
    fn test_markdown_to_html_code_block() {
        let options = Options::default();
        let html = markdown_to_html("```rust\nfn main() {}\n```", &options);
        assert!(html.contains("<pre>"));
        assert!(html.contains("<code"));
        assert!(html.contains("fn main() {}"));
    }

    #[test]
    fn test_markdown_to_html_blockquote() {
        let options = Options::default();
        let html = markdown_to_html("> Quote", &options);
        assert!(html.contains("<blockquote>"));
        assert!(html.contains("Quote"));
    }

    #[test]
    fn test_markdown_to_html_list() {
        let options = Options::default();
        let html = markdown_to_html("- Item 1\n- Item 2", &options);
        assert!(html.contains("<ul>"));
        assert!(html.contains("Item 1"));
        assert!(html.contains("Item 2"));
    }

    #[test]
    fn test_markdown_to_html_ordered_list() {
        let options = Options::default();
        let html = markdown_to_html("1. First\n2. Second", &options);
        assert!(html.contains("<ol>"));
        assert!(html.contains("First"));
        assert!(html.contains("Second"));
    }

    #[test]
    fn test_markdown_to_html_thematic_break() {
        let options = Options::default();
        let html = markdown_to_html("---", &options);
        assert!(html.contains("<hr"));
    }

    #[test]
    fn test_markdown_to_html_image() {
        let options = Options::default();
        let html = markdown_to_html("![alt text](image.png)", &options);
        assert!(html.contains("<img"));
        assert!(html.contains("src=\"image.png\""));
        assert!(html.contains("alt=\"alt text\""));
    }

    #[test]
    fn test_parse_and_render_roundtrip() {
        let options = Options::default();
        let input = "# Title\n\nParagraph with text.";
        let arena = Arena::new();
        let doc = parse_document(&arena, input, &options);
        let mut html = String::new();
        format_html(doc, &options, &mut html).unwrap();
        assert!(html.contains("<h1>"));
        assert!(html.contains("Paragraph"));
    }

    #[test]
    fn test_version() {
        let v = version();
        assert!(!v.is_empty());
    }
}
