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
            NodeData::Heading { level } => {
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
}
