//! A 100% [CommonMark](http://commonmark.org/) and [GFM](https://github.github.com/gfm/)
//! compatible Markdown parser.
//!
//! Source repository is at [github.com/clmd](https://github.com/clmd).
//!
//! # Safety
//!
//! This crate is 100% safe Rust - it contains no `unsafe` code.
//!
//! # Quick Start
//!
//! The simplest way to use this library is with [`markdown_to_html`]:
//!
//! ```
//! use clmd::{markdown_to_html, Options};
//!
//! let html = markdown_to_html("Hello, **world**!", &Options::default());
//! assert_eq!(html, "<p>Hello, <strong>world</strong>!</p>");
//! ```
//!
//! # Working with the AST
//!
//! For more control, you can parse the input into an AST, manipulate it, and then format it:
//!
//! ```ignore
//! use clmd::{Arena, parse_document, format_html, Options};
//! use clmd::nodes::NodeValue;
//!
//! let arena = Arena::new();
//! let options = Options::default();
//! let root = parse_document(&arena, "Hello, pretty world!", &options);
//!
//! // Manipulate the AST
//! for node in root.descendants() {
//!     if let NodeValue::Text(ref mut text) = node.data.borrow_mut().value {
//!         *text = text.to_string().replace("pretty", "beautiful").into();
//!     }
//! }
//!
//! let mut html = String::new();
//! format_html(root, &options, &mut html).unwrap();
//! assert!(html.contains("beautiful"));
//! ```
//!
//! # Using Options
//!
//! You can enable GFM extensions and configure rendering:
//!
//! ```ignore
//! use clmd::{markdown_to_html, Options};
//!
//! let mut options = Options::default();
//! options.extension.table = true;
//! options.extension.strikethrough = true;
//!
//! let markdown = "| a | b |\n|---|---|\n| c | d |\n\n~~deleted~~";
//! let html = markdown_to_html(markdown, &options);
//! assert!(html.contains("<table>"));
//! assert!(html.contains("<del>deleted</del>"));
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unstable_features,
    unused_import_braces,
    unsafe_code
)]
#![allow(
    unknown_lints,
    clippy::doc_markdown,
    clippy::too_many_arguments,
    cyclomatic_complexity
)]

/// Adapter traits for plugins.
///
/// Provides traits for customizing syntax highlighting, heading rendering,
/// and code block handling.
pub mod adapters;

/// Arena-based memory management for AST nodes.
pub mod arena;

/// Error types and parsing limits.
pub mod error;

/// Block-level parsing for CommonMark documents.
mod blocks;

/// Format converters for importing content to Markdown.
pub mod from;

/// Markdown extensions (GFM and others).
pub mod ext;

// HTML rendering for the CommonMark AST is now in render::html
pub use render::html;

/// HTML utilities (escaping, entity decoding).
pub mod html_utils;

/// Inline parsing for CommonMark documents.
pub(crate) mod inlines;

/// AST iteration and traversal
pub mod iterator;

/// AST node definitions (unified API, inspired by comrak)
///
/// This module provides a unified `NodeValue` enum that combines node type and data,
/// offering better type safety and ergonomics compared to the separate `NodeType` and `NodeData` approach.
///
pub mod nodes;

/// Options for the Markdown parser and renderer.
pub mod options;

/// Parser module for Markdown documents.
pub mod parser;

/// Plugin system for extending Markdown rendering.
pub mod plugins;

/// HTML rendering for Arena-based AST (legacy).
pub mod render;

/// Text sequence utilities.
pub mod sequence;

/// Test utilities.
pub mod test_utils;

/// String processing utilities.
pub mod strings;

/// Scanner utilities for CommonMark syntax.
pub mod scanners;

/// Prelude module for convenient imports.
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

/// Arena for allocating and managing AST nodes (NodeArena version).
pub type Arena = arena::NodeArena;

/// Node ID type - index into the arena.
pub type NodeId = arena::NodeId;

/// Invalid node ID constant.
pub use arena::INVALID_NODE_ID;

/// Re-export iterator types for tree traversal.
pub use arena::{
    AncestorIterator, ChildrenIterator, DescendantIterator, FollowingSiblingsIterator,
    PrecedingSiblingsIterator, SiblingsIterator,
};

/// Parse a Markdown document to an AST.
///
/// This is the main entry point for parsing. Returns a tuple of (arena, root_node_id).
///
/// # Example
///
/// ```
/// use clmd::{parse_document, Options};
///
/// let options = Options::default();
/// let (arena, root) = parse_document("# Hello\n\nWorld", &options);
/// ```
#[inline]
pub fn parse_document(md: &str, options: &Options) -> (Arena, NodeId) {
    parser::parse_document(md, options)
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

/// Re-export Plugins for customizing rendering.
pub use parser::options::Plugins;

/// Re-export Extension options.
pub use parser::options::Extension;

/// Re-export Parse options.
pub use parser::options::Parse;

/// Re-export Render options.
pub use parser::options::Render;

/// Re-export ResolvedReference.
pub use parser::options::ResolvedReference;

/// Re-export BrokenLinkCallback.
pub use parser::options::BrokenLinkCallback;

/// Re-export URLRewriter trait.
pub use parser::options::URLRewriter;

// =============================================================================
// Error Type Exports
// =============================================================================

pub use error::{ParseError, ParseResult, ParserLimits};

// =============================================================================
// Node Value Exports
// =============================================================================

pub use nodes::NodeValue;

// =============================================================================
// String Utilities Exports
// =============================================================================

pub use inlines::unescape_string;

// =============================================================================
// CommonMark Constants
// =============================================================================

/// Code indent threshold (4 spaces or 1 tab)
pub const CODE_INDENT: usize = 4;

/// Tab stop size
pub const TAB_STOP: usize = 4;

/// Check if a character is a space or tab
pub fn is_space_or_tab(c: char) -> bool {
    c == ' ' || c == '\t'
}

// =============================================================================
// Deprecated Type Aliases (for backward compatibility)
// =============================================================================

#[deprecated(
    since = "0.2.0",
    note = "use `clmd::parser::options::Extension` instead of `clmd::ExtensionOptions`"
)]
/// Deprecated alias: use [`parser::options::Extension`] instead.
pub type ExtensionOptions<'c> = parser::options::Extension<'c>;

#[deprecated(
    since = "0.2.0",
    note = "use `clmd::parser::options::Parse` instead of `clmd::ParseOptions`"
)]
/// Deprecated alias: use [`parser::options::Parse`] instead.
pub type ParseOptions<'c> = parser::options::Parse<'c>;

#[deprecated(
    since = "0.2.0",
    note = "use `clmd::parser::options::Render` instead of `clmd::RenderOptions`"
)]
/// Deprecated alias: use [`parser::options::Render`] instead.
pub type RenderOptions = parser::options::Render;

#[deprecated(
    since = "0.2.0",
    note = "use `clmd::parser::options::BrokenLinkReference` instead of `clmd::BrokenLinkReference`"
)]
/// Deprecated alias: use [`parser::options::BrokenLinkReference`] instead.
pub type BrokenLinkReference<'l> = parser::options::BrokenLinkReference<'l>;

#[deprecated(
    since = "0.2.0",
    note = "use `clmd::parser::options::ListStyleType` instead of `clmd::ListStyleType`"
)]
/// Deprecated alias: use [`parser::options::ListStyleType`] instead.
pub type ListStyleType = parser::options::ListStyleType;

#[deprecated(
    since = "0.2.0",
    note = "use `clmd::parser::options::WikiLinksMode` instead of `clmd::WikiLinksMode`"
)]
/// Deprecated alias: use [`parser::options::WikiLinksMode`] instead.
pub type WikiLinksMode = parser::options::WikiLinksMode;

#[deprecated(
    since = "0.2.0",
    note = "use `clmd::parser::options::RenderPlugins` instead of `clmd::RenderPlugins`"
)]
/// Deprecated alias: use [`parser::options::RenderPlugins`] instead.
pub type RenderPlugins<'p> = parser::options::RenderPlugins<'p>;

// =============================================================================
// Convenience Functions
// =============================================================================

/// Render Markdown to HTML.
///
/// This is the main entry point for converting Markdown to HTML.
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
///
/// // Basic usage
/// let html = markdown_to_html("Hello, **world**!", &Options::default());
/// assert_eq!(html, "<p>Hello, <strong>world</strong>!</p>");
///
/// // With headings and lists
/// let markdown = "# Title\n\n- Item 1\n- Item 2";
/// let html = markdown_to_html(markdown, &Options::default());
/// assert!(html.contains("<h1>"));
/// assert!(html.contains("<ul>"));
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
    _plugins: &Plugins<'_>,
) -> String {
    let (arena, root) = parser::parse_document(md, options);
    let mut out = String::new();
    format_html(&arena, root, options, &mut out).unwrap();
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
    let (arena, root) = parser::parse_document(md, options);
    let mut out = String::new();
    format_commonmark(&arena, root, options, &mut out).unwrap();
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
    let (arena, root) = parser::parse_document(md, options);
    let mut out = String::new();
    format_commonmark_with_plugins(&arena, root, options, &mut out, plugins).unwrap();
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
    let (arena, root) = parser::parse_document(md, options);
    let mut out = String::new();
    format_xml_with_plugins(&arena, root, options, &mut out, plugins).unwrap();
    out
}

/// Format an existing AST to HTML.
///
/// This function uses the HTML formatter with NodeArena.
///
/// # Arguments
///
/// * `arena` - The NodeArena containing the AST
/// * `root` - The root node ID
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
/// use clmd::{parse_document, format_html, Options};
///
/// let options = Options::default();
/// let (arena, root) = parse_document("Hello *world*", &options);
/// let mut html = String::new();
/// format_html(&arena, root, &options, &mut html).unwrap();
/// ```
pub fn format_html(
    arena: &Arena,
    root: NodeId,
    _options: &Options,
    output: &mut dyn std::fmt::Write,
) -> std::fmt::Result {
    let html = html::render(arena, root, 0);
    write!(output, "{}", html)
}

/// Format an existing AST to HTML with plugins.
///
/// # Arguments
///
/// * `arena` - The NodeArena containing the AST
/// * `root` - The root node ID
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
/// use clmd::{parse_document, format_html_with_plugins, Options, Plugins};
///
/// let options = Options::default();
/// let plugins = Plugins::default();
/// let (arena, root) = parse_document("Hello *world*", &options);
/// let mut html = String::new();
/// format_html_with_plugins(&arena, root, &options, &mut html, &plugins).unwrap();
/// ```
pub fn format_html_with_plugins(
    arena: &Arena,
    root: NodeId,
    _options: &Options,
    output: &mut dyn std::fmt::Write,
    _plugins: &Plugins<'_>,
) -> std::fmt::Result {
    write!(output, "{}", html::render(arena, root, 0))
}

/// Format an existing AST to CommonMark.
///
/// # Arguments
///
/// * `arena` - The NodeArena containing the AST
/// * `root` - The root node ID
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
/// use clmd::{parse_document, format_commonmark, Options};
///
/// let options = Options::default();
/// let (arena, root) = parse_document("Hello *world*", &options);
/// let mut cm = String::new();
/// format_commonmark(&arena, root, &options, &mut cm).unwrap();
/// ```
pub fn format_commonmark(
    arena: &Arena,
    root: NodeId,
    options: &Options,
    output: &mut dyn std::fmt::Write,
) -> std::fmt::Result {
    format_commonmark_with_plugins(arena, root, options, output, &Plugins::default())
}

/// Format an existing AST to CommonMark with plugins.
///
/// # Arguments
///
/// * `arena` - The NodeArena containing the AST
/// * `root` - The root node ID
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
/// use clmd::{parse_document, format_commonmark_with_plugins, Options, Plugins};
///
/// let options = Options::default();
/// let plugins = Plugins::default();
/// let (arena, root) = parse_document("Hello *world*", &options);
/// let mut cm = String::new();
/// format_commonmark_with_plugins(&arena, root, &options, &mut cm, &plugins).unwrap();
/// ```
pub fn format_commonmark_with_plugins(
    arena: &Arena,
    root: NodeId,
    _options: &Options,
    output: &mut dyn std::fmt::Write,
    _plugins: &Plugins<'_>,
) -> std::fmt::Result {
    write!(output, "{}", render::commonmark::render(arena, root, 0))
}

/// Format an existing AST to XML.
///
/// # Arguments
///
/// * `arena` - The NodeArena containing the AST
/// * `root` - The root node ID
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
/// use clmd::{parse_document, format_xml, Options};
///
/// let options = Options::default();
/// let (arena, root) = parse_document("Hello *world*", &options);
/// let mut xml = String::new();
/// format_xml(&arena, root, &options, &mut xml).unwrap();
/// ```
pub fn format_xml(
    arena: &Arena,
    root: NodeId,
    options: &Options,
    output: &mut dyn std::fmt::Write,
) -> std::fmt::Result {
    format_xml_with_plugins(arena, root, options, output, &Plugins::default())
}

/// Format an existing AST to XML with plugins.
///
/// # Arguments
///
/// * `arena` - The NodeArena containing the AST
/// * `root` - The root node ID
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
/// use clmd::{parse_document, format_xml_with_plugins, Options, Plugins};
///
/// let options = Options::default();
/// let plugins = Plugins::default();
/// let (arena, root) = parse_document("Hello *world*", &options);
/// let mut xml = String::new();
/// format_xml_with_plugins(&arena, root, &options, &mut xml, &plugins).unwrap();
/// ```
pub fn format_xml_with_plugins(
    arena: &Arena,
    root: NodeId,
    _options: &Options,
    output: &mut dyn std::fmt::Write,
    _plugins: &Plugins<'_>,
) -> std::fmt::Result {
    // For now, use a simple XML representation
    writeln!(output, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>")?;
    writeln!(output, "<!DOCTYPE document SYSTEM \"CommonMark.dtd\">")?;
    format_node_xml(arena, root, output, 0)
}

fn format_node_xml(
    arena: &Arena,
    node_id: NodeId,
    output: &mut dyn std::fmt::Write,
    depth: usize,
) -> std::fmt::Result {
    let node = arena.get(node_id);
    let indent = "  ".repeat(depth);

    let tag_name = match &node.value {
        NodeValue::Document => "document",
        NodeValue::Paragraph => "paragraph",
        NodeValue::Heading(_) => "heading",
        NodeValue::BlockQuote => "block_quote",
        NodeValue::List(_) => "list",
        NodeValue::Item(_) => "item",
        NodeValue::CodeBlock(_) => "code_block",
        NodeValue::ThematicBreak => "thematic_break",
        NodeValue::Text(_) => "text",
        NodeValue::Emph => "emph",
        NodeValue::Strong => "strong",
        NodeValue::Code(_) => "code",
        NodeValue::Link(_) => "link",
        NodeValue::Image(_) => "image",
        NodeValue::SoftBreak => "softbreak",
        NodeValue::HardBreak => "linebreak",
        _ => "unknown",
    };

    writeln!(output, "{}<{}>", indent, tag_name)?;

    // Process children
    let mut child_opt = node.first_child;
    while let Some(child_id) = child_opt {
        format_node_xml(arena, child_id, output, depth + 1)?;
        child_opt = arena.get(child_id).next;
    }

    writeln!(output, "{}</{}>", indent, tag_name)?;

    Ok(())
}

/// Format an existing AST to Typst.
///
/// # Arguments
///
/// * `arena` - The NodeArena containing the AST
/// * `root` - The root node ID
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
/// use clmd::{parse_document, format_typst, Options};
///
/// let options = Options::default();
/// let (arena, root) = parse_document("Hello *world*", &options);
/// let mut typst = String::new();
/// format_typst(&arena, root, &options, &mut typst).unwrap();
/// ```
pub fn format_typst(
    arena: &Arena,
    root: NodeId,
    options: &Options,
    output: &mut dyn std::fmt::Write,
) -> std::fmt::Result {
    format_typst_with_plugins(arena, root, options, output, &Plugins::default())
}

/// Format an existing AST to Typst with plugins.
///
/// # Arguments
///
/// * `arena` - The NodeArena containing the AST
/// * `root` - The root node ID
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
/// use clmd::{parse_document, format_typst_with_plugins, Options, Plugins};
///
/// let options = Options::default();
/// let plugins = Plugins::default();
/// let (arena, root) = parse_document("Hello *world*", &options);
/// let mut typst = String::new();
/// format_typst_with_plugins(&arena, root, &options, &mut typst, &plugins).unwrap();
/// ```
pub fn format_typst_with_plugins(
    _arena: &Arena,
    _root: NodeId,
    _options: &Options,
    _output: &mut dyn std::fmt::Write,
    _plugins: &Plugins<'_>,
) -> std::fmt::Result {
    // TODO: Implement Typst rendering for NodeArena
    // For now, just return Ok
    Ok(())
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
#[inline]
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
    let (arena, root) = parser::parse_document(md, options);
    let mut out = String::new();
    format_typst_with_plugins(&arena, root, options, &mut out, plugins).unwrap();
    out
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests;
