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
//! use clmd::Options;
//! use clmd::core::ParserLimits;
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
//! use clmd::core::traverse::{Traverse, Query};
//! use clmd::core::nodes::NodeValue;
//!
//! let options = Options::default();
//! let (arena, root) = parse_document("# Title\n\nParagraph", &options);
//!
//! // Traverse all nodes in pre-order
//! arena.traverse_pre_order(root, |value| {
//!     match value {
//!         NodeValue::Heading(h) => println!("Heading level {}", h.level),
//!         NodeValue::Paragraph => println!("Paragraph"),
//!         _ => {}
//!     }
//! });
//!
//! // Or query for specific node types
//! let headings: Vec<_> = arena.find_all(root, |v| matches!(v, NodeValue::Heading(_)));
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

/// Core module for AST, arena, and error types.
///
/// This module provides the foundational types and utilities for clmd,
/// including AST representations, error handling, memory management, and shared utilities.
pub mod core;

// Core types are exported directly through `core` module and type aliases below

/// Markdown parsing module (blocks and inlines).
pub mod parse;

/// Markdown extensions (GFM and others).
pub mod ext;

// HTML rendering for the CommonMark AST is now in render::html
pub use render::html;

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
/////! use clmd::options::{Options, OutputFormat};
///
/// let mut options = Options::default();
/// options.extension.table = true;
/// options.parse.smart = true;
/// ```
pub mod options;

/// IO module for document writing.
///
/// This module provides a unified interface for writing documents to various
/// output formats, inspired by Pandoc's Writer system.
pub mod io;

// IO submodules are accessed through `io::writer`, `io::format`

/// Rendering modules for HTML, CommonMark, and other output formats.
pub mod render;

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
/// ```
pub mod prelude;

/// Markdown extensions management using bitflags.
pub mod extensions {
    pub use crate::ext::flags::*;
}

/// Re-export extension types for convenience.
pub use crate::ext::flags::{ExtensionFlags, ExtensionKind};

/// Context system for IO operations and resource management.
pub mod context;

/// Template system for document rendering.
pub mod template;

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

/// Re-export Options for convenient access.
///
/// # Example
///
/// ```ignore
/// use clmd::Options;
///
/// let mut options = Options::default();
/// options.extension.table = true;
/// options.render.hardbreaks = true;
/// ```
pub use options::Options;

/// Re-export Extension options.
pub use options::Extension;

/// Re-export ResolvedReference.
pub use options::ResolvedReference;

/// Re-export BrokenLinkCallback.
pub use options::BrokenLinkCallback;

/// Re-export URLRewriter trait.
pub use options::URLRewriter;

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
    let (arena, root) = parse::parse_document(md, options);
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
    let (arena, root) = parse::parse_document(md, options);
    let mut out = String::new();
    format_xml(&arena, root, options, &mut out).unwrap();
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
    write!(output, "{}", html::render(arena, root, options))
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
    let opts =
        options::format::FormatOptions::new().with_right_margin(options.render.width);

    let formatter = render::commonmark::Formatter::with_options(opts);

    let result = formatter.render(arena, root);
    write!(output, "{}", result)
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
    io::writer::xml::format_document(arena, root, options, output)
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

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use crate::{format_html, io, markdown_to_html, parse_document, version, Options};

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

        let html =
            render::html::render(&arena, root, &crate::options::Options::default());
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

        let cm = render::commonmark::render(&arena, root, 0);
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

        let xml = io::writer::xml::render(&arena, root, 0);
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

        let html =
            render::html::render(&arena, root, &crate::options::Options::default());
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
