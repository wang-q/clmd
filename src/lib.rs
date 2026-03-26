/// Abbreviation support (not yet implemented)
pub mod abbreviation;

/// Arena-based memory management for AST nodes
///
/// This module provides the core data structures for efficient node allocation
/// and tree manipulation using arena allocation instead of Rc<RefCell>.
pub mod arena;

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
pub use iterator::{NodeIterator, NodeWalker};
pub use node::{DelimType, ListType, NodeData, NodeType, SourcePos};

/// Options for parsing and rendering
pub mod options {
    /// Default options
    pub const DEFAULT: u32 = 0;

    /// Include a `data-sourcepos` attribute on all block elements
    pub const SOURCEPOS: u32 = 1 << 1;

    /// Render `softbreak` elements as hard line breaks
    pub const HARDBREAKS: u32 = 1 << 2;

    /// Render raw HTML and unsafe links
    pub const UNSAFE: u32 = 1 << 17;

    /// Render `softbreak` elements as spaces
    pub const NOBREAKS: u32 = 1 << 4;

    /// Validate UTF-8 in the input before parsing
    pub const VALIDATE_UTF8: u32 = 1 << 9;

    /// Convert straight quotes to curly, `---` to em dashes, `--` to en dashes
    pub const SMART: u32 = 1 << 10;
}

/// Simple interface: convert Markdown to HTML
///
/// # Arguments
///
/// * `text` - The Markdown text to convert
/// * `_options` - Options for parsing and rendering (currently unused)
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

    render::arena_html::render(&arena, doc, options)
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
    render::arena_html::render(arena, root, options)
}

/// Render an Arena-based AST to XML
///
/// # Arguments
///
/// * `arena` - The node arena containing the AST
/// * `root` - The root node ID
/// * `_options` - Options for rendering (currently unused)
///
/// # Returns
///
/// The XML output as a String
pub fn render_xml(_arena: &NodeArena, _root: NodeId, _options: u32) -> String {
    // TODO: Implement XML renderer for Arena
    String::from("<!-- XML rendering not yet implemented for Arena -->")
}

/// Render an Arena-based AST as CommonMark
///
/// # Arguments
///
/// * `arena` - The node arena containing the AST
/// * `root` - The root node ID
/// * `_options` - Options for rendering (currently unused)
///
/// # Returns
///
/// The CommonMark output as a String
pub fn render_commonmark(_arena: &NodeArena, _root: NodeId, _options: u32) -> String {
    // TODO: Implement CommonMark renderer for Arena
    String::from("<!-- CommonMark rendering not yet implemented for Arena -->")
}

/// Render an Arena-based AST as LaTeX
///
/// # Arguments
///
/// * `arena` - The node arena containing the AST
/// * `root` - The root node ID
/// * `_options` - Options for rendering (currently unused)
///
/// # Returns
///
/// The LaTeX output as a String
pub fn render_latex(_arena: &NodeArena, _root: NodeId, _options: u32) -> String {
    // TODO: Implement LaTeX renderer for Arena
    String::from("<!-- LaTeX rendering not yet implemented for Arena -->")
}

/// Render an Arena-based AST as a Man page (groff format)
///
/// # Arguments
///
/// * `arena` - The node arena containing the AST
/// * `root` - The root node ID
/// * `_options` - Options for rendering (currently unused)
///
/// # Returns
///
/// The Man page output as a String
pub fn render_man(_arena: &NodeArena, _root: NodeId, _options: u32) -> String {
    // TODO: Implement Man page renderer for Arena
    String::from("<!-- Man page rendering not yet implemented for Arena -->")
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
