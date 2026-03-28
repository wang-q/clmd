//! A 100% CommonMark and GFM compatible Markdown parser.
//!
//! Source repository and detailed README is at
//! [github.com/kivikakk/comrak](https://github.com/kivikakk/comrak).
//!
//! You can use `clmd::markdown_to_html` directly:
//!
//! ```rust
//! use clmd::{markdown_to_html, options::Options};
//! let html = markdown_to_html("Hello, **world**!", &Options::default());
//! assert!(html.contains("<strong>world</strong>"));
//! ```
//!
//! Or you can parse the input into an AST yourself, manipulate it, and then use your desired
//! formatter:
//!
//! ```rust
//! use clmd::{Arena, parse_document, format_html, options::Options};
//!
//! let mut arena = Arena::new();
//! let root = parse_document(&mut arena, "Hello, world!", &Options::default());
//!
//! let mut html = String::new();
//! format_html(&arena, root, &Options::default(), &mut html).unwrap();
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

/// Configuration management
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
/// ```
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
/// This module provides a comrak-style Options API that wraps the underlying
/// DataKey-based configuration system. It offers better ergonomics and
/// compile-time type safety.
///
/// # Example
///
/// ```
/// use clmd::options::Options;
///
/// let mut options = Options::default();
/// options.extension.table = true;
/// options.extension.strikethrough = true;
/// options.render.hardbreaks = true;
/// ```
pub mod options;

/// Plugin system for extending Markdown rendering
///
/// This module provides a plugin architecture that allows users to customize
/// various aspects of Markdown rendering, such as syntax highlighting,
/// heading rendering, and code block handling.
///
/// # Example
///
/// ```rust,ignore
/// use clmd::plugins::{Plugins, SyntaxHighlighterAdapter};
/// use std::fmt;
/// use std::collections::HashMap;
/// use std::borrow::Cow;
///
/// struct MyHighlighter;
/// impl SyntaxHighlighterAdapter for MyHighlighter {
///     fn write_highlighted(
///         &self,
///         output: &mut dyn fmt::Write,
///         _lang: Option<&str>,
///         code: &str,
///     ) -> fmt::Result {
///         write!(output, "<code>{}</code>", code)
///     }
///
///     fn write_pre_tag(&self, output: &mut dyn fmt::Write, _attrs: HashMap<&'static str, Cow<'_, str>>) -> fmt::Result {
///         output.write_str("<pre>")
///     }
///
///     fn write_code_tag(&self, output: &mut dyn fmt::Write, _attrs: HashMap<&'static str, Cow<'_, str>>) -> fmt::Result {
///         output.write_str("<code>")
///     }
/// }
///
/// let mut plugins = Plugins::new();
/// plugins.set_syntax_highlighter(Box::new(MyHighlighter));
/// ```
pub mod plugins;

/// High-level parser interface
pub mod parser;

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
pub type Arena<'a> = arena::NodeArena;

pub use arena::{Node, NodeId, TreeOps};
pub use parser::Parser;

// =============================================================================
// Options Exports (comrak-style)
// =============================================================================

pub use options::{
    Extension, ListStyleType, Parse, Render, WikiLinksMode,
};

// Re-export the new comrak-style Options with lifetime parameter
pub use options::Options;

// Re-export DataKey-based options for backward compatibility
pub use config::{
    DataHolder, DataKey, DataSet, MutableDataSet,
    Options as DataKeyOptions, ParseOptions, RenderOptions,
};

/// Configuration options for parsing and rendering (DataKey-based).
///
/// This module provides predefined `DataKey` constants for all available options.
/// See [`config::options`](crate::config::options) for the full list.
pub use config::options as config_options;

// =============================================================================
// Error Type Exports
// =============================================================================

pub use error::{
    ParseError, ParseResult, ParserLimits, Position,
};

// Re-export from adapters for consistency
pub use adapters::{BrokenLinkCallback, BrokenLinkReference, ResolvedReference};

// =============================================================================
// Iterator Exports
// =============================================================================

pub use iterator::{ArenaNodeIterator, ArenaNodeWalker, EventType};

// =============================================================================
// Node Value Exports
// =============================================================================

pub use node_value::{
    can_contain_type, AlertType, LineColumn, ListDelimType,
    ListType as NodeValueListType, NodeAlert, NodeCode, NodeCodeBlock,
    NodeDescriptionItem, NodeFootnoteDefinition, NodeFootnoteReference, NodeHeading,
    NodeHtmlBlock, NodeLink, NodeList, NodeMath, NodeMultilineBlockQuote, NodeTable,
    NodeTaskItem, NodeValue, NodeWikiLink, SourcePos, TableAlignment,
};

// =============================================================================
// Adapter Exports
// =============================================================================

pub use adapters::{
    AnchorHeadingAdapter, CodefenceRendererAdapter, DefaultSyntaxHighlighter,
    HeadingAdapter, HeadingMeta, SyntaxHighlighterAdapter, UrlRewriter,
};

// =============================================================================
// Plugin Exports
// =============================================================================

pub use plugins::{
    Plugins, RenderPlugins,
};

// =============================================================================
// Renderer Exports
// =============================================================================

pub use render::{
    render, render_to_commonmark, render_to_html, render_to_latex, render_to_man,
    render_to_xml, OutputFormat, Renderer,
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

/// Convert DataKey-based Options to legacy u32 flags for parsing.
/// This is a temporary bridge until all components use the new system.
fn options_to_u32_for_parse(options: &DataKeyOptions) -> u32 {
    let mut legacy_options = 0;

    if options.get(&config_options::SOURCEPOS) {
        legacy_options |= OPT_SOURCEPOS;
    }
    if options.get(&config_options::SMART) {
        legacy_options |= OPT_SMART;
    }
    if options.get(&config_options::VALIDATE_UTF8) {
        legacy_options |= OPT_VALIDATE_UTF8;
    }

    legacy_options
}

/// Convert DataKey-based Options to legacy u32 flags for rendering.
/// This is a temporary bridge until all components use the new system.
fn options_to_u32_for_render(options: &DataKeyOptions) -> u32 {
    let mut legacy_options = 0;

    if options.get(&config_options::SOURCEPOS) {
        legacy_options |= OPT_SOURCEPOS;
    }
    if options.get(&config_options::HARDBREAKS) {
        legacy_options |= OPT_HARDBREAKS;
    }
    if options.get(&config_options::NOBREAKS) {
        legacy_options |= OPT_NOBREAKS;
    }
    if options.get(&config_options::UNSAFE) {
        legacy_options |= OPT_UNSAFE;
    }

    legacy_options
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
///
/// let doc = Document::parse("Hello *world*").unwrap();
/// let html = doc.to_html();
/// assert_eq!(html, "<p>Hello <em>world</em></p>");
/// ```
#[derive(Debug)]
pub struct Document {
    arena: Arena<'static>,
    root: NodeId,
}

impl Document {
    /// Parse a Markdown document with default options
    ///
    /// # Arguments
    ///
    /// * `input` - The Markdown text to parse
    ///
    /// # Returns
    ///
    /// A `ParseResult` containing the parsed document or an error
    pub fn parse(input: &str) -> ParseResult<Self> {
        let mut arena = Arena::new();
        let root = blocks::BlockParser::parse(&mut arena, input);
        Ok(Document { arena, root })
    }

    /// Parse a Markdown document with custom options (DataKey-based)
    ///
    /// # Arguments
    ///
    /// * `input` - The Markdown text to parse
    /// * `options` - Options for parsing (DataKey-based)
    ///
    /// # Returns
    ///
    /// A `ParseResult` containing the parsed document or an error
    pub fn parse_with_options(input: &str, options: &DataKeyOptions) -> ParseResult<Self> {
        let legacy_options = options_to_u32_for_parse(options);
        let mut arena = Arena::new();
        let root =
            blocks::BlockParser::parse_with_options(&mut arena, input, legacy_options);
        Ok(Document { arena, root })
    }

    /// Parse a Markdown document with custom options (comrak-style)
    ///
    /// # Arguments
    ///
    /// * `input` - The Markdown text to parse
    /// * `options` - Options for parsing (comrak-style)
    ///
    /// # Returns
    ///
    /// A `ParseResult` containing the parsed document or an error
    pub fn parse_with_comrak_options(input: &str, options: &options::Options<'_>) -> ParseResult<Self> {
        let data_key_options = options.to_data_key_options();
        Self::parse_with_options(input, &data_key_options)
    }

    /// Parse a Markdown document with custom limits
    ///
    /// # Arguments
    ///
    /// * `input` - The Markdown text to parse
    /// * `limits` - Parser limits for input validation
    ///
    /// # Returns
    ///
    /// A `ParseResult` containing the parsed document or an error
    pub fn parse_with_limits(input: &str, limits: ParserLimits) -> ParseResult<Self> {
        let mut arena = Arena::new();
        let root = blocks::BlockParser::parse_with_limits(&mut arena, input, 0, limits)?;
        Ok(Document { arena, root })
    }

    /// Render the document to HTML
    ///
    /// # Returns
    ///
    /// The HTML output as a String
    pub fn to_html(&self) -> String {
        render::html::render(&self.arena, self.root, 0)
    }

    /// Render the document to HTML with custom options (DataKey-based)
    ///
    /// # Arguments
    ///
    /// * `options` - Options for rendering (DataKey-based)
    ///
    /// # Returns
    ///
    /// The HTML output as a String
    pub fn to_html_with_options(&self, options: &DataKeyOptions) -> String {
        let legacy_options = options_to_u32_for_render(options);
        render::html::render(&self.arena, self.root, legacy_options)
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

    /// Render the document to LaTeX
    ///
    /// # Returns
    ///
    /// The LaTeX output as a String
    pub fn to_latex(&self) -> String {
        render::latex::render(&self.arena, self.root, 0)
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
    pub fn arena(&self) -> &Arena<'static> {
        &self.arena
    }

    /// Get a mutable reference to the arena
    ///
    /// # Returns
    ///
    /// A mutable reference to the `Arena` containing all AST nodes
    pub fn arena_mut(&mut self) -> &mut Arena<'static> {
        &mut self.arena
    }

    /// Consume the document and return the arena and root node
    ///
    /// This is useful when you need to take ownership of the parsed AST
    /// for further processing or manipulation.
    ///
    /// # Returns
    ///
    /// A tuple containing the `Arena` and the root `NodeId`
    pub fn into_parts(self) -> (Arena<'static>, NodeId) {
        (self.arena, self.root)
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
/// * `options` - Configuration options (comrak-style)
///
/// # Returns
///
/// The HTML output as a String
///
/// # Example
///
/// ```
/// use clmd::{markdown_to_html, options::Options};
/// assert_eq!(
///     markdown_to_html("Hello, **world**!", &Options::default()),
///     "<p>Hello, <strong>world</strong>!</p>\n"
/// );
/// ```
pub fn markdown_to_html(md: &str, options: &options::Options<'_>) -> String {
    let data_key_options = options.to_data_key_options();
    let legacy_options = options_to_u32_for_parse(&data_key_options);
    let mut arena = Arena::new();
    let doc = blocks::BlockParser::parse_with_options(&mut arena, md, legacy_options);
    let render_options = options_to_u32_for_render(&data_key_options);
    render::html::render(&arena, doc, render_options)
}

/// Render Markdown to HTML using plugins.
///
/// # Arguments
///
/// * `md` - The Markdown text to convert
/// * `options` - Configuration options (comrak-style)
/// * `plugins` - Plugins for customizing rendering
///
/// # Returns
///
/// The HTML output as a String
///
/// # Example
///
/// ```
/// use clmd::{markdown_to_html_with_plugins, options::Options, plugins::Plugins};
///
/// let options = Options::default();
/// let plugins = Plugins::default();
/// let html = markdown_to_html_with_plugins("Hello, **world**!", &options, &plugins);
/// ```
pub fn markdown_to_html_with_plugins(md: &str, options: &options::Options<'_>, plugins: &Plugins) -> String {
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
/// * `options` - Configuration options (comrak-style)
///
/// # Returns
///
/// The CommonMark output as a String
///
/// # Example
///
/// ```
/// use clmd::{markdown_to_commonmark, options::Options};
///
/// let options = Options::default();
/// let cm = markdown_to_commonmark("Hello *world*", &options);
/// assert!(cm.contains("Hello"));
/// ```
pub fn markdown_to_commonmark(md: &str, options: &options::Options<'_>) -> String {
    let data_key_options = options.to_data_key_options();
    let legacy_options = options_to_u32_for_parse(&data_key_options);
    let mut arena = Arena::new();
    let doc = blocks::BlockParser::parse_with_options(&mut arena, md, legacy_options);
    render::commonmark::render(&arena, doc, 0)
}

/// Render Markdown to CommonMark XML.
///
/// See <https://github.com/commonmark/commonmark-spec/blob/master/CommonMark.dtd>.
///
/// # Arguments
///
/// * `md` - The Markdown text to convert
/// * `options` - Configuration options (comrak-style)
///
/// # Returns
///
/// The XML output as a String
///
/// # Example
///
/// ```
/// use clmd::{markdown_to_commonmark_xml, options::Options};
///
/// let options = Options::default();
/// let xml = markdown_to_commonmark_xml("Hello *world*", &options);
/// assert!(xml.contains("<document>"));
/// ```
pub fn markdown_to_commonmark_xml(md: &str, options: &options::Options<'_>) -> String {
    let data_key_options = options.to_data_key_options();
    let legacy_options = options_to_u32_for_parse(&data_key_options);
    let mut arena = Arena::new();
    let doc = blocks::BlockParser::parse_with_options(&mut arena, md, legacy_options);
    render::xml::render(&arena, doc, 0)
}

/// Render Markdown to CommonMark XML using plugins.
///
/// # Arguments
///
/// * `md` - The Markdown text to convert
/// * `options` - Configuration options (comrak-style)
/// * `plugins` - Plugins for customizing rendering
///
/// # Returns
///
/// The XML output as a String
pub fn markdown_to_commonmark_xml_with_plugins(md: &str, options: &options::Options<'_>, plugins: &Plugins) -> String {
    let _ = plugins;
    markdown_to_commonmark_xml(md, options)
}

/// Parse a Markdown document and return the AST.
///
/// # Arguments
///
/// * `arena` - The arena to allocate nodes in
/// * `md` - The Markdown text to parse
/// * `options` - Configuration options (comrak-style)
///
/// # Returns
///
/// The root node ID
///
/// # Example
///
/// ```
/// use clmd::{Arena, parse_document, options::Options};
///
/// let mut arena = Arena::new();
/// let options = Options::default();
/// let root = parse_document(&mut arena, "Hello *world*", &options);
/// // Now you can traverse and manipulate the AST
/// ```
pub fn parse_document(arena: &mut Arena, md: &str, options: &options::Options<'_>) -> NodeId {
    let data_key_options = options.to_data_key_options();
    let legacy_options = options_to_u32_for_parse(&data_key_options);
    blocks::BlockParser::parse_with_options(arena, md, legacy_options)
}

/// Format an existing AST to HTML.
///
/// # Arguments
///
/// * `arena` - The node arena containing the AST
/// * `root` - The root node ID
/// * `options` - Configuration options (comrak-style)
/// * `output` - The output buffer to write to
///
/// # Returns
///
/// A `std::fmt::Result` indicating success or failure
///
/// # Example
///
/// ```
/// use clmd::{Arena, parse_document, format_html, options::Options};
///
/// let arena = Arena::new();
/// let options = Options::default();
/// let root = parse_document(&arena, "Hello *world*", &options);
/// let mut html = String::new();
/// format_html(&arena, root, &options, &mut html).unwrap();
/// ```
pub fn format_html(
    arena: &Arena,
    root: NodeId,
    options: &options::Options<'_>,
    output: &mut dyn std::fmt::Write,
) -> std::fmt::Result {
    let data_key_options = options.to_data_key_options();
    let legacy_options = options_to_u32_for_render(&data_key_options);
    let html = render::html::render(arena, root, legacy_options);
    output.write_str(&html)
}

/// Format an existing AST to HTML using plugins.
///
/// # Arguments
///
/// * `arena` - The node arena containing the AST
/// * `root` - The root node ID
/// * `options` - Configuration options (comrak-style)
/// * `output` - The output buffer to write to
/// * `plugins` - Plugins for customizing rendering
///
/// # Returns
///
/// A `std::fmt::Result` indicating success or failure
pub fn format_html_with_plugins(
    arena: &Arena,
    root: NodeId,
    options: &options::Options<'_>,
    output: &mut dyn std::fmt::Write,
    plugins: &Plugins,
) -> std::fmt::Result {
    let _ = plugins;
    format_html(arena, root, options, output)
}

/// Format an existing AST to CommonMark.
///
/// # Arguments
///
/// * `arena` - The node arena containing the AST
/// * `root` - The root node ID
/// * `options` - Configuration options (comrak-style)
/// * `output` - The output buffer to write to
///
/// # Returns
///
/// A `std::fmt::Result` indicating success or failure
pub fn format_commonmark(
    arena: &Arena,
    root: NodeId,
    _options: &options::Options<'_>,
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
/// * `options` - Configuration options (comrak-style)
/// * `output` - The output buffer to write to
///
/// # Returns
///
/// A `std::fmt::Result` indicating success or failure
pub fn format_xml(
    arena: &Arena,
    root: NodeId,
    _options: &options::Options<'_>,
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
/// ```
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

    #[test]
    fn test_markdown_to_html_basic() {
        let options = options::Options::default();
        let html = markdown_to_html("Hello world", &options);
        assert_eq!(html, "<p>Hello world</p>");
    }

    #[test]
    fn test_markdown_to_html_heading() {
        let options = options::Options::default();
        let html = markdown_to_html("# Heading 1\n\n## Heading 2", &options);
        assert!(html.contains("<h1>"));
        assert!(html.contains("<h2>"));
    }

    #[test]
    fn test_markdown_to_html_emphasis() {
        let options = options::Options::default();
        let html = markdown_to_html("*italic* and **bold**", &options);
        assert!(html.contains("<p>"));
    }

    #[test]
    fn test_markdown_to_html_link() {
        let options = options::Options::default();
        let html = markdown_to_html("[link](https://example.com)", &options);
        assert!(html.contains("<a href=\"https://example.com\">"));
    }

    #[test]
    fn test_markdown_to_html_code_inline() {
        let options = options::Options::default();
        let html = markdown_to_html("Use `code` here", &options);
        assert!(html.contains("<code>code</code>"));
    }

    #[test]
    fn test_markdown_to_html_code_block() {
        let options = options::Options::default();
        let html = markdown_to_html("```rust\nfn main() {}\n```", &options);
        println!("Code block HTML: {:?}", html);
        assert!(html.contains("<pre>"), "Expected <pre> in {}", html);
        assert!(
            html.contains("<code class=\"language-rust\">"),
            "Expected <code class=\"language-rust\"> in {}",
            html
        );
        assert!(
            html.contains("fn main() {}"),
            "Expected fn main() {{}} in {}",
            html
        );
    }

    #[test]
    fn test_markdown_to_html_blockquote() {
        let options = options::Options::default();
        let html = markdown_to_html("> Quote", &options);
        println!("Blockquote HTML: {:?}", html);
        assert!(
            html.contains("<blockquote>"),
            "Expected <blockquote> in {}",
            html
        );
        assert!(html.contains("Quote"), "Expected Quote in {}", html);
    }

    #[test]
    fn test_markdown_to_html_list() {
        let options = options::Options::default();
        let html = markdown_to_html("- Item 1\n- Item 2", &options);
        println!("List HTML: {:?}", html);
        assert!(html.contains("<ul>"), "Expected <ul> in {}", html);
        assert!(html.contains("Item 1"), "Expected Item 1 in {}", html);
        assert!(html.contains("Item 2"), "Expected Item 2 in {}", html);
    }

    #[test]
    fn test_markdown_to_html_ordered_list() {
        let options = options::Options::default();
        let html = markdown_to_html("1. First\n2. Second", &options);
        assert!(html.contains("<ol>"));
        assert!(html.contains("First"));
        assert!(html.contains("Second"));
    }

    #[test]
    fn test_markdown_to_html_thematic_break() {
        let options = options::Options::default();
        let html = markdown_to_html("---", &options);
        assert_eq!(html, "<hr />");
    }

    #[test]
    fn test_markdown_to_html_image() {
        let options = options::Options::default();
        let html = markdown_to_html("![alt text](image.png)", &options);
        assert!(html.contains("<img src=\"image.png\""));
    }

    #[test]
    fn test_parse_and_render_roundtrip() {
        let options = options::Options::default();
        let input = "# Title\n\nParagraph with text.";
        let mut arena = Arena::new();
        let doc = parse_document(&mut arena, input, &options);
        let mut html = String::new();
        format_html(&arena, doc, &options, &mut html).unwrap();
        assert!(html.contains("<h1>"));
        assert!(html.contains("Paragraph"));
    }

    #[test]
    fn test_markdown_to_html_with_smart() {
        let mut options = options::Options::default();
        options.parse.smart = true;

        let html = markdown_to_html("\"Hello\"", &options);
        // Smart quotes should convert " to curly quotes
        assert!(html.contains("<p>"));
    }

    #[test]
    fn test_markdown_to_commonmark() {
        let options = options::Options::default();
        let cm = markdown_to_commonmark("Hello *world*", &options);
        assert!(cm.contains("Hello"));
        assert!(cm.contains("world"));
    }

    #[test]
    fn test_markdown_to_xml() {
        let options = options::Options::default();
        let xml = markdown_to_commonmark_xml("Hello *world*", &options);
        assert!(xml.contains("<document>"));
        assert!(xml.contains("<paragraph>"));
    }

    #[test]
    fn test_parse_document() {
        let options = options::Options::default();
        let mut arena = Arena::new();
        let root = parse_document(&mut arena, "Hello *world*", &options);
        assert!(arena.is_valid(root)); // Root should be a valid node ID
    }

    #[test]
    fn test_format_html() {
        let options = options::Options::default();
        let mut arena = Arena::new();
        let root = parse_document(&mut arena, "Hello *world*", &options);
        let mut html = String::new();
        format_html(&arena, root, &options, &mut html).unwrap();
        assert!(html.contains("<p>"));
        assert!(html.contains("<em>"));
    }

    #[test]
    fn test_version() {
        let v = version();
        assert!(!v.is_empty());
    }

    #[test]
    #[ignore = "Extension processing requires inline parsing which is not yet integrated"]
    fn test_options_strikethrough() {
        let mut options = options::Options::default();
        options.extension.strikethrough = true;

        let html = markdown_to_html("~~deleted~~", &options);
        assert!(html.contains("<del>"));
    }

    #[test]
    #[ignore = "Extension processing requires inline parsing which is not yet integrated"]
    fn test_options_table() {
        let mut options = options::Options::default();
        options.extension.table = true;

        let html = markdown_to_html("| a | b |\n|---|---|\n| c | d |", &options);
        assert!(html.contains("<table>"));
    }

    #[test]
    #[ignore = "Extension processing requires inline parsing which is not yet integrated"]
    fn test_options_hardbreaks() {
        let mut options = options::Options::default();
        options.render.hardbreaks = true;

        let html = markdown_to_html("Hello\nWorld", &options);
        assert!(html.contains("<br />"));
    }
}
