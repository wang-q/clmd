/// Abbreviation support (not yet implemented)
pub mod abbreviation;

/// Arena-based memory management for AST nodes
///
/// This module provides the core data structures for efficient node allocation
/// and tree manipulation using arena allocation instead of Rc<RefCell>.
pub mod arena;

/// Error types and parsing limits
pub mod error;

/// AST traversal and visitor patterns
pub mod ast;

/// AST node type definitions
pub mod ast_nodes;

/// HTML attributes handling
pub mod attributes;

/// Autolink detection (URLs and email addresses)
pub mod autolink;

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

/// Definition lists support
pub mod definition;

/// Footnotes support
pub mod footnotes;

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

/// Core node types and data structures
pub mod node;

/// High-level parser interface
pub mod parser;

/// HTML rendering for Arena-based AST
///
/// This module provides HTML output generation for documents parsed
/// using the Arena-based parser.
pub mod render;

/// Text sequence utilities
pub mod sequence;

/// Strikethrough text support
pub mod strikethrough;

/// Tables support (GitHub Flavored Markdown)
pub mod tables;

/// Task lists support (GitHub Flavored Markdown)
pub mod tasklist;

/// Test utilities
pub mod test_utils;

/// Table of contents generation
pub mod toc;

/// YAML front matter support
pub mod yaml_front_matter;

pub use arena::{Node, NodeArena, NodeId, TreeOps};
pub use error::{ParseError, ParseResult, ParserLimits, Position};
pub use iterator::{ArenaNodeIterator, ArenaNodeWalker, EventType};
pub use node::{DelimType, ListType, NodeData, NodeType, SourcePos};
pub use parser::Parser;

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
    pub fn root(&self) -> NodeId {
        self.root
    }

    /// Get a reference to the arena
    pub fn arena(&self) -> &NodeArena {
        &self.arena
    }

    /// Get a mutable reference to the arena
    pub fn arena_mut(&mut self) -> &mut NodeArena {
        &mut self.arena
    }

    /// Consume the document and return the arena and root node
    pub fn into_parts(self) -> (NodeArena, NodeId) {
        (self.arena, self.root)
    }
}

/// Options for parsing and rendering
pub mod options {
    /// Default options
    pub const DEFAULT: u32 = 0;

    /// Include a `data-sourcepos` attribute on all block elements
    pub const SOURCEPOS: u32 = 1 << 0;

    /// Render `softbreak` elements as hard line breaks
    pub const HARDBREAKS: u32 = 1 << 1;

    /// Render `softbreak` elements as spaces
    pub const NOBREAKS: u32 = 1 << 2;

    /// Validate UTF-8 in the input before parsing
    pub const VALIDATE_UTF8: u32 = 1 << 3;

    /// Convert straight quotes to curly, `---` to em dashes, `--` to en dashes
    pub const SMART: u32 = 1 << 4;

    /// Render raw HTML and unsafe links
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
/// # Arguments
///
/// * `text` - The Markdown text to parse
/// * `options` - Options for parsing
///
/// # Returns
///
/// A tuple of (arena, document_node_id)
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
}
