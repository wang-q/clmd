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

/// Plugin system for extending Markdown rendering
///
/// This module provides a plugin architecture that allows users to customize
/// various aspects of Markdown rendering, such as syntax highlighting,
/// heading rendering, and code block handling.
///
/// # Example
///
/// ```
/// use clmd::plugins::{Plugins, SyntaxHighlighterAdapter};
///
/// struct MyHighlighter;
/// impl SyntaxHighlighterAdapter for MyHighlighter {
///     fn highlight(&self, code: &str, lang: Option<&str>) -> String {
///         format!("<pre><code>{}</code></pre>", code)
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

pub use arena::{Node, NodeArena, NodeId, TreeOps};
pub use config::{
    DataHolder, DataKey, DataSet, MutableDataSet, Options, ParseOptions, RenderOptions,
};
pub use error::{
    BrokenLinkCallback, BrokenLinkReference, DefaultBrokenLinkCallback, ParseError,
    ParseResult, ParserLimits, Position, ResolvedReference,
};
pub use iterator::{ArenaNodeIterator, ArenaNodeWalker, EventType};
pub use parser::Parser;

// Re-export new node_value types
pub use node_value::{
    can_contain_type, AlertType, LineColumn, ListDelimType,
    ListType as NodeValueListType, NodeAlert, NodeCode, NodeCodeBlock,
    NodeDescriptionItem, NodeFootnoteDefinition, NodeFootnoteReference, NodeHeading,
    NodeHtmlBlock, NodeLink, NodeList, NodeMath, NodeMultilineBlockQuote, NodeTable,
    NodeTaskItem, NodeValue, NodeWikiLink, SourcePos, TableAlignment,
};

// Re-export plugin types
pub use plugins::{
    AnchorHeadingAdapter, CodefenceRendererAdapter, DefaultSyntaxHighlighter,
    HeadingAdapter, Plugins, SyntaxHighlighterAdapter, UrlRewriter,
};

// Re-export renderer types
pub use render::{
    render, render_to_commonmark, render_to_html, render_to_latex, render_to_man,
    render_to_xml, OutputFormat, Renderer,
};

/// Configuration options for parsing and rendering.
///
/// This module provides predefined `DataKey` constants for all available options.
/// See [`config::options`](crate::config::options) for the full list.
// Re-export the options module from config
pub use config::options as config_options;

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
    arena: NodeArena,
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
        Self::parse_with_options(input, options::DEFAULT)
    }

    /// Parse a Markdown document with custom options
    ///
    /// # Arguments
    ///
    /// * `input` - The Markdown text to parse
    /// * `options` - Options for parsing
    ///
    /// # Returns
    ///
    /// A `ParseResult` containing the parsed document or an error
    pub fn parse_with_options(input: &str, options: u32) -> ParseResult<Self> {
        let mut arena = NodeArena::new();
        let root = blocks::BlockParser::parse_with_options(&mut arena, input, options);
        Ok(Document { arena, root })
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
        let mut arena = NodeArena::new();
        let root = blocks::BlockParser::parse_with_limits(
            &mut arena,
            input,
            options::DEFAULT,
            limits,
        )?;
        Ok(Document { arena, root })
    }

    /// Render the document to HTML
    ///
    /// # Returns
    ///
    /// The HTML output as a String
    pub fn to_html(&self) -> String {
        render::html::render(&self.arena, self.root, options::DEFAULT)
    }

    /// Render the document to HTML with custom options
    ///
    /// # Arguments
    ///
    /// * `options` - Options for rendering
    ///
    /// # Returns
    ///
    /// The HTML output as a String
    pub fn to_html_with_options(&self, options: u32) -> String {
        render::html::render(&self.arena, self.root, options)
    }

    /// Render the document to XML
    ///
    /// # Returns
    ///
    /// The XML output as a String
    pub fn to_xml(&self) -> String {
        render::xml::render(&self.arena, self.root, options::DEFAULT)
    }

    /// Render the document to CommonMark
    ///
    /// # Returns
    ///
    /// The CommonMark output as a String
    pub fn to_commonmark(&self) -> String {
        render::commonmark::render(&self.arena, self.root, options::DEFAULT)
    }

    /// Render the document to LaTeX
    ///
    /// # Returns
    ///
    /// The LaTeX output as a String
    pub fn to_latex(&self) -> String {
        render::latex::render(&self.arena, self.root, options::DEFAULT)
    }

    /// Get the root node ID
    ///
    /// # Returns
    ///
    /// The `NodeId` of the document root node
    ///
    /// # Example
    ///
    /// ```
    /// use clmd::Document;
    ///
    /// let doc = Document::parse("Hello world").unwrap();
    /// let root = doc.root();
    /// ```
    pub fn root(&self) -> NodeId {
        self.root
    }

    /// Get a reference to the arena
    ///
    /// # Returns
    ///
    /// A reference to the `NodeArena` containing all AST nodes
    ///
    /// # Example
    ///
    /// ```
    /// use clmd::Document;
    ///
    /// let doc = Document::parse("Hello world").unwrap();
    /// let arena = doc.arena();
    /// ```
    pub fn arena(&self) -> &NodeArena {
        &self.arena
    }

    /// Get a mutable reference to the arena
    ///
    /// # Returns
    ///
    /// A mutable reference to the `NodeArena` containing all AST nodes
    ///
    /// # Example
    ///
    /// ```
    /// use clmd::Document;
    ///
    /// let mut doc = Document::parse("Hello world").unwrap();
    /// let arena = doc.arena_mut();
    /// ```
    pub fn arena_mut(&mut self) -> &mut NodeArena {
        &mut self.arena
    }

    /// Consume the document and return the arena and root node
    ///
    /// This is useful when you need to take ownership of the parsed AST
    /// for further processing or manipulation.
    ///
    /// # Returns
    ///
    /// A tuple containing the `NodeArena` and the root `NodeId`
    ///
    /// # Example
    ///
    /// ```
    /// use clmd::Document;
    ///
    /// let doc = Document::parse("Hello world").unwrap();
    /// let (arena, root) = doc.into_parts();
    /// ```
    pub fn into_parts(self) -> (NodeArena, NodeId) {
        (self.arena, self.root)
    }
}

/// Legacy options for parsing and rendering (deprecated, use `config::options` instead).
///
/// # Deprecated
///
/// These bit-flag options are deprecated in favor of the type-safe `DataKey` system
/// in [`config::options`](crate::config::options).
///
/// # Migration Guide
///
/// Instead of:
/// ```rust,ignore
/// use clmd::{markdown_to_html, options};
/// let html = markdown_to_html("Hello", options::SMART | options::HARDBREAKS);
/// ```
///
/// Use:
/// ```rust,ignore
/// use clmd::config::options::{Options, SMART, HARDBREAKS};
/// let mut options = Options::new();
/// options.set(&SMART, true);
/// options.set(&HARDBREAKS, true);
/// let html = markdown_to_html_with_options("Hello", &options);
/// ```
pub mod options {
    /// Default options (no flags set).
    pub const DEFAULT: u32 = 0;

    /// Include a `data-sourcepos` attribute on all block elements.
    pub const SOURCEPOS: u32 = 1 << 0;

    /// Render `softbreak` elements as hard line breaks.
    pub const HARDBREAKS: u32 = 1 << 1;

    /// Render `softbreak` elements as spaces.
    pub const NOBREAKS: u32 = 1 << 2;

    /// Validate UTF-8 in the input before parsing.
    pub const VALIDATE_UTF8: u32 = 1 << 3;

    /// Convert straight quotes to curly, `---` to em dashes, `--` to en dashes.
    pub const SMART: u32 = 1 << 4;

    /// Render raw HTML and unsafe links.
    pub const UNSAFE: u32 = 1 << 5;
}

/// Simple interface: convert Markdown to HTML
///
/// # Arguments
///
/// * `text` - The Markdown text to convert
/// * `options` - Options for parsing and rendering (e.g., `options::SMART`)
///
/// # Returns
///
/// The HTML output as a String
///
/// # Example
///
/// ```
/// use clmd::markdown_to_html;
/// use clmd::options;
///
/// let html = markdown_to_html("Hello *world*", options::DEFAULT);
/// assert_eq!(html, "<p>Hello <em>world</em></p>");
/// ```
pub fn markdown_to_html(text: &str, options: u32) -> String {
    let mut arena = NodeArena::new();
    let doc = blocks::BlockParser::parse_with_options(&mut arena, text, options);

    render::html::render(&arena, doc, options)
}

/// Parse a CommonMark document
///
/// This is a lower-level API that gives you direct access to the arena and root node.
/// For a higher-level interface, use [`Document::parse`](crate::Document::parse).
///
/// # Arguments
///
/// * `text` - The Markdown text to parse
/// * `options` - Options for parsing (use `options::DEFAULT` for default behavior)
///
/// # Returns
///
/// A tuple of (`NodeArena`, `NodeId`) where:
/// - `NodeArena` contains all the parsed AST nodes
/// - `NodeId` is the ID of the root document node
///
/// # Example
///
/// ```
/// use clmd::{parse_document, render_html, options};
///
/// let (arena, root) = parse_document("# Hello\n\nWorld", options::DEFAULT);
/// let html = render_html(&arena, root, options::DEFAULT);
/// assert!(html.contains("<h1>Hello</h1>"));
/// ```
pub fn parse_document(text: &str, options: u32) -> (NodeArena, NodeId) {
    let mut arena = NodeArena::new();
    let doc = blocks::BlockParser::parse_with_options(&mut arena, text, options);

    (arena, doc)
}

/// Render an Arena-based AST to HTML
///
/// # Arguments
///
/// * `arena` - The node arena containing the AST
/// * `root` - The root node ID
/// * `_options` - Options for rendering (currently unused)
///
/// # Returns
///
/// The HTML output as a String
pub fn render_html(arena: &NodeArena, root: NodeId, options: u32) -> String {
    render::html::render(arena, root, options)
}

/// Render an Arena-based AST to XML
///
/// # Arguments
///
/// * `arena` - The node arena containing the AST
/// * `root` - The root node ID
/// * `options` - Options for rendering
///
/// # Returns
///
/// The XML output as a String
pub fn render_xml(arena: &NodeArena, root: NodeId, options: u32) -> String {
    render::xml::render(arena, root, options)
}

/// Render an Arena-based AST as CommonMark
///
/// # Arguments
///
/// * `arena` - The node arena containing the AST
/// * `root` - The root node ID
/// * `options` - Options for rendering
///
/// # Returns
///
/// The CommonMark output as a String
pub fn render_commonmark(arena: &NodeArena, root: NodeId, options: u32) -> String {
    render::commonmark::render(arena, root, options)
}

/// Render an Arena-based AST as LaTeX
///
/// # Arguments
///
/// * `arena` - The node arena containing the AST
/// * `root` - The root node ID
/// * `options` - Options for rendering
///
/// # Returns
///
/// The LaTeX output as a String
pub fn render_latex(arena: &NodeArena, root: NodeId, options: u32) -> String {
    render::latex::render(arena, root, options)
}

/// Render an Arena-based AST as a Man page (groff format)
///
/// # Arguments
///
/// * `arena` - The node arena containing the AST
/// * `root` - The root node ID
/// * `options` - Options for rendering
///
/// # Returns
///
/// The Man page output as a String
pub fn render_man(arena: &NodeArena, root: NodeId, options: u32) -> String {
    render::man::render(arena, root, options)
}

// =============================================================================
// New API with DataKey-based Options
// =============================================================================

/// Convert Markdown to HTML using the new Options system.
///
/// This is the recommended way to convert Markdown to HTML when using
/// the new type-safe configuration system.
///
/// # Arguments
///
/// * `text` - The Markdown text to convert
/// * `options` - Configuration options using the `Options` struct
///
/// # Returns
///
/// The HTML output as a String
///
/// # Example
///
/// ```
/// use clmd::{markdown_to_html_with_options, config::options::Options};
/// use clmd::config::options::{SMART, ENABLE_TABLES};
///
/// let mut options = Options::new();
/// options.set(&SMART, true);
/// options.set(&ENABLE_TABLES, true);
///
/// let html = markdown_to_html_with_options("Hello *world*", &options);
/// assert_eq!(html, "<p>Hello <em>world</em></p>");
/// ```
pub fn markdown_to_html_with_options(text: &str, options: &Options) -> String {
    // Convert new Options to legacy u32 flags for now
    // This is a temporary bridge until all components use the new system
    let mut legacy_options = options::DEFAULT;

    if options.get(&config_options::SOURCEPOS) {
        legacy_options |= options::SOURCEPOS;
    }
    if options.get(&config_options::SMART) {
        legacy_options |= options::SMART;
    }
    if options.get(&config_options::HARDBREAKS) {
        legacy_options |= options::HARDBREAKS;
    }
    if options.get(&config_options::NOBREAKS) {
        legacy_options |= options::NOBREAKS;
    }
    if options.get(&config_options::VALIDATE_UTF8) {
        legacy_options |= options::VALIDATE_UTF8;
    }
    if options.get(&config_options::UNSAFE) {
        legacy_options |= options::UNSAFE;
    }

    markdown_to_html(text, legacy_options)
}

/// Convert Markdown to CommonMark using the new Options system.
///
/// # Arguments
///
/// * `text` - The Markdown text to convert
/// * `options` - Configuration options using the `Options` struct
///
/// # Returns
///
/// The CommonMark output as a String
///
/// # Example
///
/// ```
/// use clmd::{markdown_to_commonmark_with_options, config::options::Options};
///
/// let options = Options::new();
/// let cm = markdown_to_commonmark_with_options("Hello *world*", &options);
/// assert!(cm.contains("Hello"));
/// ```
pub fn markdown_to_commonmark_with_options(text: &str, _options: &Options) -> String {
    let mut arena = NodeArena::new();
    let doc =
        blocks::BlockParser::parse_with_options(&mut arena, text, options::DEFAULT);
    render::commonmark::render(&arena, doc, options::DEFAULT)
}

/// Convert Markdown to XML using the new Options system.
///
/// # Arguments
///
/// * `text` - The Markdown text to convert
/// * `options` - Configuration options using the `Options` struct
///
/// # Returns
///
/// The XML output as a String
///
/// # Example
///
/// ```
/// use clmd::{markdown_to_xml_with_options, config::options::Options};
///
/// let options = Options::new();
/// let xml = markdown_to_xml_with_options("Hello *world*", &options);
/// assert!(xml.contains("<document>"));
/// ```
pub fn markdown_to_xml_with_options(text: &str, _options: &Options) -> String {
    let mut arena = NodeArena::new();
    let doc =
        blocks::BlockParser::parse_with_options(&mut arena, text, options::DEFAULT);
    render::xml::render(&arena, doc, options::DEFAULT)
}

/// Parse a Markdown document and return the AST with new Options system.
///
/// # Arguments
///
/// * `text` - The Markdown text to parse
/// * `options` - Configuration options using the `Options` struct
///
/// # Returns
///
/// A tuple of (arena, root_node_id)
///
/// # Example
///
/// ```
/// use clmd::{parse_document_with_options, config::options::Options};
/// use clmd::config::options::ENABLE_TABLES;
///
/// let mut options = Options::new();
/// options.set(&ENABLE_TABLES, true);
///
/// let (arena, root) = parse_document_with_options("| a | b |\n|---|---|\n| c | d |", &options);
/// // Now you can traverse and manipulate the AST
/// ```
pub fn parse_document_with_options(
    text: &str,
    _options: &Options,
) -> (NodeArena, NodeId) {
    // TODO: Actually use options when parser supports them
    let mut arena = NodeArena::new();
    let doc =
        blocks::BlockParser::parse_with_options(&mut arena, text, options::DEFAULT);
    (arena, doc)
}

/// Format an existing AST to HTML using the new Options system.
///
/// # Arguments
///
/// * `arena` - The node arena containing the AST
/// * `root` - The root node ID
/// * `options` - Configuration options using the `Options` struct
///
/// # Returns
///
/// The HTML output as a String
///
/// # Example
///
/// ```
/// use clmd::{parse_document_with_options, format_html_with_options, config::options::Options};
///
/// let options = Options::new();
/// let (arena, root) = parse_document_with_options("Hello *world*", &options);
/// let html = format_html_with_options(&arena, root, &options);
/// ```
pub fn format_html_with_options(
    arena: &NodeArena,
    root: NodeId,
    options: &Options,
) -> String {
    let mut legacy_options = options::DEFAULT;

    if options.get(&config_options::SOURCEPOS) {
        legacy_options |= options::SOURCEPOS;
    }
    if options.get(&config_options::HARDBREAKS) {
        legacy_options |= options::HARDBREAKS;
    }
    if options.get(&config_options::NOBREAKS) {
        legacy_options |= options::NOBREAKS;
    }
    if options.get(&config_options::UNSAFE) {
        legacy_options |= options::UNSAFE;
    }

    render::html::render(arena, root, legacy_options)
}

/// Format an existing AST to CommonMark using the new Options system.
///
/// # Arguments
///
/// * `arena` - The node arena containing the AST
/// * `root` - The root node ID
/// * `options` - Configuration options using the `Options` struct
///
/// # Returns
///
/// The CommonMark output as a String
pub fn format_commonmark_with_options(
    arena: &NodeArena,
    root: NodeId,
    _options: &Options,
) -> String {
    render::commonmark::render(arena, root, options::DEFAULT)
}

/// Format an existing AST to XML using the new Options system.
///
/// # Arguments
///
/// * `arena` - The node arena containing the AST
/// * `root` - The root node ID
/// * `options` - Configuration options using the `Options` struct
///
/// # Returns
///
/// The XML output as a String
pub fn format_xml_with_options(
    arena: &NodeArena,
    root: NodeId,
    _options: &Options,
) -> String {
    render::xml::render(arena, root, options::DEFAULT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_to_html_basic() {
        let html = markdown_to_html("Hello world", options::DEFAULT);
        assert_eq!(html, "<p>Hello world</p>");
    }

    #[test]
    fn test_markdown_to_html_heading() {
        let html = markdown_to_html("# Heading 1\n\n## Heading 2", options::DEFAULT);
        assert!(html.contains("<h1>"));
        assert!(html.contains("<h2>"));
    }

    #[test]
    fn test_markdown_to_html_emphasis() {
        let html = markdown_to_html("*italic* and **bold**", options::DEFAULT);
        assert!(html.contains("<p>"));
    }

    #[test]
    fn test_markdown_to_html_link() {
        let html = markdown_to_html("[link](https://example.com)", options::DEFAULT);
        assert!(html.contains("<a href=\"https://example.com\">"));
    }

    #[test]
    fn test_markdown_to_html_code_inline() {
        let html = markdown_to_html("Use `code` here", options::DEFAULT);
        assert!(html.contains("<code>code</code>"));
    }

    #[test]
    fn test_markdown_to_html_code_block() {
        let html = markdown_to_html("```rust\nfn main() {}\n```", options::DEFAULT);
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
        let html = markdown_to_html("> Quote", options::DEFAULT);
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
        let html = markdown_to_html("- Item 1\n- Item 2", options::DEFAULT);
        println!("List HTML: {:?}", html);
        assert!(html.contains("<ul>"), "Expected <ul> in {}", html);
        assert!(html.contains("Item 1"), "Expected Item 1 in {}", html);
        assert!(html.contains("Item 2"), "Expected Item 2 in {}", html);
    }

    #[test]
    fn test_markdown_to_html_ordered_list() {
        let html = markdown_to_html("1. First\n2. Second", options::DEFAULT);
        assert!(html.contains("<ol>"));
        assert!(html.contains("First"));
        assert!(html.contains("Second"));
    }

    #[test]
    fn test_markdown_to_html_thematic_break() {
        let html = markdown_to_html("---", options::DEFAULT);
        assert_eq!(html, "<hr />");
    }

    #[test]
    fn test_markdown_to_html_image() {
        let html = markdown_to_html("![alt text](image.png)", options::DEFAULT);
        assert!(html.contains("<img src=\"image.png\""));
    }

    #[test]
    fn test_parse_and_render_roundtrip() {
        let input = "# Title\n\nParagraph with text.";
        let (arena, doc) = parse_document(input, options::DEFAULT);
        let html = render_html(&arena, doc, options::DEFAULT);
        assert!(html.contains("<h1>"));
        assert!(html.contains("Paragraph"));
    }

    // Tests for new API with Options
    #[test]
    fn test_markdown_to_html_with_options_basic() {
        use config::options::Options;

        let options = Options::new();
        let html = markdown_to_html_with_options("Hello world", &options);
        assert!(html.contains("<p>"));
        assert!(html.contains("Hello world"));
    }

    #[test]
    fn test_markdown_to_html_with_options_smart() {
        use config::options::{Options, SMART};

        let mut options = Options::new();
        options.set(&SMART, true);

        let html = markdown_to_html_with_options("\"Hello\"", &options);
        // Smart quotes should convert " to curly quotes
        assert!(html.contains("<p>"));
    }

    #[test]
    fn test_markdown_to_commonmark_with_options() {
        use config::options::Options;

        let options = Options::new();
        let cm = markdown_to_commonmark_with_options("Hello *world*", &options);
        assert!(cm.contains("Hello"));
        assert!(cm.contains("world"));
    }

    #[test]
    fn test_markdown_to_xml_with_options() {
        use config::options::Options;

        let options = Options::new();
        let xml = markdown_to_xml_with_options("Hello *world*", &options);
        assert!(xml.contains("<document>"));
        assert!(xml.contains("<paragraph>"));
    }

    #[test]
    fn test_parse_document_with_options() {
        use config::options::{Options, ENABLE_TABLES};

        let mut options = Options::new();
        options.set(&ENABLE_TABLES, true);

        let (arena, root) = parse_document_with_options("Hello *world*", &options);
        assert!(!arena.is_empty());
        assert!(arena.is_valid(root)); // Root should be a valid node ID
    }

    #[test]
    fn test_format_html_with_options() {
        use config::options::Options;

        let options = Options::new();
        let (arena, root) = parse_document_with_options("Hello *world*", &options);
        let html = format_html_with_options(&arena, root, &options);
        assert!(html.contains("<p>"));
        assert!(html.contains("<em>"));
    }
}
