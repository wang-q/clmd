//! XML renderer (deprecated)
//!
//! ⚠️ **DEPRECATED**: This module is deprecated. Use Arena-based rendering instead.
//!
//! This module uses the old Rc<RefCell>-based AST. It will be removed in a future version.

use crate::iterator::{NodeWalker, WalkerEvent};
use crate::node::{Node, NodeData, NodeType};
use std::cell::RefCell;
use std::rc::Rc;

/// Render a node tree as XML
pub fn render(root: &Rc<RefCell<Node>>, options: u32) -> String {
    let mut renderer = XmlRenderer::new(options);
    renderer.render(root)
}

struct XmlRenderer {
    options: u32,
    output: String,
}

impl XmlRenderer {
    fn new(options: u32) -> Self {
        XmlRenderer {
            options,
            output: String::new(),
        }
    }

    fn render(&mut self, root: &Rc<RefCell<Node>>) -> String {
        self.output
            .push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        self.output
            .push_str("<!DOCTYPE document SYSTEM \"CommonMark.dtd\">\n");

        let mut walker = NodeWalker::new(root.clone());

        while let Some(event) = walker.next() {
            if event.entering {
                self.enter_node(&event);
            } else {
                self.exit_node(&event);
            }
        }

        self.output.clone()
    }

    fn enter_node(&mut self, event: &WalkerEvent) {
        let node = event.node.borrow();
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
        if node.is_leaf() {
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

    fn exit_node(&mut self, event: &WalkerEvent) {
        let node = event.node.borrow();

        // Leaf nodes are already closed in enter_node
        if !node.is_leaf() {
            let tag_name = self.node_type_to_tag(&node.node_type);
            self.output.push_str("</");
            self.output.push_str(tag_name);
            self.output.push_str(">\n");
        }
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
    use crate::node::{append_child, Node, NodeData, NodeType};

    #[test]
    fn test_escape_xml() {
        assert_eq!(escape_xml("<div>"), "&lt;div&gt;");
        assert_eq!(escape_xml("&"), "&amp;");
        assert_eq!(escape_xml("'test'"), "&apos;test&apos;");
    }

    #[test]
    fn test_render_document() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let xml = render(&root, 0);
        assert!(xml.contains("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(xml.contains("<document>"));
        assert!(xml.contains("</document>"));
    }

    #[test]
    fn test_render_paragraph() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let para = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        let text = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "Hello world".to_string(),
            },
        )));

        append_child(&root, para.clone());
        append_child(&para, text.clone());

        let xml = render(&root, 0);
        assert!(xml.contains("<paragraph>"));
        assert!(xml.contains("<text>Hello world</text>"));
        assert!(xml.contains("</paragraph>"));
    }

    #[test]
    fn test_render_bullet_list() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let list = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::List,
            NodeData::List {
                list_type: crate::node::ListType::Bullet,
                delim: crate::node::DelimType::None,
                start: 0,
                tight: true,
                bullet_char: '-',
            },
        )));
        let item = Rc::new(RefCell::new(Node::new(NodeType::Item)));
        let para = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        let text = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "Item".to_string(),
            },
        )));

        append_child(&root, list.clone());
        append_child(&list, item.clone());
        append_child(&item, para.clone());
        append_child(&para, text.clone());

        let xml = render(&root, 0);
        assert!(xml.contains("<list type=\"bullet\" tight=\"true\">"));
        assert!(xml.contains("<item>"));
        assert!(xml.contains("<text>Item</text>"));
    }

    #[test]
    fn test_render_ordered_list() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let list = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::List,
            NodeData::List {
                list_type: crate::node::ListType::Ordered,
                delim: crate::node::DelimType::Period,
                start: 1,
                tight: false,
                bullet_char: '\0',
            },
        )));
        let item = Rc::new(RefCell::new(Node::new(NodeType::Item)));

        append_child(&root, list.clone());
        append_child(&list, item.clone());

        let xml = render(&root, 0);
        assert!(xml.contains("<list type=\"ordered\" delim=\"period\">"));
    }

    #[test]
    fn test_render_ordered_list_with_start() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let list = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::List,
            NodeData::List {
                list_type: crate::node::ListType::Ordered,
                delim: crate::node::DelimType::Paren,
                start: 5,
                tight: true,
                bullet_char: '\0',
            },
        )));
        let item = Rc::new(RefCell::new(Node::new(NodeType::Item)));

        append_child(&root, list.clone());
        append_child(&list, item.clone());

        let xml = render(&root, 0);
        assert!(xml.contains(
            "<list type=\"ordered\" start=\"5\" delim=\"paren\" tight=\"true\">"
        ));
    }

    #[test]
    fn test_render_heading() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let heading = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Heading,
            NodeData::Heading {
                level: 2,
                content: "Title".to_string(),
            },
        )));

        append_child(&root, heading.clone());

        let xml = render(&root, 0);
        assert!(xml.contains("<heading level=\"2\">"));
    }

    #[test]
    fn test_render_code_block() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let code_block = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::CodeBlock,
            NodeData::CodeBlock {
                info: "rust".to_string(),
                literal: "fn main() {}".to_string(),
            },
        )));

        append_child(&root, code_block.clone());

        let xml = render(&root, 0);
        assert!(xml.contains("<code_block info=\"rust\">fn main() {}</code_block>"));
    }

    #[test]
    fn test_render_link() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let para = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        let link = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Link,
            NodeData::Link {
                url: "https://example.com".to_string(),
                title: "Example".to_string(),
            },
        )));
        let text = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "link".to_string(),
            },
        )));

        append_child(&root, para.clone());
        append_child(&para, link.clone());
        append_child(&link, text.clone());

        let xml = render(&root, 0);
        assert!(
            xml.contains("<link destination=\"https://example.com\" title=\"Example\">")
        );
        assert!(xml.contains("<text>link</text>"));
    }

    #[test]
    fn test_render_image() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let para = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        let image = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Image,
            NodeData::Image {
                url: "image.png".to_string(),
                title: "Alt text".to_string(),
            },
        )));
        let text = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "alt".to_string(),
            },
        )));

        append_child(&root, para.clone());
        append_child(&para, image.clone());
        append_child(&image, text.clone());

        let xml = render(&root, 0);
        assert!(xml.contains("<image destination=\"image.png\" title=\"Alt text\">"));
    }

    #[test]
    fn test_render_with_sourcepos() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let para = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        {
            let mut para_mut = para.borrow_mut();
            para_mut.source_pos.start_line = 1;
            para_mut.source_pos.start_column = 1;
            para_mut.source_pos.end_line = 1;
            para_mut.source_pos.end_column = 10;
        }

        append_child(&root, para.clone());

        let xml = render(&root, crate::options::SOURCEPOS);
        assert!(xml.contains("sourcepos=\"1:1-1:10\""));
    }

    #[test]
    fn test_render_blockquote() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let blockquote = Rc::new(RefCell::new(Node::new(NodeType::BlockQuote)));
        let para = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        let text = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "Quote".to_string(),
            },
        )));

        append_child(&root, blockquote.clone());
        append_child(&blockquote, para.clone());
        append_child(&para, text.clone());

        let xml = render(&root, 0);
        assert!(xml.contains("<block_quote>"));
        assert!(xml.contains("<text>Quote</text>"));
        assert!(xml.contains("</block_quote>"));
    }

    #[test]
    fn test_render_emph() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let para = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        let emph = Rc::new(RefCell::new(Node::new(NodeType::Emph)));
        let text = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "emphasized".to_string(),
            },
        )));

        append_child(&root, para.clone());
        append_child(&para, emph.clone());
        append_child(&emph, text.clone());

        let xml = render(&root, 0);
        assert!(xml.contains("<emph>"));
        assert!(xml.contains("<text>emphasized</text>"));
        assert!(xml.contains("</emph>"));
    }

    #[test]
    fn test_render_strong() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let para = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        let strong = Rc::new(RefCell::new(Node::new(NodeType::Strong)));
        let text = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "strong".to_string(),
            },
        )));

        append_child(&root, para.clone());
        append_child(&para, strong.clone());
        append_child(&strong, text.clone());

        let xml = render(&root, 0);
        assert!(xml.contains("<strong>"));
        assert!(xml.contains("<text>strong</text>"));
        assert!(xml.contains("</strong>"));
    }

    #[test]
    fn test_render_code() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let para = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        let code = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Code,
            NodeData::Code {
                literal: "code".to_string(),
            },
        )));

        append_child(&root, para.clone());
        append_child(&para, code.clone());

        let xml = render(&root, 0);
        assert!(xml.contains("<code>code</code>"));
    }

    #[test]
    fn test_render_thematic_break() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let hr = Rc::new(RefCell::new(Node::new(NodeType::ThematicBreak)));

        append_child(&root, hr.clone());

        let xml = render(&root, 0);
        assert!(xml.contains("<thematic_break />"));
    }

    #[test]
    fn test_render_html_block() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let html_block = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::HtmlBlock,
            NodeData::HtmlBlock {
                literal: "<div>content</div>".to_string(),
            },
        )));

        append_child(&root, html_block.clone());

        let xml = render(&root, 0);
        assert!(xml.contains("<html_block>&lt;div&gt;content&lt;/div&gt;</html_block>"));
    }

    #[test]
    fn test_render_soft_break() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let para = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        let soft_break = Rc::new(RefCell::new(Node::new(NodeType::SoftBreak)));

        append_child(&root, para.clone());
        append_child(&para, soft_break.clone());

        let xml = render(&root, 0);
        assert!(xml.contains("<softbreak />"));
    }

    #[test]
    fn test_render_line_break() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let para = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        let line_break = Rc::new(RefCell::new(Node::new(NodeType::LineBreak)));

        append_child(&root, para.clone());
        append_child(&para, line_break.clone());

        let xml = render(&root, 0);
        assert!(xml.contains("<linebreak />"));
    }

    #[test]
    fn test_render_empty_text() {
        let root = Rc::new(RefCell::new(Node::new(NodeType::Document)));
        let para = Rc::new(RefCell::new(Node::new(NodeType::Paragraph)));
        let text = Rc::new(RefCell::new(Node::new_with_data(
            NodeType::Text,
            NodeData::Text {
                literal: "".to_string(),
            },
        )));

        append_child(&root, para.clone());
        append_child(&para, text.clone());

        let xml = render(&root, 0);
        assert!(xml.contains("<text />"));
    }
}
