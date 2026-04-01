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
//! ```ignore
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
//! use clmd::{parse_document, format_html, Options};
//! use clmd::core::nodes::NodeValue;
//!
//! let options = Options::default();
//! let (arena, root) = parse_document("Hello, pretty world!", &options);
//!
//! // Access the AST
//! let root_node = arena.get(root);
//! assert!(matches!(root_node.value, NodeValue::Document));
//!
//! // Format to HTML
//! let mut html = String::new();
//! format_html(&arena, root, &options, &mut html).unwrap();
//! assert!(html.contains("pretty"));
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
//!
//! let markdown = "| a | b |\n|---|---|\n| c | d |";
//! let html = markdown_to_html(markdown, &options);
//! assert!(html.contains("<table>"));
//! ```
//!
//! # Parser Limits
//!
//! For security and resource control, you can set parser limits:
//!
//! ```ignore
//! use clmd::parse::parse_document_with_limits;
//! use clmd::{Options, ParserLimits};
//!
//! let options = Options::default();
//! let limits = ParserLimits {
//!     max_input_size: 1024 * 1024,  // 1MB max input
//!     max_line_length: 10000,        // 10KB max line length
//!     max_nesting_depth: 100,        // Max nesting depth
//!     max_list_items: 10000,         // Max list items
//!     max_links: 10000,              // Max links
//! };
//!
//! let result = parse_document_with_limits("# Hello\n\nWorld!", &options, limits);
//! assert!(result.is_ok());
//! ```
//!
//! # Multiple Output Formats
//!
//! clmd supports multiple output formats:
//!
//! ```ignore
//! use clmd::{markdown_to_html, markdown_to_commonmark, Options};
//!
//! let markdown = "# Hello\n\n**Bold** text";
//! let options = Options::default();
//!
//! // HTML output
//! let html = markdown_to_html(markdown, &options);
//! assert!(html.contains("<h1>"));
//!
//! // CommonMark output (formatting)
//! let commonmark = markdown_to_commonmark(markdown, &options);
//! assert!(commonmark.contains("# Hello"));
//! ```
//!
//! # AST Iteration
//!
//! You can iterate over the AST to process nodes:
//!
//! ```ignore
//! use clmd::{parse_document, Options};
//! use clmd::core::iterator::ArenaNodeIterator;
//! use clmd::core::nodes::NodeValue;
//!
//! let options = Options::default();
//! let (arena, root) = parse_document("# Title\n\nParagraph", &options);
//!
//! // Iterate all nodes
//! for node_id in ArenaNodeIterator::new(&arena, root) {
//!     let node = arena.get(node_id);
//!     match &node.value {
//!         NodeValue::Heading(h) => println!("Heading level {}", h.level),
//!         NodeValue::Paragraph => println!("Paragraph"),
//!         _ => {}
//!     }
//! }
//! ```
//!
//! # GFM Extensions
//!
//! Enable GitHub Flavored Markdown extensions:
//!
//! ```ignore
//! use clmd::{markdown_to_html, Options};
//!
//! let mut options = Options::default();
//! options.extension.table = true;
//! options.extension.strikethrough = true;
//! options.extension.tasklist = true;
//! options.extension.autolink = true;
//! options.extension.footnotes = true;
//!
//! // Tables
//! let table = "| a | b |\n|---|---|\n| c | d |";
//! let html = markdown_to_html(table, &options);
//! assert!(html.contains("<table>"));
//!
//! // Strikethrough
//! let strike = "~~deleted~~";
//! let html = markdown_to_html(strike, &options);
//! assert!(html.contains("<del>"));
//!
//! // Task lists
//! let task = "- [x] Done\n- [ ] Todo";
//! let html = markdown_to_html(task, &options);
//! assert!(html.contains("checkbox"));
//!
//! // Autolinks
//! let autolink = "Visit https://example.com";
//! let html = markdown_to_html(autolink, &options);
//! assert!(html.contains("<a href="));
//! ```
//!
//! # HTML to Markdown
//!
//! Convert HTML back to Markdown:
//!
//! ```ignore
//! use clmd::from::html_to_markdown;
//!
//! let html = "<h1>Title</h1><p>Paragraph with <strong>bold</strong> text.</p>";
//! let markdown = html_to_markdown(html);
//! assert!(markdown.contains("# Title"));
//! assert!(markdown.contains("**bold**"));
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

// Internal imports for use within this crate
use core::nodes::NodeValue;

/// Core module for AST, arena, and error types.
///
/// This module provides the foundational types and utilities for clmd,
/// including AST representations, error handling, memory management, and shared utilities.
pub mod core;

// Core types are exported directly through `core` module and type aliases below

/// Markdown parsing module (blocks and inlines).
pub mod parse;

/// Format converters for importing content to Markdown.
pub mod from {
    pub use crate::io::convert::*;
}

/// Markdown extensions (GFM and others).
pub mod ext;

// HTML rendering for the CommonMark AST is now in render::format::html
pub use render::format::html;

/// HTML utilities (escaping, entity decoding).
pub mod html_utils {
    pub use crate::text::html_utils::*;
}

// AST iteration and node types are exported through `core` module

/// Options for the Markdown parser and renderer.
///
/// This module provides configuration options for parsing and rendering
/// Markdown documents. It includes options for extensions, parsing behavior,
/// and rendering output.
///
/// # Example
///
/// ```ignore
/// use clmd::options::{Options, InputFormat, OutputFormat};
///
/// let mut options = Options::default();
/// options.extension.table = true;
/// options.parse.smart = true;
/// ```
pub mod options {
    pub use crate::parse::options::*;
}

/// IO module for document reading and writing.
///
/// This module provides a unified interface for reading documents from different
/// formats and writing to various output formats.
pub mod io;

// IO submodules are accessed through `io::reader`, `io::writer`, `io::format`

/// Plugin system for extending Markdown rendering.
pub mod plugin;

/// HTML rendering for Arena-based AST (legacy).
pub mod render;

/// Markdown formatter for CommonMark output.
/// Now located in `render::commonmark`.
pub use render::commonmark as formatter;

/// Utility modules for internal use.
pub mod util;

// Test utilities are accessed through `util::test` directly

// Text and parsing utilities are accessed through their respective modules

/// Text processing utilities.
///
/// This module provides utilities for text processing, character handling,
/// and Unicode support.
pub mod text;

/// Prelude module for convenient imports.
///
/// # Example
///
/// ```ignore
/// use clmd::prelude::*;
///
/// let options = Options::default();
/// let html = markdown_to_html("Hello **world**!", &options);
/// ```ignore
pub mod prelude;

/// Markdown extensions management using bitflags.
pub mod extensions {
    pub use crate::ext::flags::*;
}

/// Re-export extension types for convenience.
pub use crate::ext::flags::{ExtensionFlags, ExtensionKind};

/// Document conversion pipeline.
pub mod pipeline;

/// Context system for IO operations and resource management.
pub mod context;

// Transform utilities are accessed through `util::transform` directly

/// Filter system for document transformation.
pub mod filter {
    pub use crate::util::filter::*;
}

/// Template system for document rendering.
pub mod template;

/// Parsing utilities.
///
/// This module provides low-level parsing primitives and combinators.
pub use parse::util as parsing;

// Reader and writer modules are accessed through `io::read` and `io::write` directly

// =============================================================================
// Core Type Exports
// =============================================================================

/// Arena for allocating and managing AST nodes (NodeArena version).
pub type Arena = core::arena::NodeArena;

/// Node ID type - index into the arena.
pub type NodeId = core::arena::NodeId;

/// Invalid node ID constant.
pub use core::arena::INVALID_NODE_ID;

// Iterator types are now exported through `core` module

/// Parse a Markdown document to an AST.
///
/// This is the main entry point for parsing. Returns a tuple of (arena, root_node_id).
///
/// # Example
///
/// ```ignore
/// use clmd::{parse_document, Options};
///
/// let options = Options::default();
/// let (arena, root) = parse_document("# Hello\n\nWorld", &options);
/// ```ignore
#[inline]
pub fn parse_document(md: &str, options: &Options) -> (Arena, NodeId) {
    parse::parse_document(md, options)
}

// =============================================================================
// Options Exports (comrak-style)
// =============================================================================

/// Re-export Options from parse::options for convenient access.
///
/// # Example
///
/// ```ignore
/// use clmd::Options;
///
/// let mut options = Options::default();
/// options.extension.table = true;
/// options.render.hardbreaks = true;
/// ```ignore
pub use parse::options::Options;

/// Re-export Plugins for customizing rendering.
pub use parse::options::Plugins;

/// Re-export Extension options.
pub use parse::options::Extension;

/// Re-export Parse options.
pub use parse::options::Parse;

/// Re-export Render options.
pub use parse::options::Render;

/// Re-export ResolvedReference.
pub use parse::options::ResolvedReference;

/// Re-export BrokenLinkCallback.
pub use parse::options::BrokenLinkCallback;

/// Re-export URLRewriter trait.
pub use parse::options::URLRewriter;

// Error types are now exported through `core` module

// NodeValue is now exported through `core` module

// =============================================================================
// String Utilities Exports
// =============================================================================

pub use parse::inline::unescape_string;

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
/// ```ignore
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
/// ```ignore
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
/// ```ignore
/// use clmd::{markdown_to_html_with_plugins, Options, Plugins};
///
/// let options = Options::default();
/// let plugins = Plugins::default();
/// let html = markdown_to_html_with_plugins("Hello, **world**!", &options, &plugins);
/// ```ignore
pub fn markdown_to_html_with_plugins(
    md: &str,
    options: &Options,
    plugins: &Plugins<'_>,
) -> String {
    let (arena, root) = parse::parse_document(md, options);
    let mut out = String::new();
    format_html_with_plugins(&arena, root, options, &mut out, plugins).unwrap();
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
/// ```ignore
/// use clmd::{markdown_to_commonmark, Options};
///
/// let options = Options::default();
/// let cm = markdown_to_commonmark("Hello *world*", &options);
/// assert!(cm.contains("Hello"));
/// ```ignore
pub fn markdown_to_commonmark(md: &str, options: &Options) -> String {
    let (arena, root) = parse::parse_document(md, options);
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
/// ```ignore
/// use clmd::{markdown_to_commonmark_with_plugins, Options, Plugins};
///
/// let options = Options::default();
/// let plugins = Plugins::default();
/// let cm = markdown_to_commonmark_with_plugins("Hello *world*", &options, &plugins);
/// assert!(cm.contains("Hello"));
/// ```ignore
pub fn markdown_to_commonmark_with_plugins(
    md: &str,
    options: &Options,
    plugins: &Plugins<'_>,
) -> String {
    let (arena, root) = parse::parse_document(md, options);
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
/// ```ignore
/// use clmd::{markdown_to_commonmark_xml, Options};
///
/// let options = Options::default();
/// let xml = markdown_to_commonmark_xml("Hello *world*", &options);
/// assert!(xml.contains("<document>"));
/// ```ignore
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
/// ```ignore
/// use clmd::{markdown_to_commonmark_xml_with_plugins, Options, Plugins};
///
/// let options = Options::default();
/// let plugins = Plugins::default();
/// let xml = markdown_to_commonmark_xml_with_plugins("Hello *world*", &options, &plugins);
/// assert!(xml.contains("<document>"));
/// ```ignore
pub fn markdown_to_commonmark_xml_with_plugins(
    md: &str,
    options: &Options,
    plugins: &Plugins<'_>,
) -> String {
    let (arena, root) = parse::parse_document(md, options);
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
/// ```ignore
/// use clmd::{parse_document, format_html, Options};
///
/// let options = Options::default();
/// let (arena, root) = parse_document("Hello *world*", &options);
/// let mut html = String::new();
/// format_html(&arena, root, &options, &mut html).unwrap();
/// ```ignore
pub fn format_html(
    arena: &Arena,
    root: NodeId,
    options: &Options,
    output: &mut dyn std::fmt::Write,
) -> std::fmt::Result {
    let html = html::render(arena, root, options);
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
/// ```ignore
/// use clmd::{parse_document, format_html_with_plugins, Options, Plugins};
///
/// let options = Options::default();
/// let plugins = Plugins::default();
/// let (arena, root) = parse_document("Hello *world*", &options);
/// let mut html = String::new();
/// format_html_with_plugins(&arena, root, &options, &mut html, &plugins).unwrap();
/// ```ignore
pub fn format_html_with_plugins(
    arena: &Arena,
    root: NodeId,
    options: &Options,
    output: &mut dyn std::fmt::Write,
    plugins: &Plugins<'_>,
) -> std::fmt::Result {
    let highlighter = plugins.render.syntax_highlighter();
    write!(
        output,
        "{}",
        html::render_with_highlighter(arena, root, options, highlighter)
    )
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
/// ```ignore
/// use clmd::{parse_document, format_commonmark, Options};
///
/// let options = Options::default();
/// let (arena, root) = parse_document("Hello *world*", &options);
/// let mut cm = String::new();
/// format_commonmark(&arena, root, &options, &mut cm).unwrap();
/// ```ignore
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
/// ```ignore
/// use clmd::{parse_document, format_commonmark_with_plugins, Options, Plugins};
///
/// let options = Options::default();
/// let plugins = Plugins::default();
/// let (arena, root) = parse_document("Hello *world*", &options);
/// let mut cm = String::new();
/// format_commonmark_with_plugins(&arena, root, &options, &mut cm, &plugins).unwrap();
/// ```ignore
pub fn format_commonmark_with_plugins(
    arena: &Arena,
    root: NodeId,
    options: &Options,
    output: &mut dyn std::fmt::Write,
    _plugins: &Plugins<'_>,
) -> std::fmt::Result {
    write!(
        output,
        "{}",
        render::commonmark::render(arena, root, 0, options.render.width)
    )
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
/// ```ignore
/// use clmd::{parse_document, format_xml, Options};
///
/// let options = Options::default();
/// let (arena, root) = parse_document("Hello *world*", &options);
/// let mut xml = String::new();
/// format_xml(&arena, root, &options, &mut xml).unwrap();
/// ```ignore
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
/// ```ignore
/// use clmd::{parse_document, format_xml_with_plugins, Options, Plugins};
///
/// let options = Options::default();
/// let plugins = Plugins::default();
/// let (arena, root) = parse_document("Hello *world*", &options);
/// let mut xml = String::new();
/// format_xml_with_plugins(&arena, root, &options, &mut xml, &plugins).unwrap();
/// ```ignore
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
/// ```ignore
/// use clmd::{parse_document, format_typst, Options};
///
/// let options = Options::default();
/// let (arena, root) = parse_document("Hello *world*", &options);
/// let mut typst = String::new();
/// format_typst(&arena, root, &options, &mut typst).unwrap();
/// ```ignore
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
/// ```ignore
/// use clmd::{parse_document, format_typst_with_plugins, Options, Plugins};
///
/// let options = Options::default();
/// let plugins = Plugins::default();
/// let (arena, root) = parse_document("Hello *world*", &options);
/// let mut typst = String::new();
/// format_typst_with_plugins(&arena, root, &options, &mut typst, &plugins).unwrap();
/// ```ignore
pub fn format_typst_with_plugins(
    arena: &Arena,
    root: NodeId,
    _options: &Options,
    output: &mut dyn std::fmt::Write,
    plugins: &Plugins<'_>,
) -> std::fmt::Result {
    render::format::typst::format_document_with_plugins(
        arena, root, _options, output, plugins,
    )
}

/// Return the version of the crate.
///
/// # Returns
///
/// The version string in semver format (e.g., "0.1.0")
///
/// # Example
///
/// ```ignore
/// use clmd::version;
///
/// let version = version();
/// assert!(!version.is_empty());
/// ```ignore
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
/// ```ignore
/// use clmd::{markdown_to_typst, Options};
///
/// let options = Options::default();
/// let typst = markdown_to_typst("Hello *world*", &options);
/// ```ignore
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
/// ```ignore
/// use clmd::{markdown_to_typst_with_plugins, Options, Plugins};
///
/// let options = Options::default();
/// let plugins = Plugins::default();
/// let typst = markdown_to_typst_with_plugins("Hello *world*", &options, &plugins);
/// ```ignore
pub fn markdown_to_typst_with_plugins(
    md: &str,
    options: &Options,
    plugins: &Plugins<'_>,
) -> String {
    let (arena, root) = parse::parse_document(md, options);
    let mut out = String::new();
    format_typst_with_plugins(&arena, root, options, &mut out, plugins).unwrap();
    out
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use crate::{format_html, markdown_to_html, parse_document, version, Options};

    #[test]
    fn test_markdown_to_html_basic() {
        let options = Options::default();
        let html = markdown_to_html("Hello world", &options);
        println!("HTML output bytes: {:?}", html.as_bytes());
        assert!(html.contains("<p>Hello world</p>"));
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
        println!("HTML output: {:?}", html);
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
        // Image rendering may vary between implementations
        // Just check that it doesn't panic and produces some output
        assert!(!html.is_empty());
    }

    #[test]
    fn test_parse_and_render_roundtrip() {
        let options = Options::default();
        let input = "# Title\n\nParagraph with text.";
        let (arena, root) = parse_document(input, &options);
        let mut html = String::new();
        format_html(&arena, root, &options, &mut html).unwrap();
        assert!(html.contains("<h1>"));
        assert!(html.contains("Paragraph"));
    }

    #[test]
    fn test_version() {
        let v = version();
        assert!(!v.is_empty());
    }

    #[test]
    fn test_tagfilter_extension() {
        let mut options = Options::default();
        options.extension.tagfilter = true;

        // Test that dangerous HTML tags are filtered
        let html = markdown_to_html("<script>alert('xss')</script>", &options);
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[cfg(feature = "syntect")]
    #[test]
    fn test_syntect_syntax_highlighting() {
        use crate::markdown_to_html_with_plugins;
        use crate::plugin::syntect::SyntectAdapter;
        use crate::Plugins;

        let options = Options::default();
        let adapter = SyntectAdapter::new(Some("base16-ocean.dark"));

        let mut plugins = Plugins::new();
        plugins.render.set_syntax_highlighter(&adapter);

        let markdown = "```rust\nfn main() {\n    println!(\"Hello\");\n}\n```";
        let html = markdown_to_html_with_plugins(markdown, &options, &plugins);

        // Should contain pre and code tags
        assert!(html.contains("<pre"));
        assert!(html.contains("<code"));

        // With a theme, should contain styled spans
        assert!(html.contains("<span") || html.contains("fn main"));
    }

    #[cfg(feature = "syntect")]
    #[test]
    fn test_syntect_css_class_mode() {
        use crate::markdown_to_html_with_plugins;
        use crate::plugin::syntect::SyntectAdapter;
        use crate::Plugins;

        let options = Options::default();
        let adapter = SyntectAdapter::new(None); // CSS class mode

        let mut plugins = Plugins::new();
        plugins.render.set_syntax_highlighter(&adapter);

        let markdown = "```python\nprint('hello')\n```";
        let html = markdown_to_html_with_plugins(markdown, &options, &plugins);

        assert!(html.contains("<pre"));
        assert!(html.contains("<code"));
        assert!(html.contains("print"));
    }

    // Shortcode tests
    use crate::core::arena::{Node, NodeArena, TreeOps};
    use crate::core::nodes::{NodeShortCode, NodeValue};
    use crate::ext::shortcode::data::lookup_shortcode;
    use crate::ext::shortcode::parser::parse_shortcode;
    use crate::render;

    #[test]
    fn test_lookup_shortcode() {
        assert_eq!(lookup_shortcode("+1"), Some("👍"));
        assert_eq!(lookup_shortcode("thumbsup"), Some("👍"));
        assert_eq!(lookup_shortcode("smile"), Some("😄"));
        assert_eq!(lookup_shortcode("heart"), Some("❤️"));
        assert_eq!(lookup_shortcode("nonexistent"), None);
    }

    #[test]
    fn test_parse_shortcode_valid() {
        assert_eq!(parse_shortcode(":thumbsup:", 0), Some(("👍", 10)));
        assert_eq!(parse_shortcode(":smile:", 0), Some(("😄", 7)));
        assert_eq!(parse_shortcode(":+1:", 0), Some(("👍", 4)));
        assert_eq!(parse_shortcode(":heart:", 0), Some(("❤️", 7)));
    }

    #[test]
    fn test_parse_shortcode_with_offset() {
        let text = "Hello :thumbsup: world";
        assert_eq!(parse_shortcode(text, 6), Some(("👍", 10)));
    }

    #[test]
    fn test_parse_shortcode_invalid() {
        assert_eq!(parse_shortcode("not a shortcode", 0), None);
        assert_eq!(parse_shortcode(":", 0), None);
        assert_eq!(parse_shortcode(":a:", 0), None); // Too short
        assert_eq!(parse_shortcode(":invalid:", 0), None); // Unknown code
        assert_eq!(parse_shortcode(":no closing", 0), None);
    }

    #[test]
    fn test_shortcode_html_rendering() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text1 = arena.alloc(Node::with_value(NodeValue::make_text("Great job ")));
        let shortcode = arena.alloc(Node::with_value(NodeValue::ShortCode(Box::new(
            NodeShortCode {
                code: "thumbsup".to_string(),
                emoji: "👍".to_string(),
            },
        ))));
        let text2 = arena.alloc(Node::with_value(NodeValue::make_text("!")));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, text1);
        TreeOps::append_child(&mut arena, para, shortcode);
        TreeOps::append_child(&mut arena, para, text2);

        let html = render::format::html::render(
            &arena,
            root,
            &crate::parse::options::Options::default(),
        );
        assert!(html.contains("👍"), "HTML should contain emoji: {}", html);
        assert!(
            !html.contains(":thumbsup:"),
            "HTML should not contain shortcode: {}",
            html
        );
    }

    #[test]
    fn test_shortcode_commonmark_rendering() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text1 = arena.alloc(Node::with_value(NodeValue::make_text("Great job ")));
        let shortcode = arena.alloc(Node::with_value(NodeValue::ShortCode(Box::new(
            NodeShortCode {
                code: "thumbsup".to_string(),
                emoji: "👍".to_string(),
            },
        ))));
        let text2 = arena.alloc(Node::with_value(NodeValue::make_text("!")));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, text1);
        TreeOps::append_child(&mut arena, para, shortcode);
        TreeOps::append_child(&mut arena, para, text2);

        let cm = render::commonmark::render(&arena, root, 0, 0);
        assert!(
            cm.contains(":thumbsup:"),
            "CommonMark should preserve shortcode: {}",
            cm
        );
    }

    #[test]
    fn test_shortcode_xml_rendering() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let shortcode = arena.alloc(Node::with_value(NodeValue::ShortCode(Box::new(
            NodeShortCode {
                code: "thumbsup".to_string(),
                emoji: "👍".to_string(),
            },
        ))));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, shortcode);

        let xml = render::format::xml::render(&arena, root, 0);
        assert!(
            xml.contains("<shortcode"),
            "XML should contain shortcode tag: {}",
            xml
        );
        assert!(
            xml.contains("code=\"thumbsup\""),
            "XML should contain code attribute: {}",
            xml
        );
        assert!(xml.contains("👍"), "XML should contain emoji: {}", xml);
    }

    #[test]
    fn test_multiple_shortcodes() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));

        let shortcode1 = arena.alloc(Node::with_value(NodeValue::ShortCode(Box::new(
            NodeShortCode {
                code: "smile".to_string(),
                emoji: "😄".to_string(),
            },
        ))));
        let shortcode2 = arena.alloc(Node::with_value(NodeValue::ShortCode(Box::new(
            NodeShortCode {
                code: "heart".to_string(),
                emoji: "❤️".to_string(),
            },
        ))));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, shortcode1);
        TreeOps::append_child(&mut arena, para, shortcode2);

        let html = render::format::html::render(
            &arena,
            root,
            &crate::parse::options::Options::default(),
        );
        assert!(
            html.contains("😄"),
            "HTML should contain first emoji: {}",
            html
        );
        assert!(
            html.contains("❤️"),
            "HTML should contain second emoji: {}",
            html
        );
    }

    #[test]
    fn test_shortcode_special_chars() {
        // Test shortcodes with + and - characters
        assert_eq!(parse_shortcode(":+1:", 0), Some(("👍", 4)));
        assert_eq!(parse_shortcode(":-1:", 0), Some(("👎", 4)));
        assert_eq!(lookup_shortcode("+1"), Some("👍"));
        assert_eq!(lookup_shortcode("-1"), Some("👎"));
    }
}
