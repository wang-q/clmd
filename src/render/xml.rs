//! XML renderer
//!
//! This module provides XML output generation for documents parsed using the Arena-based parser.
//! Useful for debugging and AST inspection.

use crate::arena::{NodeArena, NodeId};
use crate::node::{NodeData, NodeType};

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
/// ```
/// use clmd::{parse_document, render_xml, options};
///
/// let (arena, doc) = parse_document("# Hello", options::DEFAULT);
/// let xml = render_xml(&arena, doc, options::DEFAULT);
/// assert!(xml.contains("<heading level=\"1\">"));
/// ```
pub fn render(arena: &NodeArena, root: NodeId, options: u32) -> String {
    let mut renderer = XmlRenderer::new(arena, options);
    renderer.render(root)
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
        let tag_name = self.node_type_to_tag(&node.node_type);

        self.output.push('<');
        self.output.push_str(tag_name);

        // Add source position
        if self.options & crate::options::SOURCEPOS != 0 {
            self.output.push_str(&format!(
                " sourcepos=\"{}:{}-{}:{}\"",
                node.source_pos.start_line,
                node.source_pos.start_column,
                node.source_pos.end_line,
                node.source_pos.end_column
            ));
        }

        // Add type-specific attributes
        match &node.data {
            NodeData::List {
                list_type,
                delim,
                start,
                tight,
                ..
            } => {
                match list_type {
                    crate::node::ListType::Bullet => {
                        self.output.push_str(" type=\"bullet\"");
                    }
                    crate::node::ListType::Ordered => {
                        self.output.push_str(" type=\"ordered\"");
                        if *start != 1 {
                            self.output.push_str(&format!(" start=\"{}\"", start));
                        }
                        match delim {
                            crate::node::DelimType::Period => {
                                self.output.push_str(" delim=\"period\"");
                            }
                            crate::node::DelimType::Paren => {
                                self.output.push_str(" delim=\"paren\"");
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
                if *tight {
                    self.output.push_str(" tight=\"true\"");
                }
            }
            NodeData::Heading { level, .. } => {
                self.output.push_str(&format!(" level=\"{}\"", level));
            }
            NodeData::CodeBlock { info, .. } => {
                if !info.is_empty() {
                    self.output.push_str(" info=\"");
                    self.output.push_str(&escape_xml(info));
                    self.output.push('"');
                }
            }
            NodeData::Link { url, title } | NodeData::Image { url, title } => {
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
            match &node.data {
                NodeData::Text { literal }
                | NodeData::CodeBlock { literal, .. }
                | NodeData::Code { literal, .. }
                | NodeData::HtmlBlock { literal, .. }
                | NodeData::HtmlInline { literal, .. } => {
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
            let tag_name = self.node_type_to_tag(&node.node_type);
            self.output.push_str("</");
            self.output.push_str(tag_name);
            self.output.push_str(">\n");
        }
    }

    fn is_leaf(&self, node_id: NodeId) -> bool {
        let node = self.arena.get(node_id);
        // Document is never a leaf, even when empty
        if node.node_type == NodeType::Document {
            return false;
        }
        node.first_child.is_none()
    }

    fn node_type_to_tag(&self, node_type: &NodeType) -> &'static str {
        match node_type {
            NodeType::Document => "document",
            NodeType::BlockQuote => "block_quote",
            NodeType::List => "list",
            NodeType::Item => "item",
            NodeType::CodeBlock => "code_block",
            NodeType::HtmlBlock => "html_block",
            NodeType::CustomBlock => "custom_block",
            NodeType::Paragraph => "paragraph",
            NodeType::Heading => "heading",
            NodeType::ThematicBreak => "thematic_break",
            NodeType::Table => "table",
            NodeType::TableHead => "table_head",
            NodeType::TableRow => "table_row",
            NodeType::TableCell => "table_cell",
            NodeType::Text => "text",
            NodeType::SoftBreak => "softbreak",
            NodeType::LineBreak => "linebreak",
            NodeType::Code => "code",
            NodeType::HtmlInline => "html_inline",
            NodeType::CustomInline => "custom_inline",
            NodeType::Emph => "emph",
            NodeType::Strong => "strong",
            NodeType::Link => "link",
            NodeType::Image => "image",
            NodeType::Strikethrough => "strikethrough",
            NodeType::TaskItem => "task_item",
            NodeType::FootnoteRef => "footnote_ref",
            NodeType::FootnoteDef => "footnote_def",
            NodeType::None => "none",
        }
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
        let root = arena.alloc(Node::new(NodeType::Document));
        let xml = render(&arena, root, 0);
        println!("XML output: {:?}", xml);
        assert!(xml.contains("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(xml.contains("<document>"), "Expected <document> in {}", xml);
        assert!(xml.contains("</document>"));
    }

    #[test]
    fn test_render_paragraph() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::new(NodeType::Document));
        let para = arena.alloc(Node::new(NodeType::Paragraph));
        let text = arena.alloc(Node::with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "Hello world".to_string(),
            },
        ));

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
        let root = arena.alloc(Node::new(NodeType::Document));
        let list = arena.alloc(Node::with_data(
            NodeType::List,
            NodeData::List {
                list_type: crate::node::ListType::Bullet,
                delim: crate::node::DelimType::None,
                start: 0,
                tight: true,
                bullet_char: '-',
            },
        ));
        let item = arena.alloc(Node::new(NodeType::Item));
        let para = arena.alloc(Node::new(NodeType::Paragraph));
        let text = arena.alloc(Node::with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "Item".to_string(),
            },
        ));

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
        let root = arena.alloc(Node::new(NodeType::Document));
        let list = arena.alloc(Node::with_data(
            NodeType::List,
            NodeData::List {
                list_type: crate::node::ListType::Ordered,
                delim: crate::node::DelimType::Period,
                start: 1,
                tight: false,
                bullet_char: '\0',
            },
        ));
        let item = arena.alloc(Node::new(NodeType::Item));

        TreeOps::append_child(&mut arena, root, list);
        TreeOps::append_child(&mut arena, list, item);

        let xml = render(&arena, root, 0);
        assert!(xml.contains("<list type=\"ordered\" delim=\"period\">"));
    }

    #[test]
    fn test_render_ordered_list_with_start() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::new(NodeType::Document));
        let list = arena.alloc(Node::with_data(
            NodeType::List,
            NodeData::List {
                list_type: crate::node::ListType::Ordered,
                delim: crate::node::DelimType::Paren,
                start: 5,
                tight: true,
                bullet_char: '\0',
            },
        ));
        let item = arena.alloc(Node::new(NodeType::Item));

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
        let root = arena.alloc(Node::new(NodeType::Document));
        let heading = arena.alloc(Node::with_data(
            NodeType::Heading,
            NodeData::Heading {
                level: 2,
                content: "Title".to_string(),
            },
        ));
        let text = arena.alloc(Node::with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "Title".to_string(),
            },
        ));

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
        let root = arena.alloc(Node::new(NodeType::Document));
        let code_block = arena.alloc(Node::with_data(
            NodeType::CodeBlock,
            NodeData::CodeBlock {
                info: "rust".to_string(),
                literal: "fn main() {}".to_string(),
            },
        ));

        TreeOps::append_child(&mut arena, root, code_block);

        let xml = render(&arena, root, 0);
        assert!(xml.contains("<code_block info=\"rust\">fn main() {}</code_block>"));
    }

    #[test]
    fn test_render_link() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::new(NodeType::Document));
        let para = arena.alloc(Node::new(NodeType::Paragraph));
        let link = arena.alloc(Node::with_data(
            NodeType::Link,
            NodeData::Link {
                url: "https://example.com".to_string(),
                title: "Example".to_string(),
            },
        ));
        let text = arena.alloc(Node::with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "link".to_string(),
            },
        ));

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
        let root = arena.alloc(Node::new(NodeType::Document));
        let para = arena.alloc(Node::new(NodeType::Paragraph));
        let image = arena.alloc(Node::with_data(
            NodeType::Image,
            NodeData::Image {
                url: "image.png".to_string(),
                title: "Alt text".to_string(),
            },
        ));
        let text = arena.alloc(Node::with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "alt".to_string(),
            },
        ));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, image);
        TreeOps::append_child(&mut arena, image, text);

        let xml = render(&arena, root, 0);
        assert!(xml.contains("<image destination=\"image.png\" title=\"Alt text\">"));
    }

    #[test]
    fn test_render_with_sourcepos() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::new(NodeType::Document));
        let para = arena.alloc(Node::new(NodeType::Paragraph));
        {
            let p = arena.get_mut(para);
            p.source_pos.start_line = 1;
            p.source_pos.start_column = 1;
            p.source_pos.end_line = 1;
            p.source_pos.end_column = 10;
        }

        TreeOps::append_child(&mut arena, root, para);

        let xml = render(&arena, root, crate::options::SOURCEPOS);
        assert!(xml.contains("sourcepos=\"1:1-1:10\""));
    }

    #[test]
    fn test_render_blockquote() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::new(NodeType::Document));
        let blockquote = arena.alloc(Node::new(NodeType::BlockQuote));
        let para = arena.alloc(Node::new(NodeType::Paragraph));
        let text = arena.alloc(Node::with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "Quote".to_string(),
            },
        ));

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
        let root = arena.alloc(Node::new(NodeType::Document));
        let para = arena.alloc(Node::new(NodeType::Paragraph));
        let emph = arena.alloc(Node::new(NodeType::Emph));
        let text = arena.alloc(Node::with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "emphasized".to_string(),
            },
        ));

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
        let root = arena.alloc(Node::new(NodeType::Document));
        let para = arena.alloc(Node::new(NodeType::Paragraph));
        let strong = arena.alloc(Node::new(NodeType::Strong));
        let text = arena.alloc(Node::with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "strong".to_string(),
            },
        ));

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
        let root = arena.alloc(Node::new(NodeType::Document));
        let para = arena.alloc(Node::new(NodeType::Paragraph));
        let code = arena.alloc(Node::with_data(
            NodeType::Code,
            NodeData::Code {
                literal: "code".to_string(),
            },
        ));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, code);

        let xml = render(&arena, root, 0);
        assert!(xml.contains("<code>code</code>"));
    }

    #[test]
    fn test_render_thematic_break() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::new(NodeType::Document));
        let hr = arena.alloc(Node::new(NodeType::ThematicBreak));

        TreeOps::append_child(&mut arena, root, hr);

        let xml = render(&arena, root, 0);
        assert!(xml.contains("<thematic_break />"));
    }

    #[test]
    fn test_render_html_block() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::new(NodeType::Document));
        let html_block = arena.alloc(Node::with_data(
            NodeType::HtmlBlock,
            NodeData::HtmlBlock {
                literal: "<div>content</div>".to_string(),
            },
        ));

        TreeOps::append_child(&mut arena, root, html_block);

        let xml = render(&arena, root, 0);
        assert!(xml.contains("<html_block>&lt;div&gt;content&lt;/div&gt;</html_block>"));
    }

    #[test]
    fn test_render_soft_break() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::new(NodeType::Document));
        let para = arena.alloc(Node::new(NodeType::Paragraph));
        let soft_break = arena.alloc(Node::new(NodeType::SoftBreak));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, soft_break);

        let xml = render(&arena, root, 0);
        assert!(xml.contains("<softbreak />"));
    }

    #[test]
    fn test_render_line_break() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::new(NodeType::Document));
        let para = arena.alloc(Node::new(NodeType::Paragraph));
        let line_break = arena.alloc(Node::new(NodeType::LineBreak));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, line_break);

        let xml = render(&arena, root, 0);
        assert!(xml.contains("<linebreak />"));
    }

    #[test]
    fn test_render_empty_text() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::new(NodeType::Document));
        let para = arena.alloc(Node::new(NodeType::Paragraph));
        let text = arena.alloc(Node::with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "".to_string(),
            },
        ));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, text);

        let xml = render(&arena, root, 0);
        assert!(xml.contains("<text />"));
    }
}
