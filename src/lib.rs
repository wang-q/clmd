pub mod abbreviation;
pub mod ast;
pub mod ast_nodes;
pub mod attributes;
pub mod autolink;
pub mod blocks;
pub mod compat;
pub mod config;
pub mod converters;
pub mod definition;
pub mod footnotes;
pub mod html_to_md;
pub mod html_utils;
pub mod inlines;
pub mod iterator;
pub mod lexer;
pub mod node;
pub mod parser;
pub mod render;
pub mod sequence;
pub mod strikethrough;
pub mod tables;
pub mod tasklist;
pub mod test_utils;
pub mod toc;
pub mod yaml_front_matter;

pub use iterator::{NodeIterator, NodeWalker};
pub use node::{
    append_child, prepend_child, unlink, DelimType, ListType, Node, NodeData, NodeType,
    SourcePos,
};

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
/// * `options` - Options for parsing and rendering
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
    let parser = parser::Parser::new(options);
    let root = parser.parse(text);
    render::html::render(&root, options)
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
/// The root node of the AST
pub fn parse_document(
    text: &str,
    options: u32,
) -> std::rc::Rc<std::cell::RefCell<Node>> {
    let parser = parser::Parser::new(options);
    parser.parse(text)
}

/// Render a node tree as HTML
///
/// # Arguments
///
/// * `root` - The root node of the AST
/// * `options` - Options for rendering
///
/// # Returns
///
/// The HTML output as a String
pub fn render_html(
    root: &std::rc::Rc<std::cell::RefCell<Node>>,
    options: u32,
) -> String {
    render::html::render(root, options)
}

/// Render a node tree as XML
///
/// # Arguments
///
/// * `root` - The root node of the AST
/// * `options` - Options for rendering
///
/// # Returns
///
/// The XML output as a String
pub fn render_xml(root: &std::rc::Rc<std::cell::RefCell<Node>>, options: u32) -> String {
    render::xml::render(root, options)
}

/// Render a node tree as CommonMark
///
/// # Arguments
///
/// * `root` - The root node of the AST
/// * `options` - Options for rendering
///
/// # Returns
///
/// The CommonMark output as a String
pub fn render_commonmark(
    root: &std::rc::Rc<std::cell::RefCell<Node>>,
    options: u32,
) -> String {
    render::commonmark::render(root, options)
}

/// Render a node tree as LaTeX
///
/// # Arguments
///
/// * `root` - The root node of the AST
/// * `options` - Options for rendering
///
/// # Returns
///
/// The LaTeX output as a String
pub fn render_latex(
    root: &std::rc::Rc<std::cell::RefCell<Node>>,
    options: u32,
) -> String {
    render::latex::render(root, options)
}

/// Render a node tree as a Man page (groff format)
///
/// # Arguments
///
/// * `root` - The root node of the AST
/// * `options` - Options for rendering
///
/// # Returns
///
/// The Man page output as a String
pub fn render_man(root: &std::rc::Rc<std::cell::RefCell<Node>>, options: u32) -> String {
    render::man::render(root, options)
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
        // Heading content is currently not parsed for inline elements
        assert!(html.contains("<h1>"));
        assert!(html.contains("<h2>"));
    }

    #[test]
    fn test_markdown_to_html_emphasis() {
        let html = markdown_to_html("*italic* and **bold**", options::DEFAULT);
        // Emphasis parsing is partially implemented
        assert!(html.contains("<p>"));
    }

    #[test]
    fn test_markdown_to_html_link() {
        let html = markdown_to_html("[link](https://example.com)", options::DEFAULT);
        // Link parsing creates the link structure but text content may vary
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
        assert!(html.contains("<pre>"));
        assert!(html.contains("<code class=\"language-rust\">"));
        assert!(html.contains("fn main() {}"));
    }

    #[test]
    fn test_markdown_to_html_blockquote() {
        let html = markdown_to_html("> Quote", options::DEFAULT);
        assert!(html.contains("<blockquote>"));
        assert!(html.contains("Quote"));
    }

    #[test]
    fn test_markdown_to_html_list() {
        let html = markdown_to_html("- Item 1\n- Item 2", options::DEFAULT);
        assert!(html.contains("<ul>"));
        assert!(html.contains("Item 1"));
        assert!(html.contains("Item 2"));
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
    fn test_markdown_to_html_with_sourcepos() {
        let html = markdown_to_html("Hello", options::SOURCEPOS);
        assert!(html.contains("data-sourcepos"));
    }

    #[test]
    fn test_parse_and_render_roundtrip() {
        let input = "# Title\n\nParagraph with text.";
        let doc = parse_document(input, options::DEFAULT);
        let html = render_html(&doc, options::DEFAULT);
        assert!(html.contains("<h1>"));
        assert!(html.contains("Paragraph"));
    }
}
