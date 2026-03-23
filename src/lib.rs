pub mod blocks;
pub mod inlines;
pub mod iterator;
pub mod lexer;
pub mod node;
pub mod parser;
pub mod render;

pub use iterator::{NodeIterator, NodeWalker};
pub use node::{
    append_child, prepend_child, unlink, DelimType, ListType, Node, NodeData, NodeType, SourcePos,
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
/// use md::markdown_to_html;
/// use md::options;
///
/// let html = markdown_to_html("Hello *world*", options::DEFAULT);
/// assert_eq!(html, "<p>Hello *world*</p>\n");
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
pub fn parse_document(text: &str, options: u32) -> std::rc::Rc<std::cell::RefCell<Node>> {
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
pub fn render_html(root: &std::rc::Rc<std::cell::RefCell<Node>>, options: u32) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_to_html_basic() {
        let html = markdown_to_html("Hello world", options::DEFAULT);
        assert_eq!(html, "<p>Hello world</p>\n");
    }
}
