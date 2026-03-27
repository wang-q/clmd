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

// Legacy option flags for internal use (not public API)
const OPT_SOURCEPOS: u32 = 1 << 0;
const OPT_HARDBREAKS: u32 = 1 << 1;
const OPT_NOBREAKS: u32 = 1 << 2;
const OPT_VALIDATE_UTF8: u32 = 1 << 3;
const OPT_SMART: u32 = 1 << 4;
const OPT_UNSAFE: u32 = 1 << 5;

/// Convert new Options to legacy u32 flags for parsing.
/// This is a temporary bridge until all components use the new system.
fn options_to_u32_for_parse(options: &Options) -> u32 {
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

/// Convert new Options to legacy u32 flags for rendering.
/// This is a temporary bridge until all components use the new system.
fn options_to_u32_for_render(options: &Options) -> u32 {
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
        let mut arena = NodeArena::new();
        let root = blocks::BlockParser::parse(&mut arena, input);
        Ok(Document { arena, root })
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
    pub fn parse_with_options(input: &str, options: &Options) -> ParseResult<Self> {
        let legacy_options = options_to_u32_for_parse(options);
        let mut arena = NodeArena::new();
        let root = blocks::BlockParser::parse_with_options(&mut arena, input, legacy_options);
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
            0,
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
        render::html::render(&self.arena, self.root, 0)
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
    pub fn to_html_with_options(&self, options: &Options) -> String {
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

/// Convert Markdown to HTML.
///
/// This is the primary interface for converting Markdown to HTML.
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
/// use clmd::{markdown_to_html, config::options::Options};
///
/// let options = Options::new();
/// let html = markdown_to_html("Hello *world*", &options);
/// assert_eq!(html, "<p>Hello <em>world</em></p>");
/// ```
pub fn markdown_to_html(text: &str, options: &Options) -> String {
    let legacy_options = options_to_u32_for_parse(options);
    let mut arena = NodeArena::new();
    let doc = blocks::BlockParser::parse_with_options(&mut arena, text, legacy_options);
    let render_options = options_to_u32_for_render(options);
    render::html::render(&arena, doc, render_options)
}

/// Convert Markdown to CommonMark.
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
/// use clmd::{markdown_to_commonmark, config::options::Options};
///
/// let options = Options::new();
/// let cm = markdown_to_commonmark("Hello *world*", &options);
/// assert!(cm.contains("Hello"));
/// ```
pub fn markdown_to_commonmark(text: &str, _options: &Options) -> String {
    let mut arena = NodeArena::new();
    let doc = blocks::BlockParser::parse(&mut arena, text);
    render::commonmark::render(&arena, doc, 0)
}

/// Convert Markdown to XML.
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
/// use clmd::{markdown_to_xml, config::options::Options};
///
/// let options = Options::new();
/// let xml = markdown_to_xml("Hello *world*", &options);
/// assert!(xml.contains("<document>"));
/// ```
pub fn markdown_to_xml(text: &str, _options: &Options) -> String {
    let mut arena = NodeArena::new();
    let doc = blocks::BlockParser::parse(&mut arena, text);
    render::xml::render(&arena, doc, 0)
}

/// Parse a Markdown document and return the AST.
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
/// use clmd::{parse_document, config::options::Options};
///
/// let options = Options::new();
/// let (arena, root) = parse_document("Hello *world*", &options);
/// // Now you can traverse and manipulate the AST
/// ```
pub fn parse_document(text: &str, options: &Options) -> (NodeArena, NodeId) {
    let legacy_options = options_to_u32_for_parse(options);
    let mut arena = NodeArena::new();
    let doc = blocks::BlockParser::parse_with_options(&mut arena, text, legacy_options);
    (arena, doc)
}

/// Format an existing AST to HTML.
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
/// use clmd::{parse_document, format_html, config::options::Options};
///
/// let options = Options::new();
/// let (arena, root) = parse_document("Hello *world*", &options);
/// let html = format_html(&arena, root, &options);
/// ```
pub fn format_html(arena: &NodeArena, root: NodeId, options: &Options) -> String {
    let legacy_options = options_to_u32_for_render(options);
    render::html::render(arena, root, legacy_options)
}

/// Format an existing AST to CommonMark.
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
pub fn format_commonmark(arena: &NodeArena, root: NodeId, _options: &Options) -> String {
    render::commonmark::render(arena, root, 0)
}

/// Format an existing AST to XML.
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
pub fn format_xml(arena: &NodeArena, root: NodeId, _options: &Options) -> String {
    render::xml::render(arena, root, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_to_html_basic() {
        let options = Options::new();
        let html = markdown_to_html("Hello world", &options);
        assert_eq!(html, "<p>Hello world</p>");
    }

    #[test]
    fn test_markdown_to_html_heading() {
        let options = Options::new();
        let html = markdown_to_html("# Heading 1\n\n## Heading 2", &options);
        assert!(html.contains("<h1>"));
        assert!(html.contains("<h2>"));
    }

    #[test]
    fn test_markdown_to_html_emphasis() {
        let options = Options::new();
        let html = markdown_to_html("*italic* and **bold**", &options);
        assert!(html.contains("<p>"));
    }

    #[test]
    fn test_markdown_to_html_link() {
        let options = Options::new();
        let html = markdown_to_html("[link](https://example.com)", &options);
        assert!(html.contains("<a href=\"https://example.com\">"));
    }

    #[test]
    fn test_markdown_to_html_code_inline() {
        let options = Options::new();
        let html = markdown_to_html("Use `code` here", &options);
        assert!(html.contains("<code>code</code>"));
    }

    #[test]
    fn test_markdown_to_html_code_block() {
        let options = Options::new();
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
        let options = Options::new();
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
        let options = Options::new();
        let html = markdown_to_html("- Item 1\n- Item 2", &options);
        println!("List HTML: {:?}", html);
        assert!(html.contains("<ul>"), "Expected <ul> in {}", html);
        assert!(html.contains("Item 1"), "Expected Item 1 in {}", html);
        assert!(html.contains("Item 2"), "Expected Item 2 in {}", html);
    }

    #[test]
    fn test_markdown_to_html_ordered_list() {
        let options = Options::new();
        let html = markdown_to_html("1. First\n2. Second", &options);
        assert!(html.contains("<ol>"));
        assert!(html.contains("First"));
        assert!(html.contains("Second"));
    }

    #[test]
    fn test_markdown_to_html_thematic_break() {
        let options = Options::new();
        let html = markdown_to_html("---", &options);
        assert_eq!(html, "<hr />");
    }

    #[test]
    fn test_markdown_to_html_image() {
        let options = Options::new();
        let html = markdown_to_html("![alt text](image.png)", &options);
        assert!(html.contains("<img src=\"image.png\""));
    }

    #[test]
    fn test_parse_and_render_roundtrip() {
        let options = Options::new();
        let input = "# Title\n\nParagraph with text.";
        let (arena, doc) = parse_document(input, &options);
        let html = format_html(&arena, doc, &options);
        assert!(html.contains("<h1>"));
        assert!(html.contains("Paragraph"));
    }

    #[test]
    fn test_markdown_to_html_with_smart() {
        use config::options::SMART;

        let mut options = Options::new();
        options.set(&SMART, true);

        let html = markdown_to_html("\"Hello\"", &options);
        // Smart quotes should convert " to curly quotes
        assert!(html.contains("<p>"));
    }

    #[test]
    fn test_markdown_to_commonmark() {
        let options = Options::new();
        let cm = markdown_to_commonmark("Hello *world*", &options);
        assert!(cm.contains("Hello"));
        assert!(cm.contains("world"));
    }

    #[test]
    fn test_markdown_to_xml() {
        let options = Options::new();
        let xml = markdown_to_xml("Hello *world*", &options);
        assert!(xml.contains("<document>"));
        assert!(xml.contains("<paragraph>"));
    }

    #[test]
    fn test_parse_document() {
        let options = Options::new();
        let (arena, root) = parse_document("Hello *world*", &options);
        assert!(!arena.is_empty());
        assert!(arena.is_valid(root)); // Root should be a valid node ID
    }

    #[test]
    fn test_format_html() {
        let options = Options::new();
        let (arena, root) = parse_document("Hello *world*", &options);
        let html = format_html(&arena, root, &options);
        assert!(html.contains("<p>"));
        assert!(html.contains("<em>"));
    }
}
