//! XML renderer
//!
//! This module provides XML output generation for documents parsed using the Arena-based parser.
//! Useful for debugging and AST inspection.

use crate::arena::{NodeArena, NodeId};
use crate::nodes::{
    AstNode, ListDelimType, ListType, NodeCode, NodeCodeBlock, NodeHeading, NodeHtmlBlock,
    NodeLink, NodeList, NodeValue,
};
use crate::parser::options::{Options, Plugins};
use std::fmt;

/// Render an Arena-based AST to XML
///
/// # Arguments
///
/// * `arena` - The node arena containing the AST
/// * `root` - The root node ID
/// * `options` - Rendering options
///
/// # Returns
///
/// The XML output as a String
///
/// # Example
///
/// ```ignore
/// use clmd::{parse_document, render_to_xml, parser::options::Options, Arena};
///
/// let mut arena = Arena::new();
/// let options = Options::default();
/// let doc = parse_document(&mut arena, "# Hello", &options);
/// let xml = render_to_xml(&arena, doc, 0);
/// assert!(xml.contains("<heading level=\"1\">"));
/// ```
pub fn render(arena: &NodeArena, root: NodeId, options: u32) -> String {
    let mut renderer = XmlRenderer::new(arena, options);
    renderer.render(root)
}

/// Format an AST as XML with plugins (comrak-style API).
///
/// This is a temporary implementation that provides basic XML output.
pub fn format_document_with_plugins<'a>(
    root: &'a AstNode<'a>,
    options: &Options,
    output: &mut dyn fmt::Write,
    _plugins: &Plugins<'_>,
) -> fmt::Result {
    output.write_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n")?;
    output.write_str("<!DOCTYPE document SYSTEM \"CommonMark.dtd\">\n")?;
    format_node_xml(root, options, output)
}

fn format_node_xml(
    node: &AstNode<'_>,
    options: &Options,
    output: &mut dyn fmt::Write,
) -> fmt::Result {
    let ast = node.data.borrow();
    let tag_name = ast.value.xml_node_name();

    output.write_str("<")?;
    output.write_str(tag_name)?;

    // Add source position if enabled
    if options.render.sourcepos && ast.sourcepos.start.line > 0 {
        write!(
            output,
            " sourcepos=\"{}:{}-{}:{}\"",
            ast.sourcepos.start.line,
            ast.sourcepos.start.column,
            ast.sourcepos.end.line,
            ast.sourcepos.end.column
        )?;
    }

    // Add type-specific attributes
    match &ast.value {
        NodeValue::List(list) => {
            match list.list_type {
                ListType::Bullet => {
                    output.write_str(" type=\"bullet\"")?;
                }
                ListType::Ordered => {
                    output.write_str(" type=\"ordered\"")?;
                    if list.start != 1 {
                        write!(output, " start=\"{}\"", list.start)?;
                    }
                    match list.delimiter {
                        ListDelimType::Period => {
                            output.write_str(" delim=\"period\"")?;
                        }
                        ListDelimType::Paren => {
                            output.write_str(" delim=\"paren\"")?;
                        }
                    }
                }
            }
            if list.tight {
                output.write_str(" tight=\"true\"")?;
            }
        }
        NodeValue::Heading(heading) => {
            write!(output, " level=\"{}\"", heading.level)?;
        }
        NodeValue::CodeBlock(code) => {
            if !code.info.is_empty() {
                write!(output, " info=\"{}\"", escape_xml(&code.info))?;
            }
        }
        NodeValue::Link(link) | NodeValue::Image(link) => {
            write!(output, " destination=\"{}\"", escape_xml(&link.url))?;
            if !link.title.is_empty() {
                write!(output, " title=\"{}\"", escape_xml(&link.title))?;
            }
        }
        _ => {}
    }

    // Handle leaf nodes with literal content
    if ast.value.is_leaf() {
        match &ast.value {
            NodeValue::Text(text) => {
                if !text.is_empty() {
                    output.write_str(">")?;
                    output.write_str(&escape_xml(text))?;
                    write!(output, "</{tag_name}>")?;
                } else {
                    output.write_str(" />")?;
                }
            }
            NodeValue::HtmlInline(html) | NodeValue::Raw(html) => {
                if !html.is_empty() {
                    output.write_str(">")?;
                    output.write_str(&escape_xml(html))?;
                    write!(output, "</{tag_name}>")?;
                } else {
                    output.write_str(" />")?;
                }
            }
            NodeValue::CodeBlock(code) | NodeValue::HtmlBlock(code) => {
                if !code.literal.is_empty() {
                    output.write_str(">")?;
                    output.write_str(&escape_xml(&code.literal))?;
                    write!(output, "</{tag_name}>")?;
                } else {
                    output.write_str(" />")?;
                }
            }
            NodeValue::Code(code) => {
                if !code.literal.is_empty() {
                    output.write_str(">")?;
                    output.write_str(&escape_xml(&code.literal))?;
                    write!(output, "</{tag_name}>")?;
                } else {
                    output.write_str(" />")?;
                }
            }
            _ => {
                output.write_str(" />")?;
            }
        }
    } else {
        output.write_str(">\n")?;

        // Render children
        let mut child_opt = node.first_child();
        while let Some(child) = child_opt {
            format_node_xml(child, options, output)?;
            child_opt = child.next_sibling();
        }

        write!(output, "</{tag_name}>\n")?;
    }

    Ok(())
}

struct XmlRenderer<'a> {
    arena: &'a NodeArena,
    options: u32,
    output: String,
}

impl<'a> XmlRenderer<'a> {
    fn new(arena: &'a NodeArena, options: u32) -> Self {
        XmlRenderer {
            arena,
            options,
            output: String::new(),
        }
    }

    fn render(&mut self, root: NodeId) -> String {
        self.output
            .push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        self.output
            .push_str("<!DOCTYPE document SYSTEM \"CommonMark.dtd\">\n");

        self.render_node(root, true);

        self.output.clone()
    }

    fn render_node(&mut self, node_id: NodeId, entering: bool) {
        if entering {
            self.enter_node(node_id);

            // Render children
            let node = self.arena.get(node_id);
            let mut child_opt = node.first_child;
            while let Some(child_id) = child_opt {
                self.render_node(child_id, true);
                child_opt = self.arena.get(child_id).next;
            }

            self.exit_node(node_id);
        }
    }

    fn enter_node(&mut self, node_id: NodeId) {
        let node = self.arena.get(node_id);
        let tag_name = node.value.xml_node_name();

        self.output.push('<');
        self.output.push_str(tag_name);

        // Add source position
        if self.options & crate::OPT_SOURCEPOS != 0 {
            self.output.push_str(&format!(
                " sourcepos=\"{}:{}-{}:{}\"",
                node.source_pos.start.line,
                node.source_pos.start.column,
                node.source_pos.end.line,
                node.source_pos.end.column
            ));
        }

        // Add type-specific attributes
        match &node.value {
            NodeValue::List(NodeList {
                list_type,
                delimiter,
                start,
                tight,
                ..
            }) => {
                match list_type {
                    ListType::Bullet => {
                        self.output.push_str(" type=\"bullet\"");
                    }
                    ListType::Ordered => {
                        self.output.push_str(" type=\"ordered\"");
                        if *start != 1 {
                            self.output.push_str(&format!(" start=\"{}\"", start));
                        }
                        match delimiter {
                            ListDelimType::Period => {
                                self.output.push_str(" delim=\"period\"");
                            }
                            ListDelimType::Paren => {
                                self.output.push_str(" delim=\"paren\"");
                            }
                        }
                    }
                }
                if *tight {
                    self.output.push_str(" tight=\"true\"");
                }
            }
            NodeValue::Heading(NodeHeading { level, .. }) => {
                self.output.push_str(&format!(" level=\"{}\"", level));
            }
            NodeValue::CodeBlock(NodeCodeBlock { info, .. }) => {
                if !info.is_empty() {
                    self.output.push_str(" info=\"");
                    self.output.push_str(&escape_xml(info));
                    self.output.push('"');
                }
            }
            NodeValue::Link(NodeLink { url, title })
            | NodeValue::Image(NodeLink { url, title }) => {
                self.output.push_str(" destination=\"");
                self.output.push_str(&escape_xml(url));
                self.output.push('"');
                if !title.is_empty() {
                    self.output.push_str(" title=\"");
                    self.output.push_str(&escape_xml(title));
                    self.output.push('"');
                }
            }
            _ => {}
        }

        // Handle leaf nodes with literal content
        if self.is_leaf(node_id) {
            match &node.value {
                NodeValue::Text(literal) => {
                    if !literal.is_empty() {
                        self.output.push('>');
                        self.output.push_str(&escape_xml(literal.as_ref()));
                        self.output.push_str("</");
                        self.output.push_str(tag_name);
                        self.output.push('>');
                    } else {
                        self.output.push_str(" />");
                    }
                }
                NodeValue::HtmlInline(literal) | NodeValue::Raw(literal) => {
                    if !literal.is_empty() {
                        self.output.push('>');
                        self.output.push_str(&escape_xml(literal));
                        self.output.push_str("</");
                        self.output.push_str(tag_name);
                        self.output.push('>');
                    } else {
                        self.output.push_str(" />");
                    }
                }
                NodeValue::CodeBlock(NodeCodeBlock { literal, .. })
                | NodeValue::HtmlBlock(NodeHtmlBlock { literal, .. })
                | NodeValue::Code(NodeCode { literal, .. }) => {
                    if !literal.is_empty() {
                        self.output.push('>');
                        self.output.push_str(&escape_xml(literal));
                        self.output.push_str("</");
                        self.output.push_str(tag_name);
                        self.output.push('>');
                    } else {
                        self.output.push_str(" />");
                    }
                }
                _ => {
                    self.output.push_str(" />");
                }
            }
        } else {
            self.output.push('>');
        }

        self.output.push('\n');
    }

    fn exit_node(&mut self, node_id: NodeId) {
        // Leaf nodes are already closed in enter_node
        if !self.is_leaf(node_id) {
            let node = self.arena.get(node_id);
            let tag_name = node.value.xml_node_name();
            self.output.push_str("</");
            self.output.push_str(tag_name);
            self.output.push_str(">\n");
        }
    }

    fn is_leaf(&self, node_id: NodeId) -> bool {
        let node = self.arena.get(node_id);
        // Document is never a leaf, even when empty
        if matches!(node.value, NodeValue::Document) {
            return false;
        }
        node.first_child.is_none()
    }
}

/// Escape XML special characters
fn escape_xml(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&apos;"),
            _ => result.push(c),
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arena::{Node, NodeArena, TreeOps};

    #[test]
    fn test_escape_xml() {
        assert_eq!(escape_xml("<div>"), "&lt;div&gt;");
        assert_eq!(escape_xml("&"), "&amp;");
        assert_eq!(escape_xml("'test'"), "&apos;test&apos;");
    }

    #[test]
    fn test_render_document() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let xml = render(&arena, root, 0);
        println!("XML output: {:?}", xml);
        assert!(xml.contains("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(xml.contains("<document>"), "Expected <document> in {}", xml);
        assert!(xml.contains("</document>"));
    }

    #[test]
    fn test_render_paragraph() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text =
            arena.alloc(Node::with_value(NodeValue::Text("Hello world".to_string())));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, text);

        let xml = render(&arena, root, 0);
        assert!(xml.contains("<paragraph>"));
        assert!(xml.contains("<text>Hello world</text>"));
        assert!(xml.contains("</paragraph>"));
    }

    #[test]
    fn test_render_bullet_list() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let list = arena.alloc(Node::with_value(NodeValue::List(NodeList {
            list_type: ListType::Bullet,
            delimiter: ListDelimType::Period,
            start: 1,
            tight: true,
            bullet_char: b'-',
            marker_offset: 0,
            padding: 2,
            is_task_list: false,
        })));
        let item = arena.alloc(Node::with_value(NodeValue::Item(NodeList::default())));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::Text("Item".to_string())));

        TreeOps::append_child(&mut arena, root, list);
        TreeOps::append_child(&mut arena, list, item);
        TreeOps::append_child(&mut arena, item, para);
        TreeOps::append_child(&mut arena, para, text);

        let xml = render(&arena, root, 0);
        assert!(xml.contains("<list type=\"bullet\" tight=\"true\">"));
        assert!(xml.contains("<item>"));
        assert!(xml.contains("<text>Item</text>"));
    }

    #[test]
    fn test_render_ordered_list() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let list = arena.alloc(Node::with_value(NodeValue::List(NodeList {
            list_type: ListType::Ordered,
            delimiter: ListDelimType::Period,
            start: 1,
            tight: false,
            bullet_char: 0,
            marker_offset: 0,
            padding: 3,
            is_task_list: false,
        })));
        let item = arena.alloc(Node::with_value(NodeValue::Item(NodeList::default())));

        TreeOps::append_child(&mut arena, root, list);
        TreeOps::append_child(&mut arena, list, item);

        let xml = render(&arena, root, 0);
        assert!(xml.contains("<list type=\"ordered\" delim=\"period\">"));
    }

    #[test]
    fn test_render_ordered_list_with_start() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let list = arena.alloc(Node::with_value(NodeValue::List(NodeList {
            list_type: ListType::Ordered,
            delimiter: ListDelimType::Paren,
            start: 5,
            tight: true,
            bullet_char: 0,
            marker_offset: 0,
            padding: 3,
            is_task_list: false,
        })));
        let item = arena.alloc(Node::with_value(NodeValue::Item(NodeList::default())));

        TreeOps::append_child(&mut arena, root, list);
        TreeOps::append_child(&mut arena, list, item);

        let xml = render(&arena, root, 0);
        assert!(xml.contains(
            "<list type=\"ordered\" start=\"5\" delim=\"paren\" tight=\"true\">"
        ));
    }

    #[test]
    fn test_render_heading() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let heading = arena.alloc(Node::with_value(NodeValue::Heading(NodeHeading {
            level: 2,
            setext: false,
            closed: false,
        })));
        let text = arena.alloc(Node::with_value(NodeValue::Text("Title".to_string())));

        TreeOps::append_child(&mut arena, root, heading);
        TreeOps::append_child(&mut arena, heading, text);

        let xml = render(&arena, root, 0);
        assert!(xml.contains("<heading level=\"2\">"));
        assert!(xml.contains("<text>Title</text>"));
        assert!(xml.contains("</heading>"));
    }

    #[test]
    fn test_render_code_block() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let code_block =
            arena.alloc(Node::with_value(NodeValue::CodeBlock(NodeCodeBlock {
                fenced: true,
                fence_char: b'`',
                fence_length: 3,
                fence_offset: 0,
                info: "rust".to_string(),
                literal: "fn main() {}".to_string(),
                closed: true,
            })));

        TreeOps::append_child(&mut arena, root, code_block);

        let xml = render(&arena, root, 0);
        assert!(xml.contains("<code_block info=\"rust\">fn main() {}</code_block>"));
    }

    #[test]
    fn test_render_link() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let link = arena.alloc(Node::with_value(NodeValue::Link(NodeLink {
            url: "https://example.com".to_string(),
            title: "Example".to_string(),
        })));
        let text = arena.alloc(Node::with_value(NodeValue::Text("link".to_string())));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, link);
        TreeOps::append_child(&mut arena, link, text);

        let xml = render(&arena, root, 0);
        assert!(
            xml.contains("<link destination=\"https://example.com\" title=\"Example\">")
        );
        assert!(xml.contains("<text>link</text>"));
    }

    #[test]
    fn test_render_image() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let image = arena.alloc(Node::with_value(NodeValue::Image(NodeLink {
            url: "image.png".to_string(),
            title: "Alt text".to_string(),
        })));
        let text = arena.alloc(Node::with_value(NodeValue::Text("alt".to_string())));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, image);
        TreeOps::append_child(&mut arena, image, text);

        let xml = render(&arena, root, 0);
        assert!(xml.contains("<image destination=\"image.png\" title=\"Alt text\">"));
    }

    #[test]
    fn test_render_with_sourcepos() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        {
            let p = arena.get_mut(para);
            p.source_pos.start.line = 1;
            p.source_pos.start.column = 1;
            p.source_pos.end.line = 1;
            p.source_pos.end.column = 10;
        }

        TreeOps::append_child(&mut arena, root, para);

        let xml = render(&arena, root, crate::OPT_SOURCEPOS);
        assert!(xml.contains("sourcepos=\"1:1-1:10\""));
    }

    #[test]
    fn test_render_blockquote() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let blockquote = arena.alloc(Node::with_value(NodeValue::BlockQuote));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::Text("Quote".to_string())));

        TreeOps::append_child(&mut arena, root, blockquote);
        TreeOps::append_child(&mut arena, blockquote, para);
        TreeOps::append_child(&mut arena, para, text);

        let xml = render(&arena, root, 0);
        assert!(xml.contains("<block_quote>"));
        assert!(xml.contains("<text>Quote</text>"));
        assert!(xml.contains("</block_quote>"));
    }

    #[test]
    fn test_render_emph() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let emph = arena.alloc(Node::with_value(NodeValue::Emph));
        let text =
            arena.alloc(Node::with_value(NodeValue::Text("emphasized".to_string())));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, emph);
        TreeOps::append_child(&mut arena, emph, text);

        let xml = render(&arena, root, 0);
        assert!(xml.contains("<emph>"));
        assert!(xml.contains("<text>emphasized</text>"));
        assert!(xml.contains("</emph>"));
    }

    #[test]
    fn test_render_strong() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let strong = arena.alloc(Node::with_value(NodeValue::Strong));
        let text = arena.alloc(Node::with_value(NodeValue::Text("strong".to_string())));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, strong);
        TreeOps::append_child(&mut arena, strong, text);

        let xml = render(&arena, root, 0);
        assert!(xml.contains("<strong>"));
        assert!(xml.contains("<text>strong</text>"));
        assert!(xml.contains("</strong>"));
    }

    #[test]
    fn test_render_code() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let code = arena.alloc(Node::with_value(NodeValue::Code(NodeCode {
            num_backticks: 1,
            literal: "code".to_string(),
        })));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, code);

        let xml = render(&arena, root, 0);
        assert!(xml.contains("<code>code</code>"));
    }

    #[test]
    fn test_render_thematic_break() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let hr = arena.alloc(Node::with_value(NodeValue::ThematicBreak));

        TreeOps::append_child(&mut arena, root, hr);

        let xml = render(&arena, root, 0);
        assert!(xml.contains("<thematic_break />"));
    }

    #[test]
    fn test_render_html_block() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let html_block =
            arena.alloc(Node::with_value(NodeValue::HtmlBlock(NodeHtmlBlock {
                block_type: 0,
                literal: "<div>content</div>".to_string(),
            })));

        TreeOps::append_child(&mut arena, root, html_block);

        let xml = render(&arena, root, 0);
        assert!(xml.contains("<html_block>&lt;div&gt;content&lt;/div&gt;</html_block>"));
    }

    #[test]
    fn test_render_soft_break() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let soft_break = arena.alloc(Node::with_value(NodeValue::SoftBreak));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, soft_break);

        let xml = render(&arena, root, 0);
        assert!(xml.contains("<softbreak />"));
    }

    #[test]
    fn test_render_line_break() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let line_break = arena.alloc(Node::with_value(NodeValue::HardBreak));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, line_break);

        let xml = render(&arena, root, 0);
        assert!(xml.contains("<linebreak />"));
    }

    #[test]
    fn test_render_empty_text() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::Text("".to_string())));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, text);

        let xml = render(&arena, root, 0);
        assert!(xml.contains("<text />"));
    }
}
