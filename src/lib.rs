//! A 100% CommonMark and GFM compatible Markdown parser.
//!
//! Source repository and detailed README is at
//! [github.com/kivikakk/comrak](https://github.com/kivikakk/comrak).
//!
//! You can use `clmd::markdown_to_html` directly:
//!
//! ```ignore
//! use clmd::{markdown_to_html, parser::options::Options};
//! let html = markdown_to_html("Hello, **world**!", &Options::default());
//! assert!(html.contains("<strong>world</strong>"));
//! ```
//!
//! Or you can parse the input into an AST yourself, manipulate it, and then use your desired
//! formatter:
//!
//! ```ignore
//! use clmd::{Arena, parse_document, format_html, parser::options::Options};
//!
//! let mut arena = Arena::new();
//! let options = Options::default();
//! let root = parse_document(&mut arena, "Hello, world!", &options);
//!
//! let mut html = String::new();
//! format_html(&arena, root, &options, &mut html).unwrap();
//! ```

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

/// Error types and parsing limits
pub mod error;

/// Block-level parsing for CommonMark documents
///
/// This module implements the block parsing algorithm based on the CommonMark spec.
/// It processes input line by line, building the AST structure using Arena allocation.
pub mod blocks;

/// Compatibility layer for different API versions
pub mod compat;

/// Configuration management (deprecated, use `parser::options` instead)
#[deprecated(since = "0.2.0", note = "Use `parser::options` instead")]
pub mod config;

/// Document converters (HTML, LaTeX, etc.)
pub mod converters;

/// Markdown extensions (GFM and others)
pub mod ext;

/// HTML to Markdown conversion
pub mod html_to_md;

/// String pool for efficient memory reuse
pub mod pool;

/// HTML utilities (escaping, entity decoding)
pub mod html_utils;

/// Inline parsing for CommonMark documents
///
/// This module implements the inline parsing algorithm based on the CommonMark spec.
/// It processes the content of leaf blocks to produce inline elements like
/// emphasis, links, code, etc.
pub mod inlines;

/// AST iteration and traversal
pub mod iterator;

/// Lexical analysis utilities
pub mod lexer;

/// Unified node value types (new API, inspired by comrak)
///
/// This module provides a unified `NodeValue` enum that combines node type and data,
/// offering better type safety and ergonomics compared to the separate `NodeType` and `NodeData` approach.
///
/// # Example
///
/// ```ignore
/// use clmd::node_value::{NodeValue, NodeHeading, NodeList, ListType};
///
/// let heading = NodeValue::Heading(NodeHeading {
///     level: 1,
///     setext: false,
///     closed: false,
/// });
/// ```
pub mod node_value;

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
pub mod sequence;

/// Test utilities
pub mod test_utils;

// =============================================================================
// Core Type Exports
// =============================================================================

/// Convenience type alias for arena used to hold nodes.
pub type Arena = arena::NodeArena;

pub use arena::{Node, NodeId, TreeOps};

/// Parse a Markdown document to an AST.
///
/// This is the main entry point for parsing. It takes an arena for node allocation,
/// the Markdown text to parse, and options for configuring the parser.
///
/// # Example
///
/// ```ignore
/// use clmd::{Arena, parse_document, parser::options::Options};
///
/// let mut arena = Arena::new();
/// let options = Options::default();
/// let root = parse_document(&mut arena, "# Hello\n\nWorld", &options);
/// ```
pub fn parse_document<'a>(
    arena: &'a mut Arena,
    md: &str,
    options: &parser::options::Options,
) -> NodeId {
    parser::parse_document(arena, md, options)
}

// =============================================================================
// Options Exports (comrak-style)
// =============================================================================

/// Re-export Options from parser::options
pub use parser::options::{
    BrokenLinkCallback, BrokenLinkReference, Extension, ListStyleType, Options as ParserOptions,
    Parse, Render, ResolvedReference, URLRewriter, WikiLinksMode,
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
// Node Value Exports
// =============================================================================

pub use node_value::{
    can_contain_type, AlertType, LineColumn, ListDelimType, ListType as NodeValueListType,
    NodeAlert, NodeCode, NodeCodeBlock, NodeDescriptionItem, NodeFootnoteDefinition,
    NodeFootnoteReference, NodeHeading, NodeHtmlBlock, NodeLink, NodeList, NodeMath,
    NodeMultilineBlockQuote, NodeTable, NodeTaskItem, NodeValue, NodeWikiLink, SourcePos,
    TableAlignment,
};

// =============================================================================
// Adapter Exports
// =============================================================================

pub use adapters::{
    AnchorHeadingAdapter, CodefenceRendererAdapter, DefaultSyntaxHighlighter, HeadingAdapter,
    HeadingMeta, SyntaxHighlighterAdapter, UrlRewriter,
};

// =============================================================================
// Plugin Exports
// =============================================================================

pub use plugins::{Plugins, RenderPlugins};

// =============================================================================
// Renderer Exports
// =============================================================================

pub use render::{
    render, render_to_commonmark, render_to_html, render_to_latex, render_to_man, render_to_xml,
    OutputFormat, Renderer,
};

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
fn options_to_flags(options: &parser::options::Options) -> u32 {
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
// Document Type
// =============================================================================

/// A parsed Markdown document
///
/// This type provides a high-level API for parsing and rendering Markdown.
/// It encapsulates the arena and root node, providing convenient methods
/// for rendering to various formats.
///
/// # Example
///
/// ```
/// use clmd::Document;
/// use clmd::parser::options::Options;
///
/// let doc = Document::parse("Hello *world*", &Options::default());
/// let html = doc.to_html(&Options::default());
/// assert!(html.contains("<em>world</em>"));
/// ```
#[derive(Debug)]
pub struct Document {
    arena: Arena,
    root: NodeId,
}

impl Document {
    /// Parse a Markdown document with default options
    ///
    /// # Arguments
    ///
    /// * `input` - The Markdown text to parse
    /// * `options` - Options for parsing
    ///
    /// # Returns
    ///
    /// The parsed document
    pub fn parse(input: &str, options: &parser::options::Options) -> Self {
        let mut arena = Arena::new();
        let root = parser::parse_document(&mut arena, input, options);
        Document { arena, root }
    }

    /// Render the document to HTML
    ///
    /// # Arguments
    ///
    /// * `options` - Options for rendering
    ///
    /// # Returns
    ///
    /// The HTML output as a String
    pub fn to_html(&self, options: &parser::options::Options) -> String {
        let flags = options_to_flags(options);
        render::html::render(&self.arena, self.root, flags)
    }

    /// Render the document to XML
    ///
    /// # Returns
    ///
    /// The XML output as a String
    pub fn to_xml(&self) -> String {
        render::xml::render(&self.arena, self.root, 0)
    }

    /// Render the document to CommonMark
    ///
    /// # Returns
    ///
    /// The CommonMark output as a String
    pub fn to_commonmark(&self) -> String {
        render::commonmark::render(&self.arena, self.root, 0)
    }

    /// Get the root node ID
    ///
    /// # Returns
    ///
    /// The `NodeId` of the document root node
    pub fn root(&self) -> NodeId {
        self.root
    }

    /// Get a reference to the arena
    ///
    /// # Returns
    ///
    /// A reference to the `Arena` containing all AST nodes
    pub fn arena(&self) -> &Arena {
        &self.arena
    }
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
/// ```ignore
/// use clmd::{markdown_to_html, parser::options::Options};
/// assert_eq!(
///     markdown_to_html("Hello, **world**!", &Options::default()),
///     "<p>Hello, <strong>world</strong>!</p>\n"
/// );
/// ```
pub fn markdown_to_html(md: &str, options: &parser::options::Options) -> String {
    let mut arena = Arena::new();
    let doc = parser::parse_document(&mut arena, md, options);
    let flags = options_to_flags(options);
    render::html::render(&arena, doc, flags)
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
/// use clmd::{markdown_to_html_with_plugins, parser::options::Options, plugins::Plugins};
///
/// let options = Options::default();
/// let plugins = Plugins::default();
/// let html = markdown_to_html_with_plugins("Hello, **world**!", &options, &plugins);
/// ```
pub fn markdown_to_html_with_plugins(
    md: &str,
    options: &parser::options::Options,
    plugins: &Plugins,
) -> String {
    // For now, delegate to the non-plugin version
    // TODO: Implement plugin support in renderers
    let _ = plugins;
    markdown_to_html(md, options)
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
/// use clmd::{markdown_to_commonmark, parser::options::Options};
///
/// let options = Options::default();
/// let cm = markdown_to_commonmark("Hello *world*", &options);
/// assert!(cm.contains("Hello"));
/// ```
pub fn markdown_to_commonmark(md: &str, options: &parser::options::Options) -> String {
    let mut arena = Arena::new();
    let doc = parser::parse_document(&mut arena, md, options);
    render::commonmark::render(&arena, doc, 0)
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
/// use clmd::{markdown_to_commonmark_xml, parser::options::Options};
///
/// let options = Options::default();
/// let xml = markdown_to_commonmark_xml("Hello *world*", &options);
/// assert!(xml.contains("<document>"));
/// ```
pub fn markdown_to_commonmark_xml(md: &str, options: &parser::options::Options) -> String {
    let mut arena = Arena::new();
    let doc = parser::parse_document(&mut arena, md, options);
    render::xml::render(&arena, doc, 0)
}

/// Format an existing AST to HTML.
///
/// # Arguments
///
/// * `arena` - The node arena containing the AST
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
/// use clmd::{Arena, parse_document, format_html, parser::options::Options};
///
/// let mut arena = Arena::new();
/// let options = Options::default();
/// let root = parse_document(&mut arena, "Hello *world*", &options);
/// let mut html = String::new();
/// format_html(&arena, root, &options, &mut html).unwrap();
/// ```
pub fn format_html(
    arena: &Arena,
    root: NodeId,
    options: &parser::options::Options,
    output: &mut dyn std::fmt::Write,
) -> std::fmt::Result {
    let flags = options_to_flags(options);
    let html = render::html::render(arena, root, flags);
    output.write_str(&html)
}

/// Format an existing AST to CommonMark.
///
/// # Arguments
///
/// * `arena` - The node arena containing the AST
/// * `root` - The root node ID
/// * `options` - Configuration options
/// * `output` - The output buffer to write to
///
/// # Returns
///
/// A `std::fmt::Result` indicating success or failure
pub fn format_commonmark(
    arena: &Arena,
    root: NodeId,
    _options: &parser::options::Options,
    output: &mut dyn std::fmt::Write,
) -> std::fmt::Result {
    let cm = render::commonmark::render(arena, root, 0);
    output.write_str(&cm)
}

/// Format an existing AST to XML.
///
/// # Arguments
///
/// * `arena` - The node arena containing the AST
/// * `root` - The root node ID
/// * `options` - Configuration options
/// * `output` - The output buffer to write to
///
/// # Returns
///
/// A `std::fmt::Result` indicating success or failure
pub fn format_xml(
    arena: &Arena,
    root: NodeId,
    _options: &parser::options::Options,
    output: &mut dyn std::fmt::Write,
) -> std::fmt::Result {
    let xml = render::xml::render(arena, root, 0);
    output.write_str(&xml)
}

/// Return the version of the crate.
///
/// # Returns
///
/// The version string
///
/// # Example
///
/// ```ignore
/// use clmd::version;
///
/// let version = version();
/// assert!(!version.is_empty());
/// ```
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use parser::options::Options;

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
        let mut arena = Arena::new();
        let doc = parse_document(&mut arena, input, &options);
        let mut html = String::new();
        format_html(&arena, doc, &options, &mut html).unwrap();
        assert!(html.contains("<h1>"));
        assert!(html.contains("Paragraph"));
    }

    #[test]
    fn test_version() {
        let v = version();
        assert!(!v.is_empty());
    }

    #[test]
    fn test_document_type() {
        let options = Options::default();
        let doc = Document::parse("Hello *world*", &options);
        let html = doc.to_html(&options);
        assert!(html.contains("<em>world</em>"));
    }
}
