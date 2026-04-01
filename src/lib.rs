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
//! use clmd::nodes::NodeValue;
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
//! use clmd::parser::parse_document_with_limits;
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
//! use clmd::iterator::ArenaNodeIterator;
//! use clmd::nodes::NodeValue;
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

/// Core module for AST, arena, and error types.
///
/// This module provides the foundational types and utilities for clmd,
/// including AST representations, error handling, memory management, and shared utilities.
pub mod core;

/// Unified AST definition inspired by Pandoc.
///
/// This module is re-exported from [`core::ast`](crate::core::ast) for backward compatibility.
/// New code should use `core::ast` directly.
pub mod ast {
    pub use crate::core::ast::*;
}

/// Adapter traits for plugins.
///
/// This module is re-exported from [`core::adapters`](crate::core::adapters) for backward compatibility.
/// New code should use `core::adapters` directly.
pub mod adapters {
    pub use crate::core::adapters::*;
}

/// Arena-based memory management for AST nodes.
///
/// This module is re-exported from [`core::arena`](crate::core::arena) for backward compatibility.
/// New code should use `core::arena` directly.
pub mod arena {
    pub use crate::core::arena::*;
}

/// Configuration file support.
///
/// This module is re-exported from [`context::config`](crate::context::config) for backward compatibility.
/// New code should use `context::config` directly.
pub mod config {
    pub use crate::context::config::*;
}

/// Error types and parsing limits.
///
/// This module is re-exported from [`core::error`](crate::core::error) for backward compatibility.
/// New code should use `core::error` directly.
pub mod error {
    pub use crate::core::error::*;
}

/// Block-level parsing for CommonMark documents.
mod blocks;

/// Format converters for importing content to Markdown.
///
/// This module is re-exported from [`formats::from`](crate::formats::from) for backward compatibility.
/// New code should use `formats::from` directly.
pub mod from {
    pub use crate::formats::from::*;
}

/// Markdown extensions (GFM and others).
pub mod ext;

// HTML rendering for the CommonMark AST is now in render::html
pub use render::html;

/// HTML utilities (escaping, entity decoding).
///
/// This module is re-exported from [`text::html_utils`](crate::text::html_utils) for backward compatibility.
/// New code should use `text::html_utils` directly.
pub mod html_utils {
    pub use crate::text::html_utils::*;
}

/// Inline parsing for CommonMark documents.
pub(crate) mod inlines;

/// AST iteration and traversal
///
/// This module is re-exported from [`core::iterator`](crate::core::iterator) for backward compatibility.
/// New code should use `core::iterator` directly.
pub mod iterator {
    pub use crate::core::iterator::*;
}

/// AST node definitions (unified API, inspired by comrak)
///
/// This module provides a unified `NodeValue` enum that combines node type and data,
/// offering better type safety and ergonomics compared to the separate `NodeType` and `NodeData` approach.
///
/// This module is re-exported from [`core::nodes`](crate::core::nodes) for backward compatibility.
/// New code should use `core::nodes` directly.
pub mod nodes {
    pub use crate::core::nodes::*;
}

/// Options for the Markdown parser and renderer.
///
/// This module is re-exported from [`parser::options`](crate::parser::options) for backward compatibility.
/// New code should use `parser::options` directly.
pub mod options {
    pub use crate::parser::options::*;
}

/// Format abstraction layer for document formats.
///
/// This module provides a unified interface for document formats, inspired by
/// Pandoc's format system. It supports both text and binary formats, and provides
/// format detection, MIME type mapping, and format-specific configuration.
pub mod formats;

/// Parser module for Markdown documents.
pub mod parser;

/// Plugin system for extending Markdown rendering.
pub mod plugins;

/// HTML rendering for Arena-based AST (legacy).
pub mod render;

/// Markdown formatter for CommonMark output.
pub mod formatter;

/// Text sequence utilities.
///
/// This module is re-exported from [`text::sequence`](crate::text::sequence) for backward compatibility.
/// New code should use `text::sequence` directly.
pub mod sequence {
    pub use crate::text::sequence::*;
}

/// Test utilities.
pub mod test_utils;

/// String processing utilities.
///
/// This is a re-export of [`text::strings`](crate::text::strings) for backward compatibility.
/// New code should use `text::strings` directly.
pub mod strings {
    pub use crate::text::strings::*;
}

/// Scanner utilities for CommonMark syntax.
///
/// This is a re-export of [`parsing::scanners`](crate::parsing::scanners) for backward compatibility.
/// New code should use `parsing::scanners` directly.
pub mod scanners {
    pub use crate::parsing::scanners::*;
}

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

/// Document readers for various input formats.
///
/// This module provides a unified interface for reading documents from different
/// formats, inspired by Pandoc's Reader system.
pub mod readers;

/// Document writers for various output formats.
///
/// This module provides a unified interface for writing documents to different
/// formats, inspired by Pandoc's Writer system.
pub mod writers;

/// Markdown extensions management using bitflags.
///
/// This module provides a unified way to manage Markdown extensions,
/// inspired by Pandoc's extension system.
pub mod extensions;

/// Document conversion pipeline.
///
/// This module provides a flexible pipeline system for document conversion,
/// inspired by Pandoc's conversion architecture.
pub mod pipeline;

/// Resource management system.
///
/// This module is re-exported from [`context::mediabag`](crate::context::mediabag) for backward compatibility.
/// New code should use `context::mediabag` directly.
pub mod mediabag {
    pub use crate::context::mediabag::*;
}

/// Context system for IO operations and resource management.
///
/// This module provides a unified abstraction for IO operations, logging,
/// and resource management, inspired by Pandoc's PandocMonad.
/// It allows for both real IO operations and pure/mock implementations for testing.
pub mod context;

/// URI handling utilities.
///
/// This module is re-exported from [`text::uri`](crate::text::uri) for backward compatibility.
/// New code should use `text::uri` directly.
pub mod uri {
    pub use crate::text::uri::*;
}

/// Filter system for document transformation.
///
/// This module provides a flexible filter system for transforming documents,
/// inspired by Pandoc's filter architecture. Filters can be used to modify
/// the AST between parsing and rendering.
pub mod filter;

/// Document transformation system.
///
/// This module provides a set of document transformations inspired by
/// Pandoc's transform system. Transforms can modify the AST between
/// parsing and rendering.
pub mod transforms;

/// Slide show processing utilities.
///
/// This module is re-exported from [`formats::slides`](crate::formats::slides) for backward compatibility.
/// New code should use `formats::slides` directly.
pub mod slides {
    pub use crate::formats::slides::*;
}

/// Document chunking utilities.
///
/// This module is re-exported from [`parsing::chunks`](crate::parsing::chunks) for backward compatibility.
/// New code should use `parsing::chunks` directly.
pub mod chunks {
    pub use crate::parsing::chunks::*;
}

/// Source file management utilities.
///
/// This module is re-exported from [`parsing::sources`](crate::parsing::sources) for backward compatibility.
/// New code should use `parsing::sources` directly.
pub mod sources {
    pub use crate::parsing::sources::*;
}

/// Template system for document rendering.
///
/// This module provides a flexible template system for document rendering,
/// inspired by Pandoc's template architecture. Templates use a simple
/// variable substitution syntax similar to Pandoc's templates.
pub mod template;

/// Logging system for clmd.
///
/// This module is re-exported from [`context::logging`](crate::context::logging) for backward compatibility.
/// New code should use `context::logging` directly.
pub mod logging {
    pub use crate::context::logging::*;
}

/// Shared utility functions for document processing.
///
/// This module is re-exported from [`core::shared`](crate::core::shared) for backward compatibility.
/// New code should use `core::shared` directly.
pub mod shared {
    pub use crate::core::shared::*;
}

/// Version information for clmd.
///
/// This module is re-exported from [`context::version`](crate::context::version) for backward compatibility.
/// New code should use `context::version` directly.
pub mod version {
    pub use crate::context::version::*;
}

/// UUID generation utilities.
///
/// This module is re-exported from [`context::uuid`](crate::context::uuid) for backward compatibility.
/// New code should use `context::uuid` directly.
pub mod uuid {
    pub use crate::context::uuid::*;
}

/// Process management utilities.
///
/// This module is re-exported from [`context::process`](crate::context::process) for backward compatibility.
/// New code should use `context::process` directly.
pub mod process {
    pub use crate::context::process::*;
}

/// Data file access utilities.
///
/// This module is re-exported from [`context::data`](crate::context::data) for backward compatibility.
/// New code should use `context::data` directly.
pub mod data {
    pub use crate::context::data::*;
}

/// General parsing utilities and combinators.
///
/// This module provides general-purpose parsing utilities inspired by Pandoc's
/// parsing infrastructure. It includes parser combinators, character utilities,
/// and common parsing patterns.
pub mod parsing;

/// Pandoc-style options system with separate ReaderOptions and WriterOptions.
///
/// This module provides a more structured approach to configuration,
/// separating parsing options from rendering options.
///
/// # Example
///
/// ```ignore
/// use clmd::options::{Options, ReaderOptions, WriterOptions, InputFormat, OutputFormat};
///
/// let options = Options::new(
///     ReaderOptions::gfm(),
///     WriterOptions::default().with_output_format(OutputFormat::Html)
/// );
/// ```ignore
///
/// Document reader trait and implementations.
///
/// This module provides a unified interface for reading documents from different
/// formats, inspired by Pandoc's Reader system.
///
/// This is a re-export of [`readers`](crate::readers) for backward compatibility.
/// New code should use `readers` directly.
pub mod reader {
    pub use crate::readers::*;
}

/// Document writer trait and implementations.
///
/// This module provides a unified interface for writing documents to different
/// formats, inspired by Pandoc's Writer system.
///
/// This is a re-export of [`writers`](crate::writers) for backward compatibility.
/// New code should use `writers` directly.
pub mod writer {
    pub use crate::writers::*;
}

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
/// ```ignore
/// use clmd::{parse_document, Options};
///
/// let options = Options::default();
/// let (arena, root) = parse_document("# Hello\n\nWorld", &options);
/// ```ignore
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
/// ```ignore
/// use clmd::Options;
///
/// let mut options = Options::default();
/// options.extension.table = true;
/// options.render.hardbreaks = true;
/// ```ignore
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

pub use core::error::{
    ClmdError, ClmdResult, LimitKind, ParseError, ParseResult, ParserLimits, Position,
    Range,
};

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
    let (arena, root) = parser::parse_document(md, options);
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
    let flags = parser::options_to_flags(options);
    let html = html::render(arena, root, flags);
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
    let flags = parser::options_to_flags(options);
    let highlighter = plugins.render.syntax_highlighter();
    write!(
        output,
        "{}",
        html::render_with_highlighter(arena, root, flags, highlighter)
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
    render::typst::format_document_with_plugins(arena, root, _options, output, plugins)
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
